use hex_literal::hex;
use libnostrblog::{Client, Keys, Options, PublicKey};
use tokio::sync::OnceCell;

pub static OWNER_PUBKEY: PublicKey = PublicKey::from_byte_array(hex!(
    "5ea6f3b26df9d4ddd0ea26391cf6a5f644283a05e8251e95dbce5aef1d4cc228"
));
static NOSTRL_CLIENT: OnceCell<Client> = OnceCell::const_new();
pub const NOSTR_RELAYS: &[&str] = &["wss://nos.lol", "wss://nostr.emerald.show"];

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
