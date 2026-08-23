use crate::core::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridStyle {
    Dots,
    Lines,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridConfig {
    pub visible: bool,
    pub style: GridStyle,
    pub cell_size: f32,
    pub subdivisions: u32,
    pub snap_to_grid: bool,
    pub opacity: f32,
    pub color: Option<Color>,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            visible: false,
            style: GridStyle::Lines,
            cell_size: 32.0,
            subdivisions: 4,
            snap_to_grid: false,
            opacity: 0.25,
            color: None,
        }
    }
}

impl GridConfig {
    pub fn snap_point_with_origin(
        &self,
        p: crate::core::Point,
        origin: crate::core::Point,
    ) -> crate::core::Point {
        if !self.visible || self.cell_size <= 0.0 {
            return p;
        }
        let step = self.cell_size;
        let rel_x = p.x - origin.x;
        let rel_y = p.y - origin.y;
        crate::core::Point::new(
            origin.x + (rel_x / step).round() * step,
            origin.y + (rel_y / step).round() * step,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Point;

    #[test]
    fn test_grid_snap_point() {
        let grid = GridConfig {
            visible: true,
            style: GridStyle::Dots,
            cell_size: 20.0,
            subdivisions: 4,
            snap_to_grid: true,
            opacity: 0.25,
            color: None,
        };

        let p = Point::new(18.9, 41.2);
        let snapped = grid.snap_point_with_origin(p, Point::ZERO);
        assert_eq!(snapped, Point::new(20.0, 40.0));
    }
}
