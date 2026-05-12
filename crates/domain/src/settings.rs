use serde::{Deserialize, Serialize};

use crate::notification::NotificationSettings;

pub const DEFAULT_MAX_FOCUSES: usize = 5;
pub const DEFAULT_MAX_TASKS_PER_FOCUS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct Widget {
    pub always_on_top: bool,
    pub confirm_delete: bool,
}

impl Default for Widget {
    fn default() -> Self {
        Self {
            always_on_top: false,
            confirm_delete: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct Caps {
    pub max_focuses: usize,
    pub max_tasks_per_focus: usize,
}

impl Default for Caps {
    fn default() -> Self {
        Self {
            max_focuses: DEFAULT_MAX_FOCUSES,
            max_tasks_per_focus: DEFAULT_MAX_TASKS_PER_FOCUS,
        }
    }
}

/// Which monitor indices have an active overlay window. Default: primary only (index 0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct DisplayConfig {
    pub enabled_indices: Vec<usize>,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            enabled_indices: vec![0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct Settings {
    pub caps: Caps,
    pub notifications: NotificationSettings,
    pub widget: Widget,
    pub displays: DisplayConfig,
}

impl Settings {
    pub fn to_yaml(&self) -> String {
        let enabled: Vec<String> = self
            .displays
            .enabled_indices
            .iter()
            .map(|i| i.to_string())
            .collect();
        let mut notification_keys: Vec<&String> = self.notifications.sources.keys().collect();
        notification_keys.sort();
        let mut notifications = String::from("notifications:\n");
        for k in notification_keys {
            let v = self.notifications.sources.get(k).copied().unwrap_or(true);
            notifications.push_str(&format!("  {k}: {v}\n"));
        }
        format!(
            "caps:\n  max_focuses: {}\n  max_tasks_per_focus: {}\n{notifications}widget:\n  always_on_top: {}\n  confirm_delete: {}\ndisplays:\n  enabled: {}\n",
            self.caps.max_focuses,
            self.caps.max_tasks_per_focus,
            self.widget.always_on_top,
            self.widget.confirm_delete,
            enabled.join(","),
        )
    }

    pub fn parse_yaml(input: &str) -> Settings {
        let mut settings = Settings::default();
        let mut section: &str = "";
        for raw in input.lines() {
            let line = strip_comment(raw);
            if line.trim().is_empty() {
                continue;
            }
            let indented = line.starts_with(' ') || line.starts_with('\t');
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_suffix(':') {
                if !indented {
                    section = match name {
                        "caps" => "caps",
                        "notifications" => "notifications",
                        "widget" => "widget",
                        "displays" => "displays",
                        _ => "",
                    };
                }
                continue;
            }
            let Some((key, value)) = trimmed.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match (section, key) {
                ("caps", "max_focuses") => {
                    if let Ok(n) = value.parse() {
                        settings.caps.max_focuses = n;
                    }
                }
                ("caps", "max_tasks_per_focus") => {
                    if let Ok(n) = value.parse() {
                        settings.caps.max_tasks_per_focus = n;
                    }
                }
                ("notifications", k) => {
                    if let Some(b) = parse_bool(value) {
                        settings.notifications.sources.insert(k.to_string(), b);
                    }
                }
                ("widget", "always_on_top") => {
                    if let Some(b) = parse_bool(value) {
                        settings.widget.always_on_top = b;
                    }
                }
                ("widget", "confirm_delete") => {
                    if let Some(b) = parse_bool(value) {
                        settings.widget.confirm_delete = b;
                    }
                }
                ("displays", "enabled") => {
                    settings.displays.enabled_indices = value
                        .split(',')
                        .filter_map(|s| s.trim().parse().ok())
                        .collect();
                }
                _ => {}
            }
        }
        settings
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(idx) => &line[..idx],
        None => line,
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value {
        "true" | "yes" | "on" => Some(true),
        "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::{FocusesOverCapSource, TimerExpiredSource};

    #[test]
    fn empty_input_yields_defaults() {
        let s = Settings::parse_yaml("");
        assert_eq!(s.caps.max_focuses, 5);
        assert_eq!(s.caps.max_tasks_per_focus, 7);
        assert!(s.notifications.is_enabled(&TimerExpiredSource));
        assert!(s.notifications.is_enabled(&FocusesOverCapSource));
    }

    #[test]
    fn parses_notifications_section() {
        let s = Settings::parse_yaml(
            "notifications:\n  timer_expired: false\n  focuses_over_cap: true\n",
        );
        assert!(!s.notifications.is_enabled(&TimerExpiredSource));
        assert!(s.notifications.is_enabled(&FocusesOverCapSource));
    }

    #[test]
    fn missing_notifications_section_defaults_all_enabled() {
        let s = Settings::parse_yaml("caps:\n  max_focuses: 9\n");
        assert_eq!(s.caps.max_focuses, 9);
        assert_eq!(s.caps.max_tasks_per_focus, 7);
        assert!(s.notifications.is_enabled(&TimerExpiredSource));
        assert!(s.notifications.is_enabled(&FocusesOverCapSource));
    }

    #[test]
    fn invalid_values_fall_back_to_defaults() {
        let s = Settings::parse_yaml("caps:\n  max_focuses: many\n");
        assert_eq!(s.caps.max_focuses, 5);
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let s = Settings::parse_yaml(
            "# comment\ncaps:\n  # nested comment\n  max_focuses: 2 # trailing\n\n",
        );
        assert_eq!(s.caps.max_focuses, 2);
    }

    #[test]
    fn widget_always_on_top_defaults_false() {
        let s = Settings::parse_yaml("");
        assert!(!s.widget.always_on_top);
    }

    #[test]
    fn parses_widget_always_on_top_true() {
        let s = Settings::parse_yaml("widget:\n  always_on_top: true\n");
        assert!(s.widget.always_on_top);
    }

    #[test]
    fn invalid_always_on_top_falls_back_to_default() {
        let s = Settings::parse_yaml("widget:\n  always_on_top: maybe\n");
        assert!(!s.widget.always_on_top);
    }

    #[test]
    fn confirm_delete_defaults_true() {
        let s = Settings::parse_yaml("");
        assert!(s.widget.confirm_delete);
    }

    #[test]
    fn parses_confirm_delete_false() {
        let s = Settings::parse_yaml("widget:\n  confirm_delete: false\n");
        assert!(!s.widget.confirm_delete);
    }

    #[test]
    fn to_yaml_round_trips() {
        let mut notifications = NotificationSettings::default();
        notifications.set(&TimerExpiredSource, false);
        notifications.set(&FocusesOverCapSource, true);
        let s = Settings {
            caps: Caps {
                max_focuses: 3,
                max_tasks_per_focus: 4,
            },
            notifications,
            widget: Widget {
                always_on_top: true,
                confirm_delete: false,
            },
            displays: DisplayConfig {
                enabled_indices: vec![0, 2],
            },
        };
        assert_eq!(Settings::parse_yaml(&s.to_yaml()), s);
    }

    #[test]
    fn displays_default_is_primary_only() {
        let s = Settings::parse_yaml("");
        assert_eq!(s.displays.enabled_indices, vec![0]);
    }

    #[test]
    fn parses_displays_enabled_multi() {
        let s = Settings::parse_yaml("displays:\n  enabled: 0,1,2\n");
        assert_eq!(s.displays.enabled_indices, vec![0, 1, 2]);
    }

    #[test]
    fn parses_displays_enabled_single() {
        let s = Settings::parse_yaml("displays:\n  enabled: 1\n");
        assert_eq!(s.displays.enabled_indices, vec![1]);
    }

    #[test]
    fn invalid_display_indices_ignored() {
        let s = Settings::parse_yaml("displays:\n  enabled: 0,bad,2\n");
        assert_eq!(s.displays.enabled_indices, vec![0, 2]);
    }
}
