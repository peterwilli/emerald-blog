#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use emerald_blog_v3::app::*;
    use leptos::prelude::*;
    use tokio::task::LocalSet;
    use tracing_subscriber::{EnvFilter, fmt};

    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    any_spawner::Executor::init_tokio().ok();

    LocalSet::new()
        .run_until(async move {
            let conf = get_configuration(None).unwrap();
            let leptos_options = conf.leptos_options;

            cfg_if::cfg_if! {
                if #[cfg(feature = "static-export")] {
                    use emerald_blog_v3::app::post_sync::load_server_posts_once;
                    use leptos_axum::generate_route_list_with_ssg;
                    use tracing::info;

                    info!("loading posts for static export");
                    load_server_posts_once().await;

                    let shell_options = leptos_options.clone();
                    let (_routes, static_routes) =
                        generate_route_list_with_ssg(move || shell(shell_options.clone()));
                    info!(
                        "generating static HTML in {}",
                        leptos_options.site_root.as_ref()
                    );
                    static_routes.generate(&leptos_options).await;
                    info!("static export complete");
                } else {
                    use axum::Router;
                    use emerald_blog_v3::app::post_sync::start_server_post_sync;
                    use leptos_axum::{generate_route_list, LeptosRoutes};
                    use tracing::{debug, info};

                    // Generate the list of routes in your Leptos App
                    let routes = generate_route_list(App);
                    debug!("generated route list");

                    info!("starting server post sync");
                    start_server_post_sync().await;

                    let addr = leptos_options.site_addr;
                    let app = Router::new()
                        .leptos_routes_with_context(&leptos_options, routes, move || {}, {
                            let leptos_options = leptos_options.clone();
                            move || shell(leptos_options.clone())
                        })
                        .fallback(leptos_axum::file_and_error_handler(shell))
                        .with_state(leptos_options);

                    info!("listening on http://{}", &addr);
                    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
                    axum::serve(listener, app.into_make_service())
                        .await
                        .unwrap();
                }
            }
        })
        .await;
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
