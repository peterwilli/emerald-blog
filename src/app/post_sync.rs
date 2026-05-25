use leptos::prelude::*;
use libnostrblog::objects::post::Post;

use super::AppState;

#[cfg(any(
    feature = "ssr",
    all(feature = "hydrate", feature = "client-side-nostr-fetch")
))]
use super::client::{OWNER_PUBKEY, get_nostr_client};
#[cfg(feature = "ssr")]
use libnostrblog::blog::fetch_posts::FetchPostExt;
#[cfg(all(feature = "hydrate", feature = "client-side-nostr-fetch"))]
use libnostrblog::blog::stream_posts::StreamPostsExt;
#[cfg(any(
    feature = "ssr",
    all(feature = "hydrate", feature = "client-side-nostr-fetch")
))]
use libnostrblog::{Blog, Timestamp, blog::fetch_authors::FetchAuthorsExt};

#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;
#[cfg(feature = "ssr")]
use parking_lot::RwLock;
#[cfg(feature = "ssr")]
use std::{
    env, fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, SystemTime},
};
#[cfg(feature = "ssr")]
use tokio::time;
#[cfg(feature = "ssr")]
use tracing::info;
#[cfg(any(
    feature = "ssr",
    all(feature = "hydrate", feature = "client-side-nostr-fetch")
))]
use tracing::{debug, warn};

#[cfg(feature = "ssr")]
pub type SyncedPostList = Arc<RwLock<Vec<Post<'static>>>>;
#[cfg(feature = "ssr")]
pub static POSTS_LIST: Lazy<SyncedPostList> = Lazy::new(Default::default);
#[cfg(feature = "ssr")]
static SERVER_POST_SYNC_STARTED: AtomicBool = AtomicBool::new(false);

#[server]
pub async fn seed_posts() -> Result<Vec<Post<'static>>, ServerFnError> {
    Ok(POSTS_LIST.read().clone())
}

#[component]
pub fn PostsSync() -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts.clone();

    Effect::new(move |_| {
        if let Some(posts) = posts.get() {
            state.merge_posts(posts);
            #[cfg(all(feature = "hydrate", feature = "client-side-nostr-fetch"))]
            start_client_post_sync(state.clone());
        }
    });

    ().into_view()
}

#[cfg(feature = "ssr")]
pub async fn start_server_post_sync() {
    if SERVER_POST_SYNC_STARTED.swap(true, Ordering::Relaxed) {
        return;
    }

    let blog = blog_with_authors("loading posts").await;
    if let Some(blog) = blog.as_ref() {
        if !fetch_and_replace_posts(blog).await {
            load_cached_posts();
        }
    } else {
        load_cached_posts();
    }

    start_server_post_poll(blog);
}

#[cfg(feature = "ssr")]
fn start_server_post_poll(blog: Option<Blog<'static>>) {
    let cooldown_secs = 60;
    let cooldown = Duration::from_secs(cooldown_secs);

    tokio::task::spawn_local(async move {
        let mut blog = blog;

        loop {
            time::sleep(cooldown).await;

            if blog.is_none() {
                blog = blog_with_authors("refreshing posts").await;
            }

            let Some(active_blog) = blog.as_ref() else {
                continue;
            };

            fetch_newer_posts(active_blog).await;
        }
    });
}

#[cfg(feature = "ssr")]
async fn fetch_and_replace_posts(blog: &Blog<'_>) -> bool {
    let posts = match blog.fetch_posts(None).await {
        Ok(posts) => posts,
        Err(error) => {
            warn!("failed to fetch posts: {error:?}");
            return false;
        }
    };

    debug!("fetched posts: {:?}", posts);
    replace_posts(posts);

    write_cached_posts();
    true
}

#[cfg(feature = "ssr")]
async fn fetch_newer_posts(blog: &Blog<'_>) {
    let from = latest_post_timestamp();
    let posts = match blog.fetch_posts(from).await {
        Ok(posts) => posts,
        Err(error) => {
            warn!("failed to fetch newer posts: {error:?}");
            return;
        }
    };

    if merge_posts(posts) {
        write_cached_posts();
    }
}

#[cfg(any(
    feature = "ssr",
    all(feature = "hydrate", feature = "client-side-nostr-fetch")
))]
async fn blog_with_authors(action: &str) -> Option<Blog<'static>> {
    let client = get_nostr_client().await;
    let blog = Blog::new(client.clone(), vec![OWNER_PUBKEY]);

    if let Err(error) = blog.fetch_authors().await {
        warn!("failed to fetch authors before {action}: {error:?}");
        return None;
    }

    Some(blog)
}

#[cfg(feature = "ssr")]
fn replace_posts(mut posts: Vec<Post<'static>>) {
    posts.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    *POSTS_LIST.write() = posts;
}

#[cfg(feature = "ssr")]
fn merge_posts(incoming_posts: Vec<Post<'static>>) -> bool {
    let mut posts = POSTS_LIST.write();
    let mut changed = false;

    for post in incoming_posts {
        if posts.iter().any(|existing| existing.id == post.id) {
            continue;
        }

        posts.push(post);
        changed = true;
    }

    if changed {
        posts.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    }

    changed
}

#[cfg(feature = "ssr")]
fn latest_post_timestamp() -> Option<Timestamp> {
    POSTS_LIST.read().iter().map(|post| post.created_at).max()
}

#[cfg(all(feature = "hydrate", feature = "client-side-nostr-fetch"))]
fn start_client_post_sync(state: AppState) {
    if state.client_post_sync_started.get_untracked() {
        return;
    }
    state.client_post_sync_started.set(true);

    let state = state.clone();
    leptos::task::spawn_local(async move {
        let cooldown = std::time::Duration::from_secs(10);

        loop {
            stream_newer_client_posts(state.clone()).await;
            gloo_timers::future::sleep(cooldown).await;
        }
    });
}

#[cfg(all(feature = "hydrate", feature = "client-side-nostr-fetch"))]
async fn stream_newer_client_posts(state: AppState) {
    let from = state
        .live_posts
        .with_untracked(latest_client_post_timestamp);
    let Some(blog) = blog_with_authors("streaming posts").await else {
        return;
    };

    let mut rx = match blog.stream_posts(from).await {
        Ok(rx) => rx,
        Err(error) => {
            warn!("failed to stream posts: {error:?}");
            return;
        }
    };

    while let Some(post) = rx.recv().await {
        debug!("client stream added post: {:?}", post);
        state.merge_posts(vec![post]);
    }
}

#[cfg(all(feature = "hydrate", feature = "client-side-nostr-fetch"))]
fn latest_client_post_timestamp(posts: &Vec<Post<'static>>) -> Option<Timestamp> {
    posts.iter().map(|post| post.created_at).max()
}

#[cfg(feature = "ssr")]
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
            replace_posts(posts);
            info!(
                "Loaded {post_count} cached posts from {}",
                path.to_string_lossy()
            );
            true
        }
        None => false,
    }
}

#[cfg(feature = "ssr")]
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

#[cfg(feature = "ssr")]
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

#[cfg(feature = "ssr")]
fn posts_cache_path() -> PathBuf {
    env::var_os("EMERALD_POSTS_CACHE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("emerald-blog-v3").join("posts.json"))
}
