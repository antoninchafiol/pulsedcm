use clap::Subcommand;

// ==== Commands ==== 
mod tags;
mod view;

#[derive(Subcommand)]
pub enum Commands {
    /// Work with DICOM tags
    Tags(tags::TagsArgs),
    View(view::ViewArgs),  
}

pub fn handling(path: &str, cmd: Commands) {
    match cmd {
        Commands::Tags(args) => tags::run(path, args),
        Commands::View(args) => view::run(path, args),
    }
}
