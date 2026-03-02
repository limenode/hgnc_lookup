use crate::types::{ALL_FIELDS, ArchivedHgncRecord, Cache, HgncRecord};
use std::{collections::HashMap, path::Path};

const HGNC_COMPLETE_SET_URL: &str =
    "https://storage.googleapis.com/public-download-files/hgnc/tsv/tsv/hgnc_complete_set.txt";

pub fn build_cache(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // If the file already exists, skip downloading and building
    if path.exists() {
        println!(
            "Cache file already exists at {:?}, skipping download and build.",
            path
        );
        return Ok(());
    }

    let response = reqwest::blocking::get(HGNC_COMPLETE_SET_URL)?;
    let content = response.bytes()?;

    // Read with csv crate
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(content.as_ref());

    // Iterate, build HashMap and Vec, and create Cache
    let mut map: HashMap<String, usize> = HashMap::new();
    let mut records: Vec<HgncRecord> = Vec::new();
    let mut record_idx = 0;

    for result in reader.deserialize() {
        let record: HgncRecord = result?;
        map.insert(record.hgnc_id.clone().to_uppercase(), record_idx);
        map.insert(record.symbol.clone().to_uppercase(), record_idx);
        for alias in record.alias_symbol.split('|').filter(|s| !s.is_empty()) {
            map.insert(alias.trim().to_uppercase(), record_idx);
        }
        for prev in record.prev_symbol.split('|').filter(|s| !s.is_empty()) {
            map.insert(prev.trim().to_uppercase(), record_idx);
        }
        records.push(record);
        record_idx += 1;
    }

    let cache = Cache { records, map };
    let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&cache)?;

    // Write to file
    std::fs::write(path, serialized)?;

    Ok(())
}

pub fn get_fields_from_record(
    record: &ArchivedHgncRecord,
    fields: &Option<Vec<String>>,
) -> Vec<String> {
    let fields = match fields {
        Some(v) => v,
        None => {
            return ALL_FIELDS
                .iter()
                .filter_map(|name| {
                    record
                        .get_field(name.trim())
                        .map(|value| format!("{}: {}", name.trim(), value))
                })
                .collect();
        }
    };

    fields
        .iter()
        .filter_map(|name| {
            record
                .get_field(name.trim())
                .map(|value| format!("{}: {}", name.trim(), value.to_string()))
        })
        .collect()
}
