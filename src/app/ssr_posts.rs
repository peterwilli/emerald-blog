use libnostrblog::{
    Blog,
    blog::{fetch_authors::FetchAuthorsExt, fetch_posts::FetchPostExt},
    objects::post::Post,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{
    env, fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tracing::{debug, info};

use super::client::{OWNER_PUBKEY, get_nostr_client};

pub type SyncedPostList = Arc<RwLock<Vec<Post<'static>>>>;
pub static POSTS_LIST: Lazy<SyncedPostList> = Lazy::new(Default::default);

pub async fn fetch_posts() {
    if load_cached_posts() {
        return;
    }

    let client = get_nostr_client().await;
    let blog = Blog::new(client.clone(), vec![OWNER_PUBKEY]);
    blog.fetch_authors().await.unwrap();
    let posts = blog.fetch_posts().await.unwrap();
    debug!("fetched posts: {:?}", posts);
    *POSTS_LIST.write() = posts;

    write_cached_posts();
}

fn load_cached_posts() -> bool {
    let Some(ttl) = posts_cache_ttl() else {
        return false;
    };

    let path = posts_cache_path();
    let Ok(metadata) = fs::metadata(&path) else {
        return false;
    };

    let is_fresh = metadata
        .modified()
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age <= ttl);

    if !is_fresh {
        return false;
    }

    match fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str::<Vec<Post<'static>>>(&json).ok())
    {
        Some(posts) => {
            let post_count = posts.len();
            *POSTS_LIST.write() = posts;
            info!(
                "Loaded {post_count} cached posts from {}",
                path.to_string_lossy()
            );
            true
        }
        None => false,
    }
}

fn write_cached_posts() {
    if posts_cache_ttl().is_none() {
        return;
    }

    let path = posts_cache_path();
    let posts = POSTS_LIST.read().clone();
    let Ok(json) = serde_json::to_string(&posts) else {
        return;
    };

    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    if fs::write(&path, json).is_ok() {
        info!("Cached {} posts at {}", posts.len(), path.to_string_lossy());
    }
}

fn posts_cache_ttl() -> Option<Duration> {
    let enabled_by_default = cfg!(debug_assertions);
    let raw_ttl = env::var("EMERALD_POSTS_CACHE_TTL_SECS").ok();

    if !enabled_by_default && raw_ttl.is_none() {
        return None;
    }

    let ttl_secs = raw_ttl
        .as_deref()
        .map(|ttl| ttl.parse::<u64>().unwrap())
        .unwrap_or(86400);

    (ttl_secs > 0).then(|| Duration::from_secs(ttl_secs))
}

fn posts_cache_path() -> PathBuf {
    env::var_os("EMERALD_POSTS_CACHE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("emerald-blog-v3").join("posts.json"))
}
