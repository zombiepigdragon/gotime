#!/usr/bin/sh
# Run the various tests for this application. Mainly useful due to the exciting safety requirements.

# export GODEBUG=cgocheck=2
export GOEXPERIMENT=cgocheck2 # replacement for GODEBUG

export RUSTFLAGS=-Zsanitizer=address
export RUST_TEST_THREADS=1

cargo +nightly careful run
cargo +nightly careful test
