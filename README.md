# libp2p-perf

> **WARNING** alpha!

The [iPerf](https://en.wikipedia.org/wiki/Iperf) equivalent for the libp2p
ecosystem.

In a nutshell a **client** sends as much data as possible through a single
multiplexed stream to a **server** which reads and discards all received data.
The client closes the stream after 10 seconds. Subsequently both the client and
the server print the result as the total number of bytes transferred and the
corresponding bandwidth on stdout.


## Implementations

- Golang

    - Transport: Tcp

    - Transport security: Noise or Plaintext

    - Multiplexing: Yamux

- Rust

    - Transport: Tcp

    - Transport security: Noise or Plaintext

    - Multiplexing: Yamux


## Getting started

```bash
$ make


$ ./run.sh
# Start Rust and Golang servers.

# Rust -> Rust

## Transport security noise
Interval        Transfer        Bandwidth
0 s - 10.00 s   2051 MBytes     1640.60 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0 s - 10.00 s   4070 MBytes     3255.99 MBit/s

# Rust -> Golang

## Transport security noise
Interval        Transfer        Bandwidth
0 s - 10.00 s   10897 MBytes    8716.59 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0 s - 10.00 s   16498 MBytes    13198.34 MBit/s

# Golang -> Rust

## Transport security noise
Interval        Transfer        Bandwidth
0s - 10.04 s    1980 MBytes     1577.01 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0s - 10.00 s    25149 MBytes    20117.33 MBit/s

# Golang -> Golang

## Transport security noise
Interval        Transfer        Bandwidth
0s - 10.00 s    9108 MBytes     7285.87 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0s - 10.00 s    26805 MBytes    21441.36 MBit/s


$ cat /proc/cpuinfo | grep "model name" | head -1
model name      : Intel(R) Core(TM) i7-8550U CPU @ 1.80GHz
```


## License

Licensed under either of

 * MIT license - <http://opensource.org/licenses/MIT>
 * Apache License, Version 2.0 - <http://www.apache.org/licenses/LICENSE-2.0>

at your option.
