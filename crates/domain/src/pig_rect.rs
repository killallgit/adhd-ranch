use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PigRect {
    pub x: f64,
    pub y: f64,
    pub size: f64,
}

pub trait RectUpdater: Send + Sync {
    fn update_rects(&self, label: &str, rects: Vec<PigRect>);
}
