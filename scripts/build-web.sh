#!/usr/bin/env bash
set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
web_dir="$project_dir/crates/web_app"
client_dir="$project_dir/dist/client"
server_dir="$project_dir/dist/server"

rm -rf "$project_dir/dist"
mkdir -p "$client_dir" "$server_dir"

(
  cd "$web_dir"
  env -u RUST_LOG -u NO_COLOR trunk build index.html --release --dist "$client_dir"
)

cp "$project_dir/sites/worker.js" "$server_dir/index.js"
