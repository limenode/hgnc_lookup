# hgnc_lookup

A fast, standalone command-line tool for HGNC gene symbol normalization and lookup.

`hgnc_lookup` builds a local, binary-cached lookup table from the official complete HGNC dataset and allows you to query for current HGNC symbols and associated information using a variety of input types.

## Features
* Fast lookups with zero-copy performance using an rkyv-backed binary cache.
* Automatic caching to user-level cache directory (i.e. `~/.cache/hgnc_lookup/`)
* Auto-downloads the HGNC complete dataset from the HGNC Google Cloud Storage Bucket.
* Retrieves all records for symbols with multiple matches 
  * Assigns priorities for gene symbol matching: official > previous > alias
  * Provides options to retrieve highest priority records only (default) or all matching records (`--all-matches`)
* Supports the following inputs:
  * HGNC IDs
  * Ensembl Gene IDs
  * Current HGNC symbols
  * Alias symbols
  * Previous symbols

## Installation
**Build from source**
```bash
git clone https://github.com/limenode/hgnc_lookup.git
cd hgnc_lookup
cargo build --release
```
The binary will be located at:
```text
./target/release/hgnc_lookup
```

## Data Source
This tool uses the HGNC complete gene set provided by the HUGO Gene Nomenclature Committee (HGNC).

- HGNC website: https://www.genenames.org/
- Download files: https://www.genenames.org/download/statistics-and-files/

The following URL is used by this program to retrieve the complete HGNC dataset:

* https://storage.googleapis.com/public-download-files/hgnc/tsv/tsv/hgnc_complete_set.txt

The metadata listed below will be output to stderr everytime the program is run:
* Last Modified Date and Time
* Cache Creation Date and Time
* HTTP Source URL
* HTTP ETag Header

To update the HGNC complete set to the most up-to-date version, run the binary with the following parameters:
```bash
./hgnc_lookup --clear-cache
```

## Citation / Attribution

If you use the HGNC data retrieved from this tool in published work, please cite HGNC according to their official citation guidelines:

https://www.genenames.org/help/faq/#!/#tocAnchor-1-1-7

## Acknowledgements

This project is built using several excellent Rust crates, including:

- [`rkyv`](https://github.com/rkyv/rkyv) for zero-copy serialization
- [`clap`](https://github.com/clap-rs/clap) for command-line parsing
- [`reqwest`](https://github.com/seanmonstar/reqwest) for HTTP requests
- [`dirs`](https://github.com/dirs-dev/dirs-rs) for cache directory resolution
