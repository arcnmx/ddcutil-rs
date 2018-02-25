use std::{mem, fmt};
use std::borrow::Cow;
use std::ffi::CStr;
use libc::{self, c_int, c_char};
use {sys, Error, Status};

pub type VcpFeatureCode = u8;

#[derive(Clone)]
pub struct DisplayInfo {
	handle: sys::DDCA_Display_Ref,
	display_number: i32,
	manufacturer_id: Vec<u8>,
	model_name: Vec<u8>,
	serial_number: Vec<u8>,
	edid: Box<[u8]>,
	path: DisplayPath,
}

impl DisplayInfo {
	pub fn open(&self) -> ::Result<Display> {
		unsafe {
			let mut handle = mem::uninitialized();
			let status = sys::ddca_open_display(self.handle, &mut handle as *mut _);
			Error::from_status(status).map(|_| Display::from_raw(handle))
		}
	}

	pub unsafe fn from_raw(raw: &sys::DDCA_Display_Info) -> Self {
		fn from_ptr(ptr: *const c_char) -> Vec<u8> {
			if ptr.is_null() {
				Default::default()
			} else {
				unsafe {
					CStr::from_ptr(ptr).to_bytes().to_owned()
				}
			}
		}

		DisplayInfo {
			handle: raw.dref,
			display_number: raw.dispno,
			manufacturer_id: from_ptr(raw.mfg_id),
			model_name: from_ptr(raw.model_name),
			serial_number: from_ptr(raw.sn),
			edid: raw.edid_bytes().to_owned().into(),
			path: DisplayPath::from_raw(&raw.path, raw.usb_bus, raw.usb_device)
				.unwrap_or_else(|_| DisplayPath::Usb {
					// stupid fallback, but should never happen...
					bus_number: raw.usb_bus,
					device_number: raw.usb_device,
					hiddev_device_number: -1,
				}),
		}
	}

	pub fn enumerate() -> ::Result<DisplayInfoList> {
		unsafe {
			let res = sys::ddca_get_display_info_list();
			if res.is_null() {
				Err(Error::new(Status::new(libc::EINVAL)))
			} else {
				Ok(DisplayInfoList::from_raw(res))
			}
		}
	}

	pub fn raw(&self) -> sys::DDCA_Display_Ref {
		self.handle
	}

	pub fn display_number(&self) -> i32 {
		self.display_number
	}

	pub fn manufacturer_id(&self) -> Cow<str> {
		String::from_utf8_lossy(&self.manufacturer_id)
	}

	pub fn manufacturer_id_bytes(&self) -> &[u8] {
		&self.manufacturer_id
	}

	pub fn model_name(&self) -> Cow<str> {
		String::from_utf8_lossy(&self.model_name)
	}

	pub fn model_name_bytes(&self) -> &[u8] {
		&self.model_name
	}

	pub fn serial_number(&self) -> Cow<str> {
		String::from_utf8_lossy(&self.serial_number)
	}

	pub fn serial_number_bytes(&self) -> &[u8] {
		&self.serial_number
	}

	pub fn edid(&self) -> &[u8] {
		&self.edid
	}

	pub fn path(&self) -> DisplayPath {
		self.path
	}
}

impl fmt::Debug for DisplayInfo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("DisplayInfo")
			.field("display_number", &self.display_number)
			.field("manufacturer_id", &self.manufacturer_id())
			.field("model_name", &self.model_name())
			.field("serial_number", &self.serial_number())
			.field("path", &self.path())
			.finish()
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplayPath {
	I2c {
		bus_number: i32,
	},
	Usb {
		bus_number: i32,
		device_number: i32,
		hiddev_device_number: i32,
	},
	Adl {
		adapter_index: i32,
		display_index: i32,
	},
}

impl DisplayPath {
	pub fn from_raw(path: &sys::DDCA_IO_Path, usb_bus: c_int, usb_device: c_int) -> Result<Self, ()> {
		match path.io_mode {
			sys::DDCA_IO_DEVI2C => Ok(DisplayPath::I2c {
				bus_number: path.i2c_busno(),
			}),
			sys::DDCA_IO_USB => Ok(DisplayPath::Usb {
				bus_number: usb_bus as _,
				device_number: usb_device as _,
				hiddev_device_number: path.hiddev_devno(),
			}),
			sys::DDCA_IO_ADL => Ok(DisplayPath::Adl {
				adapter_index: path.adlno().iAdapterIndex,
				display_index: path.adlno().iDisplayIndex,
			}),
			_ => Err(()),
		}
	}
}

pub struct DisplayInfoList {
	handle: *mut sys::DDCA_Display_Info_List,
}

impl fmt::Debug for DisplayInfoList {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_list().entries(self.into_iter()).finish()
	}
}

impl DisplayInfoList {
	pub unsafe fn from_raw(handle: *mut sys::DDCA_Display_Info_List) -> Self {
		DisplayInfoList {
			handle: handle,
		}
	}

	pub fn raw(&self) -> &sys::DDCA_Display_Info_List {
		unsafe { &*self.handle }
	}

	pub fn len(&self) -> usize {
		self.raw().info().len() as usize
	}

	pub fn get(&self, index: usize) -> DisplayInfo {
		unsafe {
			DisplayInfo::from_raw(&self.raw().info()[index])
		}
	}
}

impl<'a> IntoIterator for &'a DisplayInfoList {
	type Item = DisplayInfo;
	type IntoIter = DisplayInfoIter<'a>;

	fn into_iter(self) -> Self::IntoIter {
		DisplayInfoIter {
			list: self,
			index: 0,
		}
	}
}

impl Drop for DisplayInfoList {
	fn drop(&mut self) {
		unsafe {
			sys::ddca_free_display_info_list(self.handle)
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct DisplayInfoIter<'a> {
	list: &'a DisplayInfoList,
	index: usize,
}

impl<'a> Iterator for DisplayInfoIter<'a> {
	type Item = DisplayInfo;

	fn next(&mut self) -> Option<Self::Item> {
		if self.index < self.list.len() {
			let index = self.index;
			self.index += 1;
			Some(self.list.get(index))
		} else {
			None
		}
	}
}

#[derive(Debug)]
pub struct Display {
	handle: sys::DDCA_Display_Handle,
}

impl Display {
	pub unsafe fn from_raw(handle: sys::DDCA_Display_Handle) -> Self {
		Display {
			handle: handle,
		}
	}

	pub fn vcp_set_simple(&self, code: VcpFeatureCode, value: u8) -> ::Result<()> {
		unsafe {
			Error::from_status(sys::ddca_set_simple_nc_vcp_value(
				self.handle, code as _, value
			)).map(drop)
		}
	}

	pub fn vcp_set_raw(&self, code: VcpFeatureCode, value: u16) -> ::Result<()> {
		unsafe {
			Error::from_status(sys::ddca_set_raw_vcp_value(
				self.handle, code as _, (value >> 8) as u8, value as u8
			)).map(drop)
		}
	}

	pub fn vcp_set_continuous(&self, code: VcpFeatureCode, value: i32) -> ::Result<()> {
		unsafe {
			Error::from_status(sys::ddca_set_continuous_vcp_value(
				self.handle, code as _, value
			)).map(drop)
		}
	}

	pub fn vcp_get_value(&self, code: VcpFeatureCode) -> ::Result<VcpValue> {
		unsafe {
			let mut raw = mem::uninitialized();
			Error::from_status(sys::ddca_get_any_vcp_value(
				self.handle, code as _, sys::DDCA_NON_TABLE_VCP_VALUE_PARM, &mut raw
			)).map(drop)?;
			let raw = &mut *raw;
			if raw.value_type != sys::DDCA_NON_TABLE_VCP_VALUE || raw.opcode != code {
				libc::free(raw as *mut _ as *mut _);
				return Err(Error::new(Status::new(libc::EINVAL)))
			}
			let value = VcpValue::from_raw(raw.c_nc());
			libc::free(raw as *mut _ as *mut _);
			Ok(value)
		}
	}

	pub fn vcp_get_table(&self, code: VcpFeatureCode) -> ::Result<Vec<u8>> {
		unsafe {
			let mut raw = mem::uninitialized();
			Error::from_status(sys::ddca_get_any_vcp_value(
				self.handle, code as _, sys::DDCA_TABLE_VCP_VALUE_PARM, &mut raw
			)).map(drop)?;
			let raw = &mut *raw;
			if raw.value_type != sys::DDCA_TABLE_VCP_VALUE || raw.opcode != code {
				libc::free(raw as *mut _ as *mut _);
				return Err(Error::new(Status::new(libc::EINVAL)))
			}
			let value = raw.t().bytes().to_owned();
			libc::free(raw.t().bytes as *mut _);
			libc::free(raw as *mut _ as *mut _);
			Ok(value)
		}
	}

	pub fn raw(&self) -> sys::DDCA_Display_Handle {
		self.handle
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VcpValue {
	pub value: u16,
	pub maximum: u16,
}

impl VcpValue {
	pub fn from_raw(raw: &sys::DDCA_Non_Table_Value) -> Self {
		VcpValue {
			value: raw.value(),
			maximum: raw.maximum(),
		}
	}
}

impl Drop for Display {
	fn drop(&mut self) {
		unsafe {
			sys::ddca_close_display(self.handle);
		}
	}
}

#[test]
fn test_displays() {
	for display in &DisplayInfo::enumerate().unwrap() {
		drop(display.open());
	}
}
