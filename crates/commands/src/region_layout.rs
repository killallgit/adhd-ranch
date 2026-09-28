use std::collections::BTreeMap;

use crate::render_scene::{AnimalRegion, AnimalRegionLayout, RenderRect, RenderSceneError};

const REGION_GAP_PX: f64 = 16.0;
const REGION_HUES: [u16; 8] = [12, 45, 96, 145, 195, 250, 290, 330];

pub(crate) fn layout_regions(
    regions: &[AnimalRegion],
    area: &RenderRect,
    maximum_size: f64,
) -> Result<Vec<AnimalRegionLayout>, RenderSceneError> {
    if !maximum_size.is_finite() || maximum_size < 0.0 {
        return Err(RenderSceneError::InvalidRegionSize);
    }
    let mut regions_by_id = BTreeMap::new();
    for region in regions {
        if region.id.trim().is_empty() {
            return Err(RenderSceneError::EmptyRegionId);
        }
        regions_by_id.insert(region.id.clone(), region.label.clone());
    }
    if regions_by_id.is_empty() {
        return Ok(Vec::new());
    }

    let count = regions_by_id.len();
    let columns = (count as f64).sqrt().ceil() as usize;
    let rows = count.div_ceil(columns);
    let region_width = (area.w / columns as f64 - REGION_GAP_PX)
        .max(0.0)
        .min(maximum_size);
    let region_height = (area.h / rows as f64 - REGION_GAP_PX)
        .max(0.0)
        .min(maximum_size);
    let cell_width = region_width + REGION_GAP_PX;
    let cell_height = region_height + REGION_GAP_PX;
    let origin_x = area.x + (area.w - cell_width * columns as f64) / 2.0;
    let origin_y = area.y + (area.h - cell_height * rows as f64) / 2.0;

    regions_by_id
        .into_iter()
        .enumerate()
        .map(|(index, (id, label))| {
            let hue = region_hue(&id);
            let region = AnimalRegion::new(id, label)?;
            let rect = RenderRect::new(
                origin_x + (index % columns) as f64 * cell_width + REGION_GAP_PX / 2.0,
                origin_y + (index / columns) as f64 * cell_height + REGION_GAP_PX / 2.0,
                region_width,
                region_height,
            )?;
            Ok(AnimalRegionLayout::from_region(region, rect, hue))
        })
        .collect()
}

fn region_hue(region_id: &str) -> u16 {
    let hash = region_id.encode_utf16().fold(0_u64, |hash, unit| {
        (hash * 31 + u64::from(unit)) % 0xffff_ffff
    });
    REGION_HUES[hash as usize % REGION_HUES.len()]
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::render_scene::{AnimalRegion, RenderRect};

    use super::layout_regions;

    fn region(id: &str, label: &str) -> AnimalRegion {
        AnimalRegion::new(id, label).unwrap()
    }

    #[test]
    fn deduplicates_and_sorts_regions_with_stable_hues() {
        let area = RenderRect::new(0.0, 0.0, 1_000.0, 800.0).unwrap();
        let regions = vec![
            region("b", "Bee"),
            region("a", "First"),
            region("a", "Final"),
        ];
        let layouts = layout_regions(&regions, &area, 320.0).unwrap();
        let resized = layout_regions(
            &regions,
            &RenderRect::new(0.0, 0.0, 800.0, 600.0).unwrap(),
            320.0,
        )
        .unwrap();

        assert_eq!(
            layouts
                .iter()
                .map(|layout| layout.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(layouts[0].label, "Final");
        assert_eq!(layouts[0].hue, resized[0].hue);
        assert_eq!(layouts[1].hue, resized[1].hue);
        assert_ne!(layouts[0].rect, resized[0].rect);
    }

    #[test]
    fn centers_cells_after_maximum_size_capping() {
        let layouts = layout_regions(
            &[region("repo", "Repository")],
            &RenderRect::new(0.0, 0.0, 1_000.0, 800.0).unwrap(),
            320.0,
        )
        .unwrap();

        assert_eq!(
            layouts[0].rect,
            RenderRect::new(340.0, 240.0, 320.0, 320.0).unwrap()
        );
    }

    #[test]
    fn empty_input_produces_no_layouts() {
        assert!(layout_regions(
            &[],
            &RenderRect::new(0.0, 0.0, 1_000.0, 800.0).unwrap(),
            320.0,
        )
        .unwrap()
        .is_empty());
    }

    #[test]
    fn proves_region_identity_and_display_only_label_constraints() {
        assert!(AnimalRegion::new("", "Empty id").is_err());
        let layouts = layout_regions(
            &[
                region("repo", "Display label"),
                region("repo", "Latest label"),
            ],
            &RenderRect::new(0.0, 0.0, 1_000.0, 800.0).unwrap(),
            320.0,
        )
        .unwrap();
        let ids: HashSet<_> = layouts.iter().map(|layout| &layout.id).collect();
        assert_eq!(ids.len(), layouts.len());
        let object = serde_json::to_value(&layouts[0]).unwrap();
        assert_eq!(
            object
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<HashSet<_>>(),
            HashSet::from(["id", "label", "rect", "hue"]),
        );
    }
}
