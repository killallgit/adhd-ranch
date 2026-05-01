use serde::{Deserialize, Serialize};

pub const DEFAULT_MAX_FOCUSES: usize = 5;
pub const DEFAULT_MAX_TASKS_PER_FOCUS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Widget {
    pub always_on_top: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Alerts {
    pub system_notifications: bool,
}

impl Default for Alerts {
    fn default() -> Self {
        Self {
            system_notifications: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Settings {
    pub caps: Caps,
    pub alerts: Alerts,
    pub widget: Widget,
}

impl Settings {
    pub fn to_yaml(&self) -> String {
        format!(
            "caps:\n  max_focuses: {}\n  max_tasks_per_focus: {}\nalerts:\n  system_notifications: {}\nwidget:\n  always_on_top: {}\n",
            self.caps.max_focuses,
            self.caps.max_tasks_per_focus,
            self.alerts.system_notifications,
            self.widget.always_on_top,
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
                        "alerts" => "alerts",
                        "widget" => "widget",
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
                ("alerts", "system_notifications") => {
                    if let Some(b) = parse_bool(value) {
                        settings.alerts.system_notifications = b;
                    }
                }
                ("widget", "always_on_top") => {
                    if let Some(b) = parse_bool(value) {
                        settings.widget.always_on_top = b;
                    }
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

    #[test]
    fn empty_input_yields_defaults() {
        let s = Settings::parse_yaml("");
        assert_eq!(s.caps.max_focuses, 5);
        assert_eq!(s.caps.max_tasks_per_focus, 7);
        assert!(s.alerts.system_notifications);
    }

    #[test]
    fn parses_full_yaml() {
        let s = Settings::parse_yaml(
            "caps:\n  max_focuses: 3\n  max_tasks_per_focus: 4\nalerts:\n  system_notifications: false\n",
        );
        assert_eq!(s.caps.max_focuses, 3);
        assert_eq!(s.caps.max_tasks_per_focus, 4);
        assert!(!s.alerts.system_notifications);
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        let s = Settings::parse_yaml("caps:\n  max_focuses: 9\n");
        assert_eq!(s.caps.max_focuses, 9);
        assert_eq!(s.caps.max_tasks_per_focus, 7);
        assert!(s.alerts.system_notifications);
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
    fn to_yaml_round_trips() {
        let s = Settings {
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
        };
        assert_eq!(Settings::parse_yaml(&s.to_yaml()), s);
    }
}
