#!/usr/bin/env bash

# Testing script for quick iteration on code:
# - does quick checks first
# - reproduces CI pipeline

fail() {
  echo "$1 failed with code: $?"
  exit 1
}

build_wasm() {
  cargo "$1-wasm" || fail "Compiling $1"
  # wasm-opt "wasm/$1.wasm" -Oz \
  #   -o "wasm/$1-opt.wasm" || fail "Minifying $1"
  # mv "wasm/$1-opt.wasm" "wasm/$1.wasm"
}

kill_the_damn_sandbox() {
  killall near-sandbox
  pkill near-sandbox
}

cargo +nightly fmt || fail "Formatting"
cargo lint || fail "Linting"

# prevent factory checking from failing
touch wasm/store.wasm

cargo check -p mintbase-deps --features store-wasm --message-format short || fail "Checking store"
cargo check -p mintbase-deps --features factory-wasm --message-format short || fail "Checking factory"
cargo check -p mintbase-deps --features helper-wasm --message-format short || fail "Checking helper"
cargo check -p simple-market-contract --message-format short || fail "Checking market"
cargo check -p mintbase-near-indexer || fail "Checking indexer"

build_wasm store
build_wasm factory
build_wasm helper
build_wasm market
cargo indexer || fail "Compiling indexer"

# Sandbox node is sometimes running in the background and causing problems
# -> kill sandbox in case I used it manually
kill_the_damn_sandbox

# (cd testing && npm test -- -m "approvals::core") || fail "Testing"
(cd testing && npm test) || {
  kill_the_damn_sandbox
  fail "Testing"
}

# Be a good scripty-boy and clean up!
kill_the_damn_sandbox
