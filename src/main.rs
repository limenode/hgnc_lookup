mod cache_functions;
mod types;

use cache_functions::get_cache_path;
use cache_functions::get_fields_from_record;
use clap::{Parser, ValueEnum};
use indexmap::IndexMap;
use rkyv::rancor;
use std::error::Error;
use std::io::BufRead;
use std::path::PathBuf;
use types::{ArchivedCache, ArchivedHgncRecord};

#[derive(ValueEnum, Clone, Debug)]
enum OutputType {
    Pretty,
    Json,
    JsonPretty,
    Csv,
    Tsv,
}

/// Result of a query operation
enum QueryResult {
    Found(IndexMap<String, String>),
    NotFound(String),
}

#[derive(Parser, Debug)]
#[command(
    name = "hgnc_lookup",
    about = "HGNC symbol normalization / lookup tool",
    long_about = None
)]
struct Cli {
    /// Fields to query (e.g. hgnc_id, symbol, alias_symbol)
    #[arg(short, long, value_delimiter = ',')]
    fields: Option<Vec<String>>,

    /// Output format
    #[arg(short, long)]
    output_type: Option<OutputType>,

    /// Run benchmark before processing queries
    #[arg(long)]
    benchmark: bool,

    /// Clear cache file before building (forces re-download)
    #[arg(long)]
    clear_cache: bool,
}

fn benchmark_lookups(
    cache: &ArchivedCache,
    n: usize,
    percent_hits: f64,
) -> Result<(), Box<dyn Error>> {
    use rand::RngExt;
    use rand::prelude::{IndexedRandom, SliceRandom};
    use std::time::{Duration, Instant};

    println!("\n=== Benchmark: Average Lookup Time ===");

    // Collect all keys from the HashMap
    let all_keys: Vec<String> = cache.map.iter().map(|(k, _)| k.to_string()).collect();

    let total_keys = all_keys.len();
    println!("Total keys in cache: {}", total_keys);

    if total_keys == 0 {
        println!("No keys found in cache. Skipping benchmark.");
        return Ok(());
    }

    // Limit n to the number of available keys
    let n = n.min(total_keys);
    println!("Performing {} lookups...", n);

    // Randomly sample n keys, with 5% being random strings (misses)
    let mut rng = rand::rng();

    let n_hits = (n as f64 * percent_hits) as usize;
    let n_misses = n - n_hits;

    let mut sampled_keys: Vec<String> = all_keys.sample(&mut rng, n_hits).cloned().collect();

    // Generate random strings for misses
    for _ in 0..n_misses {
        let random_string: String = (0..10)
            .map(|_| rng.random_range(b'A'..=b'Z') as char)
            .collect();
        sampled_keys.push(random_string);
    }

    // Shuffle to mix hits and misses
    sampled_keys.shuffle(&mut rng);

    // Perform lookups and time each one
    let mut durations = Vec::with_capacity(n);
    let mut success_count = 0;

    for key in sampled_keys {
        let start = Instant::now();
        let result = query_map(cache, key.clone().as_str());
        let duration = start.elapsed();

        durations.push(duration);

        if result.is_some() {
            success_count += 1;
        } else {
            // Uncomment the line below to see which keys were misses
            // println!("Missed key: {}", key);
        }
    }

    // Calculate statistics
    let total_duration: Duration = durations.iter().sum();
    let avg_duration = total_duration / n as u32;

    durations.sort();
    let min_duration = durations.first().unwrap();
    let max_duration = durations.last().unwrap();
    let median_duration = durations[n / 2];
    let p95_duration = durations[(n * 95) / 100];
    let p99_duration = durations[(n * 99) / 100];

    // Report results
    println!("\n--- Benchmark Results ---");
    println!("Successful lookups: {} / {}", success_count, n);
    println!(
        "Success rate: {:.2}%",
        (success_count as f64 / n as f64) * 100.0
    );
    println!("Average lookup time: {:?}", avg_duration);
    println!("Median lookup time:  {:?}", median_duration);
    println!("Min lookup time:     {:?}", min_duration);
    println!("Max lookup time:     {:?}", max_duration);
    println!("95th percentile:     {:?}", p95_duration);
    println!("99th percentile:     {:?}", p99_duration);
    println!("Total time:          {:?}", total_duration);
    println!(
        "Lookups per second:  {:.2}",
        n as f64 / total_duration.as_secs_f64()
    );
    println!("=========================\n");

    Ok(())
}

fn query_map<'a>(cache: &'a ArchivedCache, query: &str) -> Option<&'a ArchivedHgncRecord> {
    let idx = cache.map.get(query.to_uppercase().as_str())?;
    cache.records.get(idx.to_native() as usize)
}

/// Process a query and return a QueryResult
fn process_query<'a>(
    cache: &'a ArchivedCache,
    query: &str,
    fields: &Option<Vec<String>>,
) -> QueryResult {
    match query_map(cache, query) {
        Some(record) => {
            let record_map = get_fields_from_record(record, fields);
            QueryResult::Found(record_map)
        }
        None => QueryResult::NotFound(query.to_string()),
    }
}

/// Print a query result in the specified format
fn print_query_result(
    result: &QueryResult,
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    match (result, output_type) {
        // Found cases
        (QueryResult::Found(record_map), OutputType::Pretty) => {
            for (field, value) in record_map {
                println!("{}: {}", field, value);
            }
        }
        (QueryResult::Found(record_map), OutputType::Json) => {
            let json = serde_json::to_string(&record_map)?;
            println!("{},", json);
        }
        (QueryResult::Found(record_map), OutputType::JsonPretty) => {
            let json = serde_json::to_string_pretty(&record_map)?;
            println!("{},", json);
        }
        (QueryResult::Found(record_map), OutputType::Csv) => {
            let mut wtr = csv::WriterBuilder::new().from_writer(std::io::stdout());
            wtr.write_record(record_map.keys())?;
            wtr.write_record(record_map.values())?;
            wtr.flush()?;
        }
        (QueryResult::Found(record_map), OutputType::Tsv) => {
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .from_writer(std::io::stdout());
            wtr.write_record(record_map.keys())?;
            wtr.write_record(record_map.values())?;
            wtr.flush()?;
        }

        // NotFound cases
        (QueryResult::NotFound(query), OutputType::Json) => {
            let error_obj = serde_json::json!({
                "error": "not_found",
                "query": query,
                "message": format!("No record found for query: {}", query)
            });
            let json = serde_json::to_string(&error_obj)?;
            println!("{}", json);
        }
        (QueryResult::NotFound(query), OutputType::JsonPretty) => {
            let error_obj = serde_json::json!({
                "error": "not_found",
                "query": query,
                "message": format!("No record found for query: {}", query)
            });
            let json = serde_json::to_string_pretty(&error_obj)?;
            println!("{},", json);
        }
        (QueryResult::NotFound(query), OutputType::Pretty | OutputType::Csv | OutputType::Tsv) => {
            eprintln!("No record found for query: {}", query);
        }
    }
    Ok(())
}

fn main() {
    let args = Cli::parse();

    let cache_path: PathBuf = get_cache_path().expect("Failed to determine cache path");

    // Clear cache file if --clear-cache is set
    if args.clear_cache {
        if (&cache_path).exists() {
            std::fs::remove_file(&cache_path).expect("Failed to clear cache file");
            // println!("Cache file cleared: {:?}", cache_path);
        } else {
            // println!("No cache file to clear at: {:?}", cache_path);
        }
    }

    // Build cache (download and serialize) if it doesn't exist
    cache_functions::build_cache().expect("Failed to download cache");

    // Load cache from file
    let bytes = std::fs::read(&cache_path).expect("Failed to read cache file");
    let archived_cache = rkyv::access::<ArchivedCache, rancor::Error>(&bytes).unwrap();

    // Optionally benchmark lookups
    if args.benchmark {
        benchmark_lookups(archived_cache, 100000, 0.95).expect("Failed to benchmark lookups");
    }

    // Determine output type (default to Pretty)
    let output_type = args.output_type.unwrap_or(OutputType::Pretty);
    println!("Output type: {:?}", output_type);

    // Read queries from stdin (one per line)
    let stdin = std::io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        };

        // Trim the line and skip if it's empty
        let query: &str = line.trim();
        if query.is_empty() {
            continue; // Skip empty lines
        }

        // Process the query and get the result
        let result = process_query(archived_cache, query, &args.fields);

        // Print the result in the specified format
        if let Err(e) = print_query_result(&result, &output_type) {
            eprintln!("Error printing result: {}", e);
        }
    }
}
