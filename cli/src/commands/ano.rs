use clap::Args;
use std::path::PathBuf;

use pulsedcm_commands_ano::{Action, Policy, threading_handling};

use crate::commands::{ArgRun};

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

    /// Output directory to save anonymized files.
    /// If omitted, input files will be overwritten in-place.
    /// Must be a directory if specified.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Show the changed args for the file
    /// If multiple files it'll stop processing after the 1st to give an output
    #[arg(short, long, default_value= "false")]
    dry: bool,

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
            self.action.unwrap_or(Action::Zero), 
            self.policy.unwrap_or(Policy::Basic), 
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
