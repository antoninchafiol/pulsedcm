use clap::Subcommand;

// ==== Commands ==== 
mod tags;

#[derive(Subcommand)]
pub enum Commands {
    /// Work with DICOM tags
    Tags(tags::TagsArgs),
}

pub fn handling(path: &str, cmd: Commands) {
    match cmd {
        Commands::Tags(args) => tags::run(path, args),
    }
}
