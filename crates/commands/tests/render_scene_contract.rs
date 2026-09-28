use std::collections::BTreeSet;

use adhd_ranch_commands::render_scene::{
    Animal, AnimalRegionLayout, AnimalSize, Motion, RenderRect, RenderScene, RenderSceneRequest,
    Species, SpeciesProfile,
};
use serde_json::json;

fn pig_profile() -> SpeciesProfile {
    SpeciesProfile {
        species: Species::Pig,
        base_size_px: 48.0,
        movement_footprint_px: 48.0,
        walk_speed_px_per_second: 60.0,
        friction: 0.97,
        minimum_speed_fraction: 0.35,
        maximum_toss_speed_multiplier: 6.0,
        turn_min_ms: 3_000,
        turn_max_ms: 8_000,
        frame_interval_ms: 150,
    }
}

#[test]
fn renderer_values_serialize_to_the_exact_camel_case_contract() {
    let area = RenderRect::new(1.0, 2.0, 640.0, 480.0).unwrap();
    let fixed = AnimalSize::fixed(48.0).unwrap();
    let linear = AnimalSize::linear(48.0, 96.0, 1_000, 60_000).unwrap();
    let animal = Animal::new(
        "focus-1",
        "Write tests",
        Species::Pig,
        linear.clone(),
        None,
        Motion::Walking,
    )
    .unwrap();
    let layout = AnimalRegionLayout::new("repo-1", "ADHD Ranch", area.clone(), 145).unwrap();
    let profile = pig_profile();
    let request = RenderSceneRequest { area: area.clone() };
    let scene = RenderScene::new(
        vec![animal.clone()],
        vec![layout.clone()],
        vec![profile.clone()],
    )
    .unwrap();

    assert_eq!(serde_json::to_value(Species::Pig).unwrap(), json!("pig"));
    assert_eq!(
        serde_json::to_value(Motion::Walking).unwrap(),
        json!("walking")
    );
    assert_eq!(
        serde_json::to_value(Motion::Resting).unwrap(),
        json!("resting")
    );
    assert_eq!(
        serde_json::to_value(area).unwrap(),
        json!({ "x": 1.0, "y": 2.0, "w": 640.0, "h": 480.0 })
    );
    assert_eq!(
        serde_json::to_value(fixed).unwrap(),
        json!({ "kind": "fixed", "px": 48.0 })
    );
    assert_eq!(
        serde_json::to_value(linear).unwrap(),
        json!({
            "kind": "linear",
            "fromPx": 48.0,
            "toPx": 96.0,
            "startedAtMs": 1_000,
            "durationMs": 60_000
        })
    );
    assert_eq!(
        serde_json::to_value(animal).unwrap(),
        json!({
            "id": "focus-1",
            "label": "Write tests",
            "species": "pig",
            "size": {
                "kind": "linear",
                "fromPx": 48.0,
                "toPx": 96.0,
                "startedAtMs": 1_000,
                "durationMs": 60_000
            },
            "regionId": null,
            "motion": "walking"
        })
    );
    assert_eq!(
        serde_json::to_value(layout).unwrap(),
        json!({
            "id": "repo-1",
            "label": "ADHD Ranch",
            "rect": { "x": 1.0, "y": 2.0, "w": 640.0, "h": 480.0 },
            "hue": 145
        })
    );
    assert_eq!(
        serde_json::to_value(profile).unwrap(),
        json!({
            "species": "pig",
            "baseSizePx": 48.0,
            "movementFootprintPx": 48.0,
            "walkSpeedPxPerSecond": 60.0,
            "friction": 0.97,
            "minimumSpeedFraction": 0.35,
            "maximumTossSpeedMultiplier": 6.0,
            "turnMinMs": 3_000,
            "turnMaxMs": 8_000,
            "frameIntervalMs": 150
        })
    );
    assert_eq!(
        serde_json::to_value(request).unwrap(),
        json!({ "area": { "x": 1.0, "y": 2.0, "w": 640.0, "h": 480.0 } })
    );
    assert_eq!(
        serde_json::to_value(scene).unwrap(),
        json!({
            "animals": [{
                "id": "focus-1",
                "label": "Write tests",
                "species": "pig",
                "size": {
                    "kind": "linear",
                    "fromPx": 48.0,
                    "toPx": 96.0,
                    "startedAtMs": 1_000,
                    "durationMs": 60_000
                },
                "regionId": null,
                "motion": "walking"
            }],
            "regions": [{
                "id": "repo-1",
                "label": "ADHD Ranch",
                "rect": { "x": 1.0, "y": 2.0, "w": 640.0, "h": 480.0 },
                "hue": 145
            }],
            "speciesProfiles": [{
                "species": "pig",
                "baseSizePx": 48.0,
                "movementFootprintPx": 48.0,
                "walkSpeedPxPerSecond": 60.0,
                "friction": 0.97,
                "minimumSpeedFraction": 0.35,
                "maximumTossSpeedMultiplier": 6.0,
                "turnMinMs": 3_000,
                "turnMaxMs": 8_000,
                "frameIntervalMs": 150
            }]
        })
    );
}

#[test]
fn animal_validation_proves_the_renderer_only_invariants() {
    // id: "Non-empty and unique across composed Focus and Agent Animals".
    assert!(Animal::new(
        "",
        "label",
        Species::Pig,
        AnimalSize::fixed(48.0).unwrap(),
        None,
        Motion::Walking,
    )
    .is_err());

    let duplicate = Animal::new(
        "same-id",
        "Second",
        Species::Pig,
        AnimalSize::fixed(48.0).unwrap(),
        None,
        Motion::Walking,
    )
    .unwrap();
    let first = Animal::new(
        "same-id",
        "First",
        Species::Pig,
        AnimalSize::fixed(48.0).unwrap(),
        None,
        Motion::Walking,
    )
    .unwrap();
    assert!(RenderScene::new(vec![first, duplicate], vec![], vec![pig_profile()]).is_err());

    // label: "Display-only; carries no source identity". Exact keys prove there is no source tag.
    let animal = Animal::new(
        "id",
        "plain label",
        Species::Pig,
        AnimalSize::fixed(48.0).unwrap(),
        None,
        Motion::Resting,
    )
    .unwrap();
    let animal_json = serde_json::to_value(animal).unwrap();
    let keys: BTreeSet<_> = animal_json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        BTreeSet::from(["id", "label", "motion", "regionId", "size", "species"])
    );

    // species: "Explicitly `pig` in this feature; never inferred from source identity".
    assert_eq!(serde_json::to_value(Species::Pig).unwrap(), json!("pig"));

    // size: "Resolves to a finite value greater than zero and contains no Timer vocabulary".
    for px in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(AnimalSize::fixed(px).is_err());
    }
    assert!(AnimalSize::linear(48.0, 96.0, 0, 0).is_err());
    let size_json = serde_json::to_value(AnimalSize::fixed(48.0).unwrap()).unwrap();
    let size_keys: BTreeSet<_> = size_json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(size_keys, BTreeSet::from(["kind", "px"]));

    // regionId: "`null` means full DisplaySpace; non-null is resolved against current regions".
    let whole_display = Animal::new(
        "display-animal",
        "Display",
        Species::Pig,
        AnimalSize::fixed(48.0).unwrap(),
        None,
        Motion::Walking,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(whole_display).unwrap()["regionId"],
        json!(null)
    );
    assert!(Animal::new(
        "bad-region",
        "Bad region",
        Species::Pig,
        AnimalSize::fixed(48.0).unwrap(),
        Some(String::new()),
        Motion::Walking,
    )
    .is_err());

    // motion: "Must be a supported Motion value". Deserialization rejects unknown values.
    assert!(serde_json::from_value::<Motion>(json!("flying")).is_err());
}
