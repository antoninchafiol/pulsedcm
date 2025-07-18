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
}

//#[derive(Subcommand)]
//enum Commands {
//    Tags(TagsArgs),
//}


fn main() {
    let cli = Cli::parse();
    commands::handling(&cli.path, cli.command);
}
