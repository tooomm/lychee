use std::{fs, path::Path};

use anyhow::{Context, Result};
use dashmap::DashMap;
use lychee_lib::{Status, Uri};
use serde::{Deserialize, Serialize};

// pub(crate) struct Cache(DashMap<Uri, Status>);
pub(crate) type Cache = DashMap<String, Status>;

pub(crate) trait StoreExt {
    fn store<T: AsRef<Path>>(&self, path: T) -> Result<()>;
    fn load<T: AsRef<Path>>(path: T) -> Result<Cache>;
}

#[derive(Debug, Deserialize)]
struct Record {
    uri: Uri,
    status: Status,
}

impl StoreExt for Cache {
    fn store<T: AsRef<Path>>(&self, path: T) -> Result<()> {
        // Toml expects the keys to be strings
        // Do the mapping here in order to keep the same interface in case we change the cache format in the future.
        // let data = self
        //     .iter()
        //     .map(|i| (i.key().to_string(), i.value()))
        //     .collect();
        let serialized = toml::to_string(&self)?;
        fs::write(&path, serialized).context(format!(
            "Cannot read cache from {}",
            path.as_ref().display()
        ))
    }

    fn load<T: AsRef<Path>>(path: T) -> Result<Cache> {
        todo!()
        // let map = DashMap::new();
        // let mut rdr = csv::Reader::from_path(path)?;
        // for result in rdr.deserialize() {
        //     let (uri, status): (Uri, Status) = result?;
        //     println!("uri: {:?}, status: {:?}", uri, status);
        //     map.insert(uri, status);
        // }
        // Ok(map)
    }
}
