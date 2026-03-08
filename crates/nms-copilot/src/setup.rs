//! Interactive setup wizard for first-time NMS Copilot configuration.
//!
//! When the REPL starts with no save file configured and auto-detect fails,
//! this wizard guides the user through finding and selecting their NMS save file.

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

/// Errors that can occur during the setup wizard.
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    /// The user cancelled the setup wizard.
    #[error("setup cancelled by user")]
    Cancelled,

    /// No NMS installation could be found.
    #[error("no NMS installation found")]
    NoInstallation,

    /// Error from save file discovery.
    #[error(transparent)]
    Locate(#[from] locate::LocateError),

    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Run the interactive setup wizard.
///
/// Guides the user through selecting an NMS save file by:
/// 1. Detecting the platform-default NMS save directory
/// 2. Listing account directories
/// 3. Listing save slots within the selected account
/// 4. Optionally saving the selection to `~/.nms-copilot/config.toml`
///
/// Returns the resolved path to the selected save file.
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
                .with_prompt("Enter path to your NMS save directory (or a specific save file)")
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
    if list_saves(save_dir).is_ok() {
        if let Some(parent) = save_dir.parent() {
            if let Ok(accounts) = list_accounts(parent) {
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

    // Read existing config as toml::Value to preserve other sections
    let mut table = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        content
            .parse::<toml::Value>()
            .unwrap_or(toml::Value::Table(toml::map::Map::new()))
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    // Update [save] section
    let root = table.as_table_mut().unwrap();
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

    std::fs::write(&config_path, table.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
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
