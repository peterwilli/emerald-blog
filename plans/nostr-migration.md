# Nostr Publishing Migration Plan

## Goal

Move browser-side Nostr publishing out of `core-js` and into the hydrated Rust/WASM app. Keep `window.nostr` as the NIP-07 signing provider only, and use the existing Rust `libnostrblog`/`nostr-sdk` client to publish signed events to relays.

This removes the duplicate relay-publishing stack currently split between:

- Rust/WASM: `src/app/client.rs` and hydrated post polling
- JavaScript: `core-js/src/nostr.js` using `nostr-tools` `SimplePool`

## Current State

- `src/app/client.rs` builds a `libnostrblog::Client`, adds relays, and connects it.
- Hydrated browser code already uses that client for polling posts in `src/app/components/post_list.rs`.
- `src/app/nostr_browser.rs` is a Rust wrapper, but it delegates publishing to `window.EmeraldBlogCore.*`.
- `core-js/src/nostr.js` currently handles event construction, NIP-07 signing, and relay publishing through `nostr-tools`.
- `window.nostr` is already the desired signing interface.

The WASM target currently compiles with the Nostr client stack:

```bash
cargo check --target wasm32-unknown-unknown --no-default-features --features hydrate
```

## Target Architecture

Publishing flow after migration:

1. Rust/WASM constructs an unsigned Nostr event.
2. Rust/WASM calls `window.nostr.getPublicKey()` when needed.
3. Rust/WASM passes the unsigned event object to `window.nostr.signEvent(...)`.
4. Rust/WASM parses the signed result into `libnostrblog::Event`.
5. Rust/WASM verifies the event ID and signature.
6. Rust/WASM publishes with `get_nostr_client().await.send_event(&event).await`.

`core-js` remains responsible for the emerald background package. It no longer needs to publish Nostr events or maintain relay connections.

## Implementation Steps

### 1. Add Native NIP-07 Interop In Rust

Update `src/app/nostr_browser.rs` to expose low-level helpers:

- `has_nostr_signer() -> bool`
- `get_nostr_public_key() -> Result<PublicKey, String>`
- `sign_unsigned_event(unsigned: UnsignedEvent) -> Result<Event, String>`

Implementation notes:

- Use `web_sys::window()` and `js_sys::Reflect` to read `window.nostr`.
- Validate `getPublicKey` and `signEvent` are callable.
- Convert Rust unsigned event data to a JS object or JSON-derived `JsValue`.
- Await NIP-07 promises with `wasm_bindgen_futures::JsFuture`.
- Parse the signed result through `serde_wasm_bindgen` if added, or through `js_sys::JSON::stringify` plus `Event::from_json`.
- Call `event.verify()` before publishing.

### 2. Replace JS Publishing Calls With Rust Publishing

Replace the current `call_core_promise(...)` publishing path in `src/app/nostr_browser.rs`.

Each public function should construct, sign, validate, and publish in Rust:

- `publish_nostr_post(PostDraft)`
- `publish_comment(post_id, content)`
- `publish_comment_approval(comment_id)`
- `publish_post_deletion(post_id, reason, expected_author)`

Use the same event shapes currently generated in `core-js/src/nostr.js`:

- Blog post: kind `1`, tags `title`, optional `summary`, optional `image`, repeated `t`
- Comment: kind `1111`, tags `E`, `K`, `e`, `k`
- Comment approval: kind `1985`, tags `e`, `L`, `l`
- Deletion: kind `5`, tags `e`, `k`

### 3. Share Event Construction With Existing Rust Code Where Practical

Prefer existing `nostr-sdk` and `libnostrblog` builders:

- Use `EventBuilder` when it can express the event cleanly.
- Use `Tag::parse(...)` for custom tag arrays.
- Use `UnsignedEvent::new(...)` when direct construction is clearer.
- Use existing helpers from `libnostrblog::blog::comments` and deletion modules if their visibility and behavior match the browser workflow.

Avoid introducing a broad abstraction until the four publishing paths are migrated and duplication is visible.

### 4. Preserve Safety Checks

For signed events returned by NIP-07:

- Verify event ID and signature before sending.
- Ensure deletion events are signed by `expected_author` when supplied.
- Surface signer errors distinctly from relay publish errors where possible.
- Keep the existing user-facing messages mostly stable.

### 5. Remove JS Relay Publishing

Once Rust publishing works:

- Remove `core-js/src/nostr.js` publishing functions or reduce them to compatibility wrappers calling Rust exports if needed.
- Remove `nostr-tools` from `core-js/package.json` if no other code uses it.
- Remove generated relay module and generator if no JS code consumes relays.
- Remove `core-js/scripts/generate-nostr-relays.mjs`.
- Simplify `core-js/src/index.js` exports to background-only behavior, unless a compatibility surface is intentionally retained.

### 6. Revisit `shell` Script Ordering

`src/app/mod.rs` currently loads `/emerald-core.js` before hydration scripts.

After migration:

- Confirm `emerald-core.js` is no longer required for Nostr publishing.
- Keep it for the emerald background if still needed.
- Ensure NIP-07 signing calls only occur after island hydration and user interaction.

### 7. Verification

Run:

```bash
cargo fmt
cargo check
cargo check --target wasm32-unknown-unknown --no-default-features --features hydrate
pnpm build:core-js
```

For browser validation:

- Open the admin post page with a NIP-07 extension available.
- Publish a draft post and confirm it appears on configured relays.
- Submit a signed comment and confirm it enters moderation.
- Approve a comment from the admin moderation tab.
- Delete a post with the expected-author guard enabled.
- Confirm post polling still works after publishing.

## Risks And Decisions

- `serde_wasm_bindgen` may be worth adding to avoid brittle manual JS object conversion. If added, keep it scoped to hydrate-side NIP-07 interop.
- `nostr-sdk` client lifetime should remain singleton-backed through `get_nostr_client()` so browser polling and publishing share relay connections.
- If `Event::verify()` depends on `std` in a way that becomes awkward in WASM, use `verify_with_ctx` with an explicit `Secp256k1` context.
- If NIP-07 extensions mutate event fields unexpectedly, reject the signed result unless it matches the intended kind/content/tags constraints.

## Cleanup Criteria

The migration is complete when:

- Browser publishing succeeds without `nostr-tools`.
- `window.nostr` is used only for signer detection, public key lookup, and signing.
- Rust/WASM owns relay publishing for posts, comments, moderation approvals, and deletions.
- The hydrated app uses one Nostr client stack for polling and publishing.
- Core JS no longer contains relay defaults or relay publishing code.
