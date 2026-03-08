//! Skeleton parser for NomNom JSON export format.
//!
//! NomNom exports are nearly identical to deobfuscated standard JSON.
//! This module provides format detection and a passthrough parser.
//! Key normalization will be added when real NomNom exports are available
//! for testing.

use nms_save::model::SaveRoot;

/// Errors that can occur when parsing NomNom format files.
#[derive(Debug, thiserror::Error)]
pub enum NomNomError {
    /// The input JSON does not match expected NomNom format markers.
    #[error("not a NomNom format file")]
    NotNomNom,
    /// JSON parsing failed.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Detect whether a JSON string is likely NomNom format.
///
/// Checks for the presence of top-level keys that NomNom exports include.
/// This is a heuristic -- it may produce false positives on standard
/// deobfuscated save JSON.
pub fn is_nomnom_format(json: &str) -> bool {
    json.contains("\"Version\"") && json.contains("\"PlayerStateData\"")
}

/// Parse a NomNom JSON export into a [`SaveRoot`].
///
/// Currently a passthrough to `serde_json` -- key normalization will be
/// added when real NomNom exports are available for testing.
pub fn parse_nomnom(json: &str) -> Result<SaveRoot, NomNomError> {
    if !is_nomnom_format(json) {
        return Err(NomNomError::NotNomNom);
    }
    let save: SaveRoot = serde_json::from_str(json)?;
    Ok(save)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_nomnom_format_positive() {
        let json = r#"{"Version": 4720, "PlayerStateData": {}}"#;
        assert!(is_nomnom_format(json));
    }

    #[test]
    fn test_is_nomnom_format_negative_missing_version() {
        let json = r#"{"PlayerStateData": {}}"#;
        assert!(!is_nomnom_format(json));
    }

    #[test]
    fn test_is_nomnom_format_negative_missing_player() {
        let json = r#"{"Version": 4720}"#;
        assert!(!is_nomnom_format(json));
    }

    #[test]
    fn test_is_nomnom_format_negative_empty() {
        assert!(!is_nomnom_format(""));
    }

    #[test]
    fn test_parse_nomnom_not_nomnom_returns_error() {
        let result = parse_nomnom(r#"{"foo": "bar"}"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, NomNomError::NotNomNom));
        assert_eq!(err.to_string(), "not a NomNom format file");
    }

    #[test]
    fn test_parse_nomnom_invalid_json_returns_error() {
        let json = r#"{"Version": 4720, "PlayerStateData": {}, not valid"#;
        let result = parse_nomnom(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, NomNomError::Json(_)));
        assert!(err.to_string().starts_with("JSON parse error:"));
    }

    #[test]
    fn test_parse_nomnom_valid_save() {
        let json = r#"{
            "Version": 4720,
            "Platform": "Mac|Final",
            "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "NomNom Test", "TotalPlayTime": 42},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "ExpeditionContext": {
                "GameMode": 6,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": []
                }
            },
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": []}}}
        }"#;
        let save = parse_nomnom(json).unwrap();
        assert_eq!(save.version, 4720);
        assert_eq!(save.common_state_data.save_name, "NomNom Test");
    }
}
