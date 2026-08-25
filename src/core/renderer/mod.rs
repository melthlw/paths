pub mod draw;
pub mod export;

pub use draw::{
    draw_alignment_grid, draw_corner_origin_drag, draw_element_node, draw_infinite_dot_grid,
    draw_modifier_canvas_overlays, draw_pages, draw_rulers, draw_selection_highlight,
    draw_snap_guides, draw_user_guides,
};

pub use export::{
    ExportConfig, ExportFormat, ExportScope, export_document,
};

use cairo::{Format, ImageSurface};
use skia_safe::{self as skia, Color4f, surfaces};
use std::cell::RefCell;

use crate::core::color::Color;
use crate::core::document::Document;
use crate::core::geometry::{Point, Viewport};
use crate::core::grid::GridConfig;
use crate::core::ruler::{Guide, RulerConfig};
use crate::core::snap::SnapGuide;

pub const ARTBOARD_WIDTH: f32 = 794.0;
pub const ARTBOARD_HEIGHT: f32 = 1123.0;

thread_local! {
    static RENDERER_FONT_MGR: skia::FontMgr = skia::FontMgr::new();
    static RENDERER_TYPEFACE: RefCell<Option<skia::Typeface>> = const { RefCell::new(None) };
    static RENDERER_BOLD_TYPEFACE: RefCell<Option<skia::Typeface>> = const { RefCell::new(None) };
}

pub fn get_ui_typeface() -> skia::Typeface {
    RENDERER_TYPEFACE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref tf) = *opt {
            return tf.clone();
        }
        let tf = RENDERER_FONT_MGR.with(|mgr| {
            mgr.match_family_style("Cantarell", skia::FontStyle::normal())
                .or_else(|| mgr.match_family_style("Inter", skia::FontStyle::normal()))
                .or_else(|| mgr.match_family_style("Adwaita Sans", skia::FontStyle::normal()))
                .or_else(|| mgr.match_family_style("DejaVu Sans", skia::FontStyle::normal()))
                .or_else(|| mgr.match_family_style("Sans", skia::FontStyle::normal()))
                .or_else(|| mgr.legacy_make_typeface(None, skia::FontStyle::normal()))
                .unwrap_or_else(|| skia::Font::default().typeface())
        });
        *opt = Some(tf.clone());
        tf
    })
}

pub fn get_ui_bold_typeface() -> skia::Typeface {
    RENDERER_BOLD_TYPEFACE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if let Some(ref tf) = *opt {
            return tf.clone();
        }
        let tf = RENDERER_FONT_MGR.with(|mgr| {
            mgr.match_family_style("Cantarell", skia::FontStyle::bold())
                .or_else(|| mgr.match_family_style("Inter", skia::FontStyle::bold()))
                .or_else(|| mgr.match_family_style("Adwaita Sans", skia::FontStyle::bold()))
                .or_else(|| mgr.match_family_style("DejaVu Sans", skia::FontStyle::bold()))
                .or_else(|| mgr.match_family_style("Sans", skia::FontStyle::bold()))
                .or_else(|| mgr.legacy_make_typeface(None, skia::FontStyle::bold()))
                .unwrap_or_else(|| skia::Font::default().typeface())
        });
        *opt = Some(tf.clone());
        tf
    })
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RenderOptions {
    pub hardware_accelerated: bool,
    pub high_precision_aa: bool,
    pub canvas_bg_color: Option<Color>,
    pub page_bg_color: Option<Color>,
    pub page_shadow: bool,
    pub page_border: bool,
    pub show_workspace_dots: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            hardware_accelerated: true,
            high_precision_aa: true,
            canvas_bg_color: None,
            page_bg_color: None,
            page_shadow: true,
            page_border: true,
            show_workspace_dots: true,
        }
    }
}

#[derive(Default)]
pub struct SkiaRenderer {
    surface: Option<skia::Surface>,
    width: i32,
    height: i32,
}

impl SkiaRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_surface(&mut self, width: i32, height: i32) -> &mut skia::Surface {
        if self.surface.is_none() || self.width != width || self.height != height {
            let w = width.max(1);
            let h = height.max(1);
            let surface =
                surfaces::raster_n32_premul((w, h)).expect("Failed to create Skia raster surface");
            self.surface = Some(surface);
            self.width = w;
            self.height = h;
        }
        self.surface.as_mut().unwrap()
    }

    pub fn render(
        &mut self,
        width: i32,
        height: i32,
        document: &Document,
        viewport: &Viewport,
        grid_config: &GridConfig,
        snap_guides: &[SnapGuide],
        ruler_config: &RulerConfig,
        render_options: &RenderOptions,
        cursor_pos: Point,
        live_drag_guide: Option<&Guide>,
        hovered_guide_id: Option<u64>,
        corner_drag_current: Option<Point>,
        show_selection_bbox: bool,
        render_overlay: impl FnOnce(&skia::Canvas, &Viewport),
    ) -> Option<ImageSurface> {
        if width <= 0 || height <= 0 {
            return None;
        }

        let surface = self.ensure_surface(width, height);
        let canvas = surface.canvas();

        let is_dark = libadwaita::StyleManager::default().is_dark();
        if let Some(c) = render_options.canvas_bg_color {
            canvas.clear(c.to_skia());
        } else if let Some(theme_bg) = crate::ui::theme::current_visual_theme().workspace_bg_color()
        {
            canvas.clear(theme_bg.to_skia());
        } else if is_dark {
            canvas.clear(Color4f::new(0.14, 0.14, 0.15, 1.0));
        } else {
            canvas.clear(Color4f::new(
                250.0 / 255.0,
                250.0 / 255.0,
                251.0 / 255.0,
                1.0,
            ));
        }

        let widget_size = (width as f32, height as f32);
        let cx = widget_size.0 / 2.0;
        let cy = widget_size.1 / 2.0;

        if render_options.show_workspace_dots {
            draw_infinite_dot_grid(canvas, viewport, widget_size, is_dark);
        }

        canvas.save();
        canvas.translate(skia::Vector::new(cx + viewport.pan.x, cy + viewport.pan.y));
        canvas.scale((viewport.zoom, viewport.zoom));

        draw_pages(canvas, document, viewport.zoom, is_dark, render_options);

        if grid_config.visible {
            draw_alignment_grid(
                canvas,
                widget_size,
                viewport,
                is_dark,
                grid_config,
                ruler_config,
            );
        }

        if ruler_config.guides_visible {
            draw_user_guides(
                canvas,
                &document.guides,
                live_drag_guide,
                hovered_guide_id,
                viewport.zoom,
            );
        }

        let mut visited_clones = std::collections::HashSet::new();
        for element in &document.elements {
            draw_element_node(
                canvas,
                element,
                document,
                render_options.high_precision_aa,
                render_options.hardware_accelerated,
                &mut visited_clones,
            );
        }

        if show_selection_bbox {
            if let Some(sel_bounds) = document.selection_bounds() {
                draw_selection_highlight(canvas, sel_bounds, viewport.zoom);
            }
        }

        render_overlay(canvas, viewport);

        if !snap_guides.is_empty() {
            draw_snap_guides(canvas, snap_guides, viewport.zoom);
        }

        canvas.restore();

        if let Some(drag_pos) = corner_drag_current {
            draw_corner_origin_drag(canvas, widget_size, viewport, drag_pos, is_dark);
        }

        if ruler_config.visible {
            draw_rulers(
                canvas,
                widget_size,
                viewport,
                ruler_config,
                cursor_pos,
                document.selection_bounds(),
                is_dark,
            );
        }

        let image_info = surface.image_info();
        let row_bytes = image_info.min_row_bytes();
        let mut image_surface = ImageSurface::create(Format::ARgb32, width, height).ok()?;
        {
            let mut data = image_surface.data().ok()?;
            let success = surface.read_pixels(&image_info, &mut data, row_bytes, (0, 0));
            if !success {
                return None;
            }
        }
        Some(image_surface)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::Document;
    use crate::core::element::RectElement;
    use crate::core::geometry::Rect;

    #[test]
    fn test_export_document_current_page() {
        let mut doc = Document::default();
        let rect = RectElement::new(Rect::new(50.0, 50.0, 100.0, 100.0), None, None);
        doc.add_element(crate::core::element::Element::Rect(rect));

        let mut config = ExportConfig::default();
        config.base_filename = "test_export".to_string();
        config.format = ExportFormat::Png;
        config.scope = ExportScope::CurrentPage;

        let results = export_document(&doc, &config);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "test_export.png");
        assert!(!results[0].1.is_empty());
    }

    #[test]
    fn test_export_document_all_pages_separate() {
        let mut doc = Document::default();
        doc.add_page(
            Some("Capa".to_string()),
            Rect::new(0.0, 0.0, ARTBOARD_WIDTH, ARTBOARD_HEIGHT),
        );

        let mut config = ExportConfig::default();
        config.base_filename = "doc".to_string();
        config.format = ExportFormat::Jpg;
        config.separate_files = true;
        config.scope = ExportScope::AllPages;

        let results = export_document(&doc, &config);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "doc_Page 1.jpg");
        assert_eq!(results[1].0, "doc_Capa.jpg");
    }
}
