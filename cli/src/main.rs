mod cmd_tags;

use clap::{Parser, Subcommand};
use cmd_tags::TagsArgs;



#[derive(Parser)]
#[command(name = "pulsedcm")]
#[command(about = "DICOM CLI Tool", long_about = None)]
struct Cli {
    /// Path to the dcm file/folder
    #[arg(value_name = "PATH", default_value=".")]
    path: String,

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
        Commands::Tags(args) => cmd_tags::run(cli.path.as_str(), args),
    }
}
