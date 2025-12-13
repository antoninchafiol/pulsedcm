use clap::Args;
use std::path::PathBuf;

use pulsedcm_commands_ano::{threading_handling};

use crate::commands::{ArgRun};

#[derive(Args, Debug)]
pub struct AnoArgs {
    /// If omitted, input files will be overwritten in-place.
    /// Must be a directory if specified.
    #[arg(long)]
    out: Option<PathBuf>,

    #[arg(short, long, default_value_t = 1)]
    workers: usize,

    #[arg(short, long, default_value_t = 1)]
    batch: usize,
    /// Show the changed args for the file
    /// If multiple files it'll stop processing after the 1st to give an output
    #[arg(short, long, default_value= "false")]
    dry: bool,

    /// Read now include PixelData and tags past it 
    /// Otherwise it'd stop before PixelData
    #[arg(long)]
    with_pixel_data: bool,

    /// Determine if we hash the UID or preserve UID as requested in Chapter E
    #[arg(short, long)]
    uid_hash: bool, 
} 

impl ArgRun for AnoArgs {
    fn run_multiple(self, path: &str, files: Vec<PathBuf>,  verbose:bool, jobs:usize) {
        let mut dry_arg = self.dry;
        let default_out_path = PathBuf::from(&path);
        match threading_handling(
            files, 
            self.out.unwrap_or_else(|| {
                if verbose {
                    println!("out argument has issue when parsing"); 
                }
                default_out_path
            }),
            &mut dry_arg, 
            self.with_pixel_data,
            jobs,
            self.batch,
            verbose,
            &self.uid_hash,
        ){
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error when running ano command: {}", e);
            }
        };
    }
}
