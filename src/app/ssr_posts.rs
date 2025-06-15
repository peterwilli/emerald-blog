use libnostrblog::{
    Blog,
    blog::{fetch_authors::FetchAuthorsExt, poll_posts::PollPostsExt},
    objects::post::Post,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::sync::Arc;

use super::client::{OWNER_PUBKEY, get_nostr_client};

pub type SyncedPostList = Arc<RwLock<Vec<Post<'static>>>>;
pub static POSTS_LIST: Lazy<SyncedPostList> = Lazy::new(Default::default);

pub async fn poll_posts() {
    let client = get_nostr_client().await;
    let blog = Blog::new(client.clone(), vec![*OWNER_PUBKEY]);
    blog.fetch_authors().await.unwrap();
    let mut rx = blog.poll_posts(None).await.unwrap();
    while let Some(post) = rx.recv().await {
        println!("Added post: {:?}", post);
        POSTS_LIST.write().push(post);
    }
}
