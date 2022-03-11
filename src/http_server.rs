use std::hash::Hash;
use std::thread;
use std::time::{Duration, SystemTime};

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use log::info;
use prometheus::{Encoder, TextEncoder};

use kv_cache::Cache;

use crate::metrics::{METRIC_CACHE_SIZE, METRIC_REQUEST_DURATION};

pub struct HttpServer<K: Eq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> {
    cache: Cache<K, V>,
}

impl<K: Eq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> HttpServer<K, V> {
    pub fn new(cache: Cache<K, V>) -> Self {
        HttpServer { cache }
    }

    pub async fn serve(&self) -> Result<(), hyper::Error> {
        // metrics
        let addr = ([127, 0, 0, 1], 9001).into();

        // start size metric reporting thread
        let cache = self.cache.clone();
        thread::spawn(move || loop {
            METRIC_CACHE_SIZE.set(cache.len() as f64);
            thread::sleep(Duration::from_secs(5));
        });

        // build metric_http_server
        let cache = self.cache.clone();
        let metric_service = make_service_fn(move |_| {
            let cache = cache.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| metric_handler(cache.clone(), req)))
            }
        });
        let metric_http_server = Server::bind(&addr);

        info!("Metric HTTP server listening on http://{}/metrics", addr);

        metric_http_server.serve(metric_service).await
    }
}

async fn metric_handler<K, V>(
    cache: Cache<K, V>,
    req: Request<Body>,
) -> Result<Response<Body>, hyper::Error>
where
    K: Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    let start_time = SystemTime::now();
    let result = match req.method() {
        // Serve some instructions at /
        &Method::GET => {
            let key: &str = req.uri().path();
            if key.len() < 2 {
                return Ok(Response::builder()
                    .status(400)
                    .body(Body::from("Must provide a key in the path"))
                    .unwrap());
            }
            if key.to_lowercase().eq("/size") {
                return Ok(Response::new(Body::from(cache.len().to_string())));
            }
            if key.to_lowercase().eq("/metrics") {
                let encoder = TextEncoder::new();
                let mut buffer = vec![];
                let mf = prometheus::gather();
                encoder.encode(&mf, &mut buffer).unwrap();
                return Ok(Response::builder()
                    .header(hyper::header::CONTENT_TYPE, encoder.format_type())
                    .body(Body::from(buffer))
                    .unwrap());
            }
            (Ok(Response::default()), "get")
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            (Ok(not_found), "other")
        }
    };

    let duration = SystemTime::now().duration_since(start_time).unwrap();
    METRIC_REQUEST_DURATION
        .with_label_values(&[result.1])
        .observe(duration.as_secs_f64());
    return result.0;
}
