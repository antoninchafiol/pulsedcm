use clap::Args;

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
    
    /// Number of threads to launch to process
    #[arg(long, value_name="NUMBER")]
    jobs: Option<usize>,

} 

pub fn run(path: &str, args: ViewArgs){
    view_run(
        path, 
        args.open.unwrap_or(0), 
        args.temp, 
        args.jobs.clone()
    );
}




