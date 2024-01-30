use std::{fs, io::Cursor, ops::Add as _, path::PathBuf};

use clap::Parser;
use counter::Counter;
use nombrilo::{anvil::parse_region, distribution, Chunk};
use rayon::iter::{ParallelBridge, ParallelIterator};
use tabled::{settings::Style, Table, Tabled};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Region files or directories containing region files. Default is the
    /// current directory.
    region: Option<Vec<PathBuf>>,

    /// Display the top N most common blocks. Default is 10.
    #[arg(short)]
    n: Option<usize>,

    /// Blocks to ignore, with or without `minecraft:`. Default is none.
    #[arg(short, long)]
    ignore: Option<Vec<String>>,

    /// Sort the output by count. Default is false.
    #[arg(short, long)]
    sorted: bool,

    /// Print additional information, including time taken. Default is false.
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
struct BlockDistribution {
    block: String,
    count: u64,
}

impl From<(String, u64)> for BlockDistribution {
    fn from((block, count): (String, u64)) -> Self {
        Self { block, count }
    }
}

fn flatten_path(region: PathBuf) -> Vec<PathBuf> {
    let region_type = fs::metadata(&region).unwrap().file_type();

    if region_type.is_file() {
        return vec![region];
    }

    fs::read_dir(region)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().unwrap().is_file()
                && entry.file_name().to_string_lossy().ends_with(".mca")
                && entry.metadata().unwrap().len() > 0
        })
        .map(|entry| entry.path())
        .collect()
}

fn parse_file(region: PathBuf) -> Vec<Chunk> {
    let file = fs::read(region).unwrap();
    let mut reader = Cursor::new(file);
    parse_region(&mut reader).unwrap()
}

fn main() {
    let start = std::time::Instant::now();

    let cli = Cli::parse();
    let regions = cli.region.unwrap_or(vec![".".into()]);

    let mut block_distribution = regions
        .into_iter()
        .flat_map(flatten_path)
        .par_bridge()
        .map(parse_file)
        .flatten()
        .map(distribution::chunk)
        .reduce(Counter::<String, u64>::new, Counter::add);

    if let Some(ignore) = cli.ignore {
        for block in ignore {
            block_distribution.remove(&block);
            block_distribution.remove(&format!("minecraft:{}", block));
        }
    }

    let top_n = cli.n.unwrap_or(10);
    let displayed: Vec<BlockDistribution> = if cli.sorted {
        block_distribution
            .k_most_common_ordered(top_n)
            .into_iter()
            .map(BlockDistribution::from)
            .collect()
    } else {
        block_distribution
            .into_iter()
            .take(top_n)
            .map(BlockDistribution::from)
            .collect()
    };

    let mut table = Table::new(displayed);
    table.with(Style::modern());

    println!("{}", table);
    if cli.verbose {
        println!("Done in {:?}.", start.elapsed());
    }
}
