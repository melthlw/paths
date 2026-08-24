use gtk4::gdk;
use gtk4::glib;
use skia_safe as skia;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct CursorCache {
    cursors: RefCell<HashMap<String, gdk::Cursor>>,
}

impl Default for CursorCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorCache {
    pub fn new() -> Self {
        Self {
            cursors: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_cursor(&self, name: &str) -> Option<gdk::Cursor> {
        if let Some(c) = self.cursors.borrow().get(name) {
            return Some(c.clone());
        }

        let maybe_cursor = Self::create_cursor_by_name(name);
        if let Some(ref c) = maybe_cursor {
            self.cursors.borrow_mut().insert(name.to_string(), c.clone());
        }
        maybe_cursor
    }

    fn create_cursor_by_name(name: &str) -> Option<gdk::Cursor> {
        match name {
            // Tool Cursors
            "tool:pen" => Self::make_pen_cursor(None),
            "tool:pen_add" => Self::make_pen_cursor(Some(Badge::Plus)),
            "tool:pen_remove" => Self::make_pen_cursor(Some(Badge::Minus)),
            "tool:pen_close" => Self::make_pen_cursor(Some(Badge::Close)),

            "tool:node" => Self::make_node_cursor(None),
            "tool:node_add" => Self::make_node_cursor(Some(Badge::Plus)),
            "tool:node_curve" => Self::make_node_cursor(Some(Badge::Curve)),

            "tool:select" => gdk::Cursor::from_name("default", None),
            "tool:rotate" => Self::make_rotate_cursor(),

            "tool:rectangle" => Self::make_shape_cursor(ShapeType::Rectangle),
            "tool:circle" => Self::make_shape_cursor(ShapeType::Circle),
            "tool:star" => Self::make_shape_cursor(ShapeType::Star),
            "tool:spiral" => Self::make_shape_cursor(ShapeType::Spiral),
            "tool:triangle" => Self::make_shape_cursor(ShapeType::Triangle),

            "tool:brush" => Self::make_brush_cursor(),
            "tool:eraser" => Self::make_eraser_cursor(),
            "tool:text" => Self::make_text_cursor(),
            "tool:gradient" => Self::make_gradient_cursor(),
            "tool:eyedropper" => Self::make_eyedropper_cursor(),
            "tool:measure" => Self::make_measure_cursor(),
            "tool:page" => Self::make_page_cursor(),
            "tool:zoom_in" => gdk::Cursor::from_name("zoom-in", None),
            "tool:zoom_out" => gdk::Cursor::from_name("zoom-out", None),

            // Standard Fallbacks
            other => gdk::Cursor::from_name(other, None),
        }
    }

    fn create_cursor_from_skia(
        width: i32,
        height: i32,
        hotspot_x: i32,
        hotspot_y: i32,
        draw_fn: impl FnOnce(&skia::Canvas),
    ) -> Option<gdk::Cursor> {
        let mut surface = skia::surfaces::raster_n32_premul((width, height))?;
        let canvas = surface.canvas();
        canvas.clear(skia::Color::TRANSPARENT);
        draw_fn(canvas);

        let image = surface.image_snapshot();
        let info = skia::ImageInfo::new(
            (width, height),
            skia::ColorType::RGBA8888,
            skia::AlphaType::Premul,
            None,
        );
        let stride = (width * 4) as usize;
        let mut bytes = vec![0u8; stride * height as usize];
        if !image.read_pixels(&info, &mut bytes, stride, (0, 0), skia::image::CachingHint::Disallow) {
            return None;
        }

        let glib_bytes = glib::Bytes::from(&bytes);
        let texture = gdk::MemoryTexture::new(
            width,
            height,
            gdk::MemoryFormat::R8g8b8a8Premultiplied,
            &glib_bytes,
            stride,
        );

        Some(gdk::Cursor::from_texture(
            &texture,
            hotspot_x,
            hotspot_y,
            Option::<&gdk::Cursor>::None,
        ))
    }

    fn make_pen_cursor(badge: Option<Badge>) -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 2, 2, |canvas| {
            let mut halo_paint = skia::Paint::default();
            halo_paint.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.6), None);
            halo_paint.set_style(skia::PaintStyle::Stroke);
            halo_paint.set_stroke_width(2.5);
            halo_paint.set_anti_alias(true);

            let mut body_paint = skia::Paint::default();
            body_paint.set_color4f(skia::Color4f::new(0.95, 0.95, 0.98, 1.0), None);
            body_paint.set_style(skia::PaintStyle::Fill);
            body_paint.set_anti_alias(true);

            let mut stroke_paint = skia::Paint::default();
            stroke_paint.set_color4f(skia::Color4f::new(0.12, 0.15, 0.20, 1.0), None);
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_stroke_width(1.2);
            stroke_paint.set_anti_alias(true);

            // Pen nib path
            let mut path = skia::PathBuilder::new();
            path.move_to((2.0, 2.0));
            path.line_to((9.0, 16.0));
            path.line_to((15.0, 15.0));
            path.line_to((16.0, 9.0));
            path.close();

            let nib = path.detach();
            canvas.draw_path(&nib, &halo_paint);
            canvas.draw_path(&nib, &body_paint);
            canvas.draw_path(&nib, &stroke_paint);

            // Center slit & breather hole
            let mut line_paint = skia::Paint::default();
            line_paint.set_color4f(skia::Color4f::new(0.2, 0.2, 0.25, 1.0), None);
            line_paint.set_style(skia::PaintStyle::Stroke);
            line_paint.set_stroke_width(1.0);
            line_paint.set_anti_alias(true);
            canvas.draw_line((2.0, 2.0), (10.0, 10.0), &line_paint);
            canvas.draw_circle((10.0, 10.0), 1.2, &line_paint);

            if let Some(b) = badge {
                Self::draw_badge(canvas, 19.0, 19.0, b);
            }
        })
    }

    fn make_node_cursor(badge: Option<Badge>) -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 2, 2, |canvas| {
            let mut halo = skia::Paint::default();
            halo.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.6), None);
            halo.set_style(skia::PaintStyle::Stroke);
            halo.set_stroke_width(2.5);
            halo.set_anti_alias(true);

            let mut body = skia::Paint::default();
            body.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            body.set_style(skia::PaintStyle::Fill);
            body.set_anti_alias(true);

            let mut border = skia::Paint::default();
            border.set_color4f(skia::Color4f::new(0.15, 0.45, 0.85, 1.0), None);
            border.set_style(skia::PaintStyle::Stroke);
            border.set_stroke_width(1.5);
            border.set_anti_alias(true);

            // Precision hollow pointer arrow
            let mut arrow = skia::PathBuilder::new();
            arrow.move_to((2.0, 2.0));
            arrow.line_to((2.0, 16.0));
            arrow.line_to((6.0, 12.0));
            arrow.line_to((10.0, 19.0));
            arrow.line_to((12.5, 17.5));
            arrow.line_to((8.5, 11.0));
            arrow.line_to((14.0, 11.0));
            arrow.close();

            let ap = arrow.detach();
            canvas.draw_path(&ap, &halo);
            canvas.draw_path(&ap, &body);
            canvas.draw_path(&ap, &border);

            // Node indicator badge or requested badge
            if let Some(b) = badge {
                Self::draw_badge(canvas, 19.0, 19.0, b);
            } else {
                // Small node square badge at bottom right
                let mut node_bg = skia::Paint::default();
                node_bg.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
                node_bg.set_style(skia::PaintStyle::Fill);
                node_bg.set_anti_alias(true);

                let mut node_fg = skia::Paint::default();
                node_fg.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                node_fg.set_style(skia::PaintStyle::Fill);
                node_fg.set_anti_alias(true);

                let r = skia::Rect::from_xywh(15.0, 15.0, 8.0, 8.0);
                canvas.draw_round_rect(r, 1.5, 1.5, &node_bg);
                let inner = skia::Rect::from_xywh(17.0, 17.0, 4.0, 4.0);
                canvas.draw_rect(inner, &node_fg);
            }
        })
    }

    fn make_shape_cursor(shape: ShapeType) -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 6, 6, |canvas| {
            // Precision Crosshair at top-left (hotspot at (6, 6))
            Self::draw_precision_crosshair(canvas, 6.0, 6.0);

            // Shape badge container at (18, 18)
            let mut bg = skia::Paint::default();
            bg.set_color4f(skia::Color4f::new(0.15, 0.18, 0.22, 0.9), None);
            bg.set_style(skia::PaintStyle::Fill);
            bg.set_anti_alias(true);

            let mut fg = skia::Paint::default();
            fg.set_color4f(skia::Color4f::new(0.9, 0.92, 0.95, 1.0), None);
            fg.set_style(skia::PaintStyle::Stroke);
            fg.set_stroke_width(1.2);
            fg.set_anti_alias(true);

            let badge_rect = skia::Rect::from_xywh(14.0, 14.0, 15.0, 15.0);
            canvas.draw_round_rect(badge_rect, 3.0, 3.0, &bg);

            match shape {
                ShapeType::Rectangle => {
                    canvas.draw_rect(skia::Rect::from_xywh(16.5, 17.5, 10.0, 8.0), &fg);
                }
                ShapeType::Circle => {
                    canvas.draw_circle((21.5, 21.5), 4.5, &fg);
                }
                ShapeType::Star => {
                    let mut p = skia::PathBuilder::new();
                    let cx = 21.5;
                    let cy = 21.5;
                    for i in 0..10 {
                        let angle = (i as f32) * std::f32::consts::PI / 5.0 - std::f32::consts::FRAC_PI_2;
                        let r = if i % 2 == 0 { 5.0 } else { 2.2 };
                        let x = cx + angle.cos() * r;
                        let y = cy + angle.sin() * r;
                        if i == 0 {
                            p.move_to((x, y));
                        } else {
                            p.line_to((x, y));
                        }
                    }
                    p.close();
                    canvas.draw_path(&p.detach(), &fg);
                }
                ShapeType::Spiral => {
                    let mut p = skia::PathBuilder::new();
                    let cx = 21.5;
                    let cy = 21.5;
                    p.move_to((cx, cy));
                    for i in 1..=24 {
                        let t = i as f32 / 24.0;
                        let angle = t * std::f32::consts::PI * 3.5;
                        let r = t * 4.5;
                        p.line_to((cx + angle.cos() * r, cy + angle.sin() * r));
                    }
                    canvas.draw_path(&p.detach(), &fg);
                }
                ShapeType::Triangle => {
                    let mut p = skia::PathBuilder::new();
                    p.move_to((21.5, 16.5));
                    p.line_to((26.5, 25.5));
                    p.line_to((16.5, 25.5));
                    p.close();
                    canvas.draw_path(&p.detach(), &fg);
                }
            }
        })
    }

    fn make_brush_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 2, 2, |canvas| {
            let mut halo = skia::Paint::default();
            halo.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.6), None);
            halo.set_style(skia::PaintStyle::Stroke);
            halo.set_stroke_width(2.5);
            halo.set_anti_alias(true);

            let mut body = skia::Paint::default();
            body.set_color4f(skia::Color4f::new(0.95, 0.95, 0.95, 1.0), None);
            body.set_style(skia::PaintStyle::Fill);
            body.set_anti_alias(true);

            let mut tip = skia::Paint::default();
            tip.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
            tip.set_style(skia::PaintStyle::Fill);
            tip.set_anti_alias(true);

            let mut stroke = skia::Paint::default();
            stroke.set_color4f(skia::Color4f::new(0.12, 0.15, 0.20, 1.0), None);
            stroke.set_style(skia::PaintStyle::Stroke);
            stroke.set_stroke_width(1.2);
            stroke.set_anti_alias(true);

            let mut p = skia::PathBuilder::new();
            p.move_to((2.0, 2.0));
            p.line_to((7.0, 10.0));
            p.line_to((19.0, 22.0));
            p.line_to((23.0, 18.0));
            p.line_to((11.0, 6.0));
            p.close();

            let bp = p.detach();
            canvas.draw_path(&bp, &halo);
            canvas.draw_path(&bp, &body);
            canvas.draw_path(&bp, &stroke);

            // Brush bristle tip
            let mut tp = skia::PathBuilder::new();
            tp.move_to((2.0, 2.0));
            tp.line_to((7.0, 10.0));
            tp.line_to((11.0, 6.0));
            tp.close();
            canvas.draw_path(&tp.detach(), &tip);
        })
    }

    fn make_eraser_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 10, 10, |canvas| {
            let mut outer = skia::Paint::default();
            outer.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.7), None);
            outer.set_style(skia::PaintStyle::Stroke);
            outer.set_stroke_width(2.0);
            outer.set_anti_alias(true);

            let mut inner = skia::Paint::default();
            inner.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            inner.set_style(skia::PaintStyle::Stroke);
            inner.set_stroke_width(1.2);
            inner.set_anti_alias(true);

            canvas.draw_circle((10.0, 10.0), 6.5, &outer);
            canvas.draw_circle((10.0, 10.0), 6.5, &inner);
            canvas.draw_point((10.0, 10.0), &inner);

            // Eraser angled badge
            let mut badge_bg = skia::Paint::default();
            badge_bg.set_color4f(skia::Color4f::new(0.95, 0.4, 0.4, 1.0), None);
            badge_bg.set_style(skia::PaintStyle::Fill);
            badge_bg.set_anti_alias(true);

            let r = skia::Rect::from_xywh(18.0, 16.0, 11.0, 11.0);
            canvas.draw_round_rect(r, 2.0, 2.0, &badge_bg);
        })
    }

    fn make_eyedropper_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 2, 22, |canvas| {
            let mut halo = skia::Paint::default();
            halo.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.6), None);
            halo.set_style(skia::PaintStyle::Stroke);
            halo.set_stroke_width(2.5);
            halo.set_anti_alias(true);

            let mut body = skia::Paint::default();
            body.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            body.set_style(skia::PaintStyle::Fill);
            body.set_anti_alias(true);

            let mut stroke = skia::Paint::default();
            stroke.set_color4f(skia::Color4f::new(0.12, 0.15, 0.20, 1.0), None);
            stroke.set_style(skia::PaintStyle::Stroke);
            stroke.set_stroke_width(1.2);
            stroke.set_anti_alias(true);

            let mut p = skia::PathBuilder::new();
            p.move_to((2.0, 22.0));
            p.line_to((2.0, 18.0));
            p.line_to((14.0, 6.0));
            p.line_to((18.0, 10.0));
            p.line_to((6.0, 22.0));
            p.close();

            let dp = p.detach();
            canvas.draw_path(&dp, &halo);
            canvas.draw_path(&dp, &body);
            canvas.draw_path(&dp, &stroke);

            // Pipette bulb
            let mut bulb = skia::PathBuilder::new();
            bulb.move_to((14.0, 6.0));
            bulb.line_to((19.0, 1.0));
            bulb.line_to((23.0, 5.0));
            bulb.line_to((18.0, 10.0));
            bulb.close();

            let mut bulb_paint = skia::Paint::default();
            bulb_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
            bulb_paint.set_style(skia::PaintStyle::Fill);
            bulb_paint.set_anti_alias(true);

            let bp = bulb.detach();
            canvas.draw_path(&bp, &bulb_paint);
            canvas.draw_path(&bp, &stroke);
        })
    }

    fn make_text_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 7, 10, |canvas| {
            let mut halo = skia::Paint::default();
            halo.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.6), None);
            halo.set_style(skia::PaintStyle::Stroke);
            halo.set_stroke_width(2.0);
            halo.set_anti_alias(true);

            let mut stroke = skia::Paint::default();
            stroke.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            stroke.set_style(skia::PaintStyle::Stroke);
            stroke.set_stroke_width(1.5);
            stroke.set_anti_alias(true);

            // I-Beam
            canvas.draw_line((4.0, 2.0), (10.0, 2.0), &halo);
            canvas.draw_line((4.0, 18.0), (10.0, 18.0), &halo);
            canvas.draw_line((7.0, 2.0), (7.0, 18.0), &halo);

            canvas.draw_line((4.0, 2.0), (10.0, 2.0), &stroke);
            canvas.draw_line((4.0, 18.0), (10.0, 18.0), &stroke);
            canvas.draw_line((7.0, 2.0), (7.0, 18.0), &stroke);

            // 'A' badge
            let mut bg = skia::Paint::default();
            bg.set_color4f(skia::Color4f::new(0.15, 0.45, 0.85, 1.0), None);
            bg.set_style(skia::PaintStyle::Fill);
            bg.set_anti_alias(true);

            let r = skia::Rect::from_xywh(16.0, 14.0, 13.0, 13.0);
            canvas.draw_round_rect(r, 2.5, 2.5, &bg);

            let mut font_paint = skia::Paint::default();
            font_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            font_paint.set_style(skia::PaintStyle::Stroke);
            font_paint.set_stroke_width(1.2);
            font_paint.set_anti_alias(true);

            // Draw clean 'A' glyph inside badge
            let mut a_path = skia::PathBuilder::new();
            a_path.move_to((19.0, 24.0));
            a_path.line_to((22.5, 17.0));
            a_path.line_to((26.0, 24.0));
            a_path.move_to((20.5, 22.0));
            a_path.line_to((24.5, 22.0));
            canvas.draw_path(&a_path.detach(), &font_paint);
        })
    }

    fn make_gradient_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 6, 6, |canvas| {
            Self::draw_precision_crosshair(canvas, 6.0, 6.0);

            // Gradient square badge
            let badge_rect = skia::Rect::from_xywh(14.0, 14.0, 15.0, 15.0);
            let mut border = skia::Paint::default();
            border.set_color4f(skia::Color4f::new(0.9, 0.92, 0.95, 1.0), None);
            border.set_style(skia::PaintStyle::Stroke);
            border.set_stroke_width(1.2);
            border.set_anti_alias(true);

            let mut grad_paint = skia::Paint::default();
            grad_paint.set_style(skia::PaintStyle::Fill);
            grad_paint.set_anti_alias(true);
            let colors = [
                skia::Color4f::new(0.2, 0.55, 0.95, 1.0),
                skia::Color4f::new(0.9, 0.3, 0.7, 1.0),
            ];
            let colors_desc = skia::gradient::Colors::new(&colors[..], None, skia::TileMode::Clamp, None);
            let grad_desc = skia::gradient::Gradient::new(colors_desc, skia::gradient::Interpolation::default());
            if let Some(shader) = skia::gradient::shaders::linear_gradient(
                (skia::Point::new(14.0, 14.0), skia::Point::new(29.0, 29.0)),
                &grad_desc,
                None,
            ) {
                grad_paint.set_shader(shader);
            }
            canvas.draw_round_rect(badge_rect, 2.5, 2.5, &grad_paint);
            canvas.draw_round_rect(badge_rect, 2.5, 2.5, &border);
        })
    }

    fn make_measure_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 6, 6, |canvas| {
            Self::draw_precision_crosshair(canvas, 6.0, 6.0);

            let badge_rect = skia::Rect::from_xywh(14.0, 15.0, 15.0, 13.0);
            let mut bg = skia::Paint::default();
            bg.set_color4f(skia::Color4f::new(0.95, 0.85, 0.3, 1.0), None);
            bg.set_style(skia::PaintStyle::Fill);
            bg.set_anti_alias(true);

            let mut border = skia::Paint::default();
            border.set_color4f(skia::Color4f::new(0.2, 0.2, 0.2, 1.0), None);
            border.set_style(skia::PaintStyle::Stroke);
            border.set_stroke_width(1.0);
            border.set_anti_alias(true);

            canvas.draw_round_rect(badge_rect, 1.5, 1.5, &bg);
            canvas.draw_round_rect(badge_rect, 1.5, 1.5, &border);

            // Tick marks
            canvas.draw_line((17.0, 15.0), (17.0, 19.0), &border);
            canvas.draw_line((20.0, 15.0), (20.0, 18.0), &border);
            canvas.draw_line((23.0, 15.0), (23.0, 19.0), &border);
            canvas.draw_line((26.0, 15.0), (26.0, 18.0), &border);
        })
    }

    fn make_page_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 6, 6, |canvas| {
            Self::draw_precision_crosshair(canvas, 6.0, 6.0);

            let mut border = skia::Paint::default();
            border.set_color4f(skia::Color4f::new(0.95, 0.95, 0.98, 1.0), None);
            border.set_style(skia::PaintStyle::Stroke);
            border.set_stroke_width(1.2);
            border.set_anti_alias(true);

            let mut bg = skia::Paint::default();
            bg.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 0.35), None);
            bg.set_style(skia::PaintStyle::Fill);

            let r = skia::Rect::from_xywh(14.0, 14.0, 14.0, 15.0);
            canvas.draw_rect(r, &bg);
            canvas.draw_rect(r, &border);

            // Folded corner
            canvas.draw_line((24.0, 14.0), (28.0, 18.0), &border);
        })
    }

    fn make_rotate_cursor() -> Option<gdk::Cursor> {
        Self::create_cursor_from_skia(32, 32, 16, 16, |canvas| {
            let mut halo = skia::Paint::default();
            halo.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.7), None);
            halo.set_style(skia::PaintStyle::Stroke);
            halo.set_stroke_width(3.0);
            halo.set_anti_alias(true);

            let mut stroke = skia::Paint::default();
            stroke.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            stroke.set_style(skia::PaintStyle::Stroke);
            stroke.set_stroke_width(1.6);
            stroke.set_anti_alias(true);

            let mut p = skia::PathBuilder::new();
            p.arc_to(skia::Rect::from_xywh(10.0, 10.0, 12.0, 12.0), -60.0, 240.0, false);

            let arc = p.detach();
            canvas.draw_path(&arc, &halo);
            canvas.draw_path(&arc, &stroke);

            // Arrowheads on curved path
            let mut arr = skia::PathBuilder::new();
            arr.move_to((19.0, 6.0));
            arr.line_to((24.0, 8.0));
            arr.line_to((21.0, 13.0));
            arr.close();

            let ap = arr.detach();
            let mut fill = skia::Paint::default();
            fill.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            fill.set_style(skia::PaintStyle::Fill);
            fill.set_anti_alias(true);

            canvas.draw_path(&ap, &halo);
            canvas.draw_path(&ap, &fill);
        })
    }

    fn draw_precision_crosshair(canvas: &skia::Canvas, cx: f32, cy: f32) {
        let mut halo = skia::Paint::default();
        halo.set_color4f(skia::Color4f::new(0.0, 0.0, 0.0, 0.7), None);
        halo.set_style(skia::PaintStyle::Stroke);
        halo.set_stroke_width(2.5);
        halo.set_anti_alias(true);

        let mut stroke = skia::Paint::default();
        stroke.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        stroke.set_style(skia::PaintStyle::Stroke);
        stroke.set_stroke_width(1.2);
        stroke.set_anti_alias(true);

        // Cross lines with center gap
        canvas.draw_line((cx - 5.0, cy), (cx - 1.5, cy), &halo);
        canvas.draw_line((cx + 1.5, cy), (cx + 5.0, cy), &halo);
        canvas.draw_line((cx, cy - 5.0), (cx, cy - 1.5), &halo);
        canvas.draw_line((cx, cy + 1.5), (cx, cy + 5.0), &halo);

        canvas.draw_line((cx - 5.0, cy), (cx - 1.5, cy), &stroke);
        canvas.draw_line((cx + 1.5, cy), (cx + 5.0, cy), &stroke);
        canvas.draw_line((cx, cy - 5.0), (cx, cy - 1.5), &stroke);
        canvas.draw_line((cx, cy + 1.5), (cx, cy + 5.0), &stroke);

        // Center dot
        let mut dot = skia::Paint::default();
        dot.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        dot.set_style(skia::PaintStyle::Fill);
        canvas.draw_circle((cx, cy), 0.7, &dot);
    }

    fn draw_badge(canvas: &skia::Canvas, x: f32, y: f32, badge: Badge) {
        let mut bg = skia::Paint::default();
        bg.set_color4f(skia::Color4f::new(0.12, 0.15, 0.20, 0.95), None);
        bg.set_style(skia::PaintStyle::Fill);
        bg.set_anti_alias(true);

        let mut border = skia::Paint::default();
        border.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 0.9), None);
        border.set_style(skia::PaintStyle::Stroke);
        border.set_stroke_width(1.0);
        border.set_anti_alias(true);

        let mut icon_paint = skia::Paint::default();
        icon_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        icon_paint.set_style(skia::PaintStyle::Stroke);
        icon_paint.set_stroke_width(1.4);
        icon_paint.set_anti_alias(true);

        let circle = (x + 5.0, y + 5.0);
        canvas.draw_circle(circle, 5.5, &bg);
        canvas.draw_circle(circle, 5.5, &border);

        match badge {
            Badge::Plus => {
                canvas.draw_line((x + 2.5, y + 5.0), (x + 7.5, y + 5.0), &icon_paint);
                canvas.draw_line((x + 5.0, y + 2.5), (x + 5.0, y + 7.5), &icon_paint);
            }
            Badge::Minus => {
                canvas.draw_line((x + 2.5, y + 5.0), (x + 7.5, y + 5.0), &icon_paint);
            }
            Badge::Close => {
                canvas.draw_circle(circle, 2.5, &icon_paint);
            }
            Badge::Curve => {
                let mut p = skia::PathBuilder::new();
                p.move_to((x + 2.5, y + 6.5));
                p.quad_to((x + 5.0, y + 2.5), (x + 7.5, y + 6.5));
                canvas.draw_path(&p.detach(), &icon_paint);
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Badge {
    Plus,
    Minus,
    Close,
    Curve,
}

#[derive(Clone, Copy)]
enum ShapeType {
    Rectangle,
    Circle,
    Star,
    Spiral,
    Triangle,
}
