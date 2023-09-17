mod lnd;
mod nostr;

use anyhow::Result;
use nostr_sdk::prelude::*;
use tokio::fs;

pub async fn run(
    address: String,
    cert_file: String,
    mac_file: String,
    receiver_pubkey: String,
    relays: Vec<String>,
) -> Result<()> {
    let key = if let Ok(secret_key) = fs::read_to_string(".privkey").await {
        secret_key
    } else {
        let keys = Keys::generate();
        let privkey = keys.secret_key().unwrap().to_bech32()?;
        fs::write(".privkey", &privkey).await?;
        privkey
    };

    let nostr_client = nostr::Nostr::new(&key, &receiver_pubkey, relays).await?;
    let mut lnd_client = lnd::Lnd::new(address, cert_file, mac_file).await?;

    lnd_client.subscribe_and_notify(nostr_client).await?;

    Ok(())
}
