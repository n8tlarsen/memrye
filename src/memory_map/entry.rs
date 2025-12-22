use crate::memory_map::{maybe_hex_str_or_unsigned, Access, Field};
use anyhow::anyhow;
use derive_more::Display;
use log::error;
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferOne, serde_as, OneOrMany};

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
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    fields: Vec<Field>,
}
