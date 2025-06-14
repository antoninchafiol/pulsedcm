use std::{fs, path};
use std::path::Path;
use clap::{Parser, Subcommand};
use dicom::object::open_file;
use dicom::dictionary_std::StandardDataDictionary;
use dicom::core::{DataDictionary, Tag};

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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
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
            let files: Vec<String> = list_all_files(cli.path.as_str());
            for f in files {
                let path: &str = f.as_str();
                println!("[{}]----", path);
                let _ = tags(path, kind);
            }
        }
        Commands::View => {
            let files: Vec<String> = list_all_files(cli.path.as_str());
        }
    }
}

fn list_all_files(user_path: &str) -> Vec<String>{
    let mut res: Vec<String> = Vec::new();
    let pa = Path::new(user_path);
    if pa.is_file(){
        if pa.extension().unwrap() == "dcm" {
            res.push(user_path.to_string());
        } 
    } else {
        for p in fs::read_dir(user_path).unwrap(){
            if let Ok(p) = p {
                if let Some(p) = p.path().to_str(){
                    res.extend(list_all_files(p));
                }
            }
        }
    }
    res
}

fn tags(path: &str, kind: &str) -> Result<(), Box<dyn std::error::Error>>{
    if Path::new(path).is_file() {

    }
    let obj = open_file(path)?;
    match kind {
        "all" => {
            for element in obj.into_iter() {
                let tag: Tag = element.header().tag;
                let vr = element.header().vr();
                let name = StandardDataDictionary
                    .by_tag(tag)
                    .map(|entry| entry.alias)
                    .unwrap_or("Unknown");
                let value: String = element.value()
                    .to_str()
                    .map(|cow| cow.into_owned())
                    .unwrap_or_else(|_| "[Binary]".to_string());
            print_colorize(tag, vr.to_string(), value.as_str(), name);
            }
            return Ok(())
        },
        "short" => {
            short_tagging(&obj);
            return Ok(())
        }
        other => {
            specific_tagging(other, &obj);
            return Ok(())
        }
    };
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
            let name = StandardDataDictionary
                .by_tag(*tag)
                .map(|entry| entry.alias)
                .unwrap_or_else(|| "Unknown");
            let value: String = element.value()
                .to_str()
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| "[Binary]".to_string());
            print_colorize(*tag, vr.to_string(), value.as_str(), name);
        }
    }
}
fn specific_tagging(input_kind: &str, obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>){
    let output: Vec<&str> = input_kind.split(",").collect();
    for element in obj.into_iter() {
        let tag: Tag = element.header().tag;
        let name = StandardDataDictionary
            .by_tag(tag)
            .map(|entry| entry.alias)
            .unwrap_or("Unknown");

        if output.contains(&name){
            let vr = element.header().vr();
            let value: String = element.value()
                .to_str()
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| "[Binary]".to_string());
            print_colorize(tag, vr.to_string(), value.as_str(), name);
        }
    }

}


fn print_colorize(tag: Tag, vr: &str, value: &str, name: &str){
    let color = if is_phi_tag(tag) {
        "\x1b[1;91m" // Red
    } else if is_warning_tag(tag) {
        "\x1b[1;93m" // Yellow/Orange
    } else if value == "[Binary]" || name == "Unknown" {
        "\x1b[90m" // Grey
    } else {
        "\x1b[1m"  // Bold default
    };
    let greyed =  if value == "[Binary]" || name == "Unknown" {
        "\x1b[90m" // Grey
    } else {
        "\x1b[0m"
    };
    println!(
        "{}({:04X},{:04X})\x1b[0m {}{:<2} {:<30} {}\x1b[0m",
        color,
        tag.group(),
        tag.element(),
        greyed,
        vr,
        name,
        value
    );


}
pub fn is_phi_tag(tag: Tag) -> bool {
    matches!(tag,
        Tag(0x0010, 0x0010) // Patient's Name
        | Tag(0x0010, 0x0020) // Patient ID
        | Tag(0x0010, 0x0030) // Patient's Birth Date
        | Tag(0x0010, 0x0032) // Patient's Birth Time
        | Tag(0x0010, 0x0040) // Patient's Sex
        | Tag(0x0010, 0x1000) // Other Patient IDs
        | Tag(0x0010, 0x1001) // Other Patient Names
        | Tag(0x0010, 0x1005) // Patient's Birth Name
        | Tag(0x0010, 0x1060) // Patient’s Mother's Birth Name
        | Tag(0x0010, 0x2154) // Patient's Telephone Numbers
        | Tag(0x0010, 0x2180) // Occupation
        | Tag(0x0010, 0x1040) // Patient's Address
        | Tag(0x0038, 0x0300) // Current Patient Location
        | Tag(0x0038, 0x0400) // Patient's Institution Residence
    )
}
pub fn is_warning_tag(tag: Tag) -> bool {
    matches!(tag,
        Tag(0x0008, 0x0050) // Accession Number
        | Tag(0x0008, 0x0080) // Institution Name
        | Tag(0x0008, 0x0081) // Institution Address
        | Tag(0x0008, 0x0090) // Referring Physician’s Name
        | Tag(0x0008, 0x0092) // Referring Physician’s Address
        | Tag(0x0008, 0x0094) // Referring Physician’s Telephone Numbers
        | Tag(0x0008, 0x1010) // Station Name
        | Tag(0x0008, 0x1040) // Institutional Department Name
        | Tag(0x0008, 0x1050) // Performing Physician’s Name
        | Tag(0x0008, 0x1070) // Operator’s Name
        | Tag(0x0008, 0x1030) // Study Description
        | Tag(0x0008, 0x103E) // Series Description
        | Tag(0x0018, 0x1000) // Device Serial Number
        | Tag(0x0018, 0x1030) // Protocol Name
    )
}
