pub mod composite;
pub mod field;
pub mod protocol;
pub mod resolved;
pub mod serde_helpers;

pub use composite::{Array, Cluster, Composite, Entry};
pub use field::Field;
pub use protocol::Protocol;
pub use serde_helpers::{EnumMap, HexStrOrUnsigned, IntegerOrString};

use derive_more::Display;
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use serde_with::{formats::PreferOne, serde_as, DefaultOnNull, OneOrMany};

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

pub fn get_memory_map_schema() -> String {
    let schema = schema_for!(MemoryMap);
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    serde::Serialize::serialize(&schema, &mut ser).expect("Failed to serialize schema");
    String::from_utf8(buf).expect("Failed to convert serial buffer to string")
}
