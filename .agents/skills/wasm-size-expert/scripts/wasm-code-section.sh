#!/usr/bin/env bash
set -euo pipefail

wasm="${1:-target/site/pkg/emerald_blog.wasm}"
top_n="${2:-40}"

if [[ ! -f "$wasm" ]]; then
  echo "WASM file not found: $wasm" >&2
  exit 1
fi

if ! command -v wasm-objdump >/dev/null 2>&1; then
  echo "wasm-objdump is required for Code section details. Install WABT for your platform." >&2
  exit 1
fi

tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT

wasm-objdump -j Code -x "$wasm" > "$tmp"

code_count=$(awk -F'[][]' '/^Code\[[0-9]+\]:/ { print $2 }' "$tmp")
code_bytes=$(awk -F'size=' '/ - func\[[0-9]+\] size=/ { sum += $2 } END { print sum + 0 }' "$tmp")

printf 'file: %s\n' "$wasm"
printf 'code functions: %s\n' "${code_count:-unknown}"
printf 'sum of function body sizes: %s bytes (%.2f MiB)\n' "$code_bytes" "$(awk -v b="$code_bytes" 'BEGIN { print b / 1024 / 1024 }')"

echo
printf 'top %s functions by body size:\n' "$top_n"
awk '/ - func\[[0-9]+\] size=/ {
  line = $0
  gsub(/^ - /, "", line)
  split(line, parts, " size=")
  print parts[2] "\t" parts[1]
}' "$tmp" | sort -nr | awk -F'\t' -v limit="$top_n" 'count < limit {
  printf "%10d bytes  %s\n", $1, $2
  count++
}'

echo
echo 'raw wasm-objdump command:'
printf 'wasm-objdump -j Code -x %q\n' "$wasm"
