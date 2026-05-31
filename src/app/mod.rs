use layout::header::Header;
use leptos::prelude::*;
use leptos_meta::{MetaTags, provide_meta_context};
use leptos_router::{
    SsrMode, StaticSegment,
    components::{Route, Router, Routes},
    path,
};
use libnostrblog::objects::post::Post;
pub mod client;
pub mod components;
pub mod layout;
pub mod markdown;
pub mod nostr_browser;
pub mod pages;
pub mod post_sync;
pub mod utils;
use pages::{
    admin_page::AdminPostPage, blog_page::BlogPage, home_page::HomePage, post_page::PostPage,
};
use post_sync::{PostsSync, seed_posts};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="theme-color" content="#080215" />
                <style>
                    "html,body{background:#080215;color-scheme:dark;}body{margin:0;min-height:100vh;}"
                </style>
                <link rel="modulepreload" href="/emerald-core.js" />
                <link rel="preload" href="/dflat.glb" r#as="fetch" crossorigin="anonymous" />
                <link rel="stylesheet" id="leptos" href="/pkg/emerald_blog.css" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <script type="module" src="/emerald-core.js"></script>
                <HydrationScripts options islands=cfg!(feature = "islands") />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[derive(Clone)]
pub struct AppState {
    pub posts: ArcOnceResource<Vec<Post<'static>>>,
    pub live_posts: ArcRwSignal<Vec<Post<'static>>>,
    #[allow(dead_code)]
    client_post_sync_started: ArcRwSignal<bool>,
}

impl AppState {
    pub fn merge_posts(&self, incoming_posts: Vec<Post<'static>>) {
        self.live_posts.update(|posts| {
            for post in incoming_posts {
                if posts.iter().any(|existing| existing.id == post.id) {
                    continue;
                }

                posts.push(post);
            }

            posts.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        });
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Providing global posts context
    let app_state = AppState {
        posts: ArcOnceResource::new(async { seed_posts().await.unwrap_or_default() }),
        live_posts: ArcRwSignal::new(Vec::new()),
        client_post_sync_started: ArcRwSignal::new(false),
    };
    provide_context(app_state);

    view! {
        <Router>
            <PostsSync />
            <Header />
            <main class="relative z-10 container mx-auto px-4 py-8 max-w-8xl">
                <Routes fallback=|| "Not found.">
                    <Route path=StaticSegment("") view=HomePage ssr=page_ssr_mode() />
                    <Route path=path!("/post/:slug") view=PostPage ssr=post_ssr_mode() />
                    <Route path=path!("blog") view=BlogPage ssr=page_ssr_mode() />
                    <Route path=path!("/admin/post") view=AdminPostPage />
                </Routes>
                <footer class="site-footer">
                    <p class="site-copyright">
                        "© " <span id="copyright-year">"2026"</span> " Peter \"Emerald\" Willemsen"
                        <br />
                        <leptos_router::components::A
                            attr:class="text-sm font-medium text-white/85 drop-shadow-[0_1px_2px_rgba(0,0,0,0.75)] transition-colors hover:text-gold"
                            href="/admin/post"
                        >
                            Admin
                        </leptos_router::components::A>

                    </p>
                    <div class="emerald-badge-rack" aria-label="Site badges">
                        <span class="emerald-badge emerald-badge-love">
                            <span class="emerald-badge-icon" aria-hidden="true"></span>
                            <span class="emerald-badge-text">"Made with love"</span>
                        </span>
                        <span class="emerald-badge emerald-badge-falling">
                            <span class="emerald-badge-icon" aria-hidden="true"></span>
                            <span class="emerald-badge-text">"Beware of falling emeralds"</span>
                        </span>
                        <span class="emerald-badge emerald-badge-nvim">
                            <span class="emerald-badge-icon" aria-hidden="true"></span>
                            <span class="emerald-badge-text">"Made with nvim"</span>
                        </span>
                        <span class="emerald-badge emerald-badge-nostr">
                            <span class="emerald-badge-icon" aria-hidden="true"></span>
                            <span class="emerald-badge-text">"Powered by Nostr"</span>
                        </span>
                    </div>
                </footer>
            </main>
        </Router>
        <div class="emerald-bg fixed left-0 right-0 top-0 bottom-0" />
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "static-export")] {
        use leptos_router::static_routes::{StaticParamsMap, StaticRoute};
        use post_sync::POSTS_LIST;

        fn page_ssr_mode() -> SsrMode {
            SsrMode::Static(StaticRoute::new())
        }

        fn post_ssr_mode() -> SsrMode {
            SsrMode::Static(StaticRoute::new().prerender_params(|| async {
                let mut params = StaticParamsMap::new();
                let slugs = POSTS_LIST
                    .read()
                    .iter()
                    .map(utils::post_slug)
                    .collect::<Vec<_>>();
                params.insert("slug", slugs);
                params
            }))
        }
    } else {
        fn page_ssr_mode() -> SsrMode {
            SsrMode::OutOfOrder
        }

        fn post_ssr_mode() -> SsrMode {
            SsrMode::Async
        }
    }
}
