# memc-kv

A simple memcached like in memory kv implemented in Rust, for demo and learning purpose (if it won't
get more useful later...)

Maybe compatible with memcached ascii protocol on commands:

- [x] `set <key> <flag> <ttl> <len> (noreply)`
- [x] `get <key>`

```
# using libmemcached's memcapable to check protocal compatibility
./clients/memcapable -h 127.0.0.1 -p 6001 -a
ascii version                           [pass]
ascii set                               [pass]
ascii set noreply                       [pass]
ascii get                               [pass]
```

Code is not fully tested, but you are welcome to add new features and tests or provide new ideas and
feedbacks.

## Implementation

Inspired by [memc-rs](https://github.com/memc-rs/memc-rs), it seems not too difficult to implement a
memcached ascii protocol compatible server for simple get/set command with the help of modern Rust
friends, including

- [tokio](https://tokio.rs/tokio/tutorial) - An asynchronous runtime for the Rust programming
  language which is perfect for handling bytes read and write on tcp ports (networking applications)
- [dashmap](https://github.com/xacrimon/dashmap) - Blazingly fast concurrent map in Rust
- [nom](https://github.com/Geal/nom) - for fast and easy parsing memcached commands

## So why doing this

Besides learning purpose, I think this may bring some extra value or potential as the design is very
simple here. Plus using a modern language like Rust, it should be quite easy to add more features
such as Http/gPRC endpoints or even making it can run a distributed fashion which support
leader-follower data replications. In the same time we probably won't be that optimized comparing
with original C implementation but `memc-kv` should be able to run fairly fast as well.

## How to start `memc-kv` locally

```
RUST_LOG=trace cargo run --release
```

## Reference links

- [memcached protocol](https://github.com/memcached/memcached/blob/master/doc/protocol.txt)
- [memcached cheatsheet](https://lzone.de/cheat-sheet/memcached)
- [tokio mini-redis code example](https://github.com/tokio-rs/mini-redis/blob/tutorial/src/frame.rs#L254-L262)
- [tokio docs](https://docs.rs/tokio/1.17.0/tokio/io/trait.AsyncReadExt.html#method.read)
- [libmemcached](https://launchpad.net/libmemcached) installation to use `memcapable` for protocol
  compatibility check
    - [download link](https://launchpad.net/libmemcached/1.0/1.0.18/+download/libmemcached-1.0.18.tar.gz)
    - [memcapable bug fix](https://bugs.launchpad.net/libmemcached/+bug/1481057)
    - [compile tips for mac](https://stackoverflow.com/questions/27004144/how-can-i-install-libmemcached-for-mac-os-x-yosemite-10-10-in-order-to-install-t)
      ```
      So with libmemcached-1.0.18, we need these changes on mac:
      
      On line 2153 of clients/memcapable.cc
      `if (hostname)` -> `if (!hostname)`
      
      In file configure
      `ac_cv_have_htonll=yes` -> `ac_cv_have_htonll=no`
      
      In file clients/memflush.cc, change 2 places
      `if (opt_servers == false)` -> `if (opt_servers == NULL)`
      ```

## To do list

- [ ] Add a LRU (least recent update) eviction policy
- [ ] Keep track the total key and value memory usage
- [ ] Adding an HTTP server
- [ ] Adding a gRPC server

## Benchmark memcached

```
docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=6001 \
    --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8 \
    --distinct-client-seed --randomize \
    --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 20 -t 8

docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=11211 \
    --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8 \
    --distinct-client-seed --randomize \
    --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 20 -t 8
```

One way to start memcached is via docker, however on my Mac it seems the application running in
docker is a bit slow.

```
docker run --name mc -d --rm -p 11211:11211 memcached memcached -m 1024
```

Or using a local running one

```
/opt/homebrew/opt/memcached/bin/memcached -l localhost -m 1024 --thread=8
```

### Benchmark details (preliminary):

| impl        | platform            | `set P99` | `get P99` |     `ops/sec` | server thread number | test thread/connection number |
|-------------|---------------------|----------:|----------:|--------------:|---------------------:|------------------------------:|
| `memc-kv`   | on M1 Air           |    17.0ms |    17.0ms |          9660 |                    ? |                          4/20 |
| `memcached` | on M1 Air           |     8.9ms |     8.7ms |         15187 |                    4 |                          4/20 |
| `memcached` | in docker on M1 Air |    30.0ms |    30.0ms |          4229 |                    4 |                          4/20 |
| `memc-kv`   | on M1 Max           |    18.0ms |    18.0ms |         15449 |                    4 |                          8/20 |
| `memcached` | on M1 Max           |    13.0ms |    14.0ms |         23078 |                    4 |                          8/20 |
| `memc-kv`   | on M1 Max           |    18.0ms |    19.0ms |         15418 |                    8 |                          8/20 |
| `memcached` | on M1 Max           |    12.0ms |    12.0ms |         23232 |                    8 |                          8/20 |


<details>
  <summary><strong>memc-kv</strong> running locally on a Macbook Air M1</summary>

```
docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=6001 \
>     --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8 \
>     --distinct-client-seed --randomize \
>     --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 10 -t 4
WARNING: The requested image's platform (linux/amd64) does not match the detected host platform (linux/arm64/v8) and no specific platform was requested
[RUN #1] Preparing benchmark client...
[RUN #1] Launching threads now...
[RUN #1 100%,  40 secs]  0 threads:      400000 ops,   10154 (avg:    9763) ops/sec, 2.44MB/sec (avg: 1.86MB/sec),  3.92 (avg:  4.07) msec latency

4         Threads
10        Connections per thread
10000     Requests per client


ALL STATS
=========================================================================
Type         Ops/sec     Hits/sec   Misses/sec      Latency       KB/sec 
-------------------------------------------------------------------------
Sets         3222.86          ---          ---      4.07800       947.64 
Gets         6437.99      2876.49      3561.49      4.06100       940.82 
Waits           0.00          ---          ---      0.00000          --- 
Totals       9660.84      2876.49      3561.49      4.06700      1888.46 


Request Latency Distribution
Type     <= msec         Percent
------------------------------------------------------------------------
SET       0.270         0.00
SET       0.310         0.00
SET       0.320         0.00
SET       0.370         0.01
SET       0.380         0.01
SET       0.400         0.01
SET       0.410         0.01
SET       0.430         0.01
SET       0.450         0.01
SET       0.460         0.01
SET       0.510         0.01
SET       0.520         0.01
SET       0.530         0.02
SET       0.550         0.02
SET       0.560         0.02
SET       0.570         0.02
SET       0.580         0.02
SET       0.590         0.02
SET       0.600         0.03
SET       0.610         0.03
SET       0.620         0.03
SET       0.630         0.03
SET       0.640         0.03
SET       0.650         0.04
SET       0.660         0.04
SET       0.670         0.04
SET       0.680         0.04
SET       0.690         0.04
SET       0.700         0.04
SET       0.710         0.05
SET       0.720         0.05
SET       0.730         0.05
SET       0.740         0.06
SET       0.750         0.06
SET       0.760         0.06
SET       0.770         0.06
SET       0.780         0.07
SET       0.790         0.07
SET       0.800         0.07
SET       0.810         0.08
SET       0.820         0.08
SET       0.830         0.09
SET       0.840         0.10
SET       0.850         0.10
SET       0.860         0.11
SET       0.870         0.11
SET       0.880         0.12
SET       0.890         0.13
SET       0.900         0.13
SET       0.910         0.14
SET       0.920         0.15
SET       0.930         0.16
SET       0.940         0.17
SET       0.950         0.18
SET       0.960         0.19
SET       0.970         0.21
SET       0.980         0.22
SET       0.990         0.23
SET       1.000         0.31
SET       1.100         0.48
SET       1.200         0.74
SET       1.300         1.00
SET       1.400         1.33
SET       1.500         1.73
SET       1.600         2.28
SET       1.700         2.94
SET       1.800         3.72
SET       1.900         4.79
SET       2.000         6.25
SET       2.100         7.94
SET       2.200         9.81
SET       2.300        11.96
SET       2.400        14.36
SET       2.500        17.19
SET       2.600        20.20
SET       2.700        23.39
SET       2.800        26.88
SET       2.900        30.48
SET       3.000        34.47
SET       3.100        38.43
SET       3.200        42.33
SET       3.300        46.09
SET       3.400        49.94
SET       3.500        53.63
SET       3.600        57.22
SET       3.700        60.59
SET       3.800        63.73
SET       3.900        66.90
SET       4.000        69.84
SET       4.100        72.40
SET       4.200        74.66
SET       4.300        76.68
SET       4.400        78.50
SET       4.500        80.08
SET       4.600        81.52
SET       4.700        82.83
SET       4.800        84.00
SET       4.900        85.10
SET       5.000        86.07
SET       5.100        86.85
SET       5.200        87.50
SET       5.300        88.08
SET       5.400        88.60
SET       5.500        89.02
SET       5.600        89.46
SET       5.700        89.87
SET       5.800        90.24
SET       5.900        90.57
SET       6.000        90.91
SET       6.100        91.16
SET       6.200        91.41
SET       6.300        91.63
SET       6.400        91.83
SET       6.500        92.02
SET       6.600        92.18
SET       6.700        92.36
SET       6.800        92.52
SET       6.900        92.70
SET       7.000        92.86
SET       7.100        93.01
SET       7.200        93.16
SET       7.300        93.27
SET       7.400        93.40
SET       7.500        93.52
SET       7.600        93.64
SET       7.700        93.75
SET       7.800        93.86
SET       7.900        93.96
SET       8.000        94.08
SET       8.100        94.18
SET       8.200        94.27
SET       8.300        94.37
SET       8.400        94.48
SET       8.500        94.58
SET       8.600        94.67
SET       8.700        94.76
SET       8.800        94.86
SET       8.900        94.95
SET       9.000        95.05
SET       9.100        95.14
SET       9.200        95.23
SET       9.300        95.32
SET       9.400        95.39
SET       9.500        95.47
SET       9.600        95.55
SET       9.700        95.65
SET       9.800        95.72
SET       9.900        95.80
SET      10.000        96.20
SET      11.000        96.88
SET      12.000        97.40
SET      13.000        97.84
SET      14.000        98.24
SET      15.000        98.57
SET      16.000        98.82
SET      17.000        99.08
SET      18.000        99.25
SET      19.000        99.41
SET      20.000        99.54
SET      21.000        99.64
SET      22.000        99.70
SET      23.000        99.76
SET      24.000        99.81
SET      25.000        99.86
SET      26.000        99.88
SET      27.000        99.90
SET      28.000        99.93
SET      29.000        99.94
SET      30.000        99.96
SET      31.000        99.97
SET      32.000        99.98
SET      33.000        99.99
SET      34.000        99.99
SET      35.000        99.99
SET      36.000        99.99
SET      37.000        99.99
SET      38.000        99.99
SET      40.000       100.00
SET      41.000       100.00
SET      42.000       100.00
SET      46.000       100.00
---
GET       0.290         0.00
GET       0.300         0.00
GET       0.310         0.00
GET       0.320         0.00
GET       0.340         0.00
GET       0.350         0.00
GET       0.360         0.00
GET       0.370         0.00
GET       0.380         0.00
GET       0.390         0.00
GET       0.400         0.01
GET       0.410         0.01
GET       0.420         0.01
GET       0.430         0.01
GET       0.440         0.01
GET       0.470         0.01
GET       0.490         0.01
GET       0.500         0.01
GET       0.510         0.01
GET       0.520         0.01
GET       0.530         0.01
GET       0.540         0.01
GET       0.550         0.01
GET       0.560         0.01
GET       0.570         0.01
GET       0.580         0.02
GET       0.590         0.02
GET       0.600         0.02
GET       0.610         0.02
GET       0.620         0.02
GET       0.630         0.02
GET       0.640         0.02
GET       0.650         0.03
GET       0.660         0.03
GET       0.670         0.03
GET       0.680         0.03
GET       0.690         0.03
GET       0.700         0.04
GET       0.710         0.04
GET       0.720         0.04
GET       0.730         0.04
GET       0.740         0.05
GET       0.750         0.05
GET       0.760         0.05
GET       0.770         0.05
GET       0.780         0.06
GET       0.790         0.07
GET       0.800         0.07
GET       0.810         0.07
GET       0.820         0.08
GET       0.830         0.08
GET       0.840         0.09
GET       0.850         0.10
GET       0.860         0.10
GET       0.870         0.11
GET       0.880         0.12
GET       0.890         0.13
GET       0.900         0.13
GET       0.910         0.14
GET       0.920         0.15
GET       0.930         0.15
GET       0.940         0.17
GET       0.950         0.18
GET       0.960         0.19
GET       0.970         0.21
GET       0.980         0.22
GET       0.990         0.23
GET       1.000         0.33
GET       1.100         0.52
GET       1.200         0.73
GET       1.300         0.98
GET       1.400         1.29
GET       1.500         1.69
GET       1.600         2.19
GET       1.700         2.86
GET       1.800         3.70
GET       1.900         4.81
GET       2.000         6.28
GET       2.100         7.97
GET       2.200         9.81
GET       2.300        12.02
GET       2.400        14.48
GET       2.500        17.23
GET       2.600        20.28
GET       2.700        23.48
GET       2.800        26.92
GET       2.900        30.56
GET       3.000        34.67
GET       3.100        38.66
GET       3.200        42.54
GET       3.300        46.42
GET       3.400        50.17
GET       3.500        53.88
GET       3.600        57.38
GET       3.700        60.78
GET       3.800        63.99
GET       3.900        67.16
GET       4.000        70.20
GET       4.100        72.79
GET       4.200        75.01
GET       4.300        77.00
GET       4.400        78.82
GET       4.500        80.37
GET       4.600        81.77
GET       4.700        83.07
GET       4.800        84.21
GET       4.900        85.26
GET       5.000        86.21
GET       5.100        87.01
GET       5.200        87.62
GET       5.300        88.20
GET       5.400        88.70
GET       5.500        89.15
GET       5.600        89.59
GET       5.700        89.96
GET       5.800        90.29
GET       5.900        90.61
GET       6.000        90.93
GET       6.100        91.19
GET       6.200        91.44
GET       6.300        91.66
GET       6.400        91.86
GET       6.500        92.06
GET       6.600        92.25
GET       6.700        92.42
GET       6.800        92.58
GET       6.900        92.74
GET       7.000        92.90
GET       7.100        93.05
GET       7.200        93.19
GET       7.300        93.31
GET       7.400        93.43
GET       7.500        93.57
GET       7.600        93.71
GET       7.700        93.82
GET       7.800        93.93
GET       7.900        94.05
GET       8.000        94.18
GET       8.100        94.30
GET       8.200        94.40
GET       8.300        94.50
GET       8.400        94.59
GET       8.500        94.70
GET       8.600        94.80
GET       8.700        94.89
GET       8.800        94.96
GET       8.900        95.05
GET       9.000        95.14
GET       9.100        95.24
GET       9.200        95.32
GET       9.300        95.39
GET       9.400        95.47
GET       9.500        95.55
GET       9.600        95.63
GET       9.700        95.71
GET       9.800        95.78
GET       9.900        95.86
GET      10.000        96.27
GET      11.000        96.92
GET      12.000        97.47
GET      13.000        97.91
GET      14.000        98.31
GET      15.000        98.65
GET      16.000        98.91
GET      17.000        99.12
GET      18.000        99.30
GET      19.000        99.45
GET      20.000        99.57
GET      21.000        99.65
GET      22.000        99.71
GET      23.000        99.77
GET      24.000        99.82
GET      25.000        99.86
GET      26.000        99.90
GET      27.000        99.92
GET      28.000        99.94
GET      29.000        99.96
GET      30.000        99.97
GET      31.000        99.97
GET      32.000        99.98
GET      33.000        99.98
GET      34.000        99.99
GET      35.000        99.99
GET      36.000        99.99
GET      37.000        99.99
GET      38.000       100.00
GET      39.000       100.00
GET      41.000       100.00
GET      45.000       100.00
GET      46.000       100.00
GET      47.000       100.00
GET      48.000       100.00
---
```

</details>

<details>
  <summary><strong>memcached</strong> running locally with `-m 1024` on a Macbook Air M1</summary>

```
docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=11211     --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8     --distinct-client-seed --randomize     --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 10 -t 4
WARNING: The requested image's platform (linux/amd64) does not match the detected host platform (linux/arm64/v8) and no specific platform was requested
[RUN #1] Preparing benchmark client...
[RUN #1] Launching threads now...
[RUN #1 100%,  25 secs]  0 threads:      400000 ops,   16233 (avg:   15881) ops/sec, 3.86MB/sec (avg: 3.03MB/sec),  2.44 (avg: 66.92) msec latencycy

4         Threads
10        Connections per thread
10000     Requests per client


ALL STATS
=========================================================================
Type         Ops/sec     Hits/sec   Misses/sec      Latency       KB/sec 
-------------------------------------------------------------------------
Sets         5066.71          ---          ---      2.51200      1486.67 
Gets        10121.28      4526.29      5594.99     99.16600      1478.46 
Waits           0.00          ---          ---      0.00000          --- 
Totals      15187.99      4526.29      5594.99     66.92200      2965.13 


Request Latency Distribution
Type     <= msec         Percent
------------------------------------------------------------------------
SET       0.290         0.00
SET       0.300         0.00
SET       0.320         0.00
SET       0.350         0.00
SET       0.370         0.01
SET       0.390         0.01
SET       0.400         0.01
SET       0.410         0.01
SET       0.420         0.01
SET       0.440         0.01
SET       0.450         0.01
SET       0.460         0.01
SET       0.470         0.02
SET       0.480         0.02
SET       0.490         0.02
SET       0.500         0.02
SET       0.510         0.02
SET       0.520         0.03
SET       0.530         0.03
SET       0.540         0.03
SET       0.550         0.03
SET       0.560         0.03
SET       0.570         0.03
SET       0.580         0.04
SET       0.590         0.04
SET       0.600         0.05
SET       0.610         0.05
SET       0.620         0.06
SET       0.630         0.06
SET       0.640         0.07
SET       0.650         0.09
SET       0.660         0.10
SET       0.670         0.10
SET       0.680         0.11
SET       0.690         0.13
SET       0.700         0.14
SET       0.710         0.15
SET       0.720         0.18
SET       0.730         0.20
SET       0.740         0.21
SET       0.750         0.24
SET       0.760         0.25
SET       0.770         0.28
SET       0.780         0.29
SET       0.790         0.33
SET       0.800         0.35
SET       0.810         0.39
SET       0.820         0.42
SET       0.830         0.46
SET       0.840         0.49
SET       0.850         0.53
SET       0.860         0.58
SET       0.870         0.62
SET       0.880         0.66
SET       0.890         0.71
SET       0.900         0.75
SET       0.910         0.80
SET       0.920         0.86
SET       0.930         0.91
SET       0.940         0.96
SET       0.950         1.02
SET       0.960         1.08
SET       0.970         1.15
SET       0.980         1.22
SET       0.990         1.31
SET       1.000         1.87
SET       1.100         3.12
SET       1.200         4.83
SET       1.300         7.13
SET       1.400        10.40
SET       1.500        14.54
SET       1.600        19.19
SET       1.700        24.43
SET       1.800        29.81
SET       1.900        35.75
SET       2.000        41.95
SET       2.100        47.64
SET       2.200        53.10
SET       2.300        58.31
SET       2.400        62.94
SET       2.500        67.29
SET       2.600        71.31
SET       2.700        74.65
SET       2.800        77.75
SET       2.900        80.33
SET       3.000        82.73
SET       3.100        84.60
SET       3.200        86.09
SET       3.300        87.35
SET       3.400        88.41
SET       3.500        89.33
SET       3.600        90.19
SET       3.700        90.80
SET       3.800        91.37
SET       3.900        91.85
SET       4.000        92.30
SET       4.100        92.65
SET       4.200        93.01
SET       4.300        93.32
SET       4.400        93.58
SET       4.500        93.84
SET       4.600        94.10
SET       4.700        94.32
SET       4.800        94.54
SET       4.900        94.73
SET       5.000        94.96
SET       5.100        95.19
SET       5.200        95.37
SET       5.300        95.56
SET       5.400        95.74
SET       5.500        95.92
SET       5.600        96.07
SET       5.700        96.21
SET       5.800        96.35
SET       5.900        96.50
SET       6.000        96.64
SET       6.100        96.77
SET       6.200        96.90
SET       6.300        97.03
SET       6.400        97.15
SET       6.500        97.27
SET       6.600        97.38
SET       6.700        97.49
SET       6.800        97.59
SET       6.900        97.69
SET       7.000        97.79
SET       7.100        97.88
SET       7.200        97.95
SET       7.300        98.04
SET       7.400        98.12
SET       7.500        98.20
SET       7.600        98.28
SET       7.700        98.35
SET       7.800        98.42
SET       7.900        98.48
SET       8.000        98.55
SET       8.100        98.62
SET       8.200        98.68
SET       8.300        98.73
SET       8.400        98.79
SET       8.500        98.84
SET       8.600        98.89
SET       8.700        98.93
SET       8.800        98.98
SET       8.900        99.01
SET       9.000        99.06
SET       9.100        99.11
SET       9.200        99.15
SET       9.300        99.18
SET       9.400        99.22
SET       9.500        99.26
SET       9.600        99.30
SET       9.700        99.33
SET       9.800        99.37
SET       9.900        99.40
SET      10.000        99.54
SET      11.000        99.73
SET      12.000        99.84
SET      13.000        99.90
SET      14.000        99.93
SET      15.000        99.95
SET      16.000        99.97
SET      17.000        99.98
SET      18.000        99.99
SET      19.000        99.99
SET      20.000       100.00
SET      21.000       100.00
SET      25.000       100.00
SET      29.000       100.00
---
GET       0.016         0.00
GET       0.320         0.00
GET       0.340         0.00
GET       0.360         0.00
GET       0.370         0.00
GET       0.410         0.00
GET       0.420         0.01
GET       0.430         0.01
GET       0.440         0.01
GET       0.450         0.01
GET       0.460         0.01
GET       0.470         0.01
GET       0.480         0.01
GET       0.490         0.02
GET       0.500         0.02
GET       0.510         0.02
GET       0.520         0.02
GET       0.530         0.02
GET       0.540         0.03
GET       0.550         0.03
GET       0.560         0.03
GET       0.570         0.04
GET       0.580         0.05
GET       0.590         0.05
GET       0.600         0.06
GET       0.610         0.06
GET       0.620         0.07
GET       0.630         0.08
GET       0.640         0.08
GET       0.650         0.09
GET       0.660         0.10
GET       0.670         0.12
GET       0.680         0.13
GET       0.690         0.14
GET       0.700         0.16
GET       0.710         0.18
GET       0.720         0.19
GET       0.730         0.21
GET       0.740         0.23
GET       0.750         0.25
GET       0.760         0.28
GET       0.770         0.30
GET       0.780         0.33
GET       0.790         0.36
GET       0.800         0.39
GET       0.810         0.42
GET       0.820         0.45
GET       0.830         0.49
GET       0.840         0.53
GET       0.850         0.56
GET       0.860         0.61
GET       0.870         0.66
GET       0.880         0.71
GET       0.890         0.76
GET       0.900         0.80
GET       0.910         0.86
GET       0.920         0.92
GET       0.930         0.98
GET       0.940         1.04
GET       0.950         1.11
GET       0.960         1.18
GET       0.970         1.26
GET       0.980         1.35
GET       0.990         1.44
GET       1.000         2.07
GET       1.100         3.35
GET       1.200         5.15
GET       1.300         7.67
GET       1.400        10.93
GET       1.500        15.20
GET       1.600        19.97
GET       1.700        25.19
GET       1.800        30.59
GET       1.900        36.53
GET       2.000        42.74
GET       2.100        48.42
GET       2.200        53.76
GET       2.300        58.95
GET       2.400        63.58
GET       2.500        67.78
GET       2.600        71.60
GET       2.700        75.01
GET       2.800        77.98
GET       2.900        80.57
GET       3.000        82.92
GET       3.100        84.79
GET       3.200        86.28
GET       3.300        87.56
GET       3.400        88.61
GET       3.500        89.50
GET       3.600        90.30
GET       3.700        90.94
GET       3.800        91.49
GET       3.900        91.96
GET       4.000        92.41
GET       4.100        92.81
GET       4.200        93.14
GET       4.300        93.45
GET       4.400        93.72
GET       4.500        93.98
GET       4.600        94.23
GET       4.700        94.48
GET       4.800        94.72
GET       4.900        94.93
GET       5.000        95.15
GET       5.100        95.36
GET       5.200        95.56
GET       5.300        95.74
GET       5.400        95.92
GET       5.500        96.11
GET       5.600        96.27
GET       5.700        96.40
GET       5.800        96.54
GET       5.900        96.67
GET       6.000        96.83
GET       6.100        96.97
GET       6.200        97.08
GET       6.300        97.19
GET       6.400        97.32
GET       6.500        97.43
GET       6.600        97.55
GET       6.700        97.66
GET       6.800        97.75
GET       6.900        97.84
GET       7.000        97.94
GET       7.100        98.02
GET       7.200        98.10
GET       7.300        98.18
GET       7.400        98.25
GET       7.500        98.32
GET       7.600        98.40
GET       7.700        98.46
GET       7.800        98.52
GET       7.900        98.59
GET       8.000        98.67
GET       8.100        98.72
GET       8.200        98.78
GET       8.300        98.83
GET       8.400        98.88
GET       8.500        98.93
GET       8.600        98.98
GET       8.700        99.02
GET       8.800        99.07
GET       8.900        99.10
GET       9.000        99.14
GET       9.100        99.18
GET       9.200        99.22
GET       9.300        99.25
GET       9.400        99.29
GET       9.500        99.32
GET       9.600        99.35
GET       9.700        99.38
GET       9.800        99.42
GET       9.900        99.45
GET      10.000        99.56
GET      11.000        99.74
GET      12.000        99.84
GET      13.000        99.90
GET      14.000        99.94
GET      15.000        99.96
GET      16.000        99.98
GET      17.000        99.98
GET      18.000        99.99
GET      19.000        99.99
GET      20.000        99.99
GET      21.000        99.99
GET      22.000       100.00
GET      23.000       100.00
GET      24.000       100.00
GET      29.000       100.00
GET    4300000.000       100.00
---
```

</details>

<details>
  <summary><strong>memcached</strong> running in docker with `-m 1024` on a Macbook Air M1</summary>

```
docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=11211 \
>     --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8 \
>     --distinct-client-seed --randomize \
>     --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 10 -t 4
WARNING: The requested image's platform (linux/amd64) does not match the detected host platform (linux/arm64/v8) and no specific platform was requested
[RUN #1] Preparing benchmark client...
[RUN #1] Launching threads now...
[RUN #1 100%,  94 secs]  0 threads:      400000 ops,    4748 (avg:    4246) ops/sec, 1.12MB/sec (avg: 828.02KB/sec),  8.39 (avg:  9.34) msec latencyyy

4         Threads
10        Connections per thread
10000     Requests per client


ALL STATS
=========================================================================
Type         Ops/sec     Hits/sec   Misses/sec      Latency       KB/sec 
-------------------------------------------------------------------------
Sets         1411.05          ---          ---      9.34700       413.69 
Gets         2818.72      1257.59      1561.14      9.33700       411.12 
Waits           0.00          ---          ---      0.00000          --- 
Totals       4229.78      1257.59      1561.14      9.34000       824.81 


Request Latency Distribution
Type     <= msec         Percent
------------------------------------------------------------------------
SET       1.200         0.00
SET       1.300         0.00
SET       1.400         0.01
SET       1.500         0.01
SET       1.600         0.02
SET       1.700         0.04
SET       1.800         0.06
SET       1.900         0.11
SET       2.000         0.18
SET       2.100         0.29
SET       2.200         0.42
SET       2.300         0.61
SET       2.400         0.81
SET       2.500         1.08
SET       2.600         1.40
SET       2.700         1.81
SET       2.800         2.28
SET       2.900         2.84
SET       3.000         3.52
SET       3.100         4.25
SET       3.200         5.00
SET       3.300         5.81
SET       3.400         6.68
SET       3.500         7.59
SET       3.600         8.53
SET       3.700         9.49
SET       3.800        10.54
SET       3.900        11.62
SET       4.000        12.82
SET       4.100        13.97
SET       4.200        15.13
SET       4.300        16.30
SET       4.400        17.43
SET       4.500        18.61
SET       4.600        19.73
SET       4.700        20.92
SET       4.800        22.08
SET       4.900        23.28
SET       5.000        24.49
SET       5.100        25.70
SET       5.200        26.87
SET       5.300        28.04
SET       5.400        29.14
SET       5.500        30.25
SET       5.600        31.26
SET       5.700        32.30
SET       5.800        33.33
SET       5.900        34.42
SET       6.000        35.59
SET       6.100        36.68
SET       6.200        37.68
SET       6.300        38.65
SET       6.400        39.65
SET       6.500        40.61
SET       6.600        41.49
SET       6.700        42.45
SET       6.800        43.36
SET       6.900        44.31
SET       7.000        45.26
SET       7.100        46.22
SET       7.200        47.14
SET       7.300        47.97
SET       7.400        48.78
SET       7.500        49.61
SET       7.600        50.43
SET       7.700        51.23
SET       7.800        52.04
SET       7.900        52.90
SET       8.000        53.78
SET       8.100        54.57
SET       8.200        55.29
SET       8.300        55.99
SET       8.400        56.64
SET       8.500        57.27
SET       8.600        57.95
SET       8.700        58.65
SET       8.800        59.34
SET       8.900        59.99
SET       9.000        60.75
SET       9.100        61.41
SET       9.200        62.00
SET       9.300        62.63
SET       9.400        63.24
SET       9.500        63.82
SET       9.600        64.39
SET       9.700        64.94
SET       9.800        65.51
SET       9.900        66.10
SET      10.000        69.13
SET      11.000        74.00
SET      12.000        77.80
SET      13.000        81.16
SET      14.000        83.94
SET      15.000        86.35
SET      16.000        88.46
SET      17.000        90.27
SET      18.000        91.83
SET      19.000        93.12
SET      20.000        94.23
SET      21.000        95.13
SET      22.000        95.93
SET      23.000        96.62
SET      24.000        97.14
SET      25.000        97.61
SET      26.000        98.00
SET      27.000        98.35
SET      28.000        98.63
SET      29.000        98.85
SET      30.000        99.03
SET      31.000        99.18
SET      32.000        99.31
SET      33.000        99.38
SET      34.000        99.47
SET      35.000        99.54
SET      36.000        99.61
SET      37.000        99.65
SET      38.000        99.70
SET      39.000        99.75
SET      40.000        99.79
SET      41.000        99.82
SET      42.000        99.85
SET      43.000        99.87
SET      44.000        99.90
SET      45.000        99.91
SET      46.000        99.92
SET      47.000        99.93
SET      48.000        99.94
SET      49.000        99.95
SET      50.000        99.95
SET      51.000        99.96
SET      52.000        99.97
SET      53.000        99.97
SET      54.000        99.98
SET      55.000        99.98
SET      56.000        99.98
SET      57.000        99.99
SET      58.000        99.99
SET      59.000        99.99
SET      60.000        99.99
SET      61.000        99.99
SET      65.000        99.99
SET      67.000        99.99
SET      69.000        99.99
SET      90.000       100.00
SET      91.000       100.00
SET      97.000       100.00
SET     120.000       100.00
---
GET       1.100         0.00
GET       1.200         0.00
GET       1.300         0.00
GET       1.400         0.00
GET       1.500         0.01
GET       1.600         0.02
GET       1.700         0.04
GET       1.800         0.07
GET       1.900         0.11
GET       2.000         0.19
GET       2.100         0.30
GET       2.200         0.45
GET       2.300         0.62
GET       2.400         0.86
GET       2.500         1.17
GET       2.600         1.52
GET       2.700         1.94
GET       2.800         2.44
GET       2.900         3.04
GET       3.000         3.76
GET       3.100         4.50
GET       3.200         5.27
GET       3.300         6.11
GET       3.400         6.97
GET       3.500         7.91
GET       3.600         8.85
GET       3.700         9.85
GET       3.800        10.88
GET       3.900        12.02
GET       4.000        13.22
GET       4.100        14.42
GET       4.200        15.59
GET       4.300        16.75
GET       4.400        17.92
GET       4.500        19.05
GET       4.600        20.20
GET       4.700        21.36
GET       4.800        22.55
GET       4.900        23.74
GET       5.000        24.98
GET       5.100        26.14
GET       5.200        27.29
GET       5.300        28.41
GET       5.400        29.44
GET       5.500        30.48
GET       5.600        31.53
GET       5.700        32.56
GET       5.800        33.64
GET       5.900        34.73
GET       6.000        35.83
GET       6.100        36.93
GET       6.200        37.90
GET       6.300        38.88
GET       6.400        39.86
GET       6.500        40.79
GET       6.600        41.65
GET       6.700        42.58
GET       6.800        43.50
GET       6.900        44.51
GET       7.000        45.52
GET       7.100        46.46
GET       7.200        47.33
GET       7.300        48.14
GET       7.400        48.96
GET       7.500        49.78
GET       7.600        50.57
GET       7.700        51.37
GET       7.800        52.19
GET       7.900        53.02
GET       8.000        53.87
GET       8.100        54.72
GET       8.200        55.44
GET       8.300        56.16
GET       8.400        56.86
GET       8.500        57.53
GET       8.600        58.17
GET       8.700        58.85
GET       8.800        59.59
GET       8.900        60.29
GET       9.000        61.03
GET       9.100        61.72
GET       9.200        62.36
GET       9.300        62.98
GET       9.400        63.58
GET       9.500        64.16
GET       9.600        64.72
GET       9.700        65.25
GET       9.800        65.80
GET       9.900        66.38
GET      10.000        69.33
GET      11.000        73.96
GET      12.000        77.83
GET      13.000        81.12
GET      14.000        83.99
GET      15.000        86.42
GET      16.000        88.48
GET      17.000        90.25
GET      18.000        91.77
GET      19.000        93.03
GET      20.000        94.14
GET      21.000        95.06
GET      22.000        95.88
GET      23.000        96.59
GET      24.000        97.15
GET      25.000        97.59
GET      26.000        97.96
GET      27.000        98.30
GET      28.000        98.58
GET      29.000        98.79
GET      30.000        98.99
GET      31.000        99.14
GET      32.000        99.27
GET      33.000        99.37
GET      34.000        99.45
GET      35.000        99.52
GET      36.000        99.58
GET      37.000        99.63
GET      38.000        99.69
GET      39.000        99.73
GET      40.000        99.77
GET      41.000        99.80
GET      42.000        99.82
GET      43.000        99.85
GET      44.000        99.87
GET      45.000        99.88
GET      46.000        99.90
GET      47.000        99.91
GET      48.000        99.92
GET      49.000        99.93
GET      50.000        99.94
GET      51.000        99.94
GET      52.000        99.95
GET      53.000        99.96
GET      54.000        99.97
GET      55.000        99.97
GET      56.000        99.97
GET      57.000        99.97
GET      58.000        99.98
GET      59.000        99.98
GET      60.000        99.98
GET      61.000        99.98
GET      62.000        99.98
GET      63.000        99.98
GET      64.000        99.98
GET      66.000        99.99
GET      67.000        99.99
GET      68.000        99.99
GET      69.000        99.99
GET      70.000        99.99
GET      71.000        99.99
GET      72.000        99.99
GET      77.000        99.99
GET      80.000        99.99
GET      82.000       100.00
GET      88.000       100.00
GET      91.000       100.00
GET      98.000       100.00
GET     100.000       100.00
GET     110.000       100.00
GET     120.000       100.00
---
```

</details>

<details>
  <summary>memc-kv running locally on a Macbook Pro M1 Max `-m 1024` `-t 4`; test t=8, c=20</summary>

```
docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=6001     --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8     --distinct-client-seed --randomize     --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 20 -t 8
WARNING: The requested image's platform (linux/amd64) does not match the detected host platform (linux/arm64/v8) and no specific platform was requested
[RUN #1] Preparing benchmark client...
[RUN #1] Launching threads now...
[RUN #1 1%,   0 secs]  8 threads:       15918 ops,   16237 (avg:   16237) ops/sec, 1.93MB/sec (avg: 1.93MB/sec),  [RUN #1 2%,   1 secs]  8 threads:       31608 ops,   15647 (avg:   15939) ops/sec, 1.90MB/sec (avg: 1.91MB/sec), 1[RUN #1 3%,   2 secs]  8 threads:       47283 ops,   15660 (avg:   15845) ops/sec, 2.03MB/sec (avg: 1.95MB/sec), 1[RUN #1 4%,   3 secs]  8 threads:       63082 ops,   15795 (avg:   15833) ops/sec, 2.19MB/sec (avg: 2.01MB/sec), 1[RUN #1 5%,   4 secs]  8 threads:       78467 ops,   15377 (avg:   15741) ops/sec, 2.27MB/sec (avg: 2.06MB/sec), 1[RUN #1 6%,   5 secs]  8 threads:       94150 ops,   15677 (avg:   15731) ops/sec, 2.41MB/sec (avg: 2.12MB/sec), 1[RUN #1 7%,   6 secs]  8 threads:      109589 ops,   15433 (avg:   15688) ops/sec, 2.50MB/sec (avg: 2.18MB/sec), 1[RUN #1 8%,   7 secs]  8 threads:      125294 ops,   15699 (avg:   15689) ops/sec, 2.67MB/sec (avg: 2.24MB/sec), 1[RUN #1 9%,   8 secs]  8 threads:      140938 ops,   15635 (avg:   15683) ops/sec, 2.71MB/sec (avg: 2.29MB/sec), 1[RUN #1 10%,   9 secs]  8 threads:      156649 ops,   15701 (avg:   15685) ops/sec, 2.84MB/sec (avg: 2.35MB/sec), [RUN #1 11%,  10 secs]  8 threads:      172217 ops,   15562 (avg:   15674) ops/sec, 2.91MB/sec (avg: 2.40MB/sec), [RUN #1 12%,  11 secs]  8 threads:      187800 ops,   15578 (avg:   15666) ops/sec, 3.01MB/sec (avg: 2.45MB/sec), [RUN #1 13%,  12 secs]  8 threads:      203399 ops,   15595 (avg:   15660) ops/sec, 3.04MB/sec (avg: 2.49MB/sec), [RUN #1 14%,  13 secs]  8 threads:      219022 ops,   15617 (avg:   15657) ops/sec, 3.21MB/sec (avg: 2.54MB/sec), [RUN #1 15%,  14 secs]  8 threads:      234644 ops,   15617 (avg:   15655) ops/sec, 3.25MB/sec (avg: 2.59MB/sec), [RUN #1 16%,  15 secs]  8 threads:      250173 ops,   15523 (avg:   15646) ops/sec, 3.26MB/sec (avg: 2.63MB/sec), [RUN #1 17%,  16 secs]  8 threads:      265238 ops,   15057 (avg:   15612) ops/sec, 3.22MB/sec (avg: 2.67MB/sec), [RUN #1 18%,  17 secs]  8 threads:      280806 ops,   15560 (avg:   15609) ops/sec, 3.44MB/sec (avg: 2.71MB/sec), [RUN #1 19%,  18 secs]  8 threads:      296154 ops,   15343 (avg:   15595) ops/sec, 3.39MB/sec (avg: 2.75MB/sec), [RUN #1 19%,  19 secs]  8 threads:      311821 ops,   15656 (avg:   15598) ops/sec, 3.55MB/sec (avg: 2.79MB/sec), [RUN #1 20%,  20 secs]  8 threads:      327346 ops,   15519 (avg:   15594) ops/sec, 3.54MB/sec (avg: 2.82MB/sec), [RUN #1 21%,  21 secs]  8 threads:      342895 ops,   15544 (avg:   15592) ops/sec, 3.61MB/sec (avg: 2.86MB/sec), [RUN #1 22%,  22 secs]  8 threads:      358311 ops,   15410 (avg:   15584) ops/sec, 3.63MB/sec (avg: 2.89MB/sec), [RUN #1 23%,  23 secs]  8 threads:      373872 ops,   15552 (avg:   15583) ops/sec, 3.68MB/sec (avg: 2.93MB/sec), [RUN #1 24%,  24 secs]  8 threads:      389325 ops,   15448 (avg:   15577) ops/sec, 3.68MB/sec (avg: 2.96MB/sec), [RUN #1 25%,  25 secs]  8 threads:      404705 ops,   15373 (avg:   15569) ops/sec, 3.74MB/sec (avg: 2.99MB/sec), [RUN #1 26%,  26 secs]  8 threads:      420196 ops,   15516 (avg:   15567) ops/sec, 3.86MB/sec (avg: 3.02MB/sec), [RUN #1 27%,  27 secs]  8 threads:      435621 ops,   15416 (avg:   15562) ops/sec, 3.83MB/sec (avg: 3.05MB/sec), [RUN #1 28%,  28 secs]  8 threads:      451091 ops,   15462 (avg:   15559) ops/sec, 3.84MB/sec (avg: 3.07MB/sec), [RUN #1 29%,  29 secs]  8 threads:      466612 ops,   15513 (avg:   15557) ops/sec, 3.94MB/sec (avg: 3.10MB/sec), [RUN #1 30%,  30 secs]  8 threads:      482174 ops,   15556 (avg:   15557) ops/sec, 3.95MB/sec (avg: 3.13MB/sec), [RUN #1 31%,  31 secs]  8 threads:      497366 ops,   15183 (avg:   15545) ops/sec, 3.90MB/sec (avg: 3.15MB/sec), [RUN #1 32%,  32 secs]  8 threads:      512764 ops,   15392 (avg:   15541) ops/sec, 3.96MB/sec (avg: 3.18MB/sec), [RUN #1 33%,  33 secs]  8 threads:      527906 ops,   15132 (avg:   15529) ops/sec, 3.92MB/sec (avg: 3.20MB/sec), [RUN #1 34%,  34 secs]  8 threads:      543252 ops,   15340 (avg:   15523) ops/sec, 4.02MB/sec (avg: 3.22MB/sec), [RUN #1 35%,  35 secs]  8 threads:      558618 ops,   15361 (avg:   15519) ops/sec, 4.04MB/sec (avg: 3.25MB/sec), [RUN #1 36%,  36 secs]  8 threads:      574070 ops,   15441 (avg:   15517) ops/sec, 4.09MB/sec (avg: 3.27MB/sec), [RUN #1 37%,  37 secs]  8 threads:      589318 ops,   15240 (avg:   15509) ops/sec, 4.05MB/sec (avg: 3.29MB/sec), [RUN #1 38%,  38 secs]  8 threads:      604737 ops,   15407 (avg:   15507) ops/sec, 4.15MB/sec (avg: 3.31MB/sec), [RUN #1 39%,  39 secs]  8 threads:      620225 ops,   15479 (avg:   15506) ops/sec, 4.16MB/sec (avg: 3.33MB/sec), [RUN #1 40%,  40 secs]  8 threads:      635542 ops,   15313 (avg:   15501) ops/sec, 4.14MB/sec (avg: 3.35MB/sec), [RUN #1 41%,  41 secs]  8 threads:      650945 ops,   15394 (avg:   15499) ops/sec, 4.19MB/sec (avg: 3.37MB/sec), [RUN #1 42%,  42 secs]  8 threads:      666416 ops,   15457 (avg:   15498) ops/sec, 4.20MB/sec (avg: 3.39MB/sec), [RUN #1 43%,  43 secs]  8 threads:      681724 ops,   15302 (avg:   15493) ops/sec, 4.16MB/sec (avg: 3.41MB/sec), [RUN #1 44%,  45 secs]  8 threads:      697125 ops,   15395 (avg:   15491) ops/sec, 4.23MB/sec (avg: 3.43MB/sec), [RUN #1 45%,  46 secs]  8 threads:      712529 ops,   15399 (avg:   15489) ops/sec, 4.20MB/sec (avg: 3.45MB/sec), [RUN #1 46%,  47 secs]  8 threads:      728012 ops,   15478 (avg:   15489) ops/sec, 4.27MB/sec (avg: 3.46MB/sec), [RUN #1 46%,  48 secs]  8 threads:      743557 ops,   15540 (avg:   15490) ops/sec, 4.34MB/sec (avg: 3.48MB/sec), [RUN #1 47%,  49 secs]  8 threads:      758905 ops,   15338 (avg:   15487) ops/sec, 4.23MB/sec (avg: 3.50MB/sec), [RUN #1 48%,  50 secs]  8 threads:      774353 ops,   15443 (avg:   15486) ops/sec, 4.30MB/sec (avg: 3.51MB/sec), [RUN #1 49%,  51 secs]  8 threads:      789854 ops,   15490 (avg:   15486) ops/sec, 4.35MB/sec (avg: 3.53MB/sec), [RUN #1 50%,  52 secs]  8 threads:      805287 ops,   15419 (avg:   15485) ops/sec, 4.32MB/sec (avg: 3.54MB/sec), [RUN #1 51%,  53 secs]  8 threads:      820760 ops,   15463 (avg:   15484) ops/sec, 4.34MB/sec (avg: 3.56MB/sec), [RUN #1 52%,  54 secs]  8 threads:      836249 ops,   15486 (avg:   15484) ops/sec, 4.33MB/sec (avg: 3.57MB/sec), [RUN #1 53%,  55 secs]  8 threads:      851776 ops,   15518 (avg:   15485) ops/sec, 4.38MB/sec (avg: 3.59MB/sec), [RUN #1 54%,  56 secs]  8 threads:      867203 ops,   15423 (avg:   15484) ops/sec, 4.37MB/sec (avg: 3.60MB/sec), [RUN #1 55%,  57 secs]  8 threads:      882532 ops,   15323 (avg:   15481) ops/sec, 4.33MB/sec (avg: 3.61MB/sec), [RUN #1 56%,  58 secs]  8 threads:      897828 ops,   15291 (avg:   15478) ops/sec, 4.33MB/sec (avg: 3.63MB/sec), [RUN #1 57%,  59 secs]  8 threads:      913195 ops,   15359 (avg:   15476) ops/sec, 4.34MB/sec (avg: 3.64MB/sec), [RUN #1 58%,  60 secs]  8 threads:      928658 ops,   15457 (avg:   15475) ops/sec, 4.36MB/sec (avg: 3.65MB/sec), [RUN #1 59%,  61 secs]  8 threads:      944054 ops,   15392 (avg:   15474) ops/sec, 4.36MB/sec (avg: 3.66MB/sec), [RUN #1 60%,  62 secs]  8 threads:      959431 ops,   15371 (avg:   15472) ops/sec, 4.36MB/sec (avg: 3.67MB/sec), [RUN #1 61%,  63 secs]  8 threads:      975018 ops,   15581 (avg:   15474) ops/sec, 4.45MB/sec (avg: 3.69MB/sec), [RUN #1 62%,  64 secs]  8 threads:      990065 ops,   15039 (avg:   15467) ops/sec, 4.28MB/sec (avg: 3.70MB/sec), [RUN #1 63%,  65 secs]  8 threads:     1004857 ops,   14788 (avg:   15457) ops/sec, 4.22MB/sec (avg: 3.70MB/sec), [RUN #1 64%,  66 secs]  8 threads:     1020284 ops,   15422 (avg:   15456) ops/sec, 4.43MB/sec (avg: 3.71MB/sec), [RUN #1 65%,  67 secs]  8 threads:     1035761 ops,   15473 (avg:   15457) ops/sec, 4.42MB/sec (avg: 3.73MB/sec), [RUN #1 66%,  68 secs]  8 threads:     1051247 ops,   15481 (avg:   15457) ops/sec, 4.41MB/sec (avg: 3.74MB/sec), [RUN #1 67%,  69 secs]  8 threads:     1066520 ops,   15269 (avg:   15454) ops/sec, 4.39MB/sec (avg: 3.75MB/sec), [RUN #1 68%,  70 secs]  8 threads:     1081872 ops,   15347 (avg:   15453) ops/sec, 4.42MB/sec (avg: 3.75MB/sec), [RUN #1 69%,  71 secs]  8 threads:     1097297 ops,   15421 (avg:   15452) ops/sec, 4.43MB/sec (avg: 3.76MB/sec), [RUN #1 70%,  72 secs]  8 threads:     1112853 ops,   15548 (avg:   15454) ops/sec, 4.49MB/sec (avg: 3.77MB/sec), [RUN #1 71%,  73 secs]  8 threads:     1128362 ops,   15505 (avg:   15454) ops/sec, 4.47MB/sec (avg: 3.78MB/sec), [RUN #1 71%,  74 secs]  8 threads:     1143787 ops,   15420 (avg:   15454) ops/sec, 4.47MB/sec (avg: 3.79MB/sec), [RUN #1 72%,  75 secs]  8 threads:     1159303 ops,   15511 (avg:   15455) ops/sec, 4.48MB/sec (avg: 3.80MB/sec), [RUN #1 73%,  76 secs]  8 threads:     1174785 ops,   15477 (avg:   15455) ops/sec, 4.48MB/sec (avg: 3.81MB/sec), [RUN #1 74%,  77 secs]  8 threads:     1189871 ops,   15080 (avg:   15450) ops/sec, 4.36MB/sec (avg: 3.82MB/sec), [RUN #1 75%,  78 secs]  8 threads:     1205367 ops,   15491 (avg:   15450) ops/sec, 4.46MB/sec (avg: 3.83MB/sec), [RUN #1 76%,  79 secs]  8 threads:     1220844 ops,   15472 (avg:   15451) ops/sec, 4.49MB/sec (avg: 3.83MB/sec), [RUN #1 77%,  80 secs]  8 threads:     1236164 ops,   15312 (avg:   15449) ops/sec, 4.42MB/sec (avg: 3.84MB/sec), [RUN #1 78%,  81 secs]  8 threads:     1251561 ops,   15391 (avg:   15448) ops/sec, 4.48MB/sec (avg: 3.85MB/sec), [RUN #1 79%,  82 secs]  8 threads:     1266947 ops,   15380 (avg:   15447) ops/sec, 4.46MB/sec (avg: 3.86MB/sec), [RUN #1 80%,  83 secs]  8 threads:     1282402 ops,   15451 (avg:   15448) ops/sec, 4.46MB/sec (avg: 3.86MB/sec), [RUN #1 81%,  84 secs]  8 threads:     1297855 ops,   15444 (avg:   15447) ops/sec, 4.46MB/sec (avg: 3.87MB/sec), [RUN #1 82%,  85 secs]  8 threads:     1313262 ops,   15402 (avg:   15447) ops/sec, 4.44MB/sec (avg: 3.88MB/sec), [RUN #1 83%,  86 secs]  8 threads:     1328659 ops,   15392 (avg:   15446) ops/sec, 4.48MB/sec (avg: 3.89MB/sec), [RUN #1 84%,  87 secs]  8 threads:     1344124 ops,   15471 (avg:   15447) ops/sec, 4.50MB/sec (avg: 3.89MB/sec), [RUN #1 85%,  88 secs]  8 threads:     1359485 ops,   15350 (avg:   15445) ops/sec, 4.44MB/sec (avg: 3.90MB/sec), [RUN #1 86%,  89 secs]  8 threads:     1374832 ops,   15343 (avg:   15444) ops/sec, 4.46MB/sec (avg: 3.91MB/sec), [RUN #1 87%,  90 secs]  8 threads:     1390293 ops,   15457 (avg:   15444) ops/sec, 4.49MB/sec (avg: 3.91MB/sec), [RUN #1 88%,  91 secs]  8 threads:     1405653 ops,   15354 (avg:   15443) ops/sec, 4.46MB/sec (avg: 3.92MB/sec), [RUN #1 89%,  92 secs]  8 threads:     1421071 ops,   15411 (avg:   15443) ops/sec, 4.51MB/sec (avg: 3.92MB/sec), [RUN #1 90%,  93 secs]  8 threads:     1436533 ops,   15457 (avg:   15443) ops/sec, 4.52MB/sec (avg: 3.93MB/sec), [RUN #1 91%,  94 secs]  8 threads:     1451920 ops,   15381 (avg:   15443) ops/sec, 4.48MB/sec (avg: 3.94MB/sec), [RUN #1 92%,  95 secs]  8 threads:     1467327 ops,   15401 (avg:   15442) ops/sec, 4.47MB/sec (avg: 3.94MB/sec), [RUN #1 93%,  96 secs]  8 threads:     1482867 ops,   15534 (avg:   15443) ops/sec, 4.54MB/sec (avg: 3.95MB/sec), [RUN #1 94%,  97 secs]  8 threads:     1498358 ops,   15487 (avg:   15444) ops/sec, 4.50MB/sec (avg: 3.95MB/sec), [RUN #1 95%,  98 secs]  8 threads:     1513693 ops,   15327 (avg:   15442) ops/sec, 4.47MB/sec (avg: 3.96MB/sec), [RUN #1 96%,  99 secs]  8 threads:     1529222 ops,   15522 (avg:   15443) ops/sec, 4.52MB/sec (avg: 3.96MB/sec), [RUN #1 97%, 100 secs]  8 threads:     1544491 ops,   15263 (avg:   15441) ops/sec, 4.48MB/sec (avg: 3.97MB/sec), [RUN #1 97%, 101 secs]  8 threads:     1559865 ops,   15363 (avg:   15441) ops/sec, 4.49MB/sec (avg: 3.98MB/sec), [RUN #1 98%, 102 secs]  8 threads:     1575356 ops,   15485 (avg:   15441) ops/sec, 4.54MB/sec (avg: 3.98MB/sec), [RUN #1 99%, 103 secs]  8 threads:     1590784 ops,   15424 (avg:   15441) ops/sec, 4.51MB/sec (avg: 3.99MB/sec), [RUN #1 100%, 103 secs]  0 threads:     1600000 ops,   15424 (avg:   15455) ops/sec, 4.51MB/sec (avg: 3.99MB/sec), 10.36 (avg: 10.33) msec latency

8         Threads
20        Connections per thread
10000     Requests per client


ALL STATS
=========================================================================
Type         Ops/sec     Hits/sec   Misses/sec      Latency       KB/sec 
-------------------------------------------------------------------------
Sets         5153.79          ---          ---     10.32500      1515.98 
Gets        10295.23      8364.47      1930.76     10.33800      2570.49 
Waits           0.00          ---          ---      0.00000          --- 
Totals      15449.03      8364.47      1930.76     10.33300      4086.47 


Request Latency Distribution
Type     <= msec         Percent
------------------------------------------------------------------------
SET       0.180         0.00
SET       0.190         0.00
SET       0.200         0.00
SET       0.240         0.00
SET       0.280         0.00
SET       0.370         0.00
SET       0.380         0.00
SET       0.390         0.00
SET       0.400         0.00
SET       0.420         0.00
SET       0.440         0.00
SET       0.490         0.00
SET       0.500         0.00
SET       0.520         0.00
SET       0.590         0.00
SET       0.640         0.00
SET       0.650         0.00
SET       0.670         0.00
SET       0.720         0.00
SET       0.750         0.00
SET       0.760         0.00
SET       0.780         0.01
SET       0.810         0.01
SET       0.820         0.01
SET       0.830         0.01
SET       0.840         0.01
SET       0.890         0.01
SET       0.940         0.01
SET       0.950         0.01
SET       0.960         0.01
SET       0.970         0.01
SET       0.980         0.01
SET       1.000         0.01
SET       1.100         0.01
SET       1.200         0.01
SET       1.300         0.01
SET       1.400         0.02
SET       1.500         0.02
SET       1.600         0.02
SET       1.700         0.02
SET       1.800         0.02
SET       1.900         0.03
SET       2.000         0.03
SET       2.100         0.03
SET       2.200         0.03
SET       2.300         0.04
SET       2.400         0.04
SET       2.500         0.04
SET       2.600         0.04
SET       2.700         0.05
SET       2.800         0.05
SET       2.900         0.06
SET       3.000         0.06
SET       3.100         0.06
SET       3.200         0.07
SET       3.300         0.07
SET       3.400         0.08
SET       3.500         0.09
SET       3.600         0.09
SET       3.700         0.10
SET       3.800         0.10
SET       3.900         0.11
SET       4.000         0.11
SET       4.100         0.12
SET       4.200         0.12
SET       4.300         0.13
SET       4.400         0.13
SET       4.500         0.14
SET       4.600         0.15
SET       4.700         0.15
SET       4.800         0.16
SET       4.900         0.17
SET       5.000         0.18
SET       5.100         0.18
SET       5.200         0.19
SET       5.300         0.20
SET       5.400         0.20
SET       5.500         0.21
SET       5.600         0.22
SET       5.700         0.23
SET       5.800         0.23
SET       5.900         0.25
SET       6.000         0.27
SET       6.100         0.29
SET       6.200         0.31
SET       6.300         0.34
SET       6.400         0.37
SET       6.500         0.41
SET       6.600         0.46
SET       6.700         0.53
SET       6.800         0.61
SET       6.900         0.72
SET       7.000         0.86
SET       7.100         1.02
SET       7.200         1.22
SET       7.300         1.44
SET       7.400         1.71
SET       7.500         2.06
SET       7.600         2.48
SET       7.700         2.95
SET       7.800         3.49
SET       7.900         4.15
SET       8.000         4.94
SET       8.100         5.85
SET       8.200         6.86
SET       8.300         8.04
SET       8.400         9.39
SET       8.500        10.92
SET       8.600        12.64
SET       8.700        14.54
SET       8.800        16.61
SET       8.900        18.93
SET       9.000        21.51
SET       9.100        24.17
SET       9.200        26.87
SET       9.300        29.77
SET       9.400        32.77
SET       9.500        35.82
SET       9.600        38.99
SET       9.700        42.06
SET       9.800        45.21
SET       9.900        48.38
SET      10.000        64.57
SET      11.000        83.48
SET      12.000        91.00
SET      13.000        94.20
SET      14.000        96.04
SET      15.000        97.27
SET      16.000        98.09
SET      17.000        98.65
SET      18.000        99.04
SET      19.000        99.35
SET      20.000        99.55
SET      21.000        99.68
SET      22.000        99.78
SET      23.000        99.85
SET      24.000        99.89
SET      25.000        99.93
SET      26.000        99.95
SET      27.000        99.97
SET      28.000        99.98
SET      29.000        99.99
SET      30.000        99.99
SET      31.000        99.99
SET      32.000        99.99
SET      33.000        99.99
SET      34.000       100.00
SET      35.000       100.00
SET      37.000       100.00
SET      38.000       100.00
SET      39.000       100.00
SET      40.000       100.00
SET      42.000       100.00
SET      43.000       100.00
SET      44.000       100.00
---
GET       0.340         0.00
GET       0.430         0.00
GET       0.520         0.00
GET       0.540         0.00
GET       0.680         0.00
GET       0.690         0.00
GET       0.700         0.00
GET       0.710         0.00
GET       0.730         0.00
GET       0.760         0.00
GET       0.780         0.00
GET       0.800         0.00
GET       0.820         0.00
GET       0.830         0.00
GET       0.850         0.00
GET       0.860         0.00
GET       0.870         0.00
GET       0.880         0.00
GET       0.940         0.00
GET       0.950         0.00
GET       0.960         0.00
GET       0.980         0.00
GET       0.990         0.00
GET       1.000         0.00
GET       1.100         0.00
GET       1.200         0.00
GET       1.300         0.01
GET       1.400         0.01
GET       1.500         0.01
GET       1.600         0.01
GET       1.700         0.01
GET       1.800         0.01
GET       1.900         0.01
GET       2.000         0.02
GET       2.100         0.02
GET       2.200         0.02
GET       2.300         0.02
GET       2.400         0.02
GET       2.500         0.02
GET       2.600         0.02
GET       2.700         0.02
GET       2.800         0.03
GET       2.900         0.03
GET       3.000         0.03
GET       3.100         0.03
GET       3.200         0.04
GET       3.300         0.04
GET       3.400         0.05
GET       3.500         0.05
GET       3.600         0.05
GET       3.700         0.05
GET       3.800         0.06
GET       3.900         0.06
GET       4.000         0.06
GET       4.100         0.06
GET       4.200         0.06
GET       4.300         0.07
GET       4.400         0.07
GET       4.500         0.07
GET       4.600         0.08
GET       4.700         0.08
GET       4.800         0.08
GET       4.900         0.09
GET       5.000         0.09
GET       5.100         0.10
GET       5.200         0.11
GET       5.300         0.11
GET       5.400         0.12
GET       5.500         0.12
GET       5.600         0.13
GET       5.700         0.14
GET       5.800         0.14
GET       5.900         0.15
GET       6.000         0.17
GET       6.100         0.19
GET       6.200         0.21
GET       6.300         0.25
GET       6.400         0.28
GET       6.500         0.33
GET       6.600         0.39
GET       6.700         0.46
GET       6.800         0.55
GET       6.900         0.66
GET       7.000         0.81
GET       7.100         0.98
GET       7.200         1.20
GET       7.300         1.44
GET       7.400         1.73
GET       7.500         2.09
GET       7.600         2.49
GET       7.700         2.98
GET       7.800         3.54
GET       7.900         4.21
GET       8.000         5.02
GET       8.100         5.92
GET       8.200         6.95
GET       8.300         8.13
GET       8.400         9.45
GET       8.500        10.96
GET       8.600        12.66
GET       8.700        14.52
GET       8.800        16.56
GET       8.900        18.82
GET       9.000        21.40
GET       9.100        24.02
GET       9.200        26.72
GET       9.300        29.55
GET       9.400        32.52
GET       9.500        35.55
GET       9.600        38.64
GET       9.700        41.72
GET       9.800        44.87
GET       9.900        48.04
GET      10.000        64.16
GET      11.000        83.32
GET      12.000        90.92
GET      13.000        94.15
GET      14.000        96.04
GET      15.000        97.27
GET      16.000        98.08
GET      17.000        98.66
GET      18.000        99.05
GET      19.000        99.34
GET      20.000        99.53
GET      21.000        99.67
GET      22.000        99.77
GET      23.000        99.85
GET      24.000        99.89
GET      25.000        99.92
GET      26.000        99.95
GET      27.000        99.97
GET      28.000        99.98
GET      29.000        99.98
GET      30.000        99.99
GET      31.000        99.99
GET      32.000        99.99
GET      33.000        99.99
GET      34.000        99.99
GET      35.000        99.99
GET      36.000       100.00
GET      37.000       100.00
GET      38.000       100.00
GET      39.000       100.00
GET      40.000       100.00
GET      41.000       100.00
GET      42.000       100.00
GET      43.000       100.00
GET      44.000       100.00
GET      45.000       100.00
GET      46.000       100.00
GET      51.000       100.00
---
```

</details>

<details>
  <summary>memcached running locally on a Macbook Pro M1 Max `-m 1024` `-t 4`; test t=8, c=20</summary>

```
 docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=11211     --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8     --distinct-client-seed --randomize     --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 20 -t 8
WARNING: The requested image's platform (linux/amd64) does not match the detected host platform (linux/arm64/v8) and no specific platform was requested
[RUN #1] Preparing benchmark client...
[RUN #1] Launching threads now...
[RUN #1 1%,   0 secs]  8 threads:       23287 ops,   23750 (avg:   23750) ops/sec, 2.81MB/sec (avg: 2.81MB/sec),  [RUN #1 3%,   1 secs]  8 threads:       46248 ops,   22902 (avg:   23321) ops/sec, 2.97MB/sec (avg: 2.89MB/sec),  [RUN #1 4%,   2 secs]  8 threads:       69392 ops,   23135 (avg:   23259) ops/sec, 3.25MB/sec (avg: 3.01MB/sec),  [RUN #1 6%,   3 secs]  8 threads:       92892 ops,   23492 (avg:   23317) ops/sec, 3.53MB/sec (avg: 3.14MB/sec),  [RUN #1 7%,   4 secs]  8 threads:      116320 ops,   23419 (avg:   23338) ops/sec, 3.82MB/sec (avg: 3.28MB/sec),  [RUN #1 9%,   5 secs]  8 threads:      139977 ops,   23650 (avg:   23390) ops/sec, 4.08MB/sec (avg: 3.41MB/sec),  [RUN #1 10%,   6 secs]  8 threads:      163437 ops,   23449 (avg:   23398) ops/sec, 4.27MB/sec (avg: 3.53MB/sec), [RUN #1 12%,   7 secs]  8 threads:      186842 ops,   23391 (avg:   23397) ops/sec, 4.43MB/sec (avg: 3.65MB/sec), [RUN #1 13%,   8 secs]  8 threads:      210136 ops,   23287 (avg:   23385) ops/sec, 4.61MB/sec (avg: 3.75MB/sec), [RUN #1 15%,   9 secs]  8 threads:      233714 ops,   23569 (avg:   23404) ops/sec, 4.84MB/sec (avg: 3.86MB/sec), [RUN #1 16%,  10 secs]  8 threads:      257193 ops,   23471 (avg:   23410) ops/sec, 5.02MB/sec (avg: 3.97MB/sec), [RUN #1 18%,  11 secs]  8 threads:      280812 ops,   23606 (avg:   23426) ops/sec, 5.13MB/sec (avg: 4.06MB/sec), [RUN #1 19%,  12 secs]  8 threads:      304627 ops,   23807 (avg:   23455) ops/sec, 5.34MB/sec (avg: 4.16MB/sec), [RUN #1 21%,  13 secs]  8 threads:      328417 ops,   23771 (avg:   23478) ops/sec, 5.45MB/sec (avg: 4.26MB/sec), [RUN #1 22%,  14 secs]  8 threads:      352209 ops,   23786 (avg:   23498) ops/sec, 5.56MB/sec (avg: 4.34MB/sec), [RUN #1 24%,  15 secs]  8 threads:      376008 ops,   23790 (avg:   23517) ops/sec, 5.68MB/sec (avg: 4.43MB/sec), [RUN #1 25%,  16 secs]  8 threads:      399956 ops,   23940 (avg:   23542) ops/sec, 5.78MB/sec (avg: 4.51MB/sec), [RUN #1 26%,  17 secs]  8 threads:      423760 ops,   23826 (avg:   23557) ops/sec, 5.86MB/sec (avg: 4.58MB/sec), [RUN #1 28%,  18 secs]  8 threads:      447551 ops,   23785 (avg:   23569) ops/sec, 5.95MB/sec (avg: 4.65MB/sec), [RUN #1 29%,  19 secs]  8 threads:      471107 ops,   23549 (avg:   23568) ops/sec, 5.95MB/sec (avg: 4.72MB/sec), [RUN #1 31%,  20 secs]  8 threads:      494817 ops,   23703 (avg:   23575) ops/sec, 6.05MB/sec (avg: 4.78MB/sec), [RUN #1 32%,  21 secs]  8 threads:      518508 ops,   23683 (avg:   23580) ops/sec, 6.13MB/sec (avg: 4.84MB/sec), [RUN #1 34%,  22 secs]  8 threads:      542350 ops,   23836 (avg:   23591) ops/sec, 6.25MB/sec (avg: 4.90MB/sec), [RUN #1 35%,  23 secs]  8 threads:      566065 ops,   23709 (avg:   23596) ops/sec, 6.25MB/sec (avg: 4.96MB/sec), [RUN #1 37%,  24 secs]  8 threads:      589917 ops,   23845 (avg:   23606) ops/sec, 6.39MB/sec (avg: 5.02MB/sec), [RUN #1 38%,  25 secs]  8 threads:      613589 ops,   23666 (avg:   23608) ops/sec, 6.37MB/sec (avg: 5.07MB/sec), [RUN #1 40%,  26 secs]  8 threads:      637174 ops,   23573 (avg:   23607) ops/sec, 6.36MB/sec (avg: 5.12MB/sec), [RUN #1 41%,  27 secs]  8 threads:      660787 ops,   23606 (avg:   23607) ops/sec, 6.43MB/sec (avg: 5.16MB/sec), [RUN #1 43%,  28 secs]  8 threads:      684197 ops,   23388 (avg:   23599) ops/sec, 6.39MB/sec (avg: 5.21MB/sec), [RUN #1 44%,  29 secs]  8 threads:      707361 ops,   23063 (avg:   23581) ops/sec, 6.32MB/sec (avg: 5.24MB/sec), [RUN #1 46%,  30 secs]  8 threads:      730605 ops,   23207 (avg:   23569) ops/sec, 6.40MB/sec (avg: 5.28MB/sec), [RUN #1 47%,  31 secs]  8 threads:      752914 ops,   22295 (avg:   23529) ops/sec, 6.18MB/sec (avg: 5.31MB/sec), [RUN #1 48%,  32 secs]  8 threads:      774237 ops,   21317 (avg:   23462) ops/sec, 5.94MB/sec (avg: 5.33MB/sec), [RUN #1 50%,  33 secs]  8 threads:      797363 ops,   23105 (avg:   23452) ops/sec, 6.46MB/sec (avg: 5.36MB/sec), [RUN #1 51%,  34 secs]  8 threads:      820472 ops,   23100 (avg:   23442) ops/sec, 6.47MB/sec (avg: 5.39MB/sec), [RUN #1 53%,  36 secs]  8 threads:      843290 ops,   22809 (avg:   23424) ops/sec, 6.44MB/sec (avg: 5.42MB/sec), [RUN #1 54%,  37 secs]  8 threads:      866114 ops,   22813 (avg:   23408) ops/sec, 6.44MB/sec (avg: 5.45MB/sec), [RUN #1 56%,  38 secs]  8 threads:      888654 ops,   22529 (avg:   23384) ops/sec, 6.40MB/sec (avg: 5.47MB/sec), [RUN #1 57%,  39 secs]  8 threads:      911966 ops,   23303 (avg:   23382) ops/sec, 6.63MB/sec (avg: 5.50MB/sec), [RUN #1 58%,  40 secs]  8 threads:      935287 ops,   23313 (avg:   23381) ops/sec, 6.60MB/sec (avg: 5.53MB/sec), [RUN #1 60%,  41 secs]  8 threads:      958991 ops,   23686 (avg:   23388) ops/sec, 6.76MB/sec (avg: 5.56MB/sec), [RUN #1 61%,  42 secs]  8 threads:      982312 ops,   23313 (avg:   23386) ops/sec, 6.68MB/sec (avg: 5.59MB/sec), [RUN #1 62%,  43 secs]  8 threads:      995521 ops,   13203 (avg:   23149) ops/sec, 3.77MB/sec (avg: 5.55MB/sec), [RUN #1 63%,  44 secs]  8 threads:     1004278 ops,    8724 (avg:   22820) ops/sec, 2.50MB/sec (avg: 5.48MB/sec), [RUN #1 64%,  45 secs]  8 threads:     1025956 ops,   21669 (avg:   22795) ops/sec, 6.20MB/sec (avg: 5.49MB/sec), [RUN #1 66%,  46 secs]  8 threads:     1049478 ops,   23516 (avg:   22810) ops/sec, 6.76MB/sec (avg: 5.52MB/sec), [RUN #1 67%,  47 secs]  8 threads:     1073188 ops,   23701 (avg:   22829) ops/sec, 6.82MB/sec (avg: 5.55MB/sec), [RUN #1 69%,  48 secs]  8 threads:     1096819 ops,   23643 (avg:   22846) ops/sec, 6.82MB/sec (avg: 5.57MB/sec), [RUN #1 70%,  49 secs]  8 threads:     1120469 ops,   23641 (avg:   22863) ops/sec, 6.86MB/sec (avg: 5.60MB/sec), [RUN #1 72%,  50 secs]  8 threads:     1144142 ops,   23661 (avg:   22879) ops/sec, 6.85MB/sec (avg: 5.63MB/sec), [RUN #1 73%,  51 secs]  8 threads:     1167870 ops,   23720 (avg:   22895) ops/sec, 6.86MB/sec (avg: 5.65MB/sec), [RUN #1 74%,  52 secs]  8 threads:     1191494 ops,   23616 (avg:   22909) ops/sec, 6.85MB/sec (avg: 5.67MB/sec), [RUN #1 76%,  53 secs]  8 threads:     1215102 ops,   23594 (avg:   22922) ops/sec, 6.83MB/sec (avg: 5.69MB/sec), [RUN #1 77%,  54 secs]  8 threads:     1238553 ops,   23445 (avg:   22932) ops/sec, 6.83MB/sec (avg: 5.72MB/sec), [RUN #1 79%,  55 secs]  8 threads:     1262227 ops,   23668 (avg:   22945) ops/sec, 6.86MB/sec (avg: 5.74MB/sec), [RUN #1 80%,  56 secs]  8 threads:     1285803 ops,   23565 (avg:   22956) ops/sec, 6.85MB/sec (avg: 5.76MB/sec), [RUN #1 82%,  57 secs]  8 threads:     1309400 ops,   23592 (avg:   22967) ops/sec, 6.88MB/sec (avg: 5.78MB/sec), [RUN #1 83%,  58 secs]  8 threads:     1333031 ops,   23622 (avg:   22978) ops/sec, 6.89MB/sec (avg: 5.80MB/sec), [RUN #1 85%,  59 secs]  8 threads:     1356557 ops,   23516 (avg:   22988) ops/sec, 6.85MB/sec (avg: 5.81MB/sec), [RUN #1 86%,  60 secs]  8 threads:     1380108 ops,   23546 (avg:   22997) ops/sec, 6.88MB/sec (avg: 5.83MB/sec), [RUN #1 88%,  61 secs]  8 threads:     1403882 ops,   23766 (avg:   23009) ops/sec, 6.89MB/sec (avg: 5.85MB/sec), [RUN #1 89%,  62 secs]  8 threads:     1427524 ops,   23630 (avg:   23019) ops/sec, 6.91MB/sec (avg: 5.87MB/sec), [RUN #1 91%,  63 secs]  8 threads:     1450790 ops,   23254 (avg:   23023) ops/sec, 6.77MB/sec (avg: 5.88MB/sec), [RUN #1 92%,  64 secs]  8 threads:     1473966 ops,   23169 (avg:   23025) ops/sec, 6.76MB/sec (avg: 5.89MB/sec), [RUN #1 94%,  65 secs]  8 threads:     1497607 ops,   23634 (avg:   23035) ops/sec, 6.90MB/sec (avg: 5.91MB/sec), [RUN #1 95%,  66 secs]  8 threads:     1521281 ops,   23668 (avg:   23044) ops/sec, 6.88MB/sec (avg: 5.92MB/sec), [RUN #1 97%,  67 secs]  8 threads:     1544242 ops,   22954 (avg:   23043) ops/sec, 6.72MB/sec (avg: 5.94MB/sec), [RUN #1 98%,  68 secs]  8 threads:     1567542 ops,   23294 (avg:   23047) ops/sec, 6.83MB/sec (avg: 5.95MB/sec), [RUN #1 99%,  69 secs]  8 threads:     1590357 ops,   22806 (avg:   23043) ops/sec, 6.66MB/sec (avg: 5.96MB/sec), [RUN #1 100%,  69 secs]  0 threads:     1600000 ops,   22806 (avg:   23066) ops/sec, 6.66MB/sec (avg: 5.97MB/sec),  7.00 (avg:  6.92) msec latency

8         Threads
20        Connections per thread
10000     Requests per client


ALL STATS
=========================================================================
Type         Ops/sec     Hits/sec   Misses/sec      Latency       KB/sec 
-------------------------------------------------------------------------
Sets         7699.04          ---          ---      6.89000      2268.08 
Gets        15379.62     12516.95      2862.66      6.93900      3847.93 
Waits           0.00          ---          ---      0.00000          --- 
Totals      23078.66     12516.95      2862.66      6.92300      6116.01 


Request Latency Distribution
Type     <= msec         Percent
------------------------------------------------------------------------
SET       0.220         0.00
SET       0.240         0.00
SET       0.280         0.00
SET       0.300         0.00
SET       0.320         0.00
SET       0.330         0.00
SET       0.340         0.00
SET       0.460         0.00
SET       0.470         0.00
SET       0.490         0.00
SET       0.510         0.00
SET       0.520         0.00
SET       0.530         0.00
SET       0.560         0.00
SET       0.580         0.01
SET       0.610         0.01
SET       0.620         0.01
SET       0.630         0.01
SET       0.640         0.01
SET       0.650         0.01
SET       0.660         0.01
SET       0.680         0.01
SET       0.690         0.01
SET       0.700         0.01
SET       0.710         0.01
SET       0.730         0.01
SET       0.750         0.01
SET       0.780         0.01
SET       0.800         0.01
SET       0.810         0.01
SET       0.820         0.01
SET       0.830         0.01
SET       0.840         0.01
SET       0.850         0.01
SET       0.860         0.01
SET       0.870         0.01
SET       0.880         0.01
SET       0.890         0.01
SET       0.900         0.01
SET       0.910         0.01
SET       0.930         0.01
SET       0.940         0.02
SET       0.960         0.02
SET       0.970         0.02
SET       0.980         0.02
SET       0.990         0.02
SET       1.000         0.02
SET       1.100         0.03
SET       1.200         0.03
SET       1.300         0.04
SET       1.400         0.04
SET       1.500         0.05
SET       1.600         0.06
SET       1.700         0.06
SET       1.800         0.06
SET       1.900         0.07
SET       2.000         0.07
SET       2.100         0.08
SET       2.200         0.08
SET       2.300         0.08
SET       2.400         0.09
SET       2.500         0.09
SET       2.600         0.10
SET       2.700         0.11
SET       2.800         0.12
SET       2.900         0.13
SET       3.000         0.14
SET       3.100         0.15
SET       3.200         0.17
SET       3.300         0.18
SET       3.400         0.20
SET       3.500         0.22
SET       3.600         0.25
SET       3.700         0.29
SET       3.800         0.33
SET       3.900         0.39
SET       4.000         0.47
SET       4.100         0.58
SET       4.200         0.73
SET       4.300         0.92
SET       4.400         1.15
SET       4.500         1.46
SET       4.600         1.85
SET       4.700         2.34
SET       4.800         2.95
SET       4.900         3.74
SET       5.000         4.77
SET       5.100         5.99
SET       5.200         7.42
SET       5.300         9.08
SET       5.400        11.08
SET       5.500        13.29
SET       5.600        15.88
SET       5.700        18.69
SET       5.800        21.79
SET       5.900        25.13
SET       6.000        28.73
SET       6.100        32.48
SET       6.200        36.29
SET       6.300        40.09
SET       6.400        43.88
SET       6.500        47.64
SET       6.600        51.35
SET       6.700        55.00
SET       6.800        58.53
SET       6.900        61.87
SET       7.000        65.17
SET       7.100        68.22
SET       7.200        71.06
SET       7.300        73.63
SET       7.400        76.09
SET       7.500        78.36
SET       7.600        80.45
SET       7.700        82.31
SET       7.800        84.04
SET       7.900        85.62
SET       8.000        87.06
SET       8.100        88.33
SET       8.200        89.43
SET       8.300        90.42
SET       8.400        91.24
SET       8.500        92.00
SET       8.600        92.65
SET       8.700        93.20
SET       8.800        93.69
SET       8.900        94.14
SET       9.000        94.55
SET       9.100        94.89
SET       9.200        95.17
SET       9.300        95.46
SET       9.400        95.71
SET       9.500        95.94
SET       9.600        96.16
SET       9.700        96.33
SET       9.800        96.51
SET       9.900        96.69
SET      10.000        97.42
SET      11.000        98.20
SET      12.000        98.68
SET      13.000        99.00
SET      14.000        99.17
SET      15.000        99.31
SET      16.000        99.41
SET      17.000        99.50
SET      18.000        99.57
SET      19.000        99.62
SET      20.000        99.67
SET      21.000        99.70
SET      22.000        99.74
SET      23.000        99.77
SET      24.000        99.80
SET      25.000        99.82
SET      26.000        99.84
SET      27.000        99.87
SET      28.000        99.88
SET      29.000        99.89
SET      30.000        99.90
SET      31.000        99.91
SET      32.000        99.92
SET      33.000        99.93
SET      34.000        99.93
SET      35.000        99.94
SET      36.000        99.94
SET      37.000        99.94
SET      38.000        99.95
SET      39.000        99.95
SET      40.000        99.96
SET      41.000        99.97
SET      42.000        99.97
SET      43.000        99.98
SET      44.000        99.98
SET      45.000        99.98
SET      46.000        99.98
SET      47.000        99.98
SET      48.000        99.98
SET      49.000        99.98
SET      50.000        99.98
SET      51.000        99.99
SET      52.000        99.99
SET      56.000        99.99
SET      58.000        99.99
SET      59.000        99.99
SET      64.000        99.99
SET      70.000        99.99
SET      73.000        99.99
SET      76.000        99.99
SET      77.000        99.99
SET      79.000       100.00
SET      80.000       100.00
SET      81.000       100.00
SET      82.000       100.00
SET      83.000       100.00
SET      85.000       100.00
SET      86.000       100.00
SET      93.000       100.00
---
GET       0.470         0.00
GET       0.500         0.00
GET       0.510         0.00
GET       0.550         0.00
GET       0.570         0.00
GET       0.620         0.00
GET       0.630         0.00
GET       0.650         0.00
GET       0.690         0.00
GET       0.700         0.00
GET       0.710         0.00
GET       0.730         0.00
GET       0.740         0.00
GET       0.760         0.00
GET       0.770         0.00
GET       0.780         0.00
GET       0.820         0.00
GET       0.840         0.00
GET       0.850         0.00
GET       0.860         0.00
GET       0.870         0.00
GET       0.880         0.00
GET       0.890         0.00
GET       0.900         0.00
GET       0.910         0.00
GET       0.920         0.00
GET       0.930         0.00
GET       0.940         0.00
GET       0.950         0.01
GET       0.960         0.01
GET       0.970         0.01
GET       0.980         0.01
GET       0.990         0.01
GET       1.000         0.01
GET       1.100         0.01
GET       1.200         0.01
GET       1.300         0.02
GET       1.400         0.02
GET       1.500         0.03
GET       1.600         0.03
GET       1.700         0.03
GET       1.800         0.04
GET       1.900         0.04
GET       2.000         0.04
GET       2.100         0.05
GET       2.200         0.05
GET       2.300         0.05
GET       2.400         0.06
GET       2.500         0.07
GET       2.600         0.07
GET       2.700         0.08
GET       2.800         0.09
GET       2.900         0.10
GET       3.000         0.11
GET       3.100         0.12
GET       3.200         0.13
GET       3.300         0.15
GET       3.400         0.16
GET       3.500         0.18
GET       3.600         0.21
GET       3.700         0.24
GET       3.800         0.28
GET       3.900         0.34
GET       4.000         0.42
GET       4.100         0.53
GET       4.200         0.66
GET       4.300         0.84
GET       4.400         1.09
GET       4.500         1.39
GET       4.600         1.78
GET       4.700         2.27
GET       4.800         2.87
GET       4.900         3.66
GET       5.000         4.62
GET       5.100         5.79
GET       5.200         7.19
GET       5.300         8.81
GET       5.400        10.72
GET       5.500        12.86
GET       5.600        15.32
GET       5.700        18.03
GET       5.800        20.94
GET       5.900        24.21
GET       6.000        27.75
GET       6.100        31.37
GET       6.200        35.09
GET       6.300        38.85
GET       6.400        42.63
GET       6.500        46.32
GET       6.600        49.98
GET       6.700        53.57
GET       6.800        57.02
GET       6.900        60.39
GET       7.000        63.72
GET       7.100        66.76
GET       7.200        69.62
GET       7.300        72.27
GET       7.400        74.74
GET       7.500        77.07
GET       7.600        79.18
GET       7.700        81.13
GET       7.800        82.91
GET       7.900        84.57
GET       8.000        86.05
GET       8.100        87.42
GET       8.200        88.59
GET       8.300        89.64
GET       8.400        90.56
GET       8.500        91.36
GET       8.600        92.08
GET       8.700        92.67
GET       8.800        93.20
GET       8.900        93.69
GET       9.000        94.12
GET       9.100        94.49
GET       9.200        94.81
GET       9.300        95.11
GET       9.400        95.38
GET       9.500        95.62
GET       9.600        95.84
GET       9.700        96.05
GET       9.800        96.25
GET       9.900        96.42
GET      10.000        97.22
GET      11.000        98.11
GET      12.000        98.62
GET      13.000        98.94
GET      14.000        99.14
GET      15.000        99.28
GET      16.000        99.39
GET      17.000        99.48
GET      18.000        99.54
GET      19.000        99.60
GET      20.000        99.64
GET      21.000        99.69
GET      22.000        99.73
GET      23.000        99.76
GET      24.000        99.78
GET      25.000        99.81
GET      26.000        99.83
GET      27.000        99.85
GET      28.000        99.87
GET      29.000        99.88
GET      30.000        99.90
GET      31.000        99.91
GET      32.000        99.92
GET      33.000        99.93
GET      34.000        99.93
GET      35.000        99.94
GET      36.000        99.94
GET      37.000        99.95
GET      38.000        99.95
GET      39.000        99.95
GET      40.000        99.96
GET      41.000        99.97
GET      42.000        99.97
GET      43.000        99.98
GET      44.000        99.98
GET      45.000        99.98
GET      46.000        99.98
GET      47.000        99.99
GET      48.000        99.99
GET      49.000        99.99
GET      50.000        99.99
GET      51.000        99.99
GET      52.000        99.99
GET      54.000        99.99
GET      56.000        99.99
GET      58.000        99.99
GET      63.000        99.99
GET      64.000        99.99
GET      66.000        99.99
GET      70.000        99.99
GET      73.000        99.99
GET      75.000        99.99
GET      76.000        99.99
GET      77.000        99.99
GET      78.000        99.99
GET      79.000        99.99
GET      81.000       100.00
GET      82.000       100.00
GET      83.000       100.00
GET      84.000       100.00
GET      85.000       100.00
GET      86.000       100.00
GET      93.000       100.00
GET     110.000       100.00
---
```

</details>

<details>
<summary><strong>memc-kv</strong> running locally on a Macbook Pro M1 Max `-m 1024` `-t 8`; test t=8, c=20</summary>

```
 docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=6001     --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8     --distinct-client-seed --randomize     --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 20 -t 8
WARNING: The requested image's platform (linux/amd64) does not match the detected host platform (linux/arm64/v8) and no specific platform was requested
[RUN #1] Preparing benchmark client...
[RUN #1] Launching threads now...
[RUN #1 1%,   0 secs]  8 threads:       15738 ops,   16040 (avg:   16040) ops/sec, 1.87MB/sec (avg: 1.87MB/sec),  [RUN #1 2%,   1 secs]  8 threads:       31508 ops,   15720 (avg:   15878) ops/sec, 1.95MB/sec (avg: 1.91MB/sec), 1[RUN #1 3%,   2 secs]  8 threads:       47128 ops,   15608 (avg:   15787) ops/sec, 2.05MB/sec (avg: 1.96MB/sec), 1[RUN #1 4%,   3 secs]  8 threads:       62839 ops,   15788 (avg:   15788) ops/sec, 2.21MB/sec (avg: 2.02MB/sec), 1[RUN #1 5%,   4 secs]  8 threads:       78091 ops,   15243 (avg:   15678) ops/sec, 2.26MB/sec (avg: 2.07MB/sec), 1[RUN #1 6%,   5 secs]  8 threads:       93704 ops,   15608 (avg:   15666) ops/sec, 2.44MB/sec (avg: 2.13MB/sec), 1[RUN #1 7%,   6 secs]  8 threads:      109294 ops,   15583 (avg:   15654) ops/sec, 2.49MB/sec (avg: 2.18MB/sec), 1[RUN #1 8%,   7 secs]  8 threads:      124784 ops,   15483 (avg:   15633) ops/sec, 2.60MB/sec (avg: 2.23MB/sec), 1[RUN #1 9%,   8 secs]  8 threads:      140479 ops,   15689 (avg:   15639) ops/sec, 2.74MB/sec (avg: 2.29MB/sec), 1[RUN #1 10%,   9 secs]  8 threads:      156065 ops,   15577 (avg:   15633) ops/sec, 2.83MB/sec (avg: 2.34MB/sec), [RUN #1 11%,  10 secs]  8 threads:      171659 ops,   15589 (avg:   15629) ops/sec, 2.93MB/sec (avg: 2.40MB/sec), [RUN #1 12%,  11 secs]  8 threads:      187223 ops,   15560 (avg:   15623) ops/sec, 3.02MB/sec (avg: 2.45MB/sec), [RUN #1 13%,  12 secs]  8 threads:      202669 ops,   15441 (avg:   15609) ops/sec, 3.06MB/sec (avg: 2.50MB/sec), [RUN #1 14%,  13 secs]  8 threads:      218335 ops,   15662 (avg:   15613) ops/sec, 3.16MB/sec (avg: 2.54MB/sec), [RUN #1 15%,  14 secs]  8 threads:      233633 ops,   15294 (avg:   15592) ops/sec, 3.13MB/sec (avg: 2.58MB/sec), [RUN #1 16%,  15 secs]  8 threads:      249193 ops,   15554 (avg:   15589) ops/sec, 3.28MB/sec (avg: 2.63MB/sec), [RUN #1 17%,  16 secs]  8 threads:      264790 ops,   15590 (avg:   15589) ops/sec, 3.37MB/sec (avg: 2.67MB/sec), [RUN #1 18%,  17 secs]  8 threads:      280376 ops,   15579 (avg:   15589) ops/sec, 3.40MB/sec (avg: 2.71MB/sec), [RUN #1 18%,  18 secs]  8 threads:      295841 ops,   15459 (avg:   15582) ops/sec, 3.45MB/sec (avg: 2.75MB/sec), [RUN #1 19%,  19 secs]  8 threads:      311572 ops,   15726 (avg:   15589) ops/sec, 3.54MB/sec (avg: 2.79MB/sec), [RUN #1 20%,  20 secs]  8 threads:      327159 ops,   15581 (avg:   15589) ops/sec, 3.57MB/sec (avg: 2.83MB/sec), [RUN #1 21%,  21 secs]  8 threads:      342709 ops,   15546 (avg:   15587) ops/sec, 3.59MB/sec (avg: 2.86MB/sec), [RUN #1 22%,  22 secs]  8 threads:      358298 ops,   15584 (avg:   15587) ops/sec, 3.67MB/sec (avg: 2.90MB/sec), [RUN #1 23%,  23 secs]  8 threads:      373594 ops,   15291 (avg:   15574) ops/sec, 3.64MB/sec (avg: 2.93MB/sec), [RUN #1 24%,  24 secs]  8 threads:      389231 ops,   15633 (avg:   15577) ops/sec, 3.79MB/sec (avg: 2.96MB/sec), [RUN #1 25%,  25 secs]  8 threads:      404652 ops,   15416 (avg:   15570) ops/sec, 3.76MB/sec (avg: 2.99MB/sec), [RUN #1 26%,  26 secs]  8 threads:      420080 ops,   15423 (avg:   15565) ops/sec, 3.81MB/sec (avg: 3.02MB/sec), [RUN #1 27%,  27 secs]  8 threads:      435544 ops,   15458 (avg:   15561) ops/sec, 3.83MB/sec (avg: 3.05MB/sec), [RUN #1 28%,  28 secs]  8 threads:      451054 ops,   15506 (avg:   15559) ops/sec, 3.88MB/sec (avg: 3.08MB/sec), [RUN #1 29%,  29 secs]  8 threads:      466634 ops,   15575 (avg:   15560) ops/sec, 3.93MB/sec (avg: 3.11MB/sec), [RUN #1 30%,  30 secs]  8 threads:      482044 ops,   15403 (avg:   15555) ops/sec, 3.93MB/sec (avg: 3.14MB/sec), [RUN #1 31%,  31 secs]  8 threads:      497521 ops,   15471 (avg:   15552) ops/sec, 3.94MB/sec (avg: 3.16MB/sec), [RUN #1 32%,  32 secs]  8 threads:      512970 ops,   15442 (avg:   15549) ops/sec, 3.99MB/sec (avg: 3.19MB/sec), [RUN #1 33%,  33 secs]  8 threads:      528471 ops,   15485 (avg:   15547) ops/sec, 4.01MB/sec (avg: 3.21MB/sec), [RUN #1 34%,  34 secs]  8 threads:      543845 ops,   15370 (avg:   15542) ops/sec, 4.03MB/sec (avg: 3.23MB/sec), [RUN #1 35%,  35 secs]  8 threads:      558578 ops,   14729 (avg:   15519) ops/sec, 3.88MB/sec (avg: 3.25MB/sec), [RUN #1 36%,  36 secs]  8 threads:      573885 ops,   15282 (avg:   15513) ops/sec, 4.03MB/sec (avg: 3.27MB/sec), [RUN #1 37%,  37 secs]  8 threads:      589417 ops,   15522 (avg:   15513) ops/sec, 4.15MB/sec (avg: 3.30MB/sec), [RUN #1 38%,  38 secs]  8 threads:      604828 ops,   15405 (avg:   15510) ops/sec, 4.12MB/sec (avg: 3.32MB/sec), [RUN #1 39%,  39 secs]  8 threads:      620118 ops,   15284 (avg:   15505) ops/sec, 4.10MB/sec (avg: 3.34MB/sec), [RUN #1 40%,  40 secs]  8 threads:      635378 ops,   15255 (avg:   15498) ops/sec, 4.15MB/sec (avg: 3.36MB/sec), [RUN #1 41%,  41 secs]  8 threads:      650430 ops,   15047 (avg:   15488) ops/sec, 4.08MB/sec (avg: 3.37MB/sec), [RUN #1 42%,  42 secs]  8 threads:      665145 ops,   14709 (avg:   15470) ops/sec, 4.03MB/sec (avg: 3.39MB/sec), [RUN #1 43%,  43 secs]  8 threads:      680158 ops,   15006 (avg:   15459) ops/sec, 4.09MB/sec (avg: 3.40MB/sec), [RUN #1 43%,  44 secs]  8 threads:      694905 ops,   14741 (avg:   15443) ops/sec, 4.03MB/sec (avg: 3.42MB/sec), [RUN #1 44%,  45 secs]  8 threads:      709959 ops,   15049 (avg:   15435) ops/sec, 4.15MB/sec (avg: 3.43MB/sec), [RUN #1 45%,  46 secs]  8 threads:      725478 ops,   15513 (avg:   15436) ops/sec, 4.25MB/sec (avg: 3.45MB/sec), [RUN #1 46%,  47 secs]  8 threads:      740936 ops,   15454 (avg:   15437) ops/sec, 4.31MB/sec (avg: 3.47MB/sec), [RUN #1 47%,  48 secs]  8 threads:      756290 ops,   15348 (avg:   15435) ops/sec, 4.25MB/sec (avg: 3.49MB/sec), [RUN #1 48%,  49 secs]  8 threads:      771871 ops,   15574 (avg:   15438) ops/sec, 4.31MB/sec (avg: 3.50MB/sec), [RUN #1 49%,  50 secs]  8 threads:      786564 ops,   14682 (avg:   15423) ops/sec, 4.08MB/sec (avg: 3.51MB/sec), [RUN #1 50%,  51 secs]  8 threads:      801456 ops,   14886 (avg:   15412) ops/sec, 4.16MB/sec (avg: 3.53MB/sec), [RUN #1 51%,  52 secs]  8 threads:      816846 ops,   15384 (avg:   15412) ops/sec, 4.31MB/sec (avg: 3.54MB/sec), [RUN #1 52%,  53 secs]  8 threads:      831943 ops,   15091 (avg:   15406) ops/sec, 4.23MB/sec (avg: 3.55MB/sec), [RUN #1 53%,  55 secs]  8 threads:      847363 ops,   15411 (avg:   15406) ops/sec, 4.33MB/sec (avg: 3.57MB/sec), [RUN #1 54%,  56 secs]  8 threads:      862772 ops,   15403 (avg:   15406) ops/sec, 4.32MB/sec (avg: 3.58MB/sec), [RUN #1 55%,  57 secs]  8 threads:      877925 ops,   15148 (avg:   15401) ops/sec, 4.27MB/sec (avg: 3.59MB/sec), [RUN #1 56%,  58 secs]  8 threads:      893419 ops,   15482 (avg:   15403) ops/sec, 4.36MB/sec (avg: 3.61MB/sec), [RUN #1 57%,  59 secs]  8 threads:      908907 ops,   15482 (avg:   15404) ops/sec, 4.36MB/sec (avg: 3.62MB/sec), [RUN #1 58%,  60 secs]  8 threads:      924393 ops,   15480 (avg:   15405) ops/sec, 4.37MB/sec (avg: 3.63MB/sec), [RUN #1 59%,  61 secs]  8 threads:      939792 ops,   15392 (avg:   15405) ops/sec, 4.38MB/sec (avg: 3.64MB/sec), [RUN #1 60%,  62 secs]  8 threads:      955249 ops,   15448 (avg:   15406) ops/sec, 4.41MB/sec (avg: 3.66MB/sec), [RUN #1 61%,  63 secs]  8 threads:      970677 ops,   15423 (avg:   15406) ops/sec, 4.38MB/sec (avg: 3.67MB/sec), [RUN #1 62%,  64 secs]  8 threads:      986123 ops,   15434 (avg:   15407) ops/sec, 4.41MB/sec (avg: 3.68MB/sec), [RUN #1 63%,  65 secs]  8 threads:     1001550 ops,   15420 (avg:   15407) ops/sec, 4.41MB/sec (avg: 3.69MB/sec), [RUN #1 64%,  66 secs]  8 threads:     1016934 ops,   15368 (avg:   15406) ops/sec, 4.42MB/sec (avg: 3.70MB/sec), [RUN #1 65%,  67 secs]  8 threads:     1032471 ops,   15528 (avg:   15408) ops/sec, 4.46MB/sec (avg: 3.71MB/sec), [RUN #1 65%,  68 secs]  8 threads:     1047896 ops,   15418 (avg:   15408) ops/sec, 4.40MB/sec (avg: 3.72MB/sec), [RUN #1 66%,  69 secs]  8 threads:     1063414 ops,   15514 (avg:   15410) ops/sec, 4.44MB/sec (avg: 3.73MB/sec), [RUN #1 67%,  70 secs]  8 threads:     1078689 ops,   15269 (avg:   15408) ops/sec, 4.37MB/sec (avg: 3.74MB/sec), [RUN #1 68%,  71 secs]  8 threads:     1094134 ops,   15440 (avg:   15408) ops/sec, 4.44MB/sec (avg: 3.75MB/sec), [RUN #1 69%,  72 secs]  8 threads:     1109159 ops,   15020 (avg:   15403) ops/sec, 4.37MB/sec (avg: 3.76MB/sec), [RUN #1 70%,  73 secs]  8 threads:     1124452 ops,   15289 (avg:   15401) ops/sec, 4.40MB/sec (avg: 3.77MB/sec), [RUN #1 71%,  74 secs]  8 threads:     1139704 ops,   15246 (avg:   15399) ops/sec, 4.38MB/sec (avg: 3.78MB/sec), [RUN #1 72%,  75 secs]  8 threads:     1154937 ops,   15225 (avg:   15397) ops/sec, 4.37MB/sec (avg: 3.79MB/sec), [RUN #1 73%,  76 secs]  8 threads:     1170426 ops,   15485 (avg:   15398) ops/sec, 4.48MB/sec (avg: 3.79MB/sec), [RUN #1 74%,  77 secs]  8 threads:     1185859 ops,   15429 (avg:   15398) ops/sec, 4.48MB/sec (avg: 3.80MB/sec), [RUN #1 75%,  78 secs]  8 threads:     1201148 ops,   15284 (avg:   15397) ops/sec, 4.43MB/sec (avg: 3.81MB/sec), [RUN #1 76%,  79 secs]  8 threads:     1216357 ops,   15203 (avg:   15394) ops/sec, 4.41MB/sec (avg: 3.82MB/sec), [RUN #1 77%,  80 secs]  8 threads:     1231707 ops,   15332 (avg:   15394) ops/sec, 4.45MB/sec (avg: 3.83MB/sec), [RUN #1 78%,  81 secs]  8 threads:     1247190 ops,   15478 (avg:   15395) ops/sec, 4.48MB/sec (avg: 3.84MB/sec), [RUN #1 79%,  82 secs]  8 threads:     1262565 ops,   15366 (avg:   15394) ops/sec, 4.45MB/sec (avg: 3.84MB/sec), [RUN #1 80%,  83 secs]  8 threads:     1278125 ops,   15550 (avg:   15396) ops/sec, 4.52MB/sec (avg: 3.85MB/sec), [RUN #1 81%,  84 secs]  8 threads:     1293205 ops,   15072 (avg:   15392) ops/sec, 4.38MB/sec (avg: 3.86MB/sec), [RUN #1 82%,  85 secs]  8 threads:     1308627 ops,   15414 (avg:   15393) ops/sec, 4.51MB/sec (avg: 3.86MB/sec), [RUN #1 83%,  86 secs]  8 threads:     1324175 ops,   15528 (avg:   15394) ops/sec, 4.55MB/sec (avg: 3.87MB/sec), [RUN #1 84%,  87 secs]  8 threads:     1339562 ops,   15378 (avg:   15394) ops/sec, 4.45MB/sec (avg: 3.88MB/sec), [RUN #1 85%,  88 secs]  8 threads:     1354991 ops,   15421 (avg:   15394) ops/sec, 4.48MB/sec (avg: 3.89MB/sec), [RUN #1 86%,  89 secs]  8 threads:     1370503 ops,   15507 (avg:   15396) ops/sec, 4.51MB/sec (avg: 3.89MB/sec), [RUN #1 87%,  90 secs]  8 threads:     1385656 ops,   15149 (avg:   15393) ops/sec, 4.42MB/sec (avg: 3.90MB/sec), [RUN #1 88%,  91 secs]  8 threads:     1401035 ops,   15373 (avg:   15393) ops/sec, 4.49MB/sec (avg: 3.91MB/sec), [RUN #1 89%,  92 secs]  8 threads:     1416506 ops,   15464 (avg:   15393) ops/sec, 4.51MB/sec (avg: 3.91MB/sec), [RUN #1 89%,  93 secs]  8 threads:     1431996 ops,   15483 (avg:   15394) ops/sec, 4.53MB/sec (avg: 3.92MB/sec), [RUN #1 90%,  94 secs]  8 threads:     1447282 ops,   15284 (avg:   15393) ops/sec, 4.43MB/sec (avg: 3.92MB/sec), [RUN #1 91%,  95 secs]  8 threads:     1462551 ops,   15261 (avg:   15392) ops/sec, 4.43MB/sec (avg: 3.93MB/sec), [RUN #1 92%,  96 secs]  8 threads:     1477766 ops,   15209 (avg:   15390) ops/sec, 4.46MB/sec (avg: 3.94MB/sec), [RUN #1 93%,  97 secs]  8 threads:     1493115 ops,   15342 (avg:   15389) ops/sec, 4.44MB/sec (avg: 3.94MB/sec), [RUN #1 94%,  98 secs]  8 threads:     1508674 ops,   15551 (avg:   15391) ops/sec, 4.54MB/sec (avg: 3.95MB/sec), [RUN #1 95%,  99 secs]  8 threads:     1524212 ops,   15534 (avg:   15392) ops/sec, 4.53MB/sec (avg: 3.95MB/sec), [RUN #1 96%, 100 secs]  8 threads:     1539674 ops,   15456 (avg:   15393) ops/sec, 4.51MB/sec (avg: 3.96MB/sec), [RUN #1 97%, 101 secs]  8 threads:     1554918 ops,   15239 (avg:   15392) ops/sec, 4.43MB/sec (avg: 3.96MB/sec), [RUN #1 98%, 102 secs]  8 threads:     1570413 ops,   15486 (avg:   15393) ops/sec, 4.51MB/sec (avg: 3.97MB/sec), [RUN #1 99%, 103 secs]  8 threads:     1585761 ops,   15340 (avg:   15392) ops/sec, 4.50MB/sec (avg: 3.97MB/sec), [RUN #1 100%, 103 secs]  0 threads:     1600000 ops,   15340 (avg:   15402) ops/sec, 4.50MB/sec (avg: 3.98MB/sec), 10.39 (avg: 10.37) msec latency

8         Threads
20        Connections per thread
10000     Requests per client


ALL STATS
=========================================================================
Type         Ops/sec     Hits/sec   Misses/sec      Latency       KB/sec 
-------------------------------------------------------------------------
Sets         5143.61          ---          ---     10.36200      1514.89 
Gets        10274.88      8343.80      1931.08     10.37300      2565.08 
Waits           0.00          ---          ---      0.00000          --- 
Totals      15418.48      8343.80      1931.08     10.36900      4079.98 


Request Latency Distribution
Type     <= msec         Percent
------------------------------------------------------------------------
SET       0.250         0.00
SET       0.270         0.00
SET       0.350         0.00
SET       0.360         0.00
SET       0.390         0.00
SET       0.410         0.00
SET       0.450         0.00
SET       0.470         0.00
SET       0.490         0.00
SET       0.540         0.00
SET       0.550         0.00
SET       0.560         0.00
SET       0.600         0.00
SET       0.610         0.00
SET       0.620         0.00
SET       0.640         0.00
SET       0.660         0.00
SET       0.670         0.00
SET       0.680         0.00
SET       0.710         0.01
SET       0.730         0.01
SET       0.740         0.01
SET       0.760         0.01
SET       0.810         0.01
SET       0.830         0.01
SET       0.850         0.01
SET       0.870         0.01
SET       0.910         0.01
SET       0.920         0.01
SET       0.930         0.01
SET       0.960         0.01
SET       0.970         0.01
SET       0.990         0.01
SET       1.000         0.01
SET       1.100         0.01
SET       1.200         0.01
SET       1.300         0.01
SET       1.400         0.02
SET       1.500         0.02
SET       1.600         0.02
SET       1.700         0.02
SET       1.800         0.02
SET       1.900         0.03
SET       2.000         0.03
SET       2.100         0.03
SET       2.200         0.03
SET       2.300         0.03
SET       2.400         0.04
SET       2.500         0.04
SET       2.600         0.04
SET       2.700         0.04
SET       2.800         0.05
SET       2.900         0.05
SET       3.000         0.05
SET       3.100         0.06
SET       3.200         0.06
SET       3.300         0.06
SET       3.400         0.07
SET       3.500         0.07
SET       3.600         0.07
SET       3.700         0.07
SET       3.800         0.07
SET       3.900         0.08
SET       4.000         0.08
SET       4.100         0.08
SET       4.200         0.09
SET       4.300         0.09
SET       4.400         0.10
SET       4.500         0.10
SET       4.600         0.11
SET       4.700         0.12
SET       4.800         0.12
SET       4.900         0.13
SET       5.000         0.13
SET       5.100         0.14
SET       5.200         0.15
SET       5.300         0.16
SET       5.400         0.17
SET       5.500         0.18
SET       5.600         0.19
SET       5.700         0.21
SET       5.800         0.22
SET       5.900         0.23
SET       6.000         0.25
SET       6.100         0.27
SET       6.200         0.29
SET       6.300         0.32
SET       6.400         0.35
SET       6.500         0.39
SET       6.600         0.44
SET       6.700         0.50
SET       6.800         0.58
SET       6.900         0.67
SET       7.000         0.78
SET       7.100         0.92
SET       7.200         1.10
SET       7.300         1.32
SET       7.400         1.57
SET       7.500         1.87
SET       7.600         2.23
SET       7.700         2.64
SET       7.800         3.12
SET       7.900         3.70
SET       8.000         4.44
SET       8.100         5.27
SET       8.200         6.18
SET       8.300         7.31
SET       8.400         8.56
SET       8.500        10.02
SET       8.600        11.67
SET       8.700        13.45
SET       8.800        15.42
SET       8.900        17.68
SET       9.000        20.25
SET       9.100        22.90
SET       9.200        25.62
SET       9.300        28.51
SET       9.400        31.50
SET       9.500        34.64
SET       9.600        37.87
SET       9.700        41.05
SET       9.800        44.28
SET       9.900        47.54
SET      10.000        64.05
SET      11.000        83.41
SET      12.000        90.94
SET      13.000        94.14
SET      14.000        96.04
SET      15.000        97.25
SET      16.000        98.07
SET      17.000        98.63
SET      18.000        99.02
SET      19.000        99.32
SET      20.000        99.52
SET      21.000        99.65
SET      22.000        99.75
SET      23.000        99.82
SET      24.000        99.87
SET      25.000        99.90
SET      26.000        99.92
SET      27.000        99.93
SET      28.000        99.95
SET      29.000        99.95
SET      30.000        99.96
SET      31.000        99.97
SET      32.000        99.97
SET      33.000        99.97
SET      34.000        99.98
SET      35.000        99.98
SET      36.000        99.98
SET      37.000        99.98
SET      38.000        99.99
SET      39.000        99.99
SET      40.000        99.99
SET      41.000        99.99
SET      42.000        99.99
SET      43.000       100.00
SET      44.000       100.00
SET      45.000       100.00
SET      46.000       100.00
SET      47.000       100.00
SET      48.000       100.00
SET      49.000       100.00
SET      52.000       100.00
SET      53.000       100.00
SET      55.000       100.00
---
GET       0.340         0.00
GET       0.480         0.00
GET       0.560         0.00
GET       0.640         0.00
GET       0.650         0.00
GET       0.700         0.00
GET       0.720         0.00
GET       0.790         0.00
GET       0.820         0.00
GET       0.830         0.00
GET       0.850         0.00
GET       0.860         0.00
GET       0.870         0.00
GET       0.890         0.00
GET       0.900         0.00
GET       0.930         0.00
GET       0.970         0.00
GET       0.990         0.00
GET       1.000         0.00
GET       1.100         0.00
GET       1.200         0.00
GET       1.300         0.01
GET       1.400         0.01
GET       1.500         0.01
GET       1.600         0.01
GET       1.700         0.01
GET       1.800         0.01
GET       1.900         0.01
GET       2.000         0.01
GET       2.100         0.01
GET       2.200         0.01
GET       2.300         0.02
GET       2.400         0.02
GET       2.500         0.02
GET       2.600         0.02
GET       2.700         0.02
GET       2.800         0.03
GET       2.900         0.03
GET       3.000         0.03
GET       3.100         0.03
GET       3.200         0.03
GET       3.300         0.03
GET       3.400         0.04
GET       3.500         0.04
GET       3.600         0.04
GET       3.700         0.04
GET       3.800         0.04
GET       3.900         0.05
GET       4.000         0.05
GET       4.100         0.05
GET       4.200         0.06
GET       4.300         0.06
GET       4.400         0.06
GET       4.500         0.07
GET       4.600         0.08
GET       4.700         0.08
GET       4.800         0.09
GET       4.900         0.09
GET       5.000         0.10
GET       5.100         0.10
GET       5.200         0.11
GET       5.300         0.12
GET       5.400         0.12
GET       5.500         0.13
GET       5.600         0.14
GET       5.700         0.15
GET       5.800         0.17
GET       5.900         0.18
GET       6.000         0.20
GET       6.100         0.21
GET       6.200         0.23
GET       6.300         0.26
GET       6.400         0.30
GET       6.500         0.34
GET       6.600         0.39
GET       6.700         0.45
GET       6.800         0.53
GET       6.900         0.62
GET       7.000         0.74
GET       7.100         0.89
GET       7.200         1.07
GET       7.300         1.28
GET       7.400         1.53
GET       7.500         1.85
GET       7.600         2.21
GET       7.700         2.65
GET       7.800         3.14
GET       7.900         3.70
GET       8.000         4.43
GET       8.100         5.26
GET       8.200         6.18
GET       8.300         7.28
GET       8.400         8.52
GET       8.500         9.94
GET       8.600        11.57
GET       8.700        13.34
GET       8.800        15.33
GET       8.900        17.52
GET       9.000        20.02
GET       9.100        22.67
GET       9.200        25.36
GET       9.300        28.24
GET       9.400        31.26
GET       9.500        34.39
GET       9.600        37.61
GET       9.700        40.79
GET       9.800        43.96
GET       9.900        47.25
GET      10.000        63.83
GET      11.000        83.34
GET      12.000        90.96
GET      13.000        94.13
GET      14.000        96.02
GET      15.000        97.23
GET      16.000        98.03
GET      17.000        98.59
GET      18.000        98.99
GET      19.000        99.30
GET      20.000        99.49
GET      21.000        99.64
GET      22.000        99.74
GET      23.000        99.81
GET      24.000        99.86
GET      25.000        99.89
GET      26.000        99.92
GET      27.000        99.93
GET      28.000        99.95
GET      29.000        99.96
GET      30.000        99.96
GET      31.000        99.97
GET      32.000        99.97
GET      33.000        99.98
GET      34.000        99.98
GET      35.000        99.98
GET      36.000        99.98
GET      37.000        99.98
GET      38.000        99.99
GET      39.000        99.99
GET      40.000        99.99
GET      41.000        99.99
GET      42.000        99.99
GET      43.000       100.00
GET      44.000       100.00
GET      45.000       100.00
GET      46.000       100.00
GET      47.000       100.00
GET      48.000       100.00
GET      49.000       100.00
GET      50.000       100.00
GET      51.000       100.00
GET      53.000       100.00
---
```

</details>

<details>
  <summary><strong>memcached</strong> running locally on a Macbook Pro M1 Max `-m 1024` `-t 8`; test t=8, c=20</summary>

```
docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=11211     --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8     --distinct-client-seed --randomize     --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 20 -t 8
WARNING: The requested image's platform (linux/amd64) does not match the detected host platform (linux/arm64/v8) and no specific platform was requested
[RUN #1] Preparing benchmark client...
[RUN #1] Launching threads now...
[RUN #1 1%,   0 secs]  8 threads:       23221 ops,   23685 (avg:   23685) ops/sec, 5.83MB/sec (avg: 5.83MB/sec),  [RUN #1 3%,   1 secs]  8 threads:       46701 ops,   23417 (avg:   23550) ops/sec, 5.84MB/sec (avg: 5.83MB/sec),  [RUN #1 4%,   2 secs]  8 threads:       69947 ops,   23237 (avg:   23445) ops/sec, 5.87MB/sec (avg: 5.85MB/sec),  [RUN #1 6%,   3 secs]  8 threads:       93398 ops,   23444 (avg:   23445) ops/sec, 5.98MB/sec (avg: 5.88MB/sec),  [RUN #1 7%,   4 secs]  8 threads:      116856 ops,   23452 (avg:   23446) ops/sec, 6.07MB/sec (avg: 5.92MB/sec),  [RUN #1 9%,   5 secs]  8 threads:      140243 ops,   23381 (avg:   23435) ops/sec, 6.10MB/sec (avg: 5.95MB/sec),  [RUN #1 10%,   6 secs]  8 threads:      163627 ops,   23401 (avg:   23430) ops/sec, 6.18MB/sec (avg: 5.98MB/sec), [RUN #1 12%,   7 secs]  8 threads:      186995 ops,   23347 (avg:   23420) ops/sec, 6.19MB/sec (avg: 6.01MB/sec), [RUN #1 13%,   8 secs]  8 threads:      210191 ops,   23182 (avg:   23393) ops/sec, 6.23MB/sec (avg: 6.03MB/sec), [RUN #1 15%,   9 secs]  8 threads:      233627 ops,   23430 (avg:   23397) ops/sec, 6.32MB/sec (avg: 6.06MB/sec), [RUN #1 16%,  10 secs]  8 threads:      257092 ops,   23458 (avg:   23403) ops/sec, 6.37MB/sec (avg: 6.09MB/sec), [RUN #1 18%,  11 secs]  8 threads:      280472 ops,   23369 (avg:   23400) ops/sec, 6.33MB/sec (avg: 6.11MB/sec), [RUN #1 19%,  12 secs]  8 threads:      303703 ops,   23216 (avg:   23386) ops/sec, 6.36MB/sec (avg: 6.13MB/sec), [RUN #1 20%,  13 secs]  8 threads:      327088 ops,   23371 (avg:   23385) ops/sec, 6.44MB/sec (avg: 6.15MB/sec), [RUN #1 22%,  14 secs]  8 threads:      350439 ops,   23336 (avg:   23381) ops/sec, 6.49MB/sec (avg: 6.17MB/sec), [RUN #1 23%,  15 secs]  8 threads:      373276 ops,   22831 (avg:   23347) ops/sec, 6.32MB/sec (avg: 6.18MB/sec), [RUN #1 25%,  16 secs]  8 threads:      396553 ops,   23271 (avg:   23343) ops/sec, 6.47MB/sec (avg: 6.20MB/sec), [RUN #1 26%,  17 secs]  8 threads:      419348 ops,   22786 (avg:   23312) ops/sec, 6.36MB/sec (avg: 6.21MB/sec), [RUN #1 28%,  18 secs]  8 threads:      442189 ops,   22832 (avg:   23286) ops/sec, 6.39MB/sec (avg: 6.22MB/sec), [RUN #1 29%,  19 secs]  8 threads:      465544 ops,   23344 (avg:   23289) ops/sec, 6.54MB/sec (avg: 6.23MB/sec), [RUN #1 31%,  20 secs]  8 threads:      488773 ops,   23221 (avg:   23286) ops/sec, 6.56MB/sec (avg: 6.25MB/sec), [RUN #1 32%,  21 secs]  8 threads:      511986 ops,   23207 (avg:   23282) ops/sec, 6.61MB/sec (avg: 6.27MB/sec), [RUN #1 33%,  22 secs]  8 threads:      535122 ops,   23130 (avg:   23276) ops/sec, 6.57MB/sec (avg: 6.28MB/sec), [RUN #1 35%,  23 secs]  8 threads:      558328 ops,   23200 (avg:   23273) ops/sec, 6.63MB/sec (avg: 6.29MB/sec), [RUN #1 36%,  24 secs]  8 threads:      581733 ops,   23398 (avg:   23278) ops/sec, 6.68MB/sec (avg: 6.31MB/sec), [RUN #1 38%,  25 secs]  8 threads:      604742 ops,   23003 (avg:   23267) ops/sec, 6.56MB/sec (avg: 6.32MB/sec), [RUN #1 39%,  26 secs]  8 threads:      628047 ops,   23298 (avg:   23268) ops/sec, 6.69MB/sec (avg: 6.33MB/sec), [RUN #1 41%,  27 secs]  8 threads:      651071 ops,   23018 (avg:   23259) ops/sec, 6.61MB/sec (avg: 6.34MB/sec), [RUN #1 42%,  28 secs]  8 threads:      674118 ops,   23036 (avg:   23252) ops/sec, 6.64MB/sec (avg: 6.35MB/sec), [RUN #1 44%,  29 secs]  8 threads:      696944 ops,   22819 (avg:   23237) ops/sec, 6.58MB/sec (avg: 6.36MB/sec), [RUN #1 45%,  30 secs]  8 threads:      720009 ops,   23055 (avg:   23231) ops/sec, 6.65MB/sec (avg: 6.37MB/sec), [RUN #1 46%,  31 secs]  8 threads:      743383 ops,   23364 (avg:   23235) ops/sec, 6.73MB/sec (avg: 6.38MB/sec), [RUN #1 48%,  32 secs]  8 threads:      766623 ops,   23235 (avg:   23235) ops/sec, 6.71MB/sec (avg: 6.39MB/sec), [RUN #1 49%,  33 secs]  8 threads:      789823 ops,   23193 (avg:   23234) ops/sec, 6.69MB/sec (avg: 6.40MB/sec), [RUN #1 51%,  34 secs]  8 threads:      812809 ops,   22977 (avg:   23227) ops/sec, 6.70MB/sec (avg: 6.41MB/sec), [RUN #1 52%,  35 secs]  8 threads:      836048 ops,   23231 (avg:   23227) ops/sec, 6.73MB/sec (avg: 6.42MB/sec), [RUN #1 54%,  36 secs]  8 threads:      859273 ops,   23204 (avg:   23226) ops/sec, 6.73MB/sec (avg: 6.43MB/sec), [RUN #1 55%,  37 secs]  8 threads:      882507 ops,   23225 (avg:   23226) ops/sec, 6.78MB/sec (avg: 6.44MB/sec), [RUN #1 57%,  38 secs]  8 threads:      905684 ops,   23170 (avg:   23225) ops/sec, 6.76MB/sec (avg: 6.44MB/sec), [RUN #1 58%,  39 secs]  8 threads:      929162 ops,   23470 (avg:   23231) ops/sec, 6.79MB/sec (avg: 6.45MB/sec), [RUN #1 60%,  40 secs]  8 threads:      952530 ops,   23353 (avg:   23234) ops/sec, 6.78MB/sec (avg: 6.46MB/sec), [RUN #1 61%,  41 secs]  8 threads:      975312 ops,   22773 (avg:   23223) ops/sec, 6.65MB/sec (avg: 6.46MB/sec), [RUN #1 62%,  42 secs]  8 threads:      998197 ops,   22875 (avg:   23215) ops/sec, 6.65MB/sec (avg: 6.47MB/sec), [RUN #1 64%,  43 secs]  8 threads:     1020683 ops,   22474 (avg:   23198) ops/sec, 6.58MB/sec (avg: 6.47MB/sec), [RUN #1 65%,  44 secs]  8 threads:     1043869 ops,   23169 (avg:   23197) ops/sec, 6.77MB/sec (avg: 6.48MB/sec), [RUN #1 67%,  45 secs]  8 threads:     1066849 ops,   22973 (avg:   23192) ops/sec, 6.69MB/sec (avg: 6.48MB/sec), [RUN #1 68%,  46 secs]  8 threads:     1089684 ops,   22828 (avg:   23185) ops/sec, 6.69MB/sec (avg: 6.49MB/sec), [RUN #1 70%,  47 secs]  8 threads:     1112937 ops,   23235 (avg:   23186) ops/sec, 6.77MB/sec (avg: 6.49MB/sec), [RUN #1 71%,  49 secs]  8 threads:     1135831 ops,   22887 (avg:   23180) ops/sec, 6.66MB/sec (avg: 6.50MB/sec), [RUN #1 72%,  50 secs]  8 threads:     1159049 ops,   23212 (avg:   23180) ops/sec, 6.79MB/sec (avg: 6.50MB/sec), [RUN #1 74%,  51 secs]  8 threads:     1182249 ops,   23193 (avg:   23181) ops/sec, 6.78MB/sec (avg: 6.51MB/sec), [RUN #1 75%,  52 secs]  8 threads:     1205449 ops,   23195 (avg:   23181) ops/sec, 6.81MB/sec (avg: 6.51MB/sec), [RUN #1 77%,  53 secs]  8 threads:     1228854 ops,   23397 (avg:   23185) ops/sec, 6.85MB/sec (avg: 6.52MB/sec), [RUN #1 78%,  54 secs]  8 threads:     1251980 ops,   23119 (avg:   23184) ops/sec, 6.77MB/sec (avg: 6.52MB/sec), [RUN #1 80%,  55 secs]  8 threads:     1275352 ops,   23359 (avg:   23187) ops/sec, 6.79MB/sec (avg: 6.53MB/sec), [RUN #1 81%,  56 secs]  8 threads:     1298327 ops,   22961 (avg:   23183) ops/sec, 6.72MB/sec (avg: 6.53MB/sec), [RUN #1 83%,  57 secs]  8 threads:     1321354 ops,   23007 (avg:   23180) ops/sec, 6.69MB/sec (avg: 6.54MB/sec), [RUN #1 84%,  58 secs]  8 threads:     1344727 ops,   23366 (avg:   23183) ops/sec, 6.80MB/sec (avg: 6.54MB/sec), [RUN #1 86%,  59 secs]  8 threads:     1368082 ops,   23349 (avg:   23186) ops/sec, 6.83MB/sec (avg: 6.55MB/sec), [RUN #1 87%,  60 secs]  8 threads:     1391406 ops,   23308 (avg:   23188) ops/sec, 6.81MB/sec (avg: 6.55MB/sec), [RUN #1 88%,  61 secs]  8 threads:     1414739 ops,   23324 (avg:   23190) ops/sec, 6.85MB/sec (avg: 6.55MB/sec), [RUN #1 90%,  62 secs]  8 threads:     1437938 ops,   23185 (avg:   23190) ops/sec, 6.78MB/sec (avg: 6.56MB/sec), [RUN #1 91%,  63 secs]  8 threads:     1461455 ops,   23505 (avg:   23195) ops/sec, 6.88MB/sec (avg: 6.56MB/sec), [RUN #1 93%,  64 secs]  8 threads:     1484782 ops,   23316 (avg:   23197) ops/sec, 6.85MB/sec (avg: 6.57MB/sec), [RUN #1 94%,  65 secs]  8 threads:     1507922 ops,   23129 (avg:   23196) ops/sec, 6.75MB/sec (avg: 6.57MB/sec), [RUN #1 96%,  66 secs]  8 threads:     1531228 ops,   23291 (avg:   23197) ops/sec, 6.86MB/sec (avg: 6.57MB/sec), [RUN #1 97%,  67 secs]  8 threads:     1554576 ops,   23348 (avg:   23199) ops/sec, 6.87MB/sec (avg: 6.58MB/sec), [RUN #1 99%,  68 secs]  8 threads:     1577780 ops,   23195 (avg:   23199) ops/sec, 6.80MB/sec (avg: 6.58MB/sec), [RUN #1 100%,  68 secs]  0 threads:     1600000 ops,   23195 (avg:   23229) ops/sec, 6.80MB/sec (avg: 6.59MB/sec),  6.89 (avg:  6.87) msec latency

8         Threads
20        Connections per thread
10000     Requests per client


ALL STATS
=========================================================================
Type         Ops/sec     Hits/sec   Misses/sec      Latency       KB/sec 
-------------------------------------------------------------------------
Sets         7750.39          ---          ---      6.85300      2282.01 
Gets        15482.20     14702.47       779.73      6.88400      4471.02 
Waits           0.00          ---          ---      0.00000          --- 
Totals      23232.59     14702.47       779.73      6.87400      6753.04 


Request Latency Distribution
Type     <= msec         Percent
------------------------------------------------------------------------
SET       0.220         0.00
SET       0.230         0.00
SET       0.260         0.00
SET       0.270         0.00
SET       0.300         0.00
SET       0.330         0.00
SET       0.360         0.00
SET       0.420         0.00
SET       0.440         0.00
SET       0.450         0.00
SET       0.460         0.00
SET       0.470         0.00
SET       0.480         0.00
SET       0.490         0.00
SET       0.500         0.00
SET       0.510         0.01
SET       0.520         0.01
SET       0.530         0.01
SET       0.540         0.01
SET       0.550         0.01
SET       0.560         0.01
SET       0.580         0.01
SET       0.590         0.01
SET       0.600         0.01
SET       0.610         0.01
SET       0.620         0.01
SET       0.650         0.01
SET       0.670         0.01
SET       0.680         0.01
SET       0.690         0.01
SET       0.700         0.01
SET       0.710         0.01
SET       0.720         0.01
SET       0.730         0.01
SET       0.740         0.01
SET       0.750         0.01
SET       0.760         0.01
SET       0.770         0.01
SET       0.790         0.01
SET       0.810         0.01
SET       0.820         0.01
SET       0.830         0.01
SET       0.840         0.01
SET       0.850         0.02
SET       0.860         0.02
SET       0.870         0.02
SET       0.880         0.02
SET       0.890         0.02
SET       0.900         0.02
SET       0.910         0.02
SET       0.920         0.02
SET       0.930         0.02
SET       0.950         0.02
SET       0.960         0.02
SET       0.970         0.02
SET       0.980         0.02
SET       0.990         0.02
SET       1.000         0.02
SET       1.100         0.03
SET       1.200         0.03
SET       1.300         0.03
SET       1.400         0.04
SET       1.500         0.04
SET       1.600         0.04
SET       1.700         0.05
SET       1.800         0.05
SET       1.900         0.06
SET       2.000         0.06
SET       2.100         0.06
SET       2.200         0.07
SET       2.300         0.08
SET       2.400         0.09
SET       2.500         0.09
SET       2.600         0.10
SET       2.700         0.11
SET       2.800         0.12
SET       2.900         0.13
SET       3.000         0.14
SET       3.100         0.15
SET       3.200         0.17
SET       3.300         0.18
SET       3.400         0.19
SET       3.500         0.21
SET       3.600         0.23
SET       3.700         0.25
SET       3.800         0.28
SET       3.900         0.32
SET       4.000         0.37
SET       4.100         0.45
SET       4.200         0.57
SET       4.300         0.72
SET       4.400         0.91
SET       4.500         1.16
SET       4.600         1.47
SET       4.700         1.87
SET       4.800         2.40
SET       4.900         3.07
SET       5.000         3.90
SET       5.100         4.88
SET       5.200         6.08
SET       5.300         7.49
SET       5.400         9.21
SET       5.500        11.19
SET       5.600        13.46
SET       5.700        16.05
SET       5.800        18.98
SET       5.900        22.30
SET       6.000        25.85
SET       6.100        29.54
SET       6.200        33.29
SET       6.300        37.16
SET       6.400        41.00
SET       6.500        44.88
SET       6.600        48.74
SET       6.700        52.55
SET       6.800        56.20
SET       6.900        59.73
SET       7.000        63.18
SET       7.100        66.37
SET       7.200        69.31
SET       7.300        72.17
SET       7.400        74.84
SET       7.500        77.35
SET       7.600        79.61
SET       7.700        81.69
SET       7.800        83.56
SET       7.900        85.27
SET       8.000        86.85
SET       8.100        88.24
SET       8.200        89.46
SET       8.300        90.53
SET       8.400        91.47
SET       8.500        92.32
SET       8.600        93.05
SET       8.700        93.69
SET       8.800        94.24
SET       8.900        94.74
SET       9.000        95.18
SET       9.100        95.55
SET       9.200        95.86
SET       9.300        96.15
SET       9.400        96.41
SET       9.500        96.64
SET       9.600        96.84
SET       9.700        97.03
SET       9.800        97.20
SET       9.900        97.36
SET      10.000        98.11
SET      11.000        98.95
SET      12.000        99.40
SET      13.000        99.64
SET      14.000        99.77
SET      15.000        99.86
SET      16.000        99.92
SET      17.000        99.95
SET      18.000        99.97
SET      19.000        99.98
SET      20.000        99.99
SET      21.000        99.99
SET      22.000       100.00
SET      23.000       100.00
SET      24.000       100.00
SET      25.000       100.00
SET      26.000       100.00
SET      27.000       100.00
SET      30.000       100.00
---
GET       0.290         0.00
GET       0.320         0.00
GET       0.390         0.00
GET       0.400         0.00
GET       0.410         0.00
GET       0.420         0.00
GET       0.440         0.00
GET       0.450         0.00
GET       0.460         0.00
GET       0.480         0.00
GET       0.490         0.00
GET       0.500         0.00
GET       0.530         0.00
GET       0.540         0.00
GET       0.560         0.00
GET       0.580         0.00
GET       0.590         0.00
GET       0.600         0.00
GET       0.610         0.00
GET       0.620         0.00
GET       0.630         0.00
GET       0.640         0.00
GET       0.670         0.00
GET       0.680         0.00
GET       0.690         0.00
GET       0.700         0.00
GET       0.720         0.00
GET       0.730         0.01
GET       0.740         0.01
GET       0.750         0.01
GET       0.760         0.01
GET       0.770         0.01
GET       0.780         0.01
GET       0.800         0.01
GET       0.810         0.01
GET       0.820         0.01
GET       0.830         0.01
GET       0.850         0.01
GET       0.860         0.01
GET       0.870         0.01
GET       0.890         0.01
GET       0.900         0.01
GET       0.910         0.01
GET       0.920         0.01
GET       0.930         0.01
GET       0.940         0.01
GET       0.960         0.01
GET       0.970         0.01
GET       0.980         0.01
GET       0.990         0.01
GET       1.000         0.01
GET       1.100         0.01
GET       1.200         0.02
GET       1.300         0.02
GET       1.400         0.02
GET       1.500         0.03
GET       1.600         0.03
GET       1.700         0.03
GET       1.800         0.03
GET       1.900         0.04
GET       2.000         0.04
GET       2.100         0.04
GET       2.200         0.05
GET       2.300         0.05
GET       2.400         0.06
GET       2.500         0.06
GET       2.600         0.07
GET       2.700         0.07
GET       2.800         0.08
GET       2.900         0.09
GET       3.000         0.10
GET       3.100         0.11
GET       3.200         0.12
GET       3.300         0.13
GET       3.400         0.14
GET       3.500         0.16
GET       3.600         0.18
GET       3.700         0.20
GET       3.800         0.23
GET       3.900         0.27
GET       4.000         0.32
GET       4.100         0.41
GET       4.200         0.52
GET       4.300         0.67
GET       4.400         0.86
GET       4.500         1.10
GET       4.600         1.42
GET       4.700         1.82
GET       4.800         2.35
GET       4.900         3.00
GET       5.000         3.83
GET       5.100         4.82
GET       5.200         5.97
GET       5.300         7.38
GET       5.400         9.09
GET       5.500        11.01
GET       5.600        13.21
GET       5.700        15.74
GET       5.800        18.56
GET       5.900        21.72
GET       6.000        25.15
GET       6.100        28.74
GET       6.200        32.43
GET       6.300        36.24
GET       6.400        40.06
GET       6.500        43.93
GET       6.600        47.74
GET       6.700        51.55
GET       6.800        55.17
GET       6.900        58.68
GET       7.000        62.11
GET       7.100        65.31
GET       7.200        68.35
GET       7.300        71.23
GET       7.400        73.92
GET       7.500        76.43
GET       7.600        78.74
GET       7.700        80.86
GET       7.800        82.76
GET       7.900        84.53
GET       8.000        86.19
GET       8.100        87.64
GET       8.200        88.88
GET       8.300        90.02
GET       8.400        91.01
GET       8.500        91.88
GET       8.600        92.66
GET       8.700        93.33
GET       8.800        93.92
GET       8.900        94.45
GET       9.000        94.90
GET       9.100        95.30
GET       9.200        95.65
GET       9.300        95.96
GET       9.400        96.23
GET       9.500        96.48
GET       9.600        96.71
GET       9.700        96.92
GET       9.800        97.10
GET       9.900        97.28
GET      10.000        98.04
GET      11.000        98.90
GET      12.000        99.36
GET      13.000        99.61
GET      14.000        99.76
GET      15.000        99.86
GET      16.000        99.92
GET      17.000        99.95
GET      18.000        99.97
GET      19.000        99.98
GET      20.000        99.99
GET      21.000        99.99
GET      22.000       100.00
GET      23.000       100.00
GET      24.000       100.00
GET      27.000       100.00
---
```

</details>