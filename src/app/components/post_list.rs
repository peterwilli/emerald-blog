use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use libnostrblog::objects::post::Post;
use std::sync::Arc;

use crate::app::{
    AppState,
    components::{category_filter::CategoryFilter, post_card::PostCard},
    utils::{decode_path_segment, is_art_post, post_has_category},
};

#[component]
pub fn PostList(#[prop(optional)] show_category_filter: bool) -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts.clone();
    view! {
        <div>
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    posts
                        .try_get()
                        .flatten()
                        .map(|posts| {
                            view! { <PostListView initial_posts=posts show_category_filter /> }
                        })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn PostListView(initial_posts: Vec<Post<'static>>, show_category_filter: bool) -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let live_posts = state.live_posts.clone();
    let params = use_params_map();
    let initial_posts = Arc::new(initial_posts);

    view! {
        <div>
            {move || {
                let active_category = params
                    .read()
                    .get("category")
                    .map(|category| decode_path_segment(&category));
                let current_live_posts = live_posts.try_get().unwrap_or_default();
                let posts = blog_posts(&initial_posts, &current_live_posts, active_category.as_deref());
                let filter = if show_category_filter {
                    view! {
                        <CategoryFilter
                            posts=blog_posts(&initial_posts, &current_live_posts, None)
                            base_path="/blog"
                            active_category=active_category.clone()
                        />
                    }
                    .into_any()
                } else {
                    ().into_any()
                };

                view! {
                    {filter}
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <For
                            each=move || posts.clone().into_iter()
                            key=|post| post.id
                            children=move |post: Post| {
                                view! { <PostCard post=post /> }
                            }
                        />
                    </div>
                }
            }}
        </div>
    }
}

fn blog_posts(
    initial_posts: &[Post<'static>],
    live_posts: &[Post<'static>],
    active_category: Option<&str>,
) -> Vec<Post<'static>> {
    let posts = if live_posts.is_empty() {
        initial_posts
    } else {
        live_posts
    };

    posts
        .iter()
        .filter(|post| !is_art_post(post))
        .filter(|post| {
            active_category
                .map(|category| post_has_category(post, category))
                .unwrap_or(true)
        })
        .cloned()
        .collect()
}
