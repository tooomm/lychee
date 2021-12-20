use lychee_lib::{Status, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request cache for faster checking
#[derive(Serialize, Deserialize)]
pub(crate) struct Cache {
    pub(crate) inner: HashMap<Uri, Status>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        Cache {
            inner: HashMap::new(),
        }
    }

    /// Look up a potentially cached request
    /// Returns None on cache miss
    pub(crate) fn get(&self, uri: &Uri) -> Option<&Status> {
        let res = self.inner.get(uri);
        if res.is_some() {
            println!("Cache hit!");
        }
        res
    }

    /// Look up a potentially cached request
    /// Returns None on cache miss
    pub(crate) fn set(&mut self, uri: Uri, status: Status) -> Option<Status> {
        println!("Set {}", uri);
        self.inner.insert(uri, status)
    }
}
