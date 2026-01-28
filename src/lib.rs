pub mod hgnc_cache_functions;
pub mod hgnc_struct;

use crate::hgnc_struct::{ArchivedHgncCache, ArchivedHgncRecord};
use std::error::Error;

pub fn query_lookup_table(
    query: String,
    cache: &ArchivedHgncCache,
) -> Result<&ArchivedHgncRecord, Box<dyn Error>> {
    let idx = cache.lookup.get(query.to_uppercase().as_str());

    match idx {
        Some(&index) => {
            let native_index = index.to_native() as usize;
            Ok(&cache.records[native_index])
        }
        None => Err(format!("Query '{}' not found in cache", query).into()),
    }
}
