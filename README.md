# memc-kv

A simple memcached like in memory kv implemented in Rust, 
for demo and learning purpose (if it won't get more useful later...)

Maybe compatible with memcached ascii protocol on commands:
- [x] `set <key> <flag> <ttl> <len>`
- [x] `get <key>`

Code is not fully tested, but you are welcome to add new features
and tests or provide new ideas and feedbacks.

## Implementation
Inspired by [memc-rs](https://github.com/memc-rs/memc-rs), it seems not too difficult 
to implement a memcached ascii protocol compatible server for 
simple get/set command with the help of modern Rust friends, including
- [tokio](https://tokio.rs/tokio/tutorial) - An asynchronous runtime for the Rust programming language
which is perfect for handling bytes read and write on tcp ports (networking applications)
- [dashmap](https://github.com/xacrimon/dashmap) - Blazingly fast concurrent map in Rust.

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
docker run --rm redislabs/memtier_benchmark --protocol=memcache_text --server 192.168.0.25 --port=6001 --generate-keys -n 1000 --key-maximum=10000 --ratio=2:8
```