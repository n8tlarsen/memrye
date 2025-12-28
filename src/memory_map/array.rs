use crate::memory_map::{maybe_hex_str_or_unsigned, Access, Composite};
use anyhow::anyhow;
use derive_more::Display;
use log::error;
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferOne, serde_as, OneOrMany};
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
