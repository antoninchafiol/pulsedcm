use pulsedcm_core::*;
use std::path::PathBuf;
use std::sync::Mutex;
use rayon::prelude::*;

use pulsedcm_core::{open_file as open_dcm_file, Tag, StandardDataDictionary};
use csv::Writer;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TagFlags {
    /// All tags
    All,
    /// Important tags (Like PatientName, Modality, SeriesDescription)
    Short,
    /// Comma-separated list of specific DICOM tag keywords (e.g. PatientName,StudyDate)
    Specific(Vec<String>),
}

pub fn run(path: &str, kind: TagFlags, jobs: Option<usize>, json: Option<PathBuf>, csv: Option<PathBuf>){
    let files: Vec<String> = list_all_files(path);
    let jobs : usize = jobs_handling(jobs, files.len());
    
    let export_data = Mutex::new(Vec::<SerializableDicomEntry>::new());
    let to_display = csv.is_some() || json.is_some();



    let thread_pool = rayon::ThreadPoolBuilder::new().num_threads(jobs as usize).build().unwrap();
    thread_pool.install(|| {
        files.par_iter().for_each(|f| {
            let mut out_string = String::new();
            let path: &str = f.as_str();
            if !to_display {
                out_string.push_str(&format!("[{}]----\n", path));
            }

            let obj = match open_dcm_file(path) {
                Ok(o) => o,
                Err(e) => {
                    out_string.push_str(&format!("Can't open file: {}\n", e));
                    return;
                }, 
            };
            export_data.lock().unwrap().extend(
                match &kind {
                    TagFlags::All =>  all_tagging(path, &obj, to_display, &mut out_string),
                    TagFlags::Short =>  short_tagging(path, &obj, to_display, &mut out_string),
                    TagFlags::Specific(keys) =>  specific_tagging(path, &keys, &obj, to_display, &mut out_string)
            }
            );
            println!("{}", out_string);
        });
        if let Some(json_path) = json.clone() {
            write_tag_files("json", json_path, &export_data);
        }

        if let Some(csv_path) = csv.clone() {
            write_tag_files("csv", csv_path, &export_data);
        }
    });
}

fn write_tag_files(extension_type: &str, mut arg_clone: PathBuf, export_data: &Mutex<Vec<SerializableDicomEntry>>){
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
    for entry in export_data.lock().unwrap().iter() {
        wrt.serialize(entry).unwrap();
    }

    match wrt.flush() {
        Ok(_o) => println!("\x1b[1;32mSuccessfully\x1b[0m saved CSV as \x1b[1m{:?} \x1b[0m", arg_clone),
        Err(_e) => println!("Error when writing in file")
    }

}

fn all_tagging(path: &str, obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>, to_display: bool, out_string: &mut String) -> Vec<SerializableDicomEntry>{
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
            print_colorize(tag, vr.to_string(),value.as_str(), name, out_string);
        }
    }
    return output;

}

fn short_tagging(path: &str, obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>, to_display: bool, out_string: &mut String) -> Vec<SerializableDicomEntry>{
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
                print_colorize(*tag, vr.to_string(),value.as_str(), name, out_string);
            }
        }
    }
    return output;
}

fn specific_tagging(path: &str, input_kind: &[String], obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>, to_display: bool, out_string: &mut String)-> Vec<SerializableDicomEntry>{
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
                print_colorize(tag, vr.to_string(),value.as_str(), name, out_string);
            }
        }
    }
    return output;
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


fn print_colorize(tag: Tag, vr: &str, value: &str, name: &str, out_string: &mut String){
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
    if out_string.is_empty(){
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
    else {
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

