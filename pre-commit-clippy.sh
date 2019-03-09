#!/usr/bin/env sh

set -e

cd rust/
cargo clippy --all-targets --all-features -- -D warnings
