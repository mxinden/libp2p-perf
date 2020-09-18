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
0 s - 10.00 s   11979 MBytes    9582.07 MBit/s

# Rust -> Golang
Interval        Transfer        Bandwidth
0 s - 10.00 s   21848 MBytes    17478.22 MBit/s

# Golang -> Rust
Interval        Transfer        Bandwidth
0s - 10.61 s    117 MBytes       88.21 MBit/s

# Golang -> Golang
Interval        Transfer        Bandwidth
0s - 10.00 s    22995 MBytes     18392.86 MBit/s


$ cat /proc/cpuinfo | grep "model name" | head -1
model name      : Intel(R) Core(TM) i7-8550U CPU @ 1.80GHz
```
