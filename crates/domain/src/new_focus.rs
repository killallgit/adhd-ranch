use serde::{Deserialize, Deserializer, Serialize};

use crate::error::DomainError;
use crate::timer::TimerPreset;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct NewFocus {
    title: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    timer_preset: Option<TimerPreset>,
}

impl NewFocus {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, DomainError> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        Ok(Self {
            title,
            description: description.into(),
            timer_preset: None,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn timer_preset(&self) -> Option<&TimerPreset> {
        self.timer_preset.as_ref()
    }

    #[must_use]
    pub fn with_timer_preset(mut self, timer_preset: Option<TimerPreset>) -> Self {
        self.timer_preset = timer_preset;
        self
    }
}

impl<'de> Deserialize<'de> for NewFocus {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Raw {
            title: String,
            #[serde(default)]
            description: String,
            #[serde(default)]
            timer_preset: Option<TimerPreset>,
        }
        let raw = Raw::deserialize(d)?;
        let nf = NewFocus::new(raw.title, raw.description).map_err(serde::de::Error::custom)?;
        Ok(nf.with_timer_preset(raw.timer_preset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_focus_new_accepts_valid_title() {
        let nf = NewFocus::new("Ship it", "the bug").unwrap();
        assert_eq!(nf.title, "Ship it");
        assert_eq!(nf.description, "the bug");
        assert_eq!(nf.timer_preset, None);
    }

    #[test]
    fn new_focus_new_rejects_empty_title() {
        assert_eq!(
            NewFocus::new("", "desc").unwrap_err(),
            DomainError::EmptyTitle
        );
    }

    #[test]
    fn new_focus_new_rejects_whitespace_title() {
        assert_eq!(
            NewFocus::new("   ", "desc").unwrap_err(),
            DomainError::EmptyTitle
        );
    }
}
