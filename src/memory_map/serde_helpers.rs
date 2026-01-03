use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::de::{Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::schemars_1::JsonSchemaAs;
use serde_with::{serde_as, DeserializeAs, DisplayFromStr, SerializeAs};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

pub struct IntegerOrString;

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

pub struct HexStrOrUnsigned;

impl SerializeAs<u64> for HexStrOrUnsigned {
    fn serialize_as<S>(source: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(*source)
    }
}

impl<'de> DeserializeAs<'de, u64> for HexStrOrUnsigned {
    fn deserialize_as<D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
    {
        fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct HexVisitor;

            impl Visitor<'_> for HexVisitor {
                type Value = u64;

                fn expecting(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                    fmt.write_str("\"0x\" prefixed hex string or u64")
                }

                fn visit_i64<E>(self, val: i64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    if val >= 0 {
                        Ok(val as u64)
                    } else {
                        Err(E::invalid_value(Unexpected::Signed(val), &self))
                    }
                }

                fn visit_u64<E>(self, val: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(val)
                }

                fn visit_str<E>(self, val: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    let error = E::invalid_value(Unexpected::Str(val), &self);
                    if let Some(stripped) = val.strip_prefix("0x") {
                        let deformat = stripped.to_string().replace("_", "");
                        match u64::from_str_radix(&deformat, 16) {
                            Ok(parsed_int) => Ok(parsed_int),
                            Err(_) => Err(error),
                        }
                    } else {
                        Err(error)
                    }
                }
            }
            deserializer.deserialize_any(HexVisitor)
        }

        deserialize(deserializer)
    }
}

impl JsonSchemaAs<u64> for HexStrOrUnsigned {
    fn schema_name() -> Cow<'static, str> {
        "HexStrOrUnsigned".into()
    }
    fn schema_id() -> Cow<'static, str> {
        "HexStrOrUnsigned".into()
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
#[cfg_attr(test, derive(PartialEq))]
pub struct EnumMap(#[serde_as(as = "BTreeMap<DisplayFromStr, _>")] pub BTreeMap<u64, String>);

impl fmt::Display for EnumMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some((&max_key, _)) = self.0.last_key_value() {
            match max_key {
                0..2 => {
                    for (key, value) in self.0.iter() {
                        write!(f, "0b{key:b}: {value}<br>")?;
                    }
                    return Ok(());
                }
                2..16 => {
                    for (key, value) in self.0.iter() {
                        write!(f, "0x{key:X}: {value}<br>")?;
                    }
                    return Ok(());
                }
                _ => {
                    let width = (max_key as f64).log(16f64).ceil() as usize;
                    for (key, value) in self.0.iter() {
                        write!(f, "0x{key:0width$X}: {value}<br>", width = width)?;
                    }
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}
