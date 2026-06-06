# Repository Guidelines

## Project Overview

Rust 2024 Leptos/Axum application built with `cargo-leptos`.
The app is an SSR/hydrated blog for Emerald Show. Blog content comes from Nostr through `libnostrblog`, is fetched on server startup, and is exposed to Leptos views through `AppState`.

- `src/main.rs` is compiled only with the `ssr` feature. It loads Leptos config, builds the route list, polls Nostr posts, then starts the Axum server.
- `src/lib.rs` exposes the shared app crate and the WASM hydration entry point behind the `hydrate` feature.
- `src/app/mod.rs` defines `shell`, `App`, routes, the server function `seed_posts`, and the global posts context.
- `src/app/client.rs` owns the Nostr public key, relay list, generated signer, and singleton client.
- `src/app/post_sync.rs` seeds SSR posts and keeps server/client post sync behavior together.
- Routes are `/`, `/blog`, `/blog/:category`, `/art`, `/art/:category`, and `/post/:slug`. Post links use `slug::slugify(&post.title)`.

## Project Structure & Module Organization

- `src/main.rs` starts the SSR server; `src/lib.rs` exposes shared server/WASM app code.
- `src/app/` contains routes, pages, layout, utilities, and components.
- `src/app/pages/` holds route-level page views; `src/app/components/` holds reusable UI pieces.
- `src/app/layout/` contains layout-level UI such as the header.
- `public/` contains static assets copied into the Leptos site output.
- `style/tailwind.css` is the Tailwind input configured in `Cargo.toml`.
- `end2end/` contains Playwright tests and Node package files.
- `packages/emerald-background/` contains the importable JavaScript package for the emerald background scene.
- `core-js/` contains the Parcel project that bundles the emerald background and browser Nostr helpers into `public/emerald-core.js`.
- `target/site/` and `target/` are generated build outputs and should not be edited directly.

## Build, Test, and Development Commands

- `cargo leptos watch` runs the local development server with rebuilds.
- `cargo leptos build --release` builds the production SSR binary and `target/site` assets.
- `cargo test` runs Rust tests, when present.
- `cargo leptos end-to-end` runs Playwright end-to-end tests through the configured `end2end` command.
- `cd end2end && npm install` installs browser test dependencies.
- `pnpm install --frozen-lockfile` installs frontend package dependencies used by the build.
- `pnpm build:core-js` builds the browser JavaScript bundle served as `/emerald-core.js`.

Local tools include Rust nightly, `cargo-leptos`, `wasm32-unknown-unknown`, Sass, and Node package tooling.

## Coding Style & Naming Conventions

Use `rustfmt` defaults for Rust formatting: 4-space indentation, idiomatic imports, and standard brace placement. Prefer `snake_case` for modules, functions, variables, and files; use `PascalCase` for Rust types and Leptos components. Keep page modules in `src/app/pages/` and reusable UI in `src/app/components/`.

Run `cargo fmt` before submitting changes. Use names that describe UI responsibility, for example `post_card.rs` or `blog_page.rs`.

- Keep comments rare. Prefer self-describing function, signal, resource, and component names over explanatory comments. Add comments only when they explain non-obvious behavior, feature-gated constraints, or external integration details.
- Follow the current Leptos style: `#[component]` functions returning `impl IntoView`, route pages named `HomePage`, `BlogPage`, `PostPage`, and reusable components named after the UI they render.
- Use `view!` markup directly for focused components. Extract a component when markup is reused or when a route-level view becomes hard to scan.
- When making large structural changes to Leptos views, split deeply nested `view!` trees into smaller components before they become one large generated type. This avoids Rust query-depth or recursion-limit overflows from Tachys/Leptos markup expansion.
- New features must work across all three documented deployment methods: full SSR, islands SSR, and one-shot static export with CSR. Keep route additions, server functions, hydration behavior, and post/category state compatible with each path.
- Use `cfg_if!` for large conditional compilation switches so feature-gated SSR, hydration, islands, and static-export behavior stays readable.
- Prefer Leptos signals/resources for reactive state. The existing post data flow uses `OnceResource<RwSignal<Vec<Post<'static>>>>` in `AppState`; keep consumers reading from context instead of re-fetching posts.
- Use `Suspense` around resource-backed UI and provide small, stable fallbacks.
- Keep route URLs and post slugs consistent with the existing `/post/{slugify(title)}` convention unless changing routing end to end.
- Avoid introducing extra clones or owned strings in hot rendering paths when borrowing is practical, but keep Leptos lifetime requirements clear and compilable.

## UI & Styling

- Styling is Tailwind CSS with DaisyUI enabled in `style/tailwind.css`.
- The visual language is dark glass panels, emerald/gold accents, subtle glow, and compact cards. Reuse classes such as `glass`, `logo-glow`, `social-icon`, and `post-card` when they match the need.
- Prefer Tailwind utility classes in Leptos markup for one-off layout and spacing; add CSS classes in `style/tailwind.css` for repeated effects or named visual concepts.
- Static assets belong in `public/` and are referenced from the site root, for example `/discord-icon.svg`.
- Do not edit `target/site/` to change assets or styles; update source files under `public/`, `style/`, or `src/`.

## Data & SSR Notes

- Post data is server-seeded from `POSTS_LIST` through the `seed_posts` server function. Components should use `use_context::<AppState>()` for the shared posts resource.
- Nostr relay changes belong in `src/app/client.rs`. Owner or author changes should be made there as well.
- `poll_posts()` currently runs before the Axum server starts. Be careful with changes that could block startup indefinitely or make local development depend on slow network behavior.
- `PostPage` finds posts by comparing the URL slug to `slugify(&post.title)`. If duplicate titles become possible, update the route/keying strategy consistently in `PostCard`, `PostPage`, and tests.
- Generated hydration output uses the `emerald_blog` package name from `Cargo.toml`; do not rename it casually because `shell` references `/pkg/emerald_blog.css`.

## Testing Guidelines

Playwright specs live in `end2end/tests/` and use the `*.spec.ts` naming pattern. Add coverage for route behavior, visible page content, and browser interactions. Place focused Rust tests near the code they validate using `#[cfg(test)]` modules when practical.

Before opening a PR, run `cargo test` and the relevant `cargo leptos end-to-end` path. For release-sensitive changes, also run `cargo leptos build --release`.

## Commit & Pull Request Guidelines

The current Git history is minimal and does not enforce a strict convention. Use concise, imperative commit subjects such as `Update Docker runtime image` or `Add post card tests`.

Pull requests should include a short summary, testing performed, and any configuration or deployment notes. Include screenshots or recordings for visible UI changes, and link related issues when available.

## Security & Configuration Tips

Do not commit secrets, local environment files, or generated build output. Runtime settings such as `LEPTOS_SITE_ADDR` and `LEPTOS_SITE_ROOT` should be provided through environment variables or container configuration.
