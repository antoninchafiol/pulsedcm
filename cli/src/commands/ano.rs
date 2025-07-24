use clap::Args;
use std::path::PathBuf;

use pulsedcm_commands_ano::{run as ano_run, Actions, Policy};

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
    action: Option<Actions>,
    
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

fn parse_actions(input: &str) -> Result<Actions, String>{
    match input.to_lowercase().as_str() {
        "replace" => Ok(Actions::Replace),
        "remove"  => Ok(Actions::Remove),
        "zero"    => Ok(Actions::Zero),
        other => {
            Err("should be either: 'replace', 'remove' or 'zero'".to_string())
        }
    }
}

fn parse_policy(input: &str) -> Result<Policy, String>{
    match input.to_lowercase().as_str() {
        "basic"    => Ok(Policy::Basic) ,
        "moderate" => Ok(Policy::Moderate) ,
        "strict"   => Ok(Policy::Strict) ,
        other => {
            Err("should be either: 'basic', 'moderate' or 'strict'".to_string())
        }
    }
}

pub fn run(path: &str, args: AnoArgs){
    ano_run(
        path, 
        args.action.unwrap_or(Actions::Zero), 
        args.policy.unwrap_or(Policy::Basic), 
        args.out.unwrap_or_else(|| {
            println!("out argument has issue when parsing"); 
            PathBuf::from(&path)
        }),
        args.dry, 
        args.verbose,
        args.jobs,
    );
}
