//! Key deobfuscation for NMS save file JSON.
//!
//! NMS save files (format 2002+) have obfuscated JSON keys (e.g., `"F2P"` instead
//! of `"Version"`). This module loads mapping files and recursively replaces
//! obfuscated keys with their readable equivalents.

use std::collections::HashMap;
use std::path::Path;

use crate::error::SaveError;

/// A single entry in the mapping JSON file.
#[derive(Debug, serde::Deserialize)]
struct MappingEntry {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Value")]
    value: String,
}

/// Top-level structure of a mapping JSON file.
#[derive(Debug, serde::Deserialize)]
struct MappingFile {
    #[serde(rename = "libMBIN_version")]
    version: String,
    #[serde(rename = "Mapping")]
    mapping: Vec<MappingEntry>,
}

/// Bidirectional key mapping for NMS save file deobfuscation.
///
/// Maps obfuscated 3-character keys (e.g., `"F2P"`) to readable names
/// (e.g., `"Version"`).
#[derive(Debug, Clone)]
pub struct KeyMapping {
    /// Version string from the primary mapping file.
    pub version: String,
    /// Obfuscated key -> readable name.
    entries: HashMap<String, String>,
    /// Readable-name fixups (e.g., "MultiTools" -> "Multitools").
    fixups: HashMap<String, String>,
}

impl KeyMapping {
    /// Load the bundled (compiled-in) mapping.
    ///
    /// Merges all three mapping sources:
    /// - `mapping_mbincompiler.json` (primary)
    /// - `mapping_legacy.json` (older keys)
    /// - `mapping_savewizard.json` (name fixups)
    pub fn bundled() -> Self {
        let primary_json = include_str!("../data/mapping_mbincompiler.json");
        let legacy_json = include_str!("../data/mapping_legacy.json");
        let savewizard_json = include_str!("../data/mapping_savewizard.json");

        let mut mapping = Self::from_json(primary_json)
            .expect("bundled mapping_mbincompiler.json should be valid");

        let legacy: MappingFile =
            serde_json::from_str(legacy_json).expect("bundled mapping_legacy.json should be valid");
        for entry in legacy.mapping {
            mapping.entries.entry(entry.key).or_insert(entry.value);
        }

        let savewizard: MappingFile = serde_json::from_str(savewizard_json)
            .expect("bundled mapping_savewizard.json should be valid");
        for entry in savewizard.mapping {
            mapping.fixups.insert(entry.key, entry.value);
        }

        mapping
    }

    /// Load a mapping from a JSON string (single mapping file).
    pub fn from_json(json: &str) -> Result<Self, SaveError> {
        let file: MappingFile =
            serde_json::from_str(json).map_err(|e| SaveError::MappingParseError {
                message: e.to_string(),
            })?;

        let mut entries = HashMap::with_capacity(file.mapping.len());
        for entry in file.mapping {
            entries.entry(entry.key).or_insert(entry.value);
        }

        Ok(Self {
            version: file.version,
            entries,
            fixups: HashMap::new(),
        })
    }

    /// Load a mapping from a file on disk.
    pub fn from_file(path: &Path) -> Result<Self, SaveError> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    /// Look up the readable name for an obfuscated key.
    pub fn get(&self, obfuscated: &str) -> Option<&str> {
        self.entries.get(obfuscated).map(|s| s.as_str())
    }

    /// Return the number of mapping entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether the mapping is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Deobfuscate all keys in a [`serde_json::Value`] tree in place.
    ///
    /// Walks the tree recursively. For each JSON object, replaces obfuscated
    /// keys with their readable equivalents. Unknown keys are preserved as-is.
    /// After key replacement, applies any fixups (e.g., case corrections).
    pub fn deobfuscate(&self, value: &mut serde_json::Value) {
        self.walk(value);
    }

    fn walk(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                let entries: Vec<(String, serde_json::Value)> = std::mem::take(map)
                    .into_iter()
                    .map(|(k, mut v)| {
                        self.walk(&mut v);
                        let mut new_key = self.entries.get(&k).cloned().unwrap_or(k);
                        if let Some(fixed) = self.fixups.get(&new_key) {
                            new_key = fixed.clone();
                        }
                        (new_key, v)
                    })
                    .collect();
                *map = serde_json::Map::from_iter(entries);
            }
            serde_json::Value::Array(arr) => {
                for item in arr.iter_mut() {
                    self.walk(item);
                }
            }
            _ => {}
        }
    }
}

/// Detect whether a parsed JSON value has obfuscated keys.
///
/// Checks the top-level object for known obfuscated keys (`"F2P"`, `"6f="`, `"8>q"`)
/// vs the plaintext `"Version"` key.
pub fn is_obfuscated(json: &serde_json::Value) -> bool {
    match json.as_object() {
        Some(map) => {
            if map.contains_key("F2P") {
                return true;
            }
            if map.contains_key("Version") {
                return false;
            }
            map.contains_key("6f=") || map.contains_key("8>q")
        }
        None => false,
    }
}

/// Deobfuscate JSON bytes: parse, detect obfuscation, apply mapping if needed.
///
/// Returns the (potentially deobfuscated) parsed JSON value.
pub fn deobfuscate_json(
    json_bytes: &[u8],
    mapping: &KeyMapping,
) -> Result<serde_json::Value, SaveError> {
    let mut value: serde_json::Value =
        serde_json::from_slice(json_bytes).map_err(|e| SaveError::JsonParseError {
            message: e.to_string(),
        })?;

    if is_obfuscated(&value) {
        mapping.deobfuscate(&mut value);
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn load_bundled_mapping() {
        let mapping = KeyMapping::bundled();
        assert!(
            mapping.len() > 1300,
            "expected 1300+ entries, got {}",
            mapping.len()
        );
        assert_eq!(mapping.version, "6.11.0.1");
    }

    #[test]
    fn lookup_known_keys() {
        let mapping = KeyMapping::bundled();
        assert_eq!(mapping.get("F2P"), Some("Version"));
        assert_eq!(mapping.get("6f="), Some("PlayerStateData"));
        assert_eq!(mapping.get("8>q"), Some("Platform"));
    }

    #[test]
    fn lookup_unknown_key() {
        let mapping = KeyMapping::bundled();
        assert_eq!(mapping.get("ZZZNOTAKEY"), None);
    }

    #[test]
    fn legacy_keys_merged() {
        let mapping = KeyMapping::bundled();
        assert_eq!(mapping.get("5Ta"), Some("AllowFriendBases"));
    }

    #[test]
    fn deobfuscate_simple() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({"F2P": 6726});
        mapping.deobfuscate(&mut value);
        assert_eq!(value, json!({"Version": 6726}));
    }

    #[test]
    fn deobfuscate_nested() {
        let mapping = KeyMapping::bundled();
        assert_eq!(mapping.get("Pk4"), Some("SaveName"));
        let mut value = json!({"6f=": {"Pk4": "MySave"}});
        mapping.deobfuscate(&mut value);
        assert_eq!(value, json!({"PlayerStateData": {"SaveName": "MySave"}}));
    }

    #[test]
    fn deobfuscate_array() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({
            "F2P": 6726,
            "items": [
                {"F2P": 1},
                {"F2P": 2}
            ]
        });
        mapping.deobfuscate(&mut value);
        assert_eq!(value["Version"], 6726);
        assert_eq!(value["items"][0]["Version"], 1);
        assert_eq!(value["items"][1]["Version"], 2);
    }

    #[test]
    fn deobfuscate_preserves_unknown_keys() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({"F2P": 6726, "UnknownKey123": "hello"});
        mapping.deobfuscate(&mut value);
        assert_eq!(value["Version"], 6726);
        assert_eq!(value["UnknownKey123"], "hello");
    }

    #[test]
    fn deobfuscate_already_plaintext() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({"Version": 6726, "PlayerStateData": {"SaveName": "MySave"}});
        mapping.deobfuscate(&mut value);
        assert_eq!(value["Version"], 6726);
        assert_eq!(value["PlayerStateData"]["SaveName"], "MySave");
    }

    #[test]
    fn savewizard_fixup() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({"MultiTools": [1, 2, 3]});
        mapping.deobfuscate(&mut value);
        assert!(
            value.get("Multitools").is_some(),
            "savewizard fixup should rename MultiTools to Multitools"
        );
        assert!(value.get("MultiTools").is_none());
    }

    #[test]
    fn is_obfuscated_with_f2p() {
        let value = json!({"F2P": 6726, "6f=": {}});
        assert!(is_obfuscated(&value));
    }

    #[test]
    fn is_obfuscated_with_version() {
        let value = json!({"Version": 6726, "PlayerStateData": {}});
        assert!(!is_obfuscated(&value));
    }

    #[test]
    fn is_obfuscated_no_version_key() {
        let value = json!({"8>q": "PC"});
        assert!(is_obfuscated(&value));
    }

    #[test]
    fn is_obfuscated_non_object() {
        let value = json!([1, 2, 3]);
        assert!(!is_obfuscated(&value));
    }

    #[test]
    fn is_obfuscated_empty_object() {
        let value = json!({});
        assert!(!is_obfuscated(&value));
    }

    #[test]
    fn deobfuscate_json_bytes() {
        let mapping = KeyMapping::bundled();
        let json_bytes = br#"{"F2P": 6726}"#;
        let value = deobfuscate_json(json_bytes, &mapping).unwrap();
        assert_eq!(value["Version"], 6726);
    }

    #[test]
    fn deobfuscate_json_already_plaintext() {
        let mapping = KeyMapping::bundled();
        let json_bytes = br#"{"Version": 6726}"#;
        let value = deobfuscate_json(json_bytes, &mapping).unwrap();
        assert_eq!(value["Version"], 6726);
    }

    #[test]
    fn deobfuscate_json_invalid_json() {
        let mapping = KeyMapping::bundled();
        let json_bytes = b"not json";
        let err = deobfuscate_json(json_bytes, &mapping).unwrap_err();
        assert!(matches!(err, SaveError::JsonParseError { .. }));
    }

    #[test]
    fn from_json_valid() {
        let json = r#"{"libMBIN_version":"1.0.0","Mapping":[{"Key":"abc","Value":"Alpha"}]}"#;
        let mapping = KeyMapping::from_json(json).unwrap();
        assert_eq!(mapping.version, "1.0.0");
        assert_eq!(mapping.get("abc"), Some("Alpha"));
        assert_eq!(mapping.len(), 1);
    }

    #[test]
    fn from_json_invalid() {
        let json = "not valid json";
        let err = KeyMapping::from_json(json).unwrap_err();
        assert!(matches!(err, SaveError::MappingParseError { .. }));
    }

    #[test]
    fn from_json_empty_mapping() {
        let json = r#"{"libMBIN_version":"1.0.0","Mapping":[]}"#;
        let mapping = KeyMapping::from_json(json).unwrap();
        assert!(mapping.is_empty());
    }
}
