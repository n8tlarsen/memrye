use crate::memory_map::{
    field::{FieldType, Value},
    Access,
};
use derive_more::Display;
use std::fmt;
use std::io::Write;
use tabled::Tabled;

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

#[derive(Debug, Clone)]
struct DisplayOption<T>(Option<T>)
where
    T: fmt::Display + Default;

impl<T> fmt::Display for DisplayOption<T>
where
    T: fmt::Display + Default,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref value) = self.0 {
            write!(f, "{}", value)
        } else {
            write!(f, "")
        }
    }
}

#[derive(Tabled, Debug, Clone)]
#[tabled(rename_all = "Upper Title Case")]
pub struct ResolvedField {
    /// Bit offset from the beginning of the entry.
    offset: Range,
    /// Name of the field
    name: String,
    /// Field accessibility.
    access: Access,
    /// Field type
    #[tabled(rename = "Type")]
    field_type: FieldType,
    /// The unit of measurement of a numeric type. Ignored for other types.
    unit: DisplayOption<String>,
    /// The minimum allowed value of a numeric type. Ignored for other types.
    minimum: DisplayOption<f64>,
    /// The maximum allowed value of a numeric type. Ignored for other types.
    maximum: DisplayOption<f64>,
    /// The default value of the field.
    value: Value,
}

struct SectionTable<T> {
    name: String,
    table: Vec<T>,
}

pub struct ResolvedMemoryMap {
    entries: Vec<SectionTable<ResolvedEntry>>,
    fields: Vec<SectionTable<ResolvedField>>,
}

impl ResolvedMemoryMap {
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
