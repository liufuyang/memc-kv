# http-kv

```
RUST_LOG=info cargo run --release
```

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