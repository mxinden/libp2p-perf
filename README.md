# libp2p-perf

**Archived. Use https://github.com/libp2p/test-plans/tree/master/perf instead.**

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
0 s - 10.00 s   7713 MBytes     6169.19 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0 s - 10.00 s   11703 MBytes    9362.40 MBit/s

# Rust -> Golang

## Transport security noise
Interval        Transfer        Bandwidth
0 s - 10.00 s   7603 MBytes     6080.17 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0 s - 10.00 s   22171 MBytes    17736.72 MBit/s

# Golang -> Rust

## Transport security noise
Interval        Transfer        Bandwidth
0s - 10.01 s    7919 MBytes     6331.07 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0s - 10.01 s    20871 MBytes    16682.93 MBit/s

# Golang -> Golang

## Transport security noise
Interval        Transfer        Bandwidth
0s - 10.00 s    8051 MBytes     6438.80 MBit/s

## Transport security plaintext
Interval        Transfer        Bandwidth
0s - 10.00 s    25222 MBytes    20177.50 MBit/s

$ cat /proc/cpuinfo | grep "model name" | head -1
model name      : Intel(R) Core(TM) i7-8550U CPU @ 1.80GHz
```


## License

Licensed under either of

 * MIT license - <http://opensource.org/licenses/MIT>
 * Apache License, Version 2.0 - <http://www.apache.org/licenses/LICENSE-2.0>

at your option.
