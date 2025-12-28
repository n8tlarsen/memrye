use crate::memory_map::{maybe_hex_str_or_unsigned, Access, Field};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferOne, serde_as, DefaultOnNull, OneOrMany};
use std::collections::BTreeMap;

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Array {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    elements: Vec<Composite>,
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    increment: Option<u64>,
    index_enums: Option<BTreeMap<String, u64>>,
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Cluster {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    elements: Vec<Composite>,
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Entry {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    /// Length of the entry in bytes
    bytes: u32,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<OneOrMany<_,PreferOne>>")]
    fields: Vec<Field>,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(untagged)]
pub enum Composite {
    Array(Array),
    Cluster(Cluster),
    Entry(Entry),
    Reference {
        #[serde(rename = "*ref")]
        #[garde(pattern(r"[-_ A-Za-z0-9\/]*"))]
        reference: String,
    },
}
