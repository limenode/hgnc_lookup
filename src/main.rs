mod cache_functions;
mod types;

use cache_functions::get_cache_path;
use clap::{Parser, ValueEnum};
use rkyv::rancor;
use std::error::Error;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
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
    /// Queries file with symbols/IDs to lookup
    #[arg(short, long, default_value = "-")]
    query_file: Option<PathBuf>,

    /// Fields to query (e.g. hgnc_id, symbol, alias_symbol)
    #[arg(short, long, value_delimiter = ',')]
    fields: Option<Vec<String>>,

    /// Fields file delimited by newlines
    #[arg(long)]
    fields_file: Option<PathBuf>,

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

    eprintln!("\n=== Benchmark ===");

    // Collect all keys from the HashMap
    let all_keys: Vec<String> = cache.map.iter().map(|(k, _)| k.to_string()).collect();

    let total_keys = all_keys.len();
    eprintln!("Total keys in cache: {}", total_keys);

    if total_keys == 0 {
        eprintln!("No keys found in cache. Skipping benchmark.");
        return Ok(());
    }

    // Limit n to the number of available keys
    let n = n.min(total_keys);
    eprintln!("Performing {} lookups...", n);

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
    eprintln!("\n--- Benchmark Results ---");
    eprintln!("Successful lookups: {} / {}", success_count, n);
    eprintln!(
        "Success rate: {:.2}%",
        (success_count as f64 / n as f64) * 100.0
    );
    eprintln!("Average lookup time: {:?}", avg_duration);
    eprintln!("Median lookup time:  {:?}", median_duration);
    eprintln!("Min lookup time:     {:?}", min_duration);
    eprintln!("Max lookup time:     {:?}", max_duration);
    eprintln!("95th percentile:     {:?}", p95_duration);
    eprintln!("99th percentile:     {:?}", p99_duration);
    eprintln!("Total time:          {:?}", total_duration);
    eprintln!(
        "Lookups per second:  {:.2}\n",
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

fn print_delimited_helper(
    query: &str,
    records: &[&ArchivedHgncRecord],
    fields: &[Field],
    delimiter: u8,
) {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(std::io::stdout());

    for record in records {
        let row =
            std::iter::once(query).chain(fields.iter().map(|field| record.field_value(*field)));
        wtr.write_record(row).expect("Failed to write record");
    }

    wtr.flush().expect("Failed to flush writer");
}

fn print_json_helper(
    query: &str,
    records: &[&ArchivedHgncRecord],
    fields: &[Field],
    pretty: bool,
) -> Result<(), Box<dyn Error>> {
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

    if pretty {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", serde_json::to_string(&output)?);
    }

    Ok(())
}

fn print_json_error_helper(query: &str, pretty: bool) -> Result<(), Box<dyn Error>> {
    let error_obj = serde_json::json!({
        "error": "not_found",
        "query": query,
        "message": format!("No record found for query: {}", query)
    });
    let json = serde_json::to_string(&error_obj)?;

    if pretty {
        println!("{}", serde_json::to_string_pretty(&error_obj)?);
    } else {
        println!("{}", json);
    }

    Ok(())
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
            print_json_helper(query, records, fields, false)?;
        }
        (QueryResult::Found(query, records), OutputType::JsonPretty) => {
            print_json_helper(query, records, fields, true)?;
        }
        (QueryResult::Found(query, records), OutputType::Csv) => {
            print_delimited_helper(query, records, fields, b',');
        }
        (QueryResult::Found(query, records), OutputType::Tsv) => {
            print_delimited_helper(query, records, fields, b'\t');
        }

        // NotFound cases
        (QueryResult::NotFound(query), OutputType::Json) => {
            print_json_error_helper(query, false)?;
        }
        (QueryResult::NotFound(query), OutputType::JsonPretty) => {
            print_json_error_helper(query, true)?;
        }
        (QueryResult::NotFound(query), OutputType::Pretty | OutputType::Csv | OutputType::Tsv) => {
            eprintln!("No record found for query: {}", query);
        }
    }
    Ok(())
}

fn read_fields_from_source(path: &Path) -> Result<Vec<Field>, Box<dyn std::error::Error>> {
    let reader: Box<dyn BufRead> = if path.as_os_str() == "-" {
        Box::new(BufReader::new(io::stdin().lock()))
    } else {
        Box::new(BufReader::new(std::fs::File::open(path)?))
    };

    let mut out = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line?;
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') {
            continue;
        }
        let field = Field::parse(s)
            .ok_or_else(|| format!("Invalid field '{}' at {}:{}", s, path.display(), idx + 1))?;
        out.push(field);
    }
    Ok(out)
}

fn main() {
    let args = Cli::parse();

    // Fail if both --query-file and --fields-file requests stdin (i.e. "-")
    let query_is_stdin = args.query_file.as_deref() == Some(Path::new("-"));
    let fields_is_stdin = args.fields_file.as_deref() == Some(Path::new("-"));

    if query_is_stdin && fields_is_stdin {
        eprintln!("Error: Both --query-file and --fields-file cannot be '-' (stdin)");
        eprintln!(
            "Please provide one of them as a file path or remove one of the options to read from stdin"
        );
        std::process::exit(1);
    }

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
    let mut all_fields: Vec<Field> = if let Some(path) = &args.fields_file {
        read_fields_from_source(path).expect("Failed to read fields from --fields-file")
    } else {
        Vec::new()
    };

    // 2. Extend with fields from --fields if provided
    // If no fields are provided from either source, default to ALL_FIELDS
    match args.fields {
        Some(field_strs) => {
            let new_fields: Vec<Field> = field_strs
                .iter()
                .filter_map(|s| {
                    let field_str = s.trim();
                    if field_str.is_empty() {
                        None
                    } else {
                        Field::parse(field_str)
                    }
                })
                .collect();
            all_fields.extend(new_fields);
        }
        None => {
            if all_fields.is_empty() {
                all_fields = ALL_FIELDS.to_vec();
            }
        }
    };

    // Remove duplicates while preserving order
    let mut seen = std::collections::HashSet::new();
    all_fields.retain(|field| seen.insert(*field));

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
    eprintln!("Output type: {:?}", output_type);

    // Read queries from --query-file
    let reader: Box<dyn BufRead> = if let Some(path) = &args.query_file {
        if path.as_os_str() == "-" {
            Box::new(BufReader::new(io::stdin().lock()))
        } else {
            Box::new(BufReader::new(
                std::fs::File::open(path).expect("Failed to open query file"),
            ))
        }
    } else {
        Box::new(BufReader::new(io::stdin().lock()))
    };

    let mut no_header = args.no_header;

    // If not no header, print header for CSV/TSV output once
    if !args.no_header {
        if let OutputType::Csv | OutputType::Tsv = output_type {
            let mut headers = Vec::with_capacity(all_fields.len() + 1);
            headers.push("query");
            headers.extend(all_fields.iter().map(Field::as_str));
            let delimiter = match output_type {
                OutputType::Csv => b',',
                OutputType::Tsv => b'\t',
                _ => unreachable!(),
            };
            let mut wtr = csv::WriterBuilder::new()
                .delimiter(delimiter)
                .from_writer(std::io::stdout());
            wtr.write_record(&headers).expect("Failed to write header");
            wtr.flush().expect("Failed to flush header");

            no_header = true; // Set no_header to true to avoid printing headers again in the loop
        }
    }

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
        if let Err(e) = print_query_result(&result, &all_fields, &output_type, no_header) {
            eprintln!("Error printing result: {}", e);
        }
    }
}
