use hex_literal::hex;
use libnostrblog::{Client, Keys, Options, PublicKey, ToBech32};
use tokio::sync::OnceCell;
use tracing::info;

pub static OWNER_PUBKEY: PublicKey = PublicKey::from_byte_array(hex!(
    "5ea6f3b26df9d4ddd0ea26391cf6a5f644283a05e8251e95dbce5aef1d4cc228"
));
static NOSTRL_CLIENT: OnceCell<Client> = OnceCell::const_new();
static ANONYMOUS_COMMENT_CLIENT: OnceCell<Client> = OnceCell::const_new();
pub const NOSTR_RELAYS: &[&str] = &["wss://nos.lol", "wss://nostr.emerald.show"];
const ANONYMOUS_COMMENT_RELAYS: &[&str] = &["wss://nos.lol", "wss://relay.nostr.band"];

pub async fn get_nostr_client() -> &'static Client {
    NOSTRL_CLIENT
        .get_or_init(|| async {
            let keys = Keys::generate();

            let client = Client::builder()
                .opts(Options::new().gossip(true))
                .signer(keys)
                .build();

            for relay in NOSTR_RELAYS {
                client.add_relay(*relay).await.unwrap();
            }

            client.connect().await;

            client
        })
        .await
}

pub async fn get_anonymous_comment_client() -> &'static Client {
    ANONYMOUS_COMMENT_CLIENT
        .get_or_init(|| async {
            let keys = Keys::generate();
            info!(
                "Created anonymous comment account: {}",
                keys.public_key()
                    .to_bech32()
                    .unwrap_or_else(|_| keys.public_key().to_hex())
            );

            let client = Client::builder()
                .opts(Options::new().gossip(true))
                .signer(keys)
                .build();

            for relay in ANONYMOUS_COMMENT_RELAYS {
                client.add_relay(*relay).await.unwrap();
            }

            client.connect().await;

            client
        })
        .await
}
