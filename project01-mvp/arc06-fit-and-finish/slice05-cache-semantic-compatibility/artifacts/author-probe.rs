use std::path::Path;
use nms_cache::{load_or_rebuild, read_cache, write_cache};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cache = Path::new(&args[2]);
    let save = Path::new(&args[3]);
    if args[1] == "poison" {
        let mut data = read_cache(cache)?;
        data.save_version = 9999;
        write_cache(&data, cache)?;
        println!("cached_save_version=9999");
    } else {
        let result = load_or_rebuild(cache, save, false)?;
        println!("was_cached={} save_version={} systems={} planets={}", result.was_cached, result.save_version, result.model.system_count(), result.model.planet_count());
    }
    Ok(())
}
