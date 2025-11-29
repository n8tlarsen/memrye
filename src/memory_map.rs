pub mod field;
pub mod memory_map;
pub mod protocol;

pub use field::Field;
pub use memory_map::MemoryMap;
pub use protocol::Protocol;

#[cfg(test)]
use serde::de::value::{Error as ValueError, I64Deserializer, StrDeserializer};
#[cfg(test)]
use serde::de::IntoDeserializer;
use serde::de::{Unexpected, Visitor};
use serde::Deserializer;
use std::fmt;

fn hex_str_or_unsigned<'de, D>(deserializer: D) -> Result<u64, D::Error>
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

#[test]
fn test_hex_str_ok() {
    let deserializer: StrDeserializer<ValueError> = "0xffff".into_deserializer();
    assert_eq!(hex_str_or_unsigned(deserializer), Ok(65535));
}

#[test]
fn test_hex_str_err() {
    let deserializer: StrDeserializer<ValueError> = "ffff".into_deserializer();
    assert_eq!(
        hex_str_or_unsigned(deserializer).unwrap_err().to_string(),
        "invalid value: string \"ffff\", expected \"0x\" prefixed hex string or u64"
    );
}

#[test]
fn test_negative_i64_err() {
    let deserializer: I64Deserializer<ValueError> = (-1i64).into_deserializer();
    assert_eq!(
        hex_str_or_unsigned(deserializer).unwrap_err().to_string(),
        "invalid value: integer `-1`, expected \"0x\" prefixed hex string or u64"
    );
}

#[test]
fn test_positive_i64_ok() {
    let deserializer: I64Deserializer<ValueError> = (1i64).into_deserializer();
    assert_eq!(hex_str_or_unsigned(deserializer), Ok(1));
}

fn maybe_hex_str_or_unsigned<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Some(hex_str_or_unsigned(deserializer)?))
}
