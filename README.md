# read_check

A fast, lightweight tool for computing quality statistics on Oxford Nanopore Technology (ONT) FASTQ reads. Designed to run as a pre-assembly QC step, producing a YAML summary that downstream tools and pipelines can parse easily.

## Usage

```bash
read_check [OPTIONS] <INPUT>
```

### Arguments

| Argument | Description |
|---|---|
| `<INPUT>` | Path to a FASTQ file (plain or gzipped `.fastq` / `.fastq.gz`) |

### Options

| Flag | Description |
|---|---|
| `-g`, `--genome-size` | Enable k-mer-based genome size estimation (see below) |
| `-o`, `--output <FILE>` | Output YAML file path (default: `read_stats.yaml`) |

### Examples

Basic read stats:
```bash
read_check reads.fastq.gz
```

With genome size estimation:
```bash
read_check -g reads.fastq.gz -o qc_stats.yaml
```

## Output

Results are written as YAML. Without `-g`:

```yaml
total_reads: 85432
total_bases: 1023456789
n50: 15234
genome_size: null
unique_kmers: 0
unique_read_lengths: 4201
```

With `-g`:

```yaml
total_reads: 85432
total_bases: 1023456789
n50: 15234
genome_size: 4800000
unique_kmers: 3821045
unique_read_lengths: 4201
```

### Fields

| Field | Description |
|---|---|
| `total_reads` | Number of reads in the input file |
| `total_bases` | Total number of bases across all reads |
| `n50` | Read length N50: the length at which 50% of all bases are contained in reads of this length or longer |
| `genome_size` | Estimated genome size in bases derived from k-mer depth (null if `-g` not supplied) |
| `unique_kmers` | Number of unique canonical 21-mers observed (after error filtering; 0 if `-g` not supplied) |
| `unique_read_lengths` | Number of distinct read lengths observed |

## Genome Size Estimation

When `-g` is provided, `read_check` performs a k-mer frequency analysis to estimate genome size:

1. **K-mer hashing** — canonical 21-mers (the lexicographic minimum of forward and reverse-complement) are hashed into a hashmap using a fast non-cryptographic hash (FxHashMap). To reduce memory pressure on large datasets, k-mers are only sampled from every 5th read.
2. **Error filtering** — k-mers with a count of ≤ 5 are discarded, removing the bulk of sequencing errors which appear at very low frequency.
3. **Peak depth detection** — a depth histogram is built from remaining k-mer counts. The algorithm identifies the valley between the low-depth error peak and the true genome coverage peak, then finds the mode depth above that valley.
4. **Size estimate** — genome size is calculated as `total_kmers / peak_depth`, following the standard k-mer genome size formula.

This estimate is intended to be approximate and it is used downstream to inform assembly parameters.

## Input Format

Accepts standard FASTQ format. Gzip compression is detected automatically, so both `.fastq` and `.fastq.gz` files are accepted.

## Use in auto-autocycler-nf

`read_check` is used as a pre-assembly QC step in [auto-autocycler-nf](https://github.com/SamD28/auto-autocycler-nf), an automated long-read bacterial genome assembly pipeline built around [Autocycler](https://github.com/rrwick/Autocycler).

In that context, `read_check` is run on the raw ONT FASTQ input before assembly begins. The resulting YAML is parsed by the pipeline to:

- Sanity-check that the input contains sufficient reads of decent length before committing to a full assembly run.
- Use the estimated genome size (via `-g`) to guide subsampling depth targets passed to Autocycler, ensuring assemblies are performed at appropriate and consistent coverage levels.

## Installation

Requires Rust. Build from source:

```bash
cargo build --release
# binary at: target/release/read_check
```

Or via Docker:

```bash
docker build -t read_check .
docker run --rm -v $PWD:/data read_check /data/reads.fastq.gz -g -o /data/stats.yaml
```

## Dependencies

| Crate | Purpose |
|---|---|
| [clap](https://crates.io/crates/clap) | CLI argument parsing |
| [noodles](https://crates.io/crates/noodles) | FASTQ record parsing |
| [flate2](https://crates.io/crates/flate2) | Transparent gzip decompression |
| [rustc-hash](https://crates.io/crates/rustc-hash) | Fast non-cryptographic hashing for k-mer counting |
