use clap::{Parser, Subcommand};
use cmd_tags::*;

#[derive(Parser)]
#[command(name = "pulsedcm")]
#[command(about = "DICOM CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Tags(TagsArgs),
}


fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tags(args) => pulsedcm_commands_tags::run::run(args),
    }
}
