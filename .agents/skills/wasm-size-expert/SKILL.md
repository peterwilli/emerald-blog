---
name: wasm-size-expert
description: Analyze, compare, and shrink Rust/WebAssembly artifacts. Use when Codex needs to inspect .wasm size, profile section/code size, verify release-build stripping/log removal, compare feature-flag builds, or recommend practical reductions for Leptos, wasm-bindgen, cargo-leptos, or other Rust WASM projects.
---

# WASM Size Expert

Use this skill to size release WASM artifacts, inspect where bytes are going, and propose shrink steps that match the project architecture.

## Workflow

1. Identify the release artifact. For this project, default to `target/site/pkg/emerald_blog.wasm`.
2. Build the same mode the user cares about. For Leptos release hydration, prefer:

```bash
cargo leptos build --release --lib-features hydrate,client-side-nostr-fetch
```

3. Run the bundled report:

```bash
.agents/skills/wasm-size-expert/scripts/wasm-size-report.sh target/site/pkg/emerald_blog.wasm
```

4. Inspect Code section function sizes when needed:

```bash
.agents/skills/wasm-size-expert/scripts/wasm-code-section.sh target/site/pkg/emerald_blog.wasm
```

5. If symbols are stripped, make a temporary analysis build with names preserved, then rebuild normally afterward:

```bash
CARGO_PROFILE_WASM_RELEASE_STRIP=none \
CARGO_PROFILE_WASM_RELEASE_DEBUG=0 \
cargo leptos build --release --lib-features hydrate,client-side-nostr-fetch
```

6. Check whether debug or diagnostic strings remain:

```bash
strings target/site/pkg/emerald_blog.wasm | rg -i 'debug|trace|tracing|console_error_panic_hook|log::|println'
```

Treat generic wasm-bindgen names such as `__wbindgen_debug_string`, HTTP method names such as `TRACE`, and panic/backtrace runtime strings as not app debug logging unless surrounding strings prove otherwise.

## Tooling

Prefer `wasm-tools`, maintained by Bytecode Alliance:

```bash
cargo install wasm-tools
wasm-tools objdump target/site/pkg/emerald_blog.wasm
wasm-tools demangle target/site/pkg/emerald_blog.wasm > /tmp/emerald.demangled.wasm
wasm-tools strip target/site/pkg/emerald_blog.wasm -o /tmp/emerald.stripped.wasm
```

Use `wasm-objdump` from WABT only for the detailed code-section listing requested by the user:

```bash
wasm-objdump -j Code -x target/site/pkg/emerald_blog.wasm
```

## Shrink Checklist

- Ensure release builds use `opt-level = "z"` or `"s"`, LTO, `codegen-units = 1`, `panic = "abort"`, and `strip = true` for the WASM profile where practical.
- Gate client-only diagnostics behind an opt-in feature so `tracing-wasm`, panic hooks, verbose logs, and debug callsites are absent from normal release hydration.
- Compare feature sets by rebuilding with and without the suspect feature, then measure raw and compressed deltas.
- Prefer moving heavy browser-only Rust stacks behind route features, server functions, lazy JS, or separate admin bundles when the public hydration bundle does not need them.
- Watch for common large contributors: Markdown parsers, crypto, relay/network stacks, URL/IDNA/ICU, JSON, generic UI/component monomorphization, and admin-only UI compiled into the public bundle.
- Verify any size win with a fresh release build and report raw bytes plus gzip or brotli bytes when available.

## Interpreting Results

Section sizes from `wasm-tools objdump` are exact for the binary. Per-dependency attribution is approximate in optimized Rust WASM because monomorphization, inlining, generated glue, and merged data sections obscure ownership.

For stripped release artifacts, function indices in `wasm-objdump -j Code -x` show which functions are largest but not their Rust names. Use a temporary name-preserving analysis build when symbol-level investigation is needed, and restore the normal stripped release build before reporting final shipped size.
