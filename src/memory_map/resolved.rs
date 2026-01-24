use crate::memory_map::{
    field::{FieldType, Value},
    Access,
};
use anyhow::anyhow;
use derive_more::Display;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use tabled::Tabled;

use super::{Array, Cluster, Composite, DisplayOption, Entry, Field, MemoryMap};

#[derive(Debug, Display, Clone)]
pub enum LinkOrType {
    #[display("[{text}]({link})")]
    Link {
        text: String,
        link: String,
    },
    Type(FieldType),
}

#[derive(Debug, Clone)]
pub struct Range {
    start: u64,
    end: u64,
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start == self.end {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}..{}", self.start, self.end)
        }
    }
}

#[derive(Tabled, Debug, Clone)]
#[tabled(rename_all = "Upper Title Case")]
pub struct ResolvedEntry {
    /// Name of the entry
    name: String,
    /// Address of the entry
    address: u64,
    /// Entry accessibility
    access: Access,
    /// Entry type
    #[tabled(rename = "Type")]
    entry_type: LinkOrType,
    /// The unit of measurement of a numeric type. Ignored for other types.
    unit: DisplayOption<String>,
    /// The minimum allowed value of a numeric type. Ignored for other types.
    minimum: DisplayOption<f64>,
    /// The maximum allowed value of a numeric type. Ignored for other types.
    maximum: DisplayOption<f64>,
    /// The default value of the entry.
    value: Value,
}

#[derive(Default)]
pub struct ResolvedMemoryMap {
    entries: BTreeMap<String, Vec<ResolvedEntry>>,
    fields: BTreeMap<String, Vec<Field>>,
}

impl ResolvedMemoryMap {
    pub fn new_entry_table(
        &mut self,
        name: &str,
    ) -> Result<&mut Vec<ResolvedEntry>, anyhow::Error> {
        let duplicate = self.entries.insert(name.to_string(), Vec::new());
        if duplicate.is_some() {
            Err(anyhow!("Cluster name {} already exists.", name))
        } else {
            Ok(self.entries.get_mut(name).unwrap())
        }
    }

    pub fn new_field_table(&mut self, name: &str) -> Result<&mut Vec<Field>, anyhow::Error> {
        let duplicate = self.fields.insert(name.to_string(), Vec::new());
        if duplicate.is_some() {
            Err(anyhow!("Cluster name {} already exists.", name))
        } else {
            Ok(self.fields.get_mut(name).unwrap())
        }
    }

    pub fn render(&self) -> String {
        // self.render_recursive()
        "".to_string()
    }

    pub fn render_to_writer<W, E>(&self, writer: W) -> Result<(), E>
    where
        W: Write,
        E: std::error::Error,
    {
        Result::Ok(())
    }

    fn resolve_cluster(&mut self, cluster: &Cluster, defs: &ResolvedMemoryMap) {
        let mut table = self.new_entry_table(cluster.name());
        // for item in cluster {
        //     match item {
        //         Composite::Entry()
        //     }
        // }
    }

    pub fn resolve(value: MemoryMap) -> Self {
        let mut resolved = ResolvedMemoryMap::default();
        let mut defs = ResolvedMemoryMap::default();
        for def in value.def.iter() {
            match def {
                Composite::Entry(entry) => {}
                Composite::Array(array) => {}
                Composite::Cluster(cluster) => {
                    let table = defs.new_entry_table(cluster.name());
                }
                Composite::Reference { .. } => {}
                Composite::Map { .. } => {}
            }
        }
        resolved
    }
}

pub struct MemoryTableIter {
    entry_iter: <BTreeMap<String, Vec<ResolvedEntry>> as IntoIterator>::IntoIter,
    field_iter: <BTreeMap<String, Vec<Field>> as IntoIterator>::IntoIter,
    next_field: bool,
}

pub enum EntryOrField {
    Entry((String, Vec<ResolvedEntry>)),
    Field((String, Vec<Field>)),
}

impl Iterator for MemoryTableIter {
    type Item = EntryOrField;
    fn next(&mut self) -> Option<Self::Item> {
        match self.next_field {
            false => match self.entry_iter.next() {
                Some(table) => Some(EntryOrField::Entry(table)),
                None => {
                    self.next_field = true;
                    self.next()
                }
            },
            true => self.field_iter.next().map(EntryOrField::Field),
        }
    }
}

impl IntoIterator for ResolvedMemoryMap {
    type Item = EntryOrField;
    type IntoIter = MemoryTableIter;
    fn into_iter(self) -> Self::IntoIter {
        MemoryTableIter {
            entry_iter: self.entries.into_iter(),
            field_iter: self.fields.into_iter(),
            next_field: false,
        }
    }
}

impl Extend<EntryOrField> for ResolvedMemoryMap {
    fn extend<T: IntoIterator<Item = EntryOrField>>(&mut self, iter: T) {
        for item in iter {
            match item {
                EntryOrField::Entry(entry) => {
                    self.entries.insert(entry.0, entry.1);
                }
                EntryOrField::Field(field) => {
                    self.fields.insert(field.0, field.1);
                }
            }
        }
    }
}
