//! `nms saves` command -- list all save slots.

use std::time::SystemTime;

use nms_save::locate::{group_into_slots, list_accounts, list_saves, nms_save_dir_checked};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let save_dir = nms_save_dir_checked()?;
    let accounts = list_accounts(&save_dir)?;

    for account in &accounts {
        println!("Account: {} ({})", account.name(), account.kind());

        let saves = match list_saves(account.path()) {
            Ok(s) => s,
            Err(_) => {
                println!("  No save files found.\n");
                continue;
            }
        };
        let slots = group_into_slots(&saves);

        if slots.is_empty() {
            println!("  No save slots found.\n");
            continue;
        }

        println!(
            "  {:<6} {:<10} {:<10} Most Recent",
            "Slot", "Manual", "Auto"
        );
        for slot in &slots {
            let manual = if slot.manual().is_some() { "yes" } else { "-" };
            let auto = if slot.auto().is_some() { "yes" } else { "-" };
            let recent = slot
                .most_recent()
                .map(|s| format!("{} ({})", s.save_type(), format_mtime(s.modified())))
                .unwrap_or_default();
            println!(
                "  {:<6} {:<10} {:<10} {}",
                slot.slot(),
                manual,
                auto,
                recent
            );
        }
        println!();
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
    use std::time::Duration;

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
}
