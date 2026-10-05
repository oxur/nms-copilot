use std::{fs, path::Path, time::{SystemTime, Duration}};
use nms_cache::{CacheError, CacheCompatibilityError, load_or_rebuild, read_cache, write_cache, is_cache_fresh};
fn set_time(path: &Path, seconds: u64) -> std::io::Result<()> {
    fs::File::options().write(true).open(path)?.set_times(fs::FileTimes::new().set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir=std::env::args().nth(1).unwrap(); let dir=Path::new(&dir);
    let empty=nms_cache::serialize::deserialize(&[]).unwrap_err();
    assert!(matches!(empty, CacheError::Incompatible(CacheCompatibilityError::TruncatedHeader{actual:0})));
    println!("empty_header=TruncatedHeader(0)");
    for (name,schema,semantic) in [("schema",2u32,1u32),("semantic",1,2)] {
        let save=dir.join(format!("{name}-save.json"));let cache=dir.join(format!("{name}-cache.rkyv"));
        fs::copy(dir.join("fixture.json"),&save)?;
        let cold=load_or_rebuild(&cache,&save,false)?;assert!(!cold.was_cached);assert_eq!(cold.save_version,4720);
        let mut data=read_cache(&cache)?;data.save_version=9999;write_cache(&data,&cache)?;
        let mut bytes=fs::read(&cache)?;bytes[8..12].copy_from_slice(&schema.to_le_bytes());bytes[12..16].copy_from_slice(&semantic.to_le_bytes());fs::write(&cache,bytes)?;
        set_time(&save,1700000000)?;set_time(&cache,1700000100)?;
        assert!(is_cache_fresh(&cache,&save));
        let rebuilt=load_or_rebuild(&cache,&save,false)?;assert!(!rebuilt.was_cached);assert_eq!(rebuilt.save_version,4720);
        assert_eq!(rebuilt.model.systems,cold.model.systems);assert_eq!(rebuilt.model.planets,cold.model.planets);assert_eq!(rebuilt.model.bases,cold.model.bases);assert_eq!(rebuilt.model.player_state,cold.model.player_state);
        let current=fs::read(&cache)?;assert_eq!(&current[..16],b"NMSCACHE\x01\0\0\0\x01\0\0\0");
        set_time(&cache,1700000100)?;assert!(is_cache_fresh(&cache,&save));
        let warm=load_or_rebuild(&cache,&save,false)?;assert!(warm.was_cached);assert_eq!(warm.save_version,4720);
        println!("{name}: before_reject=fresh; rebuilt=false/4720; after_rebuild=fresh; warm=true/4720; source_fields_preserved=true");
    }
    Ok(())
}
