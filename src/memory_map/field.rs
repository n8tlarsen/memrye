use crate::memory_map::{Access, EnumMap, HexStrOrUnsigned};
use anyhow::anyhow;
use derive_more::Display;
use log::{error, info};
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use serde_with::serde_as;

#[derive(Deserialize, Serialize, JsonSchema, Display, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// Single bit field
    /// Represented by the vhdl type `std_logic`
    #[display("std_logic")]
    Bit,
    /// Enumerated type
    /// Represented by the vhdl type `std_logic_vector(length-1 downto 0)`
    #[display("std_logic_vector({} downto 0):<br>{}", length-1, map)]
    Enum { length: u32, map: EnumMap },
    /// Unsigned numeric type; value is length of the field in bits.
    /// Defined by length and representing the vhdl type `signed(length-1 downto 0)`.
    #[display("unsigned({} downto 0)", _0-1)]
    Unsigned(u32),
    /// Signed numeric type; value is length of the field in bits.
    /// Defined by length and representing the vhdl type `unsigned(length-1 downto 0)`
    #[display("signed({} downto 0)", _0-1)]
    Signed(u32),
    /// Unsigned fixed point numeric type.
    /// Defined by the high and low subscripts typically representing the vhdl type
    /// `ufixed(high downto low)`.
    /// The binary point is located `low` places from the least significant digit.
    /// For exxample:
    /// ```toml
    /// ufixed.high = 11
    /// ufixed.low  = -4
    /// ```
    /// equates to:
    /// ``` vhdl
    /// ufixed(11 downto -4)
    /// ```
    /// and results in the binary fixed point form 000000000000.0000 with a resolution of
    /// 2^{-4}, a maximum value of (2^16 - 1) / (2^4), and a minimum value of 0.
    #[display("ufixed({} downto {})", high, low)]
    UFixed { high: i32, low: i32 },
    /// Signed fixed point numeric type.
    /// Defined by the high and low subscripts typically representing the vhdl type
    /// `sfixed(high downto low)`.
    /// The binary point is located `low` places from the least significant digit.
    /// For exxample:
    /// ```toml
    /// sfixed.high = 11
    /// sfixed.low  = -4
    /// ```
    /// equates to:
    /// ``` vhdl
    /// sfixed(11 downto -4)
    /// ```
    /// and results in the binary fixed point form 000000000000.0000 with a resolution of
    /// 2^{-4}, a maximum value of (2^{16-1} - 1) / (2^4), and a minimum value of
    /// -(2^{16-1}) / (2^4).
    #[display("sfixed({} downto {})", high, low)]
    SFixed { high: i32, low: i32 },
    /// String type; value is the length of the string in bytes.
    #[display("string({} downto 1)", _0)]
    String(u32),
}

fn ascii_only_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    if string.is_ascii() {
        Ok(string)
    } else {
        Err(D::Error::custom(format!(
            "string {} contains non-ascii characters",
            string
        )))
    }
}

#[derive(Deserialize, Serialize, JsonSchema, Display, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged)]
pub enum Value {
    #[serde(deserialize_with = "ascii_only_string")]
    String(String),
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
}

#[serde_as]
#[derive(Deserialize, Serialize, JsonSchema, Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Field {
    name: String,
    /// Bit offset from the beginning of the entry.
    /// If no offset is provided, elaboration assumes the field is packed directly following the
    /// previously defined field. If no prior field exists, elaboration assumes the field exists
    /// at offset zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<HexStrOrUnsigned>")]
    offset: Option<u64>,
    /// Field accessibility.
    /// If no accessibility is specified, elaboration assumes the field inherits
    /// access from its parent context
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    /// Field type
    #[serde(flatten)]
    field_type: FieldType,
    /// The default value of the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Value>,
    /// The unit of measurement of a numeric type. Ignored for other types.
    #[serde(skip_serializing_if = "Option::is_none")]
    unit: Option<String>,
    /// The minimum allowed value of a numeric type. Ignored for other types.
    #[serde(skip_serializing_if = "Option::is_none")]
    min: Option<f64>,
    /// The maximum allowed value of a numeric type. Ignored for other types.
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<f64>,
}

impl Field {
    pub fn length(&self) -> u32 {
        match self.field_type {
            FieldType::Bit => 1,
            FieldType::Enum { length, .. } => length,
            FieldType::Unsigned(length) => length,
            FieldType::Signed(length) => length,
            FieldType::UFixed { high, low } => (high - low + 1) as u32,
            FieldType::SFixed { high, low } => (high - low + 1) as u32,
            FieldType::String(length) => length * 8u32,
        }
    }

    pub fn get_offset(&self) -> Option<u64> {
        self.offset
    }

    pub fn set_offset(&mut self, value: u64) -> Result<(), anyhow::Error> {
        if self.offset.is_none() {
            self.offset = Some(value);
            Ok(())
        } else {
            let error = anyhow!("Internal error. Attempted to overwrite provided field \"offset\"");
            error!("{}", error);
            Err(error)
        }
    }

    pub fn resolve_value(&mut self) -> Result<(), anyhow::Error> {
        let result = match self.field_type {
            FieldType::String(ref length) => self.resolve_field_type_string(length),
            FieldType::Enum {
                ref length,
                ref map,
            } => self.resolve_field_type_enum(length, map),
            FieldType::Bit => self.resolve_field_type_bit(),
            FieldType::Unsigned(ref length) => self.resolve_field_type_unsigned(length),
            FieldType::Signed(ref length) => self.resolve_field_type_signed(length),
            FieldType::UFixed { ref high, ref low } => self.resolve_field_type_ufixed(high, low),
            FieldType::SFixed { ref high, ref low } => self.resolve_field_type_sfixed(high, low),
        };
        match result {
            Ok(update_value) => {
                if update_value.is_some() {
                    self.value = update_value;
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn resolve_field_type_string(&self, length: &u32) -> Result<Option<Value>, anyhow::Error> {
        if let Some(value) = &self.value {
            if let Value::String(string) = value {
                if (string.len() as u32) > *length {
                    let error = anyhow!("Provided string value is longer than the field type");
                    error!("{}", error);
                    Err(error)
                } else {
                    Ok(None)
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                Err(error)
            }
        } else {
            Ok(Some(Value::String("".to_string())))
        }
    }

    fn resolve_field_type_enum(
        &self,
        length: &u32,
        map: &EnumMap,
    ) -> Result<Option<Value>, anyhow::Error> {
        if let Some(value) = &self.value {
            match value {
                Value::Unsigned(number) => {
                    if *number > 2u64.pow(*length) - 1 {
                        let error = anyhow!(format!(
                                "Numeric value {} requires more than {} bits specified by the field type",
                                *number, *length
                            ));
                        error!("{}", error);
                        return Err(error);
                    }
                    if !(map.0.contains_key(number)) {
                        let error = anyhow!(format!(
                                "Numeric value {} is not a value specified by the enum field type of field {}",
                                *number, self.name
                            ));
                        error!("{}", error);
                        return Err(error);
                    }
                    Ok(None)
                }
                Value::String(string) => {
                    if !(map.0.values().any(|x| *x == *string)) {
                        let error = anyhow!(format!(
                                "String value {} is not a key specified by the enum field type of field {}",
                                *string, self.name
                            ));
                        error!("{}", error);
                        Err(error)
                    } else {
                        Ok(None)
                    }
                }
                _ => {
                    let error = anyhow!(format!(
                        "Provided value {} doesn't match the field type {}",
                        value, &self.field_type
                    ));
                    error!("{}", error);
                    Err(error)
                }
            }
        } else {
            // Default to the minimum enum in the map
            if let Some((_min_key, min_value)) = map.0.first_key_value() {
                Ok(Some(Value::String(min_value.clone())))
            } else {
                Ok(Some(Value::String(String::default())))
            }
        }
    }

    fn resolve_field_type_bit(&self) -> Result<Option<Value>, anyhow::Error> {
        if let Some(value) = &self.value {
            if let Value::Bool(..) = value {
                Ok(None)
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                Err(error)
            }
        } else {
            info!("Field {} value not provided, set to 0", self.name);
            Ok(Some(Value::Bool(false)))
        }
    }

    fn resolve_field_type_unsigned(&self, length: &u32) -> Result<Option<Value>, anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            if let Value::Unsigned(number) = value {
                if *number > 2u64.pow(*length) - 1 {
                    let error = anyhow!(format!(
                        "Numeric value {} requires more than {} bits specified by the field type",
                        *number, *length
                    ));
                    error!("{}", error);
                    Err(error)
                } else {
                    Ok(None)
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                Err(error)
            }
        } else {
            Ok(Some(Value::Unsigned(0)))
        }
    }

    fn resolve_field_type_signed(&self, length: &u32) -> Result<Option<Value>, anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            if let Value::Signed(number) = value {
                if (*number > 2i64.pow(*length - 1) - 1) || (*number < -2i64.pow(*length - 1)) {
                    let error = anyhow!(format!(
                        "Numeric value {} requires more than {} bits specified by the field type",
                        *number, *length
                    ));
                    error!("{}", error);
                    Err(error)
                } else {
                    Ok(None)
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                Err(error)
            }
        } else {
            Ok(Some(Value::Signed(0)))
        }
    }

    fn resolve_field_type_ufixed(
        &self,
        high: &i32,
        low: &i32,
    ) -> Result<Option<Value>, anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            // TODO: Allow unsigned conversion to float
            if let Value::Float(number) = value {
                let max = 2f64.powf(*high as f64) - 2f64.powf(*low as f64);
                if (*number > max) || (*number < 0f64) {
                    let error = anyhow!(format!(
                        "Numeric value {} cannot be represented by the field type {}",
                        *number, &self.field_type
                    ));
                    error!("{}", error);
                    Err(error)
                } else {
                    Ok(None)
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type unsigned",
                    value,
                ));
                error!("{}", error);
                Err(error)
            }
        } else {
            Ok(Some(Value::Float(0f64)))
        }
    }

    fn resolve_field_type_sfixed(
        &self,
        high: &i32,
        low: &i32,
    ) -> Result<Option<Value>, anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            if let Value::Float(number) = value {
                let max = 2f64.powf((*high - 1) as f64) - 2f64.powf(*low as f64);
                let min = -2f64.powf((*high - 1) as f64);
                if (*number > max) || (*number < min) {
                    let error = anyhow!(format!(
                        "Numeric value {} cannot be represented by the field type {}",
                        *number, &self.field_type
                    ));
                    error!("{}", error);
                    Err(error)
                } else {
                    Ok(None)
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type unsigned",
                    value,
                ));
                error!("{}", error);
                Err(error)
            }
        } else {
            Ok(Some(Value::Float(0f64)))
        }
    }
}
