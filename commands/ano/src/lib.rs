use std::{path::PathBuf, sync::{atomic::AtomicUsize, Arc}, time::Instant, usize};
use pulsedcm_core::*;
use dashmap::DashMap;
use uuid::Uuid; 

pub mod models;
pub mod uid;

use crate::models::{DEID_MAP};

// Charged to invoke rayon with the pool of files
pub fn threading_handling(
    files: Vec<PathBuf>, 
    output_path: PathBuf,
    dry: &mut bool, 
    with_pixel_data: bool,
    _workers: usize,
    batch: usize,
    verbose: bool, 
    uid_to_hash: &bool
    ) -> Result<()> {

    if !output_path.exists(){
        match std::fs::create_dir(&output_path){
            Ok(_) => {},
            Err(e) => eprintln!("Couldn't create the output folder: {e}"),
        }
    }
    if !output_path.is_dir() {
        eprintln!("Output path shouldn't be a file");
        return Ok(());
    }
    pulsedcm_core::tracing_subscriber::fmt()
        .json()
        .with_writer(std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("log.json")
            .unwrap()
        )
        .try_init().ok();

    // Compute the metadata for logging
    let job_id = Uuid::new_v4().to_string();
    let start_ms = Instant::now(); // Measure time of exec


    let span = info_span!("job",
        job_id = %job_id,
        action = "ano",
        files_total = files.len()
    );

    let _enter = span.enter();

    let thread_pool = rayon::ThreadPoolBuilder::new()
        // .num_threads(workers)
        .build()?;

    // Setup the hasmap 
    // TODO: Might need a better way to hashmap it
    let uid_map : Arc<DashMap<String, String>> = Arc::new(DashMap::new());

    let files_success= Arc::new(AtomicUsize::new(0));
    let files_failed = Arc::new(AtomicUsize::new(0));
    if let Some((first, rest)) = files.split_first(){
        let mut chunks = if batch > 1 {
            rest.chunks(batch).collect::<Vec<&[PathBuf]>>()
        } else {
            vec![rest]
        };
        if *dry {
            single_thread_process(first.into(), &mut output_path.clone(),verbose ,dry, with_pixel_data, &uid_map, uid_to_hash)?;
            *dry = false;
        }
        else {
            chunks = files.chunks(batch).collect::<Vec<&[PathBuf]>>();
        }
        for chunk in chunks {
            let _ = thread_pool.install(|| {
                let _ = chunk.par_iter().try_for_each(
                    |files: &PathBuf| -> Result<()>{
                        let uid_map = Arc::clone(&uid_map);
                        match single_thread_process(files.into(), &mut output_path.clone(), verbose , dry, with_pixel_data, &uid_map, uid_to_hash){
                            Ok(_o) => {files_success.fetch_add(1, std::sync::atomic::Ordering::Relaxed)},
                            Err(_e)=> {files_failed.fetch_add(1, std::sync::atomic::Ordering::Relaxed)},
                        };
                        Ok(())
                    });
            });
        }
    }
    let files_success = files_success.load(std::sync::atomic::Ordering::Relaxed) as u64;
    let files_failed = files_failed.load(std::sync::atomic::Ordering::Relaxed) as u64;
    let duration_ms = start_ms.elapsed().as_millis() as u64;

    let status = if files_failed == 0 {"success"}
                 else if files_success > 0 {"partial_success"}
                 else {"failed"};
    info!(
        files_success,
        files_failed,
        duration_ms,
        status,
        "job_completed"
    );
    Ok(())
}

// Handle the reading of file -> process -> write
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
    
    de_identify(&mut data, uid_map, uid_to_hash)?;

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
                            // TODO: Could do a specific "try_de_identify" function
                            if let Err(e) = de_identify(item, uid_map, uid_to_hash) {
                                eprintln!("Failed to deidentify item: {}", e);
                            }
                        }
                    }
                    });
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


#[cfg(test)]
mod tests {
    use super::*;

    pub fn create_valid_dicom() -> InMemDicomObject {
        let mut obj = InMemDicomObject::new_empty();
        obj.put(InMemElement::new(
                Tag(0x0010, 0x0010), // PatientName
                VR::PN,
                PrimitiveValue::from("John^Doe"),
        ));

        obj.put(InMemElement::new(
                Tag(0x0010, 0x0020), // PatientID
                VR::LO,
                PrimitiveValue::from("12345"),
        ));

        obj.put(InMemElement::new(
                Tag(0x0008, 0x0018), // SOPInstanceUID
                VR::UI,
                PrimitiveValue::from("1.2.3.4.5"),
        ));

        obj.put(InMemElement::new(
                Tag(0x0008, 0x0020), // StudyDate
                VR::DA,
                PrimitiveValue::from("20240101"),
        ));
        obj
    }
    pub fn create_valid_nested_sq_dicom() -> InMemDicomObject {
        let mut obj = InMemDicomObject::new_empty();
        obj.put(InMemElement::new(
                Tag(0x0010, 0x0010), // PatientName
                VR::PN,
                PrimitiveValue::from("John^Doe"),
        ));

        obj.put(InMemElement::new(
                Tag(0x0008, 0x0018), // PatientID
                VR::LO,
                PrimitiveValue::from("1.2.3.4.5"),
        ));
        obj.put(InMemElement::new(
                Tag(0x0008, 0x1115), // PatientID
                VR::SQ,
                DataSetSequence::from(vec![InMemDicomObject::from_element_iter([
                        InMemElement::new(
                            Tag(0x0010, 0x0010), // PatientName
                            VR::PN,
                            PrimitiveValue::from("John^Doe"),
                        ), 
                        InMemElement::new(
                            Tag(0x0008, 0x0018), // SOPInstanceUID
                            VR::UI,
                            PrimitiveValue::from("1.2.3.4.5"),
                        ), 
                        DataElement::new(
                            Tag(0x0008, 0x1199), // SOPInstanceUID
                            VR::SQ,
                            DataSetSequence::from(vec![InMemDicomObject::from_element_iter([
                                    DataElement::new(
                                        Tag(0x0010, 0x0010), // PatientName
                                        VR::PN,
                                        PrimitiveValue::from("John^Doe"),
                                    ), 
                                    DataElement::new(
                                        Tag(0x0008, 0x0018), // SOPInstanceUID
                                        VR::UI,
                                        PrimitiveValue::from("1.2.3.4.5"),
                                    ), 
                            ])]),
                            ), 
                ])]),
        ));
        obj
    }
    pub fn create_corrupted_dicom() -> InMemDicomObject{
        let mut obj = InMemDicomObject::new_empty();

        // Invalid UID content
        obj.put(InMemElement::new(
                Tag(0x0008, 0x0018),
                VR::UI,
                PrimitiveValue::from(vec![0xFF, 0xFF, 0xFF]),
        ));

        // Broken text encoding
        obj.put(InMemElement::new(
                Tag(0x0010, 0x0010),
                VR::PN,
                PrimitiveValue::from(vec![0x00, 0x00]),
        ));

        obj
    }

    #[test]
    fn valid_dicom_dummy_deid(){
        let mut obj = create_valid_dicom(); 
        let uid_map : Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        let uid_to_hash = false;
        let res = de_identify(&mut obj, &uid_map, &uid_to_hash);
        assert!(res.is_ok());
    }
    
    #[test]
    fn corrupted_dicom_dummy_deid(){
        let mut obj = create_corrupted_dicom(); 
        let uid_map : Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        let uid_to_hash = false;
        let res = de_identify(&mut obj, &uid_map, &uid_to_hash);
        assert!(res.is_ok() || res.is_err());
    }
    
    #[test]
    fn correctly_remove_phi(){
        let mut obj = create_valid_dicom();
        let uid_map : Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        let uid_to_hash = false;
        let res = de_identify(&mut obj, &uid_map, &uid_to_hash).unwrap();
        assert!(obj.get(Tag(0x0010, 0x0010)).is_none());
        assert!(obj.get(Tag(0x0010, 0x0020)).is_none());
        assert_ne!(obj.value_at(Tag(0x0008, 0x0018)).unwrap().string().unwrap(), "1.2.3.4");
        assert_eq!(*obj.value_at(Tag(0x0008, 0x0020)).unwrap().primitive().unwrap(), PrimitiveValue::Empty);
    }


    #[test]
    fn correct_SQ_traversal() {
        let mut obj = create_valid_nested_sq_dicom();
        let uid_map : Arc<DashMap<String, String>> = Arc::new(DashMap::new());
        let uid_to_hash = false;
        let res = de_identify(&mut obj, &uid_map, &uid_to_hash).unwrap();
        // First layer
        assert!(obj.value_at(AttributeSelector::from(Tag(0x0010, 0x0010))).is_err());
        assert_ne!(obj.value_at(Tag(0x0008, 0x0018)).unwrap().string().unwrap(), "1.2.3.4");
        assert!(obj.get(Tag(0x0008, 0x1115)).is_some(), "{:?}", obj.get(Tag(0x0008, 0x1115)));

        // Second layer
        assert!(obj.value_at(AttributeSelector::from((
                        Tag(0x0008, 0x1115),
                        0,
                        Tag(0x0010, 0x0010)
        ))).is_err());
        assert_ne!(obj.value_at(AttributeSelector::from((
                        Tag(0x0008, 0x1115),
                        0,
                        Tag(0x0008, 0x0018)
        ))).unwrap().string().unwrap(), "1.2.3.4");

        // Third layer
        assert!(obj.value_at(AttributeSelector::from((
                        Tag(0x0008, 0x1115),
                        0,
                        Tag(0x0008, 0x1199), 
                        0,
                        Tag(0x0010, 0x0010)

        ))).is_err());
        assert_ne!(obj.value_at(AttributeSelector::from((
                        Tag(0x0008, 0x1115),
                        0,
                        Tag(0x0008, 0x1199), 
                        0,
                        Tag(0x0008, 0x0018)
        ))).unwrap().string().unwrap(), "1.2.3.4");
    }
}
