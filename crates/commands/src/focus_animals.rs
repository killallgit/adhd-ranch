use adhd_ranch_domain::{Focus, TimerStatus};

use crate::render_scene::{Animal, AnimalSize, Motion, RenderSceneError, Species, SpeciesProfile};

const FOCUS_MAXIMUM_SIZE_MULTIPLIER: f64 = 3.0;

pub(crate) fn project_focus_animals(
    focuses: &[Focus],
    profile: &SpeciesProfile,
) -> Result<Vec<Animal>, RenderSceneError> {
    focuses
        .iter()
        .map(|focus| {
            let (size, motion) = match &focus.timer {
                None => (AnimalSize::fixed(profile.base_size_px)?, Motion::Walking),
                Some(timer) => {
                    let maximum_px = profile.base_size_px * FOCUS_MAXIMUM_SIZE_MULTIPLIER;
                    let size = if timer.duration_secs == 0 {
                        AnimalSize::fixed(maximum_px)?
                    } else {
                        AnimalSize::linear(
                            profile.base_size_px,
                            maximum_px,
                            timer.started_at.saturating_mul(1_000),
                            timer.duration_secs.saturating_mul(1_000),
                        )?
                    };
                    let motion = if timer.status == TimerStatus::Expired {
                        Motion::Resting
                    } else {
                        Motion::Walking
                    };
                    (size, motion)
                }
            };

            Animal::new(
                focus.id.0.clone(),
                focus.title.clone(),
                Species::Pig,
                size,
                None,
                motion,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use adhd_ranch_domain::{Focus, FocusId, FocusTimer, Task, TimerStatus};

    use crate::{
        render_scene::{AnimalSize, Motion, Species},
        species::species_profile,
    };

    use super::project_focus_animals;

    fn focus(timer: Option<FocusTimer>, tasks: Vec<Task>) -> Focus {
        Focus {
            id: FocusId("same-id".into()),
            title: "Write renderer".into(),
            description: "not rendered".into(),
            created_at: "2026-09-27T00:00:00Z".into(),
            tasks,
            timer,
        }
    }

    fn timer(duration_secs: u64, started_at: i64, status: TimerStatus) -> FocusTimer {
        FocusTimer {
            duration_secs,
            started_at,
            status,
        }
    }

    #[test]
    fn projects_source_identity_label_species_and_whole_display() {
        let profile = species_profile(Species::Pig);
        let animals = project_focus_animals(&[focus(None, vec![])], &profile).unwrap();

        assert_eq!(animals.len(), 1);
        assert_eq!(animals[0].id, "same-id");
        assert_eq!(animals[0].label, "Write renderer");
        assert_eq!(animals[0].species, Species::Pig);
        assert_eq!(animals[0].region_id, None);
        assert_eq!(animals[0].motion, Motion::Walking);
        assert_eq!(
            animals[0].size,
            AnimalSize::Fixed {
                px: profile.base_size_px
            }
        );
    }

    #[test]
    fn projects_running_and_expired_timer_curves_and_motion() {
        let profile = species_profile(Species::Pig);
        let focuses = [
            focus(
                Some(timer(120, 1_700_000_000, TimerStatus::Running)),
                vec![],
            ),
            focus(
                Some(timer(120, 1_700_000_000, TimerStatus::Expired)),
                vec![],
            ),
        ];
        let animals = project_focus_animals(&focuses, &profile).unwrap();

        let expected = AnimalSize::Linear {
            from_px: 48.0,
            to_px: 144.0,
            started_at_ms: 1_700_000_000_000,
            duration_ms: 120_000,
        };
        assert_eq!(animals[0].size, expected);
        assert_eq!(animals[0].motion, Motion::Walking);
        assert_eq!(animals[1].size, expected);
        assert_eq!(animals[1].motion, Motion::Resting);
    }

    #[test]
    fn projects_zero_duration_timer_as_fixed_maximum() {
        let profile = species_profile(Species::Pig);
        let animals = project_focus_animals(
            &[focus(
                Some(timer(0, 1_700_000_000, TimerStatus::Running)),
                vec![],
            )],
            &profile,
        )
        .unwrap();

        assert_eq!(animals[0].size, AnimalSize::Fixed { px: 144.0 });
        assert_eq!(animals[0].motion, Motion::Walking);
    }

    #[test]
    fn task_timers_do_not_affect_focus_animals() {
        let profile = species_profile(Species::Pig);
        let task = Task {
            id: "task-1".into(),
            text: "Task".into(),
            done: false,
            timer: Some(timer(120, 1_700_000_000, TimerStatus::Expired)),
        };
        let animals = project_focus_animals(&[focus(None, vec![task])], &profile).unwrap();

        assert_eq!(animals[0].size, AnimalSize::Fixed { px: 48.0 });
        assert_eq!(animals[0].motion, Motion::Walking);
    }
}
