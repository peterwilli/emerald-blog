use leptos::prelude::*;
use libnostrblog::objects::post::Post;

use super::AppState;

cfg_if::cfg_if! {
    if #[cfg(any(
        feature = "ssr",
        all(
            feature = "hydrate",
            feature = "client-side-nostr-fetch",
            not(feature = "islands")
        )
    ))] {
        use super::client::{get_nostr_client, OWNER_PUBKEY};
        use libnostrblog::{Timestamp, Filter, Kind, Metadata, objects::post::Author};
        use tracing::warn;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use libnostrblog::{
            blog::extensions::to_posts::ToPosts, types::Authors, Event,
        };
        use once_cell::sync::Lazy;
        use parking_lot::RwLock;
        use std::{
            env, fs,
            path::PathBuf,
            sync::{
                atomic::{AtomicBool, Ordering},
                Arc,
            },
            time::{Duration, SystemTime},
        };
        use tokio::time;
        use tracing::info;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(
        feature = "hydrate",
        feature = "client-side-nostr-fetch",
        not(feature = "islands")
    ))] {
        use libnostrblog::{
            Blog, blog::{fetch_authors::FetchAuthorsExt, stream_posts::StreamPostsExt},
        };
        use tracing::debug;
    }
}

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
        if let Some(posts) = posts.try_get().flatten() {
            state.merge_posts(posts);
            cfg_if::cfg_if! {
                if #[cfg(all(
                    feature = "hydrate",
                    feature = "client-side-nostr-fetch",
                    not(feature = "islands")
                ))] {
                    start_client_post_sync(state.clone());
                }
            }
        }
    });

    ().into_view()
}

#[cfg(feature = "ssr")]
pub async fn start_server_post_sync() {
    if SERVER_POST_SYNC_STARTED.swap(true, Ordering::Relaxed) {
        return;
    }

    let has_cached_posts = tokio::task::spawn_blocking(load_cached_posts)
        .await
        .unwrap_or(false);
    if has_cached_posts {
        tokio::task::spawn_local(async {
            refresh_server_posts_once().await;
            start_server_post_poll();
        });
    } else {
        refresh_server_posts_once().await;
        start_server_post_poll();
    }
}

#[cfg(feature = "ssr")]
pub async fn load_server_posts_once() {
    let _ = tokio::task::spawn_blocking(load_cached_posts).await;
    refresh_server_posts_once().await;
}

#[cfg(feature = "ssr")]
async fn refresh_server_posts_once() {
    let mut from = latest_post_timestamp();
    if time::timeout(Duration::from_secs(35), fetch_newer_posts(&mut from))
        .await
        .is_err()
    {
        warn!("initial post refresh timed out; keeping cached posts");
    }
}

#[cfg(feature = "ssr")]
fn start_server_post_poll() {
    tokio::task::spawn_local(async move {
        let mut from = latest_post_timestamp();

        loop {
            time::sleep(Duration::from_secs(60)).await;
            if time::timeout(Duration::from_secs(35), fetch_newer_posts(&mut from))
                .await
                .is_err()
            {
                warn!("post sync timed out; keeping cached posts");
            }
        }
    });
}

#[cfg(feature = "ssr")]
async fn fetch_newer_posts(from: &mut Option<Timestamp>) {
    let requested_at = Timestamp::now();
    let client = get_nostr_client().await;
    let (post_events, deletion_events, metadata_events) = tokio::join!(
        client.fetch_events(post_update_filter(*from), Duration::from_secs(10)),
        client.fetch_events(
            Filter::new().author(OWNER_PUBKEY).kind(Kind::EventDeletion),
            Duration::from_secs(10),
        ),
        client.fetch_events(
            Filter::new().author(OWNER_PUBKEY).kind(Kind::Metadata),
            Duration::from_secs(10),
        ),
    );
    let events = match post_events {
        Ok(events) => events,
        Err(error) => {
            warn!("failed to fetch newer posts: {error:?}");
            return;
        }
    };

    *from = events
        .iter()
        .map(|event| event.created_at)
        .chain(*from)
        .max()
        .or(Some(requested_at));
    let authors: Authors = Default::default();
    let author = POSTS_LIST
        .read()
        .first()
        .map(|post| post.author.clone())
        .unwrap_or_else(|| Author::from_pubkey(Arc::new(OWNER_PUBKEY)));
    authors.write().insert(author.pubkey.clone(), author);
    let posts = events.into_iter().to_posts(authors).collect();
    let mut changed = merge_posts(posts);
    match deletion_events {
        Ok(events) => {
            let mut posts = POSTS_LIST.write();
            let previous_len = posts.len();
            posts.retain(|post| !events.iter().any(|event| deletes_post(event, post)));
            changed |= posts.len() != previous_len;
        }
        Err(error) => warn!("failed to refresh post deletions: {error:?}"),
    }

    match metadata_events {
        Ok(events) => {
            if let Some(metadata) = events
                .iter()
                .max_by_key(|event| event.created_at)
                .and_then(|event| serde_json::from_str::<Metadata>(&event.content).ok())
            {
                let author = author_from_metadata(metadata);
                let mut posts = POSTS_LIST.write();
                for post in posts
                    .iter_mut()
                    .filter(|post| post.author.pubkey == author.pubkey)
                {
                    if post.author.username != author.username
                        || post.author.display_name != author.display_name
                        || post.author.about != author.about
                        || post.author.picture != author.picture
                    {
                        post.author = author.clone();
                        changed = true;
                    }
                }
            }
        }
        Err(error) => warn!("failed to refresh cached post authors: {error:?}"),
    }

    if changed {
        write_cached_posts().await;
    }
}

#[cfg(any(
    feature = "ssr",
    all(
        feature = "hydrate",
        feature = "client-side-nostr-fetch",
        not(feature = "islands")
    )
))]
fn author_from_metadata(metadata: Metadata) -> Author<'static> {
    Author {
        pubkey: std::sync::Arc::new(OWNER_PUBKEY),
        username: metadata.name.map(Into::into),
        display_name: metadata.display_name.map(Into::into),
        about: metadata.about.map(Into::into),
        picture: metadata
            .picture
            .as_deref()
            .or_else(|| {
                metadata
                    .custom
                    .get("image")
                    .and_then(|value| value.as_str())
            })
            .and_then(|value| value.parse().ok()),
    }
}

#[cfg(feature = "ssr")]
fn post_update_filter(from: Option<Timestamp>) -> Filter {
    let filter = Filter::new()
        .author(OWNER_PUBKEY)
        .kind(Kind::LongFormTextNote);
    match from {
        Some(from) => filter.since(from),
        None => filter,
    }
}

#[cfg(feature = "ssr")]
fn deletes_post(event: &Event, post: &Post<'_>) -> bool {
    let requested_kinds = event
        .tags
        .iter()
        .filter_map(|tag| {
            let [key, value, ..] = tag.as_slice() else {
                return None;
            };
            (key == "k").then_some(value.as_str())
        })
        .collect::<Vec<_>>();
    event.kind == Kind::EventDeletion
        && event.pubkey == *post.author.pubkey
        && event.tags.event_ids().any(|id| *id == post.id)
        && (requested_kinds.is_empty() || requested_kinds.contains(&"30023"))
}

#[cfg(all(
    feature = "hydrate",
    feature = "client-side-nostr-fetch",
    not(feature = "islands")
))]
async fn blog_with_authors(action: &str) -> Option<Blog<'static>> {
    let client = get_nostr_client().await;
    let blog = Blog::new(client.clone(), vec![OWNER_PUBKEY]);

    if let Err(error) = blog.fetch_authors().await {
        warn!("failed to fetch authors before {action}; continuing with key fallback: {error:?}");
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
    merge_post_list(&mut POSTS_LIST.write(), incoming_posts)
}

#[cfg(feature = "ssr")]
fn merge_post_list(posts: &mut Vec<Post<'static>>, incoming_posts: Vec<Post<'static>>) -> bool {
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

cfg_if::cfg_if! {
    if #[cfg(all(
        feature = "hydrate",
        feature = "client-side-nostr-fetch",
        not(feature = "islands")
    ))] {
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
                    refresh_client_author_metadata(state.clone()).await;
                    gloo_timers::future::sleep(cooldown).await;
                }
            });
        }

        async fn refresh_client_author_metadata(state: AppState) {
            let client = get_nostr_client().await;
            let Ok(events) = client.fetch_events(
                Filter::new().author(OWNER_PUBKEY).kind(Kind::Metadata),
                std::time::Duration::from_secs(10),
            ).await else { return; };
            let Some(author) = events.iter().max_by_key(|event| event.created_at)
                .and_then(|event| serde_json::from_str::<Metadata>(&event.content).ok())
                .map(author_from_metadata)
            else { return; };
            state.live_posts.update(|posts| {
                for post in posts.iter_mut().filter(|post| post.author.pubkey == author.pubkey) {
                    post.author = author.clone();
                }
            });
        }

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

        fn latest_client_post_timestamp(posts: &Vec<Post<'static>>) -> Option<Timestamp> {
            posts.iter().map(|post| post.created_at).max()
        }
    }
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
        info!("Using stale cached posts while refreshing from relays");
    }

    match fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str::<Vec<Post<'static>>>(&json).ok())
    {
        Some(posts) if !posts.is_empty() => {
            let post_count = posts.len();
            replace_posts(posts);
            info!(
                "Loaded {post_count} cached posts from {}",
                path.to_string_lossy()
            );
            true
        }
        _ => false,
    }
}

#[cfg(feature = "ssr")]
async fn write_cached_posts() {
    if let Err(error) = tokio::task::spawn_blocking(write_cached_posts_blocking).await {
        warn!("failed to write post cache: {error:?}");
    }
}

#[cfg(feature = "ssr")]
fn write_cached_posts_blocking() {
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

    let temporary_path = path.with_extension("json.tmp");
    if let Err(error) =
        fs::write(&temporary_path, json).and_then(|_| fs::rename(&temporary_path, &path))
    {
        warn!("failed to update post cache: {error}");
    } else {
        info!("Cached {} posts at {}", posts.len(), path.to_string_lossy());
    }
}

#[cfg(feature = "ssr")]
fn posts_cache_ttl() -> Option<Duration> {
    let ttl_secs = env::var("EMERALD_POSTS_CACHE_TTL_SECS")
        .ok()
        .and_then(|ttl| ttl.parse::<u64>().ok())
        .unwrap_or(86400);

    (ttl_secs > 0).then(|| Duration::from_secs(ttl_secs))
}

#[cfg(feature = "ssr")]
fn posts_cache_path() -> PathBuf {
    env::var_os("EMERALD_POSTS_CACHE_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("emerald-blog-v3").join("posts.json"))
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use libnostrblog::{EventBuilder, Keys, Tag, blog::deletion::post_deletion_event_builder};

    #[test]
    fn refresh_keeps_existing_posts_until_new_posts_are_available() {
        let keys = Keys::generate();
        let events = ["old post", "new post"]
            .into_iter()
            .enumerate()
            .map(|(index, content)| {
                EventBuilder::long_form_text_note(content)
                    .custom_created_at(Timestamp::from(100 + index as u64))
                    .sign_with_keys(&keys)
                    .unwrap()
            });
        let mut fetched = events.to_posts(Default::default());
        let old_post = fetched.next().unwrap();
        let new_post = fetched.next().unwrap();
        let mut posts = vec![old_post.clone()];

        assert!(!merge_post_list(&mut posts, Vec::new()));
        assert_eq!(posts[0].id, old_post.id);
        assert!(merge_post_list(&mut posts, vec![new_post.clone()]));
        assert_eq!(
            posts.iter().map(|post| post.id).collect::<Vec<_>>(),
            vec![new_post.id, old_post.id]
        );
        assert!(!merge_post_list(&mut posts, vec![old_post]));
        assert_eq!(posts.len(), 2);
    }

    #[test]
    fn periodic_post_query_keeps_inclusive_watermark() {
        let filter = serde_json::to_value(post_update_filter(Some(Timestamp::from(123)))).unwrap();
        assert_eq!(filter["since"], 123);
        assert_eq!(filter["kinds"], serde_json::json!([30023]));
        assert_eq!(
            filter["authors"],
            serde_json::json!([OWNER_PUBKEY.to_hex()])
        );
    }

    #[test]
    fn incremental_deletions_remove_older_posts_only_for_matching_author_and_kind() {
        let keys = Keys::generate();
        let event = EventBuilder::long_form_text_note("old post")
            .custom_created_at(Timestamp::from(100))
            .sign_with_keys(&keys)
            .unwrap();
        let authors: Authors = Default::default();
        let post = vec![event].into_iter().to_posts(authors).next().unwrap();
        let deletion = post_deletion_event_builder(post.id, Kind::LongFormTextNote, "remove")
            .custom_created_at(Timestamp::from(200))
            .sign_with_keys(&keys)
            .unwrap();
        assert!(deletes_post(&deletion, &post));

        let other_author = post_deletion_event_builder(post.id, Kind::LongFormTextNote, "remove")
            .sign_with_keys(&Keys::generate())
            .unwrap();
        assert!(!deletes_post(&other_author, &post));
        let other_kind = post_deletion_event_builder(post.id, Kind::TextNote, "remove")
            .sign_with_keys(&keys)
            .unwrap();
        assert!(!deletes_post(&other_kind, &post));
        let no_kind = EventBuilder::new(Kind::EventDeletion, "remove")
            .tags([Tag::event(post.id)])
            .sign_with_keys(&keys)
            .unwrap();
        assert!(deletes_post(&no_kind, &post));
    }
}
