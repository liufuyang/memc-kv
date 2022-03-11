use dashmap::mapref::one::Ref;
use dashmap::DashMap;
use log::debug;
use std::hash::Hash;
use std::ops::{Add, Deref};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

pub struct Cache<K, V> {
    map: Arc<DashMap<K, Value<V>>>,
    default_ttl: Option<Duration>,
}

// 'static is used here which means the K and V *can* live as 'static as they will be also referenced by a long running thread
impl<'a, K: 'a + Eq + Hash + Send + Sync + 'static, V: 'a + Send + Sync + 'static> Cache<K, V> {
    pub fn new(default_ttl: Option<Duration>) -> Cache<K, V> {
        let m = DashMap::new();
        let arc = Arc::new(m);
        let map = arc.clone();

        thread::spawn(move || loop {
            let old_size = map.len();
            map.retain(|_, v: &mut Value<V>| !v.is_expired());
            debug!("vacuum expired keys, size {} -> {}", old_size, map.len());
            thread::sleep(Duration::from_secs(10));
        });

        Cache {
            map: arc,
            default_ttl,
        }
    }

    pub fn get(&'a self, key: &K) -> Option<RefWrapper<'a, K, V>> {
        self.map
            .get(key)
            .and_then(|r| if r.is_expired() { None } else { Some(r.into()) })
    }

    /// Inserts a key and a value into the map. Returns the old value associated with the key if there was one.
    pub fn insert(&self, key: K, value: V, flag: u32) -> Option<V> {
        self.map
            .insert(key, Value::new(value, self.default_ttl, flag))
            .map(|v| v.value) // return old value if exist
    }

    /// Inserts a key and a value into the map. Returns the old value associated with the key if there was one.
    pub fn insert_with_ttl(
        &self,
        key: K,
        value: V,
        default_ttl_seconds: u32,
        flag: u32,
    ) -> Option<V> {
        self.map
            .insert(key, Value::new_with_ttl(value, default_ttl_seconds, flag))
            .map(|v| v.value) // return old value if exist
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

impl<K: Eq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
            default_ttl: self.default_ttl.clone(),
        }
    }
}

struct Value<V> {
    value: V,
    flag: u32,
    timestamp: Option<SystemTime>,
}

impl<V> Value<V> {
    pub fn new(value: V, ttl: Option<Duration>, flag: u32) -> Self {
        Value {
            value,
            flag,
            timestamp: ttl.map(|ttl| SystemTime::now().add(ttl)),
        }
    }

    pub fn new_with_ttl(value: V, default_ttl_seconds: u32, flag: u32) -> Self {
        Value {
            value,
            flag,
            timestamp: if default_ttl_seconds == 0 {
                None
            } else {
                Some(SystemTime::now().add(Duration::from_secs(default_ttl_seconds as u64)))
            },
        }
    }

    pub fn is_expired(&self) -> bool {
        self.timestamp
            .map(|t| t.lt(&SystemTime::now()))
            .unwrap_or(false)
    }
}

pub struct RefWrapper<'a, K, V> {
    inner: Ref<'a, K, Value<V>>,
}

impl<'a, K: Eq + Hash, V> RefWrapper<'a, K, V> {
    pub fn get_flag(&self) -> u32 {
        return self.inner.deref().flag;
    }
}

impl<'a, K: Eq + Hash, V> Deref for RefWrapper<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &V {
        &self.inner.deref().value
    }
}

impl<'a, K: Eq + Hash, V> From<Ref<'a, K, Value<V>>> for RefWrapper<'a, K, V> {
    fn from(inner: Ref<'a, K, Value<V>>) -> Self {
        RefWrapper { inner }
    }
}
