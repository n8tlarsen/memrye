use super::{Composite, Protocol};
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use serde_with::{formats::PreferOne, serde_as, OneOrMany};
use std::io::Write;

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema)]
pub struct MemoryMap {
    #[serde(flatten)]
    protocol: Protocol,
    #[serde(rename = "@map")]
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    map: Vec<Composite>,
    #[serde(rename = "@def")]
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    def: Vec<Composite>,
}

impl MemoryMap {
    pub fn resolve(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
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

pub fn get_memory_map_schema() -> String {
    let schema = schema_for!(MemoryMap);
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    serde::Serialize::serialize(&schema, &mut ser).expect("Failed to serialize schema");
    String::from_utf8(buf).expect("Failed to convert serial buffer to string")
}
