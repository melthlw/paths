use skia_safe as skia;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::core::color::Color;
use crate::core::geometry::{Point, Rect};
use super::style::BlendMode;
use super::ElementId;

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
    pub box_width: Option<f32>,
    pub box_height: Option<f32>,
    pub alignment: TextAlign,
    pub color: Color,
    pub name: Option<String>,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub blur: f32,
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
            box_width: None,
            box_height: None,
            alignment: TextAlign::Left,
            color,
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
            box_width: Some(width.max(20.0)),
            box_height: Some(height.max(font_size * 1.2)),
            alignment: TextAlign::Left,
            color,
            name: None,
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::default(),
            blur: 0.0,
        }
    }

    pub fn create_skia_font(&self) -> skia::Font {
        let typeface = get_cached_typeface(&self.font_family, self.font_weight);
        let mut font = skia::Font::default();
        font.set_typeface(typeface);
        font.set_size(self.font_size);
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
            (char_count - 1) as f32 * self.letter_spacing
        } else {
            0.0
        };
        let word_extra = space_count as f32 * self.word_spacing;
        (base_w + letter_extra + word_extra).max(0.0)
    }

    pub fn layout_lines(&self) -> Vec<String> {
        let font = self.create_skia_font();
        let paragraphs = self.text.split('\n');
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

    pub fn render(&self, canvas: &skia::Canvas) {
        if self.text.is_empty() {
            return;
        }

        let font = self.create_skia_font();

        let mut paint = skia::Paint::default();
        paint.set_color4f(self.color.to_skia(), None);
        paint.set_style(skia::PaintStyle::Fill);
        paint.set_anti_alias(true);

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

            if self.letter_spacing == 0.0 && self.word_spacing == 0.0 {
                let line_pos = skia::Point::new(self.position.x + offset_x, baseline_y);
                canvas.draw_str(line, line_pos, &font, &paint);
            } else {
                let mut cur_x = self.position.x + offset_x;
                for ch in line.chars() {
                    let s = ch.to_string();
                    let (cw, _) = font.measure_str(&s, None);
                    let p = skia::Point::new(cur_x, baseline_y);
                    canvas.draw_str(&s, p, &font, &paint);
                    let extra = if ch == ' ' {
                        self.letter_spacing + self.word_spacing
                    } else {
                        self.letter_spacing
                    };
                    cur_x += cw + extra;
                }
            }
        }
    }
}
