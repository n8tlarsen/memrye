use crate::memory_map::{maybe_hex_str_or_unsigned, Access, Field};
use schemars::JsonSchema;
use schemars::{Schema, SchemaGenerator};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::schemars_1::JsonSchemaAs;
use serde_with::{
    formats::PreferOne, serde_as, DefaultOnNull, DeserializeAs, OneOrMany, SerializeAs,
};
use std::borrow::Cow;
use std::fmt;

struct NumberOrString;

impl SerializeAs<String> for NumberOrString {
    fn serialize_as<S>(source: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(source)
    }
}

impl<'de> DeserializeAs<'de, String> for NumberOrString {
    fn deserialize_as<D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct NumberOrStringVisitor;

            impl Visitor<'_> for NumberOrStringVisitor {
                type Value = String;

                fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt.write_str("string or integer")
                }

                fn visit_i64<E>(self, val: i64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(val.to_string())
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

            deserializer.deserialize_any(NumberOrStringVisitor)
        }

        deserialize(deserializer)
    }
}

impl JsonSchemaAs<String> for NumberOrString {
    fn schema_name() -> Cow<'static, str> {
        <String as JsonSchema>::schema_name()
    }
    fn schema_id() -> Cow<'static, str> {
        <String as JsonSchema>::schema_id()
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <String as JsonSchema>::json_schema(generator)
    }
    fn inline_schema() -> bool {
        <String as JsonSchema>::inline_schema()
    }
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[serde(untagged)]
pub enum Index {
    Length(u64),
    Range {
        high: u64,
        low: u64,
    },
    List(
        #[serde_as(as = "OneOrMany<DefaultOnNull<Option<NumberOrString>>, PreferOne>")]
        Vec<Option<String>>,
    ),
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
