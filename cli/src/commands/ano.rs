use clap::Args;
use std::path::PathBuf;

use pulsedcm_commands_ano::{run as ano_run, Action, Policy};

use crate::commands::{CliCommand};

#[derive(Args, Debug)]
pub struct AnoArgs {
    /// strategy of de-idenficication:
    /// replace: Replace with dummy value
    /// zero: Zero length
    /// remove: Remove completely 
    #[arg(
        long, 
        value_name="ACTION",
        value_parser=parse_actions
    )]
    action: Option<Action>,
    
    /// De-identification profiles
    /// basic: Remove required PHI (safe)
    /// moderate: Also remove institution/device info
    /// strict: Maximum removal, leaves only technical data
    #[arg(
        long, 
        value_name="POLICY",
        value_parser=parse_policy
    )]
    policy: Option<Policy>,

    /// Number of threads to launch to process (0 = All possible threads)
    #[arg(long, value_name="NUMBER")]
    jobs: Option<usize>,

    /// Output directory to save anonymized files.
    /// If omitted, input files will be overwritten in-place.
    /// Must be a directory if specified.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Show the changed args for the file
    /// If multiple files it'll stop processing after the 1st to give an output
    #[arg(short, long, default_value= "false")]
    dry: bool,

    /// Show all changed values
    #[arg(short, long)]
    verbose: bool,

} 
fn parse_actions(input: &str) -> Result<Action, String>{
    match input.to_lowercase().as_str() {
        "replace" => Ok(Action::Replace),
        "remove"  => Ok(Action::Remove),
        "zero"    => Ok(Action::Zero),
        _other => {
            Err("should be either: 'replace', 'remove' or 'zero'".to_string())
        }
    }
}

fn parse_policy(input: &str) -> Result<Policy, String>{
    match input.to_lowercase().as_str() {
        "basic"    => Ok(Policy::Basic) ,
        "moderate" => Ok(Policy::Moderate) ,
        "strict"   => Ok(Policy::Strict) ,
        _other => {
            Err("should be either: 'basic', 'moderate' or 'strict'".to_string())
        }
    }
}


impl CliCommand for AnoArgs {
    fn run(&self, path: &str) {
        let mut dry_arg = self.dry;
        let default_out_path = PathBuf::from(&path);
        match ano_run(
            path, 
            self.action.as_ref().unwrap_or(&Action::Zero), 
            self.policy.as_ref().unwrap_or(&Policy::Basic), 
            self.out.as_ref().unwrap_or_else(|| {
                if self.verbose {
                    println!("out argument has issue when parsing"); 
                }
                &default_out_path
            }),
            &mut dry_arg, 
            self.verbose,
            self.jobs,
        ){
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error when running ano command: {}", e);
            }
        };
    }
}
