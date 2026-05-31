#!/usr/bin/env bash
set -euo pipefail

wasm="${1:-target/site/pkg/emerald_blog.wasm}"

if [[ ! -f "$wasm" ]]; then
  echo "WASM file not found: $wasm" >&2
  exit 1
fi

if ! command -v wasm-tools >/dev/null 2>&1; then
  echo "wasm-tools is required. Install with: cargo install wasm-tools" >&2
  exit 1
fi

bytes=$(wc -c < "$wasm" | tr -d ' ')
printf 'file: %s\n' "$wasm"
printf 'raw:  %s bytes (%.2f MiB)\n' "$bytes" "$(awk -v b="$bytes" 'BEGIN { print b / 1024 / 1024 }')"

if command -v gzip >/dev/null 2>&1; then
  gzip_bytes=$(gzip -9 -c "$wasm" | wc -c | tr -d ' ')
  printf 'gzip: %s bytes (%.2f MiB)\n' "$gzip_bytes" "$(awk -v b="$gzip_bytes" 'BEGIN { print b / 1024 / 1024 }')"
fi

if command -v brotli >/dev/null 2>&1; then
  brotli_bytes=$(brotli -q 11 -c "$wasm" | wc -c | tr -d ' ')
  printf 'brotli: %s bytes (%.2f MiB)\n' "$brotli_bytes" "$(awk -v b="$brotli_bytes" 'BEGIN { print b / 1024 / 1024 }')"
fi

echo
echo 'sections:'
wasm-tools objdump "$wasm" --color never | awk -F'|' -v total="$bytes" '
  NF >= 4 {
    name = $1
    size_field = $3
    count_field = $4
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", name)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", size_field)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", count_field)
    split(size_field, size_parts, " ")
    split(count_field, count_parts, " ")
    size = size_parts[1] + 0
    count = count_parts[1]
    pct = total > 0 ? (size * 100 / total) : 0
    printf "%10d bytes  %6.2f%%  %8s items  %s\n", size, pct, count, name
  }
'

echo
echo 'custom/name sections:'
wasm-tools objdump "$wasm" --color never | rg 'custom|name|producers' || true

echo
echo 'top strings that may indicate debug/logging:'
strings "$wasm" | rg -i 'debug|trace|tracing|console_error_panic_hook|log::|println' | head -40 || true
