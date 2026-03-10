//! `nms list` command -- list reference data and model collections.

use std::path::PathBuf;

use nms_core::BaseType;
use nms_core::biome::{ALL_BIOME_SUBTYPES, ALL_BIOMES};
use nms_core::galaxy::{Galaxy, GalaxyType};
use nms_core::glyph::GLYPH_TABLE;
use nms_graph::GalaxyModel;
use nms_query::display::hex_to_emoji;
use nms_query::table::{Builder, build_table, nms_theme};

use crate::ListTargetCmd;

pub fn run(
    target: ListTargetCmd,
    save: Option<PathBuf>,
    slot: Option<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    match target {
        ListTargetCmd::Galaxies { galaxy_type } => list_galaxies(galaxy_type),
        ListTargetCmd::Biomes => list_biomes(),
        ListTargetCmd::Glyphs => list_glyphs(),
        ListTargetCmd::TerrainTypes => list_terrain_types(),
        ListTargetCmd::Bases { limit, all } => list_bases(save, slot, limit, all),
        ListTargetCmd::Systems { limit, all } => list_systems(save, slot, limit, all),
    }
}

fn load_model(
    save: Option<PathBuf>,
    slot: Option<u8>,
) -> Result<GalaxyModel, Box<dyn std::error::Error>> {
    let path = crate::resolve_save_with_slot(save, slot)?;
    let save = nms_save::parse_save_file(&path)?;
    Ok(GalaxyModel::from_save(&save))
}

fn list_galaxies(galaxy_type: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let type_filter = galaxy_type
        .as_ref()
        .map(|t| t.parse::<GalaxyType>())
        .transpose()
        .map_err(|e| format!("Invalid galaxy type: {e}"))?;

    let theme = nms_theme();
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
    print!(
        "{}",
        build_table(builder, &["GALAXIES"], &theme, "Galaxies")
    );
    Ok(())
}

fn list_biomes() -> Result<(), Box<dyn std::error::Error>> {
    let theme = nms_theme();
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
            "\u{2014}".to_string()
        } else {
            variants.join(", ")
        };
        builder.push_record([biome_name, variants_str]);
    }
    builder.push_record(["".to_string(), "".to_string()]);
    print!("{}", build_table(builder, &["BIOMES"], &theme, "Biomes"));
    Ok(())
}

fn list_glyphs() -> Result<(), Box<dyn std::error::Error>> {
    let theme = nms_theme();
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
    print!(
        "{}",
        build_table(builder, &["PORTAL", "GLYPHS"], &theme, "Glyphs")
    );
    Ok(())
}

fn list_terrain_types() -> Result<(), Box<dyn std::error::Error>> {
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

    let theme = nms_theme();
    let mut builder = Builder::default();
    builder.push_record(["Index", "Name", "Description"]);
    for (idx, name, desc) in terrain_types {
        builder.push_record([idx.to_string(), name.to_string(), desc.to_string()]);
    }
    builder.push_record(["", "", ""]);
    print!(
        "{}",
        build_table(builder, &["TERRAIN", "TYPES"], &theme, "Types")
    );
    Ok(())
}

fn list_bases(
    save: Option<PathBuf>,
    slot: Option<u8>,
    limit: usize,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let model = load_model(save, slot)?;

    if model.bases.is_empty() {
        println!("  No bases found.");
        return Ok(());
    }

    let mut bases: Vec<_> = model.bases.values().collect();
    bases.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let total = bases.len();
    let effective_limit = if all || limit == 0 { total } else { limit };
    let showing = total.min(effective_limit);

    let theme = nms_theme();
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

    print!("{}", build_table(builder, &["BASES"], &theme, "Bases"));
    if showing < total {
        println!("\n  Showing {showing} of {total} bases (use --all to show all)");
    }
    Ok(())
}

fn list_systems(
    save: Option<PathBuf>,
    slot: Option<u8>,
    limit: usize,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let model = load_model(save, slot)?;

    if model.systems.is_empty() {
        println!("  No systems found.");
        return Ok(());
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
    let effective_limit = if all || limit == 0 { total } else { limit };
    let showing = total.min(effective_limit);

    let theme = nms_theme();
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

    print!("{}", build_table(builder, &["SYSTEMS"], &theme, "Systems"));
    if showing < total {
        println!("\n  Showing {showing} of {total} systems (use --all to show all)");
    }
    Ok(())
}

fn base_type_label(bt: &BaseType) -> &'static str {
    match bt {
        BaseType::HomePlanetBase => "home",
        BaseType::FreighterBase => "freighter",
        BaseType::ExternalPlanetBase => "external",
        _ => "unknown",
    }
}
