use skia_safe as skia;
use std::cell::RefCell;
use std::collections::HashMap;

use super::brush::StrokeJoin;
use super::style::{BlendMode, StrokeLayer, StrokeStyle};
use super::ElementId;
use crate::core::color::Color;
use crate::core::geometry::{Point, Rect};

thread_local! {
    static FONT_MGR: skia::FontMgr = skia::FontMgr::new();
    static TYPEFACE_CACHE: RefCell<HashMap<(String, u32), skia::Typeface>> = RefCell::new(HashMap::new());
    static SYSTEM_FONTS_CACHE: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

pub fn get_cached_typeface(family: &str, weight: u32) -> skia::Typeface {
    let key = (family.to_string(), weight);
    TYPEFACE_CACHE.with(|cache_ref| {
        let mut cache = cache_ref.borrow_mut();
        if let Some(tf) = cache.get(&key) {
            return tf.clone();
        }

        let tf = FONT_MGR.with(|font_mgr| {
            let font_weight = skia::font_style::Weight::from(weight as i32);
            let font_style = skia::FontStyle::new(
                font_weight,
                skia::font_style::Width::NORMAL,
                skia::font_style::Slant::Upright,
            );
            font_mgr
                .match_family_style(family, font_style)
                .or_else(|| font_mgr.legacy_make_typeface(None, font_style))
                .unwrap_or_else(|| skia::Font::default().typeface())
        });

        cache.insert(key, tf.clone());
        tf
    })
}

pub fn get_system_font_families() -> Vec<String> {
    SYSTEM_FONTS_CACHE.with(|cache_ref| {
        let mut cache = cache_ref.borrow_mut();
        if let Some(ref list) = *cache {
            return list.clone();
        }

        let list = FONT_MGR.with(|font_mgr| {
            let count = font_mgr.count_families();
            let mut families = Vec::with_capacity(count);
            for i in 0..count {
                let name = font_mgr.family_name(i);
                if !name.is_empty() && !families.contains(&name) {
                    families.push(name);
                }
            }
            families.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            if families.is_empty() {
                families = vec![
                    "Cantarell".to_string(),
                    "Inter".to_string(),
                    "Sans".to_string(),
                    "Serif".to_string(),
                    "Monospace".to_string(),
                    "Roboto".to_string(),
                    "Ubuntu".to_string(),
                    "DejaVu Sans".to_string(),
                    "Fira Code".to_string(),
                ];
            }
            families
        });

        *cache = Some(list.clone());
        list
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

impl Default for TextAlign {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OpenTypeFeatures {
    pub ligatures: bool,
    pub contextual_alt: bool,
    pub stylistic_alts: bool,
    pub fractions: bool,
    pub tabular_numerals: bool,
    pub kerning: bool,
}

impl Default for OpenTypeFeatures {
    fn default() -> Self {
        Self {
            ligatures: true,
            contextual_alt: true,
            stylistic_alts: false,
            fractions: false,
            tabular_numerals: false,
            kerning: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum TextCase {
    #[default]
    Normal,
    Uppercase,
    Lowercase,
    TitleCase,
    SmallCaps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum TextBaseline {
    #[default]
    Normal,
    Subscript,
    Superscript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum PathVerticalAlign {
    #[default]
    Baseline,
    Ascender,
    Descender,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum PathGlyphOrientation {
    #[default]
    Tangent,
    Upright,
    Perpendicular,
}

/// Text Vector Element
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextElement {
    pub id: ElementId,
    pub position: Point,
    pub text: String,
    pub font_family: String,
    pub font_weight: u32,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub kerning_offset: f32,
    pub opentype_features: OpenTypeFeatures,
    pub text_case: TextCase,
    #[serde(default)]
    pub underline: bool,
    #[serde(default)]
    pub strikethrough: bool,
    #[serde(default)]
    pub baseline: TextBaseline,
    #[serde(default = "default_width_scale")]
    pub font_width_scale: f32,
    pub box_width: Option<f32>,
    pub box_height: Option<f32>,
    pub alignment: TextAlign,
    pub color: Color,
    #[serde(default)]
    pub stroke_color: Option<Color>,
    #[serde(default = "default_stroke_width")]
    pub stroke_width: f32,
    #[serde(default)]
    pub stroke_style: StrokeStyle,
    #[serde(default)]
    pub stroke_join: StrokeJoin,
    #[serde(default)]
    pub strokes: Vec<StrokeLayer>,
    #[serde(default)]
    pub path_id: Option<ElementId>,
    #[serde(default)]
    pub path_offset: f32,
    #[serde(default)]
    pub path_side_inverted: bool,
    #[serde(default = "default_true")]
    pub path_align_orientation: bool,
    #[serde(default)]
    pub path_repeat: bool,
    #[serde(default = "default_path_spacing")]
    pub path_spacing: f32,
    #[serde(default)]
    pub path_valign: PathVerticalAlign,
    #[serde(default)]
    pub path_glyph_orientation: PathGlyphOrientation,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
}

fn default_stroke_width() -> f32 {
    1.0
}
fn default_true() -> bool {
    true
}
fn default_path_spacing() -> f32 {
    20.0
}
fn default_width_scale() -> f32 {
    1.0
}

impl TextElement {
    pub fn new(position: Point, text: String, font_size: f32, color: Color) -> Self {
        Self {
            id: ElementId::new(),
            position,
            text,
            font_family: "Cantarell".to_string(),
            font_weight: 400,
            font_size: font_size.max(8.0),
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            kerning_offset: 0.0,
            opentype_features: OpenTypeFeatures::default(),
            text_case: TextCase::default(),
            underline: false,
            strikethrough: false,
            baseline: TextBaseline::default(),
            font_width_scale: 1.0,
            box_width: None,
            box_height: None,
            alignment: TextAlign::Left,
            color,
            stroke_color: None,
            stroke_width: 1.0,
            stroke_style: StrokeStyle::Solid,
            stroke_join: StrokeJoin::Round,
            strokes: Vec::new(),
            path_id: None,
            path_offset: 0.0,
            path_side_inverted: false,
            path_align_orientation: true,
            path_repeat: false,
            path_spacing: 20.0,
            path_valign: PathVerticalAlign::default(),
            path_glyph_orientation: PathGlyphOrientation::default(),
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
        }
    }

    pub fn with_box(
        position: Point,
        text: String,
        font_size: f32,
        color: Color,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            id: ElementId::new(),
            position,
            text,
            font_family: "Cantarell".to_string(),
            font_weight: 400,
            font_size: font_size.max(8.0),
            line_height: 1.2,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            kerning_offset: 0.0,
            opentype_features: OpenTypeFeatures::default(),
            text_case: TextCase::default(),
            underline: false,
            strikethrough: false,
            baseline: TextBaseline::default(),
            font_width_scale: 1.0,
            box_width: Some(width.max(20.0)),
            box_height: Some(height.max(font_size * 1.2)),
            alignment: TextAlign::Left,
            color,
            stroke_color: None,
            stroke_width: 1.0,
            stroke_style: StrokeStyle::Solid,
            stroke_join: StrokeJoin::Round,
            strokes: Vec::new(),
            path_id: None,
            path_offset: 0.0,
            path_side_inverted: false,
            path_align_orientation: true,
            path_repeat: false,
            path_spacing: 20.0,
            path_valign: PathVerticalAlign::default(),
            path_glyph_orientation: PathGlyphOrientation::default(),
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
        }
    }

    pub fn formatted_text(&self) -> String {
        match self.text_case {
            TextCase::Normal => self.text.clone(),
            TextCase::Uppercase | TextCase::SmallCaps => self.text.to_uppercase(),
            TextCase::Lowercase => self.text.to_lowercase(),
            TextCase::TitleCase => {
                let c_iter = self.text.chars();
                let mut res = String::with_capacity(self.text.len());
                let mut capitalize = true;
                for ch in c_iter {
                    if ch.is_whitespace() {
                        capitalize = true;
                        res.push(ch);
                    } else if capitalize {
                        res.extend(ch.to_uppercase());
                        capitalize = false;
                    } else {
                        res.extend(ch.to_lowercase());
                    }
                }
                res
            }
        }
    }

    pub fn create_skia_font(&self) -> skia::Font {
        let typeface = get_cached_typeface(&self.font_family, self.font_weight);
        let mut font = skia::Font::default();
        font.set_typeface(typeface);
        let eff_size = if self.text_case == TextCase::SmallCaps {
            self.font_size * 0.85
        } else {
            self.font_size
        };
        font.set_size(eff_size);
        font
    }

    pub fn measure_text_line(&self, font: &skia::Font, s: &str) -> f32 {
        if s.is_empty() {
            return 0.0;
        }
        let (base_w, _) = font.measure_str(s, None);
        let char_count = s.chars().count();
        let space_count = s.chars().filter(|&c| c == ' ').count();
        let letter_extra = if char_count > 1 {
            (char_count - 1) as f32 * (self.letter_spacing + self.kerning_offset)
        } else {
            0.0
        };
        let word_extra = space_count as f32 * self.word_spacing;
        (base_w + letter_extra + word_extra).max(0.0)
    }

    pub fn layout_lines(&self) -> Vec<String> {
        let font = self.create_skia_font();
        let formatted = self.formatted_text();
        let paragraphs = formatted.split('\n');
        let mut lines = Vec::new();

        if let Some(max_w) = self.box_width {
            if max_w > 5.0 {
                for para in paragraphs {
                    if para.is_empty() {
                        lines.push(String::new());
                        continue;
                    }
                    let words = para.split(' ');
                    let mut current_line = String::new();
                    for word in words {
                        let candidate = if current_line.is_empty() {
                            word.to_string()
                        } else {
                            format!("{} {}", current_line, word)
                        };
                        let w = self.measure_text_line(&font, &candidate);
                        if w <= max_w {
                            current_line = candidate;
                        } else {
                            if !current_line.is_empty() {
                                lines.push(current_line);
                            }
                            let word_w = self.measure_text_line(&font, word);
                            if word_w <= max_w {
                                current_line = word.to_string();
                            } else {
                                let mut char_line = String::new();
                                for ch in word.chars() {
                                    let mut next_cand = char_line.clone();
                                    next_cand.push(ch);
                                    let cw = self.measure_text_line(&font, &next_cand);
                                    if cw <= max_w || char_line.is_empty() {
                                        char_line = next_cand;
                                    } else {
                                        lines.push(char_line);
                                        char_line = ch.to_string();
                                    }
                                }
                                current_line = char_line;
                            }
                        }
                    }
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }
                }
                if lines.is_empty() {
                    lines.push(String::new());
                }
                return lines;
            }
        }

        paragraphs.map(|s| s.to_string()).collect()
    }

    pub fn bounds(&self) -> Rect {
        let font = self.create_skia_font();
        let lines = self.layout_lines();

        let mut max_width: f32 = 0.0;
        for line in &lines {
            let w = self.measure_text_line(&font, line);
            if w > max_width {
                max_width = w;
            }
        }

        let line_spacing = self.font_size * self.line_height.max(0.5);
        let total_height = (lines.len() as f32) * line_spacing;

        let width = if let Some(bw) = self.box_width {
            bw.max(20.0)
        } else {
            max_width.max(20.0)
        };

        let height = if let Some(bh) = self.box_height {
            bh.max(total_height).max(self.font_size * 1.2)
        } else {
            total_height.max(self.font_size * 1.2)
        };

        Rect::new(
            self.position.x,
            self.position.y - self.font_size,
            width,
            height,
        )
    }

    pub fn hit_test(&self, p: Point) -> bool {
        self.bounds().expand(6.0).contains(p)
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.position.x += dx;
        self.position.y += dy;
    }

    pub fn scale(&mut self, origin: Point, sx: f32, sy: f32) {
        self.position.x = origin.x + (self.position.x - origin.x) * sx;
        self.position.y = origin.y + (self.position.y - origin.y) * sy;
        self.font_size = (self.font_size * sy.abs()).max(6.0);
        if let Some(w) = self.box_width {
            self.box_width = Some((w * sx.abs()).max(20.0));
        }
        if let Some(h) = self.box_height {
            self.box_height = Some((h * sy.abs()).max(self.font_size * 1.2));
        }
    }

    pub fn rotate(&mut self, center: Point, angle_rad: f32) {
        self.position = crate::core::geometry::rotate_point(self.position, center, angle_rad);
    }

    pub fn build_paints(&self) -> (Vec<skia::Paint>, skia::Paint) {
        let mut stroke_paints = Vec::new();

        // 1. Independent Stroke Layers or single stroke_color
        if !self.strokes.is_empty() {
            for layer in &self.strokes {
                if layer.enabled && layer.color.a > 0.001 && layer.width > 0.1 {
                    let mut p = skia::Paint::default();
                    p.set_color4f(layer.color.with_alpha(layer.color.a * layer.opacity).to_skia(), None);
                    p.set_style(skia::PaintStyle::Stroke);
                    p.set_stroke_width(layer.width);
                    p.set_stroke_join(self.stroke_join.to_skia());
                    p.set_anti_alias(true);
                    match layer.style {
                        StrokeStyle::Dashed => {
                            p.set_path_effect(skia::dash_path_effect::new(&[layer.width * 3.0, layer.width * 2.0], 0.0));
                        }
                        StrokeStyle::Dotted => {
                            p.set_path_effect(skia::dash_path_effect::new(&[layer.width, layer.width * 1.5], 0.0));
                        }
                        StrokeStyle::Solid => {}
                    }
                    stroke_paints.push(p);
                }
            }
        } else if let Some(sc) = self.stroke_color {
            if sc.a > 0.001 && self.stroke_width > 0.1 {
                let mut p = skia::Paint::default();
                p.set_color4f(sc.to_skia(), None);
                p.set_style(skia::PaintStyle::Stroke);
                p.set_stroke_width(self.stroke_width);
                p.set_stroke_join(self.stroke_join.to_skia());
                p.set_anti_alias(true);
                match self.stroke_style {
                    StrokeStyle::Dashed => {
                        p.set_path_effect(skia::dash_path_effect::new(&[self.stroke_width * 3.0, self.stroke_width * 2.0], 0.0));
                    }
                    StrokeStyle::Dotted => {
                        p.set_path_effect(skia::dash_path_effect::new(&[self.stroke_width, self.stroke_width * 1.5], 0.0));
                    }
                    StrokeStyle::Solid => {}
                }
                stroke_paints.push(p);
            }
        }

        // 2. Fill Paint
        let mut fill_paint = skia::Paint::default();
        fill_paint.set_color4f(self.color.to_skia(), None);
        fill_paint.set_style(skia::PaintStyle::Fill);
        fill_paint.set_anti_alias(true);

        (stroke_paints, fill_paint)
    }

    pub fn render(&self, canvas: &skia::Canvas) {
        self.render_standard(canvas);
    }

    pub fn render_with_doc(&self, canvas: &skia::Canvas, doc: Option<&crate::core::Document>) {
        if let (Some(pid), Some(document)) = (self.path_id, doc) {
            if let Some(target_elem) = document.find_element(pid) {
                let path = target_elem.to_skia_path();
                if !path.is_empty() {
                    self.render_on_path(canvas, &path);
                    return;
                }
            }
        }
        self.render_standard(canvas);
    }

    pub fn render_standard(&self, canvas: &skia::Canvas) {
        if self.text.is_empty() {
            return;
        }

        let font = self.create_skia_font();
        let (stroke_paints, fill_paint) = self.build_paints();

        let lines = self.layout_lines();
        let line_spacing = self.font_size * self.line_height.max(0.5);

        let bounds_w = if let Some(bw) = self.box_width {
            bw
        } else {
            let mut max_w: f32 = 0.0;
            for line in &lines {
                let w = self.measure_text_line(&font, line);
                if w > max_w {
                    max_w = w;
                }
            }
            max_w
        };

        for (i, line) in lines.iter().enumerate() {
            let line_w = self.measure_text_line(&font, line);
            let offset_x = match self.alignment {
                TextAlign::Left | TextAlign::Justify => 0.0,
                TextAlign::Center => (bounds_w - line_w).max(0.0) / 2.0,
                TextAlign::Right => (bounds_w - line_w).max(0.0),
            };

            let baseline_y = self.position.y + (i as f32) * line_spacing;

            let render_line = |c: &skia::Canvas, p: &skia::Paint| {
                if self.letter_spacing == 0.0 && self.word_spacing == 0.0 && self.kerning_offset == 0.0 {
                    let line_pos = skia::Point::new(self.position.x + offset_x, baseline_y);
                    c.draw_str(line, line_pos, &font, p);
                } else {
                    let mut cur_x = self.position.x + offset_x;
                    for ch in line.chars() {
                        let s = ch.to_string();
                        let (cw, _) = font.measure_str(&s, None);
                        let pos = skia::Point::new(cur_x, baseline_y);
                        c.draw_str(&s, pos, &font, p);
                        let extra = if ch == ' ' {
                            self.letter_spacing + self.word_spacing + self.kerning_offset
                        } else {
                            self.letter_spacing + self.kerning_offset
                        };
                        cur_x += cw + extra;
                    }
                }
            };

            for sp in &stroke_paints {
                render_line(canvas, sp);
            }
            render_line(canvas, &fill_paint);

            if self.underline || self.strikethrough {
                let mut dec_paint = skia::Paint::default();
                dec_paint.set_color4f(self.color.to_skia(), None);
                dec_paint.set_style(skia::PaintStyle::Stroke);
                dec_paint.set_stroke_width((self.font_size * 0.07).max(1.0));
                dec_paint.set_anti_alias(true);

                let start_x = self.position.x + offset_x;
                let end_x = start_x + line_w;

                if self.underline {
                    let u_y = baseline_y + self.font_size * 0.15;
                    canvas.draw_line(
                        skia::Point::new(start_x, u_y),
                        skia::Point::new(end_x, u_y),
                        &dec_paint,
                    );
                }
                if self.strikethrough {
                    let s_y = baseline_y - self.font_size * 0.30;
                    canvas.draw_line(
                        skia::Point::new(start_x, s_y),
                        skia::Point::new(end_x, s_y),
                        &dec_paint,
                    );
                }
            }
        }
    }

    pub fn render_on_path(&self, canvas: &skia::Canvas, path: &skia::Path) {
        if self.text.is_empty() {
            return;
        }

        let font = self.create_skia_font();
        let (stroke_paints, fill_paint) = self.build_paints();
        let mut measure = skia::PathMeasure::new(path, false, None);
        let path_len = measure.length();
        if path_len < 1.0 {
            self.render_standard(canvas);
            return;
        }

        let formatted = self.formatted_text();
        let char_vec: Vec<char> = formatted.chars().filter(|&c| c != '\n').collect();
        if char_vec.is_empty() {
            return;
        }

        let mut current_dist = self.path_offset;
        let spacing = self.path_spacing.max(1.0);

        loop {
            let start_dist = current_dist;
            for &ch in &char_vec {
                let s = ch.to_string();
                let (cw, _) = font.measure_str(&s, None);
                let advance = cw + self.letter_spacing + self.kerning_offset + if ch == ' ' { self.word_spacing } else { 0.0 };

                let char_mid_dist = current_dist + cw * 0.5;
                let sample_dist = if self.path_repeat {
                    char_mid_dist.rem_euclid(path_len)
                } else {
                    char_mid_dist
                };

                if sample_dist >= 0.0 && sample_dist <= path_len {
                    if let Some((pos, tan)) = measure.pos_tan(sample_dist) {
                        let mut angle_rad = tan.y.atan2(tan.x);
                        if self.path_side_inverted {
                            angle_rad += std::f32::consts::PI;
                        }

                        let valign_y_offset = match self.path_valign {
                            PathVerticalAlign::Baseline => 0.0,
                            PathVerticalAlign::Ascender => self.font_size * 0.75,
                            PathVerticalAlign::Descender => -self.font_size * 0.25,
                            PathVerticalAlign::Center => self.font_size * 0.35,
                        };

                        let angle_degrees = match self.path_glyph_orientation {
                            PathGlyphOrientation::Tangent => {
                                if self.path_align_orientation {
                                    angle_rad.to_degrees()
                                } else {
                                    0.0
                                }
                            }
                            PathGlyphOrientation::Upright => 0.0,
                            PathGlyphOrientation::Perpendicular => {
                                (angle_rad + std::f32::consts::FRAC_PI_2).to_degrees()
                            }
                        };

                        let render_char = |c: &skia::Canvas, p: &skia::Paint| {
                            c.save();
                            c.translate(pos);
                            c.rotate(angle_degrees, None);
                            let char_pos = skia::Point::new(-cw * 0.5, valign_y_offset);
                            c.draw_str(&s, char_pos, &font, p);
                            c.restore();
                        };

                        for sp in &stroke_paints {
                            render_char(canvas, sp);
                        }
                        render_char(canvas, &fill_paint);
                    }
                }

                current_dist += advance;
            }

            if !self.path_repeat || (current_dist - start_dist) < 0.1 || current_dist >= path_len * 2.0 {
                break;
            }

            current_dist += spacing;
        }
    }
}

pub fn get_curated_font_glyphs() -> Vec<(&'static str, Vec<char>)> {
    vec![
        (
            crate::core::gettext("Currency & Symbols").leak(),
            vec!['$', '€', '£', '¥', '₹', '₽', '₩', '₺', '₴', '¢', '¤', '©', '®', '™', '§', '¶', '†', '‡', '•', '°'],
        ),
        (
            crate::core::gettext("Math & Logic").leak(),
            vec!['+', '−', '±', '×', '÷', '=', '≠', '≈', '≤', '≥', '∞', '∑', '∏', '√', '∫', '∆', 'π', 'µ', '‰', '‱'],
        ),
        (
            crate::core::gettext("Arrows & Pointers").leak(),
            vec!['←', '↑', '→', '↓', '↔', '↕', '↖', '↗', '↘', '↙', '⇐', '⇑', '⇒', '⇓', '⇔', '➔', '⇦', '⇧', '⇨', '⇩'],
        ),
        (
            crate::core::gettext("Punctuation & Quotes").leak(),
            vec!['«', '»', '“', '”', '‘', '’', '„', '‚', '—', '–', '…', '¿', '¡', '‹', '›', '·', '‽', 'ª', 'º', '№'],
        ),
        (
            crate::core::gettext("Ligatures & Special").leak(),
            vec!['ﬁ', 'ﬂ', 'æ', 'œ', 'ß', 'Þ', 'ð', 'Ø', 'ø', 'Å', 'å', 'Æ', 'Œ', 'µ', 'ª', 'º', '¿', '¡', '©', '®'],
        ),
        (
            crate::core::gettext("Shapes & Icons").leak(),
            vec!['★', '☆', '✦', '✧', '♠', '♣', '♥', '♦', '▲', '▼', '◄', '►', '■', '□', '●', '○', '◆', '◇', '⚙', '✏'],
        ),
    ]
}

