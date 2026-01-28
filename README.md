# hgnc_lookup

A fast, standalone command-line tool for HGNC gene symbol normalization and lookup.

`hgnc_lookup` builds a local, binary-cached lookup table from the official complete HGNC dataset and allows you to resolve current HGNC symbols and associated information from aliases and previous symbols.

## Features
* Normalize gene symbols to official HGNC symbols
* Fast lookups with zero-copy performance using an rkyv-backed binary cache.
* Automatic caching to user-level cache directory (`~/.cache/hgnc_lookup/`)
* Auto-downloads the HGNC complete dataset from the HGNC Google Cloud Storage Bucket.
* Supports the following inputs:
  * current HGNC symbols
  * alias symbols
  * previous symbols

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

## Citation / Attribution

If you use the HGNC data retrieved from this tool in published work, please cite HGNC according to their official citation guidelines:

https://www.genenames.org/help/faq/#!/#tocAnchor-1-1-7

## Acknowledgements

This project is built using several excellent Rust crates, including:

- [`rkyv`](https://github.com/rkyv/rkyv) for zero-copy serialization
- [`clap`](https://github.com/clap-rs/clap) for command-line parsing
- [`reqwest`](https://github.com/seanmonstar/reqwest) for HTTP requests
- [`dirs`](https://github.com/dirs-dev/dirs-rs) for cache directory resolution
