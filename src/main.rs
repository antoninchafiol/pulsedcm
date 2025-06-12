use clap::{Parser, Subcommand};
use dicom::dictionary_std::tags;
use dicom::object::open_file;

#[derive(Parser)]
#[command(name = "app")]
#[command(about = "A CLI tool for handling DICOM files", long_about = None)]
struct Cli {
    /// Path to the dcm file
    #[arg(value_name = "PATH")]
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Work with tags
    Tags {
        #[arg(default_value = "all")]
        kind: String,
    },
    /// View a file
    View,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Tags { kind } => {
            println!("Handling 'tags' command with kind: {}", kind);
            if kind == "all" {
                println!("Listing all tags...");
            } else {
                println!("Handling tags for: {}", kind);
            }
            println!("Target path: {}", cli.path);
        }
        Commands::View => {
            println!("Handling 'view' command for file: {}", cli.path);
            // Do something with the file path
        }
    }
}
