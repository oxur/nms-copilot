use nms_core::{address::GalacticAddress, biome::Biome, player::PlayerState, system::{Planet, System}};
use nms_graph::GalaxyModel;
use nms_query::find::{execute_find, FindQuery, ReferencePoint};
use nms_query::show::{execute_show, ShowQuery, ShowResult};

fn system(galaxy: u8, x: i16, name: &str, planet: &str, infested: bool) -> System {
    System::new(GalacticAddress::new(x, 0, 0, 1, 0, galaxy), Some(name.into()),
        Some("Explorer".into()), None,
        vec![Planet::new(1, Some(Biome::Lush), None, infested, Some(planet.into()), None)])
}

fn main() {
    let out = std::env::args().nth(1).expect("fixture output path");
    for reverse in [false, true] {
        let mut model = GalaxyModel::new();
        let mut systems = vec![system(0, 0, "Shared", "Zero", false), system(9, 0, "Shared", "Nine", true)];
        if reverse { systems.reverse(); }
        for sys in systems { model.insert_system(sys); }
        for selected in [0, 9] {
            model.set_active_galaxy(selected);
            let resolved = model.system_by_name("SHARED").map(|(_, s)| s.reality_index());
            println!("reverse={reverse} selected={selected} name_galaxy={resolved:?}");
        }
    }
    let mut model = GalaxyModel::new();
    model.player_state = Some(PlayerState::new(GalacticAddress::new(0, 0, 0, 1, 0, 0), 0, None, None, 0, 0, 0));
    model.insert_system(system(0, 0, "Shared", "Zero", false));
    model.insert_system(system(9, 0, "Shared", "Nine", true));
    for query in [FindQuery { biome: Some(Biome::Lush), ..Default::default() }, FindQuery { infested: Some(true), ..Default::default() }] {
        let results = execute_find(&model, &query).unwrap();
        for r in results {
            println!("biome={:?} infested={:?} planet={} system_galaxy={} paired={} distance={}",
                query.biome, query.infested, r.planet.name.as_deref().unwrap(), r.system.reality_index(),
                r.system.planets.contains(&r.planet), r.distance_ly);
        }
    }
    let query = FindQuery { from: ReferencePoint::Address(GalacticAddress::new(0, 0, 0, 1, 0, 9)), nearest: Some(1), ..Default::default() };
    let r = execute_find(&model, &query).unwrap();
    println!("explicit_nine_active_zero_result_galaxy={}", r[0].system.reality_index());
    for i in 1..=3 { model.insert_system(system(0, i, "Distractor", "Skip", false)); }
    model.insert_system(system(0, 4, "Target", "Keep", false));
    let query = FindQuery { nearest: Some(1), name_pattern: Some("Keep".into()), ..Default::default() };
    println!("nearest_postfilter_count={}", execute_find(&model, &query).unwrap().len());
    model.set_active_galaxy(9);
    println!("current_position_selected_nine={:?}", execute_find(&model, &FindQuery::default()).map(|r| r.len()));
    if let ShowResult::System(r) = execute_show(&model, &ShowQuery::System(format!("{:012X}", GalacticAddress::new(0, 0, 0, 1, 0, 9).packed()))).unwrap() {
        println!("show_nine_player_zero_distance={:?}", r.distance_from_player);
    }
    // The fixture is a small typed 0/9 model, not a claim about save-field galaxy extraction.
    let mut fixture = GalaxyModel::new();
    fixture.player_state = Some(PlayerState::new(GalacticAddress::new(0, 0, 0, 1, 0, 0), 0, None, None, 7, 8, 9));
    fixture.insert_system(system(0, 0, "Shared", "Zero", false));
    fixture.insert_system(system(9, 0, "Shared", "Nine", true));
    fixture.set_active_galaxy(9);
    let data = nms_cache::extract_cache_data(&fixture, 4720);
    let bytes = nms_cache::serialize::serialize(&data).unwrap();
    std::fs::write(out, &bytes).unwrap();
    println!("fixture_bytes={} header={:?}", bytes.len(), &bytes[..16]);
}
