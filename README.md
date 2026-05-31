# Emerald Blog v3

Rust 2024 Leptos/Axum SSR blog for Emerald Show. Blog posts are fetched from
Nostr through `libnostrblog`, seeded into SSR state, and rendered by Leptos
routes at `/`, `/blog`, and `/post/:slug`.

## Prerequisites

Install the local build tools:

Commands below use npm. pnpm also works; add `p` to `npm` commands when using it.

```bash
rustup toolchain install nightly --allow-downgrade
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
npm install
```

Install browser test dependencies when running end-to-end tests:

```bash
cd end2end && npm install
```

## JavaScript Assets

The Nostr/browser helper bundle and emerald background scene are built into
`public/emerald-core.js`:

```bash
npm run build:core-js
```

Run this when changing files under `core-js/` or
`packages/emerald-background/`.

## Development

Run the normal SSR/hydrated app:

```bash
npm run watch
```

This uses the `Cargo.toml` cargo-leptos defaults:

```toml
bin-features = ["ssr"]
lib-features = ["hydrate"]
```

The server fetches posts on startup and continues polling for newer posts. The
client receives the SSR-seeded post list through Leptos resources.

## Deployment Paths

| Path | Best For | Pros | Cons |
| --- | --- | --- | --- |
| [Full SSR Build](docs/deployment/full-ssr.md) | A live server that should always show the newest posts and comments. | Fully SEO-optimized HTML for post routes; server keeps polling Nostr for new posts; full WASM can sync newer posts and comments in the browser; works well with dynamic routes and client-side navigation. | Requires an Axum server process; larger WASM than islands because the full app hydrates; operational work includes process hosting, logs, restarts, and runtime config. |
| [Islands SSR Build](docs/deployment/islands-ssr.md) | A live server with the smallest practical interactive WASM surface. | Fully SEO-optimized HTML for post routes; smaller WASM because only islands hydrate; posts are rendered from SSR state; Nostr actions like publishing comments still work through browser WASM. | Requires an Axum server process; browser does not add newly discovered posts to the list; less suitable if you need client-side post streaming or offline-first browsing. |
| [One-Shot SSR Then CSR](docs/deployment/one-shot-ssr-then-csr.md) | Static hosting while keeping client-side Nostr updates after the initial prerender. | Build-time posts and comments are prerendered and indexable; new posts/comments can still appear through full client-side Nostr sync; runs on any static file server or shared host; no server-side runtime to patch or monitor. | Larger WASM than islands; cannot use islands because dynamically adding new posts needs the full hydrated app/router; SEO only covers content present at export time; generated HTML must be rebuilt to make later posts available without JavaScript. |

## Testing

Run Rust tests:

```bash
cargo test
```

Run Playwright end-to-end tests:

```bash
cargo leptos end-to-end
```

For release-sensitive changes:

```bash
cargo leptos build --release
cargo leptos end-to-end --release
```
