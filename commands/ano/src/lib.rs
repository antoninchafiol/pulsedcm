use std::{path::PathBuf};

use pulsedcm_core::*;

/// Enum linked to the Actions part of the anonymization command
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Action {
    Replace,
    Zero,
    Remove,
}
impl Action {
    pub fn processing_data(&self, data: &mut FileDicomObject<InMemDicomObject>, tag: &Tag){
        match self {
                Self::Zero => {
                    data.update_value_at(*tag, |value| {
                        *value.primitive_mut().unwrap() = PrimitiveValue::from("0");
                    })
                    .unwrap_or_else(|err| {
                        eprintln!("Warning: couldn’t replace tag {:?}: {}", tag, err)
                    });
                }
                Self::Remove => {
                    if !data.remove_element(*tag) {
                        eprintln!("Warning: couldn’t remove tag {:?}", tag);
                    }
                }
                Self::Replace => {
                    data.update_value_at(*tag, |value| {
                        *value.primitive_mut().unwrap() = PrimitiveValue::from("Anonymized");
                    })
                    .unwrap_or_else(|err| {
                        eprintln!("Warning: couldn’t replace tag {:?}: {}", tag, err)
                    });
                }
        }
    }
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
    action: Action,
    policy: Policy,
    out: PathBuf,
    dry: &mut bool,
    verbose: bool,
    jobs: Option<usize>,
) -> Result<()> {
    let files = collect_dicom_files(path)?;
    let jobs: usize = jobs_handling(jobs, files.len());
    let _ = threading_handling(files, out, &action, &policy, verbose, jobs, dry)?;
    Ok(())
}

fn threading_handling(
    files: Vec<PathBuf>, 
    output_path: PathBuf,
    action: &Action,
    policy: &Policy,
    verbose: bool, 
    jobs: usize,
    dry: &mut bool, 
    ) -> Result<()> {
    
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()?;

    if let Some(first) = files.first(){
            single_thread_process(first.into(), &mut output_path.clone(),  &action, &policy, verbose , dry)?;
            *dry = false;
    }
    let _ = thread_pool.install(|| {
        let _ = files.par_iter().try_for_each(
            |file: &PathBuf| -> Result<()> {
                single_thread_process(file.into(), &mut output_path.clone(),  &action, &policy, verbose , dry)?;
                Ok(())
        });
    });

    Ok(())
}

fn single_thread_process(
    input_path: PathBuf,
    output_path: &mut PathBuf,
    action: &Action,
    policy: &Policy,
    verbose: bool,
    dry: &bool,
) -> Result<()> {
    if *dry {
        if verbose {
            println!("Launching a dry run");
        }
        let data = anonymize_file(input_path, &action, &policy, verbose)?;

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
        return Ok(());
    }

    let filename = input_path.file_name().unwrap_or_default();

    // Case where out is not specified
    if input_path.to_str().unwrap_or_default() == output_path.as_os_str().to_str().unwrap_or_default() {
        if ask_yes_no("? No output_path specified confirm to overwrite actual files") {
            output_path.push(filename);
            let data = anonymize_file(input_path,  &action, &policy, verbose)?;
            data.write_to_file(&output_path)?;
        } else {
            println!("Stopping...");
            return Ok(());
        }
    } else {
        if !output_path.is_dir() {
            eprintln!("Output path shouldn't be a file");
            return Ok(());
        }
        output_path.push(filename);

        let data = anonymize_file(input_path, &action, &policy, verbose)?;
        data.write_to_file(&output_path)?;
    }
    Ok(())
}

fn anonymize_file (
    file_path: PathBuf, 
    action: &Action, 
    policy: &Policy, 
    verbose: bool
) -> Result<FileDicomObject<InMemDicomObject>> {
    let mut data = open_file(file_path)?;
    let original_len = data.iter().count();
    for tag in policy.tags() {
        if data.element(tag).is_err() {
            if verbose {
                eprintln!("Warning: Tag {} can't be found and will be ignored", tag);
            }
        } else {
            action.processing_data(&mut data, &tag);
        };
    }
    if verbose {
        println!(
            "Removed/Changed Tags: {}",
            original_len - data.iter().count()
        );
    }
    Ok(data)
}




