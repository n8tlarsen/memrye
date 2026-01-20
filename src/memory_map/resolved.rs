use crate::memory_map::{
    field::{FieldType, Value},
    Access,
};
use derive_more::Display;
use std::fmt;
use std::io::Write;
use tabled::Tabled;

use super::{Composite, DisplayOption, Field, MemoryMap};

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

pub struct SectionTable<T> {
    name: String,
    table: Vec<T>,
}

impl<T> SectionTable<T> {
    pub fn new(name: &str) -> Self {
        SectionTable {
            name: name.to_string(),
            table: Vec::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        self.table.push(value)
    }
}

#[derive(Default)]
pub struct ResolvedMemoryMap {
    entries: Vec<SectionTable<ResolvedEntry>>,
    fields: Vec<SectionTable<Field>>,
}

impl ResolvedMemoryMap {
    pub fn new_entry_table(&mut self, name: &str) -> &mut SectionTable<ResolvedEntry> {
        self.entries.push(SectionTable::new(name));
        self.entries.last_mut().unwrap()
    }

    pub fn new_field_table(&mut self, name: &str) -> &mut SectionTable<Field> {
        self.fields.push(SectionTable::new(name));
        self.fields.last_mut().unwrap()
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
}

impl From<MemoryMap> for ResolvedMemoryMap {
    fn from(value: MemoryMap) -> Self {
        let resolved = ResolvedMemoryMap::default();
        let defs = ResolvedMemoryMap::default();
        for def in value.def.iter() {
            // if let Composite::Entry(entry) = def {
            //     defs.
            // } else {
            // }
        }
        resolved
    }
}
