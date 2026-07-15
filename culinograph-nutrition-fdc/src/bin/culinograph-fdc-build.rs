use anyhow::{Context, Result};
use clap::Parser;
use culinograph_nutrition_fdc::{BuildOptions, DEFAULT_FULL_DOWNLOAD_URL, FdcDatabaseBuilder};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    about = "Build a searchable SQLite nutrition database from the USDA FDC full CSV download"
)]
struct Args {
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long)]
    download: bool,
    #[arg(long, default_value = DEFAULT_FULL_DOWNLOAD_URL)]
    url: String,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value = "2026-04")]
    release: String,
    #[arg(long)]
    replace: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let downloaded;
    let source = if args.download {
        downloaded = tempfile::NamedTempFile::new()?;
        let mut response = reqwest::blocking::get(&args.url)?.error_for_status()?;
        let mut target = downloaded.reopen()?;
        std::io::copy(&mut response, &mut target).context("download USDA archive")?;
        downloaded.path().to_owned()
    } else {
        args.source.context("provide --source or --download")?
    };
    let report = FdcDatabaseBuilder::build(&BuildOptions {
        source,
        destination: args.output,
        release: args.release,
        replace: args.replace,
    })?;
    println!(
        "foods={} nutrients={} food_nutrients={}",
        report.foods, report.nutrients, report.food_nutrients
    );
    Ok(())
}

#[cfg(test)]
#[path = "culinograph_fdc_build_test.rs"]
mod test;
