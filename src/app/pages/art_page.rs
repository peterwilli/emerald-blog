use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;
use libnostrblog::objects::post::Post;
use std::sync::Arc;

use crate::app::{
    AppState,
    components::{
        category_filter::CategoryFilter, media_display::MediaDisplay, post_card::PostCard,
    },
    utils::{decode_path_segment, is_art_post, post_has_category, post_slug},
};

#[component]
pub fn ArtPage() -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts.clone();

    view! {
        <Title text="Emerald Art Gallery" />
        <Suspense fallback=move || {
            view! { <div class="art-gallery-skeleton glass"></div> }
        }>
            {move || {
                posts
                    .try_get()
                    .flatten()
                    .map(|posts| view! { <ArtGallery initial_posts=posts /> })
            }}
        </Suspense>
    }
}

#[component]
fn ArtGallery(initial_posts: Vec<Post<'static>>) -> impl IntoView {
    let state = use_context::<AppState>().unwrap();
    let live_posts = state.live_posts.clone();
    let params = use_params_map();
    let initial_posts = Arc::new(initial_posts);

    view! {
        {move || {
            let active_category = params
                .read()
                .get("category")
                .map(|category| decode_path_segment(&category));
            let current_live_posts = live_posts.try_get().unwrap_or_default();
            let scoped_posts = art_posts(&initial_posts, &current_live_posts, active_category.as_deref());
            let all_art_posts = art_posts(&initial_posts, &current_live_posts, None);
            let featured = scoped_posts.first().cloned();
            let remaining = scoped_posts.iter().skip(1).cloned().collect::<Vec<_>>();

            view! {
                <section class="art-gallery-hero">
                    <div>
                        <p class="art-gallery-kicker">"Emerald Art"</p>
                        <h1>"Gallery"</h1>
                    </div>
                </section>

                <CategoryFilter
                    posts=all_art_posts
                    base_path="/art"
                    active_category=active_category.clone()
                />

                {featured
                    .map(|post| view! { <FeaturedArt post=post /> }.into_any())
                    .unwrap_or_else(|| {
                        view! {
                            <div class="glass rounded-xl p-6 text-white/85">
                                "No art posts found for this category."
                            </div>
                        }.into_any()
                    })}

                <div class="art-gallery-grid">
                    <For
                        each=move || remaining.clone().into_iter()
                        key=|post| post.id
                        children=move |post| view! { <PostCard post=post /> }
                    />
                </div>
            }
        }}
    }
}

#[component]
fn FeaturedArt(post: Post<'static>) -> impl IntoView {
    let href = format!("/post/{}", post_slug(&post));

    view! {
        <a href=href class="art-gallery-feature glass">
            {post
                .featured_image
                .clone()
                .map(|image| {
                    view! {
                        <MediaDisplay
                            src=image.to_string()
                            class="art-gallery-feature-image"
                            alt=post.title.to_string()
                        />
                    }
                })}
            <div class="art-gallery-feature-copy">
                <p>"Featured"</p>
                <h2>{post.title.clone()}</h2>
                <span>{post.excerpt.clone()}</span>
            </div>
        </a>
    }
}

fn art_posts(
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
        .filter(|post| is_art_post(post))
        .filter(|post| {
            active_category
                .map(|category| post_has_category(post, category))
                .unwrap_or(true)
        })
        .cloned()
        .collect()
}
