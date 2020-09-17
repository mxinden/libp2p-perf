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
./rust/target/release/server --listen-address /ip4/0.0.0.0/tcp/9992 --private-key-pkcs8 rust/test.pk8 > /dev/null 2>&1 &
./golang/go-libp2p-perf --fake-crypto-seed --listen-address /ip4/0.0.0.0/tcp/9993 > /dev/null 2>&1 &

sleep 1

echo ""
echo "# Rust -> Rust"
./rust/target/release/client --server-address /ip4/127.0.0.1/tcp/9992

echo ""
echo "# Rust -> Golang"
./rust/target/release/client --server-address /ip4/127.0.0.1/tcp/9993


echo ""
echo "# Golang -> Rust"
./golang/go-libp2p-perf --server-address /ip4/127.0.0.1/tcp/9992/p2p/Qmcqq9TFaYbb94uwdER1BXyGfCFY4Bb1gKozxNyVvLvTSw


echo ""
echo "# Golang -> Golang"
./golang/go-libp2p-perf --server-address /ip4/127.0.0.1/tcp/9993/p2p/12D3KooWL3XJ9EMCyZvmmGXL2LMiVBtrVa2BuESsJiXkSj7333Jw

