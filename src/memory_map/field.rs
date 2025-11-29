use crate::memory_map::{maybe_hex_str_or_unsigned, Protocol};
use anyhow::anyhow;
use derive_more::Display;
use log::error;
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum BitfieldPattern {
    /// Contiguous array of bit names starting at index 0.
    /// If array length is shorter than the field, the remainging bits are marked as 'Reserved'
    FromZero(Vec<String>),
    /// Discrete key-value pairs of bit names and indices
    Discrete(HashMap<String, u64>),
}

#[derive(Deserialize, Serialize, JsonSchema, Display)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// Group of other types, typically used to describe a contiguous block of registers
    Set,
    /// String type; value is the length of the string in bytes.
    #[display("string({} downto 1)", _0)]
    String(u64),
    /// Enumerated type
    /// Represented by the vhdl type `std_logic_vector(length-1 downto 0)`
    #[display("Enum length {}", length)]
    Enum {
        length: u32,
        map: HashMap<String, u64>,
    },
    /// Bitfield with named indices
    /// Represented by the vhdl type `std_logic_vector(length-1 downto 0)`
    #[display("Bitfield length {}", length)]
    Bitfield { length: u32, bits: BitfieldPattern },
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
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum OneOrMoreField {
    One(Box<Field>),
    More(Vec<Field>),
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

#[derive(Deserialize, Serialize, JsonSchema, Display)]
#[serde(untagged)]
pub enum Value {
    #[serde(deserialize_with = "ascii_only_string")]
    String(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
}

#[derive(Deserialize, Serialize, JsonSchema, Display, Default, Debug, Copy, Clone)]
pub enum Access {
    /// Read-only access is permitted
    #[default]
    #[serde(rename = "r")]
    #[display("Read-only")]
    Read,
    /// Write-only access is permitted
    #[serde(rename = "w")]
    #[display("Write-only")]
    Write,
    /// Both read and write access is permitted
    #[serde(rename = "rw")]
    #[display("Read/Write")]
    ReadWrite,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct Field {
    name: String,
    /// Memory address. If no address is provided, elaboration assumes the field
    /// is packed directly following the previously defined address. If padding is desired to
    /// ensure allignment to Protocol.address_align, and the data type is smaller than address_align, it is
    /// required to explicitly specify the address. If no prior field exists, elaboration
    /// either inherits the address of the parent FieldType::Set or starts at zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "maybe_hex_str_or_unsigned")]
    address: Option<u64>,
    /// Register access permission.
    /// If no access permission is specified, elaboration assumes the field inherits
    /// access from its parent context.
    #[serde(skip_serializing_if = "Option::is_none")]
    access: Option<Access>,
    /// Field type
    #[serde(rename = "type")]
    field_type: FieldType,
    /// A single field object or an array of field objects. Used only when Field.FieldType is
    /// FieldType::Set.
    #[serde(skip_serializing_if = "Option::is_none")]
    contains: Option<OneOrMoreField>,
    /// The default value of the field. Ignored for FieldType::Set
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
    pub fn elaborate(&mut self, protocol: &Protocol) -> Result<(), anyhow::Error> {
        self.elaborate_recursive(
            protocol,
            &self.access.unwrap_or_default(),
            &mut self.address.unwrap_or_default(),
        )
    }

    fn elaborate_recursive(
        &mut self,
        protocol: &Protocol,
        parent_access: &Access,
        running_address: &mut u64,
    ) -> Result<(), anyhow::Error> {
        if let FieldType::Set = &self.field_type {
            if let Some(container) = &mut self.contains {
                match container {
                    OneOrMoreField::One(field) => {
                        (**field).elaborate_recursive(protocol, parent_access, running_address)
                    }
                    OneOrMoreField::More(fields) => {
                        for field in fields.iter_mut() {
                            (*field).elaborate_recursive(
                                protocol,
                                parent_access,
                                running_address,
                            )?;
                        }
                        Ok(())
                    }
                }
            } else {
                let error = anyhow!(
                    "Schema error. Field type 'set' was provided, but key 'contains' was not"
                );
                error!("{}", error);
                Err(error)
            }
        } else {
            let byte_length = match &self.field_type {
                FieldType::String(length) => {
                    self.validate_field_type_string(length)?;
                    *length
                }
                FieldType::Enum { length, map } => {
                    self.validate_field_type_enum(length, map)?;
                    ((*length as f64) / 8f64).ceil() as u64
                }
                FieldType::Bitfield { length, bits: _ } => {
                    self.validate_field_type_bitfield(length)?;
                    ((*length as f64) / 8f64).ceil() as u64
                }
                FieldType::Unsigned(length) => {
                    self.validate_field_type_unsigned(length)?;
                    ((*length as f64) / 8f64).ceil() as u64
                }
                FieldType::Signed(length) => {
                    self.validate_field_type_signed(length)?;
                    ((*length as f64) / 8f64).ceil() as u64
                }
                FieldType::UFixed { high, low } => {
                    self.validate_field_type_ufixed(high, low)?;
                    (((*high - *low + 1) as f64) / 8f64).ceil() as u64
                }
                FieldType::SFixed { high, low } => {
                    self.validate_field_type_sfixed(high, low)?;
                    (((*high - *low + 1) as f64) / 8f64).ceil() as u64
                }
                FieldType::Set => {
                    panic!() // This case is handled above so arriving here is impossible
                }
            };
            if self.access.is_none() {
                self.access = Some(*parent_access)
            }
            self.elaborate_address(&byte_length, protocol, running_address)
        }
    }

    /// Elaborates the address field and updates the running address.
    /// If no address is provided, the function will assume the field is packed directly following
    /// the previously defined address. If padding is desired to ensure allignment to
    /// Protocol.address_align, and the data type is smaller than address_align, it is required to
    /// explicitly specify the address.
    fn elaborate_address(
        &mut self,
        byte_length: &u64,
        protocol: &Protocol,
        running_address: &mut u64,
    ) -> Result<(), anyhow::Error> {
        if self.address.is_none() {
            self.address = Some(*running_address);
        }
        let my_address = &self.address.unwrap();
        let modulo = *byte_length % protocol.address_unit;
        let field_length = if modulo == 0 {
            *byte_length
        } else {
            *byte_length + (protocol.address_unit - modulo)
        };
        if (*my_address + field_length - 1) > protocol.address_max {
            let error = anyhow!(format!(
                "Field {} with address {} and length {} would overflow the protocol maximum address {}",
                self.name,
                *my_address,
                field_length,
                protocol.address_max,
            ));
            error!("{}", error);
            return Err(error);
        }
        *running_address = *my_address + field_length;
        Ok(())
    }

    fn validate_field_type_string(&self, length: &u64) -> Result<(), anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            if let Value::String(string) = value {
                if (string.len() as u64) > *length {
                    let error = anyhow!("Provided string value is longer than the field type");
                    error!("{}", error);
                    return Err(error);
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                return Err(error);
            }
        }
        Ok(())
    }

    fn validate_field_type_enum(
        &self,
        length: &u32,
        map: &HashMap<String, u64>,
    ) -> Result<(), anyhow::Error> {
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
                    if !(map.values().any(|&x| x == *number)) {
                        let error = anyhow!(format!(
                            "Numeric value {} is not a value specified by the enum field type of field {}",
                            *number, self.name
                        ));
                        error!("{}", error);
                        return Err(error);
                    }
                }
                Value::String(string) => {
                    if !(map.contains_key(string)) {
                        let error = anyhow!(format!(
                            "String value {} is not a key specified by the enum field type of field {}",
                            *string, self.name
                        ));
                        error!("{}", error);
                        return Err(error);
                    }
                }
                _ => {
                    let error = anyhow!(format!(
                        "Provided value {} doesn't match the field type {}",
                        value, &self.field_type
                    ));
                    error!("{}", error);
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    fn validate_field_type_bitfield(&self, length: &u32) -> Result<(), anyhow::Error> {
        if let Some(value) = &self.value {
            if let Value::Unsigned(number) = value {
                if *number > 2u64.pow(*length) - 1 {
                    let error = anyhow!(format!(
                        "Numeric value {} requires more than {} bits specified by the field type",
                        *number, *length
                    ));
                    error!("{}", error);
                    return Err(error);
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                return Err(error);
            }
        }
        Ok(())
    }

    fn validate_field_type_unsigned(&self, length: &u32) -> Result<(), anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            if let Value::Unsigned(number) = value {
                if *number > 2u64.pow(*length) - 1 {
                    let error = anyhow!(format!(
                        "Numeric value {} requires more than {} bits specified by the field type",
                        *number, *length
                    ));
                    error!("{}", error);
                    return Err(error);
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                return Err(error);
            }
        }
        Ok(())
    }

    fn validate_field_type_signed(&self, length: &u32) -> Result<(), anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            if let Value::Signed(number) = value {
                if (*number > 2i64.pow(*length - 1) - 1) || (*number < -2i64.pow(*length - 1)) {
                    let error = anyhow!(format!(
                        "Numeric value {} requires more than {} bits specified by the field type",
                        *number, *length
                    ));
                    error!("{}", error);
                    return Err(error);
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type {}",
                    value, &self.field_type
                ));
                error!("{}", error);
                return Err(error);
            }
        }
        Ok(())
    }

    fn validate_field_type_ufixed(&self, high: &i32, low: &i32) -> Result<(), anyhow::Error> {
        // Validate the value and length
        if let Some(value) = &self.value {
            if let Value::Float(number) = value {
                let max = 2f64.powf(*high as f64) - 2f64.powf(*low as f64);
                if (*number > max) || (*number < 0f64) {
                    let error = anyhow!(format!(
                        "Numeric value {} cannot be represented by the field type {}",
                        *number, &self.field_type
                    ));
                    error!("{}", error);
                    return Err(error);
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type unsigned",
                    value,
                ));
                error!("{}", error);
                return Err(error);
            }
        }
        Ok(())
    }

    fn validate_field_type_sfixed(&self, high: &i32, low: &i32) -> Result<(), anyhow::Error> {
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
                    return Err(error);
                }
            } else {
                let error = anyhow!(format!(
                    "Provided value {} doesn't match the field type unsigned",
                    value,
                ));
                error!("{}", error);
                return Err(error);
            }
        }
        Ok(())
    }
}
