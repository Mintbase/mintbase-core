#!/usr/bin/env bash

# Testing script for quick iteration on code:
# - does quick checks first
# - reproduces CI pipeline

fail() {
  echo "$1 failed with code: $?"
  exit 1
}

cargo +nightly fmt || fail "Formatting"
cargo lint || fail "Linting"

cargo check -p mintbase-deps --features store-wasm --message-format short || fail "Checking store"
cargo check -p mintbase-deps --features factory-wasm --message-format short || fail "Checking factory"
cargo check -p mintbase-deps --features helper-wasm --message-format short || fail "Checking helper"
cargo check -p mintbase-deps --features market-wasm --message-format short || fail "Checking market"
cargo check -p mintbase-near-indexer || fail "Checking indexer"

cargo store-wasm || fail "Compiling store"
cargo factory-wasm || fail "Compiling factory"
cargo helper-wasm || fail "Compiling helper"
cargo market-wasm || fail "Compiling market"
cargo indexer || fail "Compiling indexer"

# Sandbox node is sometimes running in the background and causing problems
# -> kill sandbox manually in case I used it manually
killall near-sandbox
pkill near-sandbox

# (cd testing && npm test -- -m "approvals::core") || fail "Testing"
(cd testing && npm test) || fail "Testing"

# Be a good scripty-boy and clean up!
killall near-sandbox
pkill near-sandbox
