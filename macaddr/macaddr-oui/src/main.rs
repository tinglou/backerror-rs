//! MAC 地址 OUI 查询命令行工具
//!
//! 用法：macaddr-oui <MAC 地址>
//!
//! 示例：
//!   macaddr-oui 00:55:DA:0A:BB:CC
//!   macaddr-oui 00-55-DA-0A-BB-CC

use macaddr_oui::{MacAddress, OUI_DB, OuiDb};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <MAC address>", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} 00:55:DA:0A:BB:CC", args[0]);
        eprintln!("  {} 00-55-DA-0A-BB-CC", args[0]);
        eprintln!();
        eprintln!("MAC address format:");
        eprintln!("  - Colon-separated: 00:11:22:33:44:55");
        eprintln!("  - Dash-separated:  00-11-22-33-44-55");
        process::exit(1);
    }

    let mac_str = &args[1];

    // 解析 MAC 地址
    let mac: MacAddress = match mac_str.parse() {
        Ok(mac) => mac,
        Err(e) => {
            eprintln!("Error: Invalid MAC address '{}'", mac_str);
            eprintln!("Details: {:?}", e);
            process::exit(1);
        }
    };

    // 输出 MAC 地址信息
    println!("MAC Address: {}", mac);
    println!("Formatted:   {}", mac);
    println!("Octets:      {:02X}-{:02X}-{:02X}-{:02X}-{:02X}-{:02X}",
        mac.octets()[0], mac.octets()[1], mac.octets()[2],
        mac.octets()[3], mac.octets()[4], mac.octets()[5]
    );
    println!();

    // 地址类型
    println!("Address Type:");
    println!("  Unicast:     {}", if mac.is_unicast() { "Yes" } else { "No" });
    println!("  Multicast:   {}", if mac.is_multicast() { "Yes" } else { "No" });
    println!("  Broadcast:   {}", if mac.is_broadcast() { "Yes" } else { "No" });
    println!("  Universal:   {}", if mac.is_universal() { "Yes" } else { "No" });
    println!("  Local:       {}", if mac.is_local() { "Yes" } else { "No" });
    println!();

    // OUI 查询
    match OUI_DB.lookup(mac) {
        Some(org_name) => {
            println!("Organization: {}", org_name);
            if OuiDb::is_virtual_nic(org_name) {
                println!("  ⚠️  This is a virtual NIC");
            }
        }
        None => {
            println!("Organization: Unknown (not found in OUI database)");
        }
    }
}
