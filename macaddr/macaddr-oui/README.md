# macaddr-oui

高性能 MAC 地址 OUI（Organizationally Unique Identifier）查询库，支持从 MAC 地址查找制造商信息。

## 特性

- 🚀 **零成本抽象**：编译时生成查找表，运行时只有二分查找
- 💾 **内存高效**：列式存储 + 紧凑编码，完整数据库仅 ~300KB
- ⚡ **快速查询**：O(log n) 时间复杂度，支持 24/28/36 位 OUI
- 🔍 **全面覆盖**：包含 52,000+ 个 OUI 条目（来自 Nmap 项目）
- 🎯 **虚拟网卡检测**：内置常见虚拟化平台网卡识别
- 📦 **灵活集成**：支持 `serde` 序列化，可选 `pnet` 互操作

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
macaddr-oui = "0.1"
```

## 快速开始

### 基本用法

```rust
use macaddr_oui::{MacAddress, OUI_DB};

// 创建 MAC 地址
let mac = MacAddress::from_str("00:55:DA:0A:BB:CC")?;

// 查询制造商信息
match OUI_DB.lookup(mac) {
    Some(org_name) => println!("制造商：{}", org_name),
    None => println!("未找到 OUI 信息"),
}
```

### 虚拟网卡检测

```rust
use macaddr_oui::OuiDb;

let org_name = OUI_DB.lookup(mac).unwrap_or("Unknown");

if OuiDb::is_virtual_nic(org_name) {
    println!("这是虚拟网卡：{}", org_name);
}
```

### 与 pnet 互操作

启用 `pnet` 特性后，可以与 `pnet_base::MacAddr` 互转：

```rust
use macaddr_oui::MacAddress;
use pnet_base::MacAddr;

let pnet_mac = MacAddr::new(0x00, 0x11, 0x22, 0x33, 0x44, 0x55);
let mac: MacAddress = pnet_mac.into();
```

### Serde 支持

启用 `serde` 特性后，支持 JSON 序列化：

```rust
use macaddr_oui::MacAddress;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Device {
    mac: MacAddress,
    name: String,
}
```

## API 文档

### `MacAddress`

以太网 MAC 地址类型（6 字节）。

#### 构造方法

```rust
// 从字节数组
let mac = MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);

// 从字符串（支持 ':' 或 '-' 分隔）
let mac: MacAddress = "00:11:22:33:44:55".parse()?;
let mac: MacAddress = "00-11-22-33-44-55".parse()?;

// 特殊地址
let zero = MacAddress::zero();
let broadcast = MacAddress::broadcast();
```

#### 属性检查

```rust
mac.is_zero()          // 是否全零地址
mac.is_broadcast()     // 是否广播地址
mac.is_unicast()       // 是否单播地址
mac.is_multicast()     // 是否组播地址
mac.is_universal()     // 是否全球管理地址 (UAA)
mac.is_local()         // 是否本地管理地址 (LAA)
```

### `OUI_DB`

预编译的 OUI 数据库实例。

```rust
use macaddr_oui::OUI_DB;

let org_name = OUI_DB.lookup(mac);  // Option<&'static str>
```

### `OuiDb`

OUI 数据库工具方法。

```rust
// 检查是否为虚拟网卡
OuiDb::is_virtual_nic("QEMU virtual NIC")  // true

// 获取子表名称
OuiDb::oui_subtable_name()  // "Ieee Registration Authority"
```

## 数据格式

支持三种 OUI 类型：

| 类型 | 位数 | 格式 | 说明 |
|------|------|------|------|
| OUI-24 (MA-L) | 24 | `XX:XX:XX` | 传统分配，前 3 字节 |
| OUI-28 (MA-M) | 28 | `XX:XX:XX:X` | 中等规模，前 3 字节 + 第 4 字节高 4 位 |
| OUI-36 (MA-S) | 36 | `XX:XX:XX:XX:X` | 小规模，前 4 字节 + 第 5 字节高 4 位 |

查询时自动匹配最精确的类型。

## 特性标志

| 特性 | 默认 | 说明 |
|------|------|------|
| `std` | ✅ | 标准库支持 |
| `serde` | ✅ | Serde 序列化支持 |
| `pnet` | ✅ | pnet_base 互操作 |

### 最小化依赖（no_std）

```toml
[dependencies]
macaddr-oui = { version = "0.1", default-features = false }
```

## 性能数据

- **数据库大小**：~300KB（编译后）
- **查询时间**：~50-100ns（L1 缓存命中）
- **OUI 条目**：52,085 个
- **唯一组织名**：29,584 个

## 数据来源

OUI 数据来自 [Nmap 项目](https://nmap.org/) 的 `nmap-mac-prefixes` 文件，该文件整合了 IEEE 官方分配的 OUI 数据。

数据在编译时通过 `build.rs` 自动生成，确保：
- 与源代码版本一致
- 无需运行时加载
- 零初始化开销

## 命令行工具

本包包含一个命令行工具，用于快速查询 MAC 地址的 OUI 信息。

### 安装

```bash
cargo install macaddr-oui
```

### 用法

```bash
# 查询 MAC 地址
macaddr-oui 00:55:DA:0A:BB:CC

# 支持横线分隔格式
macaddr-oui 00-55-DA-0A-BB-CC
```

### 输出示例

```
$ macaddr-oui 00:00:0C:00:00:00
MAC Address: 00:00:0c:00:00:00
Formatted:   00:00:0c:00:00:00
Octets:      00-00-0C-00-00-00

Address Type:
  Unicast:     Yes
  Multicast:   No
  Broadcast:   No
  Universal:   Yes
  Local:       No

Organization: Cisco Systems
```

## 测试

运行测试套件：

```bash
cargo test
```

测试覆盖：
- ✅ MAC 地址解析（冒号/横线分隔）
- ✅ 特殊地址检测（零地址、广播、组播等）
- ✅ OUI 查询（24/28/36 位）
- ✅ 虚拟网卡识别
- ✅ 批量查询
- ✅ 数据库完整性

## 许可证

Apache-2.0

## 贡献

欢迎提交 Issue 和 Pull Request！

## 相关链接

- [ crates.io](https://crates.io/crates/macaddr-oui)
- [文档](https://docs.rs/macaddr-oui)
- [IEEE OUI 查询](https://standards.ieee.org/products-programs/regauth/)
- [Nmap 项目](https://nmap.org/)
