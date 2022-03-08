#!/usr/bin/env bash

fail() {
  echo "$1 failed with code: $?"
  exit 1
}

cargo +nightly fmt || fail "Formatting"
cargo lint || fail "Linting"

cargo store-wasm || fail "Compiling store"
cargo factory-wasm || fail "Compiling factory"
cargo helper-wasm || fail "Compiling helper"
cargo market-wasm || fail "Compiling market"
# TODO: check for compiling indexer before committing to repo
cargo check -p mintbase-near-indexer || fail "Checking indexer"

# (cd testing && npm test -- -m "approvals::core") || fail "Testing"
(cd testing && npm test) || fail "Testing"
killall near-sandbox # fuck this crap of the sandbox
pkill near-sandbox   # staying alive in the background
