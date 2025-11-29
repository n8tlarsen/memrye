use super::hex_str_or_unsigned;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Protocol {
    /// An optional name for the protocol
    pub name: Option<String>,
    /// Maximum address in terms of addressUnit.
    /// Accepts '0x' prefixed hex strings with underscores allowed between digits to enhance readability
    #[serde(deserialize_with = "hex_str_or_unsigned")]
    pub address_max: u64,
    /// Number of bytes accessed with one address
    pub address_unit: u64,
    /// Number of bytes aligned with the protocol's minimum transfer size. Must be greater or equal
    /// to addressUnit
    pub address_align: u64,
}
