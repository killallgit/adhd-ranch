use serde::{Deserialize, Serialize};

use crate::notification::NotificationSettings;

pub const DEFAULT_MAX_FOCUSES: usize = 5;
pub const DEFAULT_MAX_TASKS_PER_FOCUS: usize = 7;

pub const DEFAULT_MAX_PEN_SIZE: u32 = 320;
/// The smallest pen the settings will accept. A crowded ranch can still divide its
/// grid below this — the floor is on what may be asked for, not on what gets drawn.
pub const MIN_PEN_SIZE: u32 = 160;
/// Above this a lone pen is the whole ranch again, which is what the cap exists to stop.
pub const MAX_PEN_SIZE: u32 = 960;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct AgentsConfig {
    pub enabled: bool,
}

/// How large a pen is allowed to grow, whatever the ranch has room for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct PenConfig {
    pub max_size: u32,
}

/// Deserialize by hand so the clamp has one home. `update_settings` hands a `Settings`
/// straight from IPC to the workflow, which persists it verbatim — a derived impl would
/// let a `max_size` of 0 reach settings.yaml and draw every pen with no room in it,
/// until the next launch parsed the file and silently rewrote it.
impl<'de> Deserialize<'de> for PenConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            max_size: u32,
        }

        Ok(Self::clamped(Wire::deserialize(deserializer)?.max_size))
    }
}

impl Default for PenConfig {
    fn default() -> Self {
        Self {
            max_size: DEFAULT_MAX_PEN_SIZE,
        }
    }
}

impl PenConfig {
    /// Every way in lands here — a hand-edited file through `parse_yaml`, and IPC
    /// through `Deserialize`. A nonsense number becomes the nearest usable one rather
    /// than a ranch with no room to walk in it.
    pub fn clamped(max_size: u32) -> Self {
        Self {
            max_size: max_size.clamp(MIN_PEN_SIZE, MAX_PEN_SIZE),
        }
    }
}

/// Minute chips the timer picker always shows. They are not stored in settings.
pub const BUILTIN_TIMER_MINUTES: [u64; 3] = [10, 30, 60];

/// Extra timer durations the picker remembers. Built-ins stay out of this list so
/// the chips do not show the same length twice.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct TimerPresets {
    pub saved_minutes: Vec<u64>,
}

impl TimerPresets {
    pub fn from_minutes(minutes: impl IntoIterator<Item = u64>) -> Self {
        let mut saved_minutes = Vec::new();
        for minutes in minutes {
            if minutes < 1
                || BUILTIN_TIMER_MINUTES.contains(&minutes)
                || saved_minutes.contains(&minutes)
            {
                continue;
            }
            saved_minutes.push(minutes);
        }
        Self { saved_minutes }
    }
}

impl<'de> Deserialize<'de> for TimerPresets {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            #[serde(default)]
            saved_minutes: Vec<u64>,
        }

        Ok(Self::from_minutes(
            Wire::deserialize(deserializer)?.saved_minutes,
        ))
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
    #[serde(default)]
    pub agents: AgentsConfig,
    #[serde(default)]
    pub pens: PenConfig,
    #[serde(default)]
    pub timers: TimerPresets,
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
        let saved = TimerPresets::from_minutes(self.timers.saved_minutes.iter().copied());
        let saved_minutes = saved
            .saved_minutes
            .iter()
            .map(|minutes| minutes.to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "caps:\n  max_focuses: {}\n  max_tasks_per_focus: {}\n{notifications}widget:\n  always_on_top: {}\n  confirm_delete: {}\ndisplays:\n  enabled: {}\nagents:\n  enabled: {}\npens:\n  max_size: {}\ntimers:\n  saved_minutes: {saved_minutes}\n",
            self.caps.max_focuses,
            self.caps.max_tasks_per_focus,
            self.widget.always_on_top,
            self.widget.confirm_delete,
            enabled.join(","),
            self.agents.enabled,
            self.pens.max_size,
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
                        "agents" => "agents",
                        "pens" => "pens",
                        "timers" => "timers",
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
                ("agents", "enabled") => {
                    if let Some(b) = parse_bool(value) {
                        settings.agents.enabled = b;
                    }
                }
                ("pens", "max_size") => {
                    if let Ok(n) = value.parse() {
                        settings.pens = PenConfig::clamped(n);
                    }
                }
                ("timers", "saved_minutes") => {
                    settings.timers = TimerPresets::from_minutes(
                        value.split(',').filter_map(|part| part.trim().parse().ok()),
                    );
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
            agents: AgentsConfig { enabled: true },
            pens: PenConfig { max_size: 400 },
            timers: TimerPresets::from_minutes([15, 45]),
        };
        assert_eq!(Settings::parse_yaml(&s.to_yaml()), s);
    }

    #[test]
    fn agents_default_to_disabled() {
        let s = Settings::parse_yaml("widget:\n  always_on_top: true\n");
        assert!(!s.agents.enabled);
    }

    #[test]
    fn parses_agents_enabled_true() {
        let s = Settings::parse_yaml("agents:\n  enabled: true\n");
        assert!(s.agents.enabled);
    }

    #[test]
    fn settings_json_without_agents_deserializes_disabled() {
        let json = serde_json::to_value(Settings::default()).unwrap();
        let mut object = json.as_object().unwrap().clone();
        object.remove("agents");

        let s: Settings = serde_json::from_value(serde_json::Value::Object(object)).unwrap();

        assert!(!s.agents.enabled);
    }

    #[test]
    fn pens_default_to_a_size_smaller_than_any_ranch() {
        let s = Settings::parse_yaml("");

        assert_eq!(s.pens.max_size, DEFAULT_MAX_PEN_SIZE);
    }

    #[test]
    fn parses_pen_max_size() {
        let s = Settings::parse_yaml("pens:\n  max_size: 480\n");

        assert_eq!(s.pens.max_size, 480);
    }

    #[test]
    fn a_pen_too_small_to_walk_in_is_raised_to_the_smallest_usable_one() {
        let s = Settings::parse_yaml("pens:\n  max_size: 4\n");

        assert_eq!(s.pens.max_size, MIN_PEN_SIZE);
    }

    #[test]
    fn a_pen_larger_than_any_ranch_is_cut_back_to_the_cap() {
        let s = Settings::parse_yaml("pens:\n  max_size: 99999\n");

        assert_eq!(s.pens.max_size, MAX_PEN_SIZE);
    }

    #[test]
    fn settings_json_without_pens_deserializes_to_the_default_size() {
        let json = serde_json::to_value(Settings::default()).unwrap();
        let mut object = json.as_object().unwrap().clone();
        object.remove("pens");

        let s: Settings = serde_json::from_value(serde_json::Value::Object(object)).unwrap();

        assert_eq!(s.pens.max_size, DEFAULT_MAX_PEN_SIZE);
    }

    #[test]
    fn a_pen_size_over_ipc_is_clamped_like_a_hand_edited_one() {
        let below: PenConfig =
            serde_json::from_value(serde_json::json!({ "max_size": 0 })).unwrap();
        let above: PenConfig =
            serde_json::from_value(serde_json::json!({ "max_size": 99_999 })).unwrap();

        assert_eq!(below.max_size, MIN_PEN_SIZE);
        assert_eq!(above.max_size, MAX_PEN_SIZE);
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

    #[test]
    fn missing_timer_section_keeps_an_empty_saved_list() {
        let s = Settings::parse_yaml("pens:\n  max_size: 480\n");
        assert!(s.timers.saved_minutes.is_empty());
    }

    #[test]
    fn saved_timer_minutes_drop_builtins_duplicates_and_junk() {
        let s = Settings::parse_yaml("timers:\n  saved_minutes: 15,10,15,0,nope,45\n");
        assert_eq!(s.timers.saved_minutes, vec![15, 45]);
    }

    #[test]
    fn settings_json_without_timers_deserializes_to_an_empty_list() {
        let json = serde_json::to_value(Settings::default()).unwrap();
        let mut object = json.as_object().unwrap().clone();
        object.remove("timers");

        let s: Settings = serde_json::from_value(serde_json::Value::Object(object)).unwrap();

        assert!(s.timers.saved_minutes.is_empty());
    }

    #[test]
    fn settings_json_normalizes_saved_timer_minutes() {
        let s: TimerPresets = serde_json::from_value(serde_json::json!({
            "saved_minutes": [10, 15, 15, 0, 90]
        }))
        .unwrap();

        assert_eq!(s.saved_minutes, vec![15, 90]);
    }
}
