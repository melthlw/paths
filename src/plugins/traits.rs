use skia_safe as skia;
use std::collections::HashSet;

use crate::core::{
    Color, Document, ElementId, GridConfig, KeyEvent, PathEditorConfig, Point, PointerEvent, Rect,
    RulerConfig, SnapConfig, SnapEngine, SnapGuide, Viewport,
};

pub struct PluginContext<'a> {
    pub document: &'a mut Document,
    pub viewport: &'a mut Viewport,
    pub grid_config: &'a GridConfig,
    pub snap_config: &'a SnapConfig,
    pub ruler_config: &'a RulerConfig,
    pub path_editor_config: &'a PathEditorConfig,
    pub active_snap_guides: &'a mut Vec<SnapGuide>,
    pub active_fill_color: Color,
    pub active_stroke_color: Option<Color>,
    pub active_stroke_width: f32,

    pub widget_size: (f32, f32),
    pub needs_redraw: bool,
    pub cursor_name: Option<&'static str>,
}

impl<'a> PluginContext<'a> {
    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    pub fn set_cursor(&mut self, cursor: &'static str) {
        self.cursor_name = Some(cursor);
    }

    pub fn snap_point(&mut self, point: Point, exclude_ids: &HashSet<ElementId>) -> Point {
        let res = SnapEngine::snap_point(
            point,
            self.document,
            self.viewport,
            self.snap_config,
            self.grid_config,
            self.ruler_config,
            exclude_ids,
        );
        *self.active_snap_guides = res.guides;
        res.point
    }

    pub fn snap_rect(&mut self, rect: Rect, exclude_ids: &HashSet<ElementId>) -> (Point, Point) {
        let res = SnapEngine::snap_rect(
            rect,
            self.document,
            self.viewport,
            self.snap_config,
            self.grid_config,
            self.ruler_config,
            exclude_ids,
        );
        *self.active_snap_guides = res.guides;
        (res.point, res.delta)
    }

    pub fn clear_snap_guides(&mut self) {
        if !self.active_snap_guides.is_empty() {
            self.active_snap_guides.clear();
            self.needs_redraw = true;
        }
    }
}

/// Feature Plugin: Core vector tool and document logic
pub trait FeaturePlugin: 'static + Send + Sync {
    fn on_activate(&mut self, _ctx: &mut PluginContext) {}
    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent);
    fn on_pointer_move(&mut self, ctx: &mut PluginContext, event: &PointerEvent);
    fn on_pointer_up(&mut self, ctx: &mut PluginContext, event: &PointerEvent);
    fn on_double_click(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) -> bool {
        false
    }

    fn on_key_down(&mut self, _ctx: &mut PluginContext, _event: &KeyEvent) -> bool {
        false
    }

    fn on_cancel(&mut self, _ctx: &mut PluginContext) {}

    fn is_editing(&self) -> bool {
        false
    }

    fn as_pen_feature_mut(&mut self) -> Option<&mut crate::plugins::features::pen::PenFeature> {
        None
    }

    fn as_brush_feature(&self) -> Option<&crate::plugins::features::brush::BrushFeature> {
        None
    }

    fn as_brush_feature_mut(
        &mut self,
    ) -> Option<&mut crate::plugins::features::brush::BrushFeature> {
        None
    }

    fn render_overlay(
        &self,
        _ctx: &PluginRenderContext,
        _canvas: &skia::Canvas,
        _viewport: &Viewport,
    ) {
    }

    /// Get shape-specific tool parameters for the UI toolbar
    fn get_shape_params(&self) -> Option<crate::core::element::ShapeOrigin> {
        None
    }

    /// Update shape-specific tool parameters from the UI toolbar
    fn set_shape_params(&mut self, _origin: &crate::core::element::ShapeOrigin) {}

    /// Get selected node indices for path editing
    fn get_selected_nodes(&self) -> Vec<(ElementId, usize)> {
        Vec::new()
    }

    /// Set selected node indices for path editing
    fn set_selected_nodes(&mut self, _nodes: Vec<(ElementId, usize)>) {}

    /// Clear selected node indices for path editing
    fn clear_selected_nodes(&mut self) {}

    /// Select all nodes in active path
    fn select_all_nodes(&mut self, _ctx: &mut PluginContext) {}
}

pub struct PluginRenderContext<'a> {
    pub document: &'a Document,
    pub active_fill_color: Color,
    pub active_stroke_color: Option<Color>,
    pub active_stroke_width: f32,
    pub path_editor_config: &'a PathEditorConfig,
}

/// Description of a toolbar item provided by a UI plugin
#[derive(Debug, Clone)]
pub struct ToolbarItemDescriptor {
    pub tool_id: &'static str,
    pub icon_name: &'static str,
    pub icon_resource: Option<&'static str>,
    pub tooltip: &'static str,
    pub group_id: Option<&'static str>,
    pub order: i32,
}

/// UI Plugin: Visual interfaces, buttons, inspectors, and HUD overlays
pub trait UiPlugin: Send + Sync {
    fn toolbar_item(&self) -> Option<ToolbarItemDescriptor> {
        None
    }
}

/// Unified Studio Plugin: Encapsulates both Feature logic and UI components together
pub trait StudioPlugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;

    /// Core plugins are built-in and NOT visible in the user's plugin settings.
    /// External/imported plugins should override this to return false.
    fn is_core(&self) -> bool {
        true
    }

    fn feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin>;
    fn feature(&self) -> Option<&dyn FeaturePlugin>;
    fn ui(&self) -> Option<&dyn UiPlugin>;
}
