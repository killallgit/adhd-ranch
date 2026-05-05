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

/// Returns a growth multiplier in 1.0..=3.0 based on elapsed time vs total duration.
/// 1.0 at t=0, 3.0 at t>=duration_secs.
pub fn growth_factor(elapsed_secs: u64, duration_secs: u64) -> f32 {
    if duration_secs == 0 {
        return 3.0;
    }
    let progress = (elapsed_secs as f32 / duration_secs as f32).clamp(0.0, 1.0);
    1.0 + progress * 2.0
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
    fn growth_factor_at_start_is_one() {
        assert!((growth_factor(0, 120) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn growth_factor_at_half_duration_is_two() {
        assert!((growth_factor(60, 120) - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn growth_factor_at_full_duration_is_three() {
        assert!((growth_factor(120, 120) - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn growth_factor_past_duration_clamped_to_three() {
        assert!((growth_factor(9999, 120) - 3.0).abs() < f32::EPSILON);
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
