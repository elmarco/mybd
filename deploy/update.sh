#!/usr/bin/env bash
set -euo pipefail

APP=mybd
REPO_DIR="/opt/$APP"
SERVICE="$APP.service"

cd "$REPO_DIR"

echo "==> Pulling latest changes..."
git pull --ff-only

echo "==> Building release..."
cargo leptos build --release

echo "==> Running migrations..."
cargo run -p data_tool --release -- populate

echo "==> Restarting service..."
systemctl --user restart "$SERVICE"

echo "==> Status:"
systemctl --user --no-pager status "$SERVICE"
