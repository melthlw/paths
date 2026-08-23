use skia_safe as skia;

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Point) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn to_skia(self) -> skia::Point {
        skia::Point::new(self.x, self.y)
    }

    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }
}

impl std::ops::Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f32> for Point {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_points(p1: Point, p2: Point) -> Self {
        let x = p1.x.min(p2.x);
        let y = p1.y.min(p2.y);
        let width = (p1.x - p2.x).abs();
        let height = (p1.y - p2.y).abs();
        Self::new(x, y, width, height)
    }

    pub fn round(self) -> Self {
        let norm = self.normalize();
        Self {
            x: norm.x.round(),
            y: norm.y.round(),
            width: norm.width.round().max(1.0),
            height: norm.height.round().max(1.0),
        }
    }

    pub fn normalize(self) -> Self {
        let mut x = self.x;
        let mut y = self.y;
        let mut width = self.width;
        let mut height = self.height;

        if width < 0.0 {
            x += width;
            width = -width;
        }
        if height < 0.0 {
            y += height;
            height = -height;
        }

        Self::new(x, y, width, height)
    }

    pub fn contains(self, p: Point) -> bool {
        let norm = self.normalize();
        p.x >= norm.x && p.x <= norm.x + norm.width && p.y >= norm.y && p.y <= norm.y + norm.height
    }

    pub fn intersects(self, other: Rect) -> bool {
        let a = self.normalize();
        let b = other.normalize();
        !(a.x + a.width < b.x
            || b.x + b.width < a.x
            || a.y + a.height < b.y
            || b.y + b.height < a.y)
    }

    pub fn expand(self, margin: f32) -> Self {
        Self::new(
            self.x - margin,
            self.y - margin,
            self.width + margin * 2.0,
            self.height + margin * 2.0,
        )
    }

    pub fn union(self, other: Rect) -> Self {
        let a = self.normalize();
        let b = other.normalize();
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = (a.x + a.width).max(b.x + b.width);
        let max_y = (a.y + a.height).max(b.y + b.height);
        Self::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    pub fn center(self) -> Point {
        let norm = self.normalize();
        Point::new(norm.x + norm.width / 2.0, norm.y + norm.height / 2.0)
    }

    pub fn to_skia(self) -> skia::Rect {
        let norm = self.normalize();
        skia::Rect::from_xywh(norm.x, norm.y, norm.width, norm.height)
    }
}

pub fn rotate_point(p: Point, center: Point, angle_rad: f32) -> Point {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();
    let dx = p.x - center.x;
    let dy = p.y - center.y;
    Point::new(
        center.x + dx * cos_a - dy * sin_a,
        center.y + dx * sin_a + dy * cos_a,
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub pan: Point,
    pub zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan: Point::ZERO,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    pub const MIN_ZOOM: f32 = 0.05;
    pub const MAX_ZOOM: f32 = 40.0;

    pub fn screen_to_world(&self, screen: Point, widget_size: (f32, f32)) -> Point {
        let cx = widget_size.0 / 2.0;
        let cy = widget_size.1 / 2.0;
        Point::new(
            (screen.x - cx - self.pan.x) / self.zoom,
            (screen.y - cy - self.pan.y) / self.zoom,
        )
    }

    pub fn zoom_at(&mut self, screen_focus: Point, zoom_factor: f32, widget_size: (f32, f32)) {
        let old_zoom = self.zoom;
        let new_zoom = (self.zoom * zoom_factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
        if (new_zoom - old_zoom).abs() < f32::EPSILON {
            return;
        }

        let world_before = self.screen_to_world(screen_focus, widget_size);
        self.zoom = new_zoom;
        let world_after = self.screen_to_world(screen_focus, widget_size);

        self.pan.x += (world_after.x - world_before.x) * self.zoom;
        self.pan.y += (world_after.y - world_before.y) * self.zoom;
    }

    pub fn pan_by(&mut self, dx: f32, dy: f32) {
        self.pan.x += dx;
        self.pan.y += dy;
    }

    pub fn zoom_to_rect(&mut self, world_rect: Rect, widget_size: (f32, f32)) {
        let norm = world_rect.normalize();
        if norm.width <= 0.0 || norm.height <= 0.0 || widget_size.0 <= 0.0 || widget_size.1 <= 0.0 {
            return;
        }
        let padding = 40.0;
        let avail_w = (widget_size.0 - padding * 2.0).max(50.0);
        let avail_h = (widget_size.1 - padding * 2.0).max(50.0);

        let scale_x = avail_w / norm.width;
        let scale_y = avail_h / norm.height;
        let target_zoom = scale_x.min(scale_y).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);

        self.zoom = target_zoom;
        let center = norm.center();
        self.pan.x = -center.x * target_zoom;
        self.pan.y = -center.y * target_zoom;
    }

    pub fn reset(&mut self) {
        self.pan = Point::ZERO;
        self.zoom = 1.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    Rotate,
}

impl TransformHandle {
    pub fn cursor_name(self) -> &'static str {
        match self {
            TransformHandle::TopLeft | TransformHandle::BottomRight => "nwse-resize",
            TransformHandle::TopRight | TransformHandle::BottomLeft => "nesw-resize",
            TransformHandle::Top | TransformHandle::Bottom => "ns-resize",
            TransformHandle::Left | TransformHandle::Right => "ew-resize",
            TransformHandle::Rotate => "crosshair",
        }
    }

    pub fn position(self, bounds: Rect, zoom: f32) -> Point {
        let r = bounds.normalize();
        match self {
            TransformHandle::TopLeft => Point::new(r.x, r.y),
            TransformHandle::Top => Point::new(r.x + r.width / 2.0, r.y),
            TransformHandle::TopRight => Point::new(r.x + r.width, r.y),
            TransformHandle::Right => Point::new(r.x + r.width, r.y + r.height / 2.0),
            TransformHandle::BottomRight => Point::new(r.x + r.width, r.y + r.height),
            TransformHandle::Bottom => Point::new(r.x + r.width / 2.0, r.y + r.height),
            TransformHandle::BottomLeft => Point::new(r.x, r.y + r.height),
            TransformHandle::Left => Point::new(r.x, r.y + r.height / 2.0),
            TransformHandle::Rotate => Point::new(r.x + r.width / 2.0, r.y - 20.0 / zoom),
        }
    }

    pub fn opposite_anchor(self, bounds: Rect) -> Point {
        let r = bounds.normalize();
        match self {
            TransformHandle::TopLeft => Point::new(r.x + r.width, r.y + r.height),
            TransformHandle::Top => Point::new(r.x + r.width / 2.0, r.y + r.height),
            TransformHandle::TopRight => Point::new(r.x, r.y + r.height),
            TransformHandle::Right => Point::new(r.x, r.y + r.height / 2.0),
            TransformHandle::BottomRight => Point::new(r.x, r.y),
            TransformHandle::Bottom => Point::new(r.x + r.width / 2.0, r.y),
            TransformHandle::BottomLeft => Point::new(r.x + r.width, r.y),
            TransformHandle::Left => Point::new(r.x + r.width, r.y + r.height / 2.0),
            TransformHandle::Rotate => r.center(),
        }
    }
}

pub fn hit_transform_handle(bounds: Rect, p: Point, zoom: f32) -> Option<TransformHandle> {
    let handles = [
        TransformHandle::Rotate,
        TransformHandle::TopLeft,
        TransformHandle::Top,
        TransformHandle::TopRight,
        TransformHandle::Right,
        TransformHandle::BottomRight,
        TransformHandle::Bottom,
        TransformHandle::BottomLeft,
        TransformHandle::Left,
    ];
    let radius = (8.0 / zoom).max(6.0);
    for h in handles {
        let pos = h.position(bounds, zoom);
        if p.distance_to(pos) <= radius {
            return Some(h);
        }
    }
    None
}

pub fn calculate_resize_scales(
    handle: TransformHandle,
    origin: Point,
    initial_bounds: Rect,
    current_world: Point,
    shift_pressed: bool,
) -> (f32, f32) {
    let initial_w = initial_bounds.width.max(1.0);
    let initial_h = initial_bounds.height.max(1.0);

    let mut new_w = match handle {
        TransformHandle::TopLeft
        | TransformHandle::BottomLeft
        | TransformHandle::Left => (origin.x - current_world.x).abs(),
        TransformHandle::TopRight
        | TransformHandle::BottomRight
        | TransformHandle::Right => (current_world.x - origin.x).abs(),
        TransformHandle::Top | TransformHandle::Bottom => initial_w,
        TransformHandle::Rotate => initial_w,
    };

    let mut new_h = match handle {
        TransformHandle::TopLeft | TransformHandle::TopRight | TransformHandle::Top => {
            (origin.y - current_world.y).abs()
        }
        TransformHandle::BottomLeft
        | TransformHandle::BottomRight
        | TransformHandle::Bottom => (current_world.y - origin.y).abs(),
        TransformHandle::Left | TransformHandle::Right => initial_h,
        TransformHandle::Rotate => initial_h,
    };

    if shift_pressed {
        let aspect = initial_w / initial_h;
        match handle {
            TransformHandle::Top | TransformHandle::Bottom => {
                new_w = (new_h * aspect).round();
            }
            TransformHandle::Left | TransformHandle::Right => {
                new_h = (new_w / aspect).round();
            }
            _ => {
                let size = (new_w / aspect).max(new_h).round();
                new_w = (size * aspect).round();
                new_h = size;
            }
        }
    }

    new_w = new_w.round().max(1.0);
    new_h = new_h.round().max(1.0);

    (new_w / initial_w, new_h / initial_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_pixel_round() {
        let p = Point::new(10.4, 20.6);
        assert_eq!(p.round(), Point::new(10.0, 21.0));
    }

    #[test]
    fn test_rect_pixel_round() {
        let r = Rect::new(10.2, 20.7, 49.6, 50.1);
        let rounded = r.round();
        assert_eq!(rounded, Rect::new(10.0, 21.0, 50.0, 50.0));
    }

    #[test]
    fn test_viewport_zoom_to_rect() {
        let mut vp = Viewport::default();
        let target = Rect::new(100.0, 100.0, 400.0, 200.0);
        let widget_size = (800.0, 600.0);
        vp.zoom_to_rect(target, widget_size);

        // Center of screen (400, 300) should map to target center in world space (300, 200)
        let world_center = vp.screen_to_world(Point::new(400.0, 300.0), widget_size);
        assert!((world_center.x - target.center().x).abs() < 1.0);
        assert!((world_center.y - target.center().y).abs() < 1.0);
        assert!(vp.zoom > 0.5 && vp.zoom < 3.0);
    }
}
