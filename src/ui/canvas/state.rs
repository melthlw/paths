use gtk4::gdk;
use std::collections::HashSet;

use crate::core::{
    Color, Document, Element, GridConfig, Guide, GuideOrientation, Point, PointerButton,
    PointerEvent, RulerConfig, SkiaRenderer, SnapConfig, SnapEngine, SnapGuide, Viewport,
};
use crate::plugins::{PluginContext, PluginManager};

pub struct CanvasState {
    pub document: Document,
    pub viewport: Viewport,
    pub grid_config: GridConfig,
    pub snap_config: SnapConfig,
    pub ruler_config: RulerConfig,
    pub dragging_guide: Option<(GuideOrientation, f32)>,
    pub dragging_guide_id: Option<u64>,
    pub hovered_guide_id: Option<u64>,
    pub dragging_corner_origin: bool,
    pub corner_drag_current: Option<Point>,
    pub active_snap_guides: Vec<SnapGuide>,
    pub plugin_manager: PluginManager,
    pub renderer: SkiaRenderer,
    pub active_fill_color: Color,
    pub active_stroke_color: Option<Color>,
    pub active_stroke_width: f32,
    pub widget_size: (f32, f32),
    pub cursor_pos: Point,
    pub active_cursor: Option<&'static str>,
    pub is_space_down: bool,
    pub is_panning: bool,
    pub pan_last_pos: Point,
    pub on_tool_change: Option<Box<dyn Fn(&'static str)>>,
    pub on_status_change: Option<
        Box<
            dyn Fn(
                &str,
                f32,
                usize,
                Option<crate::core::Rect>,
                (Option<Color>, Option<Color>, f32),
                bool,
                Option<(String, u32, f32, f32, crate::core::TextAlign, f32, f32)>,
                bool,
                bool,
                bool,
                bool,
                Option<crate::core::ShapeOrigin>,
                Option<(String, f32, f32, f32, f32, usize)>,
                Vec<crate::core::layer::LayerItemInfo>,
                bool,
                bool,
                Option<(Vec<crate::core::FillLayer>, Vec<crate::core::StrokeLayer>)>,
                Option<(crate::core::BlendMode, f32, f32)>,
                Option<Point>,
                Vec<Option<Color>>,
            ),
        >,
    >,
    pub shortcuts: crate::core::ShortcutManager,
    pub render_options: crate::core::RenderOptions,
    pub path_editor_config: crate::core::PathEditorConfig,
    pub transform_options: crate::core::TransformOptions,
    pub is_universal_selecting: bool,
    is_switching_tool: bool,
    pub current_file_path: Option<std::path::PathBuf>,
    pub is_dirty: bool,
    pub has_centered_initial_page: bool,
    pub on_file_state_changed: Option<Box<dyn Fn(Option<&std::path::Path>, bool)>>,
    pub on_file_dialog_request: Option<Box<dyn Fn(crate::core::ShortcutAction)>>,
}

impl CanvasState {
    pub fn new() -> Self {
        Self::with_doc(Document::new())
    }

    pub fn with_doc(doc: Document) -> Self {
        Self {
            document: doc,
            viewport: Viewport::default(),
            grid_config: GridConfig::default(),
            snap_config: SnapConfig::default(),
            ruler_config: RulerConfig::default(),
            dragging_guide: None,
            dragging_guide_id: None,
            hovered_guide_id: None,
            dragging_corner_origin: false,
            corner_drag_current: None,
            active_snap_guides: Vec::new(),
            plugin_manager: PluginManager::new(),
            renderer: SkiaRenderer::new(),
            render_options: crate::core::RenderOptions::default(),
            path_editor_config: crate::core::PathEditorConfig::default(),
            transform_options: crate::core::TransformOptions::default(),
            is_universal_selecting: false,
            active_fill_color: Color::BLACK,
            active_stroke_color: None,
            active_stroke_width: 2.0,
            widget_size: (800.0, 600.0),
            cursor_pos: Point::ZERO,
            active_cursor: None,
            is_space_down: false,
            is_panning: false,
            pan_last_pos: Point::ZERO,
            on_tool_change: None,
            on_status_change: None,
            shortcuts: crate::core::ShortcutManager::new(),
            is_switching_tool: false,
            current_file_path: None,
            is_dirty: false,
            has_centered_initial_page: false,
            on_file_state_changed: None,
            on_file_dialog_request: None,
        }
    }

    pub fn notify_file_state(&self) {
        if let Some(cb) = &self.on_file_state_changed {
            cb(self.current_file_path.as_deref(), self.is_dirty);
        }
    }

    pub fn mark_dirty(&mut self) {
        if !self.is_dirty {
            self.is_dirty = true;
            self.notify_file_state();
        }
    }

    pub fn notify_status(&self) {
        if let Some(cb) = &self.on_status_change {
            let bounds = self.document.selection_bounds();
            let tool_id = self.plugin_manager.active_id();
            let style = self.document.get_selected_style().unwrap_or((
                Some(self.active_fill_color),
                self.active_stroke_color,
                self.active_stroke_width,
            ));
            let is_editing_text = tool_id == "text" && self.plugin_manager.is_editing();
            let has_text_selected = self.document.elements.iter().any(|e| {
                self.document.selected_ids.contains(&e.id())
                    && matches!(e, crate::core::Element::Text(_))
            });
            let text_info = if is_editing_text || tool_id == "text" || (tool_id == "select" && has_text_selected) {
                self.document.get_selected_text_info()
            } else {
                None
            };
            let can_convert = self.document.has_non_path_selected();
            let shape_origin = self.document.get_selected_shape_origin().or_else(|| {
                self.plugin_manager
                    .active_feature()
                    .and_then(|f| f.get_shape_params())
            });
            let page_info = self.document.active_page().map(|p| {
                let r = p.rect.normalize();
                (
                    p.name.clone(),
                    r.x,
                    r.y,
                    r.width,
                    r.height,
                    self.document.pages.len(),
                )
            });
            let layers_info = self.document.get_layers_info();
            let can_undo = self.document.can_undo();
            let can_redo = self.document.can_redo();
            let fills_and_strokes = self.document.get_selected_fills_and_strokes();
            let blend_info = self.document.get_selection_blend_info();
            let node_coord = if tool_id == "path_editor" || tool_id == "path-editor" {
                if let Some(feat) = self.plugin_manager.feature_by_id("path_editor") {
                    let nodes = feat.get_selected_nodes();
                    self.document.get_selected_node_coord(&nodes)
                } else {
                    None
                }
            } else {
                None
            };
            let doc_colors = self.document.get_document_colors();
            cb(
                tool_id,
                self.viewport.zoom,
                self.document.selected_ids.len(),
                bounds,
                style,
                is_editing_text,
                text_info,
                self.grid_config.visible,
                self.ruler_config.visible,
                self.snap_config.enabled,
                can_convert,
                shape_origin,
                page_info,
                layers_info,
                can_undo,
                can_redo,
                fills_and_strokes,
                blend_info,
                node_coord,
                doc_colors,
            );
        }
    }

    pub fn set_active_tool(&mut self, tool_id: &'static str) {
        if self.is_switching_tool {
            return;
        }
        self.is_switching_tool = true;

        let widget_size = self.widget_size;
        let active_fill = self.active_fill_color;
        let active_stroke = self.active_stroke_color;
        let active_stroke_w = self.active_stroke_width;
        let mut ctx = PluginContext {
            document: &mut self.document,
            viewport: &mut self.viewport,
            grid_config: &self.grid_config,
            snap_config: &self.snap_config,
            ruler_config: &self.ruler_config,
            path_editor_config: &self.path_editor_config,
            active_snap_guides: &mut self.active_snap_guides,
            active_fill_color: active_fill,
            active_stroke_color: active_stroke,
            active_stroke_width: active_stroke_w,
            widget_size,
            needs_redraw: false,
            cursor_name: None,
        };

        self.plugin_manager.set_active_tool(tool_id, &mut ctx);

        self.active_cursor = ctx.cursor_name;

        if let Some(cb) = &self.on_tool_change {
            cb(tool_id);
        }
        self.notify_status();
        self.is_switching_tool = false;
    }

    pub fn pointer_down(
        &mut self,
        screen_pos: Point,
        button: PointerButton,
        shift_pressed: bool,
        ctrl_pressed: bool,
        alt_pressed: bool,
    ) -> (bool, Option<&'static str>) {
        self.cursor_pos = screen_pos;

        if (self.is_space_down || button == PointerButton::Middle)
            && button != PointerButton::Secondary
        {
            self.is_panning = true;
            self.pan_last_pos = screen_pos;
            self.notify_status();
            return (true, Some("grabbing"));
        }

        // 1. Click on Rulers
        if self.ruler_config.visible {
            let t = self.ruler_config.thickness;
            if screen_pos.x <= t && screen_pos.y <= t {
                if button == PointerButton::Primary {
                    self.dragging_corner_origin = true;
                    self.corner_drag_current = Some(Point::ZERO);
                    return (true, Some("crosshair"));
                }
            } else if screen_pos.x > t && screen_pos.y <= t {
                if button == PointerButton::Primary {
                    let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
                    self.dragging_guide = Some((GuideOrientation::Horizontal, world_pos.y));
                    self.dragging_guide_id = None;
                    return (true, Some("ns-resize"));
                }
            } else if screen_pos.y > t && screen_pos.x <= t {
                if button == PointerButton::Primary {
                    let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
                    self.dragging_guide = Some((GuideOrientation::Vertical, world_pos.x));
                    self.dragging_guide_id = None;
                    return (true, Some("ew-resize"));
                }
            }
        }

        let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
        let event = PointerEvent {
            screen_pos,
            world_pos,
            button: Some(button),
            shift_pressed,
            ctrl_pressed,
            alt_pressed,
        };

        let widget_size = self.widget_size;
        let active_fill = self.active_fill_color;
        let active_stroke = self.active_stroke_color;
        let active_stroke_w = self.active_stroke_width;

        // 2. Click on existing guides in canvas
        if self.ruler_config.guides_visible {
            let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
            if let Some(guide) = self
                .document
                .find_guide_near(world_pos, 8.0 / self.viewport.zoom.max(0.1))
                .cloned()
            {
                if button == PointerButton::Primary {
                    self.dragging_guide = Some((guide.orientation, guide.position));
                    self.dragging_guide_id = Some(guide.id);
                    self.hovered_guide_id = Some(guide.id);
                    let cursor = match guide.orientation {
                        GuideOrientation::Horizontal => "ns-resize",
                        GuideOrientation::Vertical => "ew-resize",
                    };
                    return (true, Some(cursor));
                } else if button == PointerButton::Secondary {
                    self.document.remove_guide(guide.id);
                    self.hovered_guide_id = None;
                    self.notify_status();
                    return (true, Some("default"));
                }
            }
        }

        let is_selecting_active = self.plugin_manager.active_id() == "select";
        let hit_handle = if !is_selecting_active {
            self.document
                .selection_bounds()
                .and_then(|b| crate::core::hit_transform_handle(b, world_pos, self.viewport.zoom))
        } else {
            None
        };

        if (ctrl_pressed || hit_handle.is_some()) && !is_selecting_active {
            self.is_universal_selecting = true;
        } else {
            self.is_universal_selecting = false;
        }

        let (redraw, cursor) = {
            let mut ctx = PluginContext {
                document: &mut self.document,
                viewport: &mut self.viewport,
                grid_config: &self.grid_config,
                snap_config: &self.snap_config,
                ruler_config: &self.ruler_config,
                path_editor_config: &self.path_editor_config,
                active_snap_guides: &mut self.active_snap_guides,
                active_fill_color: active_fill,
                active_stroke_color: active_stroke,
                active_stroke_width: active_stroke_w,
                widget_size,
                needs_redraw: false,
                cursor_name: None,
            };

            if self.is_universal_selecting {
                if let Some(feat) = self.plugin_manager.feature_by_id_mut("select") {
                    feat.on_pointer_down(&mut ctx, &event);
                }
            } else if let Some(feat) = self.plugin_manager.active_feature_mut() {
                feat.on_pointer_down(&mut ctx, &event);
            }

            (ctx.needs_redraw, ctx.cursor_name)
        };

        self.notify_status();
        (redraw, cursor)
    }

    pub fn pointer_move(
        &mut self,
        screen_pos: Point,
        shift_pressed: bool,
        ctrl_pressed: bool,
        alt_pressed: bool,
    ) -> (bool, Option<&'static str>) {
        self.cursor_pos = screen_pos;

        if self.is_panning {
            let dx = screen_pos.x - self.pan_last_pos.x;
            let dy = screen_pos.y - self.pan_last_pos.y;
            self.pan_last_pos = screen_pos;
            self.viewport.pan_by(dx, dy);
            return (true, Some("grabbing"));
        }

        // 1. Dragging Corner Origin
        if self.dragging_corner_origin {
            let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
            let exclude_empty = HashSet::new();
            let snap = SnapEngine::snap_point(
                world_pos,
                &self.document,
                &self.viewport,
                &self.snap_config,
                &self.grid_config,
                &self.ruler_config,
                &exclude_empty,
            );
            self.corner_drag_current = Some(snap.point);
            return (true, Some("crosshair"));
        }

        // 2. Dragging a guide (from ruler or existing)
        if let Some((orientation, _)) = self.dragging_guide {
            let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
            let exclude_empty = HashSet::new();
            let snap = SnapEngine::snap_point(
                world_pos,
                &self.document,
                &self.viewport,
                &self.snap_config,
                &self.grid_config,
                &self.ruler_config,
                &exclude_empty,
            );
            let new_pos = match orientation {
                GuideOrientation::Horizontal => snap.point.y,
                GuideOrientation::Vertical => snap.point.x,
            };
            self.dragging_guide = Some((orientation, new_pos));

            if let Some(id) = self.dragging_guide_id {
                self.document.update_guide_position(id, new_pos);
            }
            self.notify_status();

            let cursor = match orientation {
                GuideOrientation::Horizontal => "ns-resize",
                GuideOrientation::Vertical => "ew-resize",
            };
            return (true, Some(cursor));
        }

        // 3. Hovering over rulers
        if self.ruler_config.visible {
            let t = self.ruler_config.thickness;
            if screen_pos.x <= t && screen_pos.y <= t {
                return (false, Some("crosshair"));
            }
            if screen_pos.x > t && screen_pos.y <= t {
                return (false, Some("ns-resize"));
            }
            if screen_pos.y > t && screen_pos.x <= t {
                return (false, Some("ew-resize"));
            }
        }

        // 4. Hovering over guides on canvas
        let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
        if self.ruler_config.guides_visible {
            if let Some(guide) = self
                .document
                .find_guide_near(world_pos, 8.0 / self.viewport.zoom.max(0.1))
            {
                let guide_id = guide.id;
                let orientation = guide.orientation;
                let prev_hover = self.hovered_guide_id;
                self.hovered_guide_id = Some(guide_id);
                let cursor = match orientation {
                    GuideOrientation::Horizontal => "ns-resize",
                    GuideOrientation::Vertical => "ew-resize",
                };
                let needs_redraw = prev_hover != Some(guide_id);
                return (needs_redraw, Some(cursor));
            } else if self.hovered_guide_id.is_some() {
                self.hovered_guide_id = None;
            }
        }

        let event = PointerEvent {
            screen_pos,
            world_pos,
            button: None,
            shift_pressed,
            ctrl_pressed,
            alt_pressed,
        };

        let widget_size = self.widget_size;
        let active_fill = self.active_fill_color;
        let active_stroke = self.active_stroke_color;
        let active_stroke_w = self.active_stroke_width;

        let (redraw, cursor) = {
            let mut ctx = PluginContext {
                document: &mut self.document,
                viewport: &mut self.viewport,
                grid_config: &self.grid_config,
                snap_config: &self.snap_config,
                ruler_config: &self.ruler_config,
                path_editor_config: &self.path_editor_config,
                active_snap_guides: &mut self.active_snap_guides,
                active_fill_color: active_fill,
                active_stroke_color: active_stroke,
                active_stroke_width: active_stroke_w,
                widget_size,
                needs_redraw: false,
                cursor_name: None,
            };

            if self.is_universal_selecting {
                if let Some(feat) = self.plugin_manager.feature_by_id_mut("select") {
                    feat.on_pointer_move(&mut ctx, &event);
                }
            } else if let Some(feat) = self.plugin_manager.active_feature_mut() {
                feat.on_pointer_move(&mut ctx, &event);
            }

            (ctx.needs_redraw, ctx.cursor_name)
        };
        (redraw, cursor)
    }

    pub fn pointer_up(
        &mut self,
        screen_pos: Point,
        button: PointerButton,
        shift_pressed: bool,
        ctrl_pressed: bool,
        alt_pressed: bool,
    ) -> (bool, Option<&'static str>) {
        if self.is_panning {
            self.is_panning = false;
            let cursor = if self.is_space_down {
                Some("grab")
            } else {
                None
            };
            self.notify_status();
            return (true, cursor);
        }

        // 1. Release Corner Origin Drag
        if self.dragging_corner_origin {
            self.dragging_corner_origin = false;
            let t = self.ruler_config.thickness;
            if let Some(drag_pt) = self.corner_drag_current.take() {
                if screen_pos.x > t || screen_pos.y > t {
                    self.ruler_config.custom_origin = Some(drag_pt);
                }
            }
            self.notify_status();
            return (true, Some("default"));
        }

        // 2. Release guide drag
        if let Some((orientation, pos)) = self.dragging_guide.take() {
            let t = self.ruler_config.thickness;
            let dropped_on_canvas = match orientation {
                GuideOrientation::Horizontal => screen_pos.y > t,
                GuideOrientation::Vertical => screen_pos.x > t,
            };

            if dropped_on_canvas {
                if let Some(id) = self.dragging_guide_id.take() {
                    self.document.update_guide_position(id, pos);
                } else {
                    self.document.add_guide(Guide::new(orientation, pos));
                }
            } else if let Some(id) = self.dragging_guide_id.take() {
                self.document.remove_guide(id);
            }
            self.notify_status();
            return (true, Some("default"));
        }

        let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);
        let event = PointerEvent {
            screen_pos,
            world_pos,
            button: Some(button),
            shift_pressed,
            ctrl_pressed,
            alt_pressed,
        };

        let widget_size = self.widget_size;
        let active_fill = self.active_fill_color;
        let active_stroke = self.active_stroke_color;
        let active_stroke_w = self.active_stroke_width;

        let (redraw, cursor) = {
            let mut ctx = PluginContext {
                document: &mut self.document,
                viewport: &mut self.viewport,
                grid_config: &self.grid_config,
                snap_config: &self.snap_config,
                ruler_config: &self.ruler_config,
                path_editor_config: &self.path_editor_config,
                active_snap_guides: &mut self.active_snap_guides,
                active_fill_color: active_fill,
                active_stroke_color: active_stroke,
                active_stroke_width: active_stroke_w,
                widget_size,
                needs_redraw: false,
                cursor_name: None,
            };

            if self.is_universal_selecting {
                if let Some(feat) = self.plugin_manager.feature_by_id_mut("select") {
                    feat.on_pointer_up(&mut ctx, &event);
                }
                self.is_universal_selecting = false;
            } else if let Some(feat) = self.plugin_manager.active_feature_mut() {
                feat.on_pointer_up(&mut ctx, &event);
            }

            (ctx.needs_redraw, ctx.cursor_name)
        };

        self.notify_status();
        (redraw, cursor)
    }

    pub fn double_click(&mut self, screen_pos: Point) -> (bool, Option<&'static str>) {
        if self.ruler_config.visible {
            let t = self.ruler_config.thickness;
            if screen_pos.x <= t && screen_pos.y <= t {
                self.ruler_config.reset_origin();
                self.notify_status();
                return (true, None);
            }
        }

        let world_pos = self.viewport.screen_to_world(screen_pos, self.widget_size);

        let hit_text_handle_id = self.document.elements.iter().find_map(|el| {
            if let Element::Text(t) = el {
                if crate::plugins::features::text::hit_text_box_handle(
                    t.bounds(),
                    world_pos,
                    self.viewport.zoom,
                )
                .is_some()
                {
                    return Some(t.id);
                }
            }
            None
        });

        if let Some(id) = hit_text_handle_id {
            self.document.snapshot();
            if let Some(Element::Text(t)) =
                self.document.elements.iter_mut().find(|el| el.id() == id)
            {
                t.box_width = None;
                t.box_height = None;
            }
            self.notify_status();
            return (true, None);
        }

        if let Some(hit_id) = self.document.hit_test(world_pos) {
            let target_tool: Option<&'static str> = self
                .document
                .elements
                .iter()
                .find(|el| el.id() == hit_id)
                .and_then(|el| match el {
                    Element::Text(_) => Some("text"),
                    Element::Rect(_) => Some("rectangle"),
                    Element::Brush(_) => Some("brush"),
                    Element::Group(_) => Some("select"),
                    Element::Image(_) => Some("select"),
                    Element::Clone(_) => Some("select"),
                    Element::Path(p) => match &p.shape_origin {
                        Some(crate::core::ShapeOrigin::Rectangle { .. }) => Some("rectangle"),
                        Some(crate::core::ShapeOrigin::Circle { .. }) => Some("circle"),
                        Some(crate::core::ShapeOrigin::Star { .. }) => Some("star"),
                        Some(crate::core::ShapeOrigin::Triangle { .. }) => Some("triangle"),
                        Some(crate::core::ShapeOrigin::Spiral { .. }) => Some("spiral"),
                        None => Some("path_editor"),
                    },
                });

            if let Some(tool) = target_tool {
                self.document.select(hit_id, false);
                if self.plugin_manager.active_id() != tool {
                    self.set_active_tool(tool);
                }
                if let Some(origin) = self.document.get_selected_shape_origin() {
                    if let Some(feat) = self.plugin_manager.active_feature_mut() {
                        feat.set_shape_params(&origin);
                    }
                }
            }
        }

        let event = PointerEvent {
            screen_pos,
            world_pos,
            button: Some(PointerButton::Primary),
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
        };
        let widget_size = self.widget_size;
        let active_fill = self.active_fill_color;
        let active_stroke = self.active_stroke_color;
        let active_stroke_w = self.active_stroke_width;
        let mut ctx = PluginContext {
            document: &mut self.document,
            viewport: &mut self.viewport,
            grid_config: &self.grid_config,
            snap_config: &self.snap_config,
            ruler_config: &self.ruler_config,
            path_editor_config: &self.path_editor_config,
            active_snap_guides: &mut self.active_snap_guides,
            active_fill_color: active_fill,
            active_stroke_color: active_stroke,
            active_stroke_width: active_stroke_w,
            widget_size,
            needs_redraw: false,
            cursor_name: None,
        };
        let handled = if let Some(feat) = self.plugin_manager.active_feature_mut() {
            feat.on_double_click(&mut ctx, &event)
        } else {
            false
        };
        let cursor = ctx.cursor_name;
        let redraw = ctx.needs_redraw || handled;
        self.notify_status();
        (redraw, cursor)
    }

    pub fn execute_shortcut_action(&mut self, action: crate::core::ShortcutAction) -> bool {
        match action {
            crate::core::ShortcutAction::ToolSelect => self.set_active_tool("select"),
            crate::core::ShortcutAction::ToolPathEditor => self.set_active_tool("path_editor"),
            crate::core::ShortcutAction::ToolRectangle => self.set_active_tool("rectangle"),
            crate::core::ShortcutAction::ToolCircle => self.set_active_tool("circle"),
            crate::core::ShortcutAction::ToolStar => self.set_active_tool("star"),
            crate::core::ShortcutAction::ToolTriangle => self.set_active_tool("triangle"),
            crate::core::ShortcutAction::ToolSpiral => self.set_active_tool("spiral"),
            crate::core::ShortcutAction::ToolPen => self.set_active_tool("pen"),
            crate::core::ShortcutAction::ToolBrush => self.set_active_tool("brush"),
            crate::core::ShortcutAction::ToolText => self.set_active_tool("text"),
            crate::core::ShortcutAction::ToolPaintBucket => self.set_active_tool("paint_bucket"),
            crate::core::ShortcutAction::ToolEyedropper => self.set_active_tool("eyedropper"),
            crate::core::ShortcutAction::ToolGradient => self.set_active_tool("gradient"),
            crate::core::ShortcutAction::ToolMeshGradient => self.set_active_tool("mesh_gradient"),
            crate::core::ShortcutAction::ToolMeasure => self.set_active_tool("measure"),
            crate::core::ShortcutAction::ToolZoom => self.set_active_tool("zoom"),
            crate::core::ShortcutAction::ToolPage => self.set_active_tool("page"),

            crate::core::ShortcutAction::Zoom100 => {
                self.viewport.zoom = 1.0;
            }
            crate::core::ShortcutAction::ZoomPage => {
                let rect = self
                    .document
                    .active_page()
                    .map(|p| p.rect)
                    .unwrap_or_else(|| crate::core::Rect::new(0.0, 0.0, 794.0, 1123.0));
                self.viewport.zoom_to_rect(rect, self.widget_size);
            }
            crate::core::ShortcutAction::ZoomSelection => {
                let rect = self
                    .document
                    .selection_bounds()
                    .or_else(|| self.document.active_page().map(|p| p.rect))
                    .unwrap_or_else(|| crate::core::Rect::new(-400.0, -300.0, 800.0, 600.0));
                self.viewport.zoom_to_rect(rect, self.widget_size);
            }
            crate::core::ShortcutAction::ZoomFitAll => {
                let mut all: Option<crate::core::Rect> = None;
                for el in &self.document.elements {
                    all = match all {
                        Some(a) => Some(a.union(el.bounds())),
                        None => Some(el.bounds()),
                    };
                }
                for page in &self.document.pages {
                    all = match all {
                        Some(a) => Some(a.union(page.rect)),
                        None => Some(page.rect),
                    };
                }
                let rect =
                    all.unwrap_or_else(|| crate::core::Rect::new(-400.0, -300.0, 800.0, 600.0));
                self.viewport.zoom_to_rect(rect, self.widget_size);
            }

            crate::core::ShortcutAction::ToggleRulers => {
                self.ruler_config.visible = !self.ruler_config.visible;
            }
            crate::core::ShortcutAction::ToggleGuides => {
                self.ruler_config.guides_visible = !self.ruler_config.guides_visible;
            }
            crate::core::ShortcutAction::ToggleGrid => {
                self.grid_config.visible = !self.grid_config.visible;
            }
            crate::core::ShortcutAction::ToggleSnap => {
                self.snap_config.enabled = !self.snap_config.enabled;
            }

            crate::core::ShortcutAction::Save => {
                if let Some(ref path) = self.current_file_path.clone() {
                    if crate::core::save_document_to_file(&self.document, &path).is_ok() {
                        self.is_dirty = false;
                        self.notify_file_state();
                        return true;
                    }
                }
                if let Some(ref cb) = self.on_file_dialog_request {
                    cb(crate::core::ShortcutAction::SaveAs);
                }
            }
            crate::core::ShortcutAction::SaveAs
            | crate::core::ShortcutAction::Open
            | crate::core::ShortcutAction::NewDocument
            | crate::core::ShortcutAction::Export => {
                if let Some(ref cb) = self.on_file_dialog_request {
                    cb(action);
                }
            }

            crate::core::ShortcutAction::Undo => {
                self.document.undo();
                self.mark_dirty();
            }
            crate::core::ShortcutAction::Redo => {
                self.document.redo();
                self.mark_dirty();
            }
            crate::core::ShortcutAction::Copy => {
                self.document.copy_selected();
            }
            crate::core::ShortcutAction::Cut => {
                self.document.cut_selected();
            }
            crate::core::ShortcutAction::Paste => {
                self.document.paste(None);
            }
            crate::core::ShortcutAction::Duplicate => {
                self.document.duplicate_selected();
            }
            crate::core::ShortcutAction::Delete => {
                self.document.remove_selected();
            }
            crate::core::ShortcutAction::SelectAll => {
                self.document.select_all();
            }
            crate::core::ShortcutAction::Deselect => {
                self.document.selected_ids.clear();
            }
            crate::core::ShortcutAction::Group => {
                self.document.group_selected();
            }
            crate::core::ShortcutAction::Ungroup => {
                self.document.ungroup_selected();
            }
            crate::core::ShortcutAction::ConvertToPath => {
                self.document.convert_selected_to_path();
            }
            crate::core::ShortcutAction::BringForward => {
                self.document.bring_selected_forward();
            }
            crate::core::ShortcutAction::BringToFront => {
                self.document.bring_selected_to_front();
            }
            crate::core::ShortcutAction::SendBackward => {
                self.document.send_selected_backward();
            }
            crate::core::ShortcutAction::SendToBack => {
                self.document.send_selected_to_back();
            }
            crate::core::ShortcutAction::ToggleLock => {
                let ids: Vec<_> = self.document.selected_ids.iter().copied().collect();
                for id in ids {
                    let cur = self
                        .document
                        .elements
                        .iter()
                        .find(|e| e.id() == id)
                        .map(|e| e.locked())
                        .unwrap_or(false);
                    self.document.set_element_locked(id, !cur);
                }
            }
            crate::core::ShortcutAction::ToggleHide => {
                let ids: Vec<_> = self.document.selected_ids.iter().copied().collect();
                for id in ids {
                    let cur = self
                        .document
                        .elements
                        .iter()
                        .find(|e| e.id() == id)
                        .map(|e| e.visible())
                        .unwrap_or(true);
                    self.document.set_element_visibility(id, !cur);
                }
            }
            crate::core::ShortcutAction::CloneSelected => {
                self.document.clone_selected();
            }
            crate::core::ShortcutAction::UnlinkClone => {
                self.document.unlink_selected_clones();
            }
            crate::core::ShortcutAction::SelectOriginal => {
                self.document.select_original_element();
            }
        }
        self.notify_status();
        true
    }

    pub fn on_key_pressed(&mut self, key_event: &crate::core::KeyEvent) -> (bool, bool) {
        if (key_event.key == gdk::Key::Delete || key_event.key == gdk::Key::BackSpace)
            && !self.plugin_manager.is_editing()
        {
            if let Some(guide_id) = self.hovered_guide_id.take() {
                self.document.remove_guide(guide_id);
                self.notify_status();
                return (true, true);
            }
        }

        if self.plugin_manager.is_editing() {
            let widget_size = self.widget_size;
            let active_fill = self.active_fill_color;
            let active_stroke = self.active_stroke_color;
            let active_stroke_w = self.active_stroke_width;

            let (handled, needs_redraw) = {
                let mut ctx = PluginContext {
                    document: &mut self.document,
                    viewport: &mut self.viewport,
                    grid_config: &self.grid_config,
                    snap_config: &self.snap_config,
                    ruler_config: &self.ruler_config,
                    path_editor_config: &self.path_editor_config,
                    active_snap_guides: &mut self.active_snap_guides,
                    active_fill_color: active_fill,
                    active_stroke_color: active_stroke,
                    active_stroke_width: active_stroke_w,
                    widget_size,
                    needs_redraw: false,
                    cursor_name: None,
                };

                let handled = if let Some(feat) = self.plugin_manager.active_feature_mut() {
                    feat.on_key_down(&mut ctx, key_event)
                } else {
                    false
                };
                (handled, ctx.needs_redraw)
            };

            if handled || needs_redraw {
                self.notify_status();
                return (handled, needs_redraw);
            }
        }

        if let Some(action) = self.shortcuts.action_for_event(
            key_event.key,
            key_event.ctrl_pressed,
            key_event.shift_pressed,
            key_event.alt_pressed,
        ) {
            let handled = self.execute_shortcut_action(action);
            return (handled, true);
        }

        let widget_size = self.widget_size;
        let active_fill = self.active_fill_color;
        let active_stroke = self.active_stroke_color;
        let active_stroke_w = self.active_stroke_width;

        let (handled, needs_redraw) = {
            let mut ctx = PluginContext {
                document: &mut self.document,
                viewport: &mut self.viewport,
                grid_config: &self.grid_config,
                snap_config: &self.snap_config,
                ruler_config: &self.ruler_config,
                path_editor_config: &self.path_editor_config,
                active_snap_guides: &mut self.active_snap_guides,
                active_fill_color: active_fill,
                active_stroke_color: active_stroke,
                active_stroke_width: active_stroke_w,
                widget_size,
                needs_redraw: false,
                cursor_name: None,
            };

            let handled = if let Some(feat) = self.plugin_manager.active_feature_mut() {
                feat.on_key_down(&mut ctx, key_event)
            } else {
                false
            };
            (handled, ctx.needs_redraw)
        };

        if needs_redraw {
            self.notify_status();
        }

        (handled, needs_redraw)
    }

    pub fn render_frame(&mut self, width: i32, height: i32) -> Option<cairo::ImageSurface> {
        self.widget_size = (width as f32, height as f32);

        if !self.has_centered_initial_page && width > 50 && height > 50 {
            self.has_centered_initial_page = true;
            let page_rect = self
                .document
                .active_page()
                .map(|p| p.rect.normalize())
                .unwrap_or_else(|| crate::core::Rect::new(0.0, 0.0, 794.0, 1123.0));
            self.viewport.zoom_to_rect(page_rect, self.widget_size);
            self.notify_status();
        }

        let active_fill = self.active_fill_color;
        let active_stroke = self.active_stroke_color;
        let active_stroke_w = self.active_stroke_width;

        let live_guide_obj = self.dragging_guide.map(|(orientation, pos)| Guide {
            id: 0,
            orientation,
            position: pos,
            color: None,
            locked: false,
        });

        let active_tool = self.plugin_manager.active_id();
        let show_selection_bbox = active_tool != "path_editor"
            && (active_tool != "pen" || self.is_universal_selecting)
            && (active_tool != "brush" || self.is_universal_selecting || !self.document.selected_ids.is_empty());

        let mut renderer = std::mem::take(&mut self.renderer);
        let result = renderer.render(
            width,
            height,
            &self.document,
            &self.viewport,
            &self.grid_config,
            &self.active_snap_guides,
            &self.ruler_config,
            &self.render_options,
            self.cursor_pos,
            live_guide_obj.as_ref(),
            self.hovered_guide_id,
            self.corner_drag_current,
            show_selection_bbox,
            |canvas, vp| {
                let render_ctx = crate::plugins::PluginRenderContext {
                    document: &self.document,
                    active_fill_color: active_fill,
                    active_stroke_color: active_stroke,
                    active_stroke_width: active_stroke_w,
                    path_editor_config: &self.path_editor_config,
                };
                self.plugin_manager.render_overlay(&render_ctx, canvas, vp);
                if self.is_universal_selecting && active_tool != "select" {
                    if let Some(feat) = self.plugin_manager.feature_by_id("select") {
                        feat.render_overlay(&render_ctx, canvas, vp);
                    }
                }
            },
        );
        self.renderer = renderer;
        result
    }
}
