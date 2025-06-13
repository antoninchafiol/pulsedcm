use clap::{Parser, Subcommand};
use dicom::dictionary_std::tags;
use dicom::object::open_file;
use dicom::dump::dump_file;
use dicom::dictionary_std::StandardDataDictionary;
use dicom::core::{DataDictionary, Tag};

use colored::Colorize;
use dicom::object::{FileDicomObject, InMemDicomObject};

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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum TagFlags {
    All,
    Short,
    /// Comma-separated list of specific DICOM tag keywords (e.g. PatientName,StudyDate)
    Specific,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Tags { kind } => {
            println!("Target path: {}", cli.path);
            println!("Handling 'tags' command with kind: {}", kind);
            tags(&cli.path, kind);
        }
        Commands::View => {
            println!("Handling 'view' command for file: {}", cli.path);
            // Do something with the file path
        }
    }
}

fn tags(path: &str, kind: &str) -> Result<(), Box<dyn std::error::Error>>{
    let obj = open_file(path)?;


    let all: bool = true;
    match kind {
        "all" => all = true;
        "short" => ;
        _ =>
    }
    for element in obj.into_iter() {
        let tag: Tag = element.header().tag;
        let vr = element.header().vr();
        // Look up a human‐readable name if you like
        let n = StandardDataDictionary;
        let name = n
            .by_tag(tag)
            .map(|entry| entry.alias)
            .unwrap_or("Unknown");
        let value: String = element.value()
            .to_str()
            .map(|cow| cow.into_owned())
            .unwrap_or_else(|_| "[Binary]".to_string());

        println!(
            "\x1b[93m({:04X},{:04X})\x1b[0m {:<2} {:<30} {}",
            tag.group(),
            tag.element(),
            vr,
            name,
            value
        );
    }

    Ok(())
}


fn short_tagging(obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>){
    let short_tags = [
        Tag(0x0010, 0x0010), // PatientName
        Tag(0x0008, 0x0020), // StudyDate
        Tag(0x0008, 0x0060), // Modality
        Tag(0x0008, 0x103E), // SeriesDescription
    ];

    for tag in &short_tags{
        if let Ok(element) = obj.element(*tag){
            let vr = element.header().vr();
            let n = StandardDataDictionary;
            let name = n
                .by_tag(*tag)
                .map(|entry| entry.alias)
                .unwrap_or("Unknown");
            let value: String = element.value()
                .to_str()
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| "[Binary]".to_string());

            println!(
                "\x1b[93m({:04X},{:04X})\x1b[0m {:<2} {:<30} {}",
                tag.group(),
                tag.element(),
                vr,
                name,
                value
            );
        }
    }
}
