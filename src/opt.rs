use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(version = clap::crate_version!(), author = clap::crate_authors!(", "))]
pub struct Opts {
    #[clap(short, long, default_value = "/usr/bin/java")]
    pub java: PathBuf,
    #[clap(short = 'g', long, default_value = "jplag.jar")]
    pub jplag: PathBuf,
    #[clap(short, long, default_value = "source")]
    pub source_dir: PathBuf,
    #[clap(short, long, default_value = "result")]
    pub result_dir: PathBuf,
    #[clap(short, long, default_value = "tmp")]
    pub temp_dir: PathBuf,
    #[clap(short, long, default_values = &[], multiple_occurrences = true)]
    pub problems: Vec<String>,
}
