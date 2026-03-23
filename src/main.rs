mod cache_functions;
mod types;

use cache_functions::get_cache_path;
use clap::{Parser, ValueEnum};
use rkyv::rancor;
use std::error::Error;
use std::io::BufRead;
use std::path::PathBuf;
use types::ArchivedCache;

use crate::types::{ALL_FIELDS, ArchivedHgncRecord, Field, MatchSelection};

#[derive(ValueEnum, Clone, Debug)]
enum OutputType {
    Pretty,
    Json,
    JsonPretty,
    Csv,
    Tsv,
}

/// Result of a query operation
enum QueryResult<'q, 'r> {
    Found(&'q str, Vec<&'r ArchivedHgncRecord>), // Query and list of matching records
    NotFound(&'q str),
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

    /// Specify number of lookups for benchmark (default: 10,000)
    #[arg(long, default_value_t = 10000)]
    benchmark_lookups: usize,

    /// Specify percentage of hits vs misses for benchmark (default: 95.0)
    #[arg(long, default_value_t = 95.0)]
    benchmark_hits: f64,

    /// Clear cache file before building (forces re-download)
    #[arg(long)]
    clear_cache: bool,

    /// Return all matches instead of just the highest priority one
    #[arg(long)]
    all_matches: bool,

    /// Don't print out headers/labels in the output
    #[arg(long)]
    no_header: bool,
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
        let result = lookup_gene(cache, &key, MatchSelection::All);
        let duration = start.elapsed();

        durations.push(duration);

        if matches!(result, QueryResult::Found(_, _)) {
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

    Ok(())
}

fn lookup_gene<'q, 'r>(
    cache: &'r ArchivedCache,
    query: &'q str,
    selection: MatchSelection,
) -> QueryResult<'q, 'r> {
    let records: Vec<_> = cache.matching_records(query, selection).collect();

    if records.is_empty() {
        QueryResult::NotFound(query)
    } else {
        QueryResult::Found(query, records)
    }
}

/// Print a query result in the specified format
fn print_query_result(
    result: &QueryResult,
    fields: &[Field],
    output_type: &OutputType,
    no_header: bool,
) -> Result<(), Box<dyn Error>> {
    match (result, output_type) {
        // Found cases
        (QueryResult::Found(query, records), OutputType::Pretty) => {
            println!("Query: {}", query);
            println!("Found {} match(es)\n", records.len());

            for (idx, record) in records.iter().enumerate() {
                if idx > 0 {
                    println!("\n{}", "=".repeat(80));
                    println!();
                }

                for (field, value) in record.selected_fields(fields) {
                    if no_header {
                        println!("{}", value);
                    } else {
                        println!("{}: {}", field, value);
                    }
                }
            }
        }
        (QueryResult::Found(query, records), OutputType::Json) => {
            let records_json: Vec<_> = records
                .iter()
                .map(|record| {
                    let mut obj = serde_json::Map::new();
                    for (field, value) in record.selected_fields(fields) {
                        obj.insert(
                            field.to_string(),
                            serde_json::Value::String(value.to_string()),
                        );
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();
            let output = serde_json::json!({
                "query": query,
                "count": records.len(),
                "records": records_json
            });
            println!("{}", serde_json::to_string(&output)?);
        }
        (QueryResult::Found(query, records), OutputType::JsonPretty) => {
            let records_json: Vec<_> = records
                .iter()
                .map(|record| {
                    let mut obj = serde_json::Map::new();
                    for (field, value) in record.selected_fields(fields) {
                        obj.insert(
                            field.to_string(),
                            serde_json::Value::String(value.to_string()),
                        );
                    }
                    serde_json::Value::Object(obj)
                })
                .collect();
            let output = serde_json::json!({
                "query": query,
                "count": records.len(),
                "records": records_json
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        (QueryResult::Found(query, records), OutputType::Csv) => {
            let mut wtr = csv::WriterBuilder::new().from_writer(std::io::stdout());

            if !no_header {
                let mut headers = Vec::with_capacity(fields.len() + 1);
                headers.push("query");
                headers.extend(fields.iter().map(Field::as_str));
                wtr.write_record(&headers)?;
            }

            for record in records {
                let mut row = Vec::with_capacity(fields.len() + 1);
                row.push(*query);
                row.extend(fields.iter().map(|field| record.field_value(*field)));
                wtr.write_record(&row)?;
            }

            wtr.flush()?;
        }
        (QueryResult::Found(query, records), OutputType::Tsv) => {
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .from_writer(std::io::stdout());

            if !no_header {
                let mut headers = Vec::with_capacity(fields.len() + 1);
                headers.push("query");
                headers.extend(fields.iter().map(Field::as_str));
                wtr.write_record(&headers)?;
            }
            for record in records {
                let mut row = Vec::with_capacity(fields.len() + 1);
                row.push(*query);
                row.extend(fields.iter().map(|field| record.field_value(*field)));
                wtr.write_record(&row)?;
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

    // Parse CLI fields to Vec<Fields>
    let fields: Vec<Field> = match args.fields {
        Some(field_strs) => field_strs.iter().filter_map(|s| Field::parse(s)).collect(),
        None => ALL_FIELDS.to_vec(), // Use all fields if none specified
    };

    // Optionally benchmark lookups
    if args.benchmark {
        benchmark_lookups(
            archived_cache,
            args.benchmark_lookups,
            args.benchmark_hits / 100.0,
        )
        .expect("Failed to benchmark lookups");
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

        let selection = if args.all_matches {
            MatchSelection::All
        } else {
            MatchSelection::HighestPriority
        };

        // Process the query and get the result
        let result = lookup_gene(archived_cache, query, selection);

        // Print the result in the specified format
        if let Err(e) = print_query_result(&result, &fields, &output_type, args.no_header) {
            eprintln!("Error printing result: {}", e);
        }
    }
}
