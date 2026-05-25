use leptos::prelude::*;
use libnostrblog::objects::post::Post;
#[cfg(feature = "hydrate")]
use libnostrblog::{
    Blog, Timestamp,
    blog::{fetch_authors::FetchAuthorsExt, poll_posts::PollPostsExt},
};

#[cfg(feature = "hydrate")]
use crate::app::client::{OWNER_PUBKEY, get_nostr_client};
use crate::app::{AppState, components::post_card::PostCard, utils::is_dream_art_post};
#[cfg(feature = "hydrate")]
use tracing::{debug, warn};

#[component]
pub fn PostList() -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts;
    view! {
        <div>
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    posts
                        .get()
                        .map(|posts| {
                            view! { <PostListIsland initial_posts=posts.read().clone() /> }
                        })
                }}
            </Suspense>
        </div>
    }
}

#[island]
fn PostListIsland(initial_posts: Vec<Post<'static>>) -> impl IntoView {
    let posts = RwSignal::new(initial_posts);

    #[cfg(feature = "hydrate")]
    start_post_poll(posts);

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <For
            each=move || {
                posts
                    .read()
                    .iter()
                    .filter(|post| !is_dream_art_post(post))
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
            }
            key=|post| post.id
            children=move |post: Post| {
                view! { <PostCard post=post /> }
            }
        />
        </div>
    }
}

#[cfg(feature = "hydrate")]
fn start_post_poll(posts: RwSignal<Vec<Post<'static>>>) {
    let from = posts.with_untracked(latest_timestamp);

    leptos::task::spawn_local(async move {
        let client = get_nostr_client().await;
        let blog = Blog::new(client.clone(), vec![OWNER_PUBKEY]);

        if let Err(error) = blog.fetch_authors().await {
            warn!("failed to fetch authors before polling posts: {error:?}");
            return;
        }

        let mut rx = match blog.poll_posts(from).await {
            Ok(rx) => rx,
            Err(error) => {
                warn!("failed to poll posts: {error:?}");
                return;
            }
        };

        while let Some(post) = rx.recv().await {
            debug!("poll added post: {:?}", post);
            posts.update(|posts| {
                if posts.iter().any(|existing| existing.id == post.id) {
                    return;
                }

                posts.push(post);
                posts.sort_by(|left, right| right.created_at.cmp(&left.created_at));
            });
        }
    });
}

#[cfg(feature = "hydrate")]
fn latest_timestamp(posts: &Vec<Post<'static>>) -> Option<Timestamp> {
    posts.iter().map(|post| post.created_at).max()
}
