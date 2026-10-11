use nms_core::{address::GalacticAddress,system::{System,Planet},biome::Biome};
use nms_graph::{GalaxyModel,SystemId};
use nms_graph::route::RoutingAlgorithm;
use nms_query::route::{RouteQuery,RouteFrom,TargetSelection,execute_route};
fn main(){
 let mut m=GalaxyModel::new();
 let a0=GalacticAddress::new(0,0,0,1,0,0);let b0=GalacticAddress::new(1,0,0,2,0,0);
 let a9=GalacticAddress::new(0,0,0,1,0,9);let b9=GalacticAddress::new(1,0,0,2,0,9);
 for (a,n) in [(a0,"Zero-A"),(b0,"Zero-B"),(a9,"Nine-A"),(b9,"Nine-B")] {m.insert_system(System::new(a,Some(n.into()),None,None,vec![Planet::new(1,Some(Biome::Lush),None,false,None,None)]));}
 let id=|a:&GalacticAddress|SystemId::from_address(a);
 let nearest=m.nearest_systems(&a9,1);
 let cross=m.shortest_path(id(&a0),id(&a9));
 let reachable=m.reachable_systems(id(&a9),400.0).unwrap();
 let q=RouteQuery{targets:TargetSelection::SystemIds(vec![id(&b9)]),from:RouteFrom::Address(a9),warp_range:None,within_ly:None,max_targets:None,algorithm:RoutingAlgorithm::NearestNeighbor,return_to_start:false};
 let route=execute_route(&m,&q).unwrap();
 println!("nearest_reference9_active0={:?}",nearest.iter().map(|(i,d)|(i.reality_index(),*d)).collect::<Vec<_>>());
 println!("cross_galaxy_shortest={:?}",cross.map(|r|r.total_distance_ly));
 let mut reach:Vec<_>=reachable.iter().map(|i|(i.reality_index(),i.system_address())).collect();reach.sort();println!("reachable_start9_active0={reach:?}");
 println!("query_explicit9_route_galaxies={:?}",route.route.hops.iter().map(|h|h.system_id.reality_index()).collect::<Vec<_>>());
 println!("state_active={} systems={}",m.active_galaxy,m.system_count());
 let mut dense=GalaxyModel::new();
 for ssi in 1..=120 { let a=GalacticAddress::new(0,0,0,ssi,0,0);dense.insert_system(System::new(a,None,None,None,vec![])); }
 println!("dense_reachable={}/120",dense.reachable_systems(id(&a0),1000.0).unwrap().len());
}
