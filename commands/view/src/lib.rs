use pulsedcm_core::*;
use rayon::prelude::*;
use std::path::PathBuf;

use std::error::Error;
use std::process::Command;

use tempfile::TempDir;

use jp2k;

pub fn run(path: &str, open: u8, temp: bool, jobs: Option<usize>) {
    let mut open: u8 = open;
    let is_temp: bool = temp;

    let files = match list_all_files(path) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    let jobs: usize = jobs_handling(jobs, files.len());

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs as usize)
        .build()
        .unwrap();

    if is_temp {
        match TempDir::new() {
            Ok(dir) => {
                let path = dir.path().to_path_buf();
                if open <= 0 {
                    open = 1;
                }

                thread_pool.install(|| {
                    files.par_iter().enumerate().for_each(|(idx, file)| {
                        println!("{}", file);

                        let mut input_path = PathBuf::from(file);
                        let mut output_path =
                            path.join(input_path.file_name().unwrap_or_else(|| {
                                println!("no filename in {}", input_path.display());
                                std::ffi::OsStr::new("unknown.png")
                            }));
                        output_path.set_extension("png");

                        view_processing(&mut input_path, &output_path, idx < open as usize)
                            .unwrap_or_else(|_e| {
                                eprintln!("Can't process {} : {}", input_path.display(), _e);
                            });
                    });
                });
            }
            Err(e) => {
                eprintln!("Failed to create temporary directory: {}", e);
                return;
            }
        }
        println!("\x1b[1m>> \x1b[0mPress Enter to exit and delete temporary files...");
        let _ = std::io::stdin().read_line(&mut String::new());
    } else {
        thread_pool.install(|| {
            files.par_iter().enumerate().for_each(|(idx, file)| {
                println!("{}", file);

                let mut input_path = PathBuf::from(file);
                let mut output_path = input_path.clone();
                output_path.set_extension("png");

                view_processing(&mut input_path, &output_path, idx < open as usize).unwrap_or_else(
                    |_e| {
                        eprintln!("Can't process {} : {}", input_path.display(), _e);
                    },
                );
            });
        });
    }
}

fn view_processing(
    input_path: &mut PathBuf,
    output_path: &PathBuf,
    is_to_open: bool,
) -> Result<(), Box<dyn Error>> {
    println!("jp2k");
    let dinput_path = input_path.to_str().ok_or("Can't open the path")?;
    // Takes obj
    let obj = open_file(path)?;
    let ts = obj.meta().transfer_syntax();
    // j2pk
    if ts == "1.2.840.10008.1.2.4.90" {
        let pixel_data = obj.element_by_name("PixelData").unwrap();
        let buff: Result<Vec<u8>, &str> = match pixel_data.value() {
            DicomValue::Primitive(p) => Ok(p.to_bytes().into_owned().to_vec()),
            DicomValue::PixelSequence(seq) => {
                let offset: Vec<u8> = seq.offset_table()
                    .iter()
                    .map(
                        |&v| 
                        v.to_ne_bytes()
                    )
                    .flatten()
                    .collect();

                let fragments = seq.fragments().concat();

                Ok(vec![offset, fragments].concat())
            },
            _ => Err("The output from jp2k isn't supported")
        };
    // Build the image from buffer 
    let jp2k::Image(img) = jp2k::Image::from_bytes(
        buff, 
        jp2k::Codec::JP2,
        Some(jp2k::DecodeParams::default().with_decoding_area(0, 0, 256, 256))
    )
    .unwrap();

    }
    // Non jp2k
    else {
        let image = obj.decode_pixel_data()?;
        let dynamic_image = image.to_dynamic_image(0)?;
        dynamic_image.save(&output_path)?;
        println!("Non jp2k");
    }
    if is_to_open {
        open_image(output_path.to_str().unwrap());
    }
    Ok(())
}

fn open_image(path: &str) {
    let result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", "start", "", path]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(path).spawn()
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported OS",
        ))
    };

    if let Err(e) = result {
        eprintln!("Failed to open image: {}", e);
    }
}

fn handling_pixel_data(path: &str){
    let dinput_path = input_path.to_str().ok_or("Can't open the path")?;
    // Takes obj
    let obj = open_file(path)?;
    let ts = obj.meta().transfer_syntax();
    // j2pk
    if ts == "1.2.840.10008.1.2.4.90" {
        let pixel_data = obj.element_by_name("PixelData").unwrap();
        let buff: Result<Vec<u8>, &str> = match pixel_data.value() {
            DicomValue::Primitive(p) => Ok(p.to_bytes().into_owned().to_vec()),
            DicomValue::PixelSequence(seq) => {
                let offset: Vec<u8> = seq.offset_table()
                    .iter()
                    .map(
                        |&v| 
                        v.to_ne_bytes()
                    )
                    .flatten()
                    .collect();

                let fragments = seq.fragments().concat();

                Ok(vec![offset, fragments].concat())
            },
            _ => Err("The output from jp2k isn't supported")
        };
    // Build the image from buffer 
    let jp2k::Image(img) = jp2k::Image::from_bytes(
        buff, 
        jp2k::Codec::JP2,
        Some(jp2k::DecodeParams::default().with_decoding_area(0, 0, 256, 256))
    )
    .unwrap();
    println!("jp2k");

    }
    // Non jp2k
    else {
        let image = obj.decode_pixel_data()?;
        let dynamic_image = image.to_dynamic_image(0)?;
        dynamic_image.save(&output_path)?;
        println!("Non jp2k");
    }
    if is_to_open {
        open_image(output_path.to_str().unwrap());
    }
}
