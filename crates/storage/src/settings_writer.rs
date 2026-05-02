use std::io;
use std::path::Path;

use adhd_ranch_domain::Settings;

use crate::atomic::atomic_write;

pub fn write_settings(path: &Path, settings: &Settings) -> io::Result<()> {
    atomic_write(path, settings.to_yaml().as_bytes())
}

#[cfg(test)]
mod tests {
    use adhd_ranch_domain::{Alerts, Caps, DisplayConfig, Widget};
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn write_settings_round_trips() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.yaml");
        let settings = Settings {
            caps: Caps {
                max_focuses: 3,
                max_tasks_per_focus: 4,
            },
            alerts: Alerts {
                system_notifications: false,
            },
            widget: Widget {
                always_on_top: true,
            },
            displays: DisplayConfig::default(),
        };
        write_settings(&path, &settings).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert_eq!(Settings::parse_yaml(&raw), settings);
    }

    #[test]
    fn write_settings_preserves_all_keys() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("settings.yaml");
        let original = Settings {
            caps: Caps {
                max_focuses: 7,
                max_tasks_per_focus: 10,
            },
            alerts: Alerts {
                system_notifications: true,
            },
            widget: Widget {
                always_on_top: false,
            },
            displays: DisplayConfig::default(),
        };
        write_settings(&path, &original).unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        let mut updated = Settings::parse_yaml(&raw);
        updated.widget.always_on_top = true;
        write_settings(&path, &updated).unwrap();

        let final_raw = std::fs::read_to_string(&path).unwrap();
        let final_settings = Settings::parse_yaml(&final_raw);
        assert_eq!(final_settings.caps.max_focuses, 7);
        assert_eq!(final_settings.caps.max_tasks_per_focus, 10);
        assert!(final_settings.alerts.system_notifications);
        assert!(final_settings.widget.always_on_top);
    }
}
