use skia_safe as skia;

use crate::core::{PageId, Point, PointerButton, PointerEvent, Rect, Viewport};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
enum PageHandle {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
}

#[derive(Debug, Clone, PartialEq)]
enum PageDragState {
    Idle,
    Creating {
        start_pos: Point,
        current_pos: Point,
    },
    Moving {
        page_id: PageId,
        start_pos: Point,
        orig_rect: Rect,
    },
    Resizing {
        page_id: PageId,
        handle: PageHandle,
        start_pos: Point,
        orig_rect: Rect,
    },
}

pub struct PageFeature {
    drag_state: PageDragState,
    hovered_handle: Option<PageHandle>,
}

impl Default for PageFeature {
    fn default() -> Self {
        Self::new()
    }
}

impl PageFeature {
    pub fn new() -> Self {
        Self {
            drag_state: PageDragState::Idle,
            hovered_handle: None,
        }
    }

    fn get_handle_positions(rect: Rect) -> [(PageHandle, Point); 8] {
        let r = rect.normalize();
        let mid_x = r.x + r.width / 2.0;
        let mid_y = r.y + r.height / 2.0;
        [
            (PageHandle::NorthWest, Point::new(r.x, r.y)),
            (PageHandle::North, Point::new(mid_x, r.y)),
            (PageHandle::NorthEast, Point::new(r.x + r.width, r.y)),
            (PageHandle::East, Point::new(r.x + r.width, mid_y)),
            (
                PageHandle::SouthEast,
                Point::new(r.x + r.width, r.y + r.height),
            ),
            (PageHandle::South, Point::new(mid_x, r.y + r.height)),
            (PageHandle::SouthWest, Point::new(r.x, r.y + r.height)),
            (PageHandle::West, Point::new(r.x, mid_y)),
        ]
    }

    fn hit_test_handles(rect: Rect, point: Point, zoom: f32) -> Option<PageHandle> {
        let hit_dist = 8.0 / zoom;
        for (handle, pt) in Self::get_handle_positions(rect) {
            if pt.distance_to(point) <= hit_dist {
                return Some(handle);
            }
        }
        None
    }

    fn cursor_for_handle(handle: PageHandle) -> &'static str {
        match handle {
            PageHandle::NorthWest | PageHandle::SouthEast => "nwse-resize",
            PageHandle::NorthEast | PageHandle::SouthWest => "nesw-resize",
            PageHandle::North | PageHandle::South => "ns-resize",
            PageHandle::East | PageHandle::West => "ew-resize",
        }
    }
}

impl FeaturePlugin for PageFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("crosshair");
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        let zoom = ctx.viewport.zoom;

        // 1. Check if clicking on active page resize handle
        if let Some(active_page) = ctx.document.active_page() {
            if let Some(handle) = Self::hit_test_handles(active_page.rect, event.world_pos, zoom) {
                let page_id = active_page.id;
                let orig_rect = active_page.rect;
                self.drag_state = PageDragState::Resizing {
                    page_id,
                    handle,
                    start_pos: event.world_pos,
                    orig_rect,
                };
                ctx.set_cursor(Self::cursor_for_handle(handle));
                ctx.request_redraw();
                return;
            }
        }

        // 2. Check if clicking on any page body / banner
        let mut clicked_page_id = None;
        let mut clicked_rect = Rect::ZERO;
        for page in ctx.document.pages.iter().rev() {
            let r = page.rect.normalize();
            let banner_rect = Rect::new(r.x, r.y - 20.0 / zoom, r.width, 20.0 / zoom);
            if r.contains(event.world_pos) || banner_rect.contains(event.world_pos) {
                clicked_page_id = Some(page.id);
                clicked_rect = page.rect;
                break;
            }
        }

        if let Some(pid) = clicked_page_id {
            ctx.document.select_page(pid);
            self.drag_state = PageDragState::Moving {
                page_id: pid,
                start_pos: event.world_pos,
                orig_rect: clicked_rect,
            };
            ctx.set_cursor("grab");
            ctx.request_redraw();
            return;
        }

        // 3. Otherwise start creating a new page by dragging
        self.drag_state = PageDragState::Creating {
            start_pos: event.world_pos,
            current_pos: event.world_pos,
        };
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        let zoom = ctx.viewport.zoom;

        match &mut self.drag_state {
            PageDragState::Creating { current_pos, .. } => {
                *current_pos = event.world_pos;
                ctx.set_cursor("crosshair");
                ctx.request_redraw();
            }
            PageDragState::Moving {
                page_id,
                start_pos,
                orig_rect,
            } => {
                let dx = event.world_pos.x - start_pos.x;
                let dy = event.world_pos.y - start_pos.y;
                let new_rect = Rect::new(
                    orig_rect.x + dx,
                    orig_rect.y + dy,
                    orig_rect.width,
                    orig_rect.height,
                )
                .round();
                if let Some(page) = ctx.document.pages.iter_mut().find(|p| p.id == *page_id) {
                    page.rect = new_rect;
                }
                ctx.set_cursor("grabbing");
                ctx.request_redraw();
            }
            PageDragState::Resizing {
                page_id,
                handle,
                start_pos,
                orig_rect,
            } => {
                let dx = event.world_pos.x - start_pos.x;
                let dy = event.world_pos.y - start_pos.y;
                let mut min_x = orig_rect.x;
                let mut min_y = orig_rect.y;
                let mut max_x = orig_rect.x + orig_rect.width;
                let mut max_y = orig_rect.y + orig_rect.height;

                match handle {
                    PageHandle::NorthWest => {
                        min_x += dx;
                        min_y += dy;
                    }
                    PageHandle::North => {
                        min_y += dy;
                    }
                    PageHandle::NorthEast => {
                        max_x += dx;
                        min_y += dy;
                    }
                    PageHandle::East => {
                        max_x += dx;
                    }
                    PageHandle::SouthEast => {
                        max_x += dx;
                        max_y += dy;
                    }
                    PageHandle::South => {
                        max_y += dy;
                    }
                    PageHandle::SouthWest => {
                        min_x += dx;
                        max_y += dy;
                    }
                    PageHandle::West => {
                        min_x += dx;
                    }
                }

                let new_w = (max_x - min_x).max(100.0);
                let new_h = (max_y - min_y).max(100.0);
                let new_rect = Rect::new(min_x, min_y, new_w, new_h).round();

                if let Some(page) = ctx.document.pages.iter_mut().find(|p| p.id == *page_id) {
                    page.rect = new_rect;
                }
                ctx.set_cursor(Self::cursor_for_handle(*handle));
                ctx.request_redraw();
            }
            PageDragState::Idle => {
                // Update cursor based on hover
                if let Some(active_page) = ctx.document.active_page() {
                    if let Some(handle) =
                        Self::hit_test_handles(active_page.rect, event.world_pos, zoom)
                    {
                        self.hovered_handle = Some(handle);
                        ctx.set_cursor(Self::cursor_for_handle(handle));
                        return;
                    }
                }
                self.hovered_handle = None;

                let mut is_over_page = false;
                for page in &ctx.document.pages {
                    if page.hit_test(event.world_pos) {
                        is_over_page = true;
                        break;
                    }
                }
                if is_over_page {
                    ctx.set_cursor("grab");
                } else {
                    ctx.set_cursor("crosshair");
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        match self.drag_state {
            PageDragState::Creating {
                start_pos,
                current_pos,
            } => {
                let min_x = start_pos.x.min(current_pos.x);
                let min_y = start_pos.y.min(current_pos.y);
                let width = (start_pos.x - current_pos.x).abs();
                let height = (start_pos.y - current_pos.y).abs();

                if width >= 100.0 && height >= 100.0 {
                    let rect = Rect::new(min_x, min_y, width, height).round();
                    ctx.document.add_page(None, rect);
                }
            }
            PageDragState::Moving { .. } | PageDragState::Resizing { .. } => {
                ctx.document.snapshot();
            }
            PageDragState::Idle => {}
        }

        self.drag_state = PageDragState::Idle;
        ctx.set_cursor("crosshair");
        ctx.request_redraw();
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.drag_state = PageDragState::Idle;
        self.hovered_handle = None;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        let zoom = viewport.zoom;
        let doc = ctx.document;

        // 1. Draw Active Page Highlight & Resize Handles
        if let Some(active_page) = doc.active_page() {
            let r = active_page.rect.normalize();
            let page_rect = r.to_skia();

            // Accent Outline
            let mut outline_paint = skia::Paint::default();
            outline_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
            outline_paint.set_stroke_width(2.0 / zoom);
            outline_paint.set_style(skia::PaintStyle::Stroke);
            outline_paint.set_anti_alias(true);
            canvas.draw_rect(page_rect, &outline_paint);

            // 8 Resize Handles
            for (handle, pt) in Self::get_handle_positions(active_page.rect) {
                let size = 8.0 / zoom;
                let handle_rect =
                    skia::Rect::from_xywh(pt.x - size / 2.0, pt.y - size / 2.0, size, size);

                let is_hovered = self.hovered_handle == Some(handle);

                let mut fill_paint = skia::Paint::default();
                if is_hovered {
                    fill_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
                } else {
                    fill_paint.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                }
                fill_paint.set_style(skia::PaintStyle::Fill);
                fill_paint.set_anti_alias(true);
                canvas.draw_rect(handle_rect, &fill_paint);

                let mut stroke_paint = skia::Paint::default();
                stroke_paint.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 1.0), None);
                stroke_paint.set_stroke_width(1.5 / zoom);
                stroke_paint.set_style(skia::PaintStyle::Stroke);
                stroke_paint.set_anti_alias(true);
                canvas.draw_rect(handle_rect, &stroke_paint);
            }
        }

        // 2. Draw live creation preview rectangle
        if let PageDragState::Creating {
            start_pos,
            current_pos,
        } = self.drag_state
        {
            let min_x = start_pos.x.min(current_pos.x);
            let min_y = start_pos.y.min(current_pos.y);
            let width = (start_pos.x - current_pos.x).abs();
            let height = (start_pos.y - current_pos.y).abs();
            let rect = skia::Rect::from_xywh(min_x, min_y, width, height);

            let mut create_bg = skia::Paint::default();
            create_bg.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 0.1), None);
            create_bg.set_style(skia::PaintStyle::Fill);
            create_bg.set_anti_alias(true);
            canvas.draw_rect(rect, &create_bg);

            let mut create_stroke = skia::Paint::default();
            create_stroke.set_color4f(skia::Color4f::new(0.2, 0.55, 0.95, 0.9), None);
            create_stroke.set_stroke_width(1.5 / zoom);
            create_stroke.set_style(skia::PaintStyle::Stroke);
            create_stroke.set_anti_alias(true);
            let intervals = [6.0 / zoom, 4.0 / zoom];
            create_stroke.set_path_effect(skia::dash_path_effect::new(&intervals, 0.0));
            canvas.draw_rect(rect, &create_stroke);

            // Dimension badge
            let dim_text = format!("{:.0} × {:.0} px", width, height);
            let font = skia::Font::new(crate::core::renderer::get_ui_typeface(), 11.0 / zoom);
            let (text_w, _) = font.measure_str(&dim_text, None);
            let badge_rect = skia::Rect::from_xywh(
                min_x + width / 2.0 - text_w / 2.0 - 6.0 / zoom,
                min_y + height / 2.0 - 10.0 / zoom,
                text_w + 12.0 / zoom,
                20.0 / zoom,
            );

            let mut badge_bg = skia::Paint::default();
            badge_bg.set_color4f(skia::Color4f::new(0.1, 0.1, 0.14, 0.9), None);
            badge_bg.set_style(skia::PaintStyle::Fill);
            badge_bg.set_anti_alias(true);
            canvas.draw_round_rect(badge_rect, 4.0 / zoom, 4.0 / zoom, &badge_bg);

            let mut badge_text = skia::Paint::default();
            badge_text.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
            badge_text.set_anti_alias(true);
            canvas.draw_str(
                &dim_text,
                skia::Point::new(
                    min_x + width / 2.0 - text_w / 2.0,
                    min_y + height / 2.0 + 4.0 / zoom,
                ),
                &font,
                &badge_text,
            );
        }
    }
}
