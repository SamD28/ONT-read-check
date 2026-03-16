use clap::Parser;
use noodles::fastq;
use std::fs;

mod stats;
mod utils;

#[derive(Parser, Debug)]
#[clap(author, version, about="", long_about = None)]
struct Cli {
    input: String,

    #[arg(short, long)]
    genome_size: bool,

    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let mut stats = stats::FastqStats::new();

    let mut reader = utils::read_file(args.input).map(fastq::io::Reader::new)?;

    reader.records().into_iter().for_each(|record| {
        let record = record.expect("ERROR: problem parsing fastq record");

        stats.add_read(record.sequence(), args.genome_size);
    });

    stats.calculate_n50();

    if args.genome_size {
        stats.calculate_genome_size();
    }

    let output_path = args.output.unwrap_or_else(|| "read_stats.yaml".to_string());

    fs::write(&output_path, stats.to_yaml())
        .expect("Should be able to write to the specified output file");

    Ok(())
}
