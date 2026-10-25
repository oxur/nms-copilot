//! Dispatch parsed REPL actions against the loaded galaxy model and caller-owned session.
//!
//! [`dispatch`] returns owned response text or an error; the REPL caller owns input parsing,
//! snapshot synchronization, printing, and process or terminal control.

use nms_core::BaseType;
use nms_core::address::{GalacticAddress, PortalAddress};
use nms_core::biome::Biome;
use nms_core::galaxy::{Galaxy, GalaxyType};
use nms_graph::GalaxyModel;
use nms_graph::query::BiomeFilter;
use nms_graph::route::RoutingAlgorithm;
use nms_query::display::{
    format_find_results, format_route, format_show_result, format_stats, hex_to_emoji,
};
use nms_query::find::{FindQuery, ReferencePoint, execute_find};
use nms_query::route::{RouteFrom, RouteQuery, TargetSelection, execute_route};
use nms_query::show::{ShowQuery, execute_show};
use nms_query::stats::{StatsQuery, execute_stats};
use nms_query::table::{Builder, build_table, nms_theme};
use nms_query::theme::Theme;

use nms_core::biome::{ALL_BIOME_SUBTYPES, ALL_BIOMES};
use nms_core::glyph::GLYPH_TABLE;

use crate::commands::{Action, ListTarget, SetTarget, ShowTarget};
use crate::session::SessionState;

/// Execute one parsed REPL action synchronously and return owned response text.
///
/// The caller supplies the already parsed [`Action`], shared [`GalaxyModel`], and mutable
/// [`SessionState`]. This function does not parse a command line, load or save files, refresh
/// from disk, start background work, print, open a map, exit, or control the terminal. It does
/// not unconditionally synchronize the session on entry. A caller sharing the model should
/// hold its write guard only for this call, then release it before displaying the returned text.
/// The REPL's `dispatch_repl_action` synchronizes its session immediately before dispatch and
/// prints the owned result after releasing that guard. Refresh and snapshot coherence belong to
/// that caller. `Status` reports the stored session snapshot; `Info` reports whole-model counts
/// and the physical player address. `Help` returns static text. Direct calls for `Map`, `Exit`,
/// and `Quit` return an empty string; the binary intercepts those actions before dispatch.
///
/// # Query behavior
///
/// `Find` parses an explicit biome before using the session biome default, so an invalid
/// explicit value is an error even when a default exists. The infested switch adds only the
/// positive filter; subtype and name-pattern filters are unset. An explicit `from` names a
/// base. Otherwise, stored base and address positions use their stored address, while an absent
/// or player-following position uses the model's current player position. The latter is scoped
/// to the active galaxy and can fail when that position belongs to another galaxy. A named or
/// address reference uses its own galaxy without selecting it.
///
/// For `Route`, nonempty named targets take precedence over and suppress biome parsing;
/// without targets, an explicit or session biome is required. Origin selection follows the
/// `Find` rules. An explicit warp range takes precedence over the session default. The `nn` and
/// `nearest-neighbor` spellings select nearest-neighbor routing; `2opt` and `two-opt` select
/// two-opt, which is also the default. Distance, target limits, and round-trip options are
/// forwarded without additional dispatcher validation. Routing is heuristic and best-effort;
/// it does not promise a globally optimal route or warp feasibility.
///
/// `Show` resolves system names before packed addresses in the active galaxy; base names use
/// the model-wide, case-insensitive lookup without trimming. `Stats` constructs its two flags,
/// but the shared query ignores them and summarizes the whole model. List reference data comes
/// from static galaxy, biome, glyph, and terrain catalogs; bases and systems are model-wide.
/// The galaxy catalog contains 256 entries and can be filtered by galaxy type; glyph and
/// terrain catalogs contain 16 and 32 entries. Bases and systems come from the model across all
/// galaxies. Bases sort by case-folded name. Systems sort named entries before missing or empty
/// names, then compare names case-insensitively; equal-name ordering follows map iteration and
/// is unspecified. A limit of zero or `all` shows every entry; otherwise the requested limit
/// applies. Empty models return the existing “No bases found.” or “No systems found.” message,
/// and truncated lists add the existing `Showing … of …` footer. The table format and fields
/// remain those produced by the existing formatter. Embedded themes may emit ANSI styling; this
/// API adds no caller color, piping, or plain-text policy.
///
/// # State changes
///
/// `Galaxy` without a selection reports the active and discovered galaxy views. A selection is
/// trimmed and resolved as a galaxy index from 0 through 255 or a case-insensitive name before
/// changing the model's selected view. The selected galaxy may contain no systems; its count
/// is still reported as zero, without creating a placeholder or moving the physical player.
/// Session synchronization refreshes selected-galaxy metadata and model counts, follows the
/// player's current position (or clears a missing player position), and preserves stored base
/// or manual-address positions, biome, and warp defaults. Selecting a galaxy and resetting
/// position or all state invoke this synchronization on their respective paths; dispatch does
/// not perform an unconditional entry refresh.
///
/// `Set Position` looks up a base name only, despite the parser's broader help wording; it is
/// not a direct address-setting action. Biome text is parsed before its session value changes.
/// Warp range stores any `f64`, while its confirmation text casts that value to `u64`.
/// Reset targets are lowercased before matching: `position` and `pos` reset position;
/// `biome` clears the biome; `warp-range` and `warp` clear warp range; and `all` or the empty
/// target resets all session state. An unknown target returns `Ok` with explanatory text, not
/// an error. `Status` formats the stored snapshot with its existing trailing newline;
/// `Info` reads the current model directly.
///
/// # Conversion behavior
///
/// `Convert` resolves the selected galaxy first, trimming the value and accepting an index
/// from 0 through 255 or a case-insensitive galaxy name. If multiple inputs are present, the
/// dispatcher checks glyphs, coordinates, galactic-address text, then voxel coordinates; the
/// parser normally permits at most one. Glyphs and galactic-address text both use
/// `parse_glyphs`: input is trimmed, an optional `0x` or `0X` is removed, and the remaining
/// text is parsed as twelve mixed portal glyph tokens. The galactic-address option does not
/// use the CLI packed-`u64` parser or accept its low-48-bit truncation behavior. Coordinates
/// delegate to signal-booster parsing after trimming. Voxel input requires an SSI and exactly
/// three comma-separated integers parsed as `i16`, `i8`, and `i16`; [`GalacticAddress::new`]
/// then retains 12 bits for X/Z and SSI and 4 bits for the planet. The dispatcher adds no CLI-style voxel
/// range checks. A successful conversion returns the existing nine-row format table; missing
/// input, SSI, and parse failures are returned as `String` errors.
///
/// # Examples
///
/// The returned strings are owned by the caller. `Map`, `Exit`, and `Quit` have no UI or
/// process effect when dispatched directly.
///
/// ```
/// # fn main() -> Result<(), String> {
/// use nms_copilot::commands::Action;
/// use nms_copilot::dispatch::dispatch;
/// use nms_copilot::session::SessionState;
/// use nms_graph::GalaxyModel;
///
/// let mut model = GalaxyModel::new();
/// let mut session = SessionState::from_model(&model);
/// assert_eq!(
///     dispatch(&Action::Info, &mut model, &mut session)?,
///     "Loaded model: 0 systems, 0 planets, 0 bases\nCurrent position: unknown\n",
/// );
/// let responses: Result<Vec<String>, String> = [Action::Map, Action::Exit, Action::Quit]
///     .iter()
///     .map(|action| dispatch(action, &mut model, &mut session))
///     .collect();
/// assert_eq!(responses?, vec![String::new(), String::new(), String::new()]);
/// assert_eq!(model.active_galaxy, 0);
/// assert_eq!((model.systems.len(), model.planets.len(), model.bases.len()), (0, 0, 0));
/// assert!(model.player_state.is_none());
/// assert!(session.position.is_none());
/// assert_eq!((session.biome_filter, session.warp_range), (None, None));
/// assert_eq!(
///     (session.galaxy.index, session.system_count, session.planet_count),
///     (0, 0, 0),
/// );
/// # Ok(())
/// # }
/// ```
///
/// Mutations update the supplied model view or session. This example observes a manual
/// position from galaxy 7 while selecting galaxy 9, then checks the complete bounded session
/// projection after each of the specified error and unknown-reset outcomes.
///
/// ```
/// # fn main() -> Result<(), String> {
/// use nms_copilot::commands::{Action, SetTarget};
/// use nms_copilot::dispatch::dispatch;
/// use nms_copilot::session::{PositionContext, SessionState};
/// use nms_core::{Biome, GalacticAddress, GalaxyType};
/// use nms_graph::GalaxyModel;
///
/// type View = (
///     u8, usize, usize, usize, Option<(&'static str, String, u64, u8)>,
///     Option<Biome>, Option<f64>, u8, &'static str, GalaxyType, usize, usize,
/// );
/// fn view(model: &GalaxyModel, session: &SessionState) -> View {
///     let position = session.position.as_ref().map(|position| {
///         let kind = match position {
///             PositionContext::Base { .. } => "base",
///             PositionContext::PlayerPosition(_) => "player",
///             PositionContext::Address(_) => "address",
///         };
///         let address = position.address();
///         (kind, position.label(), address.packed(), address.reality_index)
///     });
///     (
///         model.active_galaxy,
///         model.systems.len(), model.planets.len(), model.bases.len(), position,
///         session.biome_filter, session.warp_range,
///         session.galaxy.index, session.galaxy.name, session.galaxy.galaxy_type,
///         session.system_count, session.planet_count,
///     )
/// }
///
/// let mut model = GalaxyModel::new();
/// let mut session = SessionState::from_model(&model);
/// let manual = GalacticAddress::new(3, 0, 0, 1, 3, 7);
/// assert_eq!(session.set_position_address(manual), "Position set to 0x300100000003");
/// assert_eq!(
///     dispatch(&Action::Set { target: SetTarget::Biome { name: "Lush".into() } },
///              &mut model, &mut session)?,
///     "Biome filter set to Lush",
/// );
/// assert_eq!(
///     dispatch(&Action::Set { target: SetTarget::WarpRange { ly: 1200.5 } },
///              &mut model, &mut session)?,
///     "Warp range set to 1200 ly",
/// );
/// assert_eq!(
///     dispatch(&Action::Galaxy { selection: Some("9".into()) }, &mut model, &mut session)?,
///     "Switched to Eissentam (galaxy 9). 0 systems loaded.\n",
/// );
/// let expected: View = (
///     9, 0, 0, 0,
///     Some(("address", "0x300100000003".into(), 0x3001_0000_0003, 7)),
///     Some(Biome::Lush), Some(1200.5), 9, "Eissentam", GalaxyType::Lush, 0, 0,
/// );
/// assert_eq!(view(&model, &session), expected);
/// assert!(model.player_state.is_none());
/// assert_eq!(
///     dispatch(&Action::Status, &mut model, &mut session)?,
///     concat!(
///         "Galaxy:      Eissentam (9; Lush)\n",
///         "Model:       0 systems, 0 planets\n",
///         "Position:    0x300100000003 (Kikolgallr, galaxy 7)\n",
///         "Biome:       Lush\n",
///         "Warp range:  1200 ly\n",
///     ),
/// );
///
/// let cases: [(Action, Result<String, String>); 3] = [
///     (Action::Galaxy { selection: Some("256".into()) },
///      Err("Galaxy index out of range: 256 (must be 0-255)".into())),
///     (Action::Set { target: SetTarget::Position { name: "Missing".into() } },
///      Err("Base not found: \"Missing\"".into())),
///     (Action::Reset { target: "mystery".into() },
///      Ok("Unknown reset target: mystery. Use: position, biome, warp-range, all".into())),
/// ];
/// for (action, expected_result) in cases {
///     assert_eq!(dispatch(&action, &mut model, &mut session), expected_result);
///     assert_eq!(view(&model, &session), expected);
///     assert!(model.player_state.is_none());
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns existing parse, missing-input, missing-SSI, missing-base, current-position,
/// galaxy-scope, route-target, algorithm, or shared-query errors as `String`, using the existing
/// error display conversion. Query formatters return `String` rather than a fallible result and
/// may panic as described below. The exact branches and messages depend on the action and
/// existing parser/query contracts; errors are not typed and carry no promised source chain.
/// This API makes no general transaction or rollback guarantee.
///
/// # Panics
///
/// Panics may propagate from [`PortalAddress::parse_mixed`] when its 20-byte excerpt for an
/// invalid token ends inside a UTF-8 character, and from route jump-count accumulation on
/// checking builds if a warp-derived total overflows. Invalid embedded theme data could also
/// panic in the theme parser. This API does not promise general panic freedom.
pub fn dispatch(
    action: &Action,
    model: &mut GalaxyModel,
    session: &mut SessionState,
) -> Result<String, String> {
    match action {
        Action::Find {
            biome,
            infested,
            within,
            nearest,
            named,
            discoverer,
            from,
        } => {
            let biome = biome
                .as_ref()
                .map(|s| s.parse::<Biome>())
                .transpose()
                .map_err(|e| format!("Invalid biome: {e}"))?
                .or(session.biome_filter);

            let reference = match (from, &session.position) {
                (Some(name), _) => ReferencePoint::Base(name.clone()),
                (None, Some(crate::session::PositionContext::Base { address, .. }))
                | (None, Some(crate::session::PositionContext::Address(address))) => {
                    ReferencePoint::Address(*address)
                }
                (None, None | Some(crate::session::PositionContext::PlayerPosition(_))) => {
                    ReferencePoint::CurrentPosition
                }
            };

            let query = FindQuery {
                biome,
                biome_subtype: None,
                infested: if *infested { Some(true) } else { None },
                within_ly: *within,
                nearest: *nearest,
                name_pattern: None,
                discoverer: discoverer.clone(),
                named_only: *named,
                from: reference,
            };

            let results = execute_find(model, &query).map_err(|e| e.to_string())?;
            let theme = Theme::default_dark();
            Ok(format_find_results(&results, &theme))
        }

        Action::List { target } => dispatch_list(model, target),

        Action::Show { target } => dispatch_show(model, target),

        Action::Stats {
            biomes,
            discoveries,
        } => {
            let query = StatsQuery {
                biomes: *biomes || !*discoveries,
                discoveries: *discoveries || !*biomes,
            };
            let result = execute_stats(model, &query);
            let theme = Theme::default_dark();
            Ok(format_stats(&result, &theme))
        }

        Action::Route {
            biome,
            targets,
            from,
            warp_range,
            within,
            max_targets,
            algo,
            round_trip,
        } => dispatch_route(
            model,
            session,
            biome,
            targets,
            from,
            warp_range,
            within,
            max_targets,
            algo,
            round_trip,
        ),

        Action::Galaxy { selection } => dispatch_galaxy(model, session, selection.as_deref()),

        Action::Set { target } => dispatch_set(model, session, target),
        Action::Reset { target } => Ok(dispatch_reset(model, session, target)),
        Action::Status => Ok(session.format_status()),

        Action::Info => {
            let systems = model.systems.len();
            let planets = model.planets.len();
            let bases = model.bases.len();
            let pos = model
                .player_state
                .as_ref()
                .map(|ps| format!("{}", ps.current_address))
                .unwrap_or_else(|| "unknown".into());
            Ok(format!(
                "Loaded model: {systems} systems, {planets} planets, {bases} bases\n\
                 Current position: {pos}\n"
            ))
        }

        Action::Help => Ok(help_text()),

        Action::Map | Action::Exit | Action::Quit => Ok(String::new()),

        Action::Convert {
            glyphs,
            coords,
            ga,
            voxel,
            ssi,
            planet,
            galaxy,
        } => dispatch_convert(glyphs, coords, ga, voxel, *ssi, *planet, galaxy),
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_route(
    model: &GalaxyModel,
    session: &SessionState,
    biome: &Option<String>,
    targets: &[String],
    from: &Option<String>,
    warp_range: &Option<f64>,
    within: &Option<f64>,
    max_targets: &Option<usize>,
    algo: &Option<String>,
    round_trip: &bool,
) -> Result<String, String> {
    // 1. Determine targets: --target > --biome > session biome
    let target_selection = if !targets.is_empty() {
        TargetSelection::Named(targets.to_vec())
    } else {
        let biome_val = biome
            .as_ref()
            .map(|s| s.parse::<Biome>())
            .transpose()
            .map_err(|e| format!("Invalid biome: {e}"))?
            .or(session.biome_filter);

        match biome_val {
            Some(b) => TargetSelection::Biome(BiomeFilter {
                biome: Some(b),
                ..Default::default()
            }),
            None => return Err("Specify --target or --biome for route planning".into()),
        }
    };

    // 2. Determine from: --from > session position > CurrentPosition
    let route_from = match (from, &session.position) {
        (Some(name), _) => RouteFrom::Base(name.clone()),
        (None, Some(crate::session::PositionContext::Base { address, .. }))
        | (None, Some(crate::session::PositionContext::Address(address))) => {
            RouteFrom::Address(*address)
        }
        (None, None | Some(crate::session::PositionContext::PlayerPosition(_))) => {
            RouteFrom::CurrentPosition
        }
    };

    // 3. Determine warp_range: --warp-range > session warp_range > None
    let effective_warp_range = (*warp_range).or(session.warp_range);

    // 4. Parse algorithm
    let algorithm = match algo.as_deref() {
        Some("nn") | Some("nearest-neighbor") => RoutingAlgorithm::NearestNeighbor,
        Some("2opt") | Some("two-opt") | None => RoutingAlgorithm::TwoOpt,
        Some(other) => {
            return Err(format!(
                "Unknown algorithm: \"{other}\". Use: nn, nearest-neighbor, 2opt, two-opt"
            ));
        }
    };

    // 5. Build query and execute
    let query = RouteQuery {
        targets: target_selection,
        from: route_from,
        warp_range: effective_warp_range,
        within_ly: *within,
        max_targets: *max_targets,
        algorithm,
        return_to_start: *round_trip,
    };

    let result = execute_route(model, &query).map_err(|e| e.to_string())?;
    let theme = Theme::default_dark();
    Ok(format_route(&result, model, &theme))
}

fn dispatch_galaxy(
    model: &mut GalaxyModel,
    session: &mut SessionState,
    selection: Option<&str>,
) -> Result<String, String> {
    let Some(selection) = selection else {
        let discovered = model
            .discovered_galaxies()
            .into_iter()
            .map(|index| {
                let galaxy = Galaxy::by_index(index);
                format!("{} ({})", galaxy.name, galaxy.index)
            })
            .collect::<Vec<_>>();
        let discovered = if discovered.is_empty() {
            "(none)".to_string()
        } else {
            discovered.join(", ")
        };
        let active = Galaxy::by_index(model.active_galaxy);
        return Ok(format!(
            "Active galaxy: {} ({})\nDiscovered galaxies: {discovered}\n",
            active.name, active.index
        ));
    };

    let index = resolve_galaxy(selection)?;
    let galaxy = Galaxy::by_index(index);
    model.set_active_galaxy(index);
    session.sync_from_model(model);
    let count = model.spatial_for(index).map_or(0, |spatial| spatial.size());
    Ok(format!(
        "Switched to {} (galaxy {}). {count} systems loaded.\n",
        galaxy.name, galaxy.index
    ))
}

fn dispatch_list(model: &GalaxyModel, target: &ListTarget) -> Result<String, String> {
    let theme = nms_theme();

    match target {
        ListTarget::Galaxies { galaxy_type } => {
            let type_filter = galaxy_type
                .as_ref()
                .map(|t| {
                    t.parse::<GalaxyType>()
                        .map_err(|e| format!("Invalid galaxy type: {e}"))
                })
                .transpose()?;

            let mut builder = Builder::default();
            builder.push_record(["Index", "Name", "Type"]);

            for i in 0..=255u8 {
                let g = Galaxy::by_index(i);
                if let Some(ref tf) = type_filter
                    && g.galaxy_type != *tf
                {
                    continue;
                }
                builder.push_record([
                    g.index.to_string(),
                    g.name.to_string(),
                    g.galaxy_type.to_string(),
                ]);
            }
            builder.push_record(["", "", ""]);
            Ok(build_table(builder, &["GALAXIES"], &theme, "Galaxies"))
        }

        ListTarget::Biomes => {
            let mut builder = Builder::default();
            builder.push_record(["Name", "Variants"]);
            for biome in ALL_BIOMES {
                let biome_name = biome.to_string();
                let variants: Vec<String> = ALL_BIOME_SUBTYPES
                    .iter()
                    .filter_map(|sub| {
                        let sub_name = format!("{sub:?}");
                        sub_name
                            .strip_prefix(&biome_name)
                            .map(|suffix| suffix.to_string())
                    })
                    .collect();
                let variants_str = if variants.is_empty() {
                    "—".to_string()
                } else {
                    variants.join(", ")
                };
                builder.push_record([biome_name, variants_str]);
            }
            builder.push_record(["".to_string(), "".to_string()]);
            Ok(build_table(builder, &["BIOMES"], &theme, "Biomes"))
        }

        ListTarget::Glyphs => {
            let mut builder = Builder::default();
            builder.push_record(["Hex", "Name", "Abbreviation", "Emoji Approximation"]);
            for info in &GLYPH_TABLE {
                builder.push_record([
                    info.hex_char.to_string(),
                    info.name.to_string(),
                    info.abbrev.to_string(),
                    info.emoji.to_string(),
                ]);
            }
            builder.push_record(["", "", "", ""]);
            Ok(build_table(
                builder,
                &["PORTAL", "GLYPHS"],
                &theme,
                "Glyphs",
            ))
        }

        ListTarget::Bases { limit, all } => {
            if model.bases.is_empty() {
                return Ok("  No bases found.\n".into());
            }
            let mut bases: Vec<_> = model.bases.values().collect();
            bases.sort_by_cached_key(|base| base.name.to_lowercase());

            let total = bases.len();
            let effective_limit = if *all || *limit == 0 { total } else { *limit };
            let showing = total.min(effective_limit);

            let mut builder = Builder::default();
            builder.push_record(["Name", "Type", "Galaxy", "Address", "Portal Glyphs"]);

            for base in bases.iter().take(effective_limit) {
                let galaxy = Galaxy::by_index(base.address.reality_index);
                let hex = format!("{:012X}", base.address.packed());
                let glyphs = hex_to_emoji(&hex);
                builder.push_record([
                    base.name.clone(),
                    base_type_label(&base.base_type).to_string(),
                    galaxy.name.to_string(),
                    hex,
                    glyphs,
                ]);
            }
            builder.push_record(["", "", "", "", ""]);

            let mut out = build_table(builder, &["BASES"], &theme, "Bases");
            if showing < total {
                out.push_str(&format!(
                    "\n  Showing {showing} of {total} bases (use --all to show all)"
                ));
            }
            Ok(out)
        }

        ListTarget::TerrainTypes => {
            let terrain_types: &[(u8, &str, &str)] = &[
                (0, "None", "No specific terrain type"),
                (1, "Standard", "Default terrain generation"),
                (2, "HighQuality", "Enhanced terrain detail"),
                (3, "Structure", "Structural formations"),
                (4, "Beam", "Beam-shaped formations"),
                (5, "Hexagon", "Hexagonal terrain patterns"),
                (6, "FractCube", "Fractal cube formations"),
                (7, "Bubble", "Bubble-shaped terrain"),
                (8, "Shards", "Shard crystal formations"),
                (9, "Contour", "Contoured terrain features"),
                (10, "Shell", "Shell-shaped formations"),
                (11, "BoneSpire", "Bone spire formations"),
                (12, "WireCell", "Wire cell structures"),
                (13, "HydroGarden", "Hydroponic garden terrain"),
                (14, "HugePlant", "Giant plant formations"),
                (15, "HugeLush", "Giant lush vegetation"),
                (16, "HugeRing", "Giant ring formations"),
                (17, "HugeRock", "Giant rock formations"),
                (18, "HugeScorch", "Giant scorched formations"),
                (19, "HugeToxic", "Giant toxic formations"),
                (20, "Variant_A", "Terrain variant A"),
                (21, "Variant_B", "Terrain variant B"),
                (22, "Variant_C", "Terrain variant C"),
                (23, "Variant_D", "Terrain variant D"),
                (24, "Infested", "Infested terrain generation"),
                (25, "Swamp", "Swamp terrain generation"),
                (26, "Lava", "Volcanic lava terrain"),
                (27, "Worlds", "Worlds terrain generation"),
                (28, "Remix_A", "Terrain remix A"),
                (29, "Remix_B", "Terrain remix B"),
                (30, "Remix_C", "Terrain remix C"),
                (31, "Remix_D", "Terrain remix D"),
            ];

            let mut builder = Builder::default();
            builder.push_record(["Index", "Name", "Description"]);
            for (idx, name, desc) in terrain_types {
                builder.push_record([idx.to_string(), name.to_string(), desc.to_string()]);
            }
            builder.push_record(["", "", ""]);
            Ok(build_table(builder, &["TERRAIN", "TYPES"], &theme, "Types"))
        }

        ListTarget::Systems { limit, all } => {
            if model.systems.is_empty() {
                return Ok("  No systems found.\n".into());
            }
            let mut systems: Vec<_> = model.systems.values().collect();
            systems.sort_by(|a, b| {
                let a_name = a.name.as_deref().unwrap_or("");
                let b_name = b.name.as_deref().unwrap_or("");
                match (a_name.is_empty(), b_name.is_empty()) {
                    (true, false) => std::cmp::Ordering::Greater,
                    (false, true) => std::cmp::Ordering::Less,
                    _ => a_name.to_lowercase().cmp(&b_name.to_lowercase()),
                }
            });

            let total = systems.len();
            let effective_limit = if *all || *limit == 0 { total } else { *limit };
            let showing = total.min(effective_limit);

            let mut builder = Builder::default();
            builder.push_record(["Name", "Discovered Planets", "Address", "Portal Glyphs"]);

            for sys in systems.iter().take(effective_limit) {
                let name = sys.name.as_deref().unwrap_or("-");
                let planet_count = sys.planets.len();
                let hex = format!("{:012X}", sys.address.packed());
                let glyphs = hex_to_emoji(&hex);
                builder.push_record([name.to_string(), planet_count.to_string(), hex, glyphs]);
            }
            builder.push_record(["", "", "", ""]);

            let mut out = build_table(builder, &["SYSTEMS"], &theme, "Systems");
            if showing < total {
                out.push_str(&format!(
                    "\n  Showing {showing} of {total} systems (use --all to show all)"
                ));
            }
            Ok(out)
        }
    }
}

fn dispatch_show(model: &GalaxyModel, target: &ShowTarget) -> Result<String, String> {
    let query = match target {
        ShowTarget::System { name } => ShowQuery::System(name.clone()),
        ShowTarget::Base { name } => ShowQuery::Base(name.clone()),
    };
    let result = execute_show(model, &query).map_err(|e| e.to_string())?;
    let theme = Theme::default_dark();
    Ok(format_show_result(&result, &theme))
}

fn dispatch_set(
    model: &GalaxyModel,
    session: &mut SessionState,
    target: &SetTarget,
) -> Result<String, String> {
    match target {
        SetTarget::Position { name } => session.set_position_base(name, model),
        SetTarget::Biome { name } => {
            let biome: Biome = name.parse().map_err(|e| format!("Invalid biome: {e}"))?;
            Ok(session.set_biome_filter(biome))
        }
        SetTarget::WarpRange { ly } => Ok(session.set_warp_range(*ly)),
    }
}

fn dispatch_reset(model: &GalaxyModel, session: &mut SessionState, target: &str) -> String {
    match target.to_lowercase().as_str() {
        "position" | "pos" => session.reset_position(model),
        "biome" => session.clear_biome_filter().into(),
        "warp-range" | "warp" => session.clear_warp_range().into(),
        "all" | "" => session.reset_all(model).into(),
        other => format!("Unknown reset target: {other}. Use: position, biome, warp-range, all"),
    }
}

fn dispatch_convert(
    glyphs: &Option<String>,
    coords: &Option<String>,
    ga: &Option<String>,
    voxel: &Option<String>,
    ssi: Option<u16>,
    planet: u8,
    galaxy: &str,
) -> Result<String, String> {
    let reality_index = resolve_galaxy(galaxy)?;

    let addr = if let Some(g) = glyphs {
        parse_glyphs(g, reality_index)?
    } else if let Some(c) = coords {
        GalacticAddress::from_signal_booster(c.trim(), planet, reality_index)
            .map_err(|e| format!("Invalid coordinates: {e}"))?
    } else if let Some(a) = ga {
        parse_glyphs(a, reality_index)?
    } else if let Some(v) = voxel {
        let solar_system_index = ssi.ok_or("--ssi is required when using --voxel")?;
        parse_voxel(v, solar_system_index, planet, reality_index)?
    } else {
        return Err("Specify --glyphs, --coords, --ga, or --voxel".into());
    };

    Ok(format_all_formats(&addr))
}

fn parse_glyphs(input: &str, reality_index: u8) -> Result<GalacticAddress, String> {
    let trimmed = input.trim();
    let trimmed = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);

    let portal =
        PortalAddress::parse_mixed(trimmed).map_err(|e| format!("Invalid portal glyphs: {e}"))?;

    let ga = portal.to_galactic_address();
    Ok(GalacticAddress::from_packed(ga.packed(), reality_index))
}

fn parse_voxel(
    input: &str,
    solar_system_index: u16,
    planet_index: u8,
    reality_index: u8,
) -> Result<GalacticAddress, String> {
    let parts: Vec<&str> = input.trim().split(',').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Voxel position must be X,Y,Z (3 comma-separated integers), got \"{input}\""
        ));
    }

    let x: i16 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid voxel X: \"{}\"", parts[0].trim()))?;
    let y: i8 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid voxel Y: \"{}\"", parts[1].trim()))?;
    let z: i16 = parts[2]
        .trim()
        .parse()
        .map_err(|_| format!("Invalid voxel Z: \"{}\"", parts[2].trim()))?;

    Ok(GalacticAddress::new(
        x,
        y,
        z,
        solar_system_index,
        planet_index,
        reality_index,
    ))
}

fn resolve_galaxy(input: &str) -> Result<u8, String> {
    let trimmed = input.trim();

    if let Ok(idx) = trimmed.parse::<u16>() {
        if idx > 255 {
            return Err(format!("Galaxy index out of range: {idx} (must be 0-255)"));
        }
        return Ok(idx as u8);
    }

    let lower = trimmed.to_lowercase();
    for i in 0..=255u8 {
        let galaxy = Galaxy::by_index(i);
        if galaxy.name.to_lowercase() == lower {
            return Ok(i);
        }
    }

    Err(format!(
        "Unknown galaxy: \"{trimmed}\". Use a number 0-255 or a name like \"Euclid\"."
    ))
}

fn base_type_label(bt: &BaseType) -> &'static str {
    match bt {
        BaseType::HomePlanetBase => "home",
        BaseType::FreighterBase => "freighter",
        BaseType::ExternalPlanetBase => "external",
        _ => "unknown",
    }
}

fn format_all_formats(addr: &GalacticAddress) -> String {
    let galaxy = Galaxy::by_index(addr.reality_index);
    let portal = addr.to_portal_address();
    let theme = nms_theme();

    let mut builder = Builder::default();
    builder.push_record(["Format", "Value"]);
    builder.push_record(["Portal Glyphs", &portal.to_emoji_string()]);
    builder.push_record(["Hex Glyphs", &format!("{:012X}", addr.packed())]);
    builder.push_record(["Abbreviated", &portal.to_abbrev_string()]);
    builder.push_record(["Signal Booster", &addr.to_signal_booster()]);
    builder.push_record(["Galactic Address", &format!("0x{:012X}", addr.packed())]);
    builder.push_record([
        "Voxel Position",
        &format!(
            "X={}, Y={}, Z={}",
            addr.voxel_x(),
            addr.voxel_y(),
            addr.voxel_z()
        ),
    ]);
    builder.push_record([
        "System Index",
        &format!(
            "{} (0x{:03X})",
            addr.solar_system_index(),
            addr.solar_system_index()
        ),
    ]);
    builder.push_record(["Planet Index", &addr.planet_index().to_string()]);
    builder.push_record([
        "Galaxy",
        &format!("{} ({})", galaxy.name, addr.reality_index),
    ]);
    builder.push_record(["", ""]);

    build_table(
        builder,
        &["COORDINATE", "CONVERSIONS"],
        &theme,
        "Conversions",
    )
}

fn help_text() -> String {
    "\
NMS Copilot -- Interactive Galaxy Explorer

Commands:
  find       Search planets by biome, distance, name
  galaxy     Show discovered galaxies or select a galaxy view
  list       List galaxies, biomes, glyphs, bases, systems, terrain-types
  map        Open interactive galaxy map
  route      Plan a route through discovered systems
  show       Show system or base details
  stats      Display aggregate galaxy statistics
  convert    Convert between coordinate formats
  set        Set session context (position, biome, warp-range)
  reset      Reset session state (position, biome, warp-range, all)
  status     Show current session state
  info       Show loaded model summary
  help       Show this help message
  exit/quit  Exit the REPL

Live updates are shown between commands when file watching is enabled.

Examples:
  galaxy
  galaxy Eissentam
  galaxy \"Hilbert Dimension\"
  find --biome Lush --nearest 5
  route --biome Lush --warp-range 2500
  route --target \"Alpha Base\" --target \"Beta Base\"
  show system 0x050003AB8C07
  show base \"Acadia National Park\"
  stats --biomes
  convert --glyphs 01717D8A4EA2
  set biome Lush
  set position \"Home Base\"
  set warp-range 2500
  reset biome
  status
"
    .into()
}
