use skia_safe as skia;

use crate::core::{
    Element, ElementId, KeyEvent, Point, PointerButton, PointerEvent, Rect, TextElement, Viewport,
};
use crate::plugins::traits::{FeaturePlugin, PluginContext, PluginRenderContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextBoxHandle {
    Left,
    Right,
    Bottom,
    BottomLeft,
    BottomRight,
}

impl TextBoxHandle {
    pub fn position(self, bounds: Rect) -> Point {
        match self {
            TextBoxHandle::Left => Point::new(bounds.x, bounds.y + bounds.height / 2.0).round(),
            TextBoxHandle::Right => {
                Point::new(bounds.x + bounds.width, bounds.y + bounds.height / 2.0).round()
            }
            TextBoxHandle::Bottom => {
                Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height).round()
            }
            TextBoxHandle::BottomLeft => Point::new(bounds.x, bounds.y + bounds.height).round(),
            TextBoxHandle::BottomRight => {
                Point::new(bounds.x + bounds.width, bounds.y + bounds.height).round()
            }
        }
    }
}

pub fn hit_text_box_handle(bounds: Rect, p: Point, zoom: f32) -> Option<TextBoxHandle> {
    let hit_r = 18.0 / zoom.max(0.001);
    let handles = [
        TextBoxHandle::Right,
        TextBoxHandle::Bottom,
        TextBoxHandle::BottomRight,
        TextBoxHandle::Left,
        TextBoxHandle::BottomLeft,
    ];
    for h in handles {
        let pos = h.position(bounds);
        if p.distance_to(pos) <= hit_r {
            return Some(h);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextToolState {
    Idle,
    Creating {
        start: Point,
        current: Point,
    },
    DraggingBoxHandle {
        handle: TextBoxHandle,
        elem_id: ElementId,
        start_world: Point,
        init_box_w: f32,
        init_box_h: f32,
        init_pos_x: f32,
    },
}

pub struct TextFeature {
    editing_id: Option<ElementId>,
    cursor_pos: usize,
    select_all: bool,
    state: TextToolState,
    default_size: f32,
}

impl Default for TextFeature {
    fn default() -> Self {
        Self {
            editing_id: None,
            cursor_pos: 0,
            select_all: false,
            state: TextToolState::Idle,
            default_size: 32.0,
        }
    }
}

impl TextFeature {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate character index from world point click inside text element
    pub fn calc_cursor_from_point(t: &TextElement, p: Point) -> usize {
        let font = t.create_skia_font();
        let lines = t.layout_lines();
        let line_spacing = t.font_size * t.line_height.max(0.5);

        if lines.is_empty() || t.text.is_empty() {
            return 0;
        }

        let rel_y = (p.y - (t.position.y - t.font_size)).max(0.0);
        let target_line_idx = ((rel_y / line_spacing).floor() as usize).min(lines.len() - 1);

        let mut cumulative_pos = 0;
        for (i, line) in lines.iter().enumerate() {
            if i == target_line_idx {
                let mut best_col = 0;
                let mut best_dist = f32::MAX;
                let rel_x = p.x - t.position.x;

                for char_idx in 0..=line.len() {
                    let (substr_w, _) = font.measure_str(&line[..char_idx], None);
                    let dist = (substr_w - rel_x).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_col = char_idx;
                    }
                }
                return (cumulative_pos + best_col).min(t.text.len());
            }
            cumulative_pos += line.len();
            if cumulative_pos < t.text.len()
                && t.text.as_bytes().get(cumulative_pos) == Some(&b'\n')
            {
                cumulative_pos += 1;
            }
        }

        t.text.len()
    }
}

impl FeaturePlugin for TextFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        ctx.set_cursor("tool:text");
    }

    fn is_editing(&self) -> bool {
        self.editing_id.is_some()
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        // 1. Check if clicking on a text box resize handle
        let target_elem_id = self.editing_id.or_else(|| {
            if ctx.document.selected_ids.len() == 1 {
                let id = *ctx.document.selected_ids.iter().next().unwrap();
                if ctx
                    .document
                    .elements
                    .iter()
                    .any(|e| e.id() == id && matches!(e, Element::Text(_)))
                {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some(elem_id) = target_elem_id {
            if let Some(Element::Text(t)) = ctx.document.elements.iter().find(|e| e.id() == elem_id)
            {
                let bounds = t.bounds();
                if let Some(h) = hit_text_box_handle(bounds, event.world_pos, ctx.viewport.zoom) {
                    self.state = TextToolState::DraggingBoxHandle {
                        handle: h,
                        elem_id,
                        start_world: event.world_pos,
                        init_box_w: t.box_width.unwrap_or(bounds.width),
                        init_box_h: t.box_height.unwrap_or(bounds.height),
                        init_pos_x: t.position.x,
                    };
                    let cursor = match h {
                        TextBoxHandle::Left | TextBoxHandle::Right => "ew-resize",
                        TextBoxHandle::Bottom => "ns-resize",
                        TextBoxHandle::BottomRight => "nwse-resize",
                        TextBoxHandle::BottomLeft => "nesw-resize",
                    };
                    ctx.set_cursor(cursor);
                    ctx.request_redraw();
                    return;
                }
            }
        }

        // 2. Check if clicking an existing text element to edit it or reposition cursor
        if let Some(hit_id) = ctx.document.hit_test(event.world_pos) {
            let cursor_pos = if let Some(Element::Text(t)) =
                ctx.document.elements.iter().find(|e| e.id() == hit_id)
            {
                Some(Self::calc_cursor_from_point(t, event.world_pos))
            } else {
                None
            };
            if let Some(pos) = cursor_pos {
                ctx.document.select(hit_id, false);
                self.editing_id = Some(hit_id);
                self.select_all = false;
                self.cursor_pos = pos;
                self.state = TextToolState::Idle;
                ctx.set_cursor("text");
                ctx.request_redraw();
                return;
            }
        }

        // 3. Clicked on empty space: start drag for text box or click for point text
        self.editing_id = None;
        self.select_all = false;
        self.state = TextToolState::Creating {
            start: event.world_pos,
            current: event.world_pos,
        };
        ctx.document.deselect_all();
        ctx.set_cursor("tool:text");
        ctx.request_redraw();
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        match &mut self.state {
            TextToolState::DraggingBoxHandle {
                handle,
                elem_id,
                start_world,
                init_box_w,
                init_box_h,
                init_pos_x,
            } => {
                let dx = event.world_pos.x - start_world.x;
                let dy = event.world_pos.y - start_world.y;
                let target_id = *elem_id;
                let h = *handle;
                let w0 = *init_box_w;
                let h0 = *init_box_h;
                let x0 = *init_pos_x;

                if let Some(Element::Text(t)) = ctx
                    .document
                    .elements
                    .iter_mut()
                    .find(|e| e.id() == target_id)
                {
                    match h {
                        TextBoxHandle::Right => {
                            t.box_width = Some((w0 + dx).max(20.0));
                        }
                        TextBoxHandle::Bottom => {
                            t.box_height = Some((h0 + dy).max(t.font_size * 1.2));
                        }
                        TextBoxHandle::BottomRight => {
                            t.box_width = Some((w0 + dx).max(20.0));
                            t.box_height = Some((h0 + dy).max(t.font_size * 1.2));
                        }
                        TextBoxHandle::Left => {
                            let new_w = (w0 - dx).max(20.0);
                            let actual_shift = w0 - new_w;
                            t.position.x = x0 + actual_shift;
                            t.box_width = Some(new_w);
                        }
                        TextBoxHandle::BottomLeft => {
                            let new_w = (w0 - dx).max(20.0);
                            let actual_shift = w0 - new_w;
                            t.position.x = x0 + actual_shift;
                            t.box_width = Some(new_w);
                            t.box_height = Some((h0 + dy).max(t.font_size * 1.2));
                        }
                    }
                }

                let cursor = match h {
                    TextBoxHandle::Left | TextBoxHandle::Right => "ew-resize",
                    TextBoxHandle::Bottom => "ns-resize",
                    TextBoxHandle::BottomRight => "nwse-resize",
                    TextBoxHandle::BottomLeft => "nesw-resize",
                };
                ctx.set_cursor(cursor);
                ctx.request_redraw();
            }
            TextToolState::Creating { current, .. } => {
                *current = event.world_pos;
                ctx.request_redraw();
            }
            TextToolState::Idle => {
                let target_elem_id = self.editing_id.or_else(|| {
                    if ctx.document.selected_ids.len() == 1 {
                        let id = *ctx.document.selected_ids.iter().next().unwrap();
                        if ctx
                            .document
                            .elements
                            .iter()
                            .any(|e| e.id() == id && matches!(e, Element::Text(_)))
                        {
                            Some(id)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                if let Some(elem_id) = target_elem_id {
                    if let Some(Element::Text(t)) =
                        ctx.document.elements.iter().find(|e| e.id() == elem_id)
                    {
                        let bounds = t.bounds();
                        if let Some(h) =
                            hit_text_box_handle(bounds, event.world_pos, ctx.viewport.zoom)
                        {
                            let cursor = match h {
                                TextBoxHandle::Left | TextBoxHandle::Right => "ew-resize",
                                TextBoxHandle::Bottom => "ns-resize",
                                TextBoxHandle::BottomRight => "nwse-resize",
                                TextBoxHandle::BottomLeft => "nesw-resize",
                            };
                            ctx.set_cursor(cursor);
                            return;
                        }
                    }
                }

                let is_hovering_text = ctx.document.elements.iter().any(|e| {
                    if let Element::Text(t) = e {
                        t.bounds().expand(4.0).contains(event.world_pos)
                    } else {
                        false
                    }
                });

                if is_hovering_text {
                    ctx.set_cursor("text");
                } else {
                    ctx.set_cursor("tool:text");
                }
            }
        }
    }

    fn on_pointer_up(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        match self.state {
            TextToolState::DraggingBoxHandle { .. } => {
                ctx.document.snapshot();
                self.state = TextToolState::Idle;
                ctx.request_redraw();
            }
            TextToolState::Creating { start, current } => {
                let end = current;
                let rect = Rect::from_points(start, end).normalize().round();

                ctx.document.snapshot();
                let new_elem = if rect.width > 20.0 && rect.height > 15.0 {
                    // Dragged a bounding text box - starts completely empty, no placeholder!
                    TextElement::with_box(
                        Point::new(rect.x, rect.y),
                        String::new(),
                        self.default_size,
                        ctx.active_fill_color,
                        rect.width,
                        rect.height,
                    )
                } else {
                    // Clicked point text - starts completely empty, no placeholder!
                    TextElement::new(
                        start,
                        String::new(),
                        self.default_size,
                        ctx.active_fill_color,
                    )
                };

                let new_id = new_elem.id;
                ctx.document.add_element(Element::Text(new_elem));
                ctx.document.select(new_id, false);

                self.editing_id = Some(new_id);
                self.select_all = false;
                self.cursor_pos = 0;
                self.state = TextToolState::Idle;

                ctx.set_cursor("text");
                ctx.request_redraw();
            }
            TextToolState::Idle => {}
        }
    }

    fn on_double_click(&mut self, ctx: &mut PluginContext, event: &PointerEvent) -> bool {
        // 1. Check if double-clicking a text box handle: resets box size to dynamic (point text) mode!
        let hit_text_handle_id = ctx.document.elements.iter().find_map(|el| {
            if let Element::Text(t) = el {
                if hit_text_box_handle(t.bounds(), event.world_pos, ctx.viewport.zoom).is_some() {
                    return Some(t.id);
                }
            }
            None
        });

        if let Some(id) = hit_text_handle_id {
            ctx.document.snapshot();
            if let Some(Element::Text(t)) =
                ctx.document.elements.iter_mut().find(|el| el.id() == id)
            {
                t.box_width = None;
                t.box_height = None;
            }
            ctx.request_redraw();
            return true;
        }

        // 2. Double-clicking on text body: select all text
        if let Some(hit_id) = ctx.document.hit_test(event.world_pos) {
            let text_len = if let Some(Element::Text(t)) =
                ctx.document.elements.iter().find(|e| e.id() == hit_id)
            {
                Some(t.text.len())
            } else {
                None
            };
            if let Some(len) = text_len {
                ctx.document.select(hit_id, false);
                self.editing_id = Some(hit_id);
                self.select_all = true;
                self.cursor_pos = len;
                self.state = TextToolState::Idle;
                ctx.set_cursor("text");
                ctx.request_redraw();
                return true;
            }
        }
        false
    }

    fn on_key_down(&mut self, ctx: &mut PluginContext, event: &KeyEvent) -> bool {
        let editing_id = match self.editing_id {
            Some(id) => id,
            None => return false,
        };

        let key = event.key;

        // Escape: Commit text and stop editing
        if key == gtk4::gdk::Key::Escape {
            self.editing_id = None;
            self.select_all = false;
            self.state = TextToolState::Idle;
            ctx.set_cursor("default");
            ctx.request_redraw();
            return true;
        }

        // Ctrl + A: Select All text
        if event.ctrl_pressed && (key == gtk4::gdk::Key::a || key == gtk4::gdk::Key::A) {
            self.select_all = true;
            ctx.request_redraw();
            return true;
        }

        // Return / Enter: Insert newline
        if key == gtk4::gdk::Key::Return || key == gtk4::gdk::Key::KP_Enter {
            for el in &mut ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == editing_id {
                        if self.select_all {
                            t.text = "\n".to_string();
                            self.cursor_pos = 1;
                            self.select_all = false;
                        } else {
                            let pos = self.cursor_pos.min(t.text.len());
                            t.text.insert(pos, '\n');
                            self.cursor_pos += 1;
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
            }
            return true;
        }

        // BackSpace
        if key == gtk4::gdk::Key::BackSpace {
            for el in &mut ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == editing_id {
                        if self.select_all {
                            t.text.clear();
                            self.cursor_pos = 0;
                            self.select_all = false;
                        } else if self.cursor_pos > 0 && !t.text.is_empty() {
                            let pos = (self.cursor_pos - 1).min(t.text.len() - 1);
                            t.text.remove(pos);
                            self.cursor_pos -= 1;
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
            }
            return true;
        }

        // Delete
        if key == gtk4::gdk::Key::Delete {
            for el in &mut ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == editing_id {
                        if self.select_all {
                            t.text.clear();
                            self.cursor_pos = 0;
                            self.select_all = false;
                        } else if self.cursor_pos < t.text.len() {
                            t.text.remove(self.cursor_pos);
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
            }
            return true;
        }

        // Arrow Left
        if key == gtk4::gdk::Key::Left {
            self.select_all = false;
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
            }
            ctx.request_redraw();
            return true;
        }

        // Arrow Right
        if key == gtk4::gdk::Key::Right {
            self.select_all = false;
            for el in &ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == editing_id && self.cursor_pos < t.text.len() {
                        self.cursor_pos += 1;
                    }
                }
            }
            ctx.request_redraw();
            return true;
        }

        // Arrow Up: Move to previous line
        if key == gtk4::gdk::Key::Up {
            self.select_all = false;
            for el in &ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == editing_id {
                        let lines = t.layout_lines();
                        let mut cur_line = 0;
                        let mut cur_col = 0;
                        let mut cum = 0;
                        for (i, line) in lines.iter().enumerate() {
                            if self.cursor_pos <= cum + line.len() {
                                cur_line = i;
                                cur_col = self.cursor_pos - cum;
                                break;
                            }
                            cum += line.len() + 1;
                        }
                        if cur_line > 0 {
                            let prev_line = &lines[cur_line - 1];
                            let prev_col = cur_col.min(prev_line.len());
                            let mut new_pos = 0;
                            for j in 0..(cur_line - 1) {
                                new_pos += lines[j].len() + 1;
                            }
                            new_pos += prev_col;
                            self.cursor_pos = new_pos.min(t.text.len());
                        } else {
                            self.cursor_pos = 0;
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
            }
            return true;
        }

        // Arrow Down: Move to next line
        if key == gtk4::gdk::Key::Down {
            self.select_all = false;
            for el in &ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == editing_id {
                        let lines = t.layout_lines();
                        let mut cur_line = 0;
                        let mut cur_col = 0;
                        let mut cum = 0;
                        for (i, line) in lines.iter().enumerate() {
                            if self.cursor_pos <= cum + line.len() {
                                cur_line = i;
                                cur_col = self.cursor_pos - cum;
                                break;
                            }
                            cum += line.len() + 1;
                        }
                        if cur_line + 1 < lines.len() {
                            let next_line = &lines[cur_line + 1];
                            let next_col = cur_col.min(next_line.len());
                            let mut new_pos = 0;
                            for j in 0..=cur_line {
                                new_pos += lines[j].len() + 1;
                            }
                            new_pos += next_col;
                            self.cursor_pos = new_pos.min(t.text.len());
                        } else {
                            self.cursor_pos = t.text.len();
                        }
                        ctx.request_redraw();
                        return true;
                    }
                }
            }
            return true;
        }

        // Home: start of line
        if key == gtk4::gdk::Key::Home {
            self.select_all = false;
            self.cursor_pos = 0;
            ctx.request_redraw();
            return true;
        }

        // End: end of line
        if key == gtk4::gdk::Key::End {
            self.select_all = false;
            for el in &ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == editing_id {
                        self.cursor_pos = t.text.len();
                    }
                }
            }
            ctx.request_redraw();
            return true;
        }

        // Unicode character insertion
        if let Some(ch) = key.to_unicode() {
            if !ch.is_control() {
                for el in &mut ctx.document.elements {
                    if let Element::Text(t) = el {
                        if t.id == editing_id {
                            if self.select_all {
                                t.text = ch.to_string();
                                self.cursor_pos = ch.len_utf8();
                                self.select_all = false;
                            } else {
                                let pos = self.cursor_pos.min(t.text.len());
                                t.text.insert(pos, ch);
                                self.cursor_pos += ch.len_utf8();
                            }
                            ctx.request_redraw();
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    fn on_cancel(&mut self, ctx: &mut PluginContext) {
        self.editing_id = None;
        self.select_all = false;
        self.state = TextToolState::Idle;
        ctx.set_cursor("default");
        ctx.request_redraw();
    }

    fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        // 1. Dragging preview box during creation
        if let TextToolState::Creating { start, current } = self.state {
            let r = Rect::from_points(start, current).normalize().round();

            let mut fill_paint = skia::Paint::default();
            fill_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.08), None);
            fill_paint.set_style(skia::PaintStyle::Fill);

            let mut stroke_paint = skia::Paint::default();
            stroke_paint.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.9), None);
            stroke_paint.set_style(skia::PaintStyle::Stroke);
            stroke_paint.set_stroke_width((1.2 / viewport.zoom).max(1.0));
            stroke_paint.set_anti_alias(true);

            canvas.draw_rect(r.to_skia(), &fill_paint);
            canvas.draw_rect(r.to_skia(), &stroke_paint);
            return;
        }

        // 2. Active text editing box and handles
        let target_elem_id = self.editing_id.or_else(|| {
            if ctx.document.selected_ids.len() == 1 {
                let id = *ctx.document.selected_ids.iter().next().unwrap();
                if ctx
                    .document
                    .elements
                    .iter()
                    .any(|e| e.id() == id && matches!(e, Element::Text(_)))
                {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some(elem_id) = target_elem_id {
            for el in &ctx.document.elements {
                if let Element::Text(t) = el {
                    if t.id == elem_id {
                        let bounds = t.bounds();

                        // Highlight selection box when select_all is active
                        if self.select_all {
                            let mut sel_paint = skia::Paint::default();
                            sel_paint
                                .set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.28), None);
                            sel_paint.set_style(skia::PaintStyle::Fill);
                            canvas.draw_rect(bounds.to_skia(), &sel_paint);
                        }

                        // Outline bounding box
                        let mut outline = skia::Paint::default();
                        outline.set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 0.95), None);
                        outline.set_style(skia::PaintStyle::Stroke);
                        outline.set_stroke_width((1.2 / viewport.zoom).max(1.0));
                        outline.set_anti_alias(true);
                        canvas.draw_rect(bounds.to_skia(), &outline);

                        // Draw box resize handles (Right, Bottom, BottomRight)
                        let handle_r = (4.5 / viewport.zoom).clamp(3.5, 6.5);
                        let mut handle_fill = skia::Paint::default();
                        handle_fill.set_color4f(skia::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                        handle_fill.set_style(skia::PaintStyle::Fill);
                        handle_fill.set_anti_alias(true);

                        let mut handle_stroke = skia::Paint::default();
                        handle_stroke
                            .set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 1.0), None);
                        handle_stroke.set_style(skia::PaintStyle::Stroke);
                        handle_stroke.set_stroke_width((1.5 / viewport.zoom).max(1.0));
                        handle_stroke.set_anti_alias(true);

                        let handles = [
                            TextBoxHandle::Left,
                            TextBoxHandle::Right,
                            TextBoxHandle::Bottom,
                            TextBoxHandle::BottomLeft,
                            TextBoxHandle::BottomRight,
                        ];
                        for h in handles {
                            let pos = h.position(bounds);
                            canvas.draw_circle(pos.to_skia(), handle_r, &handle_fill);
                            canvas.draw_circle(pos.to_skia(), handle_r, &handle_stroke);
                        }

                        // Draw insertion cursor when in active editing mode
                        if self.editing_id.is_some() && !self.select_all {
                            let font = t.create_skia_font();
                            let lines = t.layout_lines();
                            let line_spacing = t.font_size * t.line_height.max(0.5);

                            let mut cur_line_idx = 0;
                            let mut cur_col_idx = 0;
                            let mut cum = 0;

                            for (i, line) in lines.iter().enumerate() {
                                if self.cursor_pos <= cum + line.len() {
                                    cur_line_idx = i;
                                    cur_col_idx = self.cursor_pos - cum;
                                    break;
                                }
                                cum += line.len();
                                if cum < t.text.len() && t.text.as_bytes().get(cum) == Some(&b'\n')
                                {
                                    cum += 1;
                                }
                            }

                            let line_str =
                                lines.get(cur_line_idx).map(|s| s.as_str()).unwrap_or("");
                            let clamped_col = cur_col_idx.min(line_str.len());
                            let (substr_w, _) = font.measure_str(&line_str[..clamped_col], None);

                            let cur_x = t.position.x + substr_w;
                            let cur_y_top = t.position.y + (cur_line_idx as f32) * line_spacing
                                - t.font_size * 0.88;
                            let cur_y_bot = cur_y_top + t.font_size * 1.08;

                            let mut cursor_paint = skia::Paint::default();
                            cursor_paint
                                .set_color4f(skia::Color4f::new(0.208, 0.518, 0.894, 1.0), None);
                            cursor_paint.set_style(skia::PaintStyle::Stroke);
                            cursor_paint.set_stroke_width((2.0 / viewport.zoom).max(1.5));
                            cursor_paint.set_anti_alias(true);

                            canvas.draw_line(
                                skia::Point::new(cur_x, cur_y_top),
                                skia::Point::new(cur_x, cur_y_bot),
                                &cursor_paint,
                            );
                        }
                    }
                }
            }
        }
    }
}
