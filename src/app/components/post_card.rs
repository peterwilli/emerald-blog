use leptos::prelude::*;
use libnostrblog::objects::post::Post;
use slug::slugify;

use crate::app::utils::to_date_string;

#[component]
pub fn PostCard(post: Post<'static>) -> impl IntoView {
    view! {
        <div class="glass rounded-xl p-6 transition-all duration-300 post-card">
            <h3 class="text-xl font-bold text-emerald mb-2">{post.title.clone()}</h3>
            <p class="text-emerald/70 text-sm mb-3">{to_date_string(post.created_at)}</p>
            <p class="text-emerald/80 mb-4">{post.excerpt}</p>
            <a
                href=format!("/post/{}", slugify(&post.title))
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
            </a>
        </div>
    }
}
