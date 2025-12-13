use std::{path::PathBuf, sync::Arc};
use pulsedcm_core::*;
use dashmap::DashMap; 

pub mod models;

use crate::models::{DEID_MAP};

pub fn threading_handling(
    files: Vec<PathBuf>, 
    output_path: PathBuf,
    dry: &mut bool, 
    with_pixel_data: bool,
    workers: usize,
    batch: usize,
    verbose: bool, 
    uid_to_hash: &bool
    ) -> Result<()> {
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()?;

    // Setup the hasmap 
    // TODO: Might need a better way to hashmap it
    let uid_map : Arc<DashMap<String, String>> = Arc::new(DashMap::new());

    if let Some((first, rest)) = files.split_first(){
        if *dry {
            single_thread_process(first.into(), &mut output_path.clone(),verbose ,dry, with_pixel_data, &uid_map, uid_to_hash)?;
        }
        *dry = false;
        let chunks = if batch > 1 {
            rest.chunks(batch).collect::<Vec<&[PathBuf]>>()
        } else {
            vec![rest]
        };
        for chunk in chunks {
            let _ = thread_pool.install(|| {
                let _ = chunk.par_iter().try_for_each(
                    |file: &PathBuf| -> Result<()> {
                        let uid_map = Arc::clone(&uid_map);
                        single_thread_process(file.into(), &mut output_path.clone(), verbose , dry, with_pixel_data, &uid_map, uid_to_hash)?;
                        Ok(())
                    });
            });
        }
    }

    Ok(())
}

pub fn single_thread_process(
    input_path: PathBuf,
    output_path: &mut PathBuf,
    verbose: bool,
    dry: &bool,
    with_pixel_data: bool, 
    uid_map: &Arc<DashMap<String, String>>,
    uid_to_hash: &bool
) -> Result<()> {
    let mut data = if !with_pixel_data {
        OpenFileOptions::new()
            .read_until(dicom_dictionary_std::tags::PIXEL_DATA)
            .open_file(input_path.clone())?
    } else {
        open_file(input_path.clone())?
    };
    
    // TODO: Check result
    let process_result = de_identify(&mut data, uid_map, uid_to_hash);

    // let data = de_identify_file(input_path.clone(), with_pixel_data, verbose, uid_map)?; 

    if *dry {
        if verbose {
            println!("Launching a dry run");
        }
        print_tags(&data);
        return Ok(());
    }
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

fn de_identify(
    data: &mut InMemDicomObject, 
    uid_map: &Arc<DashMap<String, String>>,
    uid_to_hash: &bool
) -> Result<()> {
    let tags: Vec<Tag> = data.tags().collect();
    for tag in tags {
        // Get all tags and retrieve the VR from it 
        let vr = data.get(tag).unwrap().vr();
            match vr {
                // If a Sequence, Rebuild a InMemDicomObject from it and recurse 
                // (Allow to search for inner tags that could match)
                VR::SQ => {
                    let _ = data.update_value_at(tag, |val| {
                    if let Some(items) = val.items_mut() {
                        for item in items {
                            de_identify(item, uid_map, uid_to_hash);
                        }
                    }
                    })?;
                },
                // If not a sequence, check if we have the attribute/tag within our policy hashmap
                _ => {
                    // Compress the tag (two u16) into a u32 for the hashmap checks
                    let compressed_tag: u32 = ((tag.0 as u32) << 16) + tag.1 as u32;
                    
                    if let Some(policy_action) = DEID_MAP.get(&compressed_tag){
                        policy_action.apply(data, &compressed_tag, &vr, uid_map, uid_to_hash)?;
                    }
                },
            }
    }
    Ok(())
}
