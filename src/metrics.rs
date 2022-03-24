use lazy_static::lazy_static;

use prometheus::{
    exponential_buckets, register_gauge, register_histogram_vec, Gauge, HistogramVec,
};

lazy_static! {
    // Metrics
    // https://gist.github.com/breeswish/bb10bccd13a7fe332ef534ff0306ceb5
    pub static ref METRIC_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "request_duration_seconds",
        "Histogram of memcached request duration in seconds",
        &["method"],
        exponential_buckets(0.005, 2.0, 10).unwrap()
        ).unwrap();

    pub static ref METRIC_REQUEST_DURATION_MEMC: HistogramVec = register_histogram_vec!(
        "memc_request_duration_seconds",
        "Histogram of memcached request duration in seconds",
        &["method"],
        exponential_buckets(0.005, 2.0, 10).unwrap()
        ).unwrap();

    pub static ref METRIC_CACHE_SIZE: Gauge = register_gauge!(
        "cache_size",
        "Size of the cache"
        ).unwrap();
}
