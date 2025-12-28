pub mod composite;
pub mod field;
pub mod protocol;
pub mod resolved;
// pub mod reference;

pub use composite::{Array, Cluster, Composite, Entry};
pub use field::Field;
pub use protocol::Protocol;

use derive_more::Display;
use schemars::schema_for;
use schemars::JsonSchema;
#[cfg(test)]
use serde::de::value::{Error as ValueError, I64Deserializer, StrDeserializer};
#[cfg(test)]
use serde::de::IntoDeserializer;
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::ser::PrettyFormatter;
use serde_with::DisplayFromStr;
use serde_with::{formats::PreferOne, serde_as, DefaultOnNull, OneOrMany};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;

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

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct MemoryMap {
    #[serde(flatten)]
    protocol: Protocol,
    #[serde(rename = "&map")]
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    map: Vec<Composite>,
    #[serde(rename = "&def")]
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<OneOrMany<_,PreferOne>>")]
    def: Vec<Composite>,
}

impl MemoryMap {
    pub fn resolve(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    pub fn render(&self) -> String {
        // self.render_recursive()
        "".to_string()
    }

    pub fn render_to_writer<W, E>(&self, writer: W) -> Result<(), E>
    where
        W: Write,
        E: std::error::Error,
    {
        Result::Ok(())
    }
}

pub fn get_memory_map_schema() -> String {
    let schema = schema_for!(MemoryMap);
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    serde::Serialize::serialize(&schema, &mut ser).expect("Failed to serialize schema");
    String::from_utf8(buf).expect("Failed to convert serial buffer to string")
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct EnumMap(#[serde_as(as = "BTreeMap<DisplayFromStr, _>")] BTreeMap<u64, String>);

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
