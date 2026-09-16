use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum TimerStatus {
    Running,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum TimerPreset {
    Two,
    Four,
    Eight,
    Sixteen,
    ThirtyTwo,
    Custom(u64), // minutes
}

impl TimerPreset {
    pub fn duration_secs(&self) -> u64 {
        match self {
            Self::Two => 120,
            Self::Four => 240,
            Self::Eight => 480,
            Self::Sixteen => 960,
            Self::ThirtyTwo => 1920,
            Self::Custom(mins) => mins.saturating_mul(60),
        }
    }
}

/// What a Timer is attached to. Every Timer has exactly one Owner, and every
/// Owner has at most one Timer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum TimerOwner {
    Focus { focus_id: String },
    Task { focus_id: String, index: usize },
}

impl TimerOwner {
    pub fn focus(focus_id: impl Into<String>) -> Self {
        Self::Focus {
            focus_id: focus_id.into(),
        }
    }

    pub fn task(focus_id: impl Into<String>, index: usize) -> Self {
        Self::Task {
            focus_id: focus_id.into(),
            index,
        }
    }

    pub fn focus_id(&self) -> &str {
        match self {
            Self::Focus { focus_id } | Self::Task { focus_id, .. } => focus_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct FocusTimer {
    pub duration_secs: u64,
    pub started_at: i64,
    pub status: TimerStatus,
}

/// Returns seconds remaining, or None if the timer has expired.
/// Short-circuits on explicit Expired status regardless of clock.
pub fn timer_remaining_secs(timer: &FocusTimer, now_secs: i64) -> Option<u64> {
    if matches!(timer.status, TimerStatus::Expired) {
        return None;
    }
    let elapsed = (now_secs - timer.started_at).max(0) as u64;
    if elapsed >= timer.duration_secs {
        None
    } else {
        Some(timer.duration_secs - elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running_timer(duration_secs: u64, started_at: i64) -> FocusTimer {
        FocusTimer {
            duration_secs,
            started_at,
            status: TimerStatus::Running,
        }
    }

    #[test]
    fn remaining_at_start_equals_duration() {
        let t = running_timer(120, 1000);
        assert_eq!(timer_remaining_secs(&t, 1000), Some(120));
    }

    #[test]
    fn remaining_mid_timer_is_correct() {
        let t = running_timer(120, 1000);
        assert_eq!(timer_remaining_secs(&t, 1060), Some(60));
    }

    #[test]
    fn remaining_at_expiry_is_none() {
        let t = running_timer(120, 1000);
        assert_eq!(timer_remaining_secs(&t, 1120), None);
    }

    #[test]
    fn remaining_past_expiry_is_none() {
        let t = running_timer(120, 1000);
        assert_eq!(timer_remaining_secs(&t, 2000), None);
    }

    #[test]
    fn remaining_returns_none_when_status_expired_regardless_of_clock() {
        let t = FocusTimer {
            duration_secs: 120,
            started_at: 1000,
            status: TimerStatus::Expired,
        };
        // Clock says there's still 60s left, but status is Expired — must return None.
        assert_eq!(timer_remaining_secs(&t, 1060), None);
    }

    #[test]
    fn custom_preset_saturates_on_overflow() {
        assert_eq!(TimerPreset::Custom(u64::MAX).duration_secs(), u64::MAX);
    }

    #[test]
    fn preset_duration_secs_correct() {
        assert_eq!(TimerPreset::Two.duration_secs(), 120);
        assert_eq!(TimerPreset::Four.duration_secs(), 240);
        assert_eq!(TimerPreset::Eight.duration_secs(), 480);
        assert_eq!(TimerPreset::Sixteen.duration_secs(), 960);
        assert_eq!(TimerPreset::ThirtyTwo.duration_secs(), 1920);
        assert_eq!(TimerPreset::Custom(10).duration_secs(), 600);
    }

    #[test]
    fn focus_timer_round_trips_via_serde() {
        let t = FocusTimer {
            duration_secs: 480,
            started_at: 1_700_000_000,
            status: TimerStatus::Running,
        };
        let json = serde_json::to_string(&t).expect("serialize");
        let back: FocusTimer = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(t, back);
    }
}
