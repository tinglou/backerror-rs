use core::fmt;
use core::str::FromStr;

#[cfg(feature = "serde")]
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

/// The number of bytes in an ethernet (MAC) address.
pub const ETHER_ADDR_LEN: usize = 6;

const LOCAL_ADDR_BIT: u8 = 0x02;
const MULTICAST_ADDR_BIT: u8 = 0x01;

/// Mac Address
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MacAddress(pub(crate) [u8; 6]);
impl MacAddress {
    pub const fn new(mac: [u8; 6]) -> Self {
        MacAddress(mac)
    }

    /// Construct an all-zero `MacAddr` instance.
    pub fn zero() -> Self {
        Default::default()
    }

    /// Construct a broadcast `MacAddr` instance.
    pub fn broadcast() -> Self {
        [0xff; ETHER_ADDR_LEN].into()
    }

    /// Returns true if a `MacAddr` is an all-zero address.
    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    /// Returns true if the MacAddr is a universally administered addresses (UAA).
    pub fn is_universal(&self) -> bool {
        !self.is_local()
    }

    /// Returns true if the MacAddr is a locally administered addresses (LAA).
    pub fn is_local(&self) -> bool {
        (self.0[0] & LOCAL_ADDR_BIT) == LOCAL_ADDR_BIT
    }

    /// Returns true if the MacAddr is a unicast address.
    pub fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }

    /// Returns true if the MacAddr is a multicast address.
    pub fn is_multicast(&self) -> bool {
        (self.0[0] & MULTICAST_ADDR_BIT) == MULTICAST_ADDR_BIT
    }

    /// Returns true if the MacAddr is a broadcast address.
    pub fn is_broadcast(&self) -> bool {
        *self == Self::broadcast()
    }

    pub fn octets(&self) -> &[u8] {
        &self.0
    }

    pub const fn from_slice(mac: &[u8]) -> Option<Self> {
        if mac.len() != 6 {
            return None;
        }
        let bytes = [mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]];
        Some(MacAddress(bytes))
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(value: [u8; 6]) -> Self {
        Self(value)
    }
}

impl fmt::Debug for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParseMacError {
    #[error("Invalid length")]
    InvalidLength,
    #[error("Invalid digit")]
    InvalidDigit,
}

impl FromStr for MacAddress {
    type Err = ParseMacError;

    /// parse mac address, e.g. `52:54:00:12:34:56` or `52-54-00-12-34-56`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = Self::from_str_sep(s, ':');
        if value.is_ok() {
            value
        } else {
            Self::from_str_sep(s, '-')
        }
    }
}

impl MacAddress {
    /// parse mac address, e.g. `52:54:00:12:34:56` or `52-54-00-12-34-56`
    fn from_str_sep(s: &str, separator: char) -> Result<Self, ParseMacError> {
        let mut parts = [0u8; 6];
        let splits = s.split(separator);
        let mut i = 0;
        for split in splits {
            if i == 6 {
                return Err(ParseMacError::InvalidLength);
            }
            match u8::from_str_radix(split, 16) {
                Ok(b) if split.len() != 0 => parts[i] = b,
                _ => return Err(ParseMacError::InvalidDigit),
            }
            i += 1;
        }

        if i == 6 {
            Ok(Self(parts))
        } else {
            Err(ParseMacError::InvalidLength)
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for MacAddress {
    /// Serializes the MAC address.
    ///
    /// It serializes either to a string or its binary representation, depending on what the format
    /// prefers.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            serializer.serialize_bytes(&self.0)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MacAddress {
    /// Deserializes the MAC address.
    ///
    /// It deserializes it from either a byte array (of size 6) or a string. If the format is
    /// self-descriptive (like JSON or MessagePack), it auto-detects it. If not, it obeys the
    /// human-readable property of the deserializer.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MacAddrVisitor;
        impl<'de> de::Visitor<'de> for MacAddrVisitor {
            type Value = MacAddress;

            fn visit_str<E: de::Error>(self, value: &str) -> Result<MacAddress, E> {
                value.parse().map_err(|err| E::custom(err))
            }

            fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<MacAddress, E> {
                if v.len() == 6 {
                    Ok(MacAddress::new([v[0], v[1], v[2], v[3], v[4], v[5]]))
                } else {
                    Err(E::invalid_length(v.len(), &self))
                }
            }

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "either a string representation of a MAC address or 6-element byte array"
                )
            }
        }

        // Decide what hint to provide to the deserializer based on if it is human readable or not
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(MacAddrVisitor)
        } else {
            deserializer.deserialize_bytes(MacAddrVisitor)
        }
    }
}

#[cfg(feature = "pnet")]
impl From<pnet_base::MacAddr> for MacAddress {
    fn from(value: pnet_base::MacAddr) -> Self {
        Self(value.octets())
    }
}