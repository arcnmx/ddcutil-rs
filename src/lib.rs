#![doc(html_root_url = "http://arcnmx.github.io/ddcutil-rs/")]
pub extern crate ddcutil_sys as sys;
extern crate libc;
#[macro_use]
extern crate bitflags;

use std::{result, str};

mod status;
pub use status::{Status, Error};
pub type Result<T> = result::Result<T, Error>;

mod display;
pub use display::*;

mod features;
pub use features::*;

pub type Version = sys::DDCA_Ddcutil_Version_Spec;

unsafe fn c_str<'a>(ptr: *const libc::c_char) -> result::Result<&'a str, str::Utf8Error> {
    use std::ffi::CStr;

    str::from_utf8(CStr::from_ptr(ptr).to_bytes())
}

pub fn version() -> Version {
    unsafe {
        sys::ddca_ddcutil_version()
    }
}

pub fn version_string() -> &'static str {
    unsafe {
        c_str(sys::ddca_ddcutil_version_string())
            .expect("ddcutil returned invalid version string")
    }
}

pub fn verification_enabled() -> bool {
    unsafe {
        sys::ddca_is_verify_enabled()
    }
}

pub fn set_verification(onoff: bool) {
    unsafe {
        sys::ddca_enable_verify(onoff)
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OutputLevel {
    Terse = sys::DDCA_OL_TERSE as _,
    Normal = sys::DDCA_OL_NORMAL as _,
    Verbose = sys::DDCA_OL_VERBOSE as _,
}

impl Default for OutputLevel {
    fn default() -> Self {
        OutputLevel::Normal
    }
}

impl OutputLevel {
    pub fn from_raw(raw: sys::DDCA_Output_Level) -> result::Result<Self, ()> {
        match raw {
            sys::DDCA_OL_TERSE => Ok(OutputLevel::Terse),
            sys::DDCA_OL_NORMAL => Ok(OutputLevel::Normal),
            sys::DDCA_OL_VERBOSE => Ok(OutputLevel::Verbose),
            _ => Err(()),
        }
    }

    pub fn to_raw(&self) -> sys::DDCA_Output_Level {
        *self as _
    }
}

pub fn output_level() -> OutputLevel {
    unsafe {
        OutputLevel::from_raw(sys::ddca_get_output_level())
            .expect("invalid output level")
    }
}

pub fn set_output_level(value: OutputLevel) {
    unsafe {
        sys::ddca_set_output_level(value.to_raw())
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RetryType {
    WriteOnly = sys::DDCA_WRITE_ONLY_TRIES as _,
    WriteRead = sys::DDCA_WRITE_READ_TRIES as _,
    MultiPart = sys::DDCA_MULTI_PART_TRIES as _,
}

impl RetryType {
    pub fn to_raw(&self) -> sys::DDCA_Retry_Type {
        *self as _
    }
}

pub fn max_retries() -> usize {
    unsafe {
        sys::ddca_max_max_tries() as _
    }
}

pub fn get_retries(ty: RetryType) -> usize {
    unsafe {
        sys::ddca_get_max_tries(ty.to_raw()) as _
    }
}

pub fn set_retries(ty: RetryType, value: usize) -> Result<()> {
    unsafe {
        Error::from_status(sys::ddca_set_max_tries(ty.to_raw(), value as _)).map(drop)
    }
}

#[test]
fn test_api() {
    let _ = version();
    let _ = version_string();
    let _ = output_level();
    let _ = verification_enabled();
    let _ = max_retries();
    for ty in vec![RetryType::WriteOnly, RetryType::WriteRead, RetryType::MultiPart] {
        let _ = get_retries(ty);
    }
}

#[test]
fn test_output_level() {
    for level in vec![OutputLevel::Terse, OutputLevel::Normal, OutputLevel::Verbose] {
        set_output_level(level);
    }
}

#[test]
fn test_retries() {
    let max = max_retries();
    for ty in vec![RetryType::WriteOnly, RetryType::WriteRead, RetryType::MultiPart] {
        for retry in 1..(max + 1) {
            assert!(set_retries(ty, retry).is_ok());
        }
        assert!(set_retries(ty, max + 1).is_err());
    }
}
