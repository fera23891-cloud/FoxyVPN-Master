#!/usr/bin/env bash
set -e
echo "Building FoxyVPN Master Core..."
cargo build --release --manifest-path Cargo.toml
echo "Done!"
