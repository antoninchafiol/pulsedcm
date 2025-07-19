use std::fs;
use serde::Serialize;

use rayon;

pub use dicom_object::{Tag, open_file, FileDicomObject, InMemDicomObject};
pub use dicom_dictionary_std::{StandardDataDictionary};
pub use dicom_core::DataDictionary;
pub use dicom_pixeldata::PixelDecoder;

pub use std::time::{SystemTime, UNIX_EPOCH};
pub use std::fs::File;
pub use std::path::Path;

pub fn list_all_files(user_path: &str) -> Vec<String>{
    let mut res: Vec<String> = Vec::new();
    let pa = Path::new(user_path);
    if pa.is_file(){
        if pa.extension().unwrap() == "dcm" {
            res.push(user_path.to_string());
        } 
    } else {
        for p in fs::read_dir(user_path).unwrap(){
            if let Ok(p) = p {
                if let Some(p) = p.path().to_str(){
                    res.extend(list_all_files(p));
                }
            }
        }
    }
    res
}

#[derive(Serialize)]
pub struct SerializableDicomEntry {
    pub filename: String,
    pub name: String,
    pub tag: String,
    pub vr: String,
    pub value: String,
}

pub fn jobs_handling(jobs: Option<usize>, max_file: usize) -> usize {
    let j = jobs.unwrap_or_else(|| {
        return 1;
    });
    if j <= 0 {
        // MAX NUMBER OF THREADS POSSIBLE BY MACHINEa
        if max_file > rayon::max_num_threads() {
            return rayon::max_num_threads();
        }
        else {
            return max_file;  
        }
    }
    else {
        return j;
    }
}
