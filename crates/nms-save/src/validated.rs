//! Whole-save address validation before typed deserialization.
//!
//! The checked parser examines the original, deobfuscated JSON value before
//! serde can supply defaults for missing address fields. It validates modeled
//! discovery rows and both player contexts, then keeps each checked address
//! paired with the typed save and raw discovery row produced from that same
//! value. The legacy [`crate::parse_save`] path remains available when callers
//! need its compatibility behavior instead.

use std::path::Path;

use nms_core::{GalacticAddress, PlayerBase, PlayerState};
use serde_json::Value;

use crate::address::{
    AddressDecodeError, decode_base_address, decode_current_address, decode_discovery_address,
    decode_previous_address,
};
use crate::error::SaveError;
use crate::mapping::{KeyMapping, deobfuscate_json};
use crate::model::{PersistentPlayerBase, RawDiscoveryRecord, SaveRoot};

/// An owned typed save with checked addresses from the original JSON value.
///
/// Construct values with [`parse_validated_save`] or
/// [`parse_validated_save_file`]. This type retains the typed [`SaveRoot`],
/// checked active [`PlayerState`], checked bases, and a source-ordered pairing
/// of each raw discovery row with its validated [`GalacticAddress`]. It is not
/// a representation of every unknown field in the original JSON.
#[derive(Debug)]
pub struct ValidatedSave {
    root: SaveRoot,
    discoveries: Vec<GalacticAddress>,
    player_state: PlayerState,
    bases: Vec<PlayerBase>,
}

impl ValidatedSave {
    /// Return the typed root deserialized from the checked source value.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let save = nms_save::parse_validated_save_file(std::path::Path::new("save.hg"))?;
    /// assert!(save.root().version > 0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn root(&self) -> &SaveRoot {
        &self.root
    }

    /// Return the active player state built with checked address values.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let save = nms_save::parse_validated_save_file(std::path::Path::new("save.hg"))?;
    /// println!("{}", save.player_state().current_reality);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn player_state(&self) -> &PlayerState {
        &self.player_state
    }

    /// Return checked bases from the active player context in source order.
    ///
    /// Each address comes from the pre-deserialization validation pass.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let save = nms_save::parse_validated_save_file(std::path::Path::new("save.hg"))?;
    /// for base in save.bases() {
    ///     println!("{}", base.name);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn bases(&self) -> &[PlayerBase] {
        &self.bases
    }

    /// Iterate over raw discovery rows and their checked physical addresses.
    ///
    /// Rows follow their original discovery-list order. The exact-size
    /// iterator pairs each typed raw row with the address decoded from its
    /// corresponding original JSON row; address parsing does not drop rows.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let save = nms_save::parse_validated_save_file(std::path::Path::new("save.hg"))?;
    /// for (record, address) in save.discoveries() {
    ///     println!("{} at {}", record.dd.dt, address.reality_index);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn discoveries(
        &self,
    ) -> impl ExactSizeIterator<Item = (&RawDiscoveryRecord, GalacticAddress)> + '_ {
        self.root
            .discovery_manager_data
            .discovery_data_v1
            .store
            .record
            .iter()
            .zip(self.discoveries.iter().copied())
    }
}

/// Parse plaintext, deobfuscated JSON after validating modeled addresses.
///
/// This function expects JSON bytes, not a compressed or obfuscated save file.
/// It checks the original JSON value before typed serde defaults can hide
/// missing or invalid address fields, then deserializes the same value.
///
/// `ActiveContext` is required to be a string; exactly `Expedition` selects
/// that context and every other string selects `BaseContext`. Both context
/// objects are required. Validation visits all discovery rows, then
/// `BaseContext`, then `ExpeditionContext`, before constructing the typed root.
///
/// The returned active player state requires `PlayerStateData` and its current
/// address in the active context. Missing player state or current addresses in
/// the inactive context are allowed; supplied inactive addresses are still
/// validated. A missing previous address or a present all-six-zero sentinel
/// produces `None`; a supplied malformed or null value is an error. Supplied
/// bases are validated in either context, but only active-context bases are
/// returned. Discovery rows, including unknown discovery types, remain paired
/// with checked addresses in source order. See [`crate::address`] for address
/// field ranges and path rules.
///
/// Missing optional discovery hierarchy produces an empty validated discovery
/// list. The typed root is still deserialized afterward, so missing required
/// top-level typed metadata can produce a JSON parse error. The root is a typed
/// subset, not a copy of every source field. Paired discoveries preserve row
/// order; signed balances use legacy casts to `u64`, and the freighter address
/// remains `None`.
///
/// Unlike [`crate::parse_save`], this checked path validates modeled source
/// addresses before serde defaults. Use [`parse_validated_save_file`] when the
/// input is a save file that needs the existing decompression and key-mapping
/// pipeline.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let bytes = std::fs::read("save.json")?;
/// let save = nms_save::parse_validated_save(&bytes)?;
/// println!("{} discoveries", save.discoveries().len());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`SaveError::Address`] for a missing or invalid modeled address or
/// container. Invalid JSON and unrelated typed metadata produce the existing
/// JSON parse error.
pub fn parse_validated_save(json: &[u8]) -> Result<ValidatedSave, SaveError> {
    let value: Value = serde_json::from_slice(json).map_err(json_error)?;
    validated_from_value(value)
}

/// Parse and validate modeled addresses from a save file.
///
/// The resulting plaintext, deobfuscated JSON value is checked before serde
/// constructs the typed root. Validation applies to the original values, so a
/// serde default cannot make a missing address appear valid. The legacy
/// [`crate::parse_save_file`] function remains available for callers that need
/// the compatibility parser without checked addresses.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let save = nms_save::parse_validated_save_file(std::path::Path::new("save.hg"))?;
/// println!("version {}", save.root().version);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`SaveError::Io`] and decompression errors from earlier stages,
/// [`SaveError::JsonParseError`] for malformed JSON or typed metadata, and
/// [`SaveError::Address`] for invalid modeled source addresses.
///
/// # Panics
///
/// Decompression can panic on a malformed compressed-size offset on 32-bit
/// targets; see [`crate::decompress::decompress_save`]. The bundled mapping
/// can also panic if its compiled-in data is invalid.
pub fn parse_validated_save_file(path: &Path) -> Result<ValidatedSave, SaveError> {
    let raw = std::fs::read(path)?;
    let decompressed = crate::decompress::decompress_save(&raw)?;
    let json = crate::sanitize_for_json(&decompressed);
    let value = if crate::is_obfuscated_bytes(json.as_bytes()) {
        let mapping = KeyMapping::bundled();
        deobfuscate_json(json.as_bytes(), &mapping)?
    } else {
        serde_json::from_slice(json.as_bytes()).map_err(json_error)?
    };
    validated_from_value(value)
}

fn validated_from_value(value: Value) -> Result<ValidatedSave, SaveError> {
    let object = value
        .as_object()
        .ok_or_else(|| address_error(invalid_type("/", "an object")))?;
    let active_context_value = object.get("ActiveContext").ok_or_else(|| {
        address_error(AddressDecodeError::Missing {
            path: "/ActiveContext".into(),
        })
    })?;
    let active_context = active_context_value
        .as_str()
        .ok_or_else(|| address_error(invalid_type("/ActiveContext", "a string")))?;

    let active_is_expedition = active_context == "Expedition";

    // Address traversal is deliberately source ordered: all discovery rows
    // precede BaseContext and ExpeditionContext, regardless of the order in
    // which the later typed model stores those members.
    let discoveries = validate_discoveries(object.get("DiscoveryManagerData"))?;
    let base_context = context_value(object.get("BaseContext"), "/BaseContext")?;
    let (base_current, base_previous, base_addresses) =
        validate_context(base_context, "/BaseContext", !active_is_expedition)?;
    let expedition_context = context_value(object.get("ExpeditionContext"), "/ExpeditionContext")?;
    let (expedition_current, expedition_previous, expedition_base_addresses) = validate_context(
        expedition_context,
        "/ExpeditionContext",
        active_is_expedition,
    )?;

    // Keep the existing typed model and its compatibility behavior. The checked
    // values above are the authority for application-visible address fields.
    let root: SaveRoot = serde_json::from_value(value).map_err(json_error)?;
    let typed_records = &root.discovery_manager_data.discovery_data_v1.store.record;
    if typed_records.len() != discoveries.len() {
        return Err(address_error(AddressDecodeError::InvalidType {
            path: "/DiscoveryManagerData/DiscoveryData-v1/Store/Record".into(),
            expected: "a record list matching the validated source rows",
        }));
    }

    let (current, previous, raw_bases, checked_base_addresses) = if active_is_expedition {
        (
            expedition_current,
            expedition_previous,
            &root
                .expedition_context
                .player_state_data
                .persistent_player_bases,
            expedition_base_addresses,
        )
    } else {
        (
            base_current,
            base_previous,
            &root.base_context.player_state_data.persistent_player_bases,
            base_addresses,
        )
    };
    if raw_bases.len() != checked_base_addresses.len() {
        return Err(address_error(AddressDecodeError::InvalidType {
            path: "/PersistentPlayerBases".into(),
            expected: "a base list matching the validated source rows",
        }));
    }
    let raw_player = if active_is_expedition {
        &root.expedition_context.player_state_data
    } else {
        &root.base_context.player_state_data
    };
    let bases = raw_bases
        .iter()
        .zip(checked_base_addresses)
        .map(|(raw, address)| core_base(raw, address))
        .collect();
    let player_state = PlayerState::new(
        current,
        current.reality_index,
        previous,
        None,
        raw_player.units as u64,
        raw_player.nanites as u64,
        raw_player.specials as u64,
    );

    Ok(ValidatedSave {
        root,
        discoveries,
        player_state,
        bases,
    })
}

fn context_value<'a>(
    value: Option<&'a Value>,
    path: &str,
) -> Result<&'a serde_json::Map<String, Value>, SaveError> {
    let value = value.ok_or_else(|| {
        address_error(AddressDecodeError::Missing {
            path: path.to_owned(),
        })
    })?;
    value
        .as_object()
        .ok_or_else(|| address_error(invalid_type(path, "an object")))
}

fn validate_context(
    context: &serde_json::Map<String, Value>,
    context_path: &str,
    active: bool,
) -> Result<
    (
        GalacticAddress,
        Option<GalacticAddress>,
        Vec<GalacticAddress>,
    ),
    SaveError,
> {
    let player_path = format!("{context_path}/PlayerStateData");
    let Some(player_value) = context.get("PlayerStateData") else {
        if active {
            return Err(address_error(AddressDecodeError::Missing {
                path: player_path,
            }));
        }
        return Ok((GalacticAddress::new(0, 0, 0, 0, 0, 0), None, Vec::new()));
    };
    let player = player_value
        .as_object()
        .ok_or_else(|| address_error(invalid_type(&player_path, "an object")))?;

    let current_path = format!("{player_path}/UniverseAddress");
    let current = if active {
        decode_current_address(player.get("UniverseAddress"), &current_path)
            .map_err(address_error)?
    } else if let Some(value) = player.get("UniverseAddress") {
        decode_current_address(Some(value), &current_path).map_err(address_error)?
    } else {
        GalacticAddress::new(0, 0, 0, 0, 0, 0)
    };

    let previous_path = format!("{player_path}/PreviousUniverseAddress");
    let previous = decode_previous_address(player.get("PreviousUniverseAddress"), &previous_path)
        .map_err(address_error)?;

    let bases_path = format!("{player_path}/PersistentPlayerBases");
    let raw_bases = match player.get("PersistentPlayerBases") {
        None => &[][..],
        Some(value) => value
            .as_array()
            .ok_or_else(|| address_error(invalid_type(&bases_path, "an array")))?,
    };
    let mut addresses = Vec::with_capacity(raw_bases.len());
    for (index, base) in raw_bases.iter().enumerate() {
        let path = format!("{bases_path}/{index}");
        let base = base
            .as_object()
            .ok_or_else(|| address_error(invalid_type(&path, "an object")))?;
        let address_path = format!("{path}/GalacticAddress");
        addresses.push(
            decode_base_address(base.get("GalacticAddress"), &address_path)
                .map_err(address_error)?,
        );
    }
    Ok((current, previous, addresses))
}

fn validate_discoveries(manager: Option<&Value>) -> Result<Vec<GalacticAddress>, SaveError> {
    let Some(manager) = manager else {
        return Ok(Vec::new());
    };
    let path = "/DiscoveryManagerData";
    let manager = manager
        .as_object()
        .ok_or_else(|| address_error(invalid_type(path, "an object")))?;
    let Some(data) = manager.get("DiscoveryData-v1") else {
        return Ok(Vec::new());
    };
    let data_path = "/DiscoveryManagerData/DiscoveryData-v1";
    let data = data
        .as_object()
        .ok_or_else(|| address_error(invalid_type(data_path, "an object")))?;
    let Some(store) = data.get("Store") else {
        return Ok(Vec::new());
    };
    let store_path = "/DiscoveryManagerData/DiscoveryData-v1/Store";
    let store = store
        .as_object()
        .ok_or_else(|| address_error(invalid_type(store_path, "an object")))?;
    let Some(records) = store.get("Record") else {
        return Ok(Vec::new());
    };
    let records_path = "/DiscoveryManagerData/DiscoveryData-v1/Store/Record";
    let records = records
        .as_array()
        .ok_or_else(|| address_error(invalid_type(records_path, "an array")))?;
    let mut addresses = Vec::with_capacity(records.len());
    for (index, record) in records.iter().enumerate() {
        let row_path = format!("{records_path}/{index}");
        let record = record
            .as_object()
            .ok_or_else(|| address_error(invalid_type(&row_path, "an object")))?;
        let discovery_path = format!("{row_path}/DD");
        let discovery_value = record.get("DD").ok_or_else(|| {
            address_error(AddressDecodeError::Missing {
                path: discovery_path.clone(),
            })
        })?;
        let discovery = discovery_value
            .as_object()
            .ok_or_else(|| address_error(invalid_type(&discovery_path, "an object")))?;
        let address_path = format!("{discovery_path}/UA");
        addresses.push(
            decode_discovery_address(discovery.get("UA"), &address_path).map_err(address_error)?,
        );
    }
    Ok(addresses)
}

fn core_base(raw: &PersistentPlayerBase, address: GalacticAddress) -> PlayerBase {
    let mut base = raw.to_core_base();
    base.address = address;
    base
}

fn invalid_type(path: &str, expected: &'static str) -> AddressDecodeError {
    AddressDecodeError::InvalidType {
        path: path.to_owned(),
        expected,
    }
}

fn address_error(error: AddressDecodeError) -> SaveError {
    SaveError::Address(error)
}

fn json_error(error: serde_json::Error) -> SaveError {
    SaveError::JsonParseError {
        message: error.to_string(),
    }
}
