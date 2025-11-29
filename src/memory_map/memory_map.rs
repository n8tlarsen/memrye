use super::{Field, Protocol};
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use std::io::Write;

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct MemoryMap {
    protocol: Protocol,
    #[serde(flatten)]
    field: Field,
}

impl MemoryMap {
    pub fn elaborate(&mut self) -> Result<(), anyhow::Error> {
        self.field.elaborate(&self.protocol)
    }

    pub fn render(&self) -> String {
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

pub fn get_memory_map_schema() -> String {
    let schema = schema_for!(MemoryMap);
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    serde::Serialize::serialize(&schema, &mut ser).expect("Failed to serialize schema");
    String::from_utf8(buf).expect("Failed to convert serial buffer to string")
}
