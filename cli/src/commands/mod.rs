use clap::Subcommand;

use crate::commands::{ano::AnoArgs, tags::TagsArgs, view::ViewArgs};

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

pub fn handling(path: &str, cmd: Commands) {
    match cmd {
        Commands::Tags(args) => tags::run(path, args),
        Commands::View(args) => view::run(path, args),
        Commands::Ano(args) => args.run(path),
    }
}


pub trait CliCommand {
    fn run(&self, path: &str);
}

