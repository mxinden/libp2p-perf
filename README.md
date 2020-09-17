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
0 s - 10.00 s   701 MBytes      560.79 MBit/s

# Rust -> Golang
Interval        Transfer        Bandwidth
0 s - 10.00 s   8489 MBytes     6789.88 MBit/s

# Golang -> Rust
Interval        Transfer        Bandwidth
0s - 10.10 s    107 MBytes       84.79 MBit/s

# Golang -> Golang
Interval        Transfer        Bandwidth
0s - 10.00 s    9020 MBytes      7215.98 MBit/s


$ cat /proc/cpuinfo | grep "model name" | head -1
model name      : Intel(R) Core(TM) i7-8550U CPU @ 1.80GHz
```
