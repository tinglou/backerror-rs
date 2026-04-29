use crate::MacAddress;

const OUI_SUBTABLE: &str = "Ieee Registration Authority";

const OUI_VIRTUAL: [&'static str; 5] = [
    "QEMU virtual NIC",
    "Bochs virtual NIC",
    "PearPC virtual NIC",
    "Cooperative Linux virtual NIC",
    "Oracle VirtualBox virtual NIC",
];

/// # Organizationally Unique Identifiers (OUI) database
///
/// ## 查询
/// 1. 取 mac 前 3 个字节，二分查找 `OuiDb::oui_24` `prefix`, 找不到返回 None，获取 `loc` 对应值
/// 2. 如 loc bit-31-30 = 0，则计算offset/length，返回 `OuiDb::names[offset..offset+length]`
/// 2. 如 loc bit-31-30 = 1，取 mac [第 4 个字节 & 0xF0], 查找 `OuiDb::oui_28`
/// 2. 如 loc bit-31-30 = 2，取 mac [第 4 个字节], 查找 `OuiDb::oui_32`
/// 2. 如 loc bit-31-30 = 3，取 mac [第 4个字节，第 5 个字节 & 0xF0], 查找 `OuiDb::oui_36`
pub struct OuiDb {
    pub(crate) names: &'static str,

    pub(crate) oui_24: OuiTable<3>,

    pub(crate) oui_28: &'static [OuiSubtable<u8>],

    pub(crate) oui_32: &'static [OuiSubtable<u8>],

    pub(crate) oui_36: &'static [OuiSubtable<u16>],
}

/// # Locate org name or subtable offset
///
/// * bit 31-30
///   - 00 指向 OuiDb::names 中的偏移量和长度，
///       - bit-29-09 offset
///       - bit-07-00 length
///   - 01 指向 OuiDb::oui_28 偏移量(bit 15-00)
///   - 02 指向 OuiDb::oui_32 偏移量(bit 15-00)
///   - 03 指向 OuiDb::oui_36 偏移量(bit 15-00)
///
type OuiLoc = u32;

/// 列存储（节约内存，提升缓存利用率）
pub struct OuiTable<const N: usize> {
    pub(crate) prefix: &'static [[u8; N]],
    pub(crate) loc: &'static [OuiLoc],
}

/// 列存储（节约内存，提升缓存利用率）
pub struct OuiSubtable<T: 'static> {
    pub(crate) prefix: &'static [T],
    pub(crate) loc: &'static [OuiLoc],
}

impl OuiDb {
    pub fn lookup(&self, mac: MacAddress) -> Option<&'static str> {
        None
    }
}
