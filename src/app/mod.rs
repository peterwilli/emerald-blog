use layout::header::Header;
use leptos::{html::Footer, prelude::*, task::spawn_local};
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
    path,
};
use libnostrblog::objects::post::Post;
pub mod client;
pub mod components;
pub mod layout;
pub mod pages;
pub mod ssr_posts;
pub mod utils;
use pages::{home_page::HomePage, post_page::PostPage};
use ssr_posts::POSTS_LIST;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <link rel="stylesheet" id="leptos" href="/pkg/emerald_blog.css" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub posts: OnceResource<RwSignal<Vec<Post<'static>>>>,
}

#[server]
pub async fn seed_posts() -> Result<Vec<Post<'static>>, ServerFnError> {
    Ok(POSTS_LIST.read().clone())
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Providing global posts context
    let app_state = AppState {
        posts: OnceResource::new(async { RwSignal::new(seed_posts().await.unwrap()) }),
    };
    provide_context(app_state);

    view! {
        <Header />
        <Router>
            <main class="relative z-10 container mx-auto px-4 py-8 max-w-6xl">
                <Routes fallback=|| "Not found.">
                    <Route path=StaticSegment("") view=HomePage />
                    <Route path=path!("/post/:slug") view=PostPage />
                </Routes>
            </main>
        </Router>
        <div class="emerald-bg fixed left-0 right-0 top-0 bottom-0" />
        <script src="/EmeraldBlog.js"></script>
    }
}
