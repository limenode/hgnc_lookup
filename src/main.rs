mod cache_functions;
mod types;

use cache_functions::get_fields_from_record;
use clap::Parser;
use rkyv::rancor;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use types::{ArchivedCache, ArchivedHgncRecord};

#[derive(Parser, Debug)]
#[command(
    name = "hgnc_lookup",
    about = "HGNC symbol normalization / lookup tool",
    long_about = None
)]
struct Cli {
    /// Input file
    input_file: Option<String>,

    /// Fields to query (e.g. hgnc_id, symbol, alias_symbol)
    #[arg(short, long, value_delimiter = ',')]
    fields: Option<Vec<String>>,
}

fn benchmark_lookups(cache: &ArchivedCache, n: usize) -> Result<(), Box<dyn Error>> {
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

    let n_hits = (n as f64 * 1.00) as usize;
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
            println!("Missed key: {}", key);
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

fn run_interactive(
    cache: &ArchivedCache,
    fields: Option<Vec<String>>,
) -> Result<(), Box<dyn Error>> {
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let query: &str = line.as_ref().unwrap();
        println!("Query: {}", query);
        let record = query_map(cache, query);
        match record {
            Some(record) => {
                println!("Found: {}", record.symbol);
                for value in get_fields_from_record(record, &fields) {
                    println!("  {}", value);
                }
            }
            None => println!("No record found for query: {}", query),
        }
    }

    Ok(())
}

fn main() {
    let args = Cli::parse();
    let cache_path: &Path = Path::new("hgnc_cache.bin");
    cache_functions::build_cache(cache_path).expect("Failed to download cache");

    let bytes = std::fs::read(cache_path).expect("Failed to read cache file");
    let archived_cache = rkyv::access::<ArchivedCache, rancor::Error>(&bytes).unwrap();

    benchmark_lookups(archived_cache, 10000).expect("Failed to benchmark lookups");

    if args.input_file.is_some() {
        // Read lines as queries from the input file
        let file = File::open(args.input_file.as_ref().unwrap());
        let reader = BufReader::new(file.expect("Failed to open input file"));
        for line in reader.lines() {
            let query: &str = line.as_ref().unwrap();
            if query.trim().is_empty() {
                continue; // Skip empty lines
            }
            println!("Query: {}", query);
            let record = query_map(archived_cache, query);
            match record {
                Some(record) => {
                    println!("Found: {}", record.symbol);
                    for value in get_fields_from_record(record, &args.fields) {
                        println!("  {}", value);
                    }
                }
                None => println!("No record found for query: {}", query),
            }
        }
    } else {
        run_interactive(archived_cache, args.fields).expect("Failed to run interactive mode");
    }
}
