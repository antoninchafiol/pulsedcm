use std::fs::File;
use std::{error, fs};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;

use serde::Serialize;
use csv::Writer;
use clap::{Parser, Args, Subcommand};
use image::{ImageEncoder, ImageBuffer};
use image::{GrayImage, Luma, DynamicImage};
use byteorder::{ByteOrder, LittleEndian};

//use dicom::object::open_file;
use dicom_object::open_file;
use dicom_dictionary_std::StandardDataDictionary;
use dicom_core::{DataDictionary, Tag};
use dicom_object::{FileDicomObject, InMemDicomObject};
use dicom_pixeldata::PixelDecoder;

#[derive(Parser)]
#[command(name = "app")]
#[command(about = "A CLI tool for handling DICOM files", long_about = None)]
struct Cli {
    /// Path to the dcm file
    #[arg(value_name = "PATH", default_value=".")]
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Work with tags
    Tags(TagsArgs),
    /// View a file
    View,
}

#[derive(Args, Debug)]
struct TagsArgs {
    #[arg(
        default_value = "all",
        value_parser = parse_tag_flags)
    ]
    kind: TagFlags,

    #[arg(long, value_name="FILE")]
    json: Option<PathBuf>,
    
    #[arg(long, value_name="FILE")]
    csv: Option<PathBuf>,
    
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TagFlags {
    /// All tags
    All,
    /// Important tags (Like PatientName, Modality, SeriesDescription)
    Short,
    /// Comma-separated list of specific DICOM tag keywords (e.g. PatientName,StudyDate)
    Specific(Vec<String>),
}

/// Modes for exporting DICOM Metadata
#[derive(Serialize)]
struct SerializableDicomEntry {
    filename: String,
    name: String,
    tag: String,
    vr: String,
    value: String,
}

fn parse_tag_flags(s: &str) -> Result<TagFlags, String> {
    match s.to_lowercase().as_str() {
        "all" => Ok(TagFlags::All),
        "short" => Ok(TagFlags::Short),
        other => {
            let keys = if other.is_empty() {
                Vec::new()
            } else {
                other
                    .split(',')
                    .map(str::trim)
                    .filter(|kw| !kw.is_empty())
                    .map(String::from)
                    .collect()
            };
            Ok(TagFlags::Specific(keys))
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Tags(args) => tags_handling(cli.path.as_str(), args),
        Commands::View => view_handling(cli.path.as_str()), 
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


fn tags_handling(path: &str, args: &TagsArgs){

    let files: Vec<String> = list_all_files(path);
    let mut export_data: Vec<SerializableDicomEntry> = Vec::new();

    let to_display = args.csv.is_some() || args.json.is_some();

    for f in files {
        let path: &str = f.as_str();
        if !to_display {
            println!("[{}]----", path);
        }

        let obj = match open_file(path) {
            Ok(o) => o,
            Err(_) => panic!("Can't open the file"),
        };
        export_data.extend(
            match &args.kind {
                TagFlags::All =>  all_tagging(path, &obj, to_display),
                TagFlags::Short =>  short_tagging(path, &obj, to_display),
                TagFlags::Specific(keys) =>  specific_tagging(path, &keys, &obj, to_display)
            }
        );
    }
    if let Some(json_path) = args.json.clone() {
        write_tag_files("json", json_path, &export_data);
    }
    
    if let Some(csv_path) = args.csv.clone() {
        write_tag_files("csv", csv_path, &export_data);
    }



}

fn write_tag_files(extension_type: &str, mut arg_clone: PathBuf, export_data: &[SerializableDicomEntry]){
    if arg_clone.extension().is_none() {
        arg_clone.set_extension(extension_type);
    }
    if arg_clone.exists() {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let stem = arg_clone.file_stem().unwrap_or_default().to_string_lossy();
        let parent = arg_clone.parent().unwrap_or_else(|| Path::new(""));
        let new_name = format!("{}_{}.{}", stem, timestamp, extension_type);
        arg_clone = parent.join(new_name);
    }
    let file = File::create(&arg_clone).unwrap();
    let mut wrt = Writer::from_writer(file);
    for entry in export_data {
        wrt.serialize(entry).unwrap();
    }

    match wrt.flush() {
        Ok(_o) => println!("\x1b[1;32mSuccessfully\x1b[0m saved CSV as \x1b[1m{:?} \x1b[0m", arg_clone),
        Err(_e) => println!("Error when writing in file")
    }

}

fn all_tagging(path: &str, obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>, to_display: bool) -> Vec<SerializableDicomEntry>{
    let mut output : Vec<SerializableDicomEntry> = Vec::new();
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
        output.push(
            SerializableDicomEntry { 
                filename: path.to_string(),
                name: name.to_string(),
                tag: format!("({:04X} {:04X})", tag.0, tag.1),
                vr: vr.to_string().to_string(),
                value: value.to_string() 
            }
        );
        if !to_display {
            print_colorize(tag, vr.to_string(),value.as_str(), name);
        }
    }
    return output;

}
fn short_tagging(path: &str, obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>, to_display: bool) -> Vec<SerializableDicomEntry>{
    let mut output: Vec<SerializableDicomEntry> = Vec::new();
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
            output.push(
                SerializableDicomEntry { 
                    filename: path.to_string(),
                    name: name.to_string(),
                    tag: format!("({:04X} {:04X})", tag.0, tag.1),
                    vr: vr.to_string().to_string(),
                    value: value.to_string() 
                }
            );
            if !to_display {
                print_colorize(*tag, vr.to_string(),value.as_str(), name);
            }
        }
    }
    return output;
}

fn specific_tagging(path: &str, input_kind: &[String], obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>, to_display: bool)-> Vec<SerializableDicomEntry>{
    let mut output: Vec<SerializableDicomEntry> = Vec::new();
    for element in obj.into_iter() {
        let tag: Tag = element.header().tag;
        let name = StandardDataDictionary
            .by_tag(tag)
            .map(|entry| entry.alias)
            .unwrap_or("Unknown");

        if input_kind.contains(&name.to_string().to_lowercase()){
            let vr = element.header().vr();
            let value: String = element.value()
                .to_str()
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| "[Binary]".to_string());
            output.push(
                SerializableDicomEntry { 
                    filename: path.to_string(),
                    name: name.to_string(),
                    tag: format!("({:04X} {:04X})", tag.0, tag.1),
                    vr: vr.to_string().to_string(),
                    value: value.to_string() 
                }
            );
            if !to_display {
                print_colorize(tag, vr.to_string(),value.as_str(), name);
            }
        }
    }
    return output;
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


fn is_phi_tag(tag: Tag) -> bool {
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

fn is_warning_tag(tag: Tag) -> bool {
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



fn view_handling(path: &str) {
    let files: Vec<String> = list_all_files(path);
    for file in files {
        let obj = match open_file(path) {
            Ok(o) => o,
            Err(_) => panic!("Can't open the file"),
        };
        view_processing(file.as_str(), &obj);
    }
}

struct DataInfo {
    bits_allocated: u16,
    bits_stored: u16,
    high_bits: u16,
    pixel_data: Vec<u8>,
    rows: u16,
    columns: u16,
}

fn view_processing(path: &str, obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>){
    convert_dicom_to_png(path);
    //if let Some(data_info) = view_get_data(obj){
    //    let png_path = PathBuf::from(path);
    //    png_path.with_extension("png");
    //
    //    //handling_image(png_path, &data_info);
    //} else {
    //    println!("ca nmarchae [as]");
    //
    //}

}


fn view_get_data(obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>) -> Option<DataInfo> {
    if let (
        Ok(bits_allocated),
        Ok(bits_stored),
        Ok(high_bits),
        Ok(pixel_data),
        Ok(rows),
        Ok(columns),
    ) = (
    obj.element(Tag(0x0028, 0x0100)),
    obj.element(Tag(0x0028, 0x0101)),
    obj.element(Tag(0x0028, 0x0102)),
    obj.element(Tag(0x7FE0, 0x0010)),
    obj.element(Tag(0x0028, 0x0010)),
    obj.element(Tag(0x0028, 0x0011)),
    ) {
        if let (
            Ok(bits_allocated),
            Ok(bits_stored),
            Ok(high_bits),
            Ok(pixel_data),
            Ok(rows),
            Ok(columns),
        ) = (
        bits_allocated.to_int::<u16>(),
        bits_stored.to_int::<u16>(),
        high_bits.to_int::<u16>(),
        pixel_data.to_bytes(),
        rows.to_int::<u16>(),
        columns.to_int::<u16>(),
        ) {
            let pixel_data_v = pixel_data.to_vec();
            return Some(DataInfo {
                bits_allocated,
                bits_stored,
                high_bits,
                pixel_data: pixel_data_v,
                rows,
                columns,
            });
        } else { None }
    } else { None }
}
fn convert_dicom_to_png(dicom_path: &str) -> Result<(), Box<dyn std::error::Error>>{
    let obj = open_file(dicom_path).unwrap();
    let image = obj.decode_pixel_data().unwrap();
    let dynamic_image = image.to_dynamic_image(0).unwrap();
    dynamic_image.save("out.png");
    println!("dgdfg");
    Ok(())
}

