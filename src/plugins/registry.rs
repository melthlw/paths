use skia_safe as skia;

use super::modules::{
    BooleanCutStudioPlugin, BooleanDifferenceStudioPlugin, BooleanDivisionStudioPlugin,
    BooleanExclusionStudioPlugin, BooleanIntersectionStudioPlugin, BooleanUnionStudioPlugin,
    BrushStudioPlugin, CircleStudioPlugin, EyedropperStudioPlugin, GradientStudioPlugin,
    MeasureStudioPlugin, MeshGradientStudioPlugin, PageStudioPlugin, PaintBucketStudioPlugin,
    PathEditorStudioPlugin, PenStudioPlugin, RectangleStudioPlugin, SelectStudioPlugin,
    SpiralStudioPlugin, StarStudioPlugin, TextStudioPlugin, TriangleStudioPlugin,
    Zoom100StudioPlugin, ZoomFitAllStudioPlugin, ZoomPageStudioPlugin, ZoomSelectionStudioPlugin,
    ZoomStudioPlugin,
};
use super::traits::{
    FeaturePlugin, PluginContext, PluginRenderContext, StudioPlugin, ToolbarItemDescriptor,
};
use crate::core::Viewport;

#[derive(Debug, Clone)]
pub struct ToolbarGroup {
    pub order: i32,
    pub tools: Vec<ToolbarItemDescriptor>,
}

/// Central registry for all plugins (core and external/dynamically loaded).
pub struct PluginRegistry {
    modules: Vec<Box<dyn StudioPlugin>>,
    _external_handles: Vec<super::external::ExternalPlugin>,
    active_id: &'static str,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        let mut registry = Self {
            modules: Vec::new(),
            _external_handles: Vec::new(),
            active_id: "select",
        };

        // Register default built-in core studio modules
        registry.register_core(Box::new(SelectStudioPlugin::new()));
        registry.register_core(Box::new(PathEditorStudioPlugin::new()));
        registry.register_core(Box::new(RectangleStudioPlugin::new()));
        registry.register_core(Box::new(CircleStudioPlugin::new()));
        registry.register_core(Box::new(StarStudioPlugin::new()));
        registry.register_core(Box::new(TriangleStudioPlugin::new()));
        registry.register_core(Box::new(SpiralStudioPlugin::new()));
        registry.register_core(Box::new(PenStudioPlugin::new()));
        registry.register_core(Box::new(BrushStudioPlugin::new()));
        registry.register_core(Box::new(TextStudioPlugin::new()));
        registry.register_core(Box::new(PaintBucketStudioPlugin::new()));
        registry.register_core(Box::new(GradientStudioPlugin::new()));
        registry.register_core(Box::new(MeshGradientStudioPlugin::new()));
        registry.register_core(Box::new(EyedropperStudioPlugin::new()));
        registry.register_core(Box::new(MeasureStudioPlugin::new()));
        registry.register_core(Box::new(ZoomStudioPlugin::new()));
        registry.register_core(Box::new(ZoomSelectionStudioPlugin::new()));
        registry.register_core(Box::new(ZoomFitAllStudioPlugin::new()));
        registry.register_core(Box::new(Zoom100StudioPlugin::new()));
        registry.register_core(Box::new(ZoomPageStudioPlugin::new()));
        registry.register_core(Box::new(PageStudioPlugin::new()));
        registry.register_core(Box::new(BooleanUnionStudioPlugin::new()));
        registry.register_core(Box::new(BooleanDifferenceStudioPlugin::new()));
        registry.register_core(Box::new(BooleanIntersectionStudioPlugin::new()));
        registry.register_core(Box::new(BooleanExclusionStudioPlugin::new()));
        registry.register_core(Box::new(BooleanDivisionStudioPlugin::new()));
        registry.register_core(Box::new(BooleanCutStudioPlugin::new()));

        // Scan and load any installed external (.so) plugins
        let external_plugins = super::external::scan_and_load_external_plugins();
        for ext in external_plugins {
            registry.modules.push(ext.plugin);
        }

        registry
    }
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a built-in core module
    pub fn register_core(&mut self, module: Box<dyn StudioPlugin>) {
        self.modules.push(module);
    }

    /// Register an external module
    pub fn register_external(&mut self, module: Box<dyn StudioPlugin>) {
        self.modules.push(module);
    }

    pub fn active_id(&self) -> &'static str {
        self.active_id
    }

    /// Returns only external (non-core) plugins, visible in Settings
    pub fn external_plugins(&self) -> Vec<(&'static str, &'static str, bool)> {
        self.modules
            .iter()
            .filter(|m| !m.is_core())
            .map(|m| (m.id(), m.name(), true))
            .collect()
    }

    // ─────────────────────────────────────────────────────────────
    // Active Tool & Event Dispatch
    // ─────────────────────────────────────────────────────────────

    pub fn set_active_tool(&mut self, id: &'static str, ctx: &mut PluginContext) {
        if self.active_id != id {
            if let Some(feat) = self.active_feature_mut() {
                feat.on_cancel(ctx);
            }
            if self.modules.iter().any(|m| m.id() == id) {
                self.active_id = id;
            }
        }
        if let Some(feat) = self.active_feature_mut() {
            feat.on_activate(ctx);
        }
        ctx.request_redraw();
    }

    pub fn active_feature(&self) -> Option<&dyn FeaturePlugin> {
        for module in &self.modules {
            if module.id() == self.active_id {
                return module.feature();
            }
        }
        None
    }

    pub fn active_feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin> {
        for module in &mut self.modules {
            if module.id() == self.active_id {
                return module.feature_mut();
            }
        }
        None
    }

    pub fn feature_by_id(&self, id: &str) -> Option<&dyn FeaturePlugin> {
        for module in &self.modules {
            if module.id() == id {
                return module.feature();
            }
        }
        None
    }

    pub fn feature_by_id_mut(&mut self, id: &str) -> Option<&mut dyn FeaturePlugin> {
        for module in &mut self.modules {
            if module.id() == id {
                return module.feature_mut();
            }
        }
        None
    }

    pub fn is_editing(&self) -> bool {
        self.active_feature().map_or(false, |f| f.is_editing())
    }

    pub fn render_overlay(
        &self,
        ctx: &PluginRenderContext,
        canvas: &skia::Canvas,
        viewport: &Viewport,
    ) {
        if let Some(feat) = self.active_feature() {
            feat.render_overlay(ctx, canvas, viewport);
        }
    }

    pub fn toolbar_groups(&self) -> Vec<ToolbarGroup> {
        let mut group_map: std::collections::BTreeMap<&'static str, Vec<ToolbarItemDescriptor>> =
            std::collections::BTreeMap::new();
        let mut standalone_items = Vec::new();

        for module in &self.modules {
            if let Some(ui) = module.ui() {
                if let Some(desc) = ui.toolbar_item() {
                    if let Some(gid) = desc.group_id {
                        group_map.entry(gid).or_default().push(desc);
                    } else {
                        standalone_items.push(desc);
                    }
                }
            }
        }

        let mut groups = Vec::new();

        for (_gid, mut tools) in group_map {
            tools.sort_by_key(|t| t.order);
            let min_order = tools.first().map(|t| t.order).unwrap_or(100);
            groups.push(ToolbarGroup {
                order: min_order,
                tools,
            });
        }

        for item in standalone_items {
            let order = item.order;
            groups.push(ToolbarGroup {
                order,
                tools: vec![item],
            });
        }

        groups.sort_by_key(|g| g.order);
        groups
    }
}
