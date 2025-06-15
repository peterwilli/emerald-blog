use libnostrblog::{Client, FromBech32, Keys, Options, PublicKey};
use once_cell::sync::Lazy;
use tokio::sync::OnceCell;

pub static OWNER_PUBKEY: Lazy<PublicKey> = Lazy::new(|| {
    PublicKey::from_bech32("npub17rfmhcfv2f078kg0jrx4efw7pcj5gxfc3j49syvtf4vn0m58ta5sg00n88")
        .unwrap()
});
static NOSTRL_CLIENT: OnceCell<Client> = OnceCell::const_new();

pub async fn get_nostr_client() -> &'static Client {
    NOSTRL_CLIENT
        .get_or_init(|| async {
            let keys = Keys::generate();

            let client = Client::builder()
                .opts(Options::new().gossip(true))
                .signer(keys)
                .build();

            client.add_relay("wss://nos.lol").await.unwrap();
            client.add_relay("wss://relay.nostr.band").await.unwrap();

            client.connect().await;

            client
        })
        .await
}
