use skia_safe as skia;

use super::style::BlendMode;
use super::{Element, ElementId};
use crate::core::geometry::{Point, Rect};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GroupElement {
    pub id: ElementId,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
    pub children: Vec<Element>,
    pub clip_element: Option<Box<Element>>,
    #[serde(default)]
    pub modifiers: Vec<crate::core::modifier::Modifier>,
}

impl GroupElement {
    pub fn new(children: Vec<Element>) -> Self {
        Self {
            id: ElementId::new(),
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
            children,
            clip_element: None,
            modifiers: Vec::new(),
        }
    }

    pub fn bounds(&self) -> Rect {
        if let Some(clip) = &self.clip_element {
            return clip.bounds();
        }
        if self.children.is_empty() {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        }
        let mut b = self.children[0].bounds();
        for child in &self.children[1..] {
            b = b.union(child.bounds());
        }
        b
    }

    pub fn hit_test(&self, p: Point) -> bool {
        if let Some(clip) = &self.clip_element {
            if !clip.hit_test(p) {
                return false;
            }
        }
        self.children.iter().any(|c| c.hit_test(p))
    }

    pub fn translate_with_options(
        &mut self,
        dx: f32,
        dy: f32,
        options: &crate::core::TransformOptions,
    ) {
        for child in &mut self.children {
            child.translate_with_options(dx, dy, options);
        }
        if let Some(clip) = &mut self.clip_element {
            clip.translate_with_options(dx, dy, options);
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.scale_with_options(origin, sx, sy, &crate::core::TransformOptions::default());
    }

    pub fn scale_with_options(
        &mut self,
        origin: Point,
        sx: f32,
        sy: f32,
        options: &crate::core::TransformOptions,
    ) {
        for child in &mut self.children {
            child.scale_with_options(origin, sx, sy, options);
        }
        if let Some(clip) = &mut self.clip_element {
            clip.scale_with_options(origin, sx, sy, options);
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        for child in &mut self.children {
            child.rotate(center, angle_rad);
        }
        if let Some(clip) = &mut self.clip_element {
            clip.rotate(center, angle_rad);
        }
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        if !self.visible {
            return;
        }
        if let Some(clip) = &self.clip_element {
            let path = clip.to_skia_path();
            canvas.save();
            canvas.clip_path(&path, None, true);
            for child in &self.children {
                self.render_child(child, canvas);
            }
            canvas.restore();
        } else {
            for child in &self.children {
                self.render_child(child, canvas);
            }
        }
    }

    fn render_child(&self, child: &Element, canvas: &skia::Canvas) {
        if !child.visible() {
            return;
        }
        if child.opacity() < 0.999
            || child.blur() > 0.001
            || child.blend_mode() != BlendMode::Normal
        {
            let mut layer_paint = skia::Paint::default();
            layer_paint.set_alpha_f(child.opacity().clamp(0.0, 1.0));
            layer_paint.set_blend_mode(child.blend_mode().to_skia());
            if child.blur() > 0.001 {
                let sigma = child.blur() * 40.0;
                if let Some(blur_filter) =
                    skia::image_filters::blur((sigma, sigma), skia::TileMode::Decal, None, None)
                {
                    layer_paint.set_image_filter(blur_filter);
                }
            }
            canvas.save_layer(&skia::canvas::SaveLayerRec::default().paint(&layer_paint));
            child.render(canvas);
            canvas.restore();
        } else {
            child.render(canvas);
        }
    }
}

fn default_one() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ImageElement {
    pub id: ElementId,
    pub name: Option<String>,
    pub rect: Rect,
    pub image_data: Vec<u8>,
    pub opacity: f32,
    pub visible: bool,
    pub locked: bool,
    pub blend_mode: BlendMode,
    pub blur: f32,
    #[serde(default)]
    pub brightness: f32,
    #[serde(default = "default_one")]
    pub contrast: f32,
    #[serde(default = "default_one")]
    pub saturation: f32,
    #[serde(default)]
    pub hue_rotate: f32,
    #[serde(default)]
    pub invert: bool,
    #[serde(default)]
    pub grayscale: bool,
    #[serde(default)]
    pub sepia: bool,
    #[serde(default)]
    pub modifiers: Vec<crate::core::modifier::Modifier>,
}

impl ImageElement {
    pub fn new(rect: Rect, image_data: Vec<u8>, name: Option<String>) -> Self {
        Self {
            id: ElementId::new(),
            name,
            rect,
            image_data,
            opacity: 1.0,
            visible: true,
            locked: false,
            blend_mode: BlendMode::Normal,
            blur: 0.0,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue_rotate: 0.0,
            invert: false,
            grayscale: false,
            sepia: false,
            modifiers: Vec::new(),
        }
    }

    pub fn new_placeholder(rect: Rect, name: Option<String>) -> Self {
        Self::new(
            rect,
            Vec::new(),
            name.or_else(|| Some("Image Frame".to_string())),
        )
    }

    pub fn bounds(&self) -> Rect {
        self.rect.normalize()
    }

    pub fn intrinsic_size(&self) -> Option<(f32, f32)> {
        if self.image_data.is_empty() {
            return None;
        }
        if let Some(image) = skia::Image::from_encoded(skia::Data::new_copy(&self.image_data)) {
            Some((image.width() as f32, image.height() as f32))
        } else {
            None
        }
    }

    pub fn hit_test(&self, p: Point) -> bool {
        self.bounds().contains(p)
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.rect.x += dx;
        self.rect.y += dy;
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.rect.x = origin.x + (self.rect.x - origin.x) * sx;
        self.rect.y = origin.y + (self.rect.y - origin.y) * sy;
        self.rect.width *= sx;
        self.rect.height *= sy;
    }

    pub fn color_matrix(&self) -> [f32; 20] {
        let mut mat = [
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
        ];

        // 1. Invert
        if self.invert {
            let inv = [
                -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
            ];
            mat = concat_color_matrices(&inv, &mat);
        }

        // 2. Sepia
        if self.sepia {
            let sep = [
                0.393, 0.769, 0.189, 0.0, 0.0, 0.349, 0.686, 0.168, 0.0, 0.0, 0.272, 0.534, 0.131,
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            ];
            mat = concat_color_matrices(&sep, &mat);
        }

        // 3. Saturation / Grayscale
        let sat = if self.grayscale { 0.0 } else { self.saturation };
        if (sat - 1.0).abs() > 0.001 {
            let lr = 0.2126;
            let lg = 0.7152;
            let lb = 0.0722;
            let inv_s = 1.0 - sat;
            let r_s = inv_s * lr;
            let g_s = inv_s * lg;
            let b_s = inv_s * lb;
            let sat_mat = [
                r_s + sat,
                g_s,
                b_s,
                0.0,
                0.0,
                r_s,
                g_s + sat,
                b_s,
                0.0,
                0.0,
                r_s,
                g_s,
                b_s + sat,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
            ];
            mat = concat_color_matrices(&sat_mat, &mat);
        }

        // 4. Hue Rotate
        if self.hue_rotate.abs() > 0.001 {
            let rad = self.hue_rotate * std::f32::consts::PI / 180.0;
            let c = rad.cos();
            let s = rad.sin();
            let hue_mat = [
                0.213 + c * 0.787 - s * 0.213,
                0.715 - c * 0.715 - s * 0.715,
                0.072 - c * 0.072 + s * 0.928,
                0.0,
                0.0,
                0.213 - c * 0.213 + s * 0.143,
                0.715 + c * 0.285 + s * 0.140,
                0.072 - c * 0.072 - s * 0.283,
                0.0,
                0.0,
                0.213 - c * 0.213 - s * 0.787,
                0.715 - c * 0.715 + s * 0.715,
                0.072 + c * 0.928 + s * 0.072,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
            ];
            mat = concat_color_matrices(&hue_mat, &mat);
        }

        // 5. Brightness & Contrast
        if self.brightness.abs() > 0.001 || (self.contrast - 1.0).abs() > 0.001 {
            let c = self.contrast;
            let b_offset = 0.5 * (1.0 - c) + self.brightness;
            let bc_mat = [
                c, 0.0, 0.0, 0.0, b_offset, 0.0, c, 0.0, 0.0, b_offset, 0.0, 0.0, c, 0.0, b_offset,
                0.0, 0.0, 0.0, 1.0, 0.0,
            ];
            mat = concat_color_matrices(&bc_mat, &mat);
        }

        mat
    }

    pub fn has_active_adjustments(&self) -> bool {
        self.brightness.abs() > 0.001
            || (self.contrast - 1.0).abs() > 0.001
            || (self.saturation - 1.0).abs() > 0.001
            || self.hue_rotate.abs() > 0.001
            || self.invert
            || self.grayscale
            || self.sepia
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        if !self.visible {
            return;
        }

        let r = self.rect.normalize();
        let dst = skia::Rect::from_xywh(r.x, r.y, r.width, r.height);

        if !self.image_data.is_empty() {
            if let Some(image) = skia::Image::from_encoded(skia::Data::new_copy(&self.image_data)) {
                let mut paint = skia::Paint::default();
                paint.set_anti_alias(true);
                paint.set_alpha_f(self.opacity.clamp(0.0, 1.0));
                paint.set_blend_mode(self.blend_mode.to_skia());

                if self.has_active_adjustments() {
                    let matrix = self.color_matrix();
                    let mut cm = skia::ColorMatrix::default();
                    cm.set_row_major(&matrix);
                    let cf = skia::color_filters::matrix(&cm, None);
                    paint.set_color_filter(cf);
                }

                if self.blur > 0.0 {
                    if let Some(bf) = skia::image_filters::blur(
                        (self.blur, self.blur),
                        skia::TileMode::Clamp,
                        None,
                        None,
                    ) {
                        paint.set_image_filter(bf);
                    }
                }

                canvas.draw_image_rect_with_sampling_options(
                    image,
                    None,
                    dst,
                    skia::SamplingOptions::default(),
                    &paint,
                );
                return;
            }
        }

        // ── 100% Vector Placeholder: Clean Light Gray (#ECECEF / #F2F2F5), Monochrome, No Colors ──
        let alpha = self.opacity.clamp(0.0, 1.0);

        // 1. Light Gray Background
        let mut bg_paint = skia::Paint::default();
        bg_paint.set_color4f(skia::Color4f::new(0.925, 0.925, 0.935, alpha), None);
        bg_paint.set_anti_alias(true);
        canvas.draw_rect(dst, &bg_paint);

        // 2. Diagonal frame guide lines (monochrome light gray)
        let mut diag_paint = skia::Paint::default();
        diag_paint.set_color4f(skia::Color4f::new(0.82, 0.82, 0.85, alpha * 0.6), None);
        diag_paint.set_stroke_width(1.0);
        diag_paint.set_anti_alias(true);
        canvas.draw_line(
            skia::Point::new(r.x, r.y),
            skia::Point::new(r.x + r.width, r.y + r.height),
            &diag_paint,
        );
        canvas.draw_line(
            skia::Point::new(r.x + r.width, r.y),
            skia::Point::new(r.x, r.y + r.height),
            &diag_paint,
        );

        // 3. Center Circular Backdrop (monochrome soft neutral gray)
        let cx = r.x + r.width * 0.5;
        let cy = r.y + r.height * 0.5;
        let min_dim = r.width.min(r.height);
        let circle_r = (min_dim * 0.25).clamp(16.0, 60.0);

        let mut circle_bg = skia::Paint::default();
        circle_bg.set_color4f(skia::Color4f::new(0.85, 0.85, 0.87, alpha), None);
        circle_bg.set_anti_alias(true);
        canvas.draw_circle(skia::Point::new(cx, cy), circle_r, &circle_bg);

        // 4. Monochrome Vector Photo Frame Glyph (Zero color!)
        let glyph_w = (circle_r * 1.12).clamp(18.0, 52.0);
        let glyph_h = glyph_w * 0.72;
        let gx = cx - glyph_w * 0.5;
        let gy = cy - glyph_h * 0.5;
        let glyph_rect = skia::Rect::from_xywh(gx, gy, glyph_w, glyph_h);

        // Outer glyph frame stroke (neutral medium gray)
        let mut glyph_frame = skia::Paint::default();
        glyph_frame.set_color4f(skia::Color4f::new(0.48, 0.48, 0.52, alpha), None);
        glyph_frame.set_style(skia::PaintStyle::Stroke);
        glyph_frame.set_stroke_width((glyph_w * 0.07).clamp(1.5, 2.5));
        glyph_frame.set_anti_alias(true);
        canvas.draw_round_rect(glyph_rect, 3.0, 3.0, &glyph_frame);

        // Sun circle (monochrome neutral gray)
        let sun_r = glyph_w * 0.10;
        let sun_x = gx + glyph_w * 0.72;
        let sun_y = gy + glyph_h * 0.32;
        let mut sun_paint = skia::Paint::default();
        sun_paint.set_color4f(skia::Color4f::new(0.48, 0.48, 0.52, alpha), None);
        sun_paint.set_anti_alias(true);
        canvas.draw_circle(skia::Point::new(sun_x, sun_y), sun_r, &sun_paint);

        // Mountain peaks (monochrome neutral gray)
        let mut mount = skia::PathBuilder::new();
        mount.move_to(skia::Point::new(gx + 1.5, gy + glyph_h - 1.5));
        mount.line_to(skia::Point::new(gx + glyph_w * 0.38, gy + glyph_h * 0.42));
        mount.line_to(skia::Point::new(gx + glyph_w * 0.60, gy + glyph_h * 0.68));
        mount.line_to(skia::Point::new(gx + glyph_w * 0.76, gy + glyph_h * 0.50));
        mount.line_to(skia::Point::new(gx + glyph_w - 1.5, gy + glyph_h - 1.5));
        mount.close();

        let mut mount_paint = skia::Paint::default();
        mount_paint.set_color4f(skia::Color4f::new(0.55, 0.55, 0.60, alpha), None);
        mount_paint.set_anti_alias(true);
        canvas.draw_path(&mount.detach(), &mount_paint);

        // 5. Outer border (neutral medium-light gray)
        let mut border_paint = skia::Paint::default();
        border_paint.set_color4f(skia::Color4f::new(0.72, 0.72, 0.76, alpha), None);
        border_paint.set_style(skia::PaintStyle::Stroke);
        border_paint.set_stroke_width(1.5);
        border_paint.set_anti_alias(true);
        canvas.draw_rect(dst, &border_paint);
    }
}

pub fn concat_color_matrices(a: &[f32; 20], b: &[f32; 20]) -> [f32; 20] {
    let mut out = [0.0f32; 20];
    for row in 0..4 {
        for col in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[row * 5 + k] * b[k * 5 + col];
            }
            out[row * 5 + col] = sum;
        }
        let mut t = a[row * 5 + 4];
        for k in 0..4 {
            t += a[row * 5 + k] * b[k * 5 + 4];
        }
        out[row * 5 + 4] = t;
    }
    out
}

/// Generates a clean, monochrome light-gray raster placeholder graphic.
#[allow(dead_code)]
pub fn generate_placeholder_image(width: u32, height: u32) -> Vec<u8> {
    let w = width.clamp(64, 2048) as i32;
    let h = height.clamp(64, 2048) as i32;
    if let Some(mut surface) = skia::surfaces::raster_n32_premul((w, h)) {
        let canvas = surface.canvas();

        // 1. Light Gray Background (#ECECEF)
        let mut bg_paint = skia::Paint::default();
        bg_paint.set_color4f(skia::Color4f::new(0.925, 0.925, 0.935, 1.0), None);
        bg_paint.set_anti_alias(true);
        canvas.draw_rect(
            skia::Rect::from_xywh(0.0, 0.0, w as f32, h as f32),
            &bg_paint,
        );

        // 2. Diagonal frame guide lines
        let mut diag_paint = skia::Paint::default();
        diag_paint.set_color4f(skia::Color4f::new(0.82, 0.82, 0.85, 0.6), None);
        diag_paint.set_stroke_width(1.0);
        diag_paint.set_anti_alias(true);
        canvas.draw_line(
            skia::Point::new(0.0, 0.0),
            skia::Point::new(w as f32, h as f32),
            &diag_paint,
        );
        canvas.draw_line(
            skia::Point::new(w as f32, 0.0),
            skia::Point::new(0.0, h as f32),
            &diag_paint,
        );

        // 3. Center Circular Backdrop
        let cx = w as f32 * 0.5;
        let cy = h as f32 * 0.5;
        let min_dim = (w as f32).min(h as f32);
        let circle_r = (min_dim * 0.25).clamp(16.0, 60.0);

        let mut circle_bg = skia::Paint::default();
        circle_bg.set_color4f(skia::Color4f::new(0.85, 0.85, 0.87, 1.0), None);
        circle_bg.set_anti_alias(true);
        canvas.draw_circle(skia::Point::new(cx, cy), circle_r, &circle_bg);

        // 4. Monochrome Vector Photo Frame Glyph (No colors!)
        let glyph_w = (circle_r * 1.12).clamp(18.0, 52.0);
        let glyph_h = glyph_w * 0.72;
        let gx = cx - glyph_w * 0.5;
        let gy = cy - glyph_h * 0.5;
        let glyph_rect = skia::Rect::from_xywh(gx, gy, glyph_w, glyph_h);

        let mut glyph_frame = skia::Paint::default();
        glyph_frame.set_color4f(skia::Color4f::new(0.48, 0.48, 0.52, 1.0), None);
        glyph_frame.set_style(skia::PaintStyle::Stroke);
        glyph_frame.set_stroke_width((glyph_w * 0.07).clamp(1.5, 2.5));
        glyph_frame.set_anti_alias(true);
        canvas.draw_round_rect(glyph_rect, 3.0, 3.0, &glyph_frame);

        let sun_r = glyph_w * 0.10;
        let sun_x = gx + glyph_w * 0.72;
        let sun_y = gy + glyph_h * 0.32;
        let mut sun_paint = skia::Paint::default();
        sun_paint.set_color4f(skia::Color4f::new(0.48, 0.48, 0.52, 1.0), None);
        sun_paint.set_anti_alias(true);
        canvas.draw_circle(skia::Point::new(sun_x, sun_y), sun_r, &sun_paint);

        let mut mount = skia::PathBuilder::new();
        mount.move_to(skia::Point::new(gx + 1.5, gy + glyph_h - 1.5));
        mount.line_to(skia::Point::new(gx + glyph_w * 0.38, gy + glyph_h * 0.42));
        mount.line_to(skia::Point::new(gx + glyph_w * 0.60, gy + glyph_h * 0.68));
        mount.line_to(skia::Point::new(gx + glyph_w * 0.76, gy + glyph_h * 0.50));
        mount.line_to(skia::Point::new(gx + glyph_w - 1.5, gy + glyph_h - 1.5));
        mount.close();

        let mut mount_paint = skia::Paint::default();
        mount_paint.set_color4f(skia::Color4f::new(0.55, 0.55, 0.60, 1.0), None);
        mount_paint.set_anti_alias(true);
        canvas.draw_path(&mount.detach(), &mount_paint);

        // 5. Border
        let mut border_paint = skia::Paint::default();
        border_paint.set_color4f(skia::Color4f::new(0.72, 0.72, 0.76, 1.0), None);
        border_paint.set_style(skia::PaintStyle::Stroke);
        border_paint.set_stroke_width(1.5);
        border_paint.set_anti_alias(true);
        canvas.draw_rect(
            skia::Rect::from_xywh(1.0, 1.0, w as f32 - 2.0, h as f32 - 2.0),
            &border_paint,
        );

        let image = surface.image_snapshot();
        if let Some(data) = image.encode(None, skia::EncodedImageFormat::PNG, None) {
            return data.as_bytes().to_vec();
        }
    }
    Vec::new()
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CloneElement {
    pub id: ElementId,
    pub source_id: ElementId,
    pub offset: Point,
    pub scale: Point,
    pub rotation: f32,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur_radius: f32,
    pub visible: bool,
    pub locked: bool,
    pub name: Option<String>,
    #[serde(default)]
    pub cached_bounds: Option<Rect>,
}

impl CloneElement {
    pub fn new(source_id: ElementId, offset: Point) -> Self {
        Self {
            id: ElementId::new(),
            source_id,
            offset,
            scale: Point::new(1.0, 1.0),
            rotation: 0.0,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            blur_radius: 0.0,
            visible: true,
            locked: false,
            name: None,
            cached_bounds: None,
        }
    }

    pub fn bounds(&self) -> Rect {
        if let Some(b) = self.cached_bounds {
            b
        } else {
            Rect::new(self.offset.x, self.offset.y, 0.0, 0.0)
        }
    }

    pub fn hit_test(&self, p: Point) -> bool {
        self.bounds().contains(p)
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.offset.x += dx;
        self.offset.y += dy;
        if let Some(ref mut b) = self.cached_bounds {
            b.x += dx;
            b.y += dy;
        }
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.offset.x = origin.x + (self.offset.x - origin.x) * sx;
        self.offset.y = origin.y + (self.offset.y - origin.y) * sy;
        self.scale.x *= sx;
        self.scale.y *= sy;
        if let Some(ref mut b) = self.cached_bounds {
            b.x = origin.x + (b.x - origin.x) * sx;
            b.y = origin.y + (b.y - origin.y) * sy;
            b.width *= sx.abs();
            b.height *= sy.abs();
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        let dx = self.offset.x - center.x;
        let dy = self.offset.y - center.y;
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        self.offset.x = center.x + (dx * cos_a - dy * sin_a);
        self.offset.y = center.y + (dx * sin_a + dy * cos_a);
        self.rotation += angle_rad;
        if let Some(ref mut b) = self.cached_bounds {
            let cx = b.x + b.width * 0.5;
            let cy = b.y + b.height * 0.5;
            let bx = center.x + ((cx - center.x) * cos_a - (cy - center.y) * sin_a);
            let by = center.y + ((cx - center.x) * sin_a + (cy - center.y) * cos_a);
            b.x = bx - b.width * 0.5;
            b.y = by - b.height * 0.5;
        }
    }

    pub fn render(&self, _canvas: &skia::Canvas) {
        // Rendered via document/renderer with master lookup
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_element_adjustments_and_color_matrix() {
        let mut img = ImageElement::new_placeholder(
            Rect::new(0.0, 0.0, 300.0, 200.0),
            Some("Test Image".to_string()),
        );
        assert!(!img.has_active_adjustments());

        img.brightness = 0.2;
        img.contrast = 1.2;
        img.saturation = 1.5;
        img.hue_rotate = 45.0;
        img.invert = true;
        assert!(img.has_active_adjustments());

        let mat = img.color_matrix();
        assert_eq!(mat.len(), 20);
        // Verify that the diagonal elements are modified by invert & adjustments
        assert!(mat[0] != 1.0);

        // Test pure invert matrix
        let mut inv_img = ImageElement::new_placeholder(Rect::new(0.0, 0.0, 100.0, 100.0), None);
        inv_img.invert = true;
        let inv_mat = inv_img.color_matrix();
        assert_eq!(inv_mat[0], -1.0);
        assert_eq!(inv_mat[4], 1.0); // Offset is 1.0 (normalized float), not 255.0

        // Test brightness & contrast offsets are normalized
        let mut bc_img = ImageElement::new_placeholder(Rect::new(0.0, 0.0, 100.0, 100.0), None);
        bc_img.contrast = 1.1;
        bc_img.brightness = 0.05;
        let bc_mat = bc_img.color_matrix();
        assert_eq!(bc_mat[0], 1.1);
        let expected_offset = 0.5 * (1.0 - 1.1) + 0.05; // 0.0
        assert!((bc_mat[4] - expected_offset).abs() < 1e-5);
    }

    #[test]
    fn test_adwaita_placeholder_generation() {
        let bytes = generate_placeholder_image(320, 240);
        assert!(!bytes.is_empty());
        // Verify PNG magic header
        assert_eq!(
            &bytes[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }
}
