use clap::Args;
use std::path::PathBuf;

use pulsedcm_commands_tags::{run as tag_run, TagFlags};

#[derive(Args, Debug)]
pub struct TagsArgs {
    /// Type of args to provide between all,short and specific tagname
    #[arg(
        default_value = "all",
        value_parser = parse_tag_flags)
    ]
    kind: TagFlags,

    /// Activate pixel data reading or not
    #[arg(long)]
    with_pixel_data: bool,

    /// Number of threads to launch to process
    #[arg(long, value_name = "NUMBER")]
    jobs: Option<usize>,

    /// Export all output as a Serialized JSON
    #[arg(long, value_name = "FILE")]
    json: Option<PathBuf>,

    /// Export all output as a Serialized CSV
    #[arg(long, value_name = "FILE")]
    csv: Option<PathBuf>,
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

pub fn run(path: &str, args: TagsArgs) {
    match tag_run(
        path,
        args.kind.clone(),
        args.with_pixel_data,
        args.jobs.clone(),
        args.json.clone(),
        args.csv.clone(),
    ) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error when running tags command: {}", e);
        }
    };
}
