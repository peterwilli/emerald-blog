# Islands SSR Build

This is the smaller-WASM live server build. The server still fetches and renders
all post data, but the browser hydrates only components marked as islands. Blog
post streaming in the browser is intentionally disabled in islands mode, even
if `client-side-nostr-fetch` is passed, so new posts are added by SSR/server
polling rather than by client-side list mutation.

The current islands are interactive browser-only surfaces such as the admin
publisher, comments section (including its composer), and art slider. Nostr publishing/comment actions
continue to use WASM, while blog post rendering remains SSR-driven.

Post pages render cached comments without waiting for Nostr. The server refreshes
comments in the background at most once per minute per requested post, and the
comments island checks the server cache every 15 seconds. A cold cache initially
renders no comments; they appear after the first successful refresh. The comment
cache resets when the server restarts.

Post snapshots are saved to disk by default. On restart, the server serves the
last snapshot immediately and refreshes it in the background, including when it
is older than `EMERALD_POSTS_CACHE_TTL_SECS` (default: 86400). Set the TTL to `0`
to disable disk caching. Set `EMERALD_POSTS_CACHE_PATH` to a file on a persistent
volume to retain snapshots across container replacements.

## Tradeoffs

Pros:

- Fully SEO-optimized HTML for `/`, `/blog`, `/art`, and individual post routes.
- Smaller WASM than full hydration because only islands hydrate.
- Blog post rendering stays SSR-driven and predictable.
- Nostr publishing and commenting still work through browser WASM.

Category filters use static route paths such as `/blog/journal` and
`/art/portraits`, rendered from SSR state. They do not require browser-side
post streaming.

Cons:

- Requires a running Axum server process.
- Browser-side blog post streaming is disabled.
- New posts are not dynamically inserted into the browser list without SSR/server updates.
- Less suitable for offline-first browsing because post discovery stays server-driven.

## Development

Commands below use npm. pnpm also works; add `p` to `npm` commands when using it.

```bash
npm run watch -- \
  --bin-features ssr,islands \
  --lib-features hydrate,islands
```

## Release Build

```bash
cargo leptos build --release \
  --bin-features ssr,islands \
  --lib-features hydrate,islands
```

## Run

```bash
LEPTOS_SITE_ROOT=target/site target/release/emerald-blog-v3
```

The server listens on `LEPTOS_SITE_ADDR`, defaulting to `127.0.0.1:3000`.

## Docker

The `Dockerfile` accepts build arguments for the Leptos binary and WASM feature
sets. The runtime image listens on port `8080` and uses `/app/site` as
`LEPTOS_SITE_ROOT`.

Build the islands SSR image:

```bash
docker build \
  -t emerald-blog-v3:islands-ssr \
  --build-arg BIN_FEATURES=ssr,islands \
  --build-arg LIB_FEATURES=hydrate,islands \
  .
```

Run it locally:

```bash
docker run --rm \
  -p 8080:8080 \
  -e RUST_LOG=info \
  -e LEPTOS_SITE_ADDR=0.0.0.0:8080 \
  emerald-blog-v3:islands-ssr
```

Open `http://localhost:8080`.

## Docker Compose

Create `compose.islands-ssr.yml`:

```yaml
services:
  emerald-blog:
    build:
      context: .
      args:
        BIN_FEATURES: ssr,islands
        LIB_FEATURES: hydrate,islands
    image: emerald-blog-v3:islands-ssr
    ports:
      - "8080:8080"
    environment:
      RUST_LOG: info
      LEPTOS_SITE_ADDR: 0.0.0.0:8080
```

Start it:

```bash
docker compose -f compose.islands-ssr.yml up --build
```

Stop it:

```bash
docker compose -f compose.islands-ssr.yml down
```
