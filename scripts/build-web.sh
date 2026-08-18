#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
web_dir="$project_dir/crates/web_app"
client_dir="$project_dir/dist/client"
server_dir="$project_dir/dist/server"
embed_dir="$project_dir/dist/embed"

rm -rf "$project_dir/dist"
mkdir -p "$client_dir" "$server_dir" "$embed_dir"

(
  cd "$web_dir"
  env -u RUST_LOG -u NO_COLOR trunk build index.html --release --dist "$client_dir"
)

cp "$project_dir/sites/worker.js" "$server_dir/index.js"
cp "$client_dir/web_app.js" "$embed_dir/web_app.js"
cp "$client_dir/web_app_bg.wasm" "$embed_dir/web_app_bg.wasm"
cp "$client_dir/murmuration-background.js" "$embed_dir/murmuration-background.js"
