#!/usr/bin/env bash

fail() {
  echo "$1 failed with code: $?"
  exit 1
}

build_wasm() {
  cargo "$1-wasm" || fail "Compiling $1"
  wasm-opt "wasm/$1.wasm" -Oz \
    -o "wasm/$1-opt.wasm" || fail "Minifying $1"
  # mv "wasm/$1-opt.wasm" "wasm/$1.wasm"
}

kill_the_damn_sandbox() {
  killall near-sandbox >/dev/null 2>&1
  pkill near-sandbox >/dev/null 2>&1
}

cargo +nightly fmt || fail "Formatting"
cargo lint || fail "Linting"

# prevent factory checking from failing
touch wasm/store.wasm

cargo check -p mintbase-deps --features store-wasm --message-format short || fail "Checking store"
cargo check -p mintbase-deps --features factory-wasm --message-format short || fail "Checking factory"
cargo check -p mintbase-deps --features helper-wasm --message-format short || fail "Checking helper"
cargo check -p simple-market-contract --message-format short || fail "Checking market"
cargo check -p mintbase-near-indexer --bin mintlake --features mintlake || fail "Checking mintlake"
cargo check -p mintbase-near-indexer --bin p2p_indexer --features p2p_indexer || fail "Checking p2p indexer"

build_wasm store
build_wasm factory
build_wasm helper
build_wasm market

# Sandbox node is sometimes running in the background and causing problems
# -> kill sandbox in case I used it manually
kill_the_damn_sandbox

# Limit to 6 parallel tests to prevent hiccups with the key store
# Doesn"t feel like it helps though.
(cd testing && npm test -- -c 6 --fail-fast) || {
  kill_the_damn_sandbox
  fail "Testing"
}

# Yup, the sandbox can be quite obnoxious at times
kill_the_damn_sandbox

cargo p2p_indexer || fail "Compiling p2p indexer"
(cd mintbase-near-indexer && ./scripts/test-p2p.sh) || fail "Testing indexer (local)"
cargo mintlake || fail "Compiling mintlake"
(cd mintbase-near-indexer && ./scripts/test-lake.sh) || fail "Testing indexer (testnet)"
