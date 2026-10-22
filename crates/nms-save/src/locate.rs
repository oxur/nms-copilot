//! NMS save file discovery: platform-specific paths, account directories, save files.

use std::fmt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Error returned by save file discovery operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LocateError {
    #[error("could not determine home/data directory")]
    NoHomeDir,

    #[error("NMS save directory not found: {0}")]
    SaveDirNotFound(PathBuf),

    #[error("no account directories found in {0}")]
    NoAccountDirs(PathBuf),

    #[error("no save files found in {0}")]
    NoSaveFiles(PathBuf),

    #[error("unsupported platform for NMS save auto-detection")]
    UnsupportedPlatform,

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Account directory
// ---------------------------------------------------------------------------

/// The kind of NMS account directory.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccountKind {
    /// Steam account with numeric Steam ID.
    Steam(u64),
    /// GOG "DefaultUser" account.
    Gog,
    /// Unrecognized directory name.
    Unknown(String),
}

impl fmt::Display for AccountKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Steam(id) => write!(f, "Steam ({id})"),
            Self::Gog => write!(f, "GOG"),
            Self::Unknown(name) => write!(f, "Unknown ({name})"),
        }
    }
}

/// An NMS account directory (e.g., `st_76561198025707979` or `DefaultUser`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountDir {
    path: PathBuf,
    kind: AccountKind,
}

impl AccountDir {
    /// Full path to the account directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The account kind (Steam, GOG, or Unknown).
    pub fn kind(&self) -> &AccountKind {
        &self.kind
    }

    /// The directory name component (e.g., `st_76561198025707979`).
    pub fn name(&self) -> &str {
        self.path.file_name().and_then(|n| n.to_str()).unwrap_or("")
    }
}

/// Parse a directory name into an [`AccountKind`].
fn parse_account_kind(name: &str) -> AccountKind {
    if name == "DefaultUser" {
        return AccountKind::Gog;
    }
    if let Some(id_str) = name.strip_prefix("st_")
        && let Ok(id) = id_str.parse::<u64>()
    {
        return AccountKind::Steam(id);
    }
    AccountKind::Unknown(name.to_string())
}

// ---------------------------------------------------------------------------
// Save file
// ---------------------------------------------------------------------------

/// Whether a save file is a manual or auto save.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SaveType {
    Manual,
    Auto,
}

impl fmt::Display for SaveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manual => write!(f, "Manual"),
            Self::Auto => write!(f, "Auto"),
        }
    }
}

/// A discovered NMS save file with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveFile {
    path: PathBuf,
    slot: u8,
    save_type: SaveType,
    modified: SystemTime,
}

impl SaveFile {
    /// Full path to the `.hg` file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Save slot number (1-15).
    pub fn slot(&self) -> u8 {
        self.slot
    }

    /// Whether this is a manual or auto save.
    pub fn save_type(&self) -> SaveType {
        self.save_type
    }

    /// File modification time.
    pub fn modified(&self) -> SystemTime {
        self.modified
    }

    /// Path to the corresponding metadata file (`mf_save*.hg`).
    pub fn metadata_path(&self) -> PathBuf {
        let name = self
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("save.hg");
        self.path.with_file_name(format!("mf_{name}"))
    }
}

/// A paired manual + auto save for a single slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSlot {
    slot: u8,
    manual: Option<SaveFile>,
    auto: Option<SaveFile>,
}

impl SaveSlot {
    /// Slot number (1-15).
    pub fn slot(&self) -> u8 {
        self.slot
    }

    /// The manual save for this slot, if present.
    pub fn manual(&self) -> Option<&SaveFile> {
        self.manual.as_ref()
    }

    /// The auto save for this slot, if present.
    pub fn auto(&self) -> Option<&SaveFile> {
        self.auto.as_ref()
    }

    /// The most recently modified save in this slot (manual or auto).
    pub fn most_recent(&self) -> Option<&SaveFile> {
        match (&self.manual, &self.auto) {
            (Some(m), Some(a)) => {
                if m.modified >= a.modified {
                    Some(m)
                } else {
                    Some(a)
                }
            }
            (Some(m), None) => Some(m),
            (None, Some(a)) => Some(a),
            (None, None) => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Save filename parsing
// ---------------------------------------------------------------------------

/// Parse a save filename into (slot, save_type).
///
/// Returns `None` unless the name is `save.hg` or a numbered save index from 2 through 30,
/// or when it names a metadata file.
fn parse_save_filename(name: &str) -> Option<(u8, SaveType)> {
    if !name.ends_with(".hg") || name.starts_with("mf_") {
        return None;
    }

    let stem = name.strip_suffix(".hg")?;

    if stem == "save" {
        // save.hg = file index 1 → slot 1, manual
        return Some((1, SaveType::Manual));
    }

    let num_str = stem.strip_prefix("save")?;
    let file_index: u8 = num_str.parse().ok()?;
    if !(2..=30).contains(&file_index) {
        return None;
    }

    // odd index = manual, even = auto
    let slot = file_index.div_ceil(2);
    let save_type = if file_index.is_multiple_of(2) {
        SaveType::Auto
    } else {
        SaveType::Manual
    };

    Some((slot, save_type))
}

// ---------------------------------------------------------------------------
// Platform-specific directory resolution
// ---------------------------------------------------------------------------

/// Return the platform-specific NMS save root directory.
///
/// - **macOS**: `~/Library/Application Support/HelloGames/NMS/`
/// - **Windows**: `%APPDATA%\HelloGames\NMS\`
/// - **Linux**: `~/.local/share/Steam/steamapps/compatdata/275850/pfx/drive_c/users/steamuser/AppData/Roaming/HelloGames/NMS/`
///
/// Does NOT verify the directory exists on disk.
pub fn nms_save_dir() -> Result<PathBuf, LocateError> {
    nms_save_dir_impl()
}

#[cfg(target_os = "macos")]
fn nms_save_dir_impl() -> Result<PathBuf, LocateError> {
    let data = dirs::data_dir().ok_or(LocateError::NoHomeDir)?;
    Ok(data.join("HelloGames").join("NMS"))
}

#[cfg(target_os = "windows")]
fn nms_save_dir_impl() -> Result<PathBuf, LocateError> {
    let data = dirs::data_dir().ok_or(LocateError::NoHomeDir)?;
    Ok(data.join("HelloGames").join("NMS"))
}

#[cfg(target_os = "linux")]
fn nms_save_dir_impl() -> Result<PathBuf, LocateError> {
    let home = dirs::home_dir().ok_or(LocateError::NoHomeDir)?;
    Ok(home.join(".local/share/Steam/steamapps/compatdata/275850/pfx/drive_c/users/steamuser/AppData/Roaming/HelloGames/NMS"))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn nms_save_dir_impl() -> Result<PathBuf, LocateError> {
    Err(LocateError::UnsupportedPlatform)
}

/// Like [`nms_save_dir`] but verifies the directory exists on disk.
pub fn nms_save_dir_checked() -> Result<PathBuf, LocateError> {
    let dir = nms_save_dir()?;
    if dir.exists() {
        Ok(dir)
    } else {
        Err(LocateError::SaveDirNotFound(dir))
    }
}

// ---------------------------------------------------------------------------
// Directory listing
// ---------------------------------------------------------------------------

/// List all account directories inside the NMS save root.
///
/// Returns directories matching `st_*` (Steam), `DefaultUser` (GOG),
/// and any other subdirectories as [`AccountKind::Unknown`].
pub fn list_accounts(save_dir: &Path) -> Result<Vec<AccountDir>, LocateError> {
    let mut accounts = Vec::new();

    for entry in std::fs::read_dir(save_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let kind = parse_account_kind(&name);
        accounts.push(AccountDir {
            path: entry.path(),
            kind,
        });
    }

    if accounts.is_empty() {
        return Err(LocateError::NoAccountDirs(save_dir.to_path_buf()));
    }

    accounts.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(accounts)
}

/// List all save files (`save*.hg`, excluding `mf_save*.hg`) in an account directory.
///
/// Results are sorted by modification time, newest first.
pub fn list_saves(account_dir: &Path) -> Result<Vec<SaveFile>, LocateError> {
    let mut saves = Vec::new();

    for entry in std::fs::read_dir(account_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if let Some((slot, save_type)) = parse_save_filename(&name) {
            let modified = entry.metadata()?.modified()?;
            saves.push(SaveFile {
                path: entry.path(),
                slot,
                save_type,
                modified,
            });
        }
    }

    if saves.is_empty() {
        return Err(LocateError::NoSaveFiles(account_dir.to_path_buf()));
    }

    // Newest first
    saves.sort_by_key(|save| std::cmp::Reverse(save.modified));
    Ok(saves)
}

/// Group save files into slot pairs (manual + auto).
///
/// Returns slots sorted by slot number. Each slot contains at most one
/// manual and one auto save.
pub fn group_into_slots(saves: &[SaveFile]) -> Vec<SaveSlot> {
    let max_slot = saves.iter().map(|s| s.slot).max().unwrap_or(0);
    let mut slots: Vec<SaveSlot> = (1..=max_slot)
        .map(|n| SaveSlot {
            slot: n,
            manual: None,
            auto: None,
        })
        .collect();

    for save in saves {
        let idx = (save.slot - 1) as usize;
        if idx < slots.len() {
            match save.save_type {
                SaveType::Manual => slots[idx].manual = Some(save.clone()),
                SaveType::Auto => slots[idx].auto = Some(save.clone()),
            }
        }
    }

    // Remove empty slots
    slots.retain(|s| s.manual.is_some() || s.auto.is_some());
    slots
}

// ---------------------------------------------------------------------------
// Convenience finders
// ---------------------------------------------------------------------------

/// Find the most recently modified save file across all accounts.
///
/// Chains [`nms_save_dir_checked`] → [`list_accounts`] → [`list_saves`]
/// and returns the single newest file.
pub fn find_most_recent_save() -> Result<SaveFile, LocateError> {
    let save_dir = nms_save_dir_checked()?;
    let accounts = list_accounts(&save_dir)?;

    let mut best: Option<SaveFile> = None;
    for account in &accounts {
        if let Ok(saves) = list_saves(account.path())
            && let Some(newest) = saves.into_iter().next()
        {
            let dominated = best.as_ref().is_none_or(|b| newest.modified > b.modified);
            if dominated {
                best = Some(newest);
            }
        }
    }

    best.ok_or(LocateError::NoSaveFiles(save_dir))
}

/// Find the most recently modified save file in a specific account directory.
pub fn find_most_recent_save_in(account_dir: &Path) -> Result<SaveFile, LocateError> {
    let saves = list_saves(account_dir)?;
    // list_saves already sorts newest-first
    Ok(saves.into_iter().next().unwrap())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File, FileTimes};
    use std::thread;
    use std::time::{Duration, UNIX_EPOCH};
    use tempfile::TempDir;

    fn save_filename(save: Option<&SaveFile>) -> Option<&str> {
        save.map(|save| save.path().file_name().unwrap().to_str().unwrap())
    }

    // -- Filename parsing --

    #[test]
    fn parse_save_hg() {
        let (slot, st) = parse_save_filename("save.hg").unwrap();
        assert_eq!(slot, 1);
        assert_eq!(st, SaveType::Manual);
    }

    #[test]
    fn parse_save2_hg() {
        let (slot, st) = parse_save_filename("save2.hg").unwrap();
        assert_eq!(slot, 1);
        assert_eq!(st, SaveType::Auto);
    }

    #[test]
    fn parse_save3_hg() {
        let (slot, st) = parse_save_filename("save3.hg").unwrap();
        assert_eq!(slot, 2);
        assert_eq!(st, SaveType::Manual);
    }

    #[test]
    fn parse_save4_hg() {
        let (slot, st) = parse_save_filename("save4.hg").unwrap();
        assert_eq!(slot, 2);
        assert_eq!(st, SaveType::Auto);
    }

    #[test]
    fn parse_save30_hg() {
        let (slot, st) = parse_save_filename("save30.hg").unwrap();
        assert_eq!(slot, 15);
        assert_eq!(st, SaveType::Auto);
    }

    #[test]
    fn test_parse_save_indices_above_slot_fifteen_are_rejected() {
        for name in ["save31.hg", "save255.hg", "save256.hg"] {
            assert_eq!(parse_save_filename(name), None, "accepted {name}");
        }
    }

    #[test]
    fn parse_mf_save_rejected() {
        assert!(parse_save_filename("mf_save.hg").is_none());
        assert!(parse_save_filename("mf_save2.hg").is_none());
    }

    #[test]
    fn parse_nonsave_rejected() {
        assert!(parse_save_filename("readme.txt").is_none());
        assert!(parse_save_filename("config.hg").is_none());
        assert!(parse_save_filename("save.json").is_none());
    }

    // -- Account kind parsing --

    #[test]
    fn account_kind_steam() {
        match parse_account_kind("st_76561198025707979") {
            AccountKind::Steam(id) => assert_eq!(id, 76561198025707979),
            other => panic!("expected Steam, got {other:?}"),
        }
    }

    #[test]
    fn account_kind_gog() {
        assert_eq!(parse_account_kind("DefaultUser"), AccountKind::Gog);
    }

    #[test]
    fn account_kind_unknown() {
        match parse_account_kind("some_other_dir") {
            AccountKind::Unknown(name) => assert_eq!(name, "some_other_dir"),
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    #[test]
    fn account_kind_steam_bad_id() {
        // st_ prefix but non-numeric ID
        match parse_account_kind("st_notanumber") {
            AccountKind::Unknown(_) => {}
            other => panic!("expected Unknown, got {other:?}"),
        }
    }

    // -- Platform path --

    #[test]
    fn nms_save_dir_contains_hellogames_nms() {
        let dir = nms_save_dir().unwrap();
        let s = dir.to_string_lossy();
        assert!(s.contains("HelloGames"), "path missing HelloGames: {s}");
        assert!(s.ends_with("NMS"), "path should end with NMS: {s}");
    }

    // -- Integration tests with temp dirs --

    #[test]
    fn list_accounts_finds_steam_and_gog() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("st_123")).unwrap();
        fs::create_dir(tmp.path().join("DefaultUser")).unwrap();

        let accounts = list_accounts(tmp.path()).unwrap();
        assert_eq!(accounts.len(), 2);

        let kinds: Vec<_> = accounts.iter().map(|a| a.kind().clone()).collect();
        assert!(kinds.contains(&AccountKind::Gog));
        assert!(kinds.contains(&AccountKind::Steam(123)));
    }

    #[test]
    fn list_accounts_skips_files() {
        let tmp = TempDir::new().unwrap();
        fs::create_dir(tmp.path().join("st_123")).unwrap();
        fs::write(tmp.path().join("not_a_dir.txt"), b"data").unwrap();

        let accounts = list_accounts(tmp.path()).unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[test]
    fn list_accounts_empty_returns_error() {
        let tmp = TempDir::new().unwrap();
        let err = list_accounts(tmp.path()).unwrap_err();
        assert!(matches!(err, LocateError::NoAccountDirs(_)));
    }

    #[test]
    fn list_saves_finds_and_sorts_by_mtime() {
        let tmp = TempDir::new().unwrap();

        // Create save files with different mtimes
        fs::write(tmp.path().join("save.hg"), b"old").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(tmp.path().join("save2.hg"), b"newer").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(tmp.path().join("save3.hg"), b"newest").unwrap();

        let saves = list_saves(tmp.path()).unwrap();
        assert_eq!(saves.len(), 3);
        // Newest first
        assert_eq!(saves[0].slot(), 2); // save3.hg = slot 2
        assert_eq!(saves[0].save_type(), SaveType::Manual);
        assert_eq!(saves[1].slot(), 1); // save2.hg = slot 1
        assert_eq!(saves[1].save_type(), SaveType::Auto);
        assert_eq!(saves[2].slot(), 1); // save.hg = slot 1
        assert_eq!(saves[2].save_type(), SaveType::Manual);
    }

    #[test]
    fn list_saves_excludes_metadata() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("save.hg"), b"data").unwrap();
        fs::write(tmp.path().join("mf_save.hg"), b"meta").unwrap();

        let saves = list_saves(tmp.path()).unwrap();
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].slot(), 1);
    }

    #[test]
    fn list_saves_empty_dir_returns_error() {
        let tmp = TempDir::new().unwrap();
        let err = list_saves(tmp.path()).unwrap_err();
        assert!(matches!(err, LocateError::NoSaveFiles(_)));
    }

    #[test]
    fn group_into_slots_pairs_correctly() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("save.hg"), b"m1").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(tmp.path().join("save2.hg"), b"a1").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(tmp.path().join("save3.hg"), b"m2").unwrap();

        let saves = list_saves(tmp.path()).unwrap();
        let slots = group_into_slots(&saves);

        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].slot(), 1);
        assert!(slots[0].manual().is_some());
        assert!(slots[0].auto().is_some());
        assert_eq!(slots[1].slot(), 2);
        assert!(slots[1].manual().is_some());
        assert!(slots[1].auto().is_none());
    }

    #[test]
    fn find_most_recent_save_in_picks_newest() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("save.hg"), b"old").unwrap();
        thread::sleep(Duration::from_millis(50));
        fs::write(tmp.path().join("save3.hg"), b"newest").unwrap();

        let newest = find_most_recent_save_in(tmp.path()).unwrap();
        assert_eq!(newest.slot(), 2);
        assert_eq!(newest.save_type(), SaveType::Manual);
    }

    #[test]
    fn save_file_metadata_path() {
        let save = SaveFile {
            path: PathBuf::from("/tmp/st_123/save3.hg"),
            slot: 2,
            save_type: SaveType::Manual,
            modified: SystemTime::UNIX_EPOCH,
        };
        assert_eq!(
            save.metadata_path(),
            PathBuf::from("/tmp/st_123/mf_save3.hg")
        );
    }

    #[test]
    fn save_slot_most_recent() {
        let older = SaveFile {
            path: PathBuf::from("/tmp/save.hg"),
            slot: 1,
            save_type: SaveType::Manual,
            modified: SystemTime::UNIX_EPOCH,
        };
        let newer = SaveFile {
            path: PathBuf::from("/tmp/save2.hg"),
            slot: 1,
            save_type: SaveType::Auto,
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(100),
        };
        let slot = SaveSlot {
            slot: 1,
            manual: Some(older),
            auto: Some(newer),
        };
        let recent = slot.most_recent().unwrap();
        assert_eq!(recent.save_type(), SaveType::Auto);
    }

    #[test]
    fn account_kind_display() {
        assert_eq!(AccountKind::Steam(12345).to_string(), "Steam (12345)");
        assert_eq!(AccountKind::Gog.to_string(), "GOG");
        assert_eq!(
            AccountKind::Unknown("foo".into()).to_string(),
            "Unknown (foo)"
        );
    }

    #[test]
    fn save_type_display() {
        assert_eq!(SaveType::Manual.to_string(), "Manual");
        assert_eq!(SaveType::Auto.to_string(), "Auto");
    }

    #[test]
    fn test_list_accounts_reports_exact_sorted_kinds_paths_and_empty_errors() {
        let root = TempDir::new().unwrap();
        let gog = root.path().join("DefaultUser");
        let steam = root.path().join("st_123");
        let unknown = root.path().join("zz_empty");
        fs::create_dir(&gog).unwrap();
        fs::create_dir(&steam).unwrap();
        fs::create_dir(&unknown).unwrap();
        fs::write(root.path().join("ordinary-file.txt"), b"not an account").unwrap();

        let accounts = list_accounts(root.path()).unwrap();
        let actual: Vec<_> = accounts
            .iter()
            .map(|account| {
                (
                    account.name(),
                    account.kind().clone(),
                    account.path().to_path_buf(),
                )
            })
            .collect();
        assert_eq!(
            actual,
            vec![
                ("DefaultUser", AccountKind::Gog, gog),
                ("st_123", AccountKind::Steam(123), steam),
                (
                    "zz_empty",
                    AccountKind::Unknown("zz_empty".to_owned()),
                    unknown.clone()
                ),
            ]
        );

        let empty_root = TempDir::new().unwrap();
        assert!(matches!(
            list_accounts(empty_root.path()),
            Err(LocateError::NoAccountDirs(path)) if path == empty_root.path()
        ));

        let file_only_root = TempDir::new().unwrap();
        fs::write(file_only_root.path().join("not-an-account"), b"file").unwrap();
        assert!(matches!(
            list_accounts(file_only_root.path()),
            Err(LocateError::NoAccountDirs(path)) if path == file_only_root.path()
        ));

        assert!(matches!(
            list_saves(&unknown),
            Err(LocateError::NoSaveFiles(path)) if path == unknown
        ));

        let regular_file = root.path().join("regular-file-account");
        fs::write(&regular_file, b"not a directory").unwrap();
        assert!(matches!(
            list_saves(&regular_file),
            Err(LocateError::Io(error)) if error.kind() == std::io::ErrorKind::NotADirectory
        ));
    }

    #[test]
    fn test_list_saves_uses_complete_literal_filename_slot_type_table() {
        let tmp = TempDir::new().unwrap();
        let account = tmp.path().join("account");
        fs::create_dir(&account).unwrap();
        let expected = [
            ("save.hg", 1, SaveType::Manual),
            ("save2.hg", 1, SaveType::Auto),
            ("save3.hg", 2, SaveType::Manual),
            ("save4.hg", 2, SaveType::Auto),
            ("save5.hg", 3, SaveType::Manual),
            ("save6.hg", 3, SaveType::Auto),
            ("save7.hg", 4, SaveType::Manual),
            ("save8.hg", 4, SaveType::Auto),
            ("save9.hg", 5, SaveType::Manual),
            ("save10.hg", 5, SaveType::Auto),
            ("save11.hg", 6, SaveType::Manual),
            ("save12.hg", 6, SaveType::Auto),
            ("save13.hg", 7, SaveType::Manual),
            ("save14.hg", 7, SaveType::Auto),
            ("save15.hg", 8, SaveType::Manual),
            ("save16.hg", 8, SaveType::Auto),
            ("save17.hg", 9, SaveType::Manual),
            ("save18.hg", 9, SaveType::Auto),
            ("save19.hg", 10, SaveType::Manual),
            ("save20.hg", 10, SaveType::Auto),
            ("save21.hg", 11, SaveType::Manual),
            ("save22.hg", 11, SaveType::Auto),
            ("save23.hg", 12, SaveType::Manual),
            ("save24.hg", 12, SaveType::Auto),
            ("save25.hg", 13, SaveType::Manual),
            ("save26.hg", 13, SaveType::Auto),
            ("save27.hg", 14, SaveType::Manual),
            ("save28.hg", 14, SaveType::Auto),
            ("save29.hg", 15, SaveType::Manual),
            ("save30.hg", 15, SaveType::Auto),
        ];
        let epoch = UNIX_EPOCH + Duration::from_secs(4_000_000_000);
        for (offset, (name, _, _)) in expected.iter().enumerate() {
            let path = account.join(name);
            fs::write(&path, b"opaque payload").unwrap();
            File::open(path)
                .unwrap()
                .set_times(
                    FileTimes::new().set_modified(epoch + Duration::from_secs(offset as u64)),
                )
                .unwrap();
        }
        for name in [
            "mf_save.hg",
            "mf_save2.hg",
            "save.hg.bak",
            "save3.hg.backup",
            "save.tmp",
            "save0.hg",
            "save1.hg",
            "save31.hg",
            "save255.hg",
            "save256.hg",
            "notes.txt",
        ] {
            let path = account.join(name);
            fs::write(&path, b"newer ignored payload").unwrap();
            File::open(path)
                .unwrap()
                .set_times(FileTimes::new().set_modified(epoch + Duration::from_secs(10_000)))
                .unwrap();
        }

        let saves = list_saves(&account).unwrap();
        let mut actual: Vec<_> = saves
            .iter()
            .map(|save| {
                (
                    save.path().file_name().unwrap().to_str().unwrap(),
                    save.slot(),
                    save.save_type(),
                )
            })
            .collect();
        actual.sort_by_key(|row| row.0);
        let mut literal = expected.to_vec();
        literal.sort_by_key(|row| row.0);
        assert_eq!(actual, literal);
        assert_eq!(saves.len(), 30);

        let slots = group_into_slots(&saves);
        let actual_groups: Vec<_> = slots
            .iter()
            .map(|slot| {
                (
                    slot.slot(),
                    save_filename(slot.manual()),
                    save_filename(slot.auto()),
                    save_filename(slot.most_recent()),
                )
            })
            .collect();
        let expected_groups = [
            (1, Some("save.hg"), Some("save2.hg"), Some("save2.hg")),
            (2, Some("save3.hg"), Some("save4.hg"), Some("save4.hg")),
            (3, Some("save5.hg"), Some("save6.hg"), Some("save6.hg")),
            (4, Some("save7.hg"), Some("save8.hg"), Some("save8.hg")),
            (5, Some("save9.hg"), Some("save10.hg"), Some("save10.hg")),
            (6, Some("save11.hg"), Some("save12.hg"), Some("save12.hg")),
            (7, Some("save13.hg"), Some("save14.hg"), Some("save14.hg")),
            (8, Some("save15.hg"), Some("save16.hg"), Some("save16.hg")),
            (9, Some("save17.hg"), Some("save18.hg"), Some("save18.hg")),
            (10, Some("save19.hg"), Some("save20.hg"), Some("save20.hg")),
            (11, Some("save21.hg"), Some("save22.hg"), Some("save22.hg")),
            (12, Some("save23.hg"), Some("save24.hg"), Some("save24.hg")),
            (13, Some("save25.hg"), Some("save26.hg"), Some("save26.hg")),
            (14, Some("save27.hg"), Some("save28.hg"), Some("save28.hg")),
            (15, Some("save29.hg"), Some("save30.hg"), Some("save30.hg")),
        ];
        assert_eq!(actual_groups, expected_groups);

        let edge = tmp.path().join("edge-cases");
        fs::create_dir(&edge).unwrap();
        for (name, seconds) in [
            ("save.hg", 100),
            ("save2.hg", 100),
            ("save3.hg", 300),
            ("save4.hg", 200),
            ("save5.hg", 200),
            ("save6.hg", 300),
            ("save7.hg", 400),
            ("save30.hg", 500),
        ] {
            let path = edge.join(name);
            fs::write(&path, b"opaque").unwrap();
            File::open(path)
                .unwrap()
                .set_times(FileTimes::new().set_modified(epoch + Duration::from_secs(seconds)))
                .unwrap();
        }
        let edge_slots = group_into_slots(&list_saves(&edge).unwrap());
        let actual_slot_numbers: Vec<_> = edge_slots.iter().map(SaveSlot::slot).collect();
        assert_eq!(actual_slot_numbers, [1, 2, 3, 4, 15]);
        let edge_row = |number| {
            let slot = edge_slots.iter().find(|slot| slot.slot() == number);
            slot.map(|slot| {
                (
                    number,
                    save_filename(slot.manual()),
                    save_filename(slot.auto()),
                    save_filename(slot.most_recent()),
                )
            })
            .unwrap_or((number, None, None, None))
        };
        let actual_edges: Vec<_> = (1..=15).map(edge_row).collect();
        let expected_edges = [
            (1, Some("save.hg"), Some("save2.hg"), Some("save.hg")),
            (2, Some("save3.hg"), Some("save4.hg"), Some("save3.hg")),
            (3, Some("save5.hg"), Some("save6.hg"), Some("save6.hg")),
            (4, Some("save7.hg"), None, Some("save7.hg")),
            (5, None, None, None),
            (6, None, None, None),
            (7, None, None, None),
            (8, None, None, None),
            (9, None, None, None),
            (10, None, None, None),
            (11, None, None, None),
            (12, None, None, None),
            (13, None, None, None),
            (14, None, None, None),
            (15, None, Some("save30.hg"), Some("save30.hg")),
        ];
        assert_eq!(actual_edges, expected_edges);
    }
}
