use crate::core::{Color, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GuideOrientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Guide {
    pub id: u64,
    pub orientation: GuideOrientation,
    pub position: f32,
    pub color: Option<Color>,
    pub locked: bool,
}

static GUIDE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl Guide {
    pub fn new(orientation: GuideOrientation, position: f32) -> Self {
        Self {
            id: GUIDE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            orientation,
            position,
            color: None,
            locked: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RulerConfig {
    pub visible: bool,
    pub guides_visible: bool,
    pub thickness: f32,
    pub origin_at_artboard_top_left: bool,
    pub custom_origin: Option<Point>,
}

impl Default for RulerConfig {
    fn default() -> Self {
        Self {
            visible: true,
            guides_visible: true,
            thickness: 22.0,
            origin_at_artboard_top_left: true,
            custom_origin: None,
        }
    }
}

impl RulerConfig {
    pub fn effective_origin(&self) -> Point {
        if let Some(custom) = self.custom_origin {
            custom
        } else {
            Point::ZERO
        }
    }

    pub fn reset_origin(&mut self) {
        self.custom_origin = None;
    }
}

/// Calculate dynamic major tick step in world units based on viewport zoom
/// to maintain a clean screen spacing of roughly 60 to 120 pixels between major ticks.
pub fn calculate_tick_step(zoom: f32) -> f32 {
    let raw_step = 80.0 / zoom.max(0.001);
    let power = 10.0_f32.powf(raw_step.log10().floor());
    let normalized = raw_step / power;

    let multiplier = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };

    multiplier * power
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_tick_step() {
        let step_1x = calculate_tick_step(1.0);
        assert!(step_1x >= 50.0 && step_1x <= 100.0);

        let step_zoom_in = calculate_tick_step(10.0);
        assert!(step_zoom_in >= 5.0 && step_zoom_in <= 10.0);

        let step_zoom_out = calculate_tick_step(0.1);
        assert!(step_zoom_out >= 500.0 && step_zoom_out <= 1000.0);
    }

    #[test]
    fn test_guide_creation() {
        let g1 = Guide::new(GuideOrientation::Horizontal, 150.0);
        let g2 = Guide::new(GuideOrientation::Vertical, 300.0);
        assert_eq!(g1.orientation, GuideOrientation::Horizontal);
        assert_eq!(g1.position, 150.0);
        assert_eq!(g2.orientation, GuideOrientation::Vertical);
        assert_eq!(g2.position, 300.0);
    }

    #[test]
    fn test_ruler_custom_origin() {
        let mut cfg = RulerConfig::default();
        assert_eq!(cfg.effective_origin(), Point::ZERO);

        cfg.custom_origin = Some(Point::new(100.0, 200.0));
        assert_eq!(cfg.effective_origin(), Point::new(100.0, 200.0));

        cfg.reset_origin();
        assert_eq!(cfg.effective_origin(), Point::ZERO);
    }
}
