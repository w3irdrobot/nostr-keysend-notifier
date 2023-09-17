use anyhow::Result;
use config::Config;
use nostr_keysend_notifier::run;
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {
    pub lnd: LndSettings,
    pub nostr: NostrSettings,
}

#[derive(Deserialize)]
struct LndSettings {
    pub host: String,
    pub cert_path: String,
    pub mac_path: String,
}

#[derive(Deserialize)]
struct NostrSettings {
    pub receiver_pubkey: String,
    pub relays: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let Settings { lnd, nostr } = Config::builder()
        .add_source(config::File::with_name("config"))
        .add_source(config::Environment::with_prefix("NKN"))
        .build()
        .unwrap()
        .try_deserialize::<Settings>()
        .unwrap();

    run(
        lnd.host,
        lnd.cert_path,
        lnd.mac_path,
        nostr.receiver_pubkey,
        nostr.relays,
    )
    .await
}
