use crate::types::{ALL_FIELDS, ArchivedHgncRecord, Cache, HgncRecord};
use indexmap::IndexMap;
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

        // Check for map collisions and log them
        if let Some(existing_idx) = map.get(&record.hgnc_id.to_uppercase()) {
            eprintln!(
                "Collision detected for HGNC ID {}: existing record at index {}, new record at index {}",
                record.hgnc_id, existing_idx, record_idx
            );
        }
        if let Some(existing_idx) = map.get(&record.symbol.to_uppercase()) {
            eprintln!(
                "Collision detected for symbol {}: existing record at index {}, new record at index {}",
                record.symbol, existing_idx, record_idx
            );
        }

        map.insert(record.hgnc_id.clone().to_uppercase(), record_idx);
        map.insert(record.symbol.clone().to_uppercase(), record_idx);
        for alias in record.alias_symbol.split('|').filter(|s| !s.is_empty()) {
            if let Some(existing_idx) = map.get(&alias.trim().to_uppercase()) {
                eprintln!(
                    "Collision detected for alias symbol {}: existing record at index {}, new record at index {}",
                    alias.trim(),
                    existing_idx,
                    record_idx
                );
            }

            map.insert(alias.trim().to_uppercase(), record_idx);
        }
        for prev in record.prev_symbol.split('|').filter(|s| !s.is_empty()) {
            if let Some(existing_idx) = map.get(&prev.trim().to_uppercase()) {
                eprintln!(
                    "Collision detected for previous symbol {}: existing record at index {}, new record at index {}",
                    prev.trim(),
                    existing_idx,
                    record_idx
                );
            }

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

fn extract_records<'a, I>(record: &ArchivedHgncRecord, fields: I) -> IndexMap<String, String>
where
    I: IntoIterator<Item = &'a str>,
{
    fields
        .into_iter()
        .filter_map(|name| {
            let trimmed = name.trim();
            record
                .get_field(trimmed)
                .filter(|value| !value.is_empty())
                .map(|value| (trimmed.to_string(), value.to_string()))
        })
        .collect()
}

pub fn get_fields_from_record(
    record: &ArchivedHgncRecord,
    fields: &Option<Vec<String>>,
) -> IndexMap<String, String> {
    match fields {
        Some(v) => extract_records(record, v.iter().map(|s| s.as_str())),
        None => extract_records(record, ALL_FIELDS.iter().copied()),
    }
}
