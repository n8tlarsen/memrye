pub mod composite;
pub mod field;
pub mod protocol;
pub mod resolved;
pub mod serde_helpers;

pub use composite::{Array, Cluster, Composite, Entry};
pub use field::Field;
pub use protocol::Protocol;
pub use serde_helpers::{DisplayOption, EnumMap, HexStrOrUnsigned, IntegerOrString};

use anyhow::anyhow;
use derive_more::Display;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use serde_with::{formats::PreferOne, serde_as, DefaultOnNull, OneOrMany};
use std::collections::{BTreeMap, HashMap};

#[derive(Deserialize, Serialize, JsonSchema, Display, Default, Debug, Copy, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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
#[derive(Deserialize, Serialize, JsonSchema, Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct MemoryMap {
    #[serde(flatten)]
    pub(crate) protocol: Protocol,
    #[serde(rename = "&map")]
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    pub(crate) map: Vec<Composite>,
    #[serde(rename = "&def")]
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<OneOrMany<_,PreferOne>>")]
    pub(crate) def: Vec<Composite>,
}

impl MemoryMap {
    pub fn get_def_map(&self) -> Result<HashMap<String, &Composite>, anyhow::Error> {
        let mut def_map = HashMap::with_capacity(self.def.len());
        for def in &self.def {
            if def_map.insert(def.name().to_string(), def).is_some() {
                return Err(anyhow!("Definition \"{}\" already exists.", def.name()));
            }
        }
        Ok(def_map)
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

pub trait Name {
    fn name(&self) -> &str;
    fn type_name() -> &'static str;
}
