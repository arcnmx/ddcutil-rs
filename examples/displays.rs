extern crate ddcutil as ddc;

fn main() {
	for display in ddc::DisplayInfo::enumerate() {
		println!("{:#?}", display);
	}
}
