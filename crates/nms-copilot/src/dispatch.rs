//! Command dispatch -- executes REPL commands against the loaded GalaxyModel.

use nms_core::BaseType;
use nms_core::address::GalacticAddress;
use nms_core::biome::Biome;
use nms_core::galaxy::{Galaxy, GalaxyType};
use nms_graph::GalaxyModel;
use nms_graph::query::BiomeFilter;
use nms_graph::route::RoutingAlgorithm;
use nms_query::display::{format_find_results, format_route, format_show_result, format_stats};
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

/// Execute a parsed REPL action against the model, returning output text.
pub fn dispatch(
    action: &Action,
    model: &GalaxyModel,
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

            let reference = match from {
                Some(name) => ReferencePoint::Base(name.clone()),
                None => ReferencePoint::CurrentPosition,
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
    let route_from = match from {
        Some(name) => RouteFrom::Base(name.clone()),
        None => match &session.position {
            Some(pos) => RouteFrom::Address(*pos.address()),
            None => RouteFrom::CurrentPosition,
        },
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
                if let Some(ref tf) = type_filter {
                    if g.galaxy_type != *tf {
                        continue;
                    }
                }
                builder.push_record([
                    g.index.to_string(),
                    g.name.to_string(),
                    g.galaxy_type.to_string(),
                ]);
            }
            builder.push_record(["", "", ""]);
            Ok(build_table(builder, "Galaxies", &theme))
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
            Ok(build_table(builder, "Biomes", &theme))
        }

        ListTarget::Glyphs => {
            let mut builder = Builder::default();
            builder.push_record(["Hex", "Symbol", "Name"]);
            for info in &GLYPH_TABLE {
                builder.push_record([
                    info.hex_char.to_string(),
                    info.emoji.to_string(),
                    info.name.to_string(),
                ]);
            }
            builder.push_record(["", "", ""]);
            Ok(build_table(builder, "Portal Glyphs", &theme))
        }

        ListTarget::Bases { limit, all } => {
            if model.bases.is_empty() {
                return Ok("  No bases found.\n".into());
            }
            let mut bases: Vec<_> = model.bases.values().collect();
            bases.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            let total = bases.len();
            let effective_limit = if *all || *limit == 0 { total } else { *limit };
            let showing = total.min(effective_limit);

            let mut builder = Builder::default();
            builder.push_record(["Name", "Type", "Galaxy", "Address"]);

            for base in bases.iter().take(effective_limit) {
                let galaxy = Galaxy::by_index(base.address.reality_index);
                builder.push_record([
                    base.name.clone(),
                    base_type_label(&base.base_type).to_string(),
                    galaxy.name.to_string(),
                    format!("0x{:012X}", base.address.packed()),
                ]);
            }
            builder.push_record(["", "", "", ""]);

            let mut out = build_table(builder, "Bases", &theme);
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
            Ok(build_table(builder, "Terrain Types", &theme))
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
            builder.push_record(["Name", "Address", "Discovered Planets"]);

            for sys in systems.iter().take(effective_limit) {
                let name = sys.name.as_deref().unwrap_or("(unnamed)");
                let planet_count = sys.planets.len();
                builder.push_record([
                    name.to_string(),
                    format!("0x{:012X}", sys.address.packed()),
                    planet_count.to_string(),
                ]);
            }
            builder.push_record(["", "", ""]);

            let mut out = build_table(builder, "Systems", &theme);
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
    let hex = input.trim();
    let hex = hex
        .strip_prefix("0x")
        .or_else(|| hex.strip_prefix("0X"))
        .unwrap_or(hex);

    if hex.len() != 12 {
        return Err(format!(
            "Portal glyphs must be exactly 12 hex digits, got {} (\"{hex}\")",
            hex.len(),
        ));
    }

    let packed =
        u64::from_str_radix(hex, 16).map_err(|_| format!("Invalid hex in glyphs: \"{hex}\""))?;

    Ok(GalacticAddress::from_packed(packed, reality_index))
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

    format!(
        "NMS Copilot -- Coordinate Conversion\n\
         =====================================\n\
         \n\
         \x20 Portal Glyphs:     {:012X}\n\
         \x20 Signal Booster:    {}\n\
         \x20 Galactic Address:  0x{:012X}\n\
         \x20 Voxel Position:    X={}, Y={}, Z={}\n\
         \x20 System Index:      {} (0x{:03X})\n\
         \x20 Planet Index:      {}\n\
         \x20 Galaxy:            {} ({})\n",
        addr.packed(),
        addr.to_signal_booster(),
        addr.packed(),
        addr.voxel_x(),
        addr.voxel_y(),
        addr.voxel_z(),
        addr.solar_system_index(),
        addr.solar_system_index(),
        addr.planet_index(),
        galaxy.name,
        addr.reality_index,
    )
}

fn help_text() -> String {
    "\
NMS Copilot -- Interactive Galaxy Explorer

Commands:
  find       Search planets by biome, distance, name
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
