use crate::render_scene::{Species, SpeciesProfile};

pub fn species_profile(species: Species) -> SpeciesProfile {
    match species {
        Species::Pig => SpeciesProfile {
            species,
            base_size_px: 48.0,
            movement_footprint_px: 48.0,
            walk_speed_px_per_second: 60.0,
            friction: 0.97,
            minimum_speed_fraction: 0.35,
            maximum_toss_speed_multiplier: 6.0,
            turn_min_ms: 3_000,
            turn_max_ms: 8_000,
            frame_interval_ms: 150,
        },
    }
}

pub fn all_species_profiles() -> Vec<SpeciesProfile> {
    vec![species_profile(Species::Pig)]
}

#[cfg(test)]
mod tests {
    use crate::render_scene::Species;

    use super::species_profile;

    #[test]
    fn pig_profile_preserves_existing_physical_policy() {
        let profile = species_profile(Species::Pig);

        assert_eq!(profile.base_size_px, 48.0);
        assert_eq!(profile.movement_footprint_px, 48.0);
        assert_eq!(profile.walk_speed_px_per_second, 60.0);
        assert_eq!(profile.friction, 0.97);
        assert_eq!(profile.minimum_speed_fraction, 0.35);
        assert_eq!(profile.maximum_toss_speed_multiplier, 6.0);
        assert_eq!(profile.turn_min_ms, 3_000);
        assert_eq!(profile.turn_max_ms, 8_000);
        assert_eq!(profile.frame_interval_ms, 150);
    }
}
