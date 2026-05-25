# core-js

Browser JavaScript bundle for Emerald Show.

It imports `@emerald-show/emerald-background`, initializes the emerald background,
and exposes `window.EmeraldBlogCore.initEmeraldBackground`.

Nostr publishing is handled by the hydrated Rust/WASM app. The JavaScript bundle
only uses `window.nostr` indirectly through Rust NIP-07 interop.

Build from the repository root:

```sh
pnpm install --frozen-lockfile
pnpm build:core-js
```

The Parcel build writes `public/emerald-core.js`.
