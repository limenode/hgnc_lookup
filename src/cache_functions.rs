use crate::types::{ALL_FIELDS, ArchivedHgncRecord, Cache, HgncRecord};
use indexmap::IndexMap;
use std::{collections::HashMap, path::Path};

const HGNC_COMPLETE_SET_URL: &str =
    "https://storage.googleapis.com/public-download-files/hgnc/tsv/tsv/hgnc_complete_set.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum KeyPriority {
    Alias = 0,
    Previous = 1,
    Standard = 2,
    Static = 3,
}

fn insert_with_priority(
    map: &mut HashMap<String, usize>,
    key_priorities: &mut HashMap<String, KeyPriority>,
    key_raw: String,
    priority: KeyPriority,
    idx: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let key = key_raw.trim().to_uppercase();

    // Check if the key is empty after trimming
    if key.is_empty() {
        return Ok(()); // Skip empty keys
    }

    let existing_priority = key_priorities.get(&key);

    if let Some(&existing_priority) = existing_priority {
        // Check for Static-Static collision - this is a fatal error
        if existing_priority == KeyPriority::Static && priority == KeyPriority::Static {
            return Err(format!(
                "FATAL: Duplicate static key detected: '{}'. Existing at index {}, new at index {}. Static keys must be unique.",
                key,
                map.get(&key).unwrap_or(&usize::MAX),
                idx
            ).into());
        }

        // Static keys should never be overridden by non-static
        if existing_priority == KeyPriority::Static {
            eprintln!(
                "Collision detected for key {}: existing priority {:?} (index {}), new priority {:?} (index {}) - keeping existing (static)",
                key,
                existing_priority,
                map.get(&key).unwrap_or(&usize::MAX),
                priority,
                idx
            );
            return Ok(());
        }

        if priority > existing_priority {
            // Higher priority, override
            eprintln!(
                "Collision detected for key {}: existing priority {:?} (index {}), new priority {:?} (index {}) - keeping new",
                key,
                existing_priority,
                map.get(&key).unwrap_or(&usize::MAX),
                priority,
                idx
            );

            map.insert(key.clone(), idx);
            key_priorities.insert(key, priority);
        } else {
            // else: lower or equal priority, skip insertion
            eprintln!(
                "Collision detected for key {}: existing priority {:?} (index {}), new priority {:?} (index {}) - keeping existing",
                key,
                existing_priority,
                map.get(&key).unwrap_or(&usize::MAX),
                priority,
                idx
            );
        }
    } else {
        // New key, insert
        map.insert(key.clone(), idx);
        key_priorities.insert(key, priority);
    }

    Ok(())
}

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
    let mut key_priorities: HashMap<String, KeyPriority> = HashMap::new();
    let mut records: Vec<HgncRecord> = Vec::new();
    let mut record_idx = 0;

    for result in reader.deserialize() {
        let record: HgncRecord = result?;

        // Insert HGNC ID as Static (will error on collision)
        let hgnc_key = record.hgnc_id.trim().to_uppercase();
        insert_with_priority(
            &mut map,
            &mut key_priorities,
            hgnc_key,
            KeyPriority::Static,
            record_idx,
        )?;

        // Insert Ensembl ID as Static (will error on collision)
        let ensembl_key = record.ensembl_gene_id.trim().to_uppercase();
        insert_with_priority(
            &mut map,
            &mut key_priorities,
            ensembl_key,
            KeyPriority::Standard,
            record_idx,
        )?;

        // Insert standard symbol
        insert_with_priority(
            &mut map,
            &mut key_priorities,
            record.symbol.trim().to_uppercase(),
            KeyPriority::Standard,
            record_idx,
        )?;

        // Insert previous symbols
        for prev in record.prev_symbol.split('|').filter(|s| !s.is_empty()) {
            insert_with_priority(
                &mut map,
                &mut key_priorities,
                prev.trim().to_uppercase(),
                KeyPriority::Previous,
                record_idx,
            )?;
        }

        // Insert alias symbols
        for alias in record.alias_symbol.split('|').filter(|s| !s.is_empty()) {
            insert_with_priority(
                &mut map,
                &mut key_priorities,
                alias.trim().to_uppercase(),
                KeyPriority::Alias,
                record_idx,
            )?;
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
