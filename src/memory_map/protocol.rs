use crate::memory_map::HexStrOrUnsigned;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Protocol {
    /// An optional name for the protocol
    pub name: Option<String>,
    /// Maximum address in terms of addressUnit.
    /// Accepts '0x' prefixed hex strings with underscores allowed between digits to enhance readability
    #[serde_as(as = "HexStrOrUnsigned")]
    pub address_max: u64,
    /// Number of bytes accessed with one address
    pub address_unit: u64,
    /// Number of bytes aligned with the protocol's minimum transfer size. Must be greater or equal
    /// to addressUnit
    pub address_align: u64,
}
