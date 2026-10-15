use hex_literal::hex;
use libnostrblog::{Client, Keys, Options, PublicKey};
use tokio::sync::OnceCell;

pub static OWNER_PUBKEY: PublicKey = PublicKey::from_byte_array(hex!(
    "cef23efdb0853ad6f854f1cb39a7bd62e031940e77429015fe9f67aa2fe74227"
));
static NOSTRL_CLIENT: OnceCell<Client> = OnceCell::const_new();
pub const NOSTR_RELAYS: &[&str] = &[
    "wss://nos.lol",
    "wss://nostr.emerald.show",
    "wss://relay.damus.io",
    "wss://relay.nostr.band",
];

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
