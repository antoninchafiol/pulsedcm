use clap::Args;

#[derive(Args, Debug)]
struct TagsArgs {
    /// Type of args to provide between all,short and specific tagname
    #[arg(
        default_value = "all",
        value_parser = parse_tag_flags)
    ]
    kind: TagFlags,

    /// Number of threads to launch to process
    #[arg(long, value_name="NUMBER")]
    jobs: Option<u8>,

    /// Export all output as a Serialized JSON
    #[arg(long, value_name="FILE")]
    json: Option<PathBuf>,
    
    /// Export all output as a Serialized CSV
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
