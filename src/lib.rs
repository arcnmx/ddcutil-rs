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

#[test]
fn test_version() {
	let _ = version();
	let _ = version_string();
}
