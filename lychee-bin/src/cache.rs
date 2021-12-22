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
        self.inner.get(uri)
    }

    /// Look up a potentially cached request
    /// Returns None on cache miss
    pub(crate) fn insert(&mut self, uri: Uri, status: Status) -> Option<Status> {
        self.inner.insert(uri, status)
    }
}
