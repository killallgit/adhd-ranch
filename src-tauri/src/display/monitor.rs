#[derive(Clone, Debug)]
pub struct LogicalMonitor {
    pub index: usize,
    pub label: String,
    pub scale_factor: f64,
    pub position: (f64, f64),
    pub size: (f64, f64),
}

impl LogicalMonitor {
    pub fn from_tauri(index: usize, m: &tauri::Monitor) -> Self {
        let sf = m.scale_factor();
        let label = m.name().map(|s| s.as_str()).unwrap_or("").to_string();
        let raw_pos = m.position();
        let raw_size = m.size();
        let logical_x = raw_pos.x as f64 / sf;
        let logical_y = raw_pos.y as f64 / sf;
        let logical_w = raw_size.width as f64 / sf;
        let logical_h = raw_size.height as f64 / sf;
        log::info!(
            "monitor[{index}] \"{label}\" sf={sf} raw_pos=({},{}) raw_size={}×{} logical=({},{} {}×{})",
            raw_pos.x, raw_pos.y, raw_size.width, raw_size.height,
            logical_x, logical_y, logical_w, logical_h
        );
        Self {
            index,
            label,
            scale_factor: sf,
            position: (logical_x, logical_y),
            size: (logical_w, logical_h),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SpanBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub fn compute_span(monitors: &[LogicalMonitor]) -> SpanBounds {
    let min_x = monitors
        .iter()
        .map(|m| m.position.0)
        .fold(f64::INFINITY, f64::min);
    let min_y = monitors
        .iter()
        .map(|m| m.position.1)
        .fold(f64::INFINITY, f64::min);
    let max_x = monitors
        .iter()
        .map(|m| m.position.0 + m.size.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let max_y = monitors
        .iter()
        .map(|m| m.position.1 + m.size.1)
        .fold(f64::NEG_INFINITY, f64::max);

    SpanBounds {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(1.0),
        height: (max_y - min_y).max(1.0),
    }
}

pub fn disambiguate_names(monitors: &mut [LogicalMonitor]) {
    let labels: Vec<String> = monitors.iter().map(|m| m.label.clone()).collect();
    for i in 0..monitors.len() {
        let has_dup = labels
            .iter()
            .enumerate()
            .any(|(j, l)| j != i && *l == labels[i]);
        if has_dup {
            let (x, y) = monitors[i].position;
            monitors[i].label = format!("{} ({}, {})", labels[i], x as i64, y as i64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon(index: usize, pos: (f64, f64), size: (f64, f64)) -> LogicalMonitor {
        LogicalMonitor {
            index,
            label: "Display".to_string(),
            scale_factor: 2.0,
            position: pos,
            size,
        }
    }

    fn named(index: usize, label: &str, pos: (f64, f64), size: (f64, f64)) -> LogicalMonitor {
        LogicalMonitor {
            index,
            label: label.to_string(),
            scale_factor: 2.0,
            position: pos,
            size,
        }
    }

    // Test 1: single landscape monitor at origin
    #[test]
    fn compute_span_single_at_origin() {
        let monitors = vec![mon(0, (0.0, 0.0), (1920.0, 1080.0))];
        assert_eq!(
            compute_span(&monitors),
            SpanBounds {
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0
            }
        );
    }

    // Test 2: two landscape monitors side-by-side
    #[test]
    fn compute_span_two_side_by_side() {
        let monitors = vec![
            mon(0, (0.0, 0.0), (1920.0, 1080.0)),
            mon(1, (1920.0, 0.0), (2560.0, 1440.0)),
        ];
        let span = compute_span(&monitors);
        assert_eq!(span.x, 0.0);
        assert_eq!(span.y, 0.0);
        assert_eq!(span.width, 4480.0); // 1920 + 2560
        assert_eq!(span.height, 1440.0);
    }

    // Test 3: secondary monitor left of primary (negative x)
    #[test]
    fn compute_span_negative_x_secondary() {
        let monitors = vec![
            mon(0, (0.0, 0.0), (1920.0, 1080.0)),
            mon(1, (-1280.0, 0.0), (1280.0, 800.0)),
        ];
        let span = compute_span(&monitors);
        assert_eq!(span.x, -1280.0);
        assert_eq!(span.y, 0.0);
        assert_eq!(span.width, 3200.0); // 1920 + 1280
        assert_eq!(span.height, 1080.0);
    }

    // Test 4: portrait monitor (270° rotated, height > width) left of landscape
    #[test]
    fn compute_span_portrait_left_of_landscape() {
        // Portrait: 1080 wide × 1920 tall (270° rotation swaps dimensions)
        let monitors = vec![
            mon(0, (0.0, 0.0), (2560.0, 1440.0)),
            mon(1, (-1080.0, 0.0), (1080.0, 1920.0)),
        ];
        let span = compute_span(&monitors);
        assert_eq!(span.x, -1080.0);
        assert_eq!(span.y, 0.0);
        assert_eq!(span.width, 3640.0); // 1080 + 2560
        assert_eq!(span.height, 1920.0);
    }

    // Test 5: two monitors with identical names → each gets " (x, y)" suffix
    #[test]
    fn disambiguate_names_identical() {
        let mut monitors = vec![
            named(0, "DELL U2720Q", (0.0, 0.0), (2560.0, 1440.0)),
            named(1, "DELL U2720Q", (2560.0, 0.0), (2560.0, 1440.0)),
        ];
        disambiguate_names(&mut monitors);
        assert_eq!(monitors[0].label, "DELL U2720Q (0, 0)");
        assert_eq!(monitors[1].label, "DELL U2720Q (2560, 0)");
    }

    // Test 6: monitors with unique names → labels unchanged
    #[test]
    fn disambiguate_names_unique() {
        let mut monitors = vec![
            named(0, "LG UltraFine", (0.0, 0.0), (2560.0, 1440.0)),
            named(1, "DELL U2720Q", (2560.0, 0.0), (2560.0, 1440.0)),
        ];
        disambiguate_names(&mut monitors);
        assert_eq!(monitors[0].label, "LG UltraFine");
        assert_eq!(monitors[1].label, "DELL U2720Q");
    }
}
