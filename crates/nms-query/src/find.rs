//! Planet/system search queries.

use nms_core::address::GalacticAddress;
use nms_core::biome::{Biome, BiomeSubType};
use nms_core::system::{Planet, System};
use nms_graph::model::PlanetKey;
use nms_graph::query::BiomeFilter;
use nms_graph::{GalaxyModel, GraphError};

/// How to determine the reference galaxy and origin for distance calculations.
///
/// See [`execute_find`] for current-position validation and explicit-reference
/// behavior. The [`crate#examples`] shows an explicit
/// address reference in use.
#[derive(Debug, Clone, Default)]
pub enum ReferencePoint {
    /// Use the player's current position from the save file.
    #[default]
    CurrentPosition,
    /// Use a named base's position.
    Base(
        /// Name resolved through the model-wide base index, without trimming.
        String,
    ),
    /// Use an explicit galactic address.
    Address(
        /// Address whose galaxy supplies the query scope and distance origin.
        GalacticAddress,
    ),
}

/// Parameters for a planet/system search.
///
/// All supplied filters are combined before the optional nearest-result limit is applied. See the [`crate#examples`] for a complete query and result observation.
#[derive(Debug, Clone, Default)]
pub struct FindQuery {
    /// Filter by biome type.
    pub biome: Option<Biome>,
    /// Filter by biome subtype/variant.
    pub biome_subtype: Option<BiomeSubType>,
    /// Filter by infested flag.
    pub infested: Option<bool>,
    /// Optional maximum distance in light-years from the selected reference.
    ///
    /// Negative values always return no results. NaN returns no results when
    /// `nearest` is `None`; with `nearest` set, NaN adds no radius cutoff.
    pub within_ly: Option<f64>,
    /// Return at most this many results after all filters, nearest first.
    pub nearest: Option<usize>,
    /// Case-insensitive substring matched against either planet or system name.
    pub name_pattern: Option<String>,
    /// Case-insensitive substring matched against the system discoverer name.
    pub discoverer: Option<String>,
    /// Include planets whose name is `Some`, including an empty string.
    pub named_only: bool,
    /// Reference point for distance calculations.
    pub from: ReferencePoint,
}

/// A cloned planet/system match and its query-relative distance.
///
/// `portal_hex` comes from the stored system address, which may retain a nonzero
/// planet index. The [`nms_graph::SystemId`] used as the system key is planet-zeroed.
/// See the [`crate#examples`] for all returned fields.
#[derive(Debug, Clone)]
pub struct FindResult {
    /// Cloned matching planet record.
    pub planet: Planet,
    /// Cloned containing system record.
    pub system: System,
    /// Distance from the reference point in light-years.
    pub distance_ly: f64,
    /// Twelve-digit uppercase hex from the stored system address; callers may render it as glyphs.
    pub portal_hex: String,
}

/// Execute a find query against one selected galaxy in the model.
///
/// Resolve a reference and return matching planets from its galaxy.
/// See the [`crate#examples`] for current-position failure
/// and explicit-address success observations.
///
/// Current-position queries require a player whose galaxy matches the active
/// galaxy. Explicit address and base references use their own galaxy without
/// changing model selection. All biome, subtype, infested, name, discoverer, and
/// named-only filters combine before `nearest`; name matching checks planet or
/// system names, while discoverer matching checks `System.discoverer`.
/// Negative radius always returns no rows. With `nearest == None`, NaN returns
/// no rows; with `nearest == Some(_)`, NaN disables the radius cutoff. Positive
/// infinity is inclusive and `nearest == Some(0)` returns no rows. Missing systems
/// for flattened planet entries are skipped, and both the key galaxy and stored
/// system-address galaxy must match the reference. Results sort by distance,
/// galaxy, canonical system address, and planet index; explicit references do not
/// mutate the model's active-galaxy selection.
///
/// # Errors
///
/// Returns [`GraphError::NoPlayerPosition`] when a current-position query has
/// no player address, [`GraphError::BaseNotFound`] when a named base is absent,
/// or [`GraphError::ReferenceGalaxyMismatch`] when the player galaxy differs
/// from the active galaxy.
pub fn execute_find(model: &GalaxyModel, query: &FindQuery) -> Result<Vec<FindResult>, GraphError> {
    let (from, galaxy) = match &query.from {
        ReferencePoint::CurrentPosition => {
            let from = model
                .player_position()
                .copied()
                .ok_or(GraphError::NoPlayerPosition)?;
            if from.reality_index != model.active_galaxy {
                return Err(GraphError::ReferenceGalaxyMismatch {
                    reference: from.reality_index,
                    active: model.active_galaxy,
                });
            }
            (from, model.active_galaxy)
        }
        ReferencePoint::Base(name) => {
            let base = model
                .base(name)
                .ok_or_else(|| GraphError::BaseNotFound(name.clone()))?;
            (base.address, base.address.reality_index)
        }
        ReferencePoint::Address(address) => (*address, address.reality_index),
    };

    let biome_filter = BiomeFilter {
        biome: query.biome,
        biome_subtype: query.biome_subtype,
        infested: query.infested,
        named_only: query.named_only,
    };

    if query.within_ly.is_some_and(|radius| radius < 0.0) {
        return Ok(Vec::new());
    }
    if query.within_ly.is_some_and(f64::is_nan) && query.nearest.is_none() {
        return Ok(Vec::new());
    }

    let name_pattern = query
        .name_pattern
        .as_ref()
        .map(|pattern| pattern.to_lowercase());
    let discoverer_pattern = query
        .discoverer
        .as_ref()
        .map(|pattern| pattern.to_lowercase());
    let mut candidates: Vec<(PlanetKey, &Planet, &System, f64)> = Vec::new();

    for (&key, planet) in &model.planets {
        let Some(system) = model.system(&key.0) else {
            continue;
        };
        if key.0.reality_index() != galaxy || system.address.reality_index != galaxy {
            continue;
        }
        if !biome_filter.matches(planet) {
            continue;
        }

        if let Some(pattern) = &name_pattern {
            let name_matches = planet
                .name
                .as_ref()
                .is_some_and(|name| name.to_lowercase().contains(pattern))
                || system
                    .name
                    .as_ref()
                    .is_some_and(|name| name.to_lowercase().contains(pattern));
            if !name_matches {
                continue;
            }
        }

        if let Some(pattern) = &discoverer_pattern
            && !system
                .discoverer
                .as_ref()
                .is_some_and(|name| name.to_lowercase().contains(pattern))
        {
            continue;
        }

        let distance = from.distance_ly(&system.address);
        if let Some(radius) = query.within_ly
            && !radius.is_nan()
            && distance > radius
        {
            continue;
        }
        candidates.push((key, planet, system, distance));
    }

    candidates.sort_by(|left, right| {
        left.3
            .total_cmp(&right.3)
            .then_with(|| left.0.0.reality_index().cmp(&right.0.0.reality_index()))
            .then_with(|| left.0.0.system_address().cmp(&right.0.0.system_address()))
            .then_with(|| left.0.1.cmp(&right.0.1))
    });

    if let Some(nearest) = query.nearest {
        candidates.truncate(nearest);
    }

    let results = candidates
        .into_iter()
        .map(|(_, planet, system, distance_ly)| FindResult {
            planet: planet.clone(),
            system: system.clone(),
            distance_ly,
            portal_hex: format!("{:012X}", system.address.packed()),
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nms_core::player::{BaseType, PlayerBase, PlayerState};
    use nms_graph::spatial::SystemId;

    fn test_model() -> GalaxyModel {
        let json = r#"{
            "Version": 4720, "Platform": "Mac|Final", "ActiveContext": "Main",
            "CommonStateData": {"SaveName": "Test", "TotalPlayTime": 100},
            "BaseContext": {
                "GameMode": 1,
                "PlayerStateData": {
                    "UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 1, "PlanetIndex": 0}},
                    "Units": 0, "Nanites": 0, "Specials": 0,
                    "PersistentPlayerBases": [{"BaseVersion": 8, "GalacticAddress": "0x001000000064", "Position": [0.0,0.0,0.0], "Forward": [1.0,0.0,0.0], "LastUpdateTimestamp": 0, "Objects": [], "RID": "", "Owner": {"LID":"","UID":"1","USN":"","PTK":"ST","TS":0}, "Name": "Alpha Base", "BaseType": {"PersistentBaseTypes": "HomePlanetBase"}, "LastEditedById": "", "LastEditedByUsername": ""}]
                }
            },
            "ExpeditionContext": {"GameMode": 6, "PlayerStateData": {"UniverseAddress": {"RealityIndex": 0, "GalacticAddress": {"VoxelX": 0, "VoxelY": 0, "VoxelZ": 0, "SolarSystemIndex": 0, "PlanetIndex": 0}}, "Units": 0, "Nanites": 0, "Specials": 0, "PersistentPlayerBases": []}},
            "DiscoveryManagerData": {"DiscoveryData-v1": {"ReserveStore": 0, "ReserveManaged": 0, "Store": {"Record": [
                {"DD": {"UA": "0x001000000064", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x101000000064", "DT": "Planet", "VP": ["0xAB", 0]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Explorer", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x002000000C80", "DT": "SolarSystem", "VP": []}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}},
                {"DD": {"UA": "0x102000000C80", "DT": "Planet", "VP": ["0xCD", 1]}, "DM": {}, "OWS": {"LID": "", "UID": "1", "USN": "Traveler", "PTK": "ST", "TS": 1700000000}, "FL": {"U": 1}}
            ]}}}
        }"#;
        nms_save::parse_save(json.as_bytes())
            .map(|save| GalaxyModel::from_save(&save))
            .unwrap()
    }

    fn system(
        galaxy: u8,
        x: i16,
        ssi: u16,
        name: &str,
        discoverer: &str,
        planets: Vec<Planet>,
    ) -> System {
        System::new(
            GalacticAddress::new(x, 0, 0, ssi, 0, galaxy),
            Some(name.into()),
            Some(discoverer.into()),
            None,
            planets,
        )
    }

    fn model_with_systems(
        systems: impl IntoIterator<Item = System>,
        active_galaxy: u8,
        player: Option<GalacticAddress>,
    ) -> GalaxyModel {
        let mut model = GalaxyModel::new();
        model.player_state = player
            .map(|address| PlayerState::new(address, address.reality_index, None, None, 0, 0, 0));
        model.set_active_galaxy(active_galaxy);
        for system in systems {
            model.insert_system(system);
        }
        model
    }

    fn result_planet_names(results: &[FindResult]) -> Vec<String> {
        results
            .iter()
            .map(|result| {
                result
                    .planet
                    .name
                    .clone()
                    .unwrap_or_else(|| "<unnamed>".into())
            })
            .collect()
    }

    #[test]
    fn test_find_all_planets() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_find_by_biome() {
        let model = test_model();
        let query = FindQuery {
            biome: Some(Biome::Lush),
            ..Default::default()
        };
        let results = execute_find(&model, &query).unwrap();
        for r in &results {
            assert_eq!(r.planet.biome, Some(Biome::Lush));
        }
    }

    #[test]
    fn test_find_nearest_limit() {
        let model = test_model();
        let query = FindQuery {
            nearest: Some(1),
            ..Default::default()
        };
        let results = execute_find(&model, &query).unwrap();
        assert!(results.len() <= 1);
    }

    #[test]
    fn test_find_from_base() {
        let model = test_model();
        let query = FindQuery {
            from: ReferencePoint::Base("Alpha Base".into()),
            ..Default::default()
        };
        let results = execute_find(&model, &query);
        assert!(results.is_ok());
    }

    #[test]
    fn test_find_from_nonexistent_base_errors() {
        let model = test_model();
        let query = FindQuery {
            from: ReferencePoint::Base("No Such Base".into()),
            ..Default::default()
        };
        assert!(execute_find(&model, &query).is_err());
    }

    #[test]
    fn test_find_results_sorted_by_distance() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        for i in 1..results.len() {
            assert!(results[i].distance_ly >= results[i - 1].distance_ly);
        }
    }

    #[test]
    fn test_find_portal_hex_is_12_digits() {
        let model = test_model();
        let query = FindQuery::default();
        let results = execute_find(&model, &query).unwrap();
        for r in &results {
            assert_eq!(r.portal_hex.len(), 12);
        }
    }

    #[test]
    fn test_find_by_discoverer() {
        let model = test_model();
        let query = FindQuery {
            discoverer: Some("Explorer".into()),
            ..Default::default()
        };
        let results = execute_find(&model, &query).unwrap();
        for r in &results {
            assert!(r.system.discoverer.as_ref().unwrap().contains("Explorer"));
        }
    }

    #[test]
    fn test_find_keeps_same_index_planets_with_their_qualified_systems() {
        let address0a = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let address0b = GalacticAddress::new(0, 0, 0, 2, 0, 0);
        let address9 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let id0a = SystemId::from_address(&address0a);
        let id0b = SystemId::from_address(&address0b);
        let id9 = SystemId::from_address(&address9);
        let systems = [
            System::new(
                address0a,
                Some("Alpha".into()),
                None,
                None,
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Alpha planet".into()),
                    None,
                )],
            ),
            System::new(
                address0b,
                Some("Beta".into()),
                None,
                None,
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Beta planet".into()),
                    None,
                )],
            ),
            System::new(
                address9,
                Some("Nine".into()),
                None,
                None,
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Nine planet".into()),
                    None,
                )],
            ),
        ];

        for order in [[0, 1, 2], [2, 1, 0]] {
            let model = model_with_systems(
                order.into_iter().map(|index| systems[index].clone()),
                0,
                Some(address0a),
            );
            let results = execute_find(
                &model,
                &FindQuery {
                    from: ReferencePoint::Address(address0a),
                    biome: Some(Biome::Lush),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(results.len(), 2);
            let pairs: Vec<_> = results
                .iter()
                .map(|result| {
                    let id = SystemId::from_address(&result.system.address);
                    let contained = result
                        .system
                        .planets
                        .iter()
                        .find(|planet| planet.index == result.planet.index)
                        .unwrap();
                    assert_eq!(contained, &result.planet);
                    (id, result.planet.name.as_deref().unwrap())
                })
                .collect();
            assert!(pairs.contains(&(id0a, "Alpha planet")));
            assert!(pairs.contains(&(id0b, "Beta planet")));
            assert!(!pairs.iter().any(|(id, _)| *id == id9));

            let nine_results = execute_find(
                &model,
                &FindQuery {
                    from: ReferencePoint::Address(address9),
                    biome: Some(Biome::Lush),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(nine_results.len(), 1);
            assert_eq!(SystemId::from_address(&nine_results[0].system.address), id9);
            assert_eq!(nine_results[0].planet.name.as_deref(), Some("Nine planet"));
        }
    }

    #[test]
    fn test_find_composes_all_planet_filters_in_every_spatial_mode() {
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let model = model_with_systems(
            [
                system(
                    0,
                    0,
                    1,
                    "System 0",
                    "Explorer",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushRoomTemp),
                        true,
                        Some("Named".into()),
                        None,
                    )],
                ),
                system(
                    0,
                    1,
                    1,
                    "System 1",
                    "Explorer",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushHumid),
                        false,
                        None,
                        None,
                    )],
                ),
                system(
                    0,
                    2,
                    1,
                    "System 2",
                    "Explorer",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        Some(BiomeSubType::LushHumid),
                        false,
                        Some("Cool".into()),
                        None,
                    )],
                ),
                system(
                    0,
                    3,
                    1,
                    "System 3",
                    "Explorer",
                    vec![Planet::new(
                        1,
                        Some(Biome::Toxic),
                        None,
                        true,
                        Some("Toxic".into()),
                        None,
                    )],
                ),
            ],
            0,
            Some(origin),
        );

        let spatial_modes = [
            (None, None),
            (Some(10_000.0), None),
            (None, Some(10)),
            (Some(10_000.0), Some(10)),
        ];
        let predicate_cases = [
            (
                BiomeFilter {
                    biome: Some(Biome::Lush),
                    ..Default::default()
                },
                vec!["Named", "<unnamed>", "Cool"],
            ),
            (
                BiomeFilter {
                    biome_subtype: Some(BiomeSubType::LushHumid),
                    ..Default::default()
                },
                vec!["<unnamed>", "Cool"],
            ),
            (
                BiomeFilter {
                    infested: Some(false),
                    ..Default::default()
                },
                vec!["<unnamed>", "Cool"],
            ),
            (
                BiomeFilter {
                    named_only: true,
                    ..Default::default()
                },
                vec!["Named", "Cool", "Toxic"],
            ),
            (
                BiomeFilter {
                    biome: Some(Biome::Lush),
                    biome_subtype: Some(BiomeSubType::LushHumid),
                    infested: Some(false),
                    named_only: true,
                },
                vec!["Cool"],
            ),
            (
                BiomeFilter {
                    biome: Some(Biome::Lava),
                    ..Default::default()
                },
                vec![],
            ),
        ];

        let mut population_assertions = 0;
        for (filter, expected) in predicate_cases {
            for (within_ly, nearest) in spatial_modes {
                let query = FindQuery {
                    biome: filter.biome,
                    biome_subtype: filter.biome_subtype,
                    infested: filter.infested,
                    named_only: filter.named_only,
                    from: ReferencePoint::Address(origin),
                    within_ly,
                    nearest,
                    ..Default::default()
                };
                assert_eq!(
                    result_planet_names(&execute_find(&model, &query).unwrap()),
                    expected
                );
                population_assertions += 1;
            }
        }
        assert_eq!(population_assertions, 24);

        for (within_ly, nearest) in spatial_modes {
            let system_name_does_not_satisfy_named_only = FindQuery {
                from: ReferencePoint::Address(origin),
                name_pattern: Some("System 1".into()),
                named_only: true,
                within_ly,
                nearest,
                ..Default::default()
            };
            assert!(
                execute_find(&model, &system_name_does_not_satisfy_named_only)
                    .unwrap()
                    .is_empty()
            );
        }
    }

    #[test]
    fn test_find_explicit_references_select_their_galaxy_without_mutation() {
        let zero0 = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let nine0 = GalacticAddress::new(0, 0, 0, 1, 0, 9);
        let mut model = model_with_systems(
            [
                system(
                    0,
                    0,
                    1,
                    "Zero 0",
                    "Zero",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Zero 0".into()),
                        None,
                    )],
                ),
                system(
                    0,
                    1,
                    1,
                    "Zero 1",
                    "Zero",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Zero 1".into()),
                        None,
                    )],
                ),
                system(
                    9,
                    0,
                    1,
                    "Nine 0",
                    "Nine",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Nine 0".into()),
                        None,
                    )],
                ),
                system(
                    9,
                    1,
                    1,
                    "Nine 1",
                    "Nine",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Nine 1".into()),
                        None,
                    )],
                ),
            ],
            0,
            Some(zero0),
        );
        model.insert_base(PlayerBase::new(
            "Outer Base".into(),
            BaseType::HomePlanetBase,
            nine0,
            [0.0; 3],
            None,
        ));

        let default_results = execute_find(&model, &FindQuery::default()).unwrap();
        assert_eq!(result_planet_names(&default_results), ["Zero 0", "Zero 1"]);
        assert_eq!(
            default_results
                .iter()
                .map(|result| result.distance_ly)
                .collect::<Vec<_>>(),
            [0.0, 400.0]
        );

        for reference in [
            ReferencePoint::Address(nine0),
            ReferencePoint::Base("Outer Base".into()),
        ] {
            for (radius, nearest) in [
                (None, None),
                (Some(400.0), None),
                (None, Some(2)),
                (Some(400.0), Some(2)),
            ] {
                let results = execute_find(
                    &model,
                    &FindQuery {
                        from: reference.clone(),
                        within_ly: radius,
                        nearest,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert_eq!(result_planet_names(&results), ["Nine 0", "Nine 1"]);
                assert_eq!(
                    results
                        .iter()
                        .map(|result| result.distance_ly)
                        .collect::<Vec<_>>(),
                    [0.0, 400.0]
                );
            }
        }

        assert_eq!(model.active_galaxy, 0);
        assert_eq!(model.player_position(), Some(&zero0));

        model.set_active_galaxy(9);
        assert!(matches!(
            execute_find(&model, &FindQuery::default()),
            Err(GraphError::ReferenceGalaxyMismatch {
                reference: 0,
                active: 9
            })
        ));
        assert_eq!(model.active_galaxy, 9);
        assert_eq!(model.player_position(), Some(&zero0));

        let empty_galaxy = execute_find(
            &model,
            &FindQuery {
                from: ReferencePoint::Address(GalacticAddress::new(0, 0, 0, 1, 0, 7)),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(empty_galaxy.is_empty());
    }

    #[test]
    fn test_find_nearest_zero_resolves_reference_before_returning_empty() {
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let mut no_player = model_with_systems([], 0, None);
        assert!(matches!(
            execute_find(
                &no_player,
                &FindQuery {
                    nearest: Some(0),
                    ..Default::default()
                }
            ),
            Err(GraphError::NoPlayerPosition)
        ));
        no_player.player_state = Some(PlayerState::new(origin, 0, None, None, 0, 0, 0));

        assert!(matches!(
            execute_find(
                &no_player,
                &FindQuery {
                    from: ReferencePoint::Base("Missing".into()),
                    nearest: Some(0),
                    ..Default::default()
                }
            ),
            Err(GraphError::BaseNotFound(name)) if name == "Missing"
        ));
        assert!(
            execute_find(
                &no_player,
                &FindQuery {
                    from: ReferencePoint::Address(origin),
                    nearest: Some(0),
                    ..Default::default()
                }
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn test_find_filters_before_nearest_and_radius_boundaries() {
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let model = model_with_systems(
            [
                system(
                    0,
                    0,
                    1,
                    "Skip 0",
                    "Other",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Skip".into()),
                        None,
                    )],
                ),
                system(
                    0,
                    1,
                    1,
                    "Skip 1",
                    "Other",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Skip".into()),
                        None,
                    )],
                ),
                system(
                    0,
                    2,
                    1,
                    "Skip 2",
                    "Other",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Skip".into()),
                        None,
                    )],
                ),
                system(
                    0,
                    4,
                    1,
                    "Target",
                    "Explorer",
                    vec![Planet::new(
                        1,
                        Some(Biome::Lush),
                        None,
                        false,
                        Some("Keep".into()),
                        None,
                    )],
                ),
            ],
            0,
            Some(origin),
        );

        let named = execute_find(
            &model,
            &FindQuery {
                from: ReferencePoint::Address(origin),
                nearest: Some(1),
                name_pattern: Some("Keep".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result_planet_names(&named), ["Keep"]);
        assert_eq!(named[0].distance_ly, 1600.0);

        let discovered = execute_find(
            &model,
            &FindQuery {
                from: ReferencePoint::Address(origin),
                nearest: Some(1),
                discoverer: Some("Explorer".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result_planet_names(&discovered), ["Keep"]);
        assert_eq!(discovered[0].distance_ly, 1600.0);

        for (radius, expected_len) in [(Some(1600.0), 1), (Some(1599.0), 0)] {
            let results = execute_find(
                &model,
                &FindQuery {
                    from: ReferencePoint::Address(origin),
                    within_ly: radius,
                    name_pattern: Some("Keep".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(results.len(), expected_len);
        }
        let combined = execute_find(
            &model,
            &FindQuery {
                from: ReferencePoint::Address(origin),
                within_ly: Some(1600.0),
                nearest: Some(1),
                name_pattern: Some("Keep".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result_planet_names(&combined), ["Keep"]);
    }

    #[test]
    fn test_find_ties_sort_by_numeric_qualified_key_and_planet_index() {
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let negative = system(
            0,
            -1,
            1,
            "Alpha",
            "Explorer",
            vec![
                Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Alpha 1".into()),
                    None,
                ),
                Planet::new(
                    2,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Alpha 2".into()),
                    None,
                ),
            ],
        );
        let positive = system(
            0,
            1,
            1,
            "Zeta",
            "Explorer",
            vec![
                Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Zeta 1".into()),
                    None,
                ),
                Planet::new(
                    2,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Zeta 2".into()),
                    None,
                ),
            ],
        );
        let expected = ["Zeta 1", "Zeta 2", "Alpha 1", "Alpha 2"];

        for systems in [
            [negative.clone(), positive.clone()],
            [positive.clone(), negative.clone()],
        ] {
            let model = model_with_systems(systems, 0, Some(origin));
            let results = execute_find(
                &model,
                &FindQuery {
                    from: ReferencePoint::Address(origin),
                    nearest: Some(4),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(result_planet_names(&results), expected);
            assert_eq!(
                results
                    .iter()
                    .map(|result| result.distance_ly)
                    .collect::<Vec<_>>(),
                [400.0, 400.0, 400.0, 400.0]
            );
        }
    }

    #[test]
    fn test_find_preserves_legacy_nonfinite_radius_behavior() {
        let origin = GalacticAddress::new(0, 0, 0, 1, 0, 0);
        let model = model_with_systems(
            [system(
                0,
                0,
                1,
                "Origin",
                "Explorer",
                vec![Planet::new(
                    1,
                    Some(Biome::Lush),
                    None,
                    false,
                    Some("Origin".into()),
                    None,
                )],
            )],
            0,
            Some(origin),
        );
        let run = |radius, nearest| {
            execute_find(
                &model,
                &FindQuery {
                    from: ReferencePoint::Address(origin),
                    within_ly: radius,
                    nearest,
                    ..Default::default()
                },
            )
            .unwrap()
        };

        assert!(run(Some(-1.0), None).is_empty());
        assert!(run(Some(f64::NEG_INFINITY), Some(1)).is_empty());
        assert_eq!(run(Some(f64::INFINITY), None).len(), 1);
        assert!(run(Some(f64::NAN), None).is_empty());
        assert_eq!(run(Some(f64::NAN), Some(1)).len(), 1);
    }
}
