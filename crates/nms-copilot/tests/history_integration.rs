use reedline::{FileBackedHistory, History, HistoryItem};

#[test]
fn history_file_can_be_created() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_history.txt");

    let history = FileBackedHistory::with_file(100, path.clone()).unwrap();
    drop(history);

    // File is created by FileBackedHistory
    assert!(path.exists());
}

#[test]
fn history_persists_across_instances() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_history.txt");

    // Write an entry
    {
        let mut history = FileBackedHistory::with_file(100, path.clone()).unwrap();
        history
            .save(HistoryItem::from_command_line("find --biome Lush"))
            .unwrap();
    }

    // Read it back in a new instance
    {
        let history = FileBackedHistory::with_file(100, path.clone()).unwrap();
        let count = history.count_all().unwrap();
        assert!(count >= 1, "Expected at least 1 history entry, got {count}");
    }
}

#[test]
fn paths_module_returns_expected_filenames() {
    let history = nms_copilot::paths::history_path();
    assert_eq!(history.file_name().unwrap(), "history.txt");

    let config = nms_copilot::paths::config_path();
    assert_eq!(config.file_name().unwrap(), "config.toml");

    let cache = nms_copilot::paths::cache_path();
    assert_eq!(cache.file_name().unwrap(), "galaxy.rkyv");
}
