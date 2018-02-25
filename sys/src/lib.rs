#![doc(html_root_url = "http://arcnmx.github.io/ddcutil-rs/")]
#![allow(non_camel_case_types, non_snake_case)]

extern crate libc;

use std::fmt;
use std::slice::from_raw_parts;
use std::ffi::{CString, CStr};
use libc::{c_char, c_int, c_void};

#[link(name = "ddcutil")]
extern {
    pub fn ddca_ddcutil_version() -> DDCA_Ddcutil_Version_Spec;
	pub fn ddca_ddcutil_version_string() -> *const c_char;

	pub fn ddca_rc_name(status_code: DDCA_Status) -> *mut c_char;
	pub fn ddca_rc_desc(status_code: DDCA_Status) -> *mut c_char;

	pub fn ddca_mccs_version_id_name(version_id: DDCA_MCCS_Version_Id) -> *mut c_char;
	pub fn ddca_mccs_version_id_desc(version_id: DDCA_MCCS_Version_Id) -> *mut c_char;

	pub fn ddca_get_display_info_list() -> *mut DDCA_Display_Info_List;
	pub fn ddca_free_display_info_list(dlist: *mut DDCA_Display_Info_List);
	pub fn ddca_report_display_info(dinfo: *mut DDCA_Display_Info, depth: c_int);
	pub fn ddca_report_display_info_list(dlist: *mut DDCA_Display_Info_List, depth: c_int);
	pub fn ddca_report_active_displays(depth: c_int) -> c_int;

	pub fn ddca_open_display(
		ddca_dref: DDCA_Display_Ref,
		p_ddca_dh: *mut DDCA_Display_Handle,
	) -> DDCA_Status;
	pub fn ddca_close_display(ddca_dh: DDCA_Display_Handle) -> DDCA_Status;

	pub fn ddca_get_any_vcp_value(
		ddca_dh: DDCA_Display_Handle,
		feature_code: DDCA_Vcp_Feature_Code,
		value_type: DDCA_Vcp_Value_Type_Parm,
		pvalrec: *mut *mut DDCA_Any_Vcp_Value,
	) -> DDCA_Status;

	pub fn ddca_set_continuous_vcp_value(
		ddca_dh: DDCA_Display_Handle,
		feature_code: DDCA_Vcp_Feature_Code,
		new_value: c_int,
	) -> DDCA_Status;

	pub fn ddca_set_simple_nc_vcp_value(
		ddca_dh: DDCA_Display_Handle,
		feature_code: DDCA_Vcp_Feature_Code,
		new_value: u8,
	) -> DDCA_Status;

	pub fn ddca_set_raw_vcp_value(
		ddca_dh: DDCA_Display_Handle,
		feature_code: DDCA_Vcp_Feature_Code,
		hi_byte: u8,
		lo_byte: u8,
	) -> DDCA_Status;
}

pub type DDCA_Status = c_int;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DDCA_Ddcutil_Version_Spec {
	pub major: u8,
	pub minor: u8,
	pub micro: u8,
}

pub type DDCA_Display_Identifier = *mut c_void;

pub type DDCA_Display_Ref = *mut c_void;

pub type DDCA_Display_Handle = *mut c_void;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DDCA_Adlno {
	pub iAdapterIndex: c_int,
	pub iDisplayIndex: c_int,
}

pub type DDCA_IO_Mode = c_int;
pub const DDCA_IO_DEVI2C: DDCA_IO_Mode = 0;
pub const DDCA_IO_ADL: DDCA_IO_Mode = 1;
pub const DDCA_IO_USB: DDCA_IO_Mode = 2;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DDCA_IO_Path {
	pub io_mode: DDCA_IO_Mode,
	// union { i2c_busno: c_int, adlno: DDCA_Adlno: adlno, hiddev_devno: c_int }
	pub _union: DDCA_Adlno,
}

impl DDCA_IO_Path {
	pub fn i2c_busno(&self) -> c_int {
		self._union.iAdapterIndex
	}

	pub fn hiddev_devno(&self) -> c_int {
		self._union.iAdapterIndex
	}

	pub fn adlno(&self) -> &DDCA_Adlno {
		&self._union
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DDCA_Display_Info {
	pub marker: [c_char; 4],
	pub dispno: c_int,
	pub path: DDCA_IO_Path,
	pub usb_bus: c_int,
	pub usb_device: c_int,
	pub mfg_id: *const c_char,
	pub model_name: *const c_char,
	pub sn: *const c_char,
	pub edid_bytes: *const u8,
	pub dref: DDCA_Display_Ref,
}

impl DDCA_Display_Info {
	pub fn mfg_id(&self) -> &CStr {
		unsafe {
			CStr::from_ptr(self.mfg_id)
		}
	}

	pub fn model_name(&self) -> &CStr {
		unsafe {
			CStr::from_ptr(self.model_name)
		}
	}

	pub fn sn(&self) -> &CStr {
		unsafe {
			CStr::from_ptr(self.sn)
		}
	}

	pub fn edid_bytes(&self) -> &[u8] {
		unsafe {
			from_raw_parts(self.edid_bytes, 0x80)
		}
	}
}

impl fmt::Debug for DDCA_Display_Info {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		unsafe {
			f.debug_struct("DDCA_Display_Info")
				.field("marker", &CString::new(from_raw_parts(self.marker.as_ptr() as *const u8, self.marker.len())))
				.field("dispno", &self.dispno)
				.field("path", &self.path)
				.field("usb_bus", &self.usb_bus)
				.field("usb_device", &self.usb_device)
				.field("mfg_id", &self.mfg_id())
				.field("model_name", &self.model_name())
				.field("sn", &self.sn())
				.field("edid_bytes", &self.edid_bytes())
				.field("dref", &self.dref)
				.finish()
		}
	}
}

#[repr(C)]
pub struct DDCA_Display_Info_List {
	pub ct: c_int,
	pub info: [DDCA_Display_Info; 0],
}

impl DDCA_Display_Info_List {
	pub fn info(&self) -> &[DDCA_Display_Info] {
		unsafe {
			from_raw_parts(self.info.as_ptr(), self.ct as usize)
		}
	}
}

impl fmt::Debug for DDCA_Display_Info_List {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self.info(), f)
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DDCA_MCCS_Version_Spec {
	pub major: u8,
	pub minor: u8,
}

pub type DDCA_MCCS_Version_Id = c_int;
pub const DDCA_VNONE: DDCA_MCCS_Version_Id = 0;
pub const DDCA_V10: DDCA_MCCS_Version_Id = 1;
pub const DDCA_V20: DDCA_MCCS_Version_Id = 2;
pub const DDCA_V21: DDCA_MCCS_Version_Id = 4;
pub const DDCA_V30: DDCA_MCCS_Version_Id = 8;
pub const DDCA_V22: DDCA_MCCS_Version_Id = 16;
pub const DDCA_VANY: DDCA_MCCS_Version_Id = DDCA_VNONE;
pub const DDCA_VUNK: DDCA_MCCS_Version_Id = DDCA_VNONE;

pub type DDCA_Vcp_Feature_Code = u8;

pub type DDCA_Vcp_Value_Type = c_int;
pub const DDCA_NON_TABLE_VCP_VALUE: DDCA_Vcp_Value_Type = 1;
pub const DDCA_TABLE_VCP_VALUE: DDCA_Vcp_Value_Type = 2;

pub type DDCA_Vcp_Value_Type_Parm = c_int;
pub const DDCA_UNSET_VCP_VALUE_TYPE_PARM: DDCA_Vcp_Value_Type_Parm = 0;
pub const DDCA_NON_TABLE_VCP_VALUE_PARM: DDCA_Vcp_Value_Type_Parm = 1;
pub const DDCA_TABLE_VCP_VALUE_PARM: DDCA_Vcp_Value_Type_Parm = 2;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DDCA_Non_Table_Value {
	pub mh: u8,
	pub ml: u8,
	pub sh: u8,
	pub sl: u8,
}

impl DDCA_Non_Table_Value {
	pub fn value(&self) -> u16 {
		((self.sh as u16) << 8) | self.sl as u16
	}

	pub fn maximum(&self) -> u16 {
		((self.mh as u16) << 8) | self.ml as u16
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct DDCA_Table_Value {
	pub bytect: u16,
	pub bytes: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _DDCA_Table_Value {
	pub bytes: *mut u8,
	pub bytect: u16,
}

impl _DDCA_Table_Value {
	pub fn bytes(&self) -> &[u8] {
		unsafe {
			from_raw_parts(self.bytes as *const _, self.bytect as usize)
		}
	}
}

impl fmt::Debug for _DDCA_Table_Value {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		fmt::Debug::fmt(self.bytes(), f)
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct DDCA_Any_Vcp_Value {
	pub opcode: DDCA_Vcp_Feature_Code,
	pub value_type: DDCA_Vcp_Value_Type,
	// union { _DDCA_Table_Value, DDCA_Non_Table_Value }
	pub _val_union: _DDCA_Table_Value,
}

impl DDCA_Any_Vcp_Value {
	pub unsafe fn c_nc(&self) -> &DDCA_Non_Table_Value {
		unsafe {
			::std::mem::transmute(self.t())
		}
	}

	pub unsafe fn t(&self) -> &_DDCA_Table_Value {
		&self._val_union
	}
}
