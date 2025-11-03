use pulsedcm_core::*;
use std::path::PathBuf;

use std::process::Command;

use tempfile::TempDir;

use jp2k::{Codec, DecodeParams, ImageBuffer, Stream};
use image;

pub fn run(
    files: Vec<PathBuf>,
    open: u8, 
    temp: bool,
    out: PathBuf,
    jobs: usize
) -> Result<()> {
    let mut open: u8 = open;
    let is_temp: bool = temp;


    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs as usize)
        .build()
        .unwrap();

    if is_temp {
        match TempDir::new() {
            Ok(tmp_dir) => {
                if open <= 0 { open = 1; }
                let tmp_path = tmp_dir.into_path();

                let _ = thread_pool.install(|| -> Result<()> {
                    let _ = files.par_iter().enumerate().try_for_each(|(idx, file)| -> Result<()> {
                        println!("{}", file.as_os_str().to_str().unwrap_or_default());

                        let mut input_path = PathBuf::from(file);
                        let mut out_clone = tmp_path.clone();


                        view_processing(&mut input_path, &mut out_clone, idx < open as usize)
                            .unwrap_or_else(|_e| {
                                eprintln!("Can't process {} : {}", input_path.display(), _e);
                            });
                    Ok(())
                    });
                    Ok(())
                });
            }
            Err(e) => {
                return Err(e.into());
            }
        }
        println!("\x1b[1m>> \x1b[0mPress Enter to exit and delete temporary files...");
        let _ = std::io::stdin().read_line(&mut String::new());
    } else {
        thread_pool.install(|| {
            files.par_iter().enumerate().for_each(|(idx, file)| {

                let mut input_path = PathBuf::from(file);
                let mut output_path = out.clone();

                view_processing(&mut input_path, &mut output_path, idx < open as usize).unwrap_or_else(
                    |_e| {
                        eprintln!("Can't process {} : {}", input_path.display(), _e);
                    },
                );
            });
        });
    }
    Ok(())
}

fn view_processing(
    input_path: &mut PathBuf,
    output_path: &mut PathBuf,
    is_to_open: bool,
) -> Result<()> {
    // let dinput_path = input_path.to_str()?;
    let obj = open_file(input_path.as_path())?;

    output_handling(input_path, output_path)?;

    let ts = obj.meta().transfer_syntax();
    if ts == "1.2.840.10008.1.2.4.90" || ts == "1.2.840.10008.1.2.4.91"{
        let buff: Vec<u8> = build_byte_buffer(obj)?;
        handle_byte_to_jp2k(buff, output_path.to_str().unwrap())?;

    } else {
        let image = obj.decode_pixel_data()?;
        let dynamic_image = image.to_dynamic_image(0)?;
        dynamic_image.save(&output_path)?;
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


fn build_byte_buffer(obj: FileDicomObject<InMemDicomObject>) -> Result<Vec<u8>>  {
    let pixel_data = obj.element_by_name("PixelData").unwrap();
    let buff: Result<Vec<u8>> = match pixel_data.value() {
        DicomValue::Primitive(p) => Ok(p.to_bytes().into_owned().to_vec()),
        DicomValue::PixelSequence(seq) => Ok(seq.fragments().concat()),
        _ => {
            return Err(PulseError::new(
                    PulseErrorKind::UnsupportedPixelData, 
                    "The output from jp2k isn't supported",
            ));
        },
    };
        buff
}

fn handle_byte_to_jp2k(buff: Vec<u8>, output_path: &str) -> Result<()> {
    let stream = Stream::from_bytes(&buff)?;
    let codec = Codec::create(jp2k::CODEC_FORMAT::OPJ_CODEC_J2K)?;

    let img_buf = ImageBuffer::build(codec, stream, DecodeParams::default())?;
    let dyn_img = match img_buf.num_bands {
        1 => image::DynamicImage::ImageLuma8(
            image::GrayImage::from_raw(img_buf.width, img_buf.height, img_buf.buffer).unwrap(),
        ),
        3 => image::DynamicImage::ImageRgb8(
            image::RgbImage::from_raw(img_buf.width, img_buf.height, img_buf.buffer).unwrap(),
        ),
        4 => image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(img_buf.width, img_buf.height, img_buf.buffer).unwrap(),
        ),
        _ => {
            return Err(PulseError::new(
                    PulseErrorKind::UnsupportedComponent, 
                    "Component not supported",
            ));
        }
    };

    dyn_img.save(output_path)?;
    Ok(())
}
