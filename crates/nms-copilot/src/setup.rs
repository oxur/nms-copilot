//! Interactive setup wizard for selecting an NMS save directory and save slot.
//!
//! The `nms-copilot` binary's startup calls the wizard after failed save
//! auto-detection on an interactive non-headless terminal, or for explicit
//! `--setup`, after an explicit `--save` option has taken precedence. Other
//! library callers may invoke the public wizard directly. Its entry point is
//! [`run_setup_wizard`], and its errors are described by [`SetupError`].

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};
use owo_colors::OwoColorize;

use nms_save::locate::{
    self, AccountDir, SaveSlot, group_into_slots, list_accounts, list_saves, nms_save_dir,
};

/// Build the dialoguer theme with our color scheme.
///
/// - Prompt text in cyan
/// - Active item prefix (`>`) in bright green
/// - Active items in default (account names are pre-colored in yellow)
fn wizard_theme() -> ColorfulTheme {
    use dialoguer::console::{Style, style};
    ColorfulTheme {
        prompt_style: Style::new().for_stderr().cyan(),
        active_item_prefix: style(">".to_string()).for_stderr().green().bright(),
        active_item_style: Style::new().for_stderr(),
        ..ColorfulTheme::default()
    }
}

/// Errors reported while interacting with the setup wizard.
///
/// `Cancelled` represents dialoguer interaction errors, including but not
/// limited to an explicit user cancellation. `NoInstallation` means the
/// prompted path did not exist. `Locate` carries a save-discovery error, and
/// `Io` carries an I/O error from configuration or data-directory handling.
///
/// # Examples
///
/// ```
/// use nms_copilot::setup::SetupError;
/// use nms_save::locate::LocateError;
/// use std::io;
///
/// let errors = [
///     SetupError::Cancelled,
///     SetupError::NoInstallation,
///     LocateError::NoHomeDir.into(),
///     io::Error::other("fixture").into(),
/// ];
/// let actual = errors.map(|error| error.to_string());
/// assert_eq!(
///     actual,
///     [
///         "setup cancelled by user".to_owned(),
///         "no NMS installation found".to_owned(),
///         "could not determine home/data directory".to_owned(),
///         "fixture".to_owned(),
///     ]
/// );
/// ```
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    /// A dialoguer interaction error occurred.
    ///
    /// It includes, but is not limited to, explicit user cancellation.
    #[error("setup cancelled by user")]
    Cancelled,

    /// The literal path entered at the prompt did not exist.
    #[error("no NMS installation found")]
    NoInstallation,

    /// A transparently carried save-discovery [`locate::LocateError`].
    #[error(transparent)]
    Locate(#[from] locate::LocateError),

    /// A configuration or data-directory I/O error occurred.
    ///
    /// It transparently carries [`std::io::Error`] for config, data-directory,
    /// read, serialization, or write errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Interactively select an NMS save file and optionally save its paths.
///
/// The wizard uses an existing platform-default save directory when available;
/// otherwise it prompts for a path, trims the input, and requires that it
/// exists. It does not expand `~`, canonicalize the path, or parse save
/// contents. A recognized account directory containing saves is selected directly
/// when found in its parent listing; otherwise account subdirectories are
/// listed. One account is selected automatically and multiple accounts are
/// offered in a menu. Accounts are not restricted to Steam. Save files are
/// grouped by numbered slot; one slot is selected automatically and multiple
/// slots are offered in slot order. The selected slot uses its most-recent
/// manual or auto save, with the manual save winning an equal modification
/// time. This is filename and metadata selection, not gameplay/save validation.
///
/// A confirmation prompt defaults to yes. On confirmation, the wizard writes
/// `dir`, `file`, and `format = "auto"` to the `[save]` table in the config
/// returned by [`crate::paths::config_path`], after
/// [`crate::paths::ensure_data_dir`]. Other TOML root tables are retained, but
/// the complete `[save]` table is replaced. Readable malformed TOML is treated
/// as a new empty config; comments and the previous save table are not
/// guaranteed to survive. The write is direct and non-atomic, with no rollback
/// if an error occurs. Declining returns the selected path without writing
/// config. The function returns an owned [`PathBuf`] only after an optional
/// config write succeeds; it does not activate that config in an existing
/// object, start a watcher/model, or validate the save.
///
/// # Examples
///
/// ```no_run
/// # fn main() -> Result<(), nms_copilot::setup::SetupError> {
/// let selected = nms_copilot::setup::run_setup_wizard()?;
/// println!("Selected {}", selected.display());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`SetupError::Cancelled`] for dialoguer interaction errors,
/// [`SetupError::NoInstallation`] when the prompted path does not exist,
/// [`SetupError::Locate`] for save-discovery failures, or [`SetupError::Io`]
/// for config/data-directory/read/serialization/write I/O errors. A failure can
/// occur after the user has interacted with the wizard.
///
/// # Panics
///
/// Standard-output writes can panic. When saving configuration, unavailable
/// home-directory resolution can panic through `config_path`; internal
/// selection assumptions also use `unwrap` for an account or a nonempty slot's
/// most-recent save.
pub fn run_setup_wizard() -> Result<PathBuf, SetupError> {
    let theme = wizard_theme();

    println!();
    println!("{}", "NMS Copilot Setup".magenta().bold());
    println!();
    println!("No save file configured. Let's find your No Man's Sky save file.");
    println!();

    // Step 1: Find the NMS save directory
    let save_dir = find_save_directory(&theme)?;

    // Step 2: Select an account
    let account = select_account(&save_dir, &theme)?;

    // Step 3: Select a save slot
    let save_path = select_save_slot(account.path(), &theme)?;

    println!();
    println!("Selected: {}", save_path.display());

    // Step 4: Offer to save config
    if Confirm::with_theme(&theme)
        .with_prompt("Save these settings to ~/.nms-copilot/config.toml?")
        .default(true)
        .interact()
        .map_err(|_| SetupError::Cancelled)?
    {
        save_config_to_file(account.path(), &save_path, "auto")?;
        println!("Settings saved.");
    } else {
        println!("Using selection for this session only.");
    }

    println!();
    Ok(save_path)
}

/// Find the NMS save directory, falling back to user input.
fn find_save_directory(theme: &ColorfulTheme) -> Result<PathBuf, SetupError> {
    match nms_save_dir() {
        Ok(dir) if dir.exists() => {
            println!("{} {}", "Found NMS save directory:".cyan(), dir.display());
            Ok(dir)
        }
        _ => {
            println!("Could not auto-detect NMS save directory.");
            println!("Common locations:");
            println!("  macOS:   ~/Library/Application Support/HelloGames/NMS/");
            println!("  Windows: %APPDATA%\\HelloGames\\NMS\\");
            println!("  Linux:   ~/.local/share/Steam/steamapps/compatdata/275850/pfx/...");
            println!();

            let input: String = Input::with_theme(theme)
                .with_prompt("Enter path to your NMS save directory")
                .interact_text()
                .map_err(|_| SetupError::Cancelled)?;

            let path = PathBuf::from(input.trim());
            if !path.exists() {
                return Err(SetupError::NoInstallation);
            }
            Ok(path)
        }
    }
}

/// Select an account directory. Auto-selects if there is only one.
fn select_account(save_dir: &Path, theme: &ColorfulTheme) -> Result<AccountDir, SetupError> {
    // If the user pointed directly to an account dir (contains save*.hg), use it
    if list_saves(save_dir).is_ok()
        && let Some(parent) = save_dir.parent()
        && let Ok(accounts) = list_accounts(parent)
    {
        let matching: Vec<_> = accounts
            .into_iter()
            .filter(|a| a.path() == save_dir)
            .collect();
        if let Some(account) = matching.into_iter().next() {
            println!(
                "Using account: {} ({})",
                account.name().yellow(),
                account.kind()
            );
            return Ok(account);
        }
    }

    let accounts = list_accounts(save_dir)?;

    if accounts.len() == 1 {
        let account = accounts.into_iter().next().unwrap();
        println!(
            "{} {} ({})",
            "Found 1 account:".cyan(),
            account.name().yellow(),
            account.kind()
        );
        return Ok(account);
    }

    println!("{}:", format!("Found {} accounts", accounts.len()).cyan());

    let labels: Vec<String> = accounts
        .iter()
        .map(|a| format!("{} ({})", a.name().yellow(), a.kind()))
        .collect();

    let selection = Select::with_theme(theme)
        .with_prompt("Select an account")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|_| SetupError::Cancelled)?;

    Ok(accounts.into_iter().nth(selection).unwrap())
}

/// Select a save slot from an account directory. Auto-selects if only one slot.
fn select_save_slot(account_dir: &Path, theme: &ColorfulTheme) -> Result<PathBuf, SetupError> {
    let saves = list_saves(account_dir)?;
    let slots = group_into_slots(&saves);

    if slots.is_empty() {
        return Err(SetupError::Locate(locate::LocateError::NoSaveFiles(
            account_dir.to_path_buf(),
        )));
    }

    if slots.len() == 1 {
        let slot = &slots[0];
        let save = slot.most_recent().unwrap();
        println!(
            "{} Slot {} ({}, {})",
            "Found 1 save slot:".cyan(),
            slot.slot(),
            save.save_type(),
            format_mtime(save.modified())
        );
        return Ok(save.path().to_path_buf());
    }

    println!("{}", format!("Found {} save slots:", slots.len()).cyan());

    let labels: Vec<String> = slots.iter().map(format_slot_label).collect();

    let selection = Select::with_theme(theme)
        .with_prompt("Select a save slot")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|_| SetupError::Cancelled)?;

    let selected = &slots[selection];
    let save = selected.most_recent().unwrap();
    Ok(save.path().to_path_buf())
}

/// Format a save slot for display in the selection menu.
fn format_slot_label(slot: &SaveSlot) -> String {
    let manual = if slot.manual().is_some() {
        "manual"
    } else {
        "-"
    };
    let auto = if slot.auto().is_some() { "auto" } else { "-" };

    let recent = slot
        .most_recent()
        .map(|s| format!("{} ({})", s.save_type(), format_mtime(s.modified())))
        .unwrap_or_default();

    format!(
        "Slot {:>2}  [{}/{}]  most recent: {}",
        slot.slot(),
        manual,
        auto,
        recent
    )
}

/// Format a modification time as a human-readable relative string.
fn format_mtime(time: SystemTime) -> String {
    match SystemTime::now().duration_since(time) {
        Ok(d) => {
            let hours = d.as_secs() / 3600;
            let days = d.as_secs() / 86400;
            if hours == 0 {
                "just now".to_string()
            } else if days == 0 {
                format!("{hours}h ago")
            } else {
                format!("{days}d ago")
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Save the selected save file settings to the config file.
///
/// Merges with any existing config, only updating the `[save]` section.
fn save_config_to_file(dir: &Path, file: &Path, format: &str) -> std::io::Result<()> {
    let config_path = crate::paths::config_path();
    crate::paths::ensure_data_dir()?;

    let config = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        toml::from_str::<toml::Value>(&content).unwrap_or(toml::Value::Table(toml::map::Map::new()))
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let config = update_save_config(config, dir, file, format);
    let serialized = toml::to_string_pretty(&config).map_err(std::io::Error::other)?;
    std::fs::write(&config_path, serialized)
}

fn update_save_config(
    mut config: toml::Value,
    dir: &Path,
    file: &Path,
    format: &str,
) -> toml::Value {
    if !config.is_table() {
        config = toml::Value::Table(toml::map::Map::new());
    }

    let root = config
        .as_table_mut()
        .expect("config value was normalized to a table");
    let mut save_table = toml::map::Map::new();
    save_table.insert(
        "dir".to_string(),
        toml::Value::String(dir.display().to_string()),
    );
    save_table.insert(
        "file".to_string(),
        toml::Value::String(file.display().to_string()),
    );
    save_table.insert(
        "format".to_string(),
        toml::Value::String(format.to_string()),
    );
    root.insert("save".to_string(), toml::Value::Table(save_table));

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::time::Duration;

    #[test]
    fn test_format_mtime_just_now() {
        let now = SystemTime::now();
        assert_eq!(format_mtime(now), "just now");
    }

    #[test]
    fn test_format_mtime_hours_ago() {
        let time = SystemTime::now() - Duration::from_secs(7200);
        let result = format_mtime(time);
        assert!(result.contains("h ago"), "expected 'h ago', got: {result}");
    }

    #[test]
    fn test_format_mtime_days_ago() {
        let time = SystemTime::now() - Duration::from_secs(86400 * 3);
        let result = format_mtime(time);
        assert!(result.contains("d ago"), "expected 'd ago', got: {result}");
    }

    #[test]
    fn test_format_mtime_future_time() {
        let time = SystemTime::now() + Duration::from_secs(3600);
        assert_eq!(format_mtime(time), "unknown");
    }

    #[test]
    fn test_format_slot_label_manual_only() {
        let slot = build_test_slot(1, true, false);
        let label = format_slot_label(&slot);
        assert!(label.contains("Slot  1"));
        assert!(label.contains("manual/-"));
        assert!(label.contains("Manual"));
    }

    #[test]
    fn test_format_slot_label_both() {
        let slot = build_test_slot(2, true, true);
        let label = format_slot_label(&slot);
        assert!(label.contains("Slot  2"));
        assert!(label.contains("manual/auto"));
    }

    #[test]
    fn test_setup_error_display() {
        let err = SetupError::Cancelled;
        assert_eq!(err.to_string(), "setup cancelled by user");

        let err = SetupError::NoInstallation;
        assert_eq!(err.to_string(), "no NMS installation found");
    }

    #[test]
    fn test_update_save_config_preserves_mcp_section() {
        let config = toml::from_str::<toml::Value>(
            r#"
            [mcp]
            host = "127.0.0.1"
            port = 5055
        "#,
        )
        .unwrap();

        let updated = update_save_config(
            config,
            Path::new("/Users/test/NMS/st_123"),
            Path::new("/Users/test/NMS/st_123/save5.hg"),
            "auto",
        );

        let serialized = toml::to_string_pretty(&updated).unwrap();
        assert!(serialized.contains("[save]"));
        assert!(serialized.contains("[mcp]"));

        let parsed: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(
            parsed.save.file.as_deref().unwrap().to_str().unwrap(),
            "/Users/test/NMS/st_123/save5.hg"
        );
        assert_eq!(parsed.mcp_http_addr().to_string(), "127.0.0.1:5055");
    }

    /// Helper to build a SaveSlot for testing via the locate module.
    fn build_test_slot(slot_num: u8, manual: bool, auto: bool) -> SaveSlot {
        let dir = tempfile::tempdir().unwrap();

        // Create save files for the specified slot
        if manual {
            let file_index = if slot_num == 1 {
                // save.hg for slot 1 manual
                "save.hg".to_string()
            } else {
                format!("save{}.hg", slot_num * 2 - 1)
            };
            std::fs::write(dir.path().join(&file_index), b"manual").unwrap();
        }
        if auto {
            let file_index = format!("save{}.hg", slot_num * 2);
            std::fs::write(dir.path().join(&file_index), b"auto").unwrap();
        }

        let saves = list_saves(dir.path()).unwrap();
        let slots = group_into_slots(&saves);
        slots.into_iter().find(|s| s.slot() == slot_num).unwrap()
    }
}
