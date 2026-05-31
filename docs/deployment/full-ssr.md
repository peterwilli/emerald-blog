# Full SSR Build

This is the default production server build. The Axum server fetches posts at
startup, keeps polling Nostr while it runs, SSR-renders routes, and ships the
full hydrated Leptos app. Enable client-side Nostr post streaming when you want
the browser to merge in newer posts/comments without waiting for a full page
load.

## Tradeoffs

Pros:

- Fully SEO-optimized HTML for `/`, `/blog`, and individual post routes.
- Server polling keeps post data fresh for crawlers and no-JavaScript visitors.
- Full WASM hydration can sync newer posts and comments in the browser.
- Works well with dynamic routes and client-side navigation.

Cons:

- Requires a running Axum server process.
- Larger WASM than islands because the full app hydrates.
- Hosting needs process supervision, logs, restarts, and runtime configuration.

## Development

Commands below use npm. pnpm also works; add `p` to `npm` commands when using it.

```bash
npm run watch -- \
  --bin-features ssr \
  --lib-features hydrate,client-side-nostr-fetch
```

## Release Build

```bash
cargo leptos build --release \
  --bin-features ssr \
  --lib-features hydrate,client-side-nostr-fetch
```

## Run

```bash
LEPTOS_SITE_ROOT=target/site target/release/emerald-blog-v3
```

The server listens on `LEPTOS_SITE_ADDR`, defaulting to `127.0.0.1:3000`.

## Docker

The `Dockerfile` builds the Leptos binary and `target/site` assets in a Rust
builder image, then copies only the release binary and site assets into a
distroless runtime image. The runtime listens on port `8080` and uses
`/app/site` as `LEPTOS_SITE_ROOT`.

Build the full SSR image with client-side Nostr post streaming enabled:

```bash
docker build \
  -t emerald-blog-v3:full-ssr \
  --build-arg BIN_FEATURES=ssr \
  --build-arg LIB_FEATURES=hydrate,client-side-nostr-fetch \
  .
```

Run it locally:

```bash
docker run --rm \
  -p 8080:8080 \
  -e RUST_LOG=info \
  -e LEPTOS_SITE_ADDR=0.0.0.0:8080 \
  emerald-blog-v3:full-ssr
```

Open `http://localhost:8080`.

## Docker Compose

Create `compose.full-ssr.yml`:

```yaml
services:
  emerald-blog:
    build:
      context: .
      args:
        BIN_FEATURES: ssr
        LIB_FEATURES: hydrate,client-side-nostr-fetch
    image: emerald-blog-v3:full-ssr
    ports:
      - "8080:8080"
    environment:
      RUST_LOG: info
      LEPTOS_SITE_ADDR: 0.0.0.0:8080
```

Start it:

```bash
docker compose -f compose.full-ssr.yml up --build
```

Stop it:

```bash
docker compose -f compose.full-ssr.yml down
```
