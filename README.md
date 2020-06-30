# libp2p-perf

> **WARNING** alpha!

The [iPerf](https://en.wikipedia.org/wiki/Iperf) equivalent for the libp2p
ecosystem.


## Getting started

```bash
# Start server
cargo run --bin server --release -- --listen-address /ip4/0.0.0.0/tcp/9992
```

```bash
# Start client
cargo run --bin client --release -- --server-address /ip4/127.0.0.1/tcp/9992
```
