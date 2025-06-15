use leptos::prelude::*;
use libnostrblog::objects::post::Post;

use crate::app::{AppState, components::post_card::PostCard};

#[component]
pub fn PostList() -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts;
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    posts
                        .get()
                        .map(|posts| {
                            view! {
                                <For
                                    each=move || posts.read().clone().into_iter()
                                    key=|post| format!("{}", post.created_at)
                                    children=move |post: Post| {
                                        view! { <PostCard post=post /> }
                                    }
                                />
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
