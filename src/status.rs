use std::{fmt, error, str};
use {sys, c_str};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Status {
	status: sys::DDCA_Status,
}

impl Status {
	pub fn new(status: sys::DDCA_Status) -> Self {
		Status {
			status: status,
		}
	}

	pub fn success(&self) -> bool {
		self.status >= 0
	}

	pub fn status(&self) -> sys::DDCA_Status {
		self.status
	}

	pub fn name(&self) -> Option<String> {
		unsafe {
			let res = sys::ddca_rc_name(self.status);
			if res.is_null() {
				None
			} else {
				c_str(res).ok().map(From::from)
			}
		}
	}

	pub fn desc(&self) -> Option<String> {
		unsafe {
			let res = sys::ddca_rc_desc(self.status);
			if res.is_null() {
				None
			} else {
				c_str(res).ok().map(From::from)
			}
		}
	}
}

impl fmt::Debug for Status {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DDCA_Status")
			.field("status", &self.status)
			.field("name", &self.name())
			.field("desc", &self.desc())
			.finish()
	}
}

impl fmt::Display for Status {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Some(err) = self.desc().or_else(|| self.name()) {
			f.write_str(&err)
		} else {
			f.write_str("unknown DDCA_Status")
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error {
	status: Status,
}

impl Error {
	pub fn new(status: Status) -> Self {
		Error {
			status: status,
		}
	}

	pub fn from_status(status: sys::DDCA_Status) -> Result<Status, Self> {
		let status = Status::new(status);
		if status.success() {
			Ok(status)
		} else {
			Err(Self::new(status))
		}
	}

	pub fn status(&self) -> Status {
		self.status
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(&self.status, f)
	}
}

impl fmt::Debug for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(&self.status, f)
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		"ddcutil error"
	}
}
