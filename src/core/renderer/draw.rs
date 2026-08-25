use skia_safe::{self as skia, Color4f, Paint, PaintStyle};

use crate::core::color::Color;
use crate::core::document::Document;
use crate::core::element::{BlendMode, Element};
use crate::core::geometry::{Point, Rect, Viewport};
use crate::core::grid::{GridConfig, GridStyle};
use crate::core::ruler::{calculate_tick_step, Guide, GuideOrientation, RulerConfig};
use crate::core::snap::SnapGuide;
use super::{get_ui_bold_typeface, get_ui_typeface, RenderOptions};

pub fn draw_pages(
    canvas: &skia::Canvas,
    document: &Document,
    zoom: f32,
    is_dark: bool,
    render_options: &RenderOptions,
) {
    let font = skia::Font::new(get_ui_typeface(), 11.0 / zoom);

    for page in &document.pages {
        let r = page.rect.normalize();
        let page_rect = skia::Rect::from_xywh(r.x, r.y, r.width, r.height);

        // 1. Professional Multi-Layer Elevation Drop Shadow
        if render_options.page_shadow {
            let mut shadow_paint = Paint::default();
            shadow_paint.set_color4f(Color4f::new(0.0, 0.0, 0.0, 0.12), None);
            shadow_paint.set_mask_filter(skia::MaskFilter::blur(skia::BlurStyle::Normal, 6.0, false));
            shadow_paint.set_anti_alias(true);
            let shadow_rect = skia::Rect::from_xywh(r.x + 3.0, r.y + 4.0, r.width, r.height);
            canvas.draw_rect(shadow_rect, &shadow_paint);

            // 2. Ambient soft corner shadow
            let mut ambient_shadow = Paint::default();
            ambient_shadow.set_color4f(Color4f::new(0.0, 0.0, 0.0, 0.06), None);
            ambient_shadow.set_mask_filter(skia::MaskFilter::blur(
                skia::BlurStyle::Normal,
                10.0,
                false,
            ));
            ambient_shadow.set_anti_alias(true);
            let ambient_rect = skia::Rect::from_xywh(r.x + 4.0, r.y + 6.0, r.width, r.height);
            canvas.draw_rect(ambient_rect, &ambient_shadow);
        }

        // 3. Paper Sheet Color
        let mut paper_paint = Paint::default();
        if let Some(c) = render_options.page_bg_color {
            paper_paint.set_color4f(c.to_skia(), None);
        } else {
            paper_paint.set_color4f(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
        }
        paper_paint.set_anti_alias(true);
        paper_paint.set_style(PaintStyle::Fill);
        canvas.draw_rect(page_rect, &paper_paint);

        // 4. Page Border
        if render_options.page_border {
            let is_active = document.active_page_id == Some(page.id);
            let mut border_paint = Paint::default();
            if is_active && document.pages.len() > 1 {
                border_paint.set_color4f(Color4f::new(0.2, 0.55, 0.95, 0.4), None);
                border_paint.set_stroke_width(1.5 / zoom);
            } else {
                border_paint.set_color4f(Color4f::new(0.0, 0.0, 0.0, 0.08), None);
                border_paint.set_stroke_width(1.0 / zoom);
            }
            border_paint.set_style(PaintStyle::Stroke);
            border_paint.set_anti_alias(true);
            canvas.draw_rect(page_rect, &border_paint);
        }

        // 5. Page Title Label Banner (Top-Left of the page)
        let label_text = format!("{}  ({:.0} × {:.0})", page.name, r.width, r.height);
        let mut text_paint = Paint::default();
        if is_dark {
            text_paint.set_color4f(Color4f::new(0.7, 0.7, 0.75, 0.85), None);
        } else {
            text_paint.set_color4f(Color4f::new(0.4, 0.42, 0.45, 0.85), None);
        }
        text_paint.set_anti_alias(true);
        canvas.draw_str(
            &label_text,
            skia::Point::new(r.x, r.y - 6.0 / zoom),
            &font,
            &text_paint,
        );
    }
}

pub fn draw_infinite_dot_grid(
    canvas: &skia::Canvas,
    viewport: &Viewport,
    widget_size: (f32, f32),
    is_dark: bool,
) {
    let grid_size = 32.0 * viewport.zoom;
    if grid_size < 12.0 {
        return;
    }

    let cx = widget_size.0 / 2.0;
    let cy = widget_size.1 / 2.0;

    let offset_x = (cx + viewport.pan.x).rem_euclid(grid_size);
    let offset_y = (cy + viewport.pan.y).rem_euclid(grid_size);

    let mut dot_paint = Paint::default();
    if let Some((_, r_fg)) = crate::ui::theme::current_visual_theme().ruler_colors() {
        let mut c = r_fg.to_skia();
        c.a = if is_dark { 0.28 } else { 0.35 };
        dot_paint.set_color4f(c, None);
    } else if is_dark {
        dot_paint.set_color4f(Color4f::new(0.38, 0.38, 0.40, 0.50), None);
    } else {
        dot_paint.set_color4f(Color4f::new(0.76, 0.76, 0.80, 0.55), None);
    }
    dot_paint.set_anti_alias(true);
    let dot_radius = (1.25 * (viewport.zoom / 1.5).clamp(0.8, 1.6)).max(1.0);
    dot_paint.set_stroke_width(dot_radius * 2.0);
    dot_paint.set_stroke_cap(skia::PaintCap::Round);

    let count_x = ((widget_size.0 - offset_x) / grid_size).ceil() as usize + 1;
    let count_y = ((widget_size.1 - offset_y) / grid_size).ceil() as usize + 1;
    let mut points: Vec<skia::Point> = Vec::with_capacity(count_x * count_y);

    let mut x = offset_x;
    while x < widget_size.0 {
        let mut y = offset_y;
        while y < widget_size.1 {
            points.push(skia::Point::new(x, y));
            y += grid_size;
        }
        x += grid_size;
    }

    if !points.is_empty() {
        canvas.draw_points(skia::canvas::PointMode::Points, &points, &dot_paint);
    }
}

pub fn draw_alignment_grid(
    canvas: &skia::Canvas,
    widget_size: (f32, f32),
    viewport: &Viewport,
    _is_dark: bool,
    config: &GridConfig,
    ruler_config: &RulerConfig,
) {
    let base_size = if config.cell_size > 0.0 {
        config.cell_size
    } else {
        32.0
    };
    let subs = config.subdivisions.max(1) as f32;
    let minor_step = base_size / subs;
    let major_step = base_size;

    let zoom = viewport.zoom;
    let line_width = (1.0 / zoom).min(1.0);

    let origin = ruler_config.effective_origin();

    let cx = widget_size.0 / 2.0;
    let cy = widget_size.1 / 2.0;

    let world_left = (-cx - viewport.pan.x) / zoom;
    let world_right = (widget_size.0 - cx - viewport.pan.x) / zoom;
    let world_top = (-cy - viewport.pan.y) / zoom;
    let world_bottom = (widget_size.1 - cy - viewport.pan.y) / zoom;

    let start_major_x = origin.x + ((world_left - origin.x) / major_step).floor() * major_step;
    let start_major_y = origin.y + ((world_top - origin.y) / major_step).floor() * major_step;

    let grid_alpha = config.opacity.clamp(0.02, 1.0);
    let base_color = config.color.unwrap_or(Color::new(0.20, 0.50, 0.90, 1.0));
    let mut major_color = base_color.to_skia();
    major_color.a = grid_alpha;
    let mut minor_color = base_color.to_skia();
    minor_color.a = grid_alpha * 0.45;

    match config.style {
        GridStyle::Lines => {
            // Minor grid lines across the visible canvas
            if minor_step * zoom >= 5.0 && subs > 1.0 {
                let mut minor_paint = Paint::default();
                minor_paint.set_color4f(minor_color, None);
                minor_paint.set_style(PaintStyle::Stroke);
                minor_paint.set_stroke_width(line_width * 0.75);
                minor_paint.set_anti_alias(true);

                let start_minor_x =
                    origin.x + ((world_left - origin.x) / minor_step).floor() * minor_step;
                let mut x = start_minor_x;
                while x <= world_right + minor_step {
                    canvas.draw_line(
                        skia::Point::new(x, world_top),
                        skia::Point::new(x, world_bottom),
                        &minor_paint,
                    );
                    x += minor_step;
                }

                let start_minor_y =
                    origin.y + ((world_top - origin.y) / minor_step).floor() * minor_step;
                let mut y = start_minor_y;
                while y <= world_bottom + minor_step {
                    canvas.draw_line(
                        skia::Point::new(world_left, y),
                        skia::Point::new(world_right, y),
                        &minor_paint,
                    );
                    y += minor_step;
                }
            }

            // Major grid lines across the visible canvas
            if major_step * zoom >= 6.0 {
                let mut major_paint = Paint::default();
                major_paint.set_color4f(major_color, None);
                major_paint.set_style(PaintStyle::Stroke);
                major_paint.set_stroke_width(line_width);
                major_paint.set_anti_alias(true);

                let mut x = start_major_x;
                while x <= world_right + major_step {
                    canvas.draw_line(
                        skia::Point::new(x, world_top),
                        skia::Point::new(x, world_bottom),
                        &major_paint,
                    );
                    x += major_step;
                }

                let mut y = start_major_y;
                while y <= world_bottom + major_step {
                    canvas.draw_line(
                        skia::Point::new(world_left, y),
                        skia::Point::new(world_right, y),
                        &major_paint,
                    );
                    y += major_step;
                }
            }

            // Origin axes
            let mut axis_paint = Paint::default();
            let mut axis_color = major_color;
            axis_color.a = (grid_alpha * 1.5).min(0.8);
            axis_paint.set_color4f(axis_color, None);
            axis_paint.set_style(PaintStyle::Stroke);
            axis_paint.set_stroke_width((1.2 / zoom).min(1.2));
            axis_paint.set_anti_alias(true);

            canvas.draw_line(
                skia::Point::new(origin.x, world_top),
                skia::Point::new(origin.x, world_bottom),
                &axis_paint,
            );
            canvas.draw_line(
                skia::Point::new(world_left, origin.y),
                skia::Point::new(world_right, origin.y),
                &axis_paint,
            );
        }
        GridStyle::Dots => {
            if major_step * zoom >= 6.0 {
                let mut dot_paint = Paint::default();
                dot_paint.set_color4f(major_color, None);
                dot_paint.set_anti_alias(true);
                dot_paint.set_style(PaintStyle::Fill);

                let dot_radius = (1.2 / zoom).clamp(0.6, 1.4);

                let mut x = start_major_x;
                while x <= world_right + major_step {
                    let mut y = start_major_y;
                    while y <= world_bottom + major_step {
                        canvas.draw_circle(skia::Point::new(x, y), dot_radius, &dot_paint);
                        y += major_step;
                    }
                    x += major_step;
                }
            }
        }
        GridStyle::None => {}
    }
}

pub fn draw_element_node(
    canvas: &skia::Canvas,
    element: &Element,
    document: &Document,
    antialiasing: bool,
    hardware_accelerated: bool,
    visited_clones: &mut std::collections::HashSet<crate::core::element::ElementId>,
) {
    if !element.visible() {
        return;
    }

    let is_layer_needed = element.opacity() < 0.999
        || element.blur() > 0.001
        || element.blend_mode() != BlendMode::Normal;

    if is_layer_needed {
        let mut layer_paint = Paint::default();
        layer_paint.set_alpha_f(element.opacity().clamp(0.0, 1.0));
        layer_paint.set_blend_mode(element.blend_mode().to_skia());
        layer_paint.set_anti_alias(antialiasing);
        if element.blur() > 0.001 {
            let sigma = element.blur() * 40.0;
            let tile_mode = if hardware_accelerated {
                skia::TileMode::Decal
            } else {
                skia::TileMode::Clamp
            };
            if let Some(blur_filter) =
                skia::image_filters::blur((sigma, sigma), tile_mode, None, None)
            {
                layer_paint.set_image_filter(blur_filter);
            }
        }
        canvas.save_layer(&skia::canvas::SaveLayerRec::default().paint(&layer_paint));
    }

    match element {
        Element::Clone(c) => {
            if visited_clones.insert(c.id) {
                if let Some(master) = document.find_element(c.source_id) {
                    canvas.save();
                    canvas.translate((c.offset.x, c.offset.y));
                    if c.rotation.abs() > 0.001 {
                        let mb = document.element_bounds(master);
                        let cx = mb.x + mb.width * 0.5;
                        let cy = mb.y + mb.height * 0.5;
                        canvas.rotate(c.rotation.to_degrees(), Some(skia::Point::new(cx, cy)));
                    }
                    if (c.scale.x - 1.0).abs() > 0.001 || (c.scale.y - 1.0).abs() > 0.001 {
                        let mb = document.element_bounds(master);
                        canvas.translate((mb.x, mb.y));
                        canvas.scale((c.scale.x, c.scale.y));
                        canvas.translate((-mb.x, -mb.y));
                    }
                    draw_element_node(
                        canvas,
                        master,
                        document,
                        antialiasing,
                        hardware_accelerated,
                        visited_clones,
                    );
                    canvas.restore();
                }
                visited_clones.remove(&c.id);
            }
        }
        other => {
            other.render_with_doc(canvas, Some(document));
        }
    }

    if is_layer_needed {
        canvas.restore();
    }
}

pub fn draw_snap_guides(canvas: &skia::Canvas, guides: &[SnapGuide], zoom: f32) {
    let stroke_width = (1.2 / zoom).max(0.75);
    let badge_size = (6.0 / zoom).max(3.0);

    let mut guide_paint = Paint::default();
    guide_paint.set_color4f(Color4f::new(1.0, 0.0, 0.48, 0.95), None);
    guide_paint.set_style(PaintStyle::Stroke);
    guide_paint.set_stroke_width(stroke_width);
    guide_paint.set_anti_alias(true);

    let mut badge_fill = Paint::default();
    badge_fill.set_color4f(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    badge_fill.set_style(PaintStyle::Fill);
    badge_fill.set_anti_alias(true);

    for guide in guides {
        match guide {
            SnapGuide::Vertical { x, y_min, y_max } => {
                canvas.draw_line(
                    skia::Point::new(*x, *y_min),
                    skia::Point::new(*x, *y_max),
                    &guide_paint,
                );
                canvas.draw_line(
                    skia::Point::new(*x - badge_size, *y_min),
                    skia::Point::new(*x + badge_size, *y_min),
                    &guide_paint,
                );
                canvas.draw_line(
                    skia::Point::new(*x - badge_size, *y_max),
                    skia::Point::new(*x + badge_size, *y_max),
                    &guide_paint,
                );
            }
            SnapGuide::Horizontal { y, x_min, x_max } => {
                canvas.draw_line(
                    skia::Point::new(*x_min, *y),
                    skia::Point::new(*x_max, *y),
                    &guide_paint,
                );
                canvas.draw_line(
                    skia::Point::new(*x_min, *y - badge_size),
                    skia::Point::new(*x_min, *y + badge_size),
                    &guide_paint,
                );
                canvas.draw_line(
                    skia::Point::new(*x_max, *y - badge_size),
                    skia::Point::new(*x_max, *y + badge_size),
                    &guide_paint,
                );
            }
            SnapGuide::Point { point } => {
                canvas.draw_circle(point.to_skia(), badge_size * 1.2, &badge_fill);
                canvas.draw_circle(point.to_skia(), badge_size * 1.2, &guide_paint);
            }
        }
    }
}

pub fn draw_selection_highlight(canvas: &skia::Canvas, bounds: Rect, zoom: f32) {
    let z = zoom.max(0.001);
    let stroke_width = 1.2 / z;
    let handle_size = 7.5 / z;
    let r = bounds.normalize();
    let theme_accent = crate::ui::theme::current_visual_theme().accent_color().to_skia();

    // 1. Subtle Outer Drop Shadow Line for Visibility
    let mut shadow_stroke = Paint::default();
    shadow_stroke.set_color4f(Color4f::new(0.0, 0.0, 0.0, 0.22), None);
    shadow_stroke.set_style(PaintStyle::Stroke);
    shadow_stroke.set_stroke_width(stroke_width + 1.0 / z);
    shadow_stroke.set_anti_alias(true);
    canvas.draw_rect(r.to_skia(), &shadow_stroke);

    // 2. Primary Vibrant Accent Stroke
    let mut stroke_paint = Paint::default();
    stroke_paint.set_color4f(theme_accent, None);
    stroke_paint.set_style(PaintStyle::Stroke);
    stroke_paint.set_stroke_width(stroke_width);
    stroke_paint.set_anti_alias(true);
    canvas.draw_rect(r.to_skia(), &stroke_paint);

    // 3. Handle Shadow & Fill Paints
    let mut handle_shadow = Paint::default();
    handle_shadow.set_color4f(Color4f::new(0.0, 0.0, 0.0, 0.30), None);
    handle_shadow.set_style(PaintStyle::Fill);
    handle_shadow.set_anti_alias(true);

    let mut handle_fill = Paint::default();
    handle_fill.set_color4f(Color4f::new(1.0, 1.0, 1.0, 1.0), None);
    handle_fill.set_style(PaintStyle::Fill);
    handle_fill.set_anti_alias(true);

    let mut handle_stroke = Paint::default();
    handle_stroke.set_color4f(theme_accent, None);
    handle_stroke.set_style(PaintStyle::Stroke);
    handle_stroke.set_stroke_width(stroke_width);
    handle_stroke.set_anti_alias(true);

    let points = [
        Point::new(r.x, r.y),
        Point::new(r.x + r.width / 2.0, r.y),
        Point::new(r.x + r.width, r.y),
        Point::new(r.x + r.width, r.y + r.height / 2.0),
        Point::new(r.x + r.width, r.y + r.height),
        Point::new(r.x + r.width / 2.0, r.y + r.height),
        Point::new(r.x, r.y + r.height),
        Point::new(r.x, r.y + r.height / 2.0),
    ];

    let half = handle_size / 2.0;
    let corner_rad = 1.5 / z;

    for p in points {
        let shadow_rect = skia::Rect::from_xywh(
            p.x - half,
            p.y - half + 0.8 / z,
            handle_size,
            handle_size,
        );
        let handle_rect = skia::Rect::from_xywh(p.x - half, p.y - half, handle_size, handle_size);
        canvas.draw_round_rect(shadow_rect, corner_rad, corner_rad, &handle_shadow);
        canvas.draw_round_rect(handle_rect, corner_rad, corner_rad, &handle_fill);
        canvas.draw_round_rect(handle_rect, corner_rad, corner_rad, &handle_stroke);
    }

    // 4. Rotation Lollipop Handle
    let rot_top = Point::new(r.x + r.width / 2.0, r.y);
    let rot_handle = Point::new(r.x + r.width / 2.0, r.y - 20.0 / z);
    let rot_radius = half * 1.15;

    canvas.draw_line(rot_top.to_skia(), rot_handle.to_skia(), &stroke_paint);
    canvas.draw_circle(
        skia::Point::new(rot_handle.x, rot_handle.y + 0.8 / z),
        rot_radius,
        &handle_shadow,
    );
    canvas.draw_circle(rot_handle.to_skia(), rot_radius, &handle_fill);
    canvas.draw_circle(rot_handle.to_skia(), rot_radius, &handle_stroke);

    // Subtle accent center dot in rotation handle
    let mut dot_paint = Paint::default();
    dot_paint.set_color4f(theme_accent, None);
    dot_paint.set_style(PaintStyle::Fill);
    dot_paint.set_anti_alias(true);
    canvas.draw_circle(rot_handle.to_skia(), rot_radius * 0.45, &dot_paint);
}

pub fn draw_user_guides(
    canvas: &skia::Canvas,
    guides: &[Guide],
    live_guide: Option<&Guide>,
    hovered_id: Option<u64>,
    zoom: f32,
) {
    let mut default_paint = Paint::default();
    default_paint.set_color4f(Color4f::new(0.0, 0.78, 1.0, 0.75), None);
    default_paint.set_style(PaintStyle::Stroke);
    default_paint.set_stroke_width((1.0 / zoom).max(0.75));
    default_paint.set_anti_alias(true);

    let mut hover_paint = Paint::default();
    hover_paint.set_color4f(Color4f::new(0.208, 0.518, 0.894, 1.0), None);
    hover_paint.set_style(PaintStyle::Stroke);
    hover_paint.set_stroke_width((1.8 / zoom).max(1.2));
    hover_paint.set_anti_alias(true);

    for guide in guides {
        let is_hovered = hovered_id == Some(guide.id);
        let p = if is_hovered {
            &hover_paint
        } else {
            &default_paint
        };

        match guide.orientation {
            GuideOrientation::Horizontal => {
                canvas.draw_line(
                    skia::Point::new(-100000.0, guide.position),
                    skia::Point::new(100000.0, guide.position),
                    p,
                );
            }
            GuideOrientation::Vertical => {
                canvas.draw_line(
                    skia::Point::new(guide.position, -100000.0),
                    skia::Point::new(guide.position, 100000.0),
                    p,
                );
            }
        }
    }

    if let Some(guide) = live_guide {
        let mut live_paint = Paint::default();
        live_paint.set_color4f(Color4f::new(0.208, 0.518, 0.894, 0.95), None);
        live_paint.set_style(PaintStyle::Stroke);
        live_paint.set_stroke_width((1.8 / zoom).max(1.2));
        live_paint.set_path_effect(skia::dash_path_effect::new(&[4.0 / zoom, 4.0 / zoom], 0.0));
        live_paint.set_anti_alias(true);

        match guide.orientation {
            GuideOrientation::Horizontal => {
                canvas.draw_line(
                    skia::Point::new(-100000.0, guide.position),
                    skia::Point::new(100000.0, guide.position),
                    &live_paint,
                );
            }
            GuideOrientation::Vertical => {
                canvas.draw_line(
                    skia::Point::new(guide.position, -100000.0),
                    skia::Point::new(guide.position, 100000.0),
                    &live_paint,
                );
            }
        }
    }
}

pub fn draw_corner_origin_drag(
    canvas: &skia::Canvas,
    widget_size: (f32, f32),
    viewport: &Viewport,
    drag_world_pos: Point,
    is_dark: bool,
) {
    let (w, h) = widget_size;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let screen_x = cx + viewport.pan.x + drag_world_pos.x * viewport.zoom;
    let screen_y = cy + viewport.pan.y + drag_world_pos.y * viewport.zoom;

    let mut line_paint = Paint::default();
    line_paint.set_color4f(Color4f::new(0.208, 0.518, 0.894, 0.85), None);
    line_paint.set_style(PaintStyle::Stroke);
    line_paint.set_stroke_width(1.0);
    line_paint.set_path_effect(skia::dash_path_effect::new(&[4.0, 4.0], 0.0));

    canvas.draw_line(
        skia::Point::new(screen_x, 0.0),
        skia::Point::new(screen_x, h),
        &line_paint,
    );
    canvas.draw_line(
        skia::Point::new(0.0, screen_y),
        skia::Point::new(w, screen_y),
        &line_paint,
    );

    let mut center_paint = Paint::default();
    center_paint.set_color4f(Color4f::new(0.208, 0.518, 0.894, 1.0), None);
    center_paint.set_style(PaintStyle::Fill);
    center_paint.set_anti_alias(true);
    canvas.draw_circle(skia::Point::new(screen_x, screen_y), 4.0, &center_paint);

    let label = format!(
        "Origem (0,0) [{:.1}, {:.1}]",
        drag_world_pos.x, drag_world_pos.y
    );
    let mut font = skia::Font::new(get_ui_typeface(), 10.0);
    font.set_subpixel(true);
    let (tw, _) = font.measure_str(&label, None);

    let badge_rect = skia::Rect::from_xywh(screen_x + 8.0, screen_y - 20.0, tw + 12.0, 18.0);
    let mut bg_paint = Paint::default();
    bg_paint.set_color4f(
        if is_dark {
            Color4f::new(0.1, 0.1, 0.12, 0.95)
        } else {
            Color4f::new(0.98, 0.98, 1.0, 0.95)
        },
        None,
    );
    bg_paint.set_style(PaintStyle::Fill);
    canvas.draw_round_rect(badge_rect, 4.0, 4.0, &bg_paint);

    let mut border = Paint::default();
    border.set_color4f(Color4f::new(0.208, 0.518, 0.894, 0.9), None);
    border.set_style(PaintStyle::Stroke);
    border.set_stroke_width(1.0);
    canvas.draw_round_rect(badge_rect, 4.0, 4.0, &border);

    let mut text_paint = Paint::default();
    text_paint.set_color4f(Color4f::new(0.208, 0.518, 0.894, 1.0), None);
    text_paint.set_anti_alias(true);
    canvas.draw_str(
        &label,
        skia::Point::new(screen_x + 14.0, screen_y - 7.0),
        &font,
        &text_paint,
    );
}

pub fn draw_rulers(
    canvas: &skia::Canvas,
    widget_size: (f32, f32),
    viewport: &Viewport,
    ruler_config: &RulerConfig,
    cursor_pos: Point,
    selection_bounds: Option<Rect>,
    is_dark: bool,
) {
    let (w, h) = widget_size;
    let t = ruler_config.thickness;
    let cx = w / 2.0;
    let cy = h / 2.0;

    let origin = ruler_config.effective_origin();
    let origin_offset_x = origin.x;
    let origin_offset_y = origin.y;

    let theme_ruler = crate::ui::theme::current_visual_theme().ruler_colors();
    let theme_accent = crate::ui::theme::current_visual_theme().accent_color().to_skia();

    let bg_color = if let Some((r_bg, _)) = theme_ruler {
        r_bg.to_skia()
    } else if is_dark {
        Color4f::new(0.14, 0.14, 0.15, 0.98)
    } else {
        Color4f::new(0.95, 0.95, 0.96, 0.98)
    };
    let border_color = if is_dark {
        Color4f::new(0.24, 0.24, 0.26, 1.0)
    } else {
        Color4f::new(0.82, 0.82, 0.84, 1.0)
    };
    let tick_color = if let Some((_, r_fg)) = theme_ruler {
        let mut c = r_fg.to_skia();
        c.a = 0.65;
        c
    } else if is_dark {
        Color4f::new(0.55, 0.55, 0.58, 1.0)
    } else {
        Color4f::new(0.55, 0.55, 0.58, 1.0)
    };
    let text_color = if let Some((_, r_fg)) = theme_ruler {
        r_fg.to_skia()
    } else if is_dark {
        Color4f::new(0.85, 0.85, 0.88, 1.0)
    } else {
        Color4f::new(0.20, 0.20, 0.22, 1.0)
    };

    let mut bg_paint = Paint::default();
    bg_paint.set_color4f(bg_color, None);
    bg_paint.set_style(PaintStyle::Fill);

    let mut border_paint = Paint::default();
    border_paint.set_color4f(border_color, None);
    border_paint.set_style(PaintStyle::Stroke);
    border_paint.set_stroke_width(1.0);

    let mut tick_paint = Paint::default();
    tick_paint.set_color4f(tick_color, None);
    tick_paint.set_style(PaintStyle::Stroke);
    tick_paint.set_stroke_width(1.0);

    let mut text_paint = Paint::default();
    text_paint.set_color4f(text_color, None);
    text_paint.set_style(PaintStyle::Fill);
    text_paint.set_anti_alias(true);

    let mut font = skia::Font::new(get_ui_typeface(), 9.0);
    font.set_subpixel(true);

    // 1. Top Horizontal Ruler Background
    canvas.draw_rect(skia::Rect::from_xywh(t, 0.0, w - t, t), &bg_paint);
    canvas.draw_line(
        skia::Point::new(t, t),
        skia::Point::new(w, t),
        &border_paint,
    );

    // 2. Left Vertical Ruler Background
    canvas.draw_rect(skia::Rect::from_xywh(0.0, t, t, h - t), &bg_paint);
    canvas.draw_line(
        skia::Point::new(t, t),
        skia::Point::new(t, h),
        &border_paint,
    );

    // 3. Selection Span Background Highlights
    let mut sel_coords = None;
    if let Some(sel) = selection_bounds {
        let sel_norm = sel.normalize();
        let screen_x1 = cx + viewport.pan.x + sel_norm.x * viewport.zoom;
        let screen_x2 = cx + viewport.pan.x + (sel_norm.x + sel_norm.width) * viewport.zoom;
        let screen_y1 = cy + viewport.pan.y + sel_norm.y * viewport.zoom;
        let screen_y2 = cy + viewport.pan.y + (sel_norm.y + sel_norm.height) * viewport.zoom;

        let mut sel_span_paint = Paint::default();
        sel_span_paint.set_color4f(Color4f::new(theme_accent.r, theme_accent.g, theme_accent.b, 0.18), None);
        sel_span_paint.set_style(PaintStyle::Fill);

        let mut sel_bracket_paint = Paint::default();
        sel_bracket_paint.set_color4f(Color4f::new(theme_accent.r, theme_accent.g, theme_accent.b, 0.9), None);
        sel_bracket_paint.set_style(PaintStyle::Stroke);
        sel_bracket_paint.set_stroke_width(1.5);

        let top_x1 = screen_x1.max(t);
        let top_x2 = screen_x2.min(w);
        if top_x2 > top_x1 {
            canvas.draw_rect(
                skia::Rect::from_xywh(top_x1, 0.0, top_x2 - top_x1, t),
                &sel_span_paint,
            );
            if screen_x1 >= t && screen_x1 <= w {
                canvas.draw_line(
                    skia::Point::new(screen_x1, 0.0),
                    skia::Point::new(screen_x1, t),
                    &sel_bracket_paint,
                );
            }
            if screen_x2 >= t && screen_x2 <= w {
                canvas.draw_line(
                    skia::Point::new(screen_x2, 0.0),
                    skia::Point::new(screen_x2, t),
                    &sel_bracket_paint,
                );
            }
        }

        let left_y1 = screen_y1.max(t);
        let left_y2 = screen_y2.min(h);
        if left_y2 > left_y1 {
            canvas.draw_rect(
                skia::Rect::from_xywh(0.0, left_y1, t, left_y2 - left_y1),
                &sel_span_paint,
            );
            if screen_y1 >= t && screen_y1 <= h {
                canvas.draw_line(
                    skia::Point::new(0.0, screen_y1),
                    skia::Point::new(t, screen_y1),
                    &sel_bracket_paint,
                );
            }
            if screen_y2 >= t && screen_y2 <= h {
                canvas.draw_line(
                    skia::Point::new(0.0, screen_y2),
                    skia::Point::new(t, screen_y2),
                    &sel_bracket_paint,
                );
            }
        }

        sel_coords = Some((sel_norm, top_x1, top_x2, left_y1, left_y2));
    }

    // 4. Horizontal Ruler Ticks & Numbers
    let step = calculate_tick_step(viewport.zoom);
    let minor_step = step / 5.0;

    let world_left = (t - cx - viewport.pan.x) / viewport.zoom;
    let world_right = (w - cx - viewport.pan.x) / viewport.zoom;
    let ruler_val_min = world_left - origin_offset_x;
    let ruler_val_max = world_right - origin_offset_x;

    let start_major = (ruler_val_min / step).floor() * step;
    let mut cur_val = start_major;

    while cur_val <= ruler_val_max + step {
        let world_x = cur_val + origin_offset_x;
        let screen_x = cx + viewport.pan.x + world_x * viewport.zoom;

        if screen_x >= t && screen_x <= w {
            canvas.draw_line(
                skia::Point::new(screen_x, t - 8.0),
                skia::Point::new(screen_x, t),
                &tick_paint,
            );

            let label = format!("{}", cur_val.round() as i32);
            canvas.draw_str(
                &label,
                skia::Point::new(screen_x + 3.0, 11.0),
                &font,
                &text_paint,
            );
        }

        for i in 1..5 {
            let minor_val = cur_val + i as f32 * minor_step;
            let m_world_x = minor_val + origin_offset_x;
            let m_screen_x = cx + viewport.pan.x + m_world_x * viewport.zoom;
            if m_screen_x >= t && m_screen_x <= w {
                let tick_len = if i == 2 || i == 3 { 5.0 } else { 3.0 };
                canvas.draw_line(
                    skia::Point::new(m_screen_x, t - tick_len),
                    skia::Point::new(m_screen_x, t),
                    &tick_paint,
                );
            }
        }

        cur_val += step;
    }

    // 5. Vertical Ruler Ticks & Numbers
    let world_top = (t - cy - viewport.pan.y) / viewport.zoom;
    let world_bottom = (h - cy - viewport.pan.y) / viewport.zoom;
    let ruler_val_y_min = world_top - origin_offset_y;
    let ruler_val_y_max = world_bottom - origin_offset_y;

    let start_major_y = (ruler_val_y_min / step).floor() * step;
    let mut cur_val_y = start_major_y;

    while cur_val_y <= ruler_val_y_max + step {
        let world_y = cur_val_y + origin_offset_y;
        let screen_y = cy + viewport.pan.y + world_y * viewport.zoom;

        if screen_y >= t && screen_y <= h {
            canvas.draw_line(
                skia::Point::new(t - 8.0, screen_y),
                skia::Point::new(t, screen_y),
                &tick_paint,
            );

            canvas.save();
            canvas.translate(skia::Vector::new(11.0, screen_y - 3.0));
            canvas.rotate(-90.0, None);
            let label = format!("{}", cur_val_y.round() as i32);
            canvas.draw_str(&label, skia::Point::new(0.0, 0.0), &font, &text_paint);
            canvas.restore();
        }

        for i in 1..5 {
            let minor_val = cur_val_y + i as f32 * minor_step;
            let m_world_y = minor_val + origin_offset_y;
            let m_screen_y = cy + viewport.pan.y + m_world_y * viewport.zoom;
            if m_screen_y >= t && m_screen_y <= h {
                let tick_len = if i == 2 || i == 3 { 5.0 } else { 3.0 };
                canvas.draw_line(
                    skia::Point::new(t - tick_len, m_screen_y),
                    skia::Point::new(t, m_screen_y),
                    &tick_paint,
                );
            }
        }

        cur_val_y += step;
    }

    // 6. Selection Dimension Badges ON TOP
    if let Some((sel_norm, top_x1, top_x2, left_y1, left_y2)) = sel_coords {
        let mut badge_text_paint = Paint::default();
        let badge_text_color = if is_dark {
            Color4f::new(0.92, 0.95, 1.0, 1.0)
        } else {
            Color4f::new(0.06, 0.24, 0.58, 1.0)
        };
        badge_text_paint.set_color4f(badge_text_color, None);
        badge_text_paint.set_anti_alias(true);

        let mut badge_border_paint = Paint::default();
        badge_border_paint.set_color4f(Color4f::new(0.208, 0.518, 0.894, 0.95), None);
        badge_border_paint.set_style(PaintStyle::Stroke);
        badge_border_paint.set_stroke_width(1.3);
        badge_border_paint.set_anti_alias(true);

        let mut dim_font = skia::Font::new(get_ui_bold_typeface(), 10.5);
        dim_font.set_subpixel(true);

        // Top Horizontal Ruler Dimension Badge
        if top_x2 > top_x1 {
            let w_val = sel_norm.width;
            let w_label = if (w_val.fract()).abs() < 0.05 {
                format!("{:.0} px", w_val)
            } else {
                format!("{:.1} px", w_val)
            };
            let (label_w, _) = dim_font.measure_str(&w_label, None);
            let mid_x = (top_x1 + top_x2) / 2.0;

            if (top_x2 - top_x1) >= label_w + 6.0 {
                let badge_w = label_w + 12.0;
                let badge_h = t - 4.0;
                let badge_x = mid_x - badge_w / 2.0;
                let badge_y = 2.0;
                let r_pill = skia::Rect::from_xywh(badge_x, badge_y, badge_w, badge_h);

                let mut shadow_paint = Paint::default();
                shadow_paint.set_color4f(
                    Color4f::new(0.0, 0.0, 0.0, if is_dark { 0.45 } else { 0.15 }),
                    None,
                );
                shadow_paint.set_style(PaintStyle::Fill);
                canvas.draw_round_rect(
                    skia::Rect::from_xywh(badge_x, badge_y + 1.0, badge_w, badge_h),
                    4.0,
                    4.0,
                    &shadow_paint,
                );

                let mut pill_bg = Paint::default();
                pill_bg.set_color4f(
                    if is_dark {
                        Color4f::new(0.12, 0.13, 0.16, 1.0)
                    } else {
                        Color4f::new(1.0, 1.0, 1.0, 1.0)
                    },
                    None,
                );
                pill_bg.set_style(PaintStyle::Fill);
                canvas.draw_round_rect(r_pill, 4.0, 4.0, &pill_bg);
                canvas.draw_round_rect(r_pill, 4.0, 4.0, &badge_border_paint);

                let text_x = badge_x + (badge_w - label_w) / 2.0;
                let text_y = badge_y + badge_h / 2.0 + 3.8;
                canvas.draw_str(
                    &w_label,
                    skia::Point::new(text_x, text_y),
                    &dim_font,
                    &badge_text_paint,
                );
            }
        }

        // Left Vertical Ruler Dimension Badge
        if left_y2 > left_y1 {
            let h_val = sel_norm.height;
            let h_label = if (h_val.fract()).abs() < 0.05 {
                format!("{:.0} px", h_val)
            } else {
                format!("{:.1} px", h_val)
            };
            let (label_w, _) = dim_font.measure_str(&h_label, None);
            let mid_y = (left_y1 + left_y2) / 2.0;

            if (left_y2 - left_y1) >= label_w + 6.0 {
                let badge_w = label_w + 12.0;
                let badge_h = t - 4.0;

                canvas.save();
                canvas.translate(skia::Vector::new(t / 2.0, mid_y));
                canvas.rotate(-90.0, None);

                let r_pill =
                    skia::Rect::from_xywh(-badge_w / 2.0, -badge_h / 2.0, badge_w, badge_h);

                let mut shadow_paint = Paint::default();
                shadow_paint.set_color4f(
                    Color4f::new(0.0, 0.0, 0.0, if is_dark { 0.45 } else { 0.15 }),
                    None,
                );
                shadow_paint.set_style(PaintStyle::Fill);
                canvas.draw_round_rect(
                    skia::Rect::from_xywh(-badge_w / 2.0, -badge_h / 2.0 + 1.0, badge_w, badge_h),
                    4.0,
                    4.0,
                    &shadow_paint,
                );

                let mut pill_bg = Paint::default();
                pill_bg.set_color4f(
                    if is_dark {
                        Color4f::new(0.12, 0.13, 0.16, 1.0)
                    } else {
                        Color4f::new(1.0, 1.0, 1.0, 1.0)
                    },
                    None,
                );
                pill_bg.set_style(PaintStyle::Fill);
                canvas.draw_round_rect(r_pill, 4.0, 4.0, &pill_bg);
                canvas.draw_round_rect(r_pill, 4.0, 4.0, &badge_border_paint);

                let text_x = -label_w / 2.0;
                let text_y = 3.8;
                canvas.draw_str(
                    &h_label,
                    skia::Point::new(text_x, text_y),
                    &dim_font,
                    &badge_text_paint,
                );
                canvas.restore();
            }
        }
    }

    // 7. Cursor Indicators on Rulers
    if cursor_pos.x >= t && cursor_pos.y >= t {
        let mut cursor_paint = Paint::default();
        cursor_paint.set_color4f(theme_accent, None);
        cursor_paint.set_style(PaintStyle::Stroke);
        cursor_paint.set_stroke_width(1.2);

        canvas.draw_line(
            skia::Point::new(cursor_pos.x, 0.0),
            skia::Point::new(cursor_pos.x, t),
            &cursor_paint,
        );

        canvas.draw_line(
            skia::Point::new(0.0, cursor_pos.y),
            skia::Point::new(t, cursor_pos.y),
            &cursor_paint,
        );
    }

    // 8. Top-Left Corner Block
    canvas.draw_rect(skia::Rect::from_xywh(0.0, 0.0, t, t), &bg_paint);
    canvas.draw_line(
        skia::Point::new(0.0, t),
        skia::Point::new(t, t),
        &border_paint,
    );
    canvas.draw_line(
        skia::Point::new(t, 0.0),
        skia::Point::new(t, t),
        &border_paint,
    );

    let mut icon_paint = Paint::default();
    icon_paint.set_color4f(
        if ruler_config.custom_origin.is_some() {
            theme_accent
        } else {
            tick_color
        },
        None,
    );
    icon_paint.set_style(PaintStyle::Stroke);
    icon_paint.set_stroke_width(1.0);
    icon_paint.set_anti_alias(true);

    let ch_center = t / 2.0;
    canvas.draw_line(
        skia::Point::new(ch_center - 5.0, ch_center),
        skia::Point::new(ch_center + 5.0, ch_center),
        &icon_paint,
    );
    canvas.draw_line(
        skia::Point::new(ch_center, ch_center - 5.0),
        skia::Point::new(ch_center, ch_center + 5.0),
        &icon_paint,
    );
    canvas.draw_circle(skia::Point::new(ch_center, ch_center), 2.5, &icon_paint);
}
