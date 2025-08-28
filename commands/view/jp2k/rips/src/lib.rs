
#[cfg(feature = "docs-rs")]
#[path = "ffi.ref.rs"]
mod ffi;

#[cfg(not(feature = "docs-rs"))]
#[allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused, improper_ctypes)]
mod ffi;

#[macro_use]
mod utils;

pub mod err {
    #[derive(Debug)]
    pub enum Error {
        Vips(Option<String>),
        NulError(std::ffi::NulError),
        Io(std::io::Error),
        Boxed(Box<dyn std::error::Error + Send + Sync>),
    }

    impl Error {
        pub(crate) fn from_vips() -> Self {
            let out = unsafe {
                let ptr = crate::ffi::vips_error_buffer();
                if ptr.is_null() {
                    None
                } else {
                    Some(std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned())
                }
            };
            Error::Vips(out)
        }
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use Error::*;
            match self {
                Vips(ref e) => {
                    let msg =
                        e.as_ref().map(|x| x.as_str()).unwrap_or_else(|| "Unknown VIPS error");
                    write!(f, "{}", msg)?;
                }
                NulError(ref e) => {
                    write!(f, "{}", e)?;
                }
                Io(ref e) => {
                    write!(f, "{}", e)?;
                }
                Boxed(ref e) => {
                    write!(f, "{}", e)?;
                }
            }

            Ok(())
        }
    }

    impl From<std::ffi::NulError> for Error {
        fn from(t: std::ffi::NulError) -> Self {
            Error::NulError(t)
        }
    }

    impl std::error::Error for Error {}

    pub type Result<T> = std::result::Result<T, Error>;
}

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

pub use ffi::{VipsAngle, VipsBandFormat, VipsKernel};

const NULL_TERM: *const c_char = ptr::null();

#[derive(Default)]
pub struct InitOptions {
    name: Option<String>,
    leak_checks_on: Option<bool>,
}

impl InitOptions {
    pub fn with_name<I: Into<String>>(mut self, name: I) -> Self {
        self.name = Some(name.into());
        self
    }
    pub fn with_leak_checks(mut self, on: bool) -> Self {
        self.leak_checks_on = Some(on);
        self
    }
}

/// Call this if you need to specify a name for your program in VIPS,
/// or if you need to turn VIPS leak checking on. Otherwise, any image
/// open/build operations will initialize VIPS

pub fn initialize_with_options(opts: InitOptions) {
    initialize_with_maybe_options(Some(opts))
}

fn initialize_with_maybe_options(opts: Option<InitOptions>) {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| unsafe {
        let opts = opts.unwrap_or_else(InitOptions::default);
        match opts.name.as_ref().and_then(|n| CString::new(n.clone()).ok()) {
            Some(name) => {
                ffi::vips_init(name.as_ptr());
            }
            None => {
                let name = CStr::from_bytes_with_nul_unchecked(b"rips\0");
                ffi::vips_init(name.as_ptr());
            }
        };

        if let Some(&leak_checks_on) = opts.leak_checks_on.as_ref() {
            ffi::vips_leak_set(leak_checks_on as c_int);
        }

        assert_eq!(libc::atexit(cleanup), 0);
    });

    extern "C" fn cleanup() {
        unsafe {
            ffi::vips_shutdown();
        }
    }
}

fn initialize() {
    initialize_with_maybe_options(None)
}

pub struct Image(*mut ffi::VipsImage);

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            ffi::g_object_unref(self.0 as *mut c_void);
        }
    }
}

impl Image {
    fn new() -> Self {
        Image(ptr::null_mut())
    }

    pub fn width(&self) -> i32 {
        unsafe { ffi::vips_image_get_width(self.0) }
    }

    pub fn height(&self) -> i32 {
        unsafe { ffi::vips_image_get_height(self.0) }
    }

    pub fn from_file<S: Into<Vec<u8>>>(path: S) -> err::Result<Image> {
        initialize();

        let path = CString::new(path)?;
        let ptr = unsafe { ffi::vips_image_new_from_file(path.as_ptr(), NULL_TERM) };
        if ptr.is_null() {
            Err(err::Error::from_vips())
        } else {
            Ok(Image(ptr))
        }
    }

    pub fn from_memory(
        buf: Vec<u8>,
        width: i32,
        height: i32,
        bands: i32,
        format: VipsBandFormat,
    ) -> err::Result<Image> {
        pub unsafe extern "C" fn post_close(_: *mut ffi::VipsImage, user_data: *mut c_void) {
            let buf = Box::from_raw(user_data as *mut Box<[u8]>);
            drop(buf);
        }

        initialize();

        let buf = buf.into_boxed_slice();

        let ptr = unsafe {
            ffi::vips_image_new_from_memory(
                buf.as_ptr() as *const c_void,
                buf.len(),
                width,
                height,
                bands,
                format,
            )
        };

        let buf = Box::new(buf);

        let raw: *mut c_void = Box::into_raw(buf) as *mut c_void;

        unsafe {
            let callback: unsafe extern "C" fn() = std::mem::transmute(post_close as *const ());
            ffi::g_signal_connect_data(
                ptr as *mut c_void,
                "postclose\0".as_ptr() as *const c_char,
                Some(callback),
                raw,
                None,
                ffi::GConnectFlags::G_CONNECT_AFTER,
            );
        };

        if ptr.is_null() {
            Err(err::Error::from_vips())
        } else {
            Ok(Image(ptr))
        }
    }

    pub fn resize(
        &self,
        scale: f64,
        vscale: Option<f64>,
        kernel: Option<VipsKernel>,
    ) -> err::Result<Image> {
        let mut out = Image::new();

        let ret = unsafe {
            var_args!(ffi::vips_resize, 
                args => [self.0, &mut out.0, scale,], 
                opts => [(vscale, vscale, "vscale\0".as_ptr(), vscale), (kernel, kernel, "kernel\0".as_ptr(), kernel),],
                term => NULL_TERM)
        };

        match ret {
            0 => Ok(out),
            _ => Err(err::Error::from_vips()),
        }
    }

    pub fn resize_to(&self, width: Option<i32>, height: Option<i32>) -> err::Result<Image> {
        match (width, height) {
            (Some(width), Some(height)) => {
                let hscale = f64::from(width) / f64::from(self.width());
                let vscale = f64::from(height) / f64::from(self.height());
                if hscale != vscale {
                    self.resize(hscale, Some(vscale), None)
                } else {
                    self.resize(hscale, None, None)
                }
            }
            (Some(width), None) => {
                self.resize(f64::from(width) / f64::from(self.width()), None, None)
            }
            (None, Some(height)) => {
                self.resize(f64::from(height) / f64::from(self.height()), None, None)
            }
            (None, None) => self.resize(1.0, None, None),
        }
    }

    pub fn crop(&self, left: i32, top: i32, width: i32, height: i32) -> err::Result<Image> {
        let mut out = Image::new();

        let ret =
            unsafe { ffi::vips_crop(self.0, &mut out.0, left, top, width, height, NULL_TERM) };

        match ret {
            0 => Ok(out),
            _ => Err(err::Error::from_vips()),
        }
    }

    pub fn rotate(&self, angle: VipsAngle) -> err::Result<Image> {
        let mut out = Image::new();

        let ret = unsafe { ffi::vips_rot(self.0, &mut out.0, angle, NULL_TERM) };

        match ret {
            0 => Ok(out),
            _ => Err(err::Error::from_vips()),
        }
    }

    pub fn write_to_file<S: Into<Vec<u8>>>(&self, path: S) -> err::Result<()> {
        let path = CString::new(path)?;
        let ret = unsafe { ffi::vips_image_write_to_file(self.0, path.as_ptr(), NULL_TERM) };
        match ret {
            0 => Ok(()),
            _ => Err(err::Error::from_vips()),
        }
    }

    pub fn to_buffer(&self, suffix: &str) -> err::Result<Vec<u8>> {
        let suffix = CString::new(String::from(suffix))?;
        let mut size = 0usize;
        let mut buf = ptr::null_mut::<u8>();

        unsafe {
            let ret = ffi::vips_image_write_to_buffer(
                self.0,
                suffix.as_ptr(),
                &mut buf as *mut *mut u8 as *mut *mut c_void,
                &mut size,
                NULL_TERM,
            );

            let slice = std::slice::from_raw_parts_mut(buf as *mut u8, size);
            let boxed: Box<[u8]> = Box::from_raw(slice);
            let out = boxed.into_vec();

            match ret {
                0 => Ok(out),
                _ => Err(err::Error::from_vips()),
            }
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut size = 0usize;
        unsafe {
            let memory: *mut u8 = ffi::vips_image_write_to_memory(self.0, &mut size) as *mut u8;
            let slice = std::slice::from_raw_parts_mut(memory, size);
            let boxed: Box<[u8]> = Box::from_raw(slice);
            boxed.into_vec()
        }
    }
}

unsafe impl Send for Image {}
unsafe impl Sync for Image {}
