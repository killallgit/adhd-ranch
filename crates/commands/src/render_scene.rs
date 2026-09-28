use std::{collections::HashSet, error::Error, fmt, sync::Arc};

use adhd_ranch_storage::{AgentSessionStore, FocusStore};
use serde::{Deserialize, Serialize};

use crate::{
    agent_animals::{project_agent_animals, AgentAnimalProjection},
    error::CommandError,
    focus_animals::project_focus_animals,
    region_layout::layout_regions,
    species::{all_species_profiles, species_profile},
    SettingsProvider,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum Species {
    Pig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum Motion {
    Walking,
    Resting,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct RenderRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl RenderRect {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Result<Self, RenderSceneError> {
        if ![x, y, w, h].into_iter().all(f64::is_finite) || w < 0.0 || h < 0.0 {
            return Err(RenderSceneError::InvalidRect);
        }
        Ok(Self { x, y, w, h })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub enum AnimalSize {
    Fixed {
        px: f64,
    },
    Linear {
        from_px: f64,
        to_px: f64,
        started_at_ms: i64,
        duration_ms: u64,
    },
}

impl AnimalSize {
    pub fn fixed(px: f64) -> Result<Self, RenderSceneError> {
        validate_positive_finite(px)?;
        Ok(Self::Fixed { px })
    }

    pub fn linear(
        from_px: f64,
        to_px: f64,
        started_at_ms: i64,
        duration_ms: u64,
    ) -> Result<Self, RenderSceneError> {
        validate_positive_finite(from_px)?;
        validate_positive_finite(to_px)?;
        if duration_ms == 0 {
            return Err(RenderSceneError::InvalidSize);
        }
        Ok(Self::Linear {
            from_px,
            to_px,
            started_at_ms,
            duration_ms,
        })
    }

    fn validate(&self) -> Result<(), RenderSceneError> {
        match self {
            Self::Fixed { px } => validate_positive_finite(*px),
            Self::Linear {
                from_px,
                to_px,
                duration_ms,
                ..
            } => {
                validate_positive_finite(*from_px)?;
                validate_positive_finite(*to_px)?;
                if *duration_ms == 0 {
                    return Err(RenderSceneError::InvalidSize);
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct Animal {
    pub id: String,
    pub label: String,
    pub species: Species,
    pub size: AnimalSize,
    pub region_id: Option<String>,
    pub motion: Motion,
}

impl Animal {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        species: Species,
        size: AnimalSize,
        region_id: Option<String>,
        motion: Motion,
    ) -> Result<Self, RenderSceneError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(RenderSceneError::EmptyAnimalId);
        }
        size.validate()?;
        if region_id.as_ref().is_some_and(|id| id.trim().is_empty()) {
            return Err(RenderSceneError::EmptyRegionId);
        }
        Ok(Self {
            id,
            label: label.into(),
            species,
            size,
            region_id,
            motion,
        })
    }

    fn validate(&self) -> Result<(), RenderSceneError> {
        if self.id.trim().is_empty() {
            return Err(RenderSceneError::EmptyAnimalId);
        }
        self.size.validate()?;
        if self
            .region_id
            .as_ref()
            .is_some_and(|id| id.trim().is_empty())
        {
            return Err(RenderSceneError::EmptyRegionId);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnimalRegion {
    pub(crate) id: String,
    pub(crate) label: String,
}

impl AnimalRegion {
    pub(crate) fn new(
        id: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<Self, RenderSceneError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(RenderSceneError::EmptyRegionId);
        }
        Ok(Self {
            id,
            label: label.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct AnimalRegionLayout {
    pub id: String,
    pub label: String,
    pub rect: RenderRect,
    pub hue: u16,
}

impl AnimalRegionLayout {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        rect: RenderRect,
        hue: u16,
    ) -> Result<Self, RenderSceneError> {
        let region = AnimalRegion::new(id, label)?;
        Ok(Self::from_region(region, rect, hue))
    }

    pub(crate) fn from_region(region: AnimalRegion, rect: RenderRect, hue: u16) -> Self {
        Self {
            id: region.id,
            label: region.label,
            rect,
            hue,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct SpeciesProfile {
    pub species: Species,
    pub base_size_px: f64,
    pub movement_footprint_px: f64,
    pub walk_speed_px_per_second: f64,
    pub friction: f64,
    pub minimum_speed_fraction: f64,
    pub maximum_toss_speed_multiplier: f64,
    pub turn_min_ms: u64,
    pub turn_max_ms: u64,
    pub frame_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct RenderSceneRequest {
    pub area: RenderRect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "export-ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "export-ts", ts(export))]
pub struct RenderScene {
    pub animals: Vec<Animal>,
    pub regions: Vec<AnimalRegionLayout>,
    pub species_profiles: Vec<SpeciesProfile>,
}

impl RenderScene {
    pub fn new(
        animals: Vec<Animal>,
        regions: Vec<AnimalRegionLayout>,
        species_profiles: Vec<SpeciesProfile>,
    ) -> Result<Self, RenderSceneError> {
        let mut animal_ids = HashSet::new();
        for animal in &animals {
            animal.validate()?;
            if !animal_ids.insert(animal.id.clone()) {
                return Err(RenderSceneError::DuplicateAnimalId(animal.id.clone()));
            }
        }

        let mut region_ids = HashSet::new();
        for region in &regions {
            if region.id.trim().is_empty() {
                return Err(RenderSceneError::EmptyRegionId);
            }
            if !region_ids.insert(region.id.clone()) {
                return Err(RenderSceneError::DuplicateRegionId(region.id.clone()));
            }
        }

        let mut profile_species = HashSet::new();
        for profile in &species_profiles {
            if !profile_species.insert(profile.species) {
                return Err(RenderSceneError::DuplicateSpeciesProfile(profile.species));
            }
        }
        for animal in &animals {
            if !profile_species.contains(&animal.species) {
                return Err(RenderSceneError::MissingSpeciesProfile(animal.species));
            }
        }

        Ok(Self {
            animals,
            regions,
            species_profiles,
        })
    }
}

pub struct RenderSceneService {
    focus_store: Arc<dyn FocusStore>,
    agent_session_store: Arc<dyn AgentSessionStore>,
    settings: SettingsProvider,
}

impl RenderSceneService {
    pub fn new(
        focus_store: Arc<dyn FocusStore>,
        agent_session_store: Arc<dyn AgentSessionStore>,
        settings: SettingsProvider,
    ) -> Self {
        Self {
            focus_store,
            agent_session_store,
            settings,
        }
    }

    pub fn get(&self, request: RenderSceneRequest) -> Result<RenderScene, CommandError> {
        let profiles = all_species_profiles();
        let pig_profile = species_profile(Species::Pig);
        let mut animals = project_focus_animals(&self.focus_store.list()?, &pig_profile)?;
        let settings = self.settings.get();
        let mut logical_regions = Vec::new();
        if settings.agents.enabled {
            let AgentAnimalProjection {
                animals: agent_animals,
                regions: agent_regions,
            } = project_agent_animals(&self.agent_session_store.list(), &pig_profile)?;
            animals.extend(agent_animals);
            logical_regions = agent_regions;
        }
        let regions = layout_regions(
            &logical_regions,
            &request.area,
            f64::from(settings.pens.max_size),
        )?;
        RenderScene::new(animals, regions, profiles).map_err(Into::into)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderSceneError {
    InvalidRect,
    InvalidSize,
    InvalidRegionSize,
    EmptyAnimalId,
    EmptyRegionId,
    DuplicateAnimalId(String),
    DuplicateRegionId(String),
    DuplicateSpeciesProfile(Species),
    MissingSpeciesProfile(Species),
}

impl fmt::Display for RenderSceneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRect => write!(f, "render rectangle must be finite and non-negative"),
            Self::InvalidSize => write!(
                f,
                "animal size must be finite, positive, and non-zero duration"
            ),
            Self::InvalidRegionSize => {
                write!(f, "maximum region size must be finite and non-negative")
            }
            Self::EmptyAnimalId => write!(f, "animal id must be non-empty"),
            Self::EmptyRegionId => write!(f, "region id must be non-empty"),
            Self::DuplicateAnimalId(id) => write!(f, "duplicate animal id: {id}"),
            Self::DuplicateRegionId(id) => write!(f, "duplicate region id: {id}"),
            Self::DuplicateSpeciesProfile(species) => {
                write!(f, "duplicate species profile: {species:?}")
            }
            Self::MissingSpeciesProfile(species) => {
                write!(f, "missing species profile: {species:?}")
            }
        }
    }
}

impl Error for RenderSceneError {}

fn validate_positive_finite(value: f64) -> Result<(), RenderSceneError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(RenderSceneError::InvalidSize)
    }
}

#[cfg(test)]
mod service_tests {
    use std::sync::{Arc, Mutex};

    use adhd_ranch_domain::{
        session::{AgentSession, Pen, SessionActivity},
        AgentsConfig, Focus, FocusId, FocusTimer, NewFocus, Settings,
    };
    use adhd_ranch_storage::{AgentSessionStore, FocusStore, FocusStoreError};

    use super::{RenderRect, RenderSceneRequest, RenderSceneService};
    use crate::SettingsProvider;

    struct Focuses(Arc<Mutex<Vec<Focus>>>);

    impl FocusStore for Focuses {
        fn list(&self) -> Result<Vec<Focus>, FocusStoreError> {
            Ok(self.0.lock().unwrap().clone())
        }

        fn create_focus(
            &self,
            _: &NewFocus,
            _: &str,
            _: &str,
            _: Option<FocusTimer>,
        ) -> Result<String, FocusStoreError> {
            unimplemented!()
        }
        fn delete_focus(&self, _: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
        fn rename_focus(&self, _: &str, _: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
        fn append_task(&self, _: &str, _: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
        fn delete_task(&self, _: &str, _: usize) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
        fn update_task(&self, _: &str, _: usize, _: &str) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
        fn toggle_task(&self, _: &str, _: usize, _: bool) -> Result<(), FocusStoreError> {
            unimplemented!()
        }
    }

    struct Sessions(Arc<Mutex<Vec<AgentSession>>>);

    impl AgentSessionStore for Sessions {
        fn list(&self) -> Vec<AgentSession> {
            self.0.lock().unwrap().clone()
        }
    }

    fn focus(id: &str, title: &str) -> Focus {
        Focus {
            id: FocusId(id.into()),
            title: title.into(),
            description: "private source field".into(),
            created_at: "2026-09-27T00:00:00Z".into(),
            tasks: vec![],
            timer: None,
        }
    }

    fn session(id: &str, name: &str) -> AgentSession {
        AgentSession {
            id: id.into(),
            name: name.into(),
            pen: Pen {
                id: "repo".into(),
                name: "Repository".into(),
            },
            activity: SessionActivity::Working,
        }
    }

    fn request() -> RenderSceneRequest {
        RenderSceneRequest {
            area: RenderRect::new(0.0, 0.0, 1_000.0, 800.0).unwrap(),
        }
    }

    fn service(
        focuses: Arc<Mutex<Vec<Focus>>>,
        sessions: Arc<Mutex<Vec<AgentSession>>>,
        agents_enabled: bool,
    ) -> RenderSceneService {
        let settings: SettingsProvider = Arc::new(move || Settings {
            agents: AgentsConfig {
                enabled: agents_enabled,
            },
            ..Settings::default()
        });
        RenderSceneService::new(
            Arc::new(Focuses(focuses)),
            Arc::new(Sessions(sessions)),
            settings,
        )
    }

    #[test]
    fn composes_mixed_and_single_domain_scenes_with_collision_safe_ids() {
        let focuses = Arc::new(Mutex::new(vec![focus("same", "Focus")]));
        let sessions = Arc::new(Mutex::new(vec![session("same", "Agent")]));
        let scene = service(focuses.clone(), sessions.clone(), true)
            .get(request())
            .unwrap();

        assert_eq!(scene.animals.len(), 2);
        assert_eq!(scene.animals[0].id, "same");
        assert_eq!(scene.animals[1].id, "agent:same");
        assert_eq!(scene.species_profiles.len(), 1);
        assert_eq!(scene.regions.len(), 1);
        assert_eq!(scene.regions[0].id, "repo");
        assert_eq!(
            scene.regions[0].rect,
            RenderRect::new(340.0, 240.0, 320.0, 320.0).unwrap()
        );

        sessions.lock().unwrap().clear();
        assert_eq!(
            service(focuses.clone(), sessions.clone(), true)
                .get(request())
                .unwrap()
                .animals
                .len(),
            1
        );
        focuses.lock().unwrap().clear();
        sessions.lock().unwrap().push(session("only", "Only agent"));
        assert_eq!(
            service(focuses, sessions, true)
                .get(request())
                .unwrap()
                .animals
                .len(),
            1
        );
    }

    #[test]
    fn filters_disabled_agents_and_changes_sources_independently() {
        let focuses = Arc::new(Mutex::new(vec![focus("focus", "Original focus")]));
        let sessions = Arc::new(Mutex::new(vec![session("agent", "Original agent")]));
        let disabled = service(focuses.clone(), sessions.clone(), false)
            .get(request())
            .unwrap();
        assert_eq!(disabled.animals.len(), 1);
        assert_eq!(disabled.animals[0].id, "focus");

        let enabled = service(focuses.clone(), sessions.clone(), true);
        let before = enabled.get(request()).unwrap();
        focuses.lock().unwrap()[0].title = "Changed focus".into();
        let after_focus = enabled.get(request()).unwrap();
        assert_eq!(after_focus.animals[0].label, "Changed focus");
        assert_eq!(after_focus.animals[1], before.animals[1]);

        sessions.lock().unwrap()[0].name = "Changed agent".into();
        let after_agent = enabled.get(request()).unwrap();
        assert_eq!(after_agent.animals[0], after_focus.animals[0]);
        assert_eq!(after_agent.animals[1].label, "Changed agent");
    }

    #[test]
    fn serialized_scene_contains_no_source_records() {
        let scene = service(
            Arc::new(Mutex::new(vec![focus("focus", "Focus")])),
            Arc::new(Mutex::new(vec![session("agent", "Agent")])),
            true,
        )
        .get(request())
        .unwrap();
        let json = serde_json::to_value(scene).unwrap();
        let animals = json["animals"].as_array().unwrap();
        for animal in animals {
            let object = animal.as_object().unwrap();
            assert_eq!(object.len(), 6);
            for forbidden in ["focus", "tasks", "timer", "session", "activity", "pen"] {
                assert!(!object.contains_key(forbidden));
            }
        }
    }
}
