use anyhow::{anyhow, Result};
use log::{debug, info};
use nostr_sdk::prelude::*;
use std::sync::Arc;

pub type NostrPool = Arc<Nostr>;

pub struct Nostr {
    client: Client,
    receiver: XOnlyPublicKey,
}

impl Nostr {
    pub async fn new(
        sender_privkey: &str,
        receiver_pubkey: &str,
        relays: Vec<String>,
    ) -> Result<NostrPool> {
        let keys = Keys::from_sk_str(sender_privkey)?;
        info!("service pubkey: {}", keys.public_key().to_bech32()?);
        let client = Client::new(&keys);
        for url in relays {
            client
                .add_relay(url.clone(), None)
                .await
                .unwrap_or_else(|_| panic!("{} connects", url));
        }
        client.connect().await;
        debug!("nostr client connected to relays");

        let receiver = Keys::from_pk_str(receiver_pubkey)?.public_key();
        info!("receiver pubkey: {}", receiver.to_bech32()?);

        Ok(Arc::new(Self { client, receiver }))
    }

    pub async fn send_message(&self, msg: &str) -> Result<()> {
        self.client
            .send_direct_msg(self.receiver, msg, None)
            .await
            .map(|_| ())
            .map_err(|e| anyhow!(e))
    }
}
