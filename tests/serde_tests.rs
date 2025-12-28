use serde_json::json;
use std::fs;
use vhdl_doc::memory_map::{Field, MemoryMap};

// #[test]
// pub fn toml_to_json() {
//     let contents = fs::read_to_string("tests/assets/memory_map.toml").expect("Failed to read file");
//     let memory_map: MemoryMap = toml::from_str(&contents).expect("Failed to parse TOML");
//     println!(
//         "{}",
//         serde_json::to_string_pretty(&memory_map).expect("Failed to serialize to JSON string")
//     );
// }

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
            "increment": 4,
            "indexEnums": {
                "0": "zero",
                "1": "one",
                "2": "two",
                "3": "three"
            }
        }
    })
    .to_string();
    let memory_map: MemoryMap = serde_json::from_str(&content).expect("Failed to parse JSON");
    println!(
        "Minimal Entry:\n{}\n",
        serde_json::to_string_pretty(&memory_map).expect("Failed to serialize to JSON string")
    )
}

// #[test]
// pub fn json_to_toml() {
//     let contents = fs::read_to_string("tests/assets/memory_map.json").expect("Failed to read file");
//     let memory_map: MemoryMap = serde_json::from_str(&contents).expect("Failed to parse JSON");
// println!(
//     "{}",
//     toml::to_string_pretty(&memory_map).expect("Failed to serialize to TOML string")
// );
// }

// #[test]
// pub fn toml_eval() {
//     let contents = fs::read_to_string("tests/assets/memory_map.json").expect("Failed to read file");
//     let mut memory_map: MemoryMap = serde_json::from_str(&contents).expect("Failed to parse JSON");
//     memory_map
//         .elaborate()
//         .expect("Failed to elaborate memory map for TOML document.");
//     println!(
//         "{}",
//         serde_json::to_string_pretty(&memory_map).expect("Failed to serialize to JSON string")
//     );
// }
