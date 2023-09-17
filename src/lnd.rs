use crate::nostr::NostrPool;
use ::time::{format_description::well_known, OffsetDateTime};
use anyhow::{Ok, Result};
use futures::{future::ready, StreamExt};
use log::{debug, error, info};
use regex::Regex;
use reqwest::Url;
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::HashMap,
    str,
    sync::{Arc, Mutex},
};
use tonic_lnd::lnrpc::invoice::InvoiceState;
use tonic_lnd::lnrpc::{InvoiceHtlc, InvoiceSubscription};
use tonic_lnd::Client;

const MESSAGE_TLV: u64 = 34349334;
const PUBKEY_TLV: u64 = 34349339;

pub struct Lnd {
    client: Client,
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl Lnd {
    pub async fn new(address: String, cert_file: String, mac_file: String) -> Result<Self> {
        let client = tonic_lnd::connect(address, cert_file, mac_file).await?;
        let cache = Arc::new(Mutex::new(HashMap::<String, String>::new()));

        Ok(Self { client, cache })
    }

    pub async fn subscribe_and_notify(&mut self, nostr_client: NostrPool) -> Result<()> {
        self.client
            .lightning()
            .subscribe_invoices(InvoiceSubscription {
                ..Default::default()
            })
            .await?
            .into_inner()
            .filter_map(|i| async { i.ok() })
            .filter(|i| ready(InvoiceState::from_i32(i.state).unwrap() == InvoiceState::Settled))
            .filter_map(|i| async {
                i.htlcs
                    .into_iter()
                    .find(|h| h.custom_records.contains_key(&MESSAGE_TLV))
            })
            .inspect(|h| debug!("htlc chat found: {:?}", h))
            .then(|h| {
                let cache = self.cache.clone();
                async move { make_htlc_message(h, cache).await }
            })
            .inspect(|msg| info!("message to send: {:?}", msg))
            .filter_map(|i| async { i.ok() })
            .for_each(|msg| {
                let nostr_client = nostr_client.clone();
                async move {
                    if let Err(e) = nostr_client.send_message(&msg).await {
                        error!("message send failed: {:?}", e)
                    }
                }
            })
            .await;

        Ok(())
    }
}

async fn make_htlc_message(
    htlc: InvoiceHtlc,
    cache: Arc<Mutex<HashMap<String, String>>>,
) -> Result<String> {
    let data = htlc.custom_records.get(&MESSAGE_TLV).unwrap();
    let keysend_msg: &str = str::from_utf8(data).unwrap();
    let resolved_at =
        OffsetDateTime::from_unix_timestamp(htlc.resolve_time)?.format(&well_known::Rfc3339)?;

    let re = Regex::new(r"^.*\s+([0-9a-z]{66}).*$").unwrap();
    let pubkey = if let Some(pubkey) = htlc.custom_records.get(&PUBKEY_TLV) {
        let pubkey = ::hex::encode(pubkey);
        debug!("pubkey from tlv: {:?}", pubkey);
        Some(pubkey)
    } else if let Some(caps) = re.captures(keysend_msg) {
        // assume it's a pubkey
        debug!("pubkey from message: {:?}", caps[1].to_string());
        Some(caps[1].to_string())
    } else {
        None
    };

    let msg = format!(
        "Keysend message received!\n\nAt {}:\n\n**{}**",
        resolved_at, keysend_msg
    );
    let msg = if let Some(pubkey) = pubkey {
        let name = get_node_name(&pubkey, cache.clone())
            .await
            .unwrap_or(pubkey.clone());
        format!(
            "{}\n\nFrom node [{}](https://amboss.space/node/{})",
            msg, name, pubkey
        )
    } else {
        msg
    };

    Ok(msg)
}

#[derive(Debug, Deserialize)]
struct AmbossResponse {
    data: AmbossData,
}

#[derive(Debug, Deserialize)]
struct AmbossData {
    #[serde(rename(deserialize = "getNodeAlias"))]
    get_node_alias: String,
}

async fn get_node_name(pubkey: &str, cache: Arc<Mutex<HashMap<String, String>>>) -> Result<String> {
    if let Some(name) = cache.lock().unwrap().get(pubkey) {
        debug!("alias found in cache: {}", name);
        return Ok(name.clone());
    }

    debug!("alias not found in cache. querying amboss");
    let client = reqwest::Client::new();
    let url = Url::parse("https://api.amboss.space/graphql").unwrap();
    let body = json!({
        "query": format!("query{{getNodeAlias(pubkey:\"{}\")}}", pubkey),
    });
    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await?
        .json::<AmbossResponse>()
        .await?;
    let alias = resp.data.get_node_alias;
    debug!("alias found in amboss: {}", alias);

    cache
        .lock()
        .unwrap()
        .insert(pubkey.to_string(), alias.clone());
    debug!("alias saved in cache: {}", alias);

    Ok(alias)
}
