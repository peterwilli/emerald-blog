# One-Shot SSR Then CSR

The `static-export` feature turns the server binary into a one-shot generator:
it fetches the latest posts once, prerenders `/`, `/blog`, and every current
`/post/{slug}`, writes HTML into `target/site`, and exits without binding a
socket.

Use full hydration plus `client-side-nostr-fetch` for this path, not islands,
because the static site needs the full client app to add newer posts after the
build.

## Tradeoffs

Pros:

- Build-time posts and comments are prerendered and indexable.
- New posts and comments can still appear through full client-side Nostr sync.
- Runs on any static file server or shared hosting.
- No server-side runtime to patch, monitor, or restart.

Cons:

- Larger WASM than islands.
- Cannot use islands because dynamically adding new posts needs the full hydrated app/router.
- SEO only covers content present at export time.
- Generated HTML must be rebuilt to make later posts available without JavaScript.

## Build

Build assets and the static-export binary:

```bash
cargo leptos build --release \
  --bin-features static-export \
  --lib-features hydrate,client-side-nostr-fetch
```

## Generate Static HTML

```bash
LEPTOS_SITE_ROOT=target/site target/release/emerald-blog-v3
```

The resulting `target/site/` directory contains the static HTML, Leptos
CSS/WASM assets, and files copied from `public/`. Serve that directory with any
static file host.

## Comments

Comments do not depend on server functions in this mode. If a visitor has a
NIP-07 signer, comments are signed by that signer. If not, the browser creates
a random Nostr private key, stores it in `window.localStorage`, and reuses it
for later visits from the same browser profile.

## Cache Fallback

If the live Nostr fetch fails, the exporter falls back to the configured post
cache when available:

```bash
EMERALD_POSTS_CACHE_PATH=/tmp/emerald-posts.json \
EMERALD_POSTS_CACHE_TTL_SECS=86400 \
LEPTOS_SITE_ROOT=target/site \
target/release/emerald-blog-v3
```

Re-run the build/export whenever you want crawlers and no-JavaScript visitors
to receive the newest posts as static HTML.
