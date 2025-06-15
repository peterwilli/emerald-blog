use crate::app::{AppState, utils::to_date_string};
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;
use slug::slugify;

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let post_slug = params.read_untracked().get("slug").unwrap();
    let state = use_context::<AppState>().unwrap();
    let posts = state.posts;
    let post = RwSignal::new(None);
    Effect::new(move || {
        if let Some(posts) = posts.get() {
            let posts = posts.read();
            let result = posts.iter().find(|&p| slugify(&p.title) == post_slug);
            if let Some(result) = result {
                post.set(Some(result.clone()));
            }
        }
    });
    let post_slug = params.read_untracked().get("slug").unwrap();

    view! {
        <div class="glass rounded-xl p-6 transition-all duration-300 post-card">
            <Suspense fallback=move || {
                view! { <p>"Loading..."</p> }
            }>
                {move || {
                    post.get()
                        .map(|post| {
                            view! {
                                <Title text=format!("{} - Emerald Show", post.title) />
                                <Meta property="og:title" content=post.title.to_string() />
                                <Meta property="og:description" content=post.excerpt.to_string() />
                                <Meta property="og:type" content="article" />
                                <Meta
                                    property="og:url"
                                    content=format!("https://emerald.show/{}", post_slug)
                                />
                                <Meta
                                    property="og:image"
                                    content="https://yourblog.com/image.jpg"
                                />
                                <Meta property="og:site_name" content="Emerald Show" />

                                <h1 class="text-2xl font-bold text-emerald mb-4">
                                    {post.title.clone()}
                                </h1>
                                <p class="text-emerald/70 text-sm mb-3">
                                    {to_date_string(post.created_at)}
                                </p>
                                <div class="prose prose-emerald max-w-none">
                                    {post.content.clone()}
                                </div>
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
