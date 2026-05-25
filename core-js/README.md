# core-js

Browser JavaScript bundle for Emerald Show.

It imports `@emerald-show/emerald-background`, initializes the emerald background,
and exposes Nostr publishing helpers for Rust/WASM callers:

```js
window.EmeraldBlogCore.publishNostrPost(draft)
window.EmeraldBlogCore.publishComment(postId, content)
window.EmeraldBlogCore.publishCommentApproval(commentId)
window.EmeraldBlogCore.publishPostDeletion(postId, reason, expectedAuthor)
window.EmeraldBlogCore.hasNostrSigner()
```

Build from the repository root:

```sh
pnpm install --frozen-lockfile
pnpm build:core-js
```

The Parcel build writes `public/emerald-core.js`.
