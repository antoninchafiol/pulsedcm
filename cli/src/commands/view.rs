use clap::Args;
use std::path::PathBuf;

use pulsedcm_commands_view::{run as view_run};
use crate::commands::CliCommand;
#[derive(Args, Debug)]
pub struct ViewArgs {

    /// Number of images to actually display via OS' PNG Viewer
    #[arg(long, value_name="NUMBER")]
    open: Option<u8>,
    
    /// Writes PNGs on temp directory instead of in the directory of DICOM files, invoke an open 1
    /// option
    #[arg(long)]
    temp: bool,
    
    /// Output directory to save created png.
    /// If omitted, input files will be saved in same directory as input.
    /// Must be a directory if specified.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Number of threads to launch to process
    #[arg(long, value_name="NUMBER")]
    jobs: Option<usize>,

} 



impl CliCommand for ViewArgs {
    fn run(&self, path: &str) {
        let default_out_path = PathBuf::from(&path);
        match view_run(
            path, 
            self.open.unwrap_or(0), 
            self.temp, 
            self.out.as_ref().unwrap_or_else(|| {
                println!("out argument has issue when parsing"); 
                &default_out_path
            }),
            self.jobs,
        ){
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error when running ano command: {}", e);
            }
        };
    }
}


