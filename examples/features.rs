extern crate ddcutil as ddc;

fn main() {
    let displays = ddc::DisplayInfo::enumerate().expect("DisplayInfo::enumerate");
    for display in &displays {
        println!("{:#?}", display);
        let handle = display.open().expect("DisplayInfo::open");
        let caps = handle.capabilities().expect("Display::capabilities()");
        for (code, cap) in &caps.features {
            let info = ddc::FeatureInfo::from_code(*code, caps.version);
            if let Ok(info) = info {
                if info.flags.is_readable() {
                    let value = if info.flags.is_non_table() {
                        handle.vcp_get_value(*code).map(|v| format!("{} / {}", v.value(), v.maximum()))
                    } else {
                        handle.vcp_get_table(*code).map(|t| format!("{:?}", t))
                    }.unwrap_or_else(|e| format!("ERR {}", e));
                    println!("VCP 0x{:02x} = {} - {} [{:?}] {}", code, value, info.name, info.flags, info.description);
                } else {
                    println!("VCP 0x{:02x} - {} [{:?}] {}", code, info.name, info.flags, info.description);
                }

                if !cap.is_empty() {
                    let values: Vec<_> = cap.iter().map(|v| (v, info.value_names.get(v))).map(|(v, name)|
                        if let Some(name) = name {
                            format!("{} {}", v, name)
                        } else {
                            v.to_string()
                        }
                    ).collect();
                    println!("  {}", values.join(", "));
                }
            } else {
                let value = handle.vcp_get_value(*code).map(|v| format!("{} / {}", v.value(), v.maximum()))
                    .unwrap_or_else(|e| format!("ERR {}", e));
                println!("VCP 0x{:02x} = {} - Unknown", code, value);
            }
        }
    }
}
