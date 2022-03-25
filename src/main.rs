#![feature(io_error_more)]

extern crate core;

mod http_server;
mod memcache_server;
mod metrics;
mod parser;

use std;
use std::time::Duration;

use kv_cache::Cache;

const EXPIRE_DURATION: Duration = Duration::from_secs(3600);

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    env_logger::init();

    let cache = Cache::<Vec<u8>, Vec<u8>>::new(Some(EXPIRE_DURATION));
    let http_server = http_server::HttpServer::new(cache.clone());
    let memcache_server = memcache_server::MemcacheServer::new(cache.clone());

    let (_, _) = tokio::join!(http_server.serve(), memcache_server.serve());
}
