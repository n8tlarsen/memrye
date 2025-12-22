use crate::memory_map::{maybe_hex_str_or_unsigned, Access, Composite};
use anyhow::anyhow;
use derive_more::Display;
use log::error;
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferOne, serde_as, OneOrMany};

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
