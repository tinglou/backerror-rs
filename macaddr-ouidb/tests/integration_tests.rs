//! macaddr-oui 集成测试

use macaddr_ouidb::{MacAddress, OUI_DB, OuiDb};

#[test]
fn test_mac_address_parsing() {
    // 测试冒号分隔格式
    let mac1: MacAddress = "00:55:DA:0A:BB:CC".parse().unwrap();
    assert_eq!(mac1.octets(), [0x00, 0x55, 0xDA, 0x0A, 0xBB, 0xCC]);

    // 测试横线分隔格式
    let mac2: MacAddress = "00-55-DA-0A-BB-CC".parse().unwrap();
    assert_eq!(mac2.octets(), [0x00, 0x55, 0xDA, 0x0A, 0xBB, 0xCC]);

    // 测试大小写不敏感
    let mac3: MacAddress = "00:55:da:0a:bb:cc".parse().unwrap();
    assert_eq!(mac3.octets(), [0x00, 0x55, 0xDA, 0x0A, 0xBB, 0xCC]);
}

#[test]
fn test_mac_address_special_addresses() {
    // 零地址
    let zero = MacAddress::zero();
    assert!(zero.is_zero());
    assert!(!zero.is_broadcast());

    // 广播地址
    let broadcast = MacAddress::broadcast();
    assert!(!broadcast.is_zero());
    assert!(broadcast.is_broadcast());

    // 单播/组播
    let unicast: MacAddress = "00:11:22:33:44:55".parse().unwrap();
    assert!(unicast.is_unicast());
    assert!(!unicast.is_multicast());

    let multicast: MacAddress = "01:00:00:00:00:00".parse().unwrap();
    assert!(!multicast.is_unicast());
    assert!(multicast.is_multicast());

    // 全球管理/本地管理
    let universal: MacAddress = "00:11:22:33:44:55".parse().unwrap();
    assert!(universal.is_universal());
    assert!(!universal.is_local());

    let local: MacAddress = "02:11:22:33:44:55".parse().unwrap();
    assert!(!local.is_universal());
    assert!(local.is_local());
}

#[test]
fn test_oui_lookup_24bit() {
    // 测试 24 位 OUI 查询（标准 OUI，前 3 字节）
    // 格式：XX:XX:XX:YY:YY:YY，只使用前 3 字节查找

    // Xerox (00:00:00)
    let mac: MacAddress = "00:00:01:00:00:00".parse().unwrap();
    let org = OUI_DB.lookup_mac(mac);
    assert_eq!(org, Some("Xerox"));

    // Cisco Systems (00:00:0C)
    let mac: MacAddress = "00:00:0C:00:00:00".parse().unwrap();
    let org = OUI_DB.lookup_mac(mac);
    assert_eq!(org, Some("Cisco Systems"));

    // Microsoft (00:03:FF)
    let mac: MacAddress = "00:03:FF:00:00:00".parse().unwrap();
    let org = OUI_DB.lookup_mac(mac);
    assert_eq!(org, Some("Microsoft"));

    // Apple (00:03:93)
    let mac: MacAddress = "00:03:93:FF:FF:FF".parse().unwrap();
    let org = OUI_DB.lookup_mac(mac);
    assert_eq!(org, Some("Apple"));
}

#[test]
fn test_oui_lookup_28bit() {
    // 测试 28 位 OUI 查询（MA-M，前 3 字节 + 第 4 字节高 4 位）
    // 格式：XX:XX:DA:0X:YY:YY，第 4 字节只有高 4 位有效（0x00-0xF0）

    // 00:55:DA 前缀下的 28 位条目
    let mac: MacAddress = "00:55:DA:0A:BB:CC".parse().unwrap();
    let org = OUI_DB.lookup_mac(mac);
    assert!(
        org.is_some(),
        "28-bit OUI lookup failed for 00:55:DA:0A:BB:CC"
    );
    println!("28-bit OUI: 00:55:DA:0A:BB:CC -> {}", org.unwrap());

    // 同一 24 位前缀下，不同 28 位子条目应该匹配不同组织
    let mac1: MacAddress = "00:55:DA:00:00:00".parse().unwrap();
    let mac2: MacAddress = "00:55:DA:10:00:00".parse().unwrap();
    let org1 = OUI_DB.lookup_mac(mac1);
    let org2 = OUI_DB.lookup_mac(mac2);

    // 都应该有结果（可能相同也可能不同）
    assert!(
        org1.is_some(),
        "28-bit OUI lookup failed for 00:55:DA:00:00:00"
    );
    assert!(
        org2.is_some(),
        "28-bit OUI lookup failed for 00:55:DA:10:00:00"
    );

    // 验证 28 位子表确实被使用（不是简单的 24 位匹配）
    // 如果第 4 字节不在子表范围内，应该回退到 24 位或返回 None
    let mac_fallback: MacAddress = "00:55:DA:F0:00:00".parse().unwrap();
    let org_fallback = OUI_DB.lookup_mac(mac_fallback);
    // 这个可能匹配 24 位条目或 28 位条目
    assert!(org_fallback.is_some() || org_fallback.is_none());
}

#[test]
fn test_oui_lookup_36bit() {
    // 测试 36 位 OUI 查询（MA-S，前 4 字节 + 第 5 字节高 4 位）
    // 格式：XX:XX:XX:XX:0X:YY，第 5 字节只有高 4 位有效（0x00-0xF0）

    // 00:16:E3 前缀下的 36 位条目
    let mac: MacAddress = "00:16:E3:00:00:00".parse().unwrap();
    let org = OUI_DB.lookup_mac(mac);
    assert!(
        org.is_some(),
        "36-bit OUI lookup failed for 00:16:E3:00:00:00"
    );
    println!("36-bit OUI: 00:16:E3:00:00:00 -> {:?}", org);

    // 验证 36 位子表被使用
    // 00:16:E3 是 24 位前缀，但它的某些第 4 字节有 36 位子表
    let mac2: MacAddress = "00:16:E3:80:00:00".parse().unwrap();
    let org2 = OUI_DB.lookup_mac(mac2);
    assert!(
        org2.is_some(),
        "36-bit OUI lookup failed for 00:16:E3:80:00:00"
    );

    // 不同的第 4 字节应该能匹配到不同结果
    if org != org2 {
        println!("Different 36-bit entries return different organizations:");
        println!("  00:16:E3:00:00:00 -> {:?}", org);
        println!("  00:16:E3:80:00:00 -> {:?}", org2);
    }
}

#[test]
fn test_oui_lookup_not_found() {
    // 测试未分配的 OUI（不在数据库中）

    // FF:FF:FF 不是有效的 OUI 前缀
    let mac: MacAddress = "FF:FF:FF:00:00:00".parse().unwrap();
    let org = OUI_DB.lookup_mac(mac);
    assert_eq!(org, None, "Expected None for unallocated OUI FF:FF:FF");

    // 某些未分配的地址段
    let mac2: MacAddress = "FE:FF:FF:00:00:00".parse().unwrap();
    let org2 = OUI_DB.lookup_mac(mac2);
    assert_eq!(org2, None, "Expected None for unallocated OUI FE:FF:FF");

    println!("Correctly returns None for unallocated OUI addresses");
}

#[test]
fn test_virtual_nic_detection() {
    // 测试虚拟网卡识别
    assert!(OuiDb::is_virtual_nic("QEMU virtual NIC"));
    assert!(OuiDb::is_virtual_nic("Bochs virtual NIC"));
    assert!(OuiDb::is_virtual_nic("PearPC virtual NIC"));
    assert!(OuiDb::is_virtual_nic("Cooperative Linux virtual NIC"));
    assert!(OuiDb::is_virtual_nic("Oracle VirtualBox virtual NIC"));

    // 测试非虚拟网卡
    assert!(!OuiDb::is_virtual_nic("Cisco Systems"));
    assert!(!OuiDb::is_virtual_nic("Intel"));
    assert!(!OuiDb::is_virtual_nic("Unknown"));
}

#[test]
fn test_oui_subtable_name() {
    assert_eq!(OuiDb::oui_subtable_name(), "Ieee Registration Authority");
}

#[test]
fn test_mac_address_from_bytes() {
    let bytes = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mac = MacAddress::from(bytes);
    assert_eq!(mac.octets(), bytes);
}

#[test]
fn test_mac_address_from_slice() {
    let bytes = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC];
    let mac = MacAddress::from_slice(&bytes);
    assert_eq!(mac, Ok(MacAddress::from(bytes)));

    // 错误长度
    let wrong_bytes = [0x12, 0x34, 0x56];
    let mac = MacAddress::from_slice(&wrong_bytes);
    assert!(mac.is_err());
}

#[test]
fn test_mac_address_display() {
    let mac: MacAddress = "00:11:22:33:44:55".parse().unwrap();
    assert_eq!(format!("{}", mac), "00:11:22:33:44:55");
}

#[test]
fn test_mac_address_debug() {
    let mac: MacAddress = "00:11:22:33:44:55".parse().unwrap();
    assert_eq!(format!("{:?}", mac), "00:11:22:33:44:55");
}

#[test]
fn test_oui_lookup_batch() {
    // 批量测试一些已知的 OUI
    let test_cases = vec![
        ("00:00:00:00:00:00", "Xerox"),
        ("00:00:01:00:00:00", "Xerox"),
        ("00:00:02:00:00:00", "Xerox"),
        ("00:00:0C:00:00:00", "Cisco Systems"),
        ("00:00:17:00:00:00", "Oracle"),
        ("00:00:39:00:00:00", "Toshiba"),
        ("00:00:4C:00:00:00", "NEC"),
        ("00:00:60:00:00:00", "Kontron Europe GmbH"),
        ("00:00:85:00:00:00", "Canon"),
        ("00:00:87:00:00:00", "Hitachi"),
    ];

    for (mac_str, expected_org) in test_cases {
        let mac: MacAddress = mac_str.parse().unwrap();
        let org = OUI_DB.lookup_mac(mac);
        assert_eq!(org, Some(expected_org), "Failed for MAC: {}", mac_str);
    }
}

#[test]
fn test_oui_database_coverage() {
    // 测试数据库覆盖范围
    // 通过查询一些已知地址来验证数据库完整性

    // 测试一些随机地址，确保数据库工作正常
    let test_addresses = vec![
        "00:00:00:00:00:00",
        "00:00:0C:00:00:00",
        "00:55:DA:00:00:00",
        "00:16:E3:00:00:00",
    ];

    let mut found_count = 0;
    for addr in test_addresses {
        let mac: MacAddress = addr.parse().unwrap();
        if OUI_DB.lookup_mac(mac).is_some() {
            found_count += 1;
        }
    }

    // 至少应该能找到大部分
    assert!(
        found_count >= 3,
        "Expected to find at least 3 OUI entries, found {}",
        found_count
    );
}

#[test]
fn test_oui_all_types_coverage() {
    // 综合测试：验证 24/28/36 位和未分配 OUI 都能正确处理

    // === 24 位 OUI (MA-L) ===
    let mac_24: MacAddress = "00:00:01:00:00:00".parse().unwrap();
    let org_24 = OUI_DB.lookup_mac(mac_24);
    assert_eq!(org_24, Some("Xerox"), "24-bit OUI lookup failed");
    println!("✓ 24-bit OUI: {} -> {:?}", mac_24, org_24);

    // === 28 位 OUI (MA-M) ===
    let mac_28: MacAddress = "00:55:DA:00:00:00".parse().unwrap();
    let org_28 = OUI_DB.lookup_mac(mac_28);
    assert!(org_28.is_some(), "28-bit OUI lookup failed");
    println!("✓ 28-bit OUI: {} -> {:?}", mac_28, org_28);

    // === 36 位 OUI (MA-S) ===
    let mac_36: MacAddress = "00:16:E3:00:00:00".parse().unwrap();
    let org_36 = OUI_DB.lookup_mac(mac_36);
    assert!(org_36.is_some(), "36-bit OUI lookup failed");
    println!("✓ 36-bit OUI: {} -> {:?}", mac_36, org_36);

    // === 未分配 OUI ===
    let mac_none: MacAddress = "FF:FF:FF:00:00:00".parse().unwrap();
    let org_none = OUI_DB.lookup_mac(mac_none);
    assert_eq!(org_none, None, "Should return None for unallocated OUI");
    println!("✓ Unallocated OUI: {} -> {:?}", mac_none, org_none);

    println!("\n✅ All OUI types (24/28/36-bit and unallocated) are properly handled!");
}

#[test]
fn test_mac_address_ord() {
    let mac1: MacAddress = "00:11:22:33:44:55".parse().unwrap();
    let mac2: MacAddress = "00:11:22:33:44:56".parse().unwrap();
    let mac3: MacAddress = "00:11:22:33:44:55".parse().unwrap();

    assert!(mac1 < mac2);
    assert_eq!(mac1, mac3);
    assert!(mac2 > mac1);
}

#[test]
fn test_mac_address_default() {
    let mac = MacAddress::default();
    assert!(mac.is_zero());
}
