use nms_core::{address::GalacticAddress, system::{System,SystemId,Planet},biome::Biome};
use nms_graph::GalaxyModel;
fn main() {
 let out=std::env::args().nth(1).unwrap();
 let a=GalacticAddress::from_packed(0x150003AB8C07,0);
 let b=GalacticAddress::from_packed(0x150003AB8C07,9);
 println!("id_equal={}",SystemId::from_address(&a)==SystemId::from_address(&b));
 for order in [false,true] {
  let mut model=GalaxyModel::new();
  let pairs=if order {[(b,"Nine",Biome::Toxic),(a,"Zero",Biome::Lush)]}else{[(a,"Zero",Biome::Lush),(b,"Nine",Biome::Toxic)]};
  for (addr,name,biome) in pairs {model.insert_system(System::new(addr,Some(name.into()),None,None,vec![Planet::new(1,Some(biome),None,false,Some(name.into()),None)]));}
  println!("reverse={order} systems={} planets={} spatial={} nodes={}",model.system_count(),model.planet_count(),model.spatial_size(),model.graph.node_count());
 }
 let mut model=GalaxyModel::new();
 model.insert_system(System::new(b,Some("Nine".into()),None,None,vec![]));model.set_active_galaxy(9);
 let data=nms_cache::extract_cache_data(&model,9999);
 let bytes=nms_cache::serialize::serialize(&data).unwrap();std::fs::write(out,&bytes).unwrap();
 let restored=nms_cache::rebuild_model(&nms_cache::serialize::deserialize(&bytes).unwrap());
 println!("active_before={} active_after={} sentinel_version={} header={:?}",model.active_galaxy,restored.active_galaxy,data.save_version,&bytes[..16]);
}
