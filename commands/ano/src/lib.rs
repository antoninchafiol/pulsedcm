use std::{ os::unix::fs::OpenOptionsExt, path::PathBuf};

pub mod models;
// pub use models;
use pulsedcm_core::*;

use crate::models::DEID_HASH;

pub fn threading_handling(
    files: Vec<PathBuf>, 
    output_path: PathBuf,
    dry: &mut bool, 
    with_pixel_data: bool,
    jobs: usize,
    verbose: bool, 
    ) -> Result<()> {
    
    println!("Dry?: {}", dry);
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()?;

    if let Some((first, rest)) = files.split_first(){
        if *dry {
            single_thread_process(first.into(), &mut output_path.clone(),verbose ,dry, with_pixel_data)?;
        }
        *dry = false;

        let _ = thread_pool.install(|| {
            let _ = rest.par_iter().try_for_each(
                |file: &PathBuf| -> Result<()> {
                    single_thread_process(file.into(), &mut output_path.clone(), verbose , dry, with_pixel_data)?;
                    Ok(())
                });
        });
    }

    Ok(())
}

pub fn single_thread_process(
    input_path: PathBuf,
    output_path: &mut PathBuf,
    verbose: bool,
    dry: &bool,
    with_pixel_data: bool
) -> Result<()> {
    let data = de_identify_file(input_path.clone(), with_pixel_data, verbose)?; 

    if *dry {
        if verbose {
            println!("Launching a dry run");
        }
        print_tags(&data);
        return Ok(());
    }
    println!("testing, {}", dry);
    let filename = input_path.file_name().unwrap_or_default();

    // Case where out is not specified
    if input_path.to_str().unwrap_or_default() == output_path.as_os_str().to_str().unwrap_or_default() {
        if ask_yes_no("? No output_path specified confirm to overwrite actual files") {
            output_path.push(filename);
            data.write_to_file(&output_path)?;
        } else {
            println!("Stopping...");
            return Ok(());
        }
    } else {
        if !output_path.is_dir() {
            eprintln!("Output path shouldn't be a file");
            return Ok(());
        }
        output_path.push(filename);
        data.write_to_file(&output_path)?;
    }
    Ok(())
}

fn de_identify_file (
    file_path: PathBuf, 
    // profile: Profile :TODO: Later implement the profile to match the right 
    // one in policyAction
    with_pixel_data: bool,
    verbose: bool
) -> Result<FileDicomObject<InMemDicomObject>> {
    
    let mut data = if !with_pixel_data {
        OpenFileOptions::new()
            .read_until(dicom_dictionary_std::tags::PIXEL_DATA)
            .open_file(file_path)?
    } else {
        open_file(file_path)?
    };

    for (key, value) in DEID_HASH.entries() {
        let rec_tag: Tag = Tag{0: key.0, 1:key.1};
        // Check if in 
        if let Ok(elem) = data.element(rec_tag) {
            let vr = elem.vr();
            value.basic.process(&mut data, &rec_tag, &vr)?;
        } else {
            continue;
        }
    }
    Ok(data)


    // for element in data.into_iter() {
    //     let tag = element.header().tag;
    //     // TODO: Get both u16 and change the DEID HASH from Tag to (u16, u16)
    //     let vr = element.header().vr;
    //     // Check if contains
    //     // if yes: apply the 1st profile
    //     // + check for the rest of profiles to add afterwards 
    //     // else : pass
    // }
    // }
    // Ok(data)



}
