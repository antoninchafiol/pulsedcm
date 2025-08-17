use serde::Serialize;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub use rayon;
pub use rayon::prelude::*;

pub use dicom_core::{DataDictionary, PrimitiveValue};
pub use dicom_dictionary_std;
pub use dicom_dictionary_std::StandardDataDictionary;
pub use dicom_object::{open_file, FileDicomObject, InMemDicomObject, OpenFileOptions, Tag};
pub use dicom_pixeldata::PixelDecoder;

pub use std::fs::File;
pub use std::path::Path;
pub use std::time::{SystemTime, UNIX_EPOCH};

use walkdir::{self, DirEntry, WalkDir};

pub fn list_all_files(user_path: &str) -> Result<Vec<String>, Box<dyn Error>> {
    // TODO: Change to a Result and check if folder/file exists
    if !PathBuf::from(user_path).exists() {
        return Err("Provided file/folder doesn't exist".into());
    }

    let mut res: Vec<String> = Vec::new();

    for entry in WalkDir::new(user_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("dcm"))
                .unwrap_or(false)
        })
    {
        res.push(entry.path().to_str().unwrap().to_string());
    }
    Ok(res)
}

#[derive(Serialize)]
pub struct SerializableDicomEntry {
    pub filename: String,
    pub name: String,
    pub tag: String,
    pub vr: String,
    pub value: String,
}

pub fn jobs_handling(jobs: Option<usize>, max_file: usize) -> usize {
    let j = jobs.unwrap_or_else(|| {
        return 1;
    });
    if j <= 0 {
        // MAX NUMBER OF THREADS POSSIBLE BY MACHINEa
        if max_file > rayon::max_num_threads() {
            return rayon::max_num_threads();
        } else {
            return max_file;
        }
    } else {
        return j;
    }
}

pub fn ask_yes_no(question: &str) -> bool {
    print!("{} Y/N: ", question);
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

pub fn print_colorize(tag: Tag, vr: &str, value: &str, name: &str, out_string: &mut String) {
    let color = if is_phi_tag(tag) {
        "\x1b[1;91m" // Red
    } else if is_warning_tag(tag) {
        "\x1b[1;93m" // Yellow/Orange
    } else if value == "[Binary]" || name == "Unknown" {
        "\x1b[90m" // Grey
    } else {
        "\x1b[1m" // Bold default
    };
    let greyed = if value == "[Binary]" || name == "Unknown" {
        "\x1b[90m" // Grey
    } else {
        "\x1b[0m"
    };
    if out_string.is_empty() {
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
    } else {
        out_string.push_str(&format!(
            "{}({:04X},{:04X})\x1b[0m {}{:<2} {:<30} {}\x1b[0m\n",
            color,
            tag.group(),
            tag.element(),
            greyed,
            vr,
            name,
            value
        ));
    }
}

fn is_phi_tag(tag: Tag) -> bool {
    matches!(
        tag,
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
                              //
    )
}

fn is_warning_tag(tag: Tag) -> bool {
    matches!(
        tag,
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
