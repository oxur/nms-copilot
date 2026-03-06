# Milestone 1.7 -- Key Deobfuscation (nms-save)

Load mapping files and replace obfuscated JSON keys with readable names.

## Overview

After LZ4 decompression (Milestone 1.5), the JSON has obfuscated keys (e.g., `"F2P"` instead of `"Version"`, `"6f="` instead of `"PlayerStateData"`). A set of mapping files provides the translation table. This milestone implements loading those mappings and recursively walking a parsed `serde_json::Value` tree to replace obfuscated keys with their readable equivalents.

Reference: `libNOM.map/Mapping_Deobfuscation.cs:56-140`, `libNOM.map/Mapping.cs:79-98`.

---

## Mapping File Format

All three mapping files share the same JSON structure:

```json
{
  "libMBIN_version": "6.11.0.1",
  "Mapping": [
    { "Key": "F2P", "Value": "Version" },
    { "Key": "6f=", "Value": "PlayerStateData" },
    ...
  ]
}
```

The `Key` field is the obfuscated key; the `Value` field is the readable name.

### Source files

Located at `/Users/oubiwann/lab/oxur/nms-copilot/workbench/libNOM.map/libNOM.map/Resources/`:

| File | Size | Entries | Notes |
|------|------|---------|-------|
| `mapping_mbincompiler.json` | ~54KB | ~2000+ | Primary mapping, latest game version |
| `mapping_legacy.json` | ~1.9KB | ~47 | Older keys from pre-Beyond versions |
| `mapping_savewizard.json` | ~81B | 1 | Single correction: `"MultiTools"` -> `"Multitools"` |

All three must be merged. The savewizard mapping is special: its `Key` is already a readable name that needs to be normalized (case correction). Apply it as a post-processing fixup on the readable names.

### Merge order

1. Load `mapping_mbincompiler.json` as the primary mapping.
2. Load `mapping_legacy.json` and add any entries whose `Key` does not already exist in the primary mapping.
3. The savewizard mapping (`"MultiTools"` -> `"Multitools"`) is a readable-name fixup. After deobfuscation with the main mapping, apply this as a second pass: if any key equals `"MultiTools"`, rename it to `"Multitools"`.

---

## Collision Handling

There is one known collision in the mapping: the obfuscated key `"NE3"` maps to different readable names depending on JSON path context. In libNOM.map, this is resolved by checking the parent path.

For Phase 1: ignore collisions. Use the first mapping entry encountered. This only affects one key (`"NE3"`) and only matters for save editing (write path), not for reading.

---

## Types

### MappingEntry (serde deserialization helper)

```rust
/// A single entry in the mapping JSON file.
#[derive(Debug, serde::Deserialize)]
struct MappingEntry {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Value")]
    value: String,
}
```

### MappingFile (serde deserialization helper)

```rust
/// Top-level structure of a mapping JSON file.
#[derive(Debug, serde::Deserialize)]
struct MappingFile {
    #[serde(rename = "libMBIN_version")]
    version: String,
    #[serde(rename = "Mapping")]
    mapping: Vec<MappingEntry>,
}
```

### KeyMapping (public API)

```rust
use std::collections::HashMap;
use std::path::Path;

/// Bidirectional key mapping for NMS save file deobfuscation.
///
/// Maps obfuscated 3-character keys (e.g., `"F2P"`) to readable names
/// (e.g., `"Version"`).
#[derive(Debug, Clone)]
pub struct KeyMapping {
    /// Version string from the primary mapping file (e.g., "6.11.0.1").
    pub version: String,

    /// Obfuscated key -> readable name.
    entries: HashMap<String, String>,

    /// Readable-name fixups (e.g., "MultiTools" -> "Multitools").
    fixups: HashMap<String, String>,
}
```

---

## Functions

### KeyMapping methods

```rust
impl KeyMapping {
    /// Load the bundled (compiled-in) mapping.
    ///
    /// Merges all three mapping sources:
    /// - mapping_mbincompiler.json (primary)
    /// - mapping_legacy.json (older keys)
    /// - mapping_savewizard.json (name fixups)
    pub fn bundled() -> Self {
        let primary_json = include_str!("../data/mapping_mbincompiler.json");
        let legacy_json = include_str!("../data/mapping_legacy.json");
        let savewizard_json = include_str!("../data/mapping_savewizard.json");

        let mut mapping = Self::from_json(primary_json)
            .expect("bundled mapping_mbincompiler.json should be valid");

        // Merge legacy entries (only add keys not already present).
        let legacy: MappingFile = serde_json::from_str(legacy_json)
            .expect("bundled mapping_legacy.json should be valid");
        for entry in legacy.mapping {
            mapping.entries.entry(entry.key).or_insert(entry.value);
        }

        // Load savewizard as fixups (readable-name corrections).
        let savewizard: MappingFile = serde_json::from_str(savewizard_json)
            .expect("bundled mapping_savewizard.json should be valid");
        for entry in savewizard.mapping {
            mapping.fixups.insert(entry.key, entry.value);
        }

        mapping
    }

    /// Load a mapping from a JSON string (single mapping file).
    ///
    /// This parses a file with the standard `{ "libMBIN_version": ..., "Mapping": [...] }` format.
    pub fn from_json(json: &str) -> Result<Self, SaveError> {
        let file: MappingFile = serde_json::from_str(json)
            .map_err(|e| SaveError::MappingParseError {
                message: e.to_string(),
            })?;

        let mut entries = HashMap::with_capacity(file.mapping.len());
        for entry in file.mapping {
            // First entry wins (ignore collisions).
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
    ///
    /// Returns `None` if the key is not in the mapping (it may already be
    /// deobfuscated or simply unknown).
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

    /// Deobfuscate all keys in a `serde_json::Value` tree in place.
    ///
    /// Walks the tree recursively. For each JSON object, replaces obfuscated
    /// keys with their readable equivalents. Unknown keys are preserved as-is.
    /// After key replacement, applies any fixups (e.g., case corrections).
    pub fn deobfuscate(&self, value: &mut serde_json::Value) {
        self.walk(value);
    }

    /// Recursive tree walker.
    fn walk(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                // Collect all entries, transform keys, rebuild the map.
                let entries: Vec<(String, serde_json::Value)> = std::mem::take(map)
                    .into_iter()
                    .map(|(k, mut v)| {
                        self.walk(&mut v);
                        let mut new_key = self
                            .entries
                            .get(&k)
                            .cloned()
                            .unwrap_or(k);
                        // Apply fixups to the (now readable) key.
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
            _ => {} // Scalars: no keys to replace.
        }
    }
}
```

### Auto-detection

```rust
/// Detect whether a parsed JSON value has obfuscated keys.
///
/// Checks the top-level object for the presence of `"F2P"` (obfuscated version key)
/// vs `"Version"` (plaintext version key).
///
/// Returns `true` if the JSON appears to be obfuscated.
pub fn is_obfuscated(json: &serde_json::Value) -> bool {
    match json.as_object() {
        Some(map) => {
            // If we see the obfuscated version key, it's obfuscated.
            if map.contains_key("F2P") {
                return true;
            }
            // If we see the plaintext version key, it's not obfuscated.
            if map.contains_key("Version") {
                return false;
            }
            // If neither key is present (e.g., accountdata), check for other
            // known obfuscated keys.
            // "6f=" = PlayerStateData, "8>q" = Platform
            map.contains_key("6f=") || map.contains_key("8>q")
        }
        None => false,
    }
}
```

### Convenience function for the full pipeline

```rust
/// Deobfuscate JSON bytes: parse, detect obfuscation, apply mapping if needed.
///
/// Returns the (potentially deobfuscated) parsed JSON value.
pub fn deobfuscate_json(
    json_bytes: &[u8],
    mapping: &KeyMapping,
) -> Result<serde_json::Value, SaveError> {
    let mut value: serde_json::Value = serde_json::from_slice(json_bytes)
        .map_err(|e| SaveError::JsonParseError {
            message: e.to_string(),
        })?;

    if is_obfuscated(&value) {
        mapping.deobfuscate(&mut value);
    }

    Ok(value)
}
```

---

## SaveError Additions

Add these variants to `SaveError` (from Milestones 1.5/1.6):

```rust
    /// Failed to parse a mapping JSON file.
    MappingParseError {
        message: String,
    },

    /// Failed to parse save JSON.
    JsonParseError {
        message: String,
    },
```

---

## Dependencies

Add to `crates/nms-save/Cargo.toml`:

```toml
[dependencies]
nms-core = { workspace = true }
lz4_flex = "0.11"
thiserror = "2"
sha2 = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Add `serde` and `serde_json` to root `Cargo.toml` `[workspace.dependencies]` if using workspace dependency management.

---

## Bundled Data Files

Copy the mapping files into the crate's source tree so they can be included with `include_str!()`:

```
crates/nms-save/
  data/
    mapping_mbincompiler.json    -- copy from workbench/libNOM.map/libNOM.map/Resources/
    mapping_legacy.json          -- copy from workbench/libNOM.map/libNOM.map/Resources/
    mapping_savewizard.json      -- copy from workbench/libNOM.map/libNOM.map/Resources/
```

Commands to copy:

```bash
mkdir -p /Users/oubiwann/lab/oxur/nms-copilot/crates/nms-save/data
cp /Users/oubiwann/lab/oxur/nms-copilot/workbench/libNOM.map/libNOM.map/Resources/mapping_mbincompiler.json \
   /Users/oubiwann/lab/oxur/nms-copilot/crates/nms-save/data/
cp /Users/oubiwann/lab/oxur/nms-copilot/workbench/libNOM.map/libNOM.map/Resources/mapping_legacy.json \
   /Users/oubiwann/lab/oxur/nms-copilot/crates/nms-save/data/
cp /Users/oubiwann/lab/oxur/nms-copilot/workbench/libNOM.map/libNOM.map/Resources/mapping_savewizard.json \
   /Users/oubiwann/lab/oxur/nms-copilot/crates/nms-save/data/
```

The `include_str!()` paths in `KeyMapping::bundled()` are relative to the source file (`src/mapping.rs`), so use `"../data/mapping_mbincompiler.json"`.

---

## File Organization

```
crates/nms-save/
  Cargo.toml
  data/
    mapping_mbincompiler.json
    mapping_legacy.json
    mapping_savewizard.json
  src/
    lib.rs          -- add: pub mod mapping;
    error.rs        -- add MappingParseError, JsonParseError variants
    decompress.rs   -- (from Milestone 1.5)
    metadata.rs     -- (from Milestone 1.6)
    xxtea.rs        -- (from Milestone 1.6)
    mapping.rs      -- KeyMapping, MappingFile, MappingEntry,
                       is_obfuscated, deobfuscate_json
```

### `src/lib.rs` additions

```rust
pub mod mapping;

pub use mapping::{KeyMapping, is_obfuscated, deobfuscate_json};
```

---

## Tests

### Loading and basic lookups

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_load_bundled_mapping() {
        let mapping = KeyMapping::bundled();
        // The primary mapping has ~2000+ entries, legacy adds ~47 more.
        assert!(mapping.len() > 2000, "expected 2000+ entries, got {}", mapping.len());
        // Version should be from the primary mapping file.
        assert_eq!(mapping.version, "6.11.0.1");
    }

    #[test]
    fn test_lookup_known_keys() {
        let mapping = KeyMapping::bundled();
        assert_eq!(mapping.get("F2P"), Some("Version"));
        assert_eq!(mapping.get("6f="), Some("PlayerStateData"));
        assert_eq!(mapping.get("8>q"), Some("Platform"));
    }

    #[test]
    fn test_lookup_unknown_key() {
        let mapping = KeyMapping::bundled();
        assert_eq!(mapping.get("ZZZNOTAKEY"), None);
    }

    #[test]
    fn test_legacy_keys_merged() {
        let mapping = KeyMapping::bundled();
        // "5Ta" -> "AllowFriendBases" is in mapping_legacy.json.
        assert_eq!(mapping.get("5Ta"), Some("AllowFriendBases"));
    }
```

### Deobfuscation

```rust
    #[test]
    fn test_deobfuscate_simple() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({"F2P": 6726});
        mapping.deobfuscate(&mut value);
        assert_eq!(value, json!({"Version": 6726}));
    }

    #[test]
    fn test_deobfuscate_nested() {
        let mapping = KeyMapping::bundled();
        // "6f=" -> "PlayerStateData", "Pk4" -> "SaveName"
        // First verify "Pk4" maps to "SaveName":
        assert_eq!(mapping.get("Pk4"), Some("SaveName"),
            "Pk4 should map to SaveName; if this fails, check the mapping file for the correct key");

        let mut value = json!({"6f=": {"Pk4": "MySave"}});
        mapping.deobfuscate(&mut value);
        assert_eq!(value, json!({"PlayerStateData": {"SaveName": "MySave"}}));
    }

    #[test]
    fn test_deobfuscate_array() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({
            "F2P": 6726,
            "items": [
                {"F2P": 1},
                {"F2P": 2}
            ]
        });
        mapping.deobfuscate(&mut value);
        // "F2P" -> "Version" everywhere, "items" stays as-is (unknown key).
        assert_eq!(value["Version"], 6726);
        assert_eq!(value["items"][0]["Version"], 1);
        assert_eq!(value["items"][1]["Version"], 2);
    }

    #[test]
    fn test_deobfuscate_preserves_unknown_keys() {
        let mapping = KeyMapping::bundled();
        let mut value = json!({"F2P": 6726, "UnknownKey123": "hello"});
        mapping.deobfuscate(&mut value);
        assert_eq!(value["Version"], 6726);
        assert_eq!(value["UnknownKey123"], "hello");
    }

    #[test]
    fn test_deobfuscate_already_plaintext() {
        let mapping = KeyMapping::bundled();
        let original = json!({"Version": 6726, "PlayerStateData": {"SaveName": "MySave"}});
        let mut value = original.clone();
        mapping.deobfuscate(&mut value);
        // Keys that are not in the obfuscated->readable mapping are preserved as-is.
        // "Version" is a readable name, not an obfuscated key, so it stays.
        assert_eq!(value["Version"], 6726);
        assert_eq!(value["PlayerStateData"]["SaveName"], "MySave");
    }

    #[test]
    fn test_savewizard_fixup() {
        let mapping = KeyMapping::bundled();
        // The savewizard mapping fixes "MultiTools" -> "Multitools".
        // If the obfuscated key for "MultiTools" is in the main mapping,
        // after deobfuscation the fixup should apply.
        // Direct test: manually insert "MultiTools" and verify fixup.
        let mut value = json!({"MultiTools": [1, 2, 3]});
        mapping.deobfuscate(&mut value);
        // "MultiTools" should be fixed to "Multitools".
        assert!(value.get("Multitools").is_some(),
            "savewizard fixup should rename MultiTools to Multitools");
        assert!(value.get("MultiTools").is_none());
    }
```

### Auto-detection

```rust
    #[test]
    fn test_is_obfuscated_with_f2p() {
        let value = json!({"F2P": 6726, "6f=": {}});
        assert!(is_obfuscated(&value));
    }

    #[test]
    fn test_is_obfuscated_with_version() {
        let value = json!({"Version": 6726, "PlayerStateData": {}});
        assert!(!is_obfuscated(&value));
    }

    #[test]
    fn test_is_obfuscated_no_version_key() {
        // AccountData might not have "Version" or "F2P" at top level.
        // Check for other known obfuscated keys.
        let value = json!({"8>q": "PC"});
        assert!(is_obfuscated(&value));
    }

    #[test]
    fn test_is_obfuscated_non_object() {
        let value = json!([1, 2, 3]);
        assert!(!is_obfuscated(&value));
    }

    #[test]
    fn test_is_obfuscated_empty_object() {
        let value = json!({});
        assert!(!is_obfuscated(&value));
    }
```

### Convenience pipeline

```rust
    #[test]
    fn test_deobfuscate_json_bytes() {
        let mapping = KeyMapping::bundled();
        let json_bytes = br#"{"F2P": 6726}"#;
        let value = deobfuscate_json(json_bytes, &mapping).unwrap();
        assert_eq!(value["Version"], 6726);
    }

    #[test]
    fn test_deobfuscate_json_already_plaintext() {
        let mapping = KeyMapping::bundled();
        let json_bytes = br#"{"Version": 6726}"#;
        let value = deobfuscate_json(json_bytes, &mapping).unwrap();
        assert_eq!(value["Version"], 6726);
    }

    #[test]
    fn test_deobfuscate_json_invalid_json() {
        let mapping = KeyMapping::bundled();
        let json_bytes = b"not json";
        let err = deobfuscate_json(json_bytes, &mapping).unwrap_err();
        assert!(matches!(err, SaveError::JsonParseError { .. }));
    }
```

### Mapping file parsing

```rust
    #[test]
    fn test_from_json_valid() {
        let json = r#"{"libMBIN_version":"1.0.0","Mapping":[{"Key":"abc","Value":"Alpha"}]}"#;
        let mapping = KeyMapping::from_json(json).unwrap();
        assert_eq!(mapping.version, "1.0.0");
        assert_eq!(mapping.get("abc"), Some("Alpha"));
        assert_eq!(mapping.len(), 1);
    }

    #[test]
    fn test_from_json_invalid() {
        let json = "not valid json";
        let err = KeyMapping::from_json(json).unwrap_err();
        assert!(matches!(err, SaveError::MappingParseError { .. }));
    }

    #[test]
    fn test_from_json_empty_mapping() {
        let json = r#"{"libMBIN_version":"1.0.0","Mapping":[]}"#;
        let mapping = KeyMapping::from_json(json).unwrap();
        assert!(mapping.is_empty());
    }
}
```

---

## Performance Notes

The bundled `mapping_mbincompiler.json` is ~54KB. Using `include_str!()` embeds it in the binary. The `HashMap` lookup is O(1) per key, and the tree walk visits each JSON node once, so deobfuscation is O(n) where n is the total number of JSON nodes.

NMS save files are typically 5-30MB of JSON. Parsing with `serde_json::Value` and walking the tree takes roughly 100-500ms on modern hardware. This is acceptable for a one-time read operation.

If performance becomes a concern, consider:
1. Using `serde_json::StreamDeserializer` to process the JSON in chunks.
2. String replacement on raw bytes (faster but fragile -- could match inside string values).
3. Only deobfuscating the fields we actually need (selective tree walk with early termination).

For Phase 1, the simple tree-walk approach is sufficient.

---

## Acceptance Criteria

1. `KeyMapping::bundled()` loads all three mapping files and merges them correctly.
2. `KeyMapping::deobfuscate()` correctly replaces obfuscated keys at all nesting levels.
3. Unknown keys are preserved as-is (not dropped or errored).
4. The savewizard fixup (`"MultiTools"` -> `"Multitools"`) is applied after deobfuscation.
5. `is_obfuscated()` correctly distinguishes obfuscated from plaintext JSON.
6. `deobfuscate_json()` skips deobfuscation for already-plaintext JSON.
7. `cargo test -p nms-save` passes all tests.
8. `cargo clippy -p nms-save` reports no warnings.
9. No `unsafe` code.
