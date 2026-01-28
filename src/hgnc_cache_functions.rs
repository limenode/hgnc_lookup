use crate::hgnc_struct::{ArchivedHgncCache, HgncCache, HgncRecord};
use rkyv::rancor;

use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const HGNC_COMPLETE_SET_URL: &str =
    "https://storage.googleapis.com/public-download-files/hgnc/tsv/tsv/hgnc_complete_set.txt";

/// Resolve ~/.cache/hgnc_lookup/hgnc_complete_set.bin (Linux) using a best-effort approach.
pub fn get_hgnc_bin_cache_path() -> Result<PathBuf, Box<dyn Error>> {
    let base_cache = dirs::cache_dir().ok_or("Could not determine user cache directory")?;
    Ok(base_cache.join("hgnc_lookup").join("hgnc_complete_set.bin"))
}

/// Ensure the parent directory exists.
fn ensure_parent_dir(path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Loads HGNC data from a tab-delimited file into an HgncCache
/// All mappings point to the index of the record in the records vector.
pub fn create_hgnc_cache_from_reader<R: BufRead>(reader: R) -> Result<HgncCache, Box<dyn Error>> {
    let mut lines = reader.lines();

    // Read and parse header
    let header_line = lines.next().ok_or("File is empty")??;
    let headers: Vec<&str> = header_line.split('\t').collect();

    // Find column indices
    let mut col_map = std::collections::HashMap::new();
    for (i, header) in headers.iter().enumerate() {
        col_map.insert(*header, i);
    }

    let mut cache = HgncCache {
        records: Vec::new(),
        lookup: std::collections::HashMap::new(),
    };

    // Process each data line
    for line_result in lines {
        let line = line_result?;
        let fields: Vec<&str> = line.split('\t').collect();

        // Helper function to get field value or empty string
        let get_field = |field_name: &str| -> String {
            col_map
                .get(field_name)
                .and_then(|&idx| fields.get(idx))
                .unwrap_or(&"")
                .to_string()
        };

        // Create HgncRecord from fields
        let record = HgncRecord {
            hgnc_id: get_field("hgnc_id"),
            symbol: get_field("symbol"),
            name: get_field("name"),
            locus_group: get_field("locus_group"),
            locus_type: get_field("locus_type"),
            status: get_field("status"),
            location: get_field("location"),
            location_sortable: get_field("location_sortable"),
            alias_symbol: get_field("alias_symbol"),
            alias_name: get_field("alias_name"),
            prev_symbol: get_field("prev_symbol"),
            prev_name: get_field("prev_name"),
            gene_group: get_field("gene_group"),
            gene_group_id: get_field("gene_group_id"),
            date_approved_reserved: get_field("date_approved_reserved"),
            date_symbol_changed: get_field("date_symbol_changed"),
            date_name_changed: get_field("date_name_changed"),
            date_modified: get_field("date_modified"),
            entrez_id: get_field("entrez_id"),
            ensembl_gene_id: get_field("ensembl_gene_id"),
            vega_id: get_field("vega_id"),
            ucsc_id: get_field("ucsc_id"),
            ena: get_field("ena"),
            refseq_accession: get_field("refseq_accession"),
            ccds_id: get_field("ccds_id"),
            uniprot_ids: get_field("uniprot_ids"),
            pubmed_id: get_field("pubmed_id"),
            mgd_id: get_field("mgd_id"),
            rgd_id: get_field("rgd_id"),
            lsdb: get_field("lsdb"),
            cosmic: get_field("cosmic"),
            omim_id: get_field("omim_id"),
            mirbase: get_field("mirbase"),
            homeodb: get_field("homeodb"),
            snornabase: get_field("snornabase"),
            bioparadigms_slc: get_field("bioparadigms_slc"),
            orphanet: get_field("orphanet"),
            pseudogene_org: get_field("pseudogene.org"),
            horde_id: get_field("horde_id"),
            merops: get_field("merops"),
            imgt: get_field("imgt"),
            iuphar: get_field("iuphar"),
            kznf_gene_catalog: get_field("kznf_gene_catalog"),
            mamit_trnadb: get_field("mamit-trnadb"),
            cd: get_field("cd"),
            lncrnadb: get_field("lncrnadb"),
            enzyme_id: get_field("enzyme_id"),
            intermediate_filament_db: get_field("intermediate_filament_db"),
            rna_central_id: get_field("rna_central_id"),
            lncipedia: get_field("lncipedia"),
            gtrnadb: get_field("gtrnadb"),
            agr: get_field("agr"),
            mane_select: get_field("mane_select"),
            gencc: get_field("gencc"),
        };

        // Get the index where this record will be stored
        let record_idx = cache.records.len();

        // Add mappings to lookup

        // 1. HGNC symbol
        cache
            .lookup
            .insert(record.symbol.to_uppercase(), record_idx);

        // 2. Alias symbols (pipe-delimited)
        if !record.alias_symbol.is_empty() {
            for alias in record.alias_symbol.split('|').filter(|s| !s.is_empty()) {
                cache.lookup.insert(alias.trim().to_uppercase(), record_idx);
            }
        }

        // 3. Previous symbols (pipe-delimited)
        if !record.prev_symbol.is_empty() {
            for prev in record.prev_symbol.split('|').filter(|s| !s.is_empty()) {
                cache.lookup.insert(prev.trim().to_uppercase(), record_idx);
            }
        }

        // Add the record to the cache
        cache.records.push(record);
    }

    Ok(cache)
}

pub fn create_hgnc_cache<P: AsRef<Path>>(file_path: P) -> Result<HgncCache, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    create_hgnc_cache_from_reader(reader)
}

fn download_hgnc_complete_set_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    // blocking client keeps integration simple for a CLI / library call
    let resp = reqwest::blocking::get(HGNC_COMPLETE_SET_URL)?;
    if !resp.status().is_success() {
        return Err(format!("HGNC download failed: HTTP {}", resp.status()).into());
    }
    let bytes = resp.bytes()?;
    Ok(bytes.to_vec())
}

pub fn dump_hgnc_cache<P: AsRef<Path>>(
    cache: &HgncCache,
    output_path: P,
) -> Result<(), Box<dyn Error>> {
    let _bytes = rkyv::to_bytes::<rancor::Error>(cache).unwrap();
    std::fs::write(output_path, _bytes)?;
    Ok(())
}

pub fn load_hgnc_cache<P: AsRef<Path>>(
    input_path: P,
) -> Result<&'static ArchivedHgncCache, Box<dyn Error>> {
    let bytes = std::fs::read(input_path)?;
    let leaked_bytes = Box::leak(bytes.into_boxed_slice());
    let archived = rkyv::access::<ArchivedHgncCache, rancor::Error>(leaked_bytes).unwrap();
    Ok(archived)
}

pub fn get_hgnc_cache<P: AsRef<Path>>(
    path: Option<P>,
) -> Result<&'static ArchivedHgncCache, Box<dyn Error>> {
    let bin_path = get_hgnc_bin_cache_path()?;
    ensure_parent_dir(&bin_path)?;

    let cache: HgncCache = match path {
        Some(p) => {
            // If file path is provided, create cache and dump to bin_path; will overwrite existing cache
            println!("Creating HGNC cache from text file: {:?}", p.as_ref());
            create_hgnc_cache(p)?
        }
        None => {
            // Check if cache file exists
            // If it does, we can load it directly
            if bin_path.exists() {
                println!("HGNC cache file found at {:?}, loading directly.", bin_path);
                return load_hgnc_cache(&bin_path);
            }
            // Otherwise, download and create cache
            println!("Downloading HGNC complete set into memory...");
            let bytes = download_hgnc_complete_set_bytes()?;
            let reader = BufReader::new(std::io::Cursor::new(bytes));
            println!("Creating HGNC cache from downloaded data...");
            create_hgnc_cache_from_reader(reader)?
        }
    };

    println!("Dumping HGNC cache to: {:?}", bin_path);
    dump_hgnc_cache(&cache, &bin_path)?;

    println!("Loading HGNC cache from: {:?}", bin_path);
    let archived_cache = load_hgnc_cache(&bin_path)?;
    println!("HGNC cache is ready.");

    Ok(archived_cache)
}
