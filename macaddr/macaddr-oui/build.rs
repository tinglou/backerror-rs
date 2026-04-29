use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// 中间数据结构：单个 OUI 条目
#[derive(Debug, Clone)]
struct OuiEntry {
    /// 原始十六进制前缀字符串（如 "000001", "0001A2B" 等）
    prefix_str: String,
    /// 组织名称
    org_name: String,
    /// 前缀位长度（24/28/32/36）
    bit_len: usize,
}



#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
enum SubtableRef {
    Oui28(usize),
    Oui32(usize),
    Oui36(usize),
}

/// 28 位子表条目
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Oui28Entry {
    prefix: u8, // 高 4 位有效
    name_idx: usize,
}

/// 32 位子表条目
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Oui32Entry {
    prefix: u8,
    name_idx: usize,
}

/// 36 位子表条目
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Oui36Entry {
    prefix: u16, // 高 12 位有效：高 8 位是第 4 字节，低 4 位是第 5 字节的高 4 位
    name_idx: usize,
}

/// 解析 nmap-mac-prefixes 文件
fn parse_nmap_file(path: &Path) -> Vec<OuiEntry> {
    let file = fs::File::open(path).expect("Failed to open nmap-mac-prefixes file");
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();

        // 跳过空行和注释
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 分割前缀和名称
        let mut parts = line.splitn(2, ' ');
        let prefix_str = parts.next().unwrap().trim();
        let org_name = parts.next().unwrap_or("").trim();

        // 根据前缀长度判断位数
        // 6 字符 = 24 位，7 字符 = 28 位，8 字符 = 32 位，9 字符 = 36 位
        let bit_len = match prefix_str.len() {
            6 => 24,
            7 => 28,
            8 => 32,
            9 => 36,
            _ => {
                eprintln!("Warning: skipping invalid prefix length {}: {}", prefix_str.len(), prefix_str);
                continue;
            }
        };

        entries.push(OuiEntry {
            prefix_str: prefix_str.to_uppercase(),
            org_name: org_name.to_string(),
            bit_len,
        });
    }

    entries
}

/// 构建中间数据结构
struct OuiDataBuilder {
    /// 所有唯一名称的列表（用于生成 names 字符串）
    names: Vec<String>,
    /// 名称到索引的映射
    name_to_idx: BTreeMap<String, usize>,
    /// 24 位表数据：prefix -> (name_idx, subtable)
    oui24_map: BTreeMap<[u8; 3], (usize, Option<SubtableRef>)>,
    /// 28 位子表列表
    oui28_tables: Vec<Vec<Oui28Entry>>,
    /// 28 位子表索引到 24 位前缀的映射
    oui28_idx_to_prefix: BTreeMap<usize, [u8; 3]>,
    /// 32 位子表列表
    oui32_tables: Vec<Vec<Oui32Entry>>,
    /// 32 位子表索引到 24 位前缀的映射
    oui32_idx_to_prefix: BTreeMap<usize, [u8; 3]>,
    /// 36 位子表列表
    oui36_tables: Vec<Vec<Oui36Entry>>,
    /// 36 位子表索引到 24 位前缀的映射
    oui36_idx_to_prefix: BTreeMap<usize, [u8; 3]>,
}

impl OuiDataBuilder {
    fn new() -> Self {
        OuiDataBuilder {
            names: Vec::new(),
            name_to_idx: BTreeMap::new(),
            oui24_map: BTreeMap::new(),
            oui28_tables: Vec::new(),
            oui28_idx_to_prefix: BTreeMap::new(),
            oui32_tables: Vec::new(),
            oui32_idx_to_prefix: BTreeMap::new(),
            oui36_tables: Vec::new(),
            oui36_idx_to_prefix: BTreeMap::new(),
        }
    }

    /// 获取或添加名称，返回索引
    fn get_or_add_name(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.name_to_idx.get(name) {
            idx
        } else {
            let idx = self.names.len();
            self.names.push(name.to_string());
            self.name_to_idx.insert(name.to_string(), idx);
            idx
        }
    }

    /// 解析十六进制前缀为字节
    fn parse_prefix_hex(prefix_str: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity((prefix_str.len() + 1) / 2);
        let mut i = 0;
        while i < prefix_str.len() {
            let end = (i + 2).min(prefix_str.len());
            let hex_part = &prefix_str[i..end];
            // 如果只有一个字符，补 0
            let hex_part = if hex_part.len() == 1 {
                format!("{}0", hex_part)
            } else {
                hex_part.to_string()
            };
            bytes.push(u8::from_str_radix(&hex_part, 16).unwrap_or(0));
            i += 2;
        }
        bytes
    }

    /// 构建所有数据结构
    fn build(&mut self, entries: Vec<OuiEntry>) {
        // 按前缀长度分组
        // entries_24: 3 字节前缀 -> 名称列表
        // entries_28: 3 字节前缀 -> (4 字节高 4 位，名称) 列表
        // entries_32: 3 字节前缀 -> (第 4 字节，名称) 列表
        // entries_36: 3 字节前缀 -> (第 4 字节 + 第 5 字节高 4 位，名称) 列表
        let mut entries_24: BTreeMap<[u8; 3], Vec<(usize, String)>> = BTreeMap::new();
        let mut entries_28: BTreeMap<[u8; 3], Vec<(u8, usize)>> = BTreeMap::new();
        let mut entries_32: BTreeMap<[u8; 3], Vec<(u8, usize)>> = BTreeMap::new();
        let mut entries_36: BTreeMap<[u8; 3], Vec<(u16, usize)>> = BTreeMap::new();

        for entry in entries {
            let name_idx = self.get_or_add_name(&entry.org_name);
            let bytes = Self::parse_prefix_hex(&entry.prefix_str);

            match entry.bit_len {
                24 => {
                    let prefix: [u8; 3] = [bytes[0], bytes[1], bytes[2]];
                    entries_24
                        .entry(prefix)
                        .or_insert_with(Vec::new)
                        .push((name_idx, entry.org_name));
                }
                28 => {
                    // 按 3 字节前缀分组，存储第 4 字节的高 4 位
                    let prefix3: [u8; 3] = [bytes[0], bytes[1], bytes[2]];
                    let key4 = bytes[3] & 0xF0;
                    entries_28
                        .entry(prefix3)
                        .or_insert_with(Vec::new)
                        .push((key4, name_idx));
                }
                32 => {
                    // 按 3 字节前缀分组，存储第 4 字节
                    let prefix3: [u8; 3] = [bytes[0], bytes[1], bytes[2]];
                    let key4 = bytes[3];
                    entries_32
                        .entry(prefix3)
                        .or_insert_with(Vec::new)
                        .push((key4, name_idx));
                }
                36 => {
                    // 按 3 字节前缀分组，存储第 4 字节 + 第 5 字节高 4 位
                    let prefix3: [u8; 3] = [bytes[0], bytes[1], bytes[2]];
                    let key5 = ((bytes[3] as u16) << 8) | ((bytes[4] & 0xF0) as u16);
                    entries_36
                        .entry(prefix3)
                        .or_insert_with(Vec::new)
                        .push((key5, name_idx));
                }
                _ => {}
            }
        }

        // 构建 28 位子表：每个 3 字节前缀对应一个子表
        let mut oui24_to_subtable: BTreeMap<[u8; 3], SubtableRef> = BTreeMap::new();

        for (prefix3, entries_list) in entries_28 {
            let subtable_idx = self.oui28_tables.len();
            let mut table: Vec<Oui28Entry> = entries_list
                .into_iter()
                .map(|(key4, name_idx)| Oui28Entry {
                    prefix: key4,
                    name_idx,
                })
                .collect();
            table.sort();
            table.dedup_by(|a, b| a.prefix == b.prefix);
            self.oui28_tables.push(table);
            self.oui28_idx_to_prefix.insert(subtable_idx, prefix3);

            oui24_to_subtable.insert(prefix3, SubtableRef::Oui28(subtable_idx));
        }

        // 构建 32 位子表：每个 3 字节前缀对应一个子表
        for (prefix3, entries_list) in entries_32 {
            let subtable_idx = self.oui32_tables.len();
            let mut table: Vec<Oui32Entry> = entries_list
                .into_iter()
                .map(|(key4, name_idx)| Oui32Entry {
                    prefix: key4,
                    name_idx,
                })
                .collect();
            table.sort();
            table.dedup_by(|a, b| a.prefix == b.prefix);
            self.oui32_tables.push(table);
            self.oui32_idx_to_prefix.insert(subtable_idx, prefix3);

            oui24_to_subtable.insert(prefix3, SubtableRef::Oui32(subtable_idx));
        }

        // 构建 36 位子表：每个 3 字节前缀对应一个子表
        for (prefix3, entries_list) in entries_36 {
            let subtable_idx = self.oui36_tables.len();
            let mut table: Vec<Oui36Entry> = entries_list
                .into_iter()
                .map(|(key5, name_idx)| Oui36Entry {
                    prefix: key5,
                    name_idx,
                })
                .collect();
            table.sort();
            table.dedup_by(|a, b| a.prefix == b.prefix);
            self.oui36_tables.push(table);
            self.oui36_idx_to_prefix.insert(subtable_idx, prefix3);

            oui24_to_subtable.insert(prefix3, SubtableRef::Oui36(subtable_idx));
        }

        // 构建 24 位表：合并所有 3 字节前缀（包括只有子表的前缀）
        let mut all_prefixes: BTreeMap<[u8; 3], Option<usize>> = BTreeMap::new();
        
        // 添加 24 位条目的前缀
        for (prefix, name_list) in &entries_24 {
            let name_idx = name_list[0].0;
            all_prefixes.insert(*prefix, Some(name_idx));
        }
        
        // 添加只有子表的前缀（使用子表中第一个条目的名称）
        for (prefix, subtable_ref) in &oui24_to_subtable {
            if !all_prefixes.contains_key(prefix) {
                // 从子表中获取第一个名称
                let name_idx = match subtable_ref {
                    SubtableRef::Oui28(idx) => self.oui28_tables[*idx].first().map(|e| e.name_idx),
                    SubtableRef::Oui32(idx) => self.oui32_tables[*idx].first().map(|e| e.name_idx),
                    SubtableRef::Oui36(idx) => self.oui36_tables[*idx].first().map(|e| e.name_idx),
                };
                all_prefixes.insert(*prefix, name_idx);
            }
        }
        
        // 构建 oui_24_map
        for (prefix, name_idx_opt) in all_prefixes {
            let name_idx = name_idx_opt.unwrap_or(0);
            let subtable = oui24_to_subtable.get(&prefix).copied();
            self.oui24_map.insert(prefix, (name_idx, subtable));
        }
    }



    /// 生成 OuiLoc 值 - 子表引用
    fn encode_loc_subtable(subtable_type: u32, subtable_idx: usize) -> u32 {
        // 格式：bit-31-30=type, bit-29-16=reserved, bit-15-00=index
        assert!(subtable_type <= 0x03, "Invalid subtable type");
        assert!(subtable_idx <= 0xFFFF, "Subtable index too large");
        (subtable_type << 30) | (subtable_idx as u32 & 0xFFFF)
    }

    /// 生成 Rust 代码
    fn generate_code(&self, output_path: &Path) {
        let mut code = String::new();

        // 生成 names 字符串
        let names_string: String = self.names.join("");
        
        // 计算每个名称的偏移量
        let mut name_offsets: Vec<usize> = Vec::with_capacity(self.names.len());
        let mut current_offset = 0;
        for name in &self.names {
            name_offsets.push(current_offset);
            current_offset += name.len();
        }

        // 文件头
        code.push_str("// Auto-generated by macaddr-oui-codegen\n");
        code.push_str("// DO NOT EDIT MANUALLY\n");
        code.push_str("// rustfmt-off\n\n");
        code.push_str("use crate::oui::{OuiSubtable, OuiTable};\n\n");
        code.push_str("use crate::oui::OuiDb;\n\n");

        // 生成 names 常量
        code.push_str(&format!(
            "const OUI_NAMES: &str = r#\"{}\"#;\n\n",
            names_string
        ));

        // 生成 oui_24 表 - 直接内嵌数据
        let oui24_prefixes: Vec<[u8; 3]> = self.oui24_map.keys().copied().collect();
        code.push_str("#[rustfmt::skip]\n");
        code.push_str("const OUI_24_TABLE: OuiTable<3> = OuiTable {\n");
        
        // prefix 字段 - 一行 20 项
        code.push_str("    prefix: &[\n        ");
        for (i, prefix) in oui24_prefixes.iter().enumerate() {
            if i > 0 && i % 20 == 0 {
                code.push_str("\n        ");
            }
            code.push_str(&format!("[{},{},{}],", prefix[0], prefix[1], prefix[2]));
        }
        code.push_str("\n    ],\n");
        
        // loc 字段 - 一行 30 项
        code.push_str("    loc: &[\n        ");
        for (i, prefix) in oui24_prefixes.iter().enumerate() {
            if i > 0 && i % 30 == 0 {
                code.push_str("\n        ");
            }
            let (name_idx, subtable) = self.oui24_map.get(prefix).unwrap();
            let loc = match subtable {
                Some(SubtableRef::Oui28(idx)) => Self::encode_loc_subtable(1, *idx),
                Some(SubtableRef::Oui32(idx)) => Self::encode_loc_subtable(2, *idx),
                Some(SubtableRef::Oui36(idx)) => Self::encode_loc_subtable(3, *idx),
                None => {
                    // Type 00: 指向 names
                    let offset = name_offsets[*name_idx];
                    let name = &self.names[*name_idx];
                    let length = name.len();
                    assert!(offset <= 0x3FFFFF, "Offset too large: {}", offset);
                    assert!(length <= 0xFF, "Length too large: {}", length);
                    ((offset as u32) << 8) | (length as u32)
                }
            };
            code.push_str(&format!("0x{:08X},", loc));
        }
        code.push_str("\n    ],\n");
        code.push_str("};\n\n");

        // 生成 oui_28 数组 - 直接内嵌所有子表数据，每表一行，添加 24 位前缀注释
        code.push_str("#[rustfmt::skip]\n");
        code.push_str("const OUI_28_TABLES: &[OuiSubtable<u8>] = &[\n");
        for (idx, table) in self.oui28_tables.iter().enumerate() {
            // 获取 24 位前缀注释
            let prefix3 = self.oui28_idx_to_prefix.get(&idx).unwrap();
            code.push_str(&format!("    /* {:02X}{:02X}{:02X} */ OuiSubtable {{ prefix: &[", prefix3[0], prefix3[1], prefix3[2]));
            for (i, entry) in table.iter().enumerate() {
                if i > 0 { code.push_str(","); }
                code.push_str(&format!("0x{:02X}", entry.prefix));
            }
            code.push_str("], loc: &[");
            
            // loc 字段 - 单行
            for (i, entry) in table.iter().enumerate() {
                if i > 0 { code.push_str(","); }
                let offset = name_offsets[entry.name_idx];
                let name = &self.names[entry.name_idx];
                let length = name.len();
                let loc = ((offset as u32) << 8) | (length as u32);
                code.push_str(&format!("0x{:08X}", loc));
            }
            code.push_str("] },\n");
        }
        code.push_str("];\n\n");

        // 生成 oui_32 数组 - 直接内嵌所有子表数据，每表一行，添加 24 位前缀注释
        code.push_str("#[rustfmt::skip]\n");
        code.push_str("const OUI_32_TABLES: &[OuiSubtable<u8>] = &[\n");
        for (idx, table) in self.oui32_tables.iter().enumerate() {
            // 获取 24 位前缀注释
            let prefix3 = self.oui32_idx_to_prefix.get(&idx).unwrap();
            code.push_str(&format!("    /* {:02X}{:02X}{:02X} */ OuiSubtable {{ prefix: &[", prefix3[0], prefix3[1], prefix3[2]));
            for (i, entry) in table.iter().enumerate() {
                if i > 0 { code.push_str(","); }
                code.push_str(&format!("0x{:02X}", entry.prefix));
            }
            code.push_str("], loc: &[");
            
            // loc 字段 - 单行
            for (i, entry) in table.iter().enumerate() {
                if i > 0 { code.push_str(","); }
                let offset = name_offsets[entry.name_idx];
                let name = &self.names[entry.name_idx];
                let length = name.len();
                let loc = ((offset as u32) << 8) | (length as u32);
                code.push_str(&format!("0x{:08X}", loc));
            }
            code.push_str("] },\n");
        }
        code.push_str("];\n\n");

        // 生成 oui_36 数组 - 直接内嵌所有子表数据，每表一行，添加 24 位前缀注释
        code.push_str("#[rustfmt::skip]\n");
        code.push_str("const OUI_36_TABLES: &[OuiSubtable<u16>] = &[\n");
        for (idx, table) in self.oui36_tables.iter().enumerate() {
            // 获取 24 位前缀注释
            let prefix3 = self.oui36_idx_to_prefix.get(&idx).unwrap();
            code.push_str(&format!("    /* {:02X}{:02X}{:02X} */ OuiSubtable {{ prefix: &[", prefix3[0], prefix3[1], prefix3[2]));
            for (i, entry) in table.iter().enumerate() {
                if i > 0 { code.push_str(","); }
                code.push_str(&format!("0x{:04X}", entry.prefix));
            }
            code.push_str("], loc: &[");
            
            // loc 字段 - 单行
            for (i, entry) in table.iter().enumerate() {
                if i > 0 { code.push_str(","); }
                let offset = name_offsets[entry.name_idx];
                let name = &self.names[entry.name_idx];
                let length = name.len();
                let loc = ((offset as u32) << 8) | (length as u32);
                code.push_str(&format!("0x{:08X}", loc));
            }
            code.push_str("] },\n");
        }
        code.push_str("];\n\n");

        // 生成 OuiDb 常量
        code.push_str("pub const OUI_DB: OuiDb = OuiDb {\n");
        code.push_str("    names: OUI_NAMES,\n");
        code.push_str("    oui_24: OUI_24_TABLE,\n");
        code.push_str("    oui_28: OUI_28_TABLES,\n");
        code.push_str("    oui_32: OUI_32_TABLES,\n");
        code.push_str("    oui_36: OUI_36_TABLES,\n");
        code.push_str("};\n");

        // 写入文件
        fs::write(output_path, &code).expect("Failed to write output file");
        println!("Generated {} with {} bytes", output_path.display(), code.len());
        println!("Statistics:");
        println!("  - Unique names: {}", self.names.len());
        println!("  - OUI-24 entries: {}", oui24_prefixes.len());
        println!("  - OUI-28 tables: {}", self.oui28_tables.len());
        println!("  - OUI-32 tables: {}", self.oui32_tables.len());
        println!("  - OUI-36 tables: {}", self.oui36_tables.len());
    }
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let input_path = Path::new(manifest_dir).join("nmap-mac-prefixes");
    let output_path = Path::new(manifest_dir).join("src").join("oui_data.rs");

    println!("Reading from: {}", input_path.display());
    println!("Writing to: {}", output_path.display());

    // 解析文件
    let entries = parse_nmap_file(&input_path);
    println!("Parsed {} entries", entries.len());

    // 构建数据结构
    let mut builder = OuiDataBuilder::new();
    builder.build(entries);

    // 生成代码
    builder.generate_code(&output_path);
}
