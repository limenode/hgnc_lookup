mod cache_functions;
mod types;

use cache_functions::{get_cache_path, get_fields_from_record};
use clap::{Parser, ValueEnum};
use indexmap::IndexMap;
use rkyv::rancor;
use std::error::Error;
use std::io::BufRead;
use std::path::PathBuf;
use types::ArchivedCache;

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
    Found(String, Vec<IndexMap<String, String>>),
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

    /// Return all matches instead of just the highest priority one
    #[arg(long)]
    all_matches: bool,
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
        let result = lookup_gene(cache, &key, false, &None);
        let duration = start.elapsed();

        durations.push(duration);

        if matches!(result, QueryResult::Found(_, _)) {
            success_count += 1;
            // print!("woop woop")
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

fn lookup_gene(
    cache: &ArchivedCache,
    query: &str,
    return_all: bool,
    fields: &Option<Vec<String>>,
) -> QueryResult {
    // Get appropriate indices iterator
    let indices = cache.get_indices(query, return_all);

    match indices {
        Some(indices) => {
            let mut records = Vec::new();

            for idx in indices {
                if let Some(record) = cache.records.get(idx) {
                    let record_map = get_fields_from_record(record, fields);
                    records.push(record_map);
                }
            }

            if records.is_empty() {
                QueryResult::NotFound(query.to_string())
            } else {
                QueryResult::Found(query.to_string(), records)
            }
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
        (QueryResult::Found(query, records), OutputType::Pretty) => {
            println!("Query: {}", query);
            println!("Found {} match(es)\n", records.len());

            for (idx, record_map) in records.iter().enumerate() {
                if idx > 0 {
                    println!("\n{}", "=".repeat(80));
                    println!();
                }

                for (field, value) in record_map {
                    println!("{}: {}", field, value);
                }
            }
        }
        (QueryResult::Found(query, records), OutputType::Json) => {
            // Output as array of objects for easy parsing
            let output = serde_json::json!({
                "query": query,
                "count": records.len(),
                "records": records
            });
            println!("{}", serde_json::to_string(&output)?);
        }
        (QueryResult::Found(query, records), OutputType::JsonPretty) => {
            // Output as array of objects for easy parsing
            let output = serde_json::json!({
                "query": query,
                "count": records.len(),
                "records": records
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        (QueryResult::Found(query, records), OutputType::Csv) => {
            let mut wtr = csv::WriterBuilder::new().from_writer(std::io::stdout());

            // Get all field names (assuming all records have same fields)
            if let Some(first_record) = records.first() {
                // Write header with "query" as first column
                let mut headers = vec!["query".to_string()];
                headers.extend(first_record.keys().map(|k| k.to_string()));
                wtr.write_record(&headers)?;

                // Write each record as a row
                for record_map in records {
                    let mut row = vec![query.to_string()];
                    row.extend(record_map.values().map(|v| v.to_string()));
                    wtr.write_record(&row)?;
                }
            }

            wtr.flush()?;
        }
        (QueryResult::Found(query, records), OutputType::Tsv) => {
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .from_writer(std::io::stdout());

            // Get all field names (assuming all records have same fields)
            if let Some(first_record) = records.first() {
                // Write header with "query" as first column
                let mut headers = vec!["query".to_string()];
                headers.extend(first_record.keys().map(|k| k.to_string()));
                wtr.write_record(&headers)?;

                // Write each record as a row
                for record_map in records {
                    let mut row = vec![query.to_string()];
                    row.extend(record_map.values().map(|v| v.to_string()));
                    wtr.write_record(&row)?;
                }
            }

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
        benchmark_lookups(archived_cache, 10000, 0.95).expect("Failed to benchmark lookups");
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
        let result = lookup_gene(archived_cache, query, args.all_matches, &args.fields);

        // Print the result in the specified format
        if let Err(e) = print_query_result(&result, &output_type) {
            eprintln!("Error printing result: {}", e);
        }
    }
}
