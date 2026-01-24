use crate::memory_map::{Access, Field, HexStrOrUnsigned, IntegerOrString};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferOne, serde_as, DefaultOnNull, OneOrMany};

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged)]
pub enum Index {
    Length(u64),
    Range { high: u64, low: u64 },
    List(#[serde_as(as = "OneOrMany<Option<IntegerOrString>, PreferOne>")] Vec<Option<String>>),
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct Array {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde_as(as = "Option<HexStrOrUnsigned>")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    elements: Box<Composite>,
    index: Index,
    #[serde_as(as = "Option<HexStrOrUnsigned>")]
    increment: Option<u64>,
}

impl Array {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Cluster {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<HexStrOrUnsigned>")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    elements: Vec<Composite>,
}

impl Cluster {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl IntoIterator for Cluster {
    type Item = Composite;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Entry {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<HexStrOrUnsigned>")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    /// Length of the entry in bytes
    bytes: u32,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<OneOrMany<_,PreferOne>>")]
    fields: Vec<Field>,
}

impl Entry {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
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
    Map {
        #[serde(rename = "*map")]
        #[garde(pattern(r"[-_ A-Za-z0-9\/]*"))]
        map: String,
    },
}
