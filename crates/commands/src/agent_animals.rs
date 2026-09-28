use std::collections::BTreeMap;

use adhd_ranch_domain::session::{AgentSession, SessionActivity};

use crate::render_scene::{
    Animal, AnimalRegion, AnimalSize, Motion, RenderSceneError, Species, SpeciesProfile,
};

pub(crate) struct AgentAnimalProjection {
    pub(crate) animals: Vec<Animal>,
    pub(crate) regions: Vec<AnimalRegion>,
}

pub(crate) fn project_agent_animals(
    sessions: &[AgentSession],
    profile: &SpeciesProfile,
) -> Result<AgentAnimalProjection, RenderSceneError> {
    let animals = sessions
        .iter()
        .map(|session| {
            Animal::new(
                format!("agent:{}", session.id),
                session.name.clone(),
                Species::Pig,
                AnimalSize::fixed(profile.base_size_px)?,
                Some(session.pen.id.clone()),
                match session.activity {
                    SessionActivity::Working => Motion::Walking,
                    SessionActivity::Idle => Motion::Resting,
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut regions_by_id = BTreeMap::new();
    for session in sessions {
        regions_by_id.insert(session.pen.id.clone(), session.pen.name.clone());
    }
    let regions = regions_by_id
        .into_iter()
        .map(|(id, label)| AnimalRegion::new(id, label))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AgentAnimalProjection { animals, regions })
}

#[cfg(test)]
mod tests {
    use adhd_ranch_domain::session::{AgentSession, Pen, SessionActivity};

    use crate::{
        render_scene::{AnimalSize, Motion, Species},
        species::species_profile,
    };

    use super::{project_agent_animals, AgentAnimalProjection};

    fn session(id: &str, pen_id: &str, activity: SessionActivity) -> AgentSession {
        AgentSession {
            id: id.into(),
            name: format!("session-{id}"),
            pen: Pen {
                id: pen_id.into(),
                name: format!("region-{pen_id}"),
            },
            activity,
        }
    }

    #[test]
    fn projects_namespaced_identity_label_species_size_region_and_motion() {
        let profile = species_profile(Species::Pig);
        let AgentAnimalProjection { animals, regions } = project_agent_animals(
            &[
                session("same-id", "repo", SessionActivity::Working),
                session("idle", "repo", SessionActivity::Idle),
            ],
            &profile,
        )
        .unwrap();

        assert_eq!(animals[0].id, "agent:same-id");
        assert_eq!(animals[0].label, "session-same-id");
        assert_eq!(animals[0].species, Species::Pig);
        assert_eq!(animals[0].size, AnimalSize::Fixed { px: 48.0 });
        assert_eq!(animals[0].region_id.as_deref(), Some("repo"));
        assert_eq!(animals[0].motion, Motion::Walking);
        assert_eq!(animals[1].motion, Motion::Resting);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].id, "repo");
        assert_eq!(regions[0].label, "region-repo");
    }

    #[test]
    fn distinct_raw_session_ids_remain_distinct_after_namespacing() {
        let profile = species_profile(Species::Pig);
        let projection = project_agent_animals(
            &[
                session("one", "repo-a", SessionActivity::Working),
                session("two", "repo-b", SessionActivity::Working),
            ],
            &profile,
        )
        .unwrap();

        assert_eq!(projection.animals[0].id, "agent:one");
        assert_eq!(projection.animals[1].id, "agent:two");
        assert_eq!(projection.regions.len(), 2);
    }

    #[test]
    fn projector_signature_accepts_only_neutral_sessions_and_renderer_policy() {
        let _: fn(
            &[AgentSession],
            &crate::render_scene::SpeciesProfile,
        ) -> Result<AgentAnimalProjection, crate::render_scene::RenderSceneError> =
            project_agent_animals;
    }
}
