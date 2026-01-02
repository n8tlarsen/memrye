use crate::memory_map::{maybe_hex_str_or_unsigned, Access, Field};
use schemars::JsonSchema;
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::schemars_1::JsonSchemaAs;
use serde_with::{
    formats::PreferOne, serde_as, DefaultOnNull, DeserializeAs, OneOrMany, SerializeAs,
};
use std::borrow::Cow;
use std::fmt;

struct IntegerOrString;

impl SerializeAs<String> for IntegerOrString {
    fn serialize_as<S>(source: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(source)
    }
}

impl<'de> DeserializeAs<'de, String> for IntegerOrString {
    fn deserialize_as<D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct IntegerOrStringVisitor;

            impl Visitor<'_> for IntegerOrStringVisitor {
                type Value = String;

                fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt.write_str("integer or string")
                }

                fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(val.to_string())
                }

                fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(val.to_string())
                }
            }

            deserializer.deserialize_any(IntegerOrStringVisitor)
        }

        deserialize(deserializer)
    }
}

impl JsonSchemaAs<String> for IntegerOrString {
    fn schema_name() -> Cow<'static, str> {
        "IntegerOrString".into()
    }
    fn schema_id() -> Cow<'static, str> {
        "IntegerOrString".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": ["integer", "string"],
            "format": ["uint64", "int64"],
        })
    }
    fn inline_schema() -> bool {
        true
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(untagged)]
pub enum Index {
    Length(u64),
    Range { high: u64, low: u64 },
    List(#[serde_as(as = "OneOrMany<Option<IntegerOrString>, PreferOne>")] Vec<Option<String>>),
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Array {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    elements: Box<Composite>,
    index: Index,
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    increment: Option<u64>,
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Cluster {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    #[serde_as(as = "OneOrMany<_,PreferOne>")]
    elements: Vec<Composite>,
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
pub struct Entry {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    address: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    /// Length of the entry in bytes
    bytes: u32,
    #[serde(default)]
    #[serde_as(as = "DefaultOnNull<OneOrMany<_,PreferOne>>")]
    fields: Vec<Field>,
}

#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
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
