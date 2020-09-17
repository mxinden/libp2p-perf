#!/usr/bin/env bash
# exit immediately when a command fails
set -e
# only exit with zero if all commands of the pipeline exit successfully
set -o pipefail
# error on unset variables
set -u

# Make sure to kill all background tasks when exiting.
trap "kill 0" EXIT

echo "# Start Rust and Golang servers."
./rust/target/release/server --listen-address /ip4/0.0.0.0/tcp/9992 &
./golang/go-libp2p-perf --listen-address /ip4/0.0.0.0/tcp/9993 &

echo ""
echo ""
echo "# Run Rust client against Rust server."
echo ""
./rust/target/release/client --server-address /ip4/127.0.0.1/tcp/9992

echo ""
echo ""
echo "# Run Rust client against Golang server."
echo ""
./rust/target/release/client --server-address /ip4/127.0.0.1/tcp/9993


