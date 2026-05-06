//! # Organizationally Unique Identifiers (OUI) database
//!
//! ## 查询
//! 1. 取 mac 前 3 个字节，二分查找 `OuiDb::oui_24` `prefix`, 找不到返回 None，获取 `loc` 对应值
//! 2. 如 loc bit-31-30 = 0，则计算 offset/length，返回 `OuiDb::names[offset..offset+length]`
//! 2. 如 loc bit-31-30 = 1，取 mac [第 4 个字节 & 0xF0], 查找 `OuiDb::oui_28`
//! 2. 如 loc bit-31-30 = 2，取 mac [第 4 个字节], 查找 `OuiDb::oui_32`
//! 2. 如 loc bit-31-30 = 3，取 mac [第 4 个字节，第 5 个字节 & 0xF0], 查找 `OuiDb::oui_36`
//!
use crate::MacAddress;

const OUI_SUBTABLE: &str = "Ieee Registration Authority";

const OUI_VIRTUAL: [&'static str; 6] = [
    "VMware",
    "QEMU virtual NIC",
    "Bochs virtual NIC",
    "PearPC virtual NIC",
    "Cooperative Linux virtual NIC",
    "Oracle VirtualBox virtual NIC",
];

/// # Organizationally Unique Identifiers (OUI) database
///

pub struct OuiDb {
    pub(crate) names: &'static str,

    /// MA-L(24 bit prefix) 24 bit ≈1677 万 传统大厂/运营商
    pub(crate) oui_24: &'static OuiTable<3>,

    /// MA-M(28 bit prefix) 20 bit ≈104 万 中型设备商/模组厂
    pub(crate) oui_28: &'static [OuiSubtable<u8>],

    pub(crate) oui_32: &'static [OuiSubtable<u8>],

    /// MA-S(36 bit prefix) 12 bit =4096 IoT/小微/专用硬件
    pub(crate) oui_36: &'static [OuiSubtable<u16>],
}

/// # Locate org name or subtable offset
///
/// ## Type 00 - 指向 names 字符串
/// ```text
/// bit:  31 30 | 29 28 ... 11 10 09 08 | 07 06 ... 01 00
///       ─────   ─────────────────────   ───────────────
///       type       offset (22 bits)        length
/// ```
///
/// ## Type 01/02/03 - 指向子表偏移量
/// ```text
/// bit:  31 30 | 29 28 ... 17 16 | 15 14 ... 01 00
///       ─────   ───────────────   ───────────────
///       type       reserved        subtable index
/// ```
///
/// * bit 31-30: type
///   - 00 指向 OuiDb::names 中的偏移量和长度
///       - bit-29-08: offset (22 bits)
///       - bit-07-00: length (8 bits)
///   - 01 指向 OuiDb::oui_28 偏移量
///       - bit-15-00: subtable index (16 bits)
///   - 02 指向 OuiDb::oui_32 偏移量
///       - bit-15-00: subtable index (16 bits)
///   - 03 指向 OuiDb::oui_36 偏移量
///       - bit-15-00: subtable index (16 bits)
///
type OuiLoc = u32;

/// 列存储（节约内存，提升缓存利用率）
pub(crate) struct OuiTable<const N: usize> {
    pub(crate) prefix: &'static [[u8; N]],
    pub(crate) loc: &'static [OuiLoc],
}

/// 列存储（节约内存，提升缓存利用率）
pub(crate) struct OuiSubtable<T: 'static> {
    pub(crate) prefix: &'static [T],
    pub(crate) loc: &'static [OuiLoc],
}

impl OuiDb {
    /// Checks if the given OUI name corresponds to a virtual NIC.
    ///
    /// # Arguments
    ///
    /// * `name` - The OUI name to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the name is a virtual NIC (e.g., QEMU, VirtualBox), otherwise `false`.
    pub fn is_virtual_nic(name: &str) -> bool {
        OUI_VIRTUAL.contains(&name)
    }

    /// Returns the name identifier of the subtable (`oui_28`, `oui_32`, `oui_36`) in the database.
    ///
    /// # Returns
    ///
    /// Returns the subtable name string `"Ieee Registration Authority"`.
    pub fn oui_subtable_name() -> &'static str {
        OUI_SUBTABLE
    }

    /// Looks up the organization name (OUI vendor information) for a given MAC address.
    ///
    /// # Arguments
    ///
    /// * `mac` - The MAC address as a [`MacAddress`] type.
    ///
    /// # Returns
    ///
    /// Returns `Some(&'static str)` if the organization name is found, otherwise `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use macaddr_ouidb::*;
    /// let mac = MacAddress::new([0x00, 0x1A, 0x2B, 0x3C, 0x4D, 0x5E]);
    /// let org = OUI_DB.lookup_mac(mac);
    /// ```
    pub fn lookup_mac(&self, mac: MacAddress) -> Option<&'static str> {
        self.lookup(mac.octets())
    }

    /// Looks up the organization name (OUI vendor information) for a given MAC address byte array.
    ///
    /// Supports 24-bit, 28-bit, 32-bit, and 36-bit OUI queries.
    ///
    /// # Arguments
    ///
    /// * `mac` - A 6-byte MAC address byte array.
    ///
    /// # Returns
    ///
    /// Returns `Some(&'static str)` if the organization name is found. Returns `None` if the MAC
    /// address length is not 6 bytes or no match is found.
    ///
    // # Query Logic
    //
    // 1. Extract the first 3 bytes and perform a binary search in `oui_24.prefix`
    // 2. Based on `loc` bits 31-30, determine the type:
    //    - `00`: Retrieve string from `names` using offset and length
    //    - `01`: Look up in `oui_28` subtable using `mac[3] & 0xF0`
    //    - `02`: Look up in `oui_32` subtable using `mac[3]`
    //    - `03`: Look up in `oui_36` subtable using `mac[3]` and `mac[4] & 0xF0` combined
    pub fn lookup(&self, mac: [u8; 6]) -> Option<&'static str> {
        // 1. 提取前 3 字节作为 oui_24 的查找键
        let key3 = [mac[0], mac[1], mac[2]];

        // 2. 二分查找 oui_24
        let idx = match self.oui_24.prefix.binary_search(&key3) {
            Ok(idx) => idx,
            Err(_) => return None,
        };

        // 3. 获取 loc 值
        let loc = self.oui_24.loc[idx];

        // 4. 根据 bit-31-30 判断类型
        match (loc >> 30) & 0x03 {
            0 => {
                // names 偏移量和长度
                // bit-29-08: offset (22 bits), bit-07-00: length (8 bits)
                let offset = ((loc >> 8) & 0x3FFFFF) as usize;
                let length = (loc & 0xFF) as usize;
                self.names.get(offset..offset + length)
            }
            1 => {
                // oui_28: 取 mac[3] & 0xF0
                let sub_idx = (loc & 0xFFFF) as usize;
                let key4 = mac[3] & 0xF0;
                self.lookup_subtable(&self.oui_28[sub_idx], key4)
            }
            2 => {
                // oui_32: 取 mac[3]
                let sub_idx = (loc & 0xFFFF) as usize;
                let key4 = mac[3];
                self.lookup_subtable(&self.oui_32[sub_idx], key4)
            }
            3 => {
                // oui_36: 取 mac[3] 和 mac[4] & 0xF0
                // 组合为 16 位：高 8 位是 mac[3], 低 8 位是 mac[4] & 0xF0
                let sub_idx = (loc & 0xFFFF) as usize;
                let key5 = ((mac[3] as u16) << 8) | ((mac[4] & 0xF0) as u16);
                self.lookup_subtable(&self.oui_36[sub_idx], key5)
            }
            _ => None,
        }
    }

    fn lookup_subtable<T: Ord + Copy>(
        &self,
        subtable: &OuiSubtable<T>,
        key: T,
    ) -> Option<&'static str> {
        let idx = match subtable.prefix.binary_search(&key) {
            Ok(idx) => idx,
            Err(_) => return None,
        };
        let loc = subtable.loc[idx];

        // subtable 的 loc 只可能是 type 0 (指向 names)
        // bit-29-08: offset (22 bits), bit-07-00: length (8 bits)
        let offset = ((loc >> 8) & 0x3FFFFF) as usize;
        let length = (loc & 0xFF) as usize;
        self.names.get(offset..offset + length)
    }
}
