use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct PigRect {
    pub x: f64,
    pub y: f64,
    pub size: f64,
}

#[derive(Clone)]
pub struct PigHitTester {
    rects: Arc<Mutex<Vec<PigRect>>>,
}

impl Default for PigHitTester {
    fn default() -> Self {
        Self::new()
    }
}

impl PigHitTester {
    pub fn new() -> Self {
        Self {
            rects: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn update(&self, rects: Vec<PigRect>) {
        if let Ok(mut guard) = self.rects.lock() {
            *guard = rects;
        }
    }

    pub fn is_hit(&self, x: f64, y: f64) -> bool {
        let Ok(guard) = self.rects.lock() else {
            return false;
        };
        guard
            .iter()
            .any(|r| x >= r.x && x <= r.x + r.size && y >= r.y && y <= r.y + r.size)
    }
}
