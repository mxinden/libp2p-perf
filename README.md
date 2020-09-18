# libp2p-perf

> **WARNING** alpha!

The [iPerf](https://en.wikipedia.org/wiki/Iperf) equivalent for the libp2p
ecosystem.

## Implementations

- Golang

- Rust


## Getting started

```bash
$ make


$ ./run.sh
# Start Rust and Golang servers.

# Rust -> Rust
Interval        Transfer        Bandwidth
0 s - 10.00 s   13338 MBytes    10670.28 MBit/s

# Rust -> Golang
Interval        Transfer        Bandwidth
0 s - 10.00 s   20365 MBytes    16292.00 MBit/s

# Golang -> Rust
Interval        Transfer        Bandwidth
0s - 10.00 s    16223 MBytes    12975.88 MBit/s

# Golang -> Golang
Interval        Transfer        Bandwidth
0s - 10.00 s    23166 MBytes    18532.68 MBit/s


$ cat /proc/cpuinfo | grep "model name" | head -1
model name      : Intel(R) Core(TM) i7-8550U CPU @ 1.80GHz
```
