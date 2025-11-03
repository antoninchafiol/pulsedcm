use std::path::PathBuf;

use clap::Subcommand;
use pulsedcm_core::{jobs_handling, collect_dicom_files};

// ==== Commands ==== 
mod tags;
mod view;
mod ano;

#[derive(Subcommand)]
pub enum Commands {
    /// Work with DICOM tags
    Tags(tags::TagsArgs),
    View(view::ViewArgs),  
    Ano(ano::AnoArgs),
}

pub fn handling(path: &str, cmd: Commands, verbose: bool, jobs: Option<usize>) {
    let command = CliCommand::new(path, verbose, jobs);
    match cmd {
        Commands::Tags(args) => command.run(args),
        Commands::View(args) => command.run(args),
        Commands::Ano(args) => command.run(args),
    }
}


pub trait ArgRun {
    fn run_multiple(self, path: &str, files: Vec<PathBuf>, verbose: bool, jobs: usize);
    //fn run_single(&self, path: &str, file: PathBuf, verbose: bool, jobs: usize);
}

pub struct CliCommand<'a> {
    pub path: &'a str,
    pub files: Vec<PathBuf>,
    pub verbose: bool, 
    pub jobs: usize,

}
impl<'a> CliCommand<'a> {
    pub fn new(path: &'a str, verbose: bool, jobs: Option<usize>) -> Self {
        let files = match collect_dicom_files(path){
            Ok(o) => o, 
            Err(e) => panic!("Couldn't determine a file collection with the provided path: {}", e),
        };
        Self {
            path    : path,
            jobs    : jobs_handling(jobs, files.len()),
            files   : files,
            verbose : verbose,
        }
    }
    pub fn run(self, arg: impl ArgRun){
        arg.run_multiple(self.path, self.files,  self.verbose, self.jobs);
        //
        //match self.files.len() {
        //    1.. => arg.run_multiple(self.path, self.files,  self.verbose, self.jobs),
        //    1   => {
        //        let file = self.files[0].clone();
        //        arg.run_single(self.path, file,  self.verbose, self.jobs);
        //    },
        //    _ => eprintln!("Can't determine number of files"),
        //}
    }
}
