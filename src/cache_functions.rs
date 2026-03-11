use crate::types::{ALL_FIELDS, ArchivedHgncRecord, Cache, HgncRecord, KeyPriority, Match};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::error::Error;

const HGNC_COMPLETE_SET_URL: &str =
    "https://storage.googleapis.com/public-download-files/hgnc/tsv/tsv/hgnc_complete_set.txt";

fn insert_match(
    map: &mut HashMap<String, Vec<Match>>,
    key: String,
    priority: KeyPriority,
    idx: usize,
) -> Result<(), Box<dyn Error>> {
    let key = key.trim().to_uppercase();

    // Skip empty keys
    if key.is_empty() {
        return Ok(()); // Skip empty keys
    }

    let new_match = Match::new(priority, idx);

    map.entry(key.clone())
        .and_modify(|matches| {
            // Check for Static-Static collision - this is a fatal error
            if priority == KeyPriority::Static
                && matches.iter().any(|m| m.priority == KeyPriority::Static)
            {
                eprintln!(
                    "FATAL: Duplicate static key detected: '{}'. This should never happen.",
                    key
                );
            }

            // Add the new match
            matches.push(new_match);

            // Keep sorted by priority (highest first)
            matches.sort_by(|a, b| b.priority.cmp(&a.priority));

            eprintln!(
                "Key '{}' now has {} match(es) with priorities: {:?}",
                key,
                matches.len(),
                matches.iter().map(|m| m.priority).collect::<Vec<_>>()
            );
        })
        .or_insert_with(|| vec![new_match]);

    Ok(())
}

pub fn get_cache_path() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let cache_path = dirs::cache_dir()
        .ok_or("Could not determine cache directory for this platform")?
        .join("hgnc_lookup")
        .join("hgnc_cache.bin");
    Ok(cache_path)
}

pub fn build_cache() -> Result<(), Box<dyn Error>> {
    // get cache directory and create if it doesn't exist
    let cache_path = get_cache_path()?;
    if let Some(parent) = cache_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // If the file already exists, skip downloading and building
    if cache_path.exists() {
        println!(
            "Cache file already exists at {:?}, skipping download and build.",
            cache_path
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
    let mut map: HashMap<String, Vec<Match>> = HashMap::new();
    let mut records: Vec<HgncRecord> = Vec::new();
    let mut record_idx = 0;

    for result in reader.deserialize() {
        let record: HgncRecord = result?;

        // Insert HGNC ID as Static (will error on collision)
        insert_match(
            &mut map,
            record.hgnc_id.clone(),
            KeyPriority::Static,
            record_idx,
        )?;

        // Insert Ensembl ID as Static (will error on collision)
        insert_match(
            &mut map,
            record.ensembl_gene_id.clone(),
            KeyPriority::Standard,
            record_idx,
        )?;

        // Insert standard symbol
        insert_match(
            &mut map,
            record.symbol.clone(),
            KeyPriority::Standard,
            record_idx,
        )?;

        // Insert previous symbols
        for prev in record.prev_symbol.split('|').filter(|s| !s.is_empty()) {
            insert_match(
                &mut map,
                prev.to_string(),
                KeyPriority::Previous,
                record_idx,
            )?;
        }

        // Insert alias symbols
        for alias in record.alias_symbol.split('|').filter(|s| !s.is_empty()) {
            insert_match(&mut map, alias.to_string(), KeyPriority::Alias, record_idx)?;
        }

        records.push(record);
        record_idx += 1;
    }

    let cache = Cache { records, map };
    let serialized = rkyv::to_bytes::<rkyv::rancor::Error>(&cache)?;

    // Write to file
    std::fs::write(cache_path, serialized)?;

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
