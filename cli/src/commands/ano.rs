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

    /// Show the changed args for the file
    /// If multiple files it'll stop processing after the 1st to give an output
    #[arg(short, long, default_value= "false")]
    dry: bool,

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
            jobs,
            verbose,
        ){
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error when running ano command: {}", e);
            }
        };
    }
}
