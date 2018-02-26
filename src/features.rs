use std::{mem, fmt};
use std::ffi::CStr;
use std::collections::HashMap;
use libc::c_char;
use {sys, Error};

pub type FeatureCode = u8;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Value {
    pub mh: u8,
    pub ml: u8,
    pub sh: u8,
    pub sl: u8,
}

impl Value {
    pub fn from_raw(raw: &sys::DDCA_Non_Table_Value) -> Self {
        Value {
            mh: raw.mh,
            ml: raw.ml,
            sh: raw.sh,
            sl: raw.sl,
        }
    }

    pub fn value(&self) -> u16 {
        ((self.sh as u16) << 8) | self.sl as u16
    }

    pub fn maximum(&self) -> u16 {
        ((self.mh as u16) << 8) | self.ml as u16
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MccsVersion {
    pub major: u8,
    pub minor: u8,
}

impl MccsVersion {
    pub fn from_raw(raw: sys::DDCA_MCCS_Version_Spec) -> Self {
        MccsVersion {
            major: raw.major,
            minor: raw.minor,
        }
    }

    pub fn from_id(raw: sys::DDCA_MCCS_Version_Id) -> Result<Self, ()> {
        match raw {
            sys::DDCA_V10 => Ok(MccsVersion { major: 1, minor: 0 }),
            sys::DDCA_V20 => Ok(MccsVersion { major: 2, minor: 0 }),
            sys::DDCA_V21 => Ok(MccsVersion { major: 2, minor: 1 }),
            sys::DDCA_V30 => Ok(MccsVersion { major: 3, minor: 0 }),
            sys::DDCA_V22 => Ok(MccsVersion { major: 2, minor: 2 }),
            _ => Err(()),
        }
    }

    pub fn id(&self) -> Result<sys::DDCA_MCCS_Version_Id, ()> {
        match *self {
            MccsVersion { major: 1, minor: 0 } => Ok(sys::DDCA_V10),
            MccsVersion { major: 2, minor: 0 } => Ok(sys::DDCA_V20),
            MccsVersion { major: 2, minor: 1 } => Ok(sys::DDCA_V21),
            MccsVersion { major: 3, minor: 0 } => Ok(sys::DDCA_V30),
            MccsVersion { major: 2, minor: 2 } => Ok(sys::DDCA_V22),
            _ => Err(()),
        }
    }
}

impl fmt::Display for MccsVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl fmt::Debug for MccsVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capabilities {
    pub version: MccsVersion,
    pub features: HashMap<FeatureCode, Vec<u8>>,
}

impl Capabilities {
    pub unsafe fn from_raw(raw: &sys::DDCA_Capabilities) -> Self {
        Capabilities {
            version: MccsVersion::from_raw(raw.version_spec),
            features: raw.vcp_codes().iter().map(|raw| (raw.feature_code, raw.values().to_owned())).collect(),
        }
    }

    pub fn from_cstr(caps: &CStr) -> ::Result<Self> {
        unsafe {
            let mut res = mem::uninitialized();
            Error::from_status(sys::ddca_parse_capabilities_string(
                caps.as_ptr() as *mut _, &mut res
            ))?;
            let caps = Capabilities::from_raw(&*res);
            sys::ddca_free_parsed_capabilities(res);
            Ok(caps)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureInfo {
    pub name: String,
    pub description: String,
    pub value_names: HashMap<u8, String>,
    pub flags: FeatureFlags,
}

impl FeatureInfo {
    pub fn from_code(code: FeatureCode, version: MccsVersion) -> ::Result<Self> {
        unsafe {
            let mut res = mem::uninitialized();
            Error::from_status(sys::ddca_get_feature_info_by_vcp_version(
                code, version.id().unwrap_or(sys::DDCA_VANY), &mut res
            ))?;
            let features = Self::from_raw(&*res);
            Error::from_status(sys::ddca_free_feature_info(res))?;
            Ok(features)
        }
    }

    pub unsafe fn from_raw(raw: &sys::DDCA_Version_Feature_Info) -> Self {
        unsafe fn from_ptr(ptr: *const c_char) -> String {
            if ptr.is_null() {
                Default::default()
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        }

        FeatureInfo {
            name: from_ptr(raw.feature_name),
            description: from_ptr(raw.desc),
            value_names: raw.sl_values().iter().map(|v| (
                v.value_code,
                from_ptr(v.value_name),
            )).collect(),
            flags: FeatureFlags::from_bits_truncate(raw.feature_flags),
        }
    }
}

bitflags! {
    pub struct FeatureFlags: u16 {
        /// Read only feature
        const RO = sys::DDCA_RO;
        /// Write only feature
        const WO = sys::DDCA_WO;
        /// Feature is both readable and writable
        const RW = sys::DDCA_RW;

        /// Normal continuous feature
        const STD_CONT = sys::DDCA_STD_CONT;
        /// Continuous feature with special interpretation
        const COMPLEX_CONT = sys::DDCA_COMPLEX_CONT;
        /// Non-continuous feature, having a defined list of values in byte SL
        const SIMPLE_NC = sys::DDCA_SIMPLE_NC;
        /// Non-continuous feature, having a complex interpretation using one or more of SL, SH, ML, MH
        const COMPLEX_NC = sys::DDCA_COMPLEX_NC;

        /// Used internally for write-only non-continuous features
        const WO_NC = sys::DDCA_WO_NC;
        /// Normal RW table type feature
        const NORMAL_TABLE = sys::DDCA_NORMAL_TABLE;
        /// Write only table feature
        const WO_TABLE = sys::DDCA_WO_TABLE;

        /// Feature is deprecated in the specified VCP version
        const DEPRECATED = sys::DDCA_DEPRECATED;

        /// DDCA_Global_Feature_Flags
        const SYNTHETIC = sys::DDCA_SYNTHETIC;
    }
}

impl FeatureFlags {
    /// Feature is either RW or RO
    pub fn is_readable(&self) -> bool {
        self.bits & sys::DDCA_READABLE != 0
    }

    /// Feature is either RW or WO
    pub fn is_writable(&self) -> bool {
        self.bits & sys::DDCA_WRITABLE != 0
    }

    /// Continuous feature, of any subtype
    pub fn is_cont(&self) -> bool {
        self.bits & sys::DDCA_CONT != 0
    }

    /// Non-continuous feature of any subtype
    pub fn is_nc(&self) -> bool {
        self.bits & sys::DDCA_NC != 0
    }

    /// Non-table feature of any type
    pub fn is_non_table(&self) -> bool {
        self.bits & sys::DDCA_NON_TABLE != 0
    }

    /// Table type feature, of any subtype
    pub fn is_table(&self) -> bool {
        self.bits & sys::DDCA_TABLE != 0
    }

    /// unused
    pub fn is_known(&self) -> bool {
        self.bits & sys::DDCA_KNOWN != 0
    }
}
