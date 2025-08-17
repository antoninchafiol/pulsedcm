use std::path::PathBuf;

use pulsedcm_core::*;

/// Enum linked to the Actions part of the anonymization command
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Actions {
    Replace,
    Zero,
    Remove,
}

/// Enum linked to the policy part of the anonymization command
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Policy {
    Basic,
    Moderate,
    Strict,
}

impl Policy {
    // https://dicom.nema.org/dicom/2013/output/chtml/part15/chapter_E.html
    fn tags(&self) -> Vec<Tag> {
        match self {
            Policy::Basic => vec![
                Tag(0x0010, 0x0010),
                Tag(0x0010, 0x0020),
                Tag(0x0010, 0x0030),
                Tag(0x0010, 0x0040),
                Tag(0x0010, 0x1000),
                Tag(0x0010, 0x1001),
                Tag(0x0010, 0x1040),
                Tag(0x0010, 0x2160),
                Tag(0x0010, 0x4000),
                Tag(0x0008, 0x0090),
                Tag(0x0008, 0x0050),
                Tag(0x0008, 0x0080),
                Tag(0x0008, 0x0081),
                Tag(0x0008, 0x1040),
                Tag(0x0008, 0x1010),
                Tag(0x0038, 0x0010),
                Tag(0x0032, 0x1032),
                Tag(0x0032, 0x1060),
                Tag(0x0032, 0x1064),
                Tag(0x0040, 0x1001),
                Tag(0x0040, 0x1003),
                Tag(0x0040, 0x1400),
                Tag(0x0008, 0x009C),
                Tag(0x0010, 0x1060),
                Tag(0x0040, 0x0243),
                Tag(0x0040, 0x0242),
                Tag(0x0040, 0x0254),
                Tag(0x0018, 0x1000),
                Tag(0x0020, 0x4000),
                Tag(0x4008, 0x0114),
            ],
            Policy::Moderate => {
                let mut base = Policy::Basic.tags();
                base.extend([
                    Tag(0x0008, 0x0020),
                    Tag(0x0008, 0x0021),
                    Tag(0x0008, 0x0022),
                    Tag(0x0008, 0x0023),
                    Tag(0x0008, 0x0030),
                    Tag(0x0008, 0x0031),
                    Tag(0x0008, 0x0032),
                    Tag(0x0008, 0x0033),
                    Tag(0x0018, 0x0015),
                    Tag(0x0018, 0x5100),
                    Tag(0x0008, 0x1070),
                    Tag(0x0018, 0x1010),
                    Tag(0x0018, 0x1050),
                    Tag(0x0018, 0x1051),
                ]);
                base
            }
            Policy::Strict => {
                let mut base = Policy::Moderate.tags();
                base.extend([
                    Tag(0x0020, 0x000D),
                    Tag(0x0020, 0x000E),
                    Tag(0x0008, 0x0018),
                    Tag(0x0018, 0x1000),
                    Tag(0x0018, 0x1002),
                    Tag(0x0008, 0x0070),
                    Tag(0x0008, 0x1011),
                    Tag(0x0008, 0x1070),
                    Tag(0x0018, 0x1004),
                    Tag(0x0018, 0x1020),
                    Tag(0x0018, 0x1030),
                    Tag(0x0040, 0xA730),
                    Tag(0x0008, 0x1090),
                    Tag(0x0008, 0x0060),
                    Tag(0x0010, 0x1020),
                    Tag(0x0010, 0x1030),
                    Tag(0x0010, 0x21B0),
                    Tag(0x0040, 0xA124),
                ]);
                base
            }
        }
    }
}

pub fn run(
    path: &str,
    action: Actions,
    policy: Policy,
    out: PathBuf,
    dry: bool,
    verbose: bool,
    jobs: Option<usize>,
) {
    let files = match list_all_files(path) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    let jobs: usize = jobs_handling(jobs, files.len());
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs as usize)
        .build()
        .unwrap();
    thread_pool.install(|| {
        files.par_iter().enumerate().for_each(|(idx, file)| {
            let mut out_clone = out.clone();
            ano_threaded_function(idx, file, &mut out_clone, dry, &action, &policy, verbose);
        });
    });
}

fn ano_threaded_function(
    idx: usize,
    file: &String,
    out: &mut PathBuf,
    dry: bool,
    action: &Actions,
    policy: &Policy,
    verbose: bool,
) {
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
            let value: String = element
                .value()
                .to_str()
                .map(|cow| cow.into_owned())
                .unwrap_or_else(|_| "[Binary]".to_string());
            let mut s = String::new();
            print_colorize(tag, vr.to_string(), value.as_str(), name, &mut s);
        }
        return;
    }
    // Case where out is not specified
    if file.as_str() == out.as_os_str().to_str().unwrap() {
        if ask_yes_no("? No out specified confirm to overwrite actual files") {
            let filename = Path::new(file).file_name().unwrap();
            out.to_owned().push(filename);
            let data = ano_file_process(PathBuf::from(file), &action, &policy, verbose);
            match data.write_to_file(&out) {
                Ok(_o) => {
                    println!("Wrote succesfully to: {}", &out.display());
                }
                Err(e) => {
                    eprintln!("Error while writing to file: {}", e);
                }
            };
        } else {
            println!("Stopping...");
            return;
        }
    } else {
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
            }
            Err(e) => {
                eprintln!("Error while writing to file: {}", e);
            }
        };
    }
}

fn ano_file_process(
    path: PathBuf,
    action: &Actions,
    policy: &Policy,
    verbose: bool,
) -> FileDicomObject<InMemDicomObject> {
    let mut data = open_file(path).unwrap();
    let original_len = data.iter().count();
    let filter = policy.tags();
    for tag in filter {
        if let Err(_) = data.element(tag) {
            eprintln!("Warning: Tag {} can't be found and will be ignored", tag);
        } else {
            match action {
                Actions::Zero => {
                    data.update_value_at(tag, |value| {
                        *value.primitive_mut().unwrap() = PrimitiveValue::from("0");
                    })
                    .unwrap_or_else(|err| {
                        eprintln!("Warning: couldn’t replace tag {:?}: {}", tag, err)
                    });
                }
                Actions::Remove => {
                    if !data.remove_element(tag) {
                        eprintln!("Warning: couldn’t remove tag {:?}", tag);
                    }
                }
                Actions::Replace => {
                    data.update_value_at(tag, |value| {
                        *value.primitive_mut().unwrap() = PrimitiveValue::from("Anonymized");
                    })
                    .unwrap_or_else(|err| {
                        eprintln!("Warning: couldn’t replace tag {:?}: {}", tag, err)
                    });
                }
            };
        }
    }
    if verbose {
        println!(
            "Original Length: {} -> New Length : {}",
            original_len,
            data.iter().count()
        );
    }
    data
}
