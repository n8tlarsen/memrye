use crate::memory_map::resolved::ResolvedEntry;
use crate::memory_map::{Access, Field, HexStrOrUnsigned, IntegerOrString, Name, Protocol};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::{formats::PreferOne, serde_as, DefaultOnNull, OneOrMany};
use std::collections::HashMap;

pub(crate) struct Parent<'a> {
    table: &'a str,
    address: u64,
    access: Access,
}

impl<'a> Default for Parent<'a> {
    fn default() -> Self {
        Parent {
            table: "Anonymous",
            address: 0u64,
            access: Access::Read,
        }
    }
}

impl<'a> Parent<'a> {
    pub fn default_with_table(table: &'a str) -> Self {
        Parent {
            table,
            address: 0u64,
            access: Access::Read,
        }
    }
    pub fn increment_address(&mut self, size: u64) {
        self.address += size;
    }
    pub fn get_table(&self) -> &str {
        self.table
    }
    pub fn get_address(&self) -> u64 {
        self.address
    }
    pub fn get_access(&self) -> Access {
        self.access
    }
}

/// The resolver trait provides a common API for resolving optional fields of composite document
/// elements
pub trait Resolver {
    /// Resolve the composite type for tabular presentation
    fn resolve(&self, parent: &Parent, def_map: &HashMap<String, &Record>, protocol: &Protocol);
    /// Return the size of the composite in bytes
    fn size(&self, def_map: &HashMap<String, &Record>) -> u64;
}

/// The Index describes a list or series expansion for use with the Array composite
#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum Index {
    Length(u64),
    Range { high: u64, low: u64 },
    List(#[serde_as(as = "OneOrMany<Option<IntegerOrString>, PreferOne>")] Vec<Option<String>>),
}

/// Helper function for deserializing
const fn default_name_index() -> i64 {
    -1
}

/// Arrays allow for list or series expansion of other elements.
#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct Array {
    /// The index in the array element's name pointing where to insert the enumerating string.
    /// Positive indexes start from the 0 position in the target string and increment towards the
    /// end of the target string. Negative indexes start from the end position in the target string
    /// and increment towards the start of the target string. Index 0 represents inserting before
    /// the array element's name while index -1 represents inserting at the end of the array
    /// element's name. Default behavior is to append to the end of the target string (-1).
    #[serde(default = "default_name_index")]
    enumeration_index: i64,
    /// Prepend the given string to the enumerating string before insterting into the target string
    #[serde(skip_serializing_if = "Option::is_none")]
    prepend_delim: Option<String>,
    /// Append the given string to the enumerating string before inserting into the target string
    #[serde(skip_serializing_if = "Option::is_none")]
    append_delim: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde_as(as = "Option<HexStrOrUnsigned>")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    /// The contents of each array element
    element: Serial,
    /// The enumerating index(es) of the array. Multi-dimensional arrays are described with an array
    /// of Index objects where the Cartesian product of each and every Index element gives the
    /// resulting list or series expansion. The first Index is the least significant and will
    /// appear last in the enumerating string.
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    index: Vec<Index>,
    /// Increment or stride length in bytes. Multi-dimensional arrays are described with an array
    /// of increment values. If present, the dimensions must match those of the index key.
    #[serde_as(as = "Option<OneOrMany<HexStrOrUnsigned, PreferOne>>")]
    increment: Option<Vec<u64>>,
}

impl Resolver for Array {
    fn resolve(&self, parent: &Parent, def_map: &HashMap<String, &Record>, protocol: &Protocol) {}
    fn size(&self, def_map: &HashMap<String, &Record>) -> u64 {
        0u64
    }
}

impl Name for Array {
    fn name(&self) -> &str {
        match &self.element {
            Serial::Cluster(cluster) => cluster.name(),
            Serial::Entry(entry) => entry.name(),
            Serial::Reference(..) => "Reference",
            Serial::Map(..) => "Map",
        }
    }
    fn type_name(&self) -> &'static str {
        "Array"
    }
}

/// Clusters represent groupings of elements and are akin to structs.
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
    elements: Vec<Record>,
}

impl Resolver for Cluster {
    fn resolve(&self, parent: &Parent, def_map: &HashMap<String, &Record>, protocol: &Protocol) {
        for item in self.into_iter() {
            match item {
                Record::Entry(entry) => entry.resolve(parent, def_map, protocol),
                Record::Array(array) => array.resolve(parent, def_map, protocol),
                Record::Cluster(cluster) => cluster.resolve(parent, def_map, protocol),
                Record::Reference { .. } => {}
                Record::Map { .. } => {}
            };
        }
    }
    fn size(&self, def_map: &HashMap<String, &Record>) -> u64 {
        0u64
    }
}

impl Name for Cluster {
    fn name(&self) -> &str {
        &self.name
    }
    fn type_name(&self) -> &'static str {
        "Cluster"
    }
}

impl IntoIterator for Cluster {
    type Item = Record;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a Cluster {
    type Item = &'a Record;
    type IntoIter = std::slice::Iter<'a, Record>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<'a> IntoIterator for &'a mut Cluster {
    type Item = &'a mut Record;
    type IntoIter = std::slice::IterMut<'a, Record>;
    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter_mut()
    }
}

/// Entries are the fundamental unit of memory. They can be further described with field
/// definitions.
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

impl Resolver for Entry {
    fn resolve(&self, parent: &Parent, def_map: &HashMap<String, &Record>, protocol: &Protocol) {}
    fn size(&self, _def_map: &HashMap<String, &Record>) -> u64 {
        self.bytes.into()
    }
}

impl Name for Entry {
    fn name(&self) -> &str {
        &self.name
    }
    fn type_name(&self) -> &'static str {
        "Entry"
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename = "*ref")]
pub struct Reference(#[garde(pattern(r"[-_ A-Za-z0-9\/]*"))] pub String);

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename = "*map")]
pub struct Map(#[garde(pattern(r"[-_ A-Za-z0-9\/]*"))] pub String);

/// Collects the memory map record types into an enum for serde processing
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged)]
pub enum Record {
    Array(Array),
    Cluster(Cluster),
    Entry(Entry),
    Reference(Reference),
    Map(Map),
}

impl Resolver for Record {
    fn resolve(&self, parent: &Parent, def_map: &HashMap<String, &Record>, protocol: &Protocol) {
        match self {
            Record::Array(array) => array.resolve(parent, def_map, protocol),
            Record::Cluster(cluster) => cluster.resolve(parent, def_map, protocol),
            Record::Entry(entry) => entry.resolve(parent, def_map, protocol),
            Record::Reference(reference) => {}
            Record::Map(map) => {}
        }
    }
    fn size(&self, def_map: &HashMap<String, &Record>) -> u64 {
        0u64
    }
}

impl Name for Record {
    fn name(&self) -> &str {
        match self {
            Record::Array(array) => array.name(),
            Record::Cluster(cluster) => cluster.name(),
            Record::Entry(entry) => entry.name(),
            Record::Reference(reference) => reference.0.as_str(),
            Record::Map(map) => map.0.as_str(),
        }
    }
    fn type_name(&self) -> &'static str {
        match self {
            Record::Array(array) => array.type_name(),
            Record::Cluster(cluster) => cluster.type_name(),
            Record::Entry(entry) => entry.type_name(),
            Record::Reference { .. } => "Reference",
            Record::Map { .. } => "Map",
        }
    }
}

impl Name for &Record {
    fn name(&self) -> &str {
        match self {
            Record::Array(array) => array.name(),
            Record::Cluster(cluster) => cluster.name(),
            Record::Entry(entry) => entry.name(),
            Record::Reference(reference) => reference.0.as_str(),
            Record::Map(map) => map.0.as_str(),
        }
    }
    fn type_name(&self) -> &'static str {
        match self {
            Record::Array(array) => array.type_name(),
            Record::Cluster(cluster) => cluster.type_name(),
            Record::Entry(entry) => entry.type_name(),
            Record::Reference { .. } => "Reference",
            Record::Map { .. } => "Map",
        }
    }
}

/// Collects types that may be represented as serial array elements into an enum for serde processing
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged)]
pub enum Serial {
    Cluster(Cluster),
    Entry(Entry),
    Reference(Reference),
    Map(Map),
}
