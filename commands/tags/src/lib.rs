use pulsedcm_core::*;
use std::sync::Mutex;
use std::{path::PathBuf};

use csv::Writer;
use pulsedcm_core::{
    collect_dicom_files,
    open_file as open_dcm_file, 
    StandardDataDictionary,
    Tag,
    Result,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TagFlags {
    /// All tags
    All,
    /// Important tags (Like PatientName, Modality, SeriesDescription)
    Short,
    /// Comma-separated list of specific DICOM tag keywords (e.g. PatientName,StudyDate)
    Specific(Vec<String>),
}

pub fn run(
    path: &str,
    kind: TagFlags,
    with_pixel_data: bool,
    jobs: Option<usize>,
    json: Option<PathBuf>,
    csv: Option<PathBuf>,
) -> Result<()> {
    // Setting up all components
    let files = collect_dicom_files(path)?;
    let jobs = jobs_handling(jobs, files.len());
    let to_display = csv.is_some() || json.is_some();

    let export_data = Mutex::new(Vec::<SerializableDicomEntry>::new());
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()?;

    let _ = thread_pool.install(|| -> Result<()> {
        let _ = files.par_iter().try_for_each(
            |f: &PathBuf| -> Result<()> {
                let mut out_string = String::new();
                let path = f.as_os_str().to_str().unwrap_or("No path");
                if !to_display {
                    out_string.push_str(&format!("[{}]----\n", path));
                }

                // Skipping pixel data when specified or exporting to JSON/CSV
                let obj = if !with_pixel_data || to_display {
                    OpenFileOptions::new()
                        .read_until(dicom_dictionary_std::tags::PIXEL_DATA)
                        .open_file(path)?
                } else {
                    open_dcm_file(path)?
                };

                export_data.lock()?.extend(
                    match &kind {
                        TagFlags::All => all_tagging(path, &obj, to_display, &mut out_string),
                        TagFlags::Short => short_tagging(path, &obj, to_display, &mut out_string),
                        TagFlags::Specific(keys) => {
                            specific_tagging(path, &keys, &obj, to_display, &mut out_string)
                        }
                }
                );
                println!("{}", out_string);
                Ok(())
            });
        if let Some(json_path) = json.clone() {
            let _ = write_tag_files("json", json_path, &export_data);
        }

        if let Some(csv_path) = csv.clone() {
            let _ = write_tag_files("csv", csv_path, &export_data);
        }
        Ok(())
    });
    Ok(())
}

fn write_tag_files(
    extension_type: &str,
    mut arg_clone: PathBuf,
    export_data: &Mutex<Vec<SerializableDicomEntry>>,
) -> Result<()> {
    if arg_clone.extension().is_none() {
        arg_clone.set_extension(extension_type);
    }
    if arg_clone.exists() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let stem = arg_clone.file_stem().unwrap_or_default().to_string_lossy();
        let parent = arg_clone.parent().unwrap_or_else(|| Path::new(""));
        let new_name = format!("{}_{}.{}", stem, timestamp, extension_type);
        arg_clone = parent.join(new_name);
    }
    let file = File::create(&arg_clone)?;
    let mut wrt = Writer::from_writer(file);
    for entry in export_data.lock()?.iter() {
        wrt.serialize(entry)?;
    }

    match wrt.flush() {
        Ok(_o) => {
            println!(
                "\x1b[1;32mSuccessfully\x1b[0m saved CSV as \x1b[1m{:?} \x1b[0m",
                arg_clone
            );
            return Ok(());
        },
        Err(e) => Err(e.into()),
    }
}

fn all_tagging(
    path: &str,
    obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>,
    to_display: bool,
    out_string: &mut String,
) -> Vec<SerializableDicomEntry> {
    let mut output: Vec<SerializableDicomEntry> = Vec::new();
    for element in obj.into_iter() {
        let tag: Tag = element.header().tag;
        let vr = element.header().vr();
        let name = StandardDataDictionary
            .by_tag(tag)
            .map(|entry| entry.alias)
            .unwrap_or("Unknown");
        let value: String = element
            .value()
            .to_str()
            .map(|cow| cow.into_owned())
            .unwrap_or_else(|_| "[Binary]".to_string());
        output.push(SerializableDicomEntry {
            filename: path.to_string(),
            name: name.to_string(),
            tag: format!("({:04X} {:04X})", tag.0, tag.1),
            vr: vr.to_string().to_string(),
            value: value.to_string(),
        });
        if !to_display {
            print_colorize(tag, vr.to_string(), value.as_str(), name, out_string);
        }
    }
    return output;
}

fn short_tagging(
    path: &str,
    obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>,
    to_display: bool,
    out_string: &mut String,
) -> Vec<SerializableDicomEntry> {
    let mut output: Vec<SerializableDicomEntry> = Vec::new();
    let short_tags = [
        Tag(0x0010, 0x0010), // PatientName
        Tag(0x0008, 0x0020), // StudyDate
        Tag(0x0008, 0x0060), // Modality
        Tag(0x0008, 0x103E), // SeriesDescription
    ];

    for tag in &short_tags {
        if let Ok(element) = obj.element(*tag) {
            let vr = element.header().vr();
            let name = StandardDataDictionary
                .by_tag(*tag)
                .map(|entry| entry.alias)
                .unwrap_or_else(|| "Unknown");
            let value: String = element
                .value()
                .to_str()
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| "[Binary]".to_string());
            output.push(SerializableDicomEntry {
                filename: path.to_string(),
                name: name.to_string(),
                tag: format!("({:04X} {:04X})", tag.0, tag.1),
                vr: vr.to_string().to_string(),
                value: value.to_string(),
            });
            if !to_display {
                print_colorize(*tag, vr.to_string(), value.as_str(), name, out_string);
            }
        }
    }
    return output;
}

fn specific_tagging(
    path: &str,
    input_kind: &[String],
    obj: &FileDicomObject<InMemDicomObject<StandardDataDictionary>>,
    to_display: bool,
    out_string: &mut String,
) -> Vec<SerializableDicomEntry> {
    let mut output: Vec<SerializableDicomEntry> = Vec::new();
    for element in obj.into_iter() {
        let tag: Tag = element.header().tag;
        let name = StandardDataDictionary
            .by_tag(tag)
            .map(|entry| entry.alias)
            .unwrap_or("Unknown");

        if input_kind.contains(&name.to_string().to_lowercase()) {
            let vr = element.header().vr();
            let value: String = element
                .value()
                .to_str()
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| "[Binary]".to_string());
            output.push(SerializableDicomEntry {
                filename: path.to_string(),
                name: name.to_string(),
                tag: format!("({:04X} {:04X})", tag.0, tag.1),
                vr: vr.to_string().to_string(),
                value: value.to_string(),
            });
            if !to_display {
                print_colorize(tag, vr.to_string(), value.as_str(), name, out_string);
            }
        }
    }
    return output;
}
