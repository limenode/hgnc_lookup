use clap::Parser;
use hgnc_lookup::{hgnc_cache_functions, hgnc_struct, query_lookup_table};
use std::error::Error;
use std::io;
use std::io::BufRead;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(
    name = "hgnc_lookup",
    version,
    about = "HGNC symbol normalization / lookup tool",
    long_about = None
)]
struct Cli {
    /// Enter interactive mode (read queries from stdin, one per line)
    #[arg(long)]
    interactive: bool,

    /// Provide a path to hgnc_complete_set.txt (or equivalent) and rebuild cache from it
    #[arg(long = "set-file", value_name = "PATH")]
    set_file: Option<PathBuf>,

    /// Rebuild the cache and ignore any existing cache file
    #[arg(long)]
    force_rebuild: bool,

    /// Delete the cache file and exit
    #[arg(long)]
    delete_cache: bool,
}

fn maybe_delete_cache_bin(force: bool) -> Result<(), Box<dyn Error>> {
    if !force {
        return Ok(());
    }

    let bin_path = hgnc_cache_functions::get_hgnc_bin_cache_path()?;
    if bin_path.exists() {
        std::fs::remove_file(&bin_path)?;
        eprintln!("Deleted cache file at: {:?}", bin_path);
    } else {
        eprintln!("No cache file to delete at: {:?}", bin_path);
    }
    Ok(())
}

fn run_interactive(
    hgnc_cache: &'static crate::hgnc_struct::ArchivedHgncCache,
) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let query = line?;

        let start = Instant::now();
        let query_res = query_lookup_table(query, hgnc_cache);
        let duration = start.elapsed();

        match query_res {
            Ok(record) => {
                println!(
                    "Found record: HGNC ID: {}, Symbol: {}, Name: {}",
                    record.hgnc_id, record.symbol, record.name
                );
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        println!("Query took: {:?}", duration);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // Delete cache first for rebuild if requested
    maybe_delete_cache_bin(cli.force_rebuild || cli.delete_cache)?;

    // Exit if only deleting cache
    if cli.delete_cache {
        return Ok(());
    }

    // Load cache
    let start = Instant::now();
    let hgnc_cache = match cli.set_file {
        Some(ref path) => hgnc_cache_functions::get_hgnc_cache(Some(path))?,
        None => hgnc_cache_functions::get_hgnc_cache::<PathBuf>(None)?,
    };
    let duration = start.elapsed();
    println!("HGNC cache is ready. Load took: {:?}", duration);

    // Default behavior: interactive if no explicit mode chosen
    if cli.interactive || (cli.set_file.is_none()) {
        run_interactive(hgnc_cache)?;
    } else {
        println!("Cache is ready. (Non-interactive mode placeholder.)");
    }

    Ok(())
}
