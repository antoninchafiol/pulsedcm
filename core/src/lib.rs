use std::fs;
use std::path::Path;
use serde::Serialize;

fn list_all_files(user_path: &str) -> Vec<String>{
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
struct SerializableDicomEntry {
    filename: String,
    name: String,
    tag: String,
    vr: String,
    value: String,
}
