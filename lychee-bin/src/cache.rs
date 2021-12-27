use std::{fs, path::Path};

use anyhow::Result;
use csv;
use dashmap::DashMap;
use lychee_lib::{Status, Uri};
use serde::{Deserialize, Serialize};

// pub(crate) struct Cache(DashMap<Uri, Status>);
pub(crate) type Cache = DashMap<Uri, Status>;

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
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        for result in self {
            wtr.serialize((result.key(), result.value()))?
        }
        Ok(())
    }

    fn load<T: AsRef<Path>>(path: T) -> Result<Cache> {
        let map = DashMap::new();
        let mut rdr = csv::Reader::from_path(path)?;
        for result in rdr.deserialize() {
            let (uri, status): (Uri, Status) = result?;
            println!("uri: {:?}, status: {:?}", uri, status);
            map.insert(uri, status);
        }
        Ok(map)
    }
}
