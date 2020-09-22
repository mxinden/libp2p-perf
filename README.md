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

## Transport protocol noise
Interval        Transfer        Bandwidth
0 s - 10.06 s   1862 MBytes     1480.67 MBit/s

## Transport protocol plaintext
Interval        Transfer        Bandwidth
0 s - 10.00 s   10267 MBytes    8209.56 MBit/s

# Rust -> Golang

## Transport protocol noise
Interval        Transfer        Bandwidth
0 s - 10.00 s   7740 MBytes     6191.96 MBit/s

## Transport protocol plaintext
Interval        Transfer        Bandwidth
0 s - 10.00 s   17588 MBytes    14070.39 MBit/s

# Golang -> Rust

## Transport protocol noise
Interval        Transfer        Bandwidth
0s - 10.03 s    2349 MBytes     1873.96 MBit/s

## Transport protocol plaintext
Interval        Transfer        Bandwidth
0s - 10.00 s    17213 MBytes    13768.89 MBit/s

# Golang -> Golang

## Transport protocol noise
Interval        Transfer        Bandwidth
0s - 10.00 s    5983 MBytes     4786.30 MBit/s

## Transport protocol plaintext
Interval        Transfer        Bandwidth
0s - 10.00 s    21399 MBytes    17115.38 MBit/s


$ cat /proc/cpuinfo | grep "model name" | head -1
model name      : Intel(R) Core(TM) i7-8550U CPU @ 1.80GHz
```
