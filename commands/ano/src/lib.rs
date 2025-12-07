use std::{path::PathBuf, sync::Arc};
use pulsedcm_core::*;
use dashmap::DashMap; 

pub mod models;
pub mod iod;

use crate::models::{DEID_HASH, DEID_MAP};

pub fn threading_handling(
    files: Vec<PathBuf>, 
    output_path: PathBuf,
    dry: &mut bool, 
    with_pixel_data: bool,
    jobs: usize,
    verbose: bool, 
    ) -> Result<()> {
    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()?;

    // Setup the hasmap 
    // TODO: Might need a better way to hashmap it
    let uid_map : Arc<DashMap<String, String>> = Arc::new(DashMap::new());

    if let Some((first, rest)) = files.split_first(){
        if *dry {
            single_thread_process(first.into(), &mut output_path.clone(),verbose ,dry, with_pixel_data, &uid_map)?;
        }
        *dry = false;

        let _ = thread_pool.install(|| {
            let _ = rest.par_iter().try_for_each(
                |file: &PathBuf| -> Result<()> {
                    let uid_map = Arc::clone(&uid_map);
                    single_thread_process(file.into(), &mut output_path.clone(), verbose , dry, with_pixel_data, &uid_map)?;
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
    with_pixel_data: bool, 
    uid_map: &Arc<DashMap<String, String>>
) -> Result<()> {
    let data = de_identify_file(input_path.clone(), with_pixel_data, verbose, uid_map)?; 

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

fn de_identify_file (
    file_path: PathBuf, 
    // profile: Profile :TODO: Later implement the profile to match the right 
    // one in policyAction
    with_pixel_data: bool,
    verbose: bool,
    uid_map: &Arc<DashMap<String, String>>
) -> Result<FileDicomObject<InMemDicomObject>> {
    
    let mut data = if !with_pixel_data {
        OpenFileOptions::new()
            .read_until(dicom_dictionary_std::tags::PIXEL_DATA)
            .open_file(file_path)?
    } else {
        open_file(file_path)?
    };
    // let metadata = data.meta();
    // let mut data = data.into_inner();

    for t in data.tags() {
        if let Some(el) = data.get(t){
            if el.header().vr == VR::SQ {
                // Rebuild and recurse
            }
            if let Some(oui) = DEID_MAP.get((t.0, t.1)){
                oui.apply(data, tag, vr, uid_map);
            }
        }

    }
    




    for (key, value) in DEID_HASH.entries() {
        let rec_tag: Tag = Tag{0: key.0, 1:key.1};
        // Check if in 
        if let Ok(elem) = data.element(rec_tag) {
            let vr = elem.vr();
            value.basic.process(&mut data, &rec_tag, &vr, uid_map)?;
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


fn de_identify(
    data: &mut InMemDicomObject, 
    uid_map: &Arc<DashMap<String, String>>,
) {
    let tags: Vec<Tag> = data.tags().collect();
    for tag in tags {
        let vr = data.get(tag).unwrap().vr();
        // Get all tags and retrieve the VR from it 
            match vr {
                VR::SQ => {
                    let _ = data.update_value_at(tag, |val| {
                    // If a Sequence, Rebuild a InMemDicomObject from it and recurse
                    if let Some(items) = val.items_mut() {
                        for item in items {
                            de_identify(item, uid_map);
                        }
                    }
                    });
                },
                _ => {
                    // If not a sequence, check if we have the attribute/tag within our policy hashmap
                    let tag_tuple = (tag.0, tag.1);
                    if let Some(policy_action) = DEID_MAP.get(&tag_tuple){
                        policy_action.apply(data, &tag_tuple, uid_map);
                    }
                },
            }
    }
}
