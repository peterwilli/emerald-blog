#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use emerald_blog_v3::app::{ssr_posts::fetch_posts, *};
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tokio::task::LocalSet;
    use tracing::{debug, info};
    use tracing_subscriber::{EnvFilter, fmt};

    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    debug!("generated route list");

    // SSR Posts
    info!("fetching posts");
    LocalSet::new()
        .run_until(async move {
            fetch_posts().await;
        })
        .await;
    info!("finished fetching posts");

    let app = Router::new()
        .leptos_routes_with_context(&leptos_options, routes, move || {}, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
