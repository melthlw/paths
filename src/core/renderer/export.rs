use skia_safe::{self as skia, surfaces, Color4f};

use crate::core::document::Document;
use crate::core::geometry::Rect;
use super::draw::draw_element_node;
use super::{ARTBOARD_HEIGHT, ARTBOARD_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Jpg,
    Svg,
    Pdf,
    WebP,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Png => "png",
            ExportFormat::Jpg => "jpg",
            ExportFormat::Svg => "svg",
            ExportFormat::Pdf => "pdf",
            ExportFormat::WebP => "webp",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportScope {
    CurrentPage,
    AllPages,
    SelectedPages(Vec<usize>),
    CompleteDocument,
    Selection,
}

#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub scope: ExportScope,
    pub format: ExportFormat,
    pub separate_files: bool,
    pub base_filename: String,
    pub scale: f32,
    pub dpi: f32,
    pub quality: i32,
    pub antialiasing: bool,
    pub transparent_background: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            scope: ExportScope::CurrentPage,
            format: ExportFormat::Png,
            separate_files: false,
            base_filename: "design".to_string(),
            scale: 1.0,
            dpi: 72.0,
            quality: 95,
            antialiasing: true,
            transparent_background: true,
        }
    }
}

pub fn render_rect_to_skia_surface(
    document: &Document,
    export_rect: Rect,
    scale: f32,
    transparent_bg: bool,
    antialiasing: bool,
) -> Option<skia::Surface> {
    let w = (export_rect.width * scale).ceil().max(1.0) as i32;
    let h = (export_rect.height * scale).ceil().max(1.0) as i32;
    let mut surface = surfaces::raster_n32_premul((w, h))?;
    let canvas = surface.canvas();

    if !transparent_bg {
        canvas.clear(Color4f::new(1.0, 1.0, 1.0, 1.0));
    } else {
        canvas.clear(Color4f::new(0.0, 0.0, 0.0, 0.0));
    }

    canvas.save();
    canvas.scale((scale, scale));
    canvas.translate(skia::Vector::new(-export_rect.x, -export_rect.y));

    let mut visited_clones = std::collections::HashSet::new();
    let mut picture_cache = crate::core::renderer::PictureCache::default();
    for element in &document.elements {
        draw_element_node(
            canvas,
            element,
            document,
            antialiasing,
            false,
            &mut visited_clones,
            &mut picture_cache,
        );
    }

    canvas.restore();
    Some(surface)
}

pub fn export_document(document: &Document, config: &ExportConfig) -> Vec<(String, Vec<u8>)> {
    let mut results = Vec::new();
    let eff_scale = config.scale * (config.dpi / 72.0);

    let tasks: Vec<(String, Rect)> = match &config.scope {
        ExportScope::CurrentPage => {
            let page = document
                .active_page()
                .cloned()
                .unwrap_or_else(crate::core::page::Page::default_a4);
            let name = format!("{}.{}", config.base_filename, config.format.extension());
            vec![(name, page.rect)]
        }
        ExportScope::AllPages => {
            if config.separate_files {
                document
                    .pages
                    .iter()
                    .enumerate()
                    .map(|(idx, p)| {
                        let page_name = if p.name.is_empty() {
                            format!("pagina_{}", idx + 1)
                        } else {
                            p.name.clone()
                        };
                        let name = format!(
                            "{}_{}.{}",
                            config.base_filename,
                            page_name,
                            config.format.extension()
                        );
                        (name, p.rect)
                    })
                    .collect()
            } else {
                let union_rect = document
                    .pages
                    .iter()
                    .map(|p| p.rect)
                    .reduce(|acc, r| acc.union(r))
                    .unwrap_or_else(|| Rect::new(0.0, 0.0, ARTBOARD_WIDTH, ARTBOARD_HEIGHT));
                let name = format!("{}.{}", config.base_filename, config.format.extension());
                vec![(name, union_rect)]
            }
        }
        ExportScope::SelectedPages(indices) => {
            if config.separate_files {
                indices
                    .iter()
                    .filter_map(|&idx| {
                        document.pages.get(idx).map(|p| {
                            let page_name = if p.name.is_empty() {
                                format!("pagina_{}", idx + 1)
                            } else {
                                p.name.clone()
                            };
                            let name = format!(
                                "{}_{}.{}",
                                config.base_filename,
                                page_name,
                                config.format.extension()
                            );
                            (name, p.rect)
                        })
                    })
                    .collect()
            } else {
                let union_rect = indices
                    .iter()
                    .filter_map(|&idx| document.pages.get(idx).map(|p| p.rect))
                    .reduce(|acc, r| acc.union(r))
                    .unwrap_or_else(|| Rect::new(0.0, 0.0, ARTBOARD_WIDTH, ARTBOARD_HEIGHT));
                let name = format!("{}.{}", config.base_filename, config.format.extension());
                vec![(name, union_rect)]
            }
        }
        ExportScope::CompleteDocument => {
            let mut total_rect = Rect::new(0.0, 0.0, ARTBOARD_WIDTH, ARTBOARD_HEIGHT);
            for p in &document.pages {
                total_rect = total_rect.union(p.rect);
            }
            for e in &document.elements {
                if e.visible() {
                    total_rect = total_rect.union(document.element_bounds(e));
                }
            }
            let name = format!("{}.{}", config.base_filename, config.format.extension());
            vec![(name, total_rect)]
        }
        ExportScope::Selection => {
            let sel_rect = document.selection_bounds().unwrap_or_else(|| {
                document
                    .active_page()
                    .map(|p| p.rect)
                    .unwrap_or_else(|| Rect::new(0.0, 0.0, ARTBOARD_WIDTH, ARTBOARD_HEIGHT))
            });
            let name = format!("{}.{}", config.base_filename, config.format.extension());
            vec![(name, sel_rect)]
        }
    };

    if config.format == ExportFormat::Svg {
        for (filename, _) in tasks {
            let svg_content = crate::core::export_document_to_svg(document);
            results.push((filename, svg_content.into_bytes()));
        }
        return results;
    }

    for (filename, rect) in tasks {
        let skia_format = match config.format {
            ExportFormat::Png | ExportFormat::Pdf | ExportFormat::Svg => {
                skia::EncodedImageFormat::PNG
            }
            ExportFormat::Jpg => skia::EncodedImageFormat::JPEG,
            ExportFormat::WebP => skia::EncodedImageFormat::WEBP,
        };

        if let Some(mut surface) = render_rect_to_skia_surface(
            document,
            rect,
            eff_scale,
            config.transparent_background,
            config.antialiasing,
        ) {
            let image = surface.image_snapshot();
            if let Some(data) =
                image.encode(None, skia_format, Some(config.quality.clamp(1, 100) as u32))
            {
                results.push((filename, data.as_bytes().to_vec()));
            }
        }
    }

    results
}
