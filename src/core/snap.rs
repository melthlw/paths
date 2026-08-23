use std::collections::HashSet;

use super::document::Document;
use super::element::ElementId;
use super::geometry::{Point, Rect, Viewport};
use super::grid::GridConfig;
use super::ruler::RulerConfig;

#[derive(Debug, Clone, Copy, PartialEq)]

pub struct SnapConfig {
    pub enabled: bool,
    pub snap_to_grid: bool,
    pub snap_to_objects: bool,
    pub snap_to_artboard: bool,
    pub snap_to_guides: bool,
    pub tolerance_screen_px: f32,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_to_grid: false,
            snap_to_objects: true,
            snap_to_artboard: true,
            snap_to_guides: true,
            tolerance_screen_px: 8.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]

pub enum SnapGuide {
    Vertical { x: f32, y_min: f32, y_max: f32 },
    Horizontal { y: f32, x_min: f32, x_max: f32 },
    Point { point: Point },
}

#[derive(Debug, Clone, Default)]
pub struct SnapResult {
    pub point: Point,
    pub delta: Point,
    pub guides: Vec<SnapGuide>,
}

pub struct SnapEngine;

impl SnapEngine {
    pub fn snap_point(
        point: Point,
        document: &Document,
        viewport: &Viewport,
        snap_cfg: &SnapConfig,
        grid_cfg: &GridConfig,
        ruler_cfg: &RulerConfig,
        exclude_ids: &HashSet<ElementId>,
    ) -> SnapResult {
        if !snap_cfg.enabled {
            return SnapResult {
                point,
                delta: Point::ZERO,
                guides: Vec::new(),
            };
        }

        let tol = snap_cfg.tolerance_screen_px / viewport.zoom.max(0.01);
        let mut best_dx: Option<(f32, f32, f32, f32)> = None; // (dx, target_x, y_min, y_max)
        let mut best_dy: Option<(f32, f32, f32, f32)> = None; // (dy, target_y, x_min, x_max)

        // 1. Grid Snapping (Relative to Ruler Origin)
        if snap_cfg.snap_to_grid && grid_cfg.visible && grid_cfg.cell_size > 0.0 {
            let snapped_grid = grid_cfg.snap_point_with_origin(point, ruler_cfg.effective_origin());
            let gdx = snapped_grid.x - point.x;
            let gdy = snapped_grid.y - point.y;
            if gdx.abs() <= tol {
                best_dx = Some((gdx, snapped_grid.x, point.y - 20.0, point.y + 20.0));
            }
            if gdy.abs() <= tol {
                best_dy = Some((gdy, snapped_grid.y, point.x - 20.0, point.x + 20.0));
            }
        }

        // 2. Artboard / Page Snapping
        if snap_cfg.snap_to_artboard {
            for page in &document.pages {
                let pr = page.rect.normalize();
                let artboard_x = [pr.x, pr.x + pr.width / 2.0, pr.x + pr.width];
                let artboard_y = [pr.y, pr.y + pr.height / 2.0, pr.y + pr.height];

                for &ax in &artboard_x {
                    let dx = ax - point.x;
                    if dx.abs() <= tol && (best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs()) {
                        best_dx = Some((dx, ax, pr.y, pr.y + pr.height));
                    }
                }

                for &ay in &artboard_y {
                    let dy = ay - point.y;
                    if dy.abs() <= tol && (best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs()) {
                        best_dy = Some((dy, ay, pr.x, pr.x + pr.width));
                    }
                }
            }
        }

        // 3. Object / Element Snapping
        if snap_cfg.snap_to_objects {
            for el in &document.elements {
                if exclude_ids.contains(&el.id()) {
                    continue;
                }
                let b = el.bounds();
                let targets_x = [b.x, b.x + b.width / 2.0, b.x + b.width];
                let targets_y = [b.y, b.y + b.height / 2.0, b.y + b.height];

                for &tx in &targets_x {
                    let dx = tx - point.x;
                    if dx.abs() <= tol && (best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs())
                    {
                        let y_min = point.y.min(b.y);
                        let y_max = point.y.max(b.y + b.height);
                        best_dx = Some((dx, tx, y_min, y_max));
                    }
                }

                for &ty in &targets_y {
                    let dy = ty - point.y;
                    if dy.abs() <= tol && (best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs())
                    {
                        let x_min = point.x.min(b.x);
                        let x_max = point.x.max(b.x + b.width);
                        best_dy = Some((dy, ty, x_min, x_max));
                    }
                }
            }
        }

        // 4. User Guide Lines Snapping
        if snap_cfg.snap_to_guides {
            for guide in &document.guides {
                match guide.orientation {
                    crate::core::ruler::GuideOrientation::Vertical => {
                        let dx = guide.position - point.x;
                        if dx.abs() <= tol
                            && (best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs())
                        {
                            best_dx = Some((dx, guide.position, point.y - 40.0, point.y + 40.0));
                        }
                    }
                    crate::core::ruler::GuideOrientation::Horizontal => {
                        let dy = guide.position - point.y;
                        if dy.abs() <= tol
                            && (best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs())
                        {
                            best_dy = Some((dy, guide.position, point.x - 40.0, point.x + 40.0));
                        }
                    }
                }
            }
        }

        let mut guides = Vec::new();
        let dx = if let Some((d, target_x, y_min, y_max)) = best_dx {
            guides.push(SnapGuide::Vertical {
                x: target_x,
                y_min: y_min - 20.0 / viewport.zoom,
                y_max: y_max + 20.0 / viewport.zoom,
            });
            d
        } else {
            0.0
        };

        let dy = if let Some((d, target_y, x_min, x_max)) = best_dy {
            guides.push(SnapGuide::Horizontal {
                y: target_y,
                x_min: x_min - 20.0 / viewport.zoom,
                x_max: x_max + 20.0 / viewport.zoom,
            });
            d
        } else {
            0.0
        };

        if let (Some((_, tx, _, _)), Some((_, ty, _, _))) = (best_dx, best_dy) {
            guides.push(SnapGuide::Point {
                point: Point::new(tx, ty),
            });
        }

        SnapResult {
            point: Point::new(point.x + dx, point.y + dy),
            delta: Point::new(dx, dy),
            guides,
        }
    }

    pub fn snap_rect(
        rect: Rect,
        document: &Document,
        viewport: &Viewport,
        snap_cfg: &SnapConfig,
        grid_cfg: &GridConfig,
        ruler_cfg: &RulerConfig,
        exclude_ids: &HashSet<ElementId>,
    ) -> SnapResult {
        if !snap_cfg.enabled {
            return SnapResult {
                point: Point::new(rect.x, rect.y),
                delta: Point::ZERO,
                guides: Vec::new(),
            };
        }

        let tol = snap_cfg.tolerance_screen_px / viewport.zoom.max(0.01);
        let ref_points_x = [rect.x, rect.x + rect.width / 2.0, rect.x + rect.width];
        let ref_points_y = [rect.y, rect.y + rect.height / 2.0, rect.y + rect.height];

        let mut best_dx: Option<(f32, f32, f32, f32)> = None; // (dx, target_x, y_min, y_max)
        let mut best_dy: Option<(f32, f32, f32, f32)> = None; // (dy, target_y, x_min, x_max)

        // 1. Grid Snapping (Relative to Ruler Origin)
        if snap_cfg.snap_to_grid && grid_cfg.visible && grid_cfg.cell_size > 0.0 {
            let origin = ruler_cfg.effective_origin();
            for &rx in &ref_points_x {
                let rel_x = rx - origin.x;
                let snapped = origin.x + (rel_x / grid_cfg.cell_size).round() * grid_cfg.cell_size;
                let dx = snapped - rx;
                if dx.abs() <= tol && (best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs()) {
                    best_dx = Some((dx, snapped, rect.y - 30.0, rect.y + rect.height + 30.0));
                }
            }

            for &ry in &ref_points_y {
                let rel_y = ry - origin.y;
                let snapped = origin.y + (rel_y / grid_cfg.cell_size).round() * grid_cfg.cell_size;
                let dy = snapped - ry;
                if dy.abs() <= tol && (best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs()) {
                    best_dy = Some((dy, snapped, rect.x - 30.0, rect.x + rect.width + 30.0));
                }
            }
        }

        // 2. Artboard / Page Snapping
        if snap_cfg.snap_to_artboard {
            for page in &document.pages {
                let pr = page.rect.normalize();
                let artboard_x = [pr.x, pr.x + pr.width / 2.0, pr.x + pr.width];
                let artboard_y = [pr.y, pr.y + pr.height / 2.0, pr.y + pr.height];

                for &rx in &ref_points_x {
                    for &ax in &artboard_x {
                        let dx = ax - rx;
                        if dx.abs() <= tol && (best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs())
                        {
                            let y_min = rect.y.min(pr.y);
                            let y_max = (rect.y + rect.height).max(pr.y + pr.height);
                            best_dx = Some((dx, ax, y_min, y_max));
                        }
                    }
                }

                for &ry in &ref_points_y {
                    for &ay in &artboard_y {
                        let dy = ay - ry;
                        if dy.abs() <= tol && (best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs())
                        {
                            let x_min = rect.x.min(pr.x);
                            let x_max = (rect.x + rect.width).max(pr.x + pr.width);
                            best_dy = Some((dy, ay, x_min, x_max));
                        }
                    }
                }
            }
        }

        // 3. User Guides Snapping
        if snap_cfg.snap_to_guides {
            for guide in &document.guides {
                match guide.orientation {
                    crate::core::ruler::GuideOrientation::Vertical => {
                        for &rx in &ref_points_x {
                            let dx = guide.position - rx;
                            if dx.abs() <= tol
                                && (best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs())
                            {
                                best_dx = Some((
                                    dx,
                                    guide.position,
                                    rect.y - 50.0,
                                    rect.y + rect.height + 50.0,
                                ));
                            }
                        }
                    }
                    crate::core::ruler::GuideOrientation::Horizontal => {
                        for &ry in &ref_points_y {
                            let dy = guide.position - ry;
                            if dy.abs() <= tol
                                && (best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs())
                            {
                                best_dy = Some((
                                    dy,
                                    guide.position,
                                    rect.x - 50.0,
                                    rect.x + rect.width + 50.0,
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 4. Object / Element Snapping
        if snap_cfg.snap_to_objects {
            for el in &document.elements {
                if exclude_ids.contains(&el.id()) {
                    continue;
                }
                let bounds = el.bounds();
                let other_x = [
                    bounds.x,
                    bounds.x + bounds.width / 2.0,
                    bounds.x + bounds.width,
                ];
                let other_y = [
                    bounds.y,
                    bounds.y + bounds.height / 2.0,
                    bounds.y + bounds.height,
                ];

                for &rx in &ref_points_x {
                    for &ox in &other_x {
                        let dx = ox - rx;
                        if dx.abs() <= tol
                            && (best_dx.is_none() || dx.abs() < best_dx.unwrap().0.abs())
                        {
                            let y_min = rect.y.min(bounds.y);
                            let y_max = (rect.y + rect.height).max(bounds.y + bounds.height);
                            best_dx = Some((dx, ox, y_min, y_max));
                        }
                    }
                }

                for &ry in &ref_points_y {
                    for &oy in &other_y {
                        let dy = oy - ry;
                        if dy.abs() <= tol
                            && (best_dy.is_none() || dy.abs() < best_dy.unwrap().0.abs())
                        {
                            let x_min = rect.x.min(bounds.x);
                            let x_max = (rect.x + rect.width).max(bounds.x + bounds.width);
                            best_dy = Some((dy, oy, x_min, x_max));
                        }
                    }
                }
            }
        }

        let mut guides = Vec::new();
        let mut final_point = Point::new(rect.x, rect.y);
        let mut delta = Point::ZERO;

        if let Some((dx, target_x, y_min, y_max)) = best_dx {
            final_point.x += dx;
            delta.x = dx;
            guides.push(SnapGuide::Vertical {
                x: target_x,
                y_min,
                y_max,
            });
        }

        if let Some((dy, target_y, x_min, x_max)) = best_dy {
            final_point.y += dy;
            delta.y = dy;
            guides.push(SnapGuide::Horizontal {
                y: target_y,
                x_min,
                x_max,
            });
        }

        SnapResult {
            point: final_point,
            delta,
            guides,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Element, RectElement};

    #[test]
    fn test_snap_point_to_artboard_center() {
        let doc = Document::new();
        let vp = Viewport::default();
        let snap_cfg = SnapConfig::default();
        let grid_cfg = GridConfig::default();
        let ruler_cfg = RulerConfig::default();
        let exclude = HashSet::new();

        let p = Point::new(2.5, -1.8);
        let res = SnapEngine::snap_point(p, &doc, &vp, &snap_cfg, &grid_cfg, &ruler_cfg, &exclude);

        assert_eq!(res.point, Point::new(0.0, 0.0));
        assert_eq!(res.guides.len(), 3);
    }

    #[test]
    fn test_snap_rect_to_object() {
        let mut doc = Document::new();
        let rect1 = RectElement::new(Rect::new(100.0, 100.0, 50.0, 50.0), None, None);
        let id1 = rect1.id;
        doc.elements.push(Element::Rect(rect1));

        let vp = Viewport::default();
        let snap_cfg = SnapConfig {
            enabled: true,
            snap_to_grid: false,
            snap_to_objects: true,
            snap_to_artboard: false,
            snap_to_guides: true,
            tolerance_screen_px: 10.0,
        };
        let grid_cfg = GridConfig::default();
        let ruler_cfg = RulerConfig::default();
        let mut exclude = HashSet::new();
        exclude.insert(id1);

        // A moving rect close to rect1's right edge (150.0)
        let moving = Rect::new(148.0, 200.0, 40.0, 40.0);
        let moving_exclude = HashSet::new();
        let res = SnapEngine::snap_rect(
            moving,
            &doc,
            &vp,
            &snap_cfg,
            &grid_cfg,
            &ruler_cfg,
            &moving_exclude,
        );

        assert_eq!(res.point.x, 150.0);
    }

    #[test]
    fn test_snap_to_user_guides() {
        let mut doc = Document::new();
        doc.add_guide(crate::core::ruler::Guide::new(
            crate::core::ruler::GuideOrientation::Vertical,
            250.0,
        ));
        doc.add_guide(crate::core::ruler::Guide::new(
            crate::core::ruler::GuideOrientation::Horizontal,
            400.0,
        ));

        let vp = Viewport::default();
        let snap_cfg = SnapConfig::default();
        let grid_cfg = GridConfig::default();
        let ruler_cfg = RulerConfig::default();
        let exclude = HashSet::new();

        let pt = Point::new(248.0, 403.0);
        let res = SnapEngine::snap_point(pt, &doc, &vp, &snap_cfg, &grid_cfg, &ruler_cfg, &exclude);

        assert_eq!(res.point.x, 250.0);
        assert_eq!(res.point.y, 400.0);
    }
}
