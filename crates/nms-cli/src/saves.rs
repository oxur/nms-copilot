//! `nms saves` command -- list all save slots.

use std::time::SystemTime;

use nms_query::table::{Builder, build_table, nms_theme};
use nms_save::locate::{
    AccountDir, LocateError, SaveSlot, group_into_slots, list_accounts, list_saves,
    nms_save_dir_checked,
};

fn slots_for_account(account: &AccountDir) -> Result<Option<Vec<SaveSlot>>, LocateError> {
    match list_saves(account.path()) {
        Ok(saves) => Ok(Some(group_into_slots(&saves))),
        Err(LocateError::NoSaveFiles(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let save_dir = nms_save_dir_checked()?;
    let accounts = list_accounts(&save_dir)?;
    let theme = nms_theme();

    for account in &accounts {
        println!("Account: {} ({})", account.name(), account.kind());

        let slots = match slots_for_account(account)? {
            Some(slots) => slots,
            None => {
                println!("  No save files found.\n");
                continue;
            }
        };

        if slots.is_empty() {
            println!("  No save slots found.\n");
            continue;
        }

        let mut builder = Builder::default();
        builder.push_record(["Number", "Manual", "Auto", "Most Recent"]);
        for slot in &slots {
            let manual = if slot.manual().is_some() { "yes" } else { "-" };
            let auto = if slot.auto().is_some() { "yes" } else { "-" };
            let recent = slot
                .most_recent()
                .map(|s| format!("{} ({})", s.save_type(), format_mtime(s.modified())))
                .unwrap_or_default();
            builder.push_record([
                slot.slot().to_string(),
                manual.to_string(),
                auto.to_string(),
                recent,
            ]);
        }
        builder.push_record(["", "", "", ""]);
        println!(
            "{}",
            build_table(builder, &["SAVE", "SLOTS"], &theme, "Slots")
        );
    }
    Ok(())
}

fn format_mtime(time: SystemTime) -> String {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let days_ago = SystemTime::now()
                .duration_since(time)
                .map(|d| d.as_secs() / 86400)
                .unwrap_or(0);
            if days_ago == 0 {
                let hours_ago = SystemTime::now()
                    .duration_since(time)
                    .map(|d| d.as_secs() / 3600)
                    .unwrap_or(0);
                if hours_ago == 0 {
                    "just now".to_string()
                } else {
                    format!("{hours_ago}h ago")
                }
            } else if days_ago < 30 {
                format!("{days_ago}d ago")
            } else {
                // Rough timestamp
                let _ = secs;
                format!("{days_ago}d ago")
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn test_format_mtime_recent() {
        let now = SystemTime::now();
        let result = format_mtime(now);
        assert_eq!(result, "just now");
    }

    #[test]
    fn test_format_mtime_hours_ago() {
        let time = SystemTime::now() - Duration::from_secs(7200);
        let result = format_mtime(time);
        assert!(result.contains("h ago"));
    }

    #[test]
    fn test_format_mtime_days_ago() {
        let time = SystemTime::now() - Duration::from_secs(86400 * 3);
        let result = format_mtime(time);
        assert!(result.contains("d ago"));
    }

    #[test]
    fn test_format_mtime_epoch() {
        let result = format_mtime(SystemTime::UNIX_EPOCH);
        assert!(result.contains("d ago"));
    }

    fn test_account() -> (TempDir, AccountDir) {
        let root = TempDir::new().unwrap();
        let account_path = root.path().join("DefaultUser");
        fs::create_dir(&account_path).unwrap();
        let account = list_accounts(root.path()).unwrap().remove(0);
        assert_eq!(account.path(), account_path);
        (root, account)
    }

    #[test]
    fn test_slots_for_account_reports_empty_directory_without_error() {
        let (_root, account) = test_account();
        assert_eq!(slots_for_account(&account).unwrap(), None);
    }

    #[test]
    fn test_slots_for_account_returns_discovered_slots() {
        let (_root, account) = test_account();
        fs::write(account.path().join("save.hg"), b"fixture").unwrap();

        let slots = slots_for_account(&account).unwrap().unwrap();
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot(), 1);
        assert!(slots[0].manual().is_some());
        assert!(slots[0].auto().is_none());
    }

    #[test]
    fn test_slots_for_account_propagates_not_a_directory_io_error() {
        let (_root, account) = test_account();
        let account_path = account.path().to_path_buf();
        fs::remove_dir(&account_path).unwrap();
        fs::write(&account_path, b"not a directory").unwrap();

        let error = slots_for_account(&account).unwrap_err();
        assert!(matches!(
            error,
            LocateError::Io(ref io_error)
                if io_error.kind() == std::io::ErrorKind::NotADirectory
        ));
    }
}
