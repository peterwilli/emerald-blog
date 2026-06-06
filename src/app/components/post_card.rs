use leptos::prelude::*;
use leptos_router::components::A;
use libnostrblog::objects::post::Post;

use crate::app::components::media_display::MediaDisplay;
use crate::app::utils::{is_art_category, post_slug, to_date_string};

#[component]
pub fn PostCard(post: Post<'static>) -> impl IntoView {
    let href = format!("/post/{}", post_slug(&post));

    view! {
        <A
            href=href
            attr:class="glass rounded-xl transition-all duration-300 post-card post-card-link overflow-hidden block group"
        >
            {post
                .featured_image
                .map(|image| {
                    view! {
                        <div class="w-full aspect-3/2">
                            <MediaDisplay
                                src=image.to_string()
                                class="w-full h-full object-cover"
                                alt=post.title.to_string()
                            />
                        </div>
                    }
                })} <div class="p-6">
                <h3 class="text-xl font-bold text-emerald mb-2">{post.title.clone()}</h3>
                <p class="text-emerald/70 text-sm mb-3">{to_date_string(post.created_at)}</p>
                <p class="text-emerald/80 mb-4">{post.excerpt}</p>
                <div class="pb-4">
                    {post
                        .categories
                        .iter()
                        .filter(|c| !is_art_category(c))
                        .map(|c| {
                            view! { <span class="post-tag">{c.clone()}</span> }
                        })
                        .collect::<Vec<_>>()}
                </div>
                <span
                    class="inline-flex items-center text-emerald font-medium hover:text-gold transition-colors duration-300"
                >
                    "Read more"
                    <svg
                        class="w-4 h-4 ml-2 transition-transform duration-300 transform group-hover:translate-x-1"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <path
                            strokelinecap="round"
                            strokelinejoin="round"
                            strokewidth="2"
                            d="M14 5l7 7m0 0l-7 7m7-7H3"
                        ></path>
                    </svg>
                </span>
            </div>
        </A>
    }
}
