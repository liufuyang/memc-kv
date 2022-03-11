# memc-kv

A simple memcached like in memory kv implemented in Rust, 
for demo and learning purpose (if it won't get more useful later...)

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

Code is not fully tested, but you are welcome to add new features
and tests or provide new ideas and feedbacks.

## Implementation
Inspired by [memc-rs](https://github.com/memc-rs/memc-rs), it seems not too difficult 
to implement a memcached ascii protocol compatible server for 
simple get/set command with the help of modern Rust friends, including
- [tokio](https://tokio.rs/tokio/tutorial) - An asynchronous runtime for the Rust programming language
which is perfect for handling bytes read and write on tcp ports (networking applications)
- [dashmap](https://github.com/xacrimon/dashmap) - Blazingly fast concurrent map in Rust
- [nom](https://github.com/Geal/nom) - for fast and easy parsing memcached commands

## So why doing this
Besides learning purpose, I think this may bring some extra value or potential
as the design is very simple here. Plus using a modern language like Rust,
it should be quite easy to add more features such as Http/gPRC endpoints or
even making it can run a distributed fashion which support leader-follower
data replications. In the same time we probably won't be that optimized 
comparing with original C implementation but `memc-kv` should be able to
run fairly fast as well. 

## How to start `memc-kv` locally
```
RUST_LOG=trace cargo run --release
```

## Reference links
- [memcached protocol](https://github.com/memcached/memcached/blob/master/doc/protocol.txt)
- [memcached cheatsheet](https://lzone.de/cheat-sheet/memcached)
- [tokio mini-redis code example](https://github.com/tokio-rs/mini-redis/blob/tutorial/src/frame.rs#L254-L262)
- [tokio docs](https://docs.rs/tokio/1.17.0/tokio/io/trait.AsyncReadExt.html#method.read)
- [libmemcached](https://launchpad.net/libmemcached) installation to use `memcapable` for protocol compatibility check
  - [download link]()
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
    --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 10 -t 4

docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server host.docker.internal --port=11211 \
    --generate-keys --key-maximum=100000 --key-prefix=key- --ratio=4:8 \
    --distinct-client-seed --randomize \
    --data-size-range=32-500 --expiry-range=10-3600 -n 10000 -c 10 -t 4
```

One way to start memcached is via docker, however on my Mac it seems the
application running in docker is a bit slow.
```
docker run --name mc -d --rm -p 11211:11211 memcached memcached -m 1024
```

### Benchmark details (preliminary):

| impl                  | `set P99` | `get P99` | `ops/sec` |
|-----------------------|----------:|----------:|----------:|
| `memc-kv` locally     |    17.0ms |    17.0ms |      9660 |
| `memcached` locally   |     8.9ms |     8.7ms |     15187 |
| `memcached` in docker |    30.0ms |    30.0ms |      4229 |

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