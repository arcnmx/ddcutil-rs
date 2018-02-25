extern crate ddcutil as ddc;

fn main() {
	let displays = ddc::DisplayInfo::enumerate().expect("DisplayInfo::enumerate");
	let display = if let Some(d) = displays.into_iter().next() {
		d
	} else {
		println!("no displays found");
		return
	};

	let handle = display.open().expect("display open");
	let value = handle.vcp_get_value(0x60).expect("vcp_get_value");
	println!("VCP 0x60 = {:?}", value);
}
