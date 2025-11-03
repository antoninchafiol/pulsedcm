mod commands;

use clap::{Parser};

#[derive(Parser)]
#[command(name = "pulsedcm")]
#[command(about = "DICOM CLI Tool", long_about = None)]
struct Cli {
    /// Path to the dcm file/folder
    #[arg(value_name = "PATH", default_value=".")]
    path: String,

    #[command(subcommand)]
    command: commands::Commands,

    /// Number of threads to launch to process
    #[arg(long, value_name = "NUMBER")]
    jobs: Option<usize>,

    /// Show all changed values
    #[arg(short, long)]
    verbose: bool,
}


fn main() {
    let cli = Cli::parse();
    commands::handling(&cli.path, cli.command, cli.verbose, cli.jobs);
}
