//! Heuristic detection and typed passthrough parsing for JSON with NomNom markers.
//!
//! Detection checks for two case-sensitive quoted substrings anywhere in the
//! input. It does not parse JSON or check top-level keys, so nested standard
//! JSON and malformed text can match. Parsing then deserializes the original
//! string directly into the legacy typed [`SaveRoot`] subset. This does not
//! normalize keys, validate a save for application use, or establish that the
//! input is an authentic NomNom export. Authentic export behavior remains
//! unverified because no such input is available.
//!
//! # Examples
//!
//! ```
//! use nms_compat::nomnom::{is_nomnom_format, parse_nomnom, NomNomError};
//!
//! let json = r#"{"Version":4720,"Platform":"Test","ActiveContext":"Main","CommonStateData":{"SaveName":"Synthetic JSON","TotalPlayTime":42},"BaseContext":{"PlayerStateData":{"Units":100,"Nanites":20,"Specials":5}},"ExpeditionContext":{},"DiscoveryManagerData":{"DiscoveryData-v1":{"Store":{"Record":[]}}}}"#;
//! assert!(is_nomnom_format(json));
//! let save = parse_nomnom(json)?;
//! assert_eq!(save.version, 4720);
//! assert_eq!(save.platform, "Test");
//! assert_eq!(save.common_state_data.save_name, "Synthetic JSON");
//! assert_eq!(save.base_context.player_state_data.units, 100);
//! assert_eq!(save.base_context.player_state_data.nanites, 20);
//! assert_eq!(save.base_context.player_state_data.specials, 5);
//! assert_eq!(save.active_context, "Main");
//! assert_eq!(save.common_state_data.total_play_time, 42);
//! assert_eq!(save.base_context.player_state_data.universe_address.reality_index, 0);
//! assert_eq!(save.base_context.player_state_data.universe_address.galactic_address.voxel_x, 0);
//! assert_eq!(save.base_context.player_state_data.universe_address.galactic_address.voxel_y, 0);
//! assert_eq!(save.base_context.player_state_data.universe_address.galactic_address.voxel_z, 0);
//! assert_eq!(save.base_context.player_state_data.universe_address.galactic_address.solar_system_index, 0);
//! assert_eq!(save.base_context.player_state_data.universe_address.galactic_address.planet_index, 0);
//! assert_eq!(save.base_context.player_state_data.previous_universe_address.reality_index, 0);
//! assert_eq!(save.base_context.player_state_data.previous_universe_address.galactic_address.voxel_x, 0);
//! assert_eq!(save.base_context.player_state_data.previous_universe_address.galactic_address.voxel_y, 0);
//! assert_eq!(save.base_context.player_state_data.previous_universe_address.galactic_address.voxel_z, 0);
//! assert_eq!(save.base_context.player_state_data.previous_universe_address.galactic_address.solar_system_index, 0);
//! assert_eq!(save.base_context.player_state_data.previous_universe_address.galactic_address.planet_index, 0);
//! assert_eq!(save.base_context.player_state_data.save_summary, "");
//! assert_eq!(save.base_context.player_state_data.health, 0);
//! assert_eq!(save.base_context.player_state_data.time_alive, 0);
//! assert_eq!(save.base_context.game_mode, 0);
//! assert!(save.base_context.player_state_data.persistent_player_bases.is_empty());
//! assert_eq!(save.expedition_context.game_mode, 0);
//! assert_eq!(save.expedition_context.player_state_data.units, 0);
//! assert_eq!(save.expedition_context.player_state_data.nanites, 0);
//! assert_eq!(save.expedition_context.player_state_data.specials, 0);
//! assert_eq!(save.expedition_context.player_state_data.universe_address.reality_index, 0);
//! assert_eq!(save.expedition_context.player_state_data.universe_address.galactic_address.voxel_x, 0);
//! assert_eq!(save.expedition_context.player_state_data.universe_address.galactic_address.voxel_y, 0);
//! assert_eq!(save.expedition_context.player_state_data.universe_address.galactic_address.voxel_z, 0);
//! assert_eq!(save.expedition_context.player_state_data.universe_address.galactic_address.solar_system_index, 0);
//! assert_eq!(save.expedition_context.player_state_data.universe_address.galactic_address.planet_index, 0);
//! assert!(save.expedition_context.player_state_data.persistent_player_bases.is_empty());
//! assert_eq!(save.discovery_manager_data.discovery_data_v1.reserve_store, 0);
//! assert_eq!(save.discovery_manager_data.discovery_data_v1.reserve_managed, 0);
//! assert!(save.discovery_manager_data.discovery_data_v1.store.record.is_empty());
//!
//! let nested = r#"{"nested":{"Version":null,"PlayerStateData":false}}"#;
//! assert!(is_nomnom_format(nested));
//! assert!(matches!(parse_nomnom(nested), Err(NomNomError::Json(error)) if error.is_data()));
//! let malformed = "not JSON \"Version\" \"PlayerStateData\"";
//! assert!(is_nomnom_format(malformed));
//! assert!(matches!(parse_nomnom(malformed), Err(NomNomError::Json(error)) if error.is_syntax()));
//! assert!(!is_nomnom_format("not JSON"));
//! assert!(!is_nomnom_format(r#"{"Version":4720}"#));
//! assert!(matches!(parse_nomnom("not JSON"), Err(NomNomError::NotNomNom)));
//! # Ok::<(), NomNomError>(())
//! ```
//!

use nms_save::model::SaveRoot;

/// Errors returned by marker-gated typed JSON parsing.
#[derive(Debug, thiserror::Error)]
pub enum NomNomError {
    /// At least one required marker substring is absent.
    #[error("not a NomNom format file")]
    NotNomNom,
    /// Typed JSON deserialization failed after the marker heuristic accepted the input.
    #[error("JSON parse error: {0}")]
    Json(
        /// Serde error returned after the marker heuristic accepts the input.
        #[from]
        serde_json::Error,
    ),
}

/// Check whether both NomNom marker substrings occur in the input.
///
/// This case-sensitive heuristic looks only for the literal substrings
/// `"Version"` and `"PlayerStateData"`; it does not parse JSON, require
/// top-level keys, or establish source format. Nested JSON and malformed text
/// can therefore return `true`, while differently cased markers return `false`.
///
/// # Examples
///
/// ```
/// use nms_compat::nomnom::is_nomnom_format;
///
/// assert!(is_nomnom_format(r#"{"Version": 4720, "PlayerStateData": {}}"#));
/// assert!(is_nomnom_format(r#"{"nested":{"Version":null,"PlayerStateData":false}}"#));
/// assert!(!is_nomnom_format(r#"{"version":4720,"PlayerStateData":{}}"#));
/// ```
///
/// See the [synthetic passthrough and heuristic examples](crate::nomnom#examples).
pub fn is_nomnom_format(json: &str) -> bool {
    json.contains("\"Version\"") && json.contains("\"PlayerStateData\"")
}

/// Parse marker-bearing JSON into the typed [`SaveRoot`] subset.
///
/// After [`is_nomnom_format`] accepts the input, this function passes the
/// original string to `serde_json::from_str`. Serde defaults and ignored
/// unknown fields are those of [`SaveRoot`]; this function does not normalize
/// keys, decode binary saves, perform checked source validation, or construct
/// a query model. Synthetic JSON is useful for demonstrating this typed
/// passthrough but is not evidence of an authentic NomNom export.
///
/// # Examples
///
/// See the [synthetic passthrough and heuristic examples](crate::nomnom#examples).
///
/// # Errors
///
/// Returns [`NomNomError::NotNomNom`] when either marker substring is absent.
/// If both markers occur, returns [`NomNomError::Json`] when serde cannot
/// deserialize the input into [`SaveRoot`]. The JSON variant preserves the
/// underlying serde error as its source.
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
