pub mod array;
pub mod cluster;
pub mod entry;
pub mod field;
pub mod memory_map;
pub mod protocol;
// pub mod reference;

pub use array::Array;
pub use cluster::Cluster;
// pub use composite::Composite;
pub use entry::Entry;
pub use field::Field;
pub use memory_map::MemoryMap;
pub use protocol::Protocol;

use derive_more::Display;
use schemars::JsonSchema;
#[cfg(test)]
use serde::de::value::{Error as ValueError, I64Deserializer, StrDeserializer};
#[cfg(test)]
use serde::de::IntoDeserializer;
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, JsonSchema, Display, Default, Debug, Copy, Clone)]
pub enum Access {
    /// Read-only access is permitted
    #[default]
    #[serde(rename = "r")]
    #[display("Read-only")]
    Read,
    /// Write-only access is permitted
    #[serde(rename = "w")]
    #[display("Write-only")]
    Write,
    /// Both read and write access is permitted
    #[serde(rename = "rw")]
    #[display("Read/Write")]
    ReadWrite,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(untagged)]
pub enum Composite {
    Array(Array),
    Cluster(Cluster),
    Entry(Entry),
    Reference {
        #[serde(rename = "@ref")]
        #[garde(pattern(r"[-_ A-Za-z0-9\/]*"))]
        reference: String,
    },
}

fn hex_str_or_unsigned<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct HexVisitor;

    impl Visitor<'_> for HexVisitor {
        type Value = u64;

        fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            fmt.write_str("\"0x\" prefixed hex string or u64")
        }

        fn visit_i64<E>(self, val: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if val >= 0 {
                Ok(val as u64)
            } else {
                Err(E::invalid_value(Unexpected::Signed(val), &self))
            }
        }

        fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(val)
        }

        fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let error = E::invalid_value(Unexpected::Str(val), &self);
            if let Some(stripped) = val.strip_prefix("0x") {
                let deformat = stripped.to_string().replace("_", "");
                match u64::from_str_radix(&deformat, 16) {
                    Ok(parsed_int) => Ok(parsed_int),
                    Err(_) => Err(error),
                }
            } else {
                Err(error)
            }
        }
    }

    deserializer.deserialize_any(HexVisitor)
}

#[test]
fn test_hex_str_ok() {
    let deserializer: StrDeserializer<ValueError> = "0xffff".into_deserializer();
    assert_eq!(hex_str_or_unsigned(deserializer), Ok(65535));
}

#[test]
fn test_hex_str_err() {
    let deserializer: StrDeserializer<ValueError> = "ffff".into_deserializer();
    assert_eq!(
        hex_str_or_unsigned(deserializer).unwrap_err().to_string(),
        "invalid value: string \"ffff\", expected \"0x\" prefixed hex string or u64"
    );
}

#[test]
fn test_negative_i64_err() {
    let deserializer: I64Deserializer<ValueError> = (-1i64).into_deserializer();
    assert_eq!(
        hex_str_or_unsigned(deserializer).unwrap_err().to_string(),
        "invalid value: integer `-1`, expected \"0x\" prefixed hex string or u64"
    );
}

#[test]
fn test_positive_i64_ok() {
    let deserializer: I64Deserializer<ValueError> = (1i64).into_deserializer();
    assert_eq!(hex_str_or_unsigned(deserializer), Ok(1));
}

fn maybe_hex_str_or_unsigned<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(hex_str_or_unsigned(deserializer)?))
}
