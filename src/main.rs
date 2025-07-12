use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::io::empty;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::Command;

use serde::Serialize;
use csv::Writer;
use clap::{Parser, Args, Subcommand};

use tempfile::TempDir;
use dicom_object::open_file;
use dicom_dictionary_std::StandardDataDictionary;
use dicom_core::{DataDictionary, DataElement, DicomValue, PrimitiveValue, Tag};
use dicom_object::{FileDicomObject, InMemDicomObject};
use dicom_pixeldata::PixelDecoder;
use dicom_object::AtAccessError;

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
    View(ViewArgs),
    Ano(AnoArgs),
}

#[derive(Args, Debug)]
struct TagsArgs {
    /// Type of args to provide between all,short and specific tagname
    #[arg(
        default_value = "all",
        value_parser = parse_tag_flags)
    ]
    kind: TagFlags,

    /// Export all output as a Serialized JSON
    #[arg(long, value_name="FILE")]
    json: Option<PathBuf>,
    
    /// Export all output as a Serialized CSV
    #[arg(long, value_name="FILE")]
    csv: Option<PathBuf>,
    
}

#[derive(Args, Debug)]
struct ViewArgs {

    /// Number of images to actually display via OS' PNG Viewer
    #[arg(long, value_name="NUMBER")]
    open: Option<u8>,
    
    /// Writes PNGs on temp directory instead of in the directory of DICOM files, invoke an open 1
    /// option
    #[arg(long)]
    temp: bool,
    
    /// Number of threads to launch to process
    #[arg(long, value_name="NUMBER")]
    jobs: Option<i8>,

} 

/// DICOM Supplement 142 Standard de-identification
/// Warning: No pixel modification for this version
#[derive(Args, Debug)]
struct AnoArgs {
    /// strategy of de-idenficication:
    /// replace: Replace with dummy value
    /// zero: Zero length
    /// remove: Remove completely 
    #[arg(
        long, 
        value_name="ACTION",
        value_parser=parse_actions
    )]
    action: Option<Actions>,
    
    /// De-identification profiles
    /// basic: Remove required PHI (safe)
    /// moderate: Also remove institution/device info
    /// strict: Maximum removal, leaves only technical data
    #[arg(
        long, 
        value_name="POLICY",
        value_parser=parse_policy
    )]
    policy: Option<Policy>,

    /// Number of threads to launch to process (0 = All possible threads)
    #[arg(long, value_name="NUMBER")]
    jobs: Option<u8>,

    /// Output directory to save anonymized files.
    /// If omitted, input files will be overwritten in-place.
    /// Must be a directory if specified.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Show the changed args for the file
    /// If multiple files it'll stop processing after the 1st to give an output
    #[arg(short, long, default_value= "false")]
    dry: bool,

    /// Show all changed values
    #[arg(short, long)]
    verbose: bool,

} 

/// Enum linked to the Actions part of the anonymization command
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Actions {
    Replace,
    Zero,
    Remove,
}

/// Enum linked to the policy part of the anonymization command
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Policy {
    Basic, 
    Moderate,
    Strict,
}

impl Policy {
    // https://dicom.nema.org/dicom/2013/output/chtml/part15/chapter_E.html
    //
    fn tags(&self) -> Vec<Tag>{
        match self { 
            Policy::Basic => vec![
                Tag(0x0010,0x0010),
                Tag(0x0010,0x0020),
                Tag(0x0010,0x0030),
                Tag(0x0010,0x0040),
                Tag(0x0010,0x1000),
                Tag(0x0010,0x1001),
                Tag(0x0010,0x1040),
                Tag(0x0010,0x2160),
                Tag(0x0010,0x4000),
                Tag(0x0008,0x0090),
                Tag(0x0008,0x0050),
                Tag(0x0008,0x0080),
                Tag(0x0008,0x0081),
                Tag(0x0008,0x1040),
                Tag(0x0008,0x1010),
                Tag(0x0038,0x0010), 
                Tag(0x0032,0x1032), 
                Tag(0x0032,0x1060), 
                Tag(0x0032,0x1064), 
                Tag(0x0040,0x1001), 
                Tag(0x0040,0x1003), 
                Tag(0x0040,0x1400), 
                Tag(0x0008,0x009C), 
                Tag(0x0010,0x1060), 
                Tag(0x0040,0x0243),
                Tag(0x0040,0x0242),
                Tag(0x0040,0x0254),
                Tag(0x0018,0x1000),
                Tag(0x0020,0x4000),
                Tag(0x4008,0x0114)
            ],
            Policy::Moderate => {
                let mut base = Policy::Basic.tags();
                base.extend([
                    Tag(0x0008,0x0020),
                    Tag(0x0008,0x0021),
                    Tag(0x0008,0x0022),
                    Tag(0x0008,0x0023),
                    Tag(0x0008,0x0030),
                    Tag(0x0008,0x0031),
                    Tag(0x0008,0x0032),
                    Tag(0x0008,0x0033),
                    Tag(0x0018,0x0015), 
                    Tag(0x0018,0x5100), 
                    Tag(0x0008,0x1070),
                    Tag(0x0018,0x1010),
                    Tag(0x0018,0x1050),
                    Tag(0x0018,0x1051)
                ]);
                base
            }
            Policy::Strict => {
                let mut base = Policy::Moderate.tags();
                base.extend([
                    Tag(0x0020,0x000D),
                    Tag(0x0020,0x000E),
                    Tag(0x0008,0x0018),
                    Tag(0x0018,0x1000),
                    Tag(0x0018,0x1002),
                    Tag(0x0008,0x0070),
                    Tag(0x0008,0x1011), 
                    Tag(0x0008,0x1070), 
                    Tag(0x0018,0x1004), 
                    Tag(0x0018,0x1020), 
                    Tag(0x0018,0x1030), 
                    Tag(0x0040,0xA730), 
                    Tag(0x0008,0x1090),
                    Tag(0x0008,0x0060),        
                    Tag(0x0010,0x1020),
                    Tag(0x0010,0x1030),
                    Tag(0x0010,0x21B0),
                    Tag(0x0040,0xA124),
                ]);
                base
            },
        }
    }
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

fn parse_actions(input: &str) -> Result<Actions, String>{
    match input.to_lowercase().as_str() {
        "replace" => Ok(Actions::Replace),
        "remove"  => Ok(Actions::Remove),
        "zero"    => Ok(Actions::Zero),
        other => {
            Err("should be either: 'replace', 'remove' or 'zero'".to_string())
        }
    }
}
fn parse_policy(input: &str) -> Result<Policy, String>{
    match input.to_lowercase().as_str() {
        "basic"    => Ok(Policy::Basic) ,
        "moderate" => Ok(Policy::Moderate) ,
        "strict"   => Ok(Policy::Strict) ,
        other => {
            Err("should be either: 'basic', 'moderate' or 'strict'".to_string())
        }
    }
}


fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Tags(args) => tags_handling(cli.path.as_str(), args),
        Commands::View(args) => view_handling(cli.path, args), 
        Commands::Ano(args) => ano_handling(cli.path, args), 
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


fn view_handling(path: String, args: &ViewArgs) {
    let mut open: u8 = args.open.unwrap_or(0);
    let _jobs: i8 = args.jobs.unwrap_or(1);
    let is_temp: bool = args.temp;

    let files: Vec<String> = list_all_files(path.as_str());
 

    if is_temp {
        match TempDir::new() {
            Ok(dir) => {
                let path = dir.path().to_path_buf();
                if open <= 0 {
                    open = 1;
                }

                for (idx, file) in files.iter().enumerate() {
                    println!("{}",file);

                    let mut input_path = PathBuf::from(file);
                    let mut output_path = path.join(input_path.file_name().unwrap_or_else(||{
                        println!("no filename in {}", input_path.display()); 
                        std::ffi::OsStr::new("unknown.png")
                    }) 
                    );
                    output_path.set_extension("png");

                    view_processing(&mut input_path, &output_path, idx < open as usize).unwrap_or_else(|_e|{
                        eprintln!("Can't process {} : {}", input_path.display(), _e);
                    });
                }
            }
            Err(e) => {
                eprintln!("Failed to create temporary directory: {}", e);
                return;
            }
        }
        println!("\x1b[1m>> \x1b[0mPress Enter to exit and delete temporary files...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }
    else {
        for (idx, file) in files.iter().enumerate() {
            println!("{}",file);

            let mut input_path = PathBuf::from(file);
            let mut output_path = input_path.clone();
            output_path.set_extension("png");

            view_processing(&mut input_path, &output_path, idx < open as usize).unwrap_or_else(|_e|{
                eprintln!("Can't process {} : {}", input_path.display(), _e);
            });
        }
    }
}

fn view_processing(input_path: &mut PathBuf, output_path: &PathBuf, is_to_open: bool) -> Result<(), Box<dyn Error>>{
    let dinput_path = input_path.to_str().ok_or("Can't open the path")?;
    let obj = open_file(dinput_path)?;
    let image = obj.decode_pixel_data()?;
    let dynamic_image = image.to_dynamic_image(0)?;
    dynamic_image.save(&output_path)?;
    if is_to_open {
        open_image(output_path.to_str().unwrap());       
    }

    Ok(())
    
}

fn open_image(path: &str){
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&["/C", "start", "",path])
            .spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Unsupported OS"))
    };

    if let Err(e) = result {
        eprintln!("Failed to open image: {}", e);
    }
}

fn ano_handling(path: String, args: &AnoArgs){
    let action :Actions = args.action.clone().unwrap_or(Actions::Zero);
    let dry    :bool    = args.dry;
    let jobs   :u8      = args.jobs.unwrap_or(1);
    let mut out    :PathBuf = args.out.clone().unwrap_or_else(|| {
        println!("out argument has issue when parsing"); 
        PathBuf::from(&path)
    });
    let policy :Policy  = args.policy.clone().unwrap_or(Policy::Basic);
    let verbose:bool    = args.verbose;
    
    let files: Vec<String> = list_all_files(path.as_str());

    for (idx, file) in files.iter().enumerate() {
        if dry && idx == 0 {
            if verbose {
                println!("Launching a dry run");
            }    
            let data = ano_file_process(PathBuf::from(file), &action, &policy, verbose);
            
            for element in data.into_iter() {
                let tag: Tag = element.header().tag;
                let vr = element.header().vr();
                let name = StandardDataDictionary
                    .by_tag(tag)
                    .map(|entry| entry.alias)
                    .unwrap_or_else(|| "Unknown");
                let value: String = element.value()
                    .to_str()
                    .map(|cow| cow.into_owned())
                    .unwrap_or_else(|_| "[Binary]".to_string());
                print_colorize(tag, vr.to_string(), value.as_str(), name);
            }
            return; 
        }
        // Case where out is not specified
        println!("{}", &out.display());
        if file.as_str() == out.as_os_str().to_str().unwrap() {
            if ask_yes_no("? No out specified confirm to overwrite actual files") {
                let filename = Path::new(file).file_name().unwrap();
                out.push(filename);
                let data = ano_file_process(PathBuf::from(file), &action, &policy, verbose);
                match data.write_to_file(&out) {
                    Ok(_o) => {
                        println!("Wrote succesfully to: {}", &out.display());
                    },
                    Err(e) => {
                        eprintln!("Error while writing to file: {}", e);
                    },
                };
            }
            else {
                println!("Stopping...");
                return;
            }
        }
        else {
            if !out.is_dir() {
                eprintln!("Output path shouldn't be a file");
                return;
            }
            let filename = Path::new(file).file_name().unwrap();
            out.push(filename);
            let data = ano_file_process(PathBuf::from(file), &action, &policy, verbose);
            match data.write_to_file(&out) {
                Ok(_o) => {
                    println!("Wrote succesfully to: {}", &out.display());
                },
                Err(e) => {
                    eprintln!("Error while writing to file: {}", e);
                },
            };
        }

    }
}

fn ano_file_process(path: PathBuf, action: &Actions, policy:&Policy,verbose: bool) -> FileDicomObject<InMemDicomObject> {
    let mut data = open_file(path).unwrap(); 
    let original_len = data.iter().count();
    let filter = policy.tags();
    for tag in filter {
        if let Err(_) = data.element(tag) {
            eprintln!("Warning: Tag {} can't be found and will be ignored", tag);
        }
        else {
            match action {
                Actions::Zero => {
                    data.update_value_at(tag, |value| {
                        *value.primitive_mut().unwrap() = PrimitiveValue::from("0");
                    }).unwrap_or_else(|err| eprintln!("Warning: couldn’t replace tag {:?}: {}", tag, err));
                },
                Actions::Remove => {
                    if !data.remove_element(tag) {
                        eprintln!("Warning: couldn’t remove tag {:?}", tag);
                    }
                }
                Actions::Replace => {
                    data.update_value_at(tag, |value| {
                        *value.primitive_mut().unwrap() = PrimitiveValue::from("Anonymized");
                    }).unwrap_or_else(|err| eprintln!("Warning: couldn’t replace tag {:?}: {}", tag, err));
                }
            };
        }
    }
    if verbose {
        println!("Original Length: {} -> New Length : {}", original_len, data.iter().count());
    }
    data
}
fn ask_yes_no(question: &str) -> bool {
    print!("{} Y/N: ", question);
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
