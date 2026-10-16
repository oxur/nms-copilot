use nms_copilot::{commands, dispatch, session::{PositionContext, SessionState}};
use nms_core::{address::GalacticAddress, biome::Biome, player::{BaseType, PlayerBase, PlayerState}, system::{Planet, System}};
use nms_graph::{GalaxyModel, EdgeStrategy};
fn query(model: &mut GalaxyModel, session: &mut SessionState, line: &str) -> Result<String,String> {
    let physical = model.player_state.clone(); let active = model.active_galaxy;
    let systems = model.systems.clone(); let planets = model.planets.clone(); let bases = model.bases.clone();
    let context = format!("{session:?}");
    let action = commands::parse_line(line)?.expect("nonempty probe command");
    let result = dispatch::dispatch(&action, model, session);
    assert_eq!(model.player_state, physical); assert_eq!(model.active_galaxy, active);
    assert_eq!(model.systems,systems); assert_eq!(model.planets,planets); assert_eq!(model.bases,bases);
    assert_eq!(format!("{session:?}"),context); result
}
fn fixture(reverse: bool) -> (GalaxyModel,SessionState) {
    let mut model=GalaxyModel::new();
    let mut values=vec![(0,0,1,"Origin-0","Leaf-0"),(0,1,2,"Twin","Bud-0"),(9,0,1,"Origin-9","Leaf-9"),(9,2,2,"Twin","Bud-9")];
    if reverse { values.reverse(); }
    for (galaxy,x,ssi,name,leaf) in values {
        model.insert_system(System::new(GalacticAddress::new(x,0,0,ssi,0,galaxy),Some(name.into()),None,None,vec![Planet::new(0,Some(Biome::Lush),None,false,Some(leaf.into()),None)]));
    }
    for (galaxy,name) in [(0,"Camp-0"),(9,"Camp-9")] { model.insert_base(PlayerBase::new(name.into(),BaseType::HomePlanetBase,GalacticAddress::new(0,0,0,1,0,galaxy),[0.0;3],None)); }
    model.player_state=Some(PlayerState::new(GalacticAddress::new(0,0,0,1,0,0),0,Some(GalacticAddress::new(4,0,0,3,0,0)),Some(GalacticAddress::new(5,0,0,4,0,0)),7,8,9));
    model.build_edges(EdgeStrategy::default()); let session=SessionState::from_model(&model); (model,session)
}
fn main() {
    let mut checks=0;
    for reverse in [false,true] { for algo in ["nn","2opt"] {
        let (mut model,mut session)=fixture(reverse);
        let find=query(&mut model,&mut session,"find --nearest 1").unwrap(); assert!(find.contains("Leaf-0") && !find.contains("Leaf-9"));checks+=1;
        session.set_position_address(GalacticAddress::new(0,0,0,1,0,9));
        let route=query(&mut model,&mut session,&format!("route --target Twin --algo {algo}")).unwrap();assert!(route.contains("Origin-9") && route.contains("~800 ly") && !route.contains("Origin-0"));checks+=1;
        let route=query(&mut model,&mut session,&format!("route --target Twin --from Camp-0 --algo {algo}")).unwrap();assert!(route.contains("Origin-0") && route.contains("~400 ly") && !route.contains("Origin-9"));checks+=1;
        session.position=Some(PositionContext::PlayerPosition(GalacticAddress::new(2,0,0,2,0,9)));
        let route=query(&mut model,&mut session,&format!("route --target Twin --algo {algo}")).unwrap();assert!(route.contains("Origin-0") && route.contains("~400 ly") && !route.contains("Origin-9"));checks+=1;
        assert_eq!(query(&mut model,&mut session,"find --from Missing").unwrap_err(),"base not found: Missing");checks+=1;
        assert_eq!(query(&mut model,&mut session,&format!("route --target Twin --from Missing --algo {algo}")).unwrap_err(),"base not found: Missing");checks+=1;
        assert_eq!(query(&mut model,&mut session,&format!("route --target Missing --algo {algo}")).unwrap_err(),"target not found: Missing");checks+=1;
        assert_eq!(query(&mut model,&mut session,"find --biome Toxic --nearest 1").unwrap(),"  No results found.\n");checks+=1;
        model.player_state=None;session.position=None;
        assert_eq!(query(&mut model,&mut session,"find --nearest 1").unwrap_err(),"no player position available");checks+=1;
        assert_eq!(query(&mut model,&mut session,&format!("route --target Twin --algo {algo}")).unwrap_err(),"no player position available");checks+=1;
    }}
    println!("{checks}/40 public reference/error/state checks passed; both insertion orders and both algorithms; complete model data and session values unchanged after each query");
}
