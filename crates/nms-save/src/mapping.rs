//! Key deobfuscation for NMS save-file JSON.
//!
//! Some save files store obfuscated object keys such as `"F2P"` in place of
//! `"Version"`. [`KeyMapping`] combines the primary mapping, older-key
//! compatibility entries, and Save Wizard spelling fixups. Deobfuscation
//! traverses objects and arrays recursively, so nested keys may be renamed even
//! when their parent key is unknown. Unknown key spellings and scalar values
//! are otherwise preserved. If keys map to the same output name, the later
//! processed entry replaces the earlier value, so the result is not lossless.
//! This module handles JSON keys only; it does not decompress a save or validate
//! address fields.

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

/// Key mapping used to deobfuscate NMS save JSON.
///
/// The primary direction maps obfuscated keys such as `"F2P"` to readable
/// names such as `"Version"`. [`Self::bundled`] adds older keys only when the
/// primary mapping lacks them and loads separate Save Wizard spelling fixups
/// for later deobfuscation. Those fixups are not mapping entries returned by
/// [`Self::get`] or counted by [`Self::len`]. [`Self::from_json`] loads one
/// mapping file without those additional sources.
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
    /// Loads `mapping_mbincompiler.json` first, then adds keys from
    /// `mapping_legacy.json` only when the primary mapping lacks that key. It
    /// also loads separate spelling fixups from `mapping_savewizard.json` for
    /// use during later deobfuscation; they do not alter a JSON value here or
    /// count as entries returned by [`Self::get`] or [`Self::len`].
    ///
    /// # Examples
    ///
    /// ```
    /// let mapping = nms_save::KeyMapping::bundled();
    /// assert_eq!(mapping.get("F2P"), Some("Version"));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if any of the mapping files compiled into this crate is invalid.
    /// Those files are project data and are expected to be valid at build time.
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

    /// Parse one mapping file from JSON text.
    ///
    /// The file must contain `libMBIN_version` and a `Mapping` array whose
    /// entries have `Key` and `Value` strings. When a key occurs more than once,
    /// the first entry is retained. This constructor does not merge the bundled
    /// legacy keys or Save Wizard fixups.
    ///
    /// # Examples
    ///
    /// ```
    /// let mapping = nms_save::KeyMapping::from_json(
    ///     r#"{"libMBIN_version":"test","Mapping":[{"Key":"abc","Value":"Alpha"},{"Key":"abc","Value":"Wrong"}]}"#,
    /// )?;
    /// assert_eq!(mapping.get("abc"), Some("Alpha"));
    /// assert_eq!(mapping.len(), 1);
    /// let mut nested = serde_json::json!({"items": [{"abc": 1, "untouched": "literal"}]});
    /// mapping.deobfuscate(&mut nested);
    /// assert_eq!(nested, serde_json::json!({"items": [{"Alpha": 1, "untouched": "literal"}]}));
    /// # Ok::<(), nms_save::SaveError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SaveError::MappingParseError`] when the JSON is malformed or
    /// does not have the expected mapping-file shape.
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

    /// Read and parse one mapping file from disk.
    ///
    /// This has the same single-file behavior as [`Self::from_json`]; it does
    /// not add bundled compatibility entries or fixups.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the path cannot be read, or
    /// [`SaveError::MappingParseError`] if its contents are not a valid mapping.
    pub fn from_file(path: &Path) -> Result<Self, SaveError> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }

    /// Look up the readable name for an obfuscated key.
    ///
    /// Returns `None` when this mapping has no entry for `obfuscated`.
    pub fn get(&self, obfuscated: &str) -> Option<&str> {
        self.entries.get(obfuscated).map(|s| s.as_str())
    }

    /// Return the number of distinct obfuscated keys in this mapping.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether the mapping contains no obfuscated keys.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Deobfuscate keys in a [`serde_json::Value`] tree in place.
    ///
    /// Walks objects and arrays recursively. For each object key, applies the
    /// obfuscated-key mapping and then any spelling fixup. Unknown key spellings
    /// are retained, but their object or array values are still traversed and
    /// may contain renamed keys; scalar values remain unchanged. If two entries
    /// produce the same output name, the later processed entry replaces the
    /// earlier value, so the result is not lossless.
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

/// Heuristically detect obfuscated keys in a parsed JSON value.
///
/// Only the top-level object is inspected. `"F2P"` is a positive marker even if
/// `"Version"` is also present; otherwise, a top-level `"Version"` selects the
/// plaintext result. If neither is present, `"6f="` or `"8>q"` is treated as an
/// obfuscation marker. Non-object values return `false`.
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

/// Parse JSON bytes and deobfuscate them when the top-level shape looks obfuscated.
///
/// Returns the parsed value, unchanged when [`is_obfuscated`] returns `false`.
/// The heuristic is deliberately limited to the marker keys documented on
/// [`is_obfuscated`].
///
/// # Examples
///
/// ```
/// let mapping = nms_save::KeyMapping::bundled();
/// let value = nms_save::deobfuscate_json(br#"{"F2P":6726}"#, &mapping)?;
/// assert_eq!(value["Version"], 6726);
/// # Ok::<(), nms_save::SaveError>(())
/// ```
/// # Errors
///
/// Returns [`SaveError::JsonParseError`] when `json_bytes` is not valid JSON.
/// This function does not perform save-file decompression or typed
/// deserialization.
///
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
