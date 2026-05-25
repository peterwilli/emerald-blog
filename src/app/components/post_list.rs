use leptos::prelude::*;
use libnostrblog::objects::post::Post;

use crate::app::{AppState, components::post_card::PostCard, utils::is_dream_art_post};

#[component]
pub fn PostList() -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts.clone();
    view! {
        <div>
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    posts
                        .get()
                        .map(|posts| {
                            view! { <PostListView initial_posts=posts /> }
                        })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn PostListView(initial_posts: Vec<Post<'static>>) -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let live_posts = state.live_posts.clone();

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <For
                each=move || {
                    let current_posts = live_posts.get();
                    let posts = if current_posts.is_empty() {
                        initial_posts.clone()
                    } else {
                        current_posts
                    };

                posts
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
