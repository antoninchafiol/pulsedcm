use clap::Args;
use std::path::PathBuf;

use pulsedcm_commands_view::{run as view_run};

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

pub fn run(path: &str, args: ViewArgs){
   match view_run(
        path, 
        args.open.unwrap_or(0), 
        args.temp, 
        args.out.unwrap_or_else(|| {
            println!("Out argument not provided, taking the path of input"); 
            PathBuf::from(&path)
        }),
        args.jobs.clone()
    ){
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error when running view command: {}", e);
        }
   };
}




