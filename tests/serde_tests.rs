use serde_json::json;
use std::collections::BTreeMap;
use toml::toml;
use vhdl_doc::memory_map::{DisplayOption, Field, MemoryMap};
use vhdl_doc::memory_map::{EnumMap, HexStrOrUnsigned, IntegerOrString};

#[test]
pub fn enum_map_json() {
    let content = json!({
        "0": "zero",
        "1": "one",
        "2": "two",
        "3": "three"
    })
    .to_string();
    let map: EnumMap = serde_json::from_str(&content).expect("Failed to parse JSON");
    let expected: EnumMap = EnumMap(BTreeMap::from([
        (0u64, "zero".to_string()),
        (1u64, "one".to_string()),
        (2u64, "two".to_string()),
        (3u64, "three".to_string()),
    ]));
    assert_eq!(map.0, expected.0);
}

#[test]
pub fn enum_map_toml() {
    let content = toml!(
        0 = "zero"
        1 = "one"
        2 = "two"
        3 = "three"
    )
    .to_string();
    let map: EnumMap = toml::from_str(&content).expect("Failed to parse TOML");
    let expected: EnumMap = EnumMap(BTreeMap::from([
        (0u64, "zero".to_string()),
        (1u64, "one".to_string()),
        (2u64, "two".to_string()),
        (3u64, "three".to_string()),
    ]));
    assert_eq!(map.0, expected.0);
}

#[test]
pub fn mm_empty() {
    let content = json!({
        "addressMax": "0xFFFF_FFFF",
        "addressUnit": 1,
        "addressAlign": 4,
        "&map" : []
    })
    .to_string();
    let memory_map: MemoryMap = serde_json::from_str(&content).expect("Failed to parse JSON");
    println!(
        "Empty Map:\n{}\n",
        serde_json::to_string_pretty(&memory_map).expect("Failed to serialize to JSON string")
    )
}

#[test]
pub fn mm_entry_minimal() {
    let content = json!({
        "addressMax": "0xFFFF_FFFF",
        "addressUnit": 1,
        "addressAlign": 4,
        "&map" : {
            "name": "Test Entry",
            "bytes": 1
        }
    })
    .to_string();
    let memory_map: MemoryMap = serde_json::from_str(&content).expect("Failed to parse JSON");
    println!(
        "Minimal Entry:\n{}\n",
        serde_json::to_string_pretty(&memory_map).expect("Failed to serialize to JSON string")
    )
}

#[test]
pub fn mm_entry_all() {
    let content = json!({
        "addressMax": "0xFFFF_FFFF",
        "addressUnit": 1,
        "addressAlign": 4,
        "&map" : {
            "name": "Test Entry",
            "address": "0x0000_1000",
            "access": "r",
            "bytes": 1,
            "fields": [
                {"name": "Test Field", "offset": 0, "unsigned": 4},
                {"name": "Test Enum", "offset": 4, "enum": {
                    "length": 4,
                    "map" : {
                        "0": "zero",
                        "1": "one",
                        "2": "two",
                        "3": "three"
                    }
                }}
            ]
        }
    })
    .to_string();
    let memory_map: MemoryMap = serde_json::from_str(&content).expect("Failed to parse JSON");
    println!(
        "Minimal Entry:\n{}\n",
        serde_json::to_string_pretty(&memory_map).expect("Failed to serialize to JSON string")
    )
}

#[test]
pub fn mm_array_all() {
    let content = json!({
        "addressMax": "0xFFFF_FFFF",
        "addressUnit": 1,
        "addressAlign": 4,
        "&map" : {
            "name": "Test Array",
            "address": "0x0000_1000",
            "access": "r",
            "elements": {
                "name": "Test Entry",
                "address": "0x0000_1000",
                "access": "r",
                "bytes": 1,
                "fields": [
                    {"name": "Test Field", "unsigned": 4},
                    {"name": "Test Enum", "enum": {
                        "length": 4,
                        "map" : {
                            "0": "zero",
                            "1": "one",
                            "2": "two",
                            "3": "three"
                        }
                    }}
                ]
            },
            "index": [1,2,null,null,5,6],
            "increment": 4
        }
    })
    .to_string();
    let memory_map: MemoryMap = serde_json::from_str(&content).expect("Failed to parse JSON");
    println!(
        "Minimal Entry:\n{}\n",
        serde_json::to_string_pretty(&memory_map).expect("Failed to serialize to JSON string")
    )
}
