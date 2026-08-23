use crate::plugins::features::{
    Zoom100Feature, ZoomFeature, ZoomFitAllFeature, ZoomPageFeature, ZoomSelectionFeature,
};
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

// 1. Primary Interactive Zoom Tool
pub struct ZoomStudioPlugin {
    feature: ZoomFeature,
    ui: ToolUiItem,
}

impl Default for ZoomStudioPlugin {
    fn default() -> Self {
        Self {
            feature: ZoomFeature::new(),
            ui: ToolUiItem::new(
                "zoom",
                "Zoom Tool",
                "zoom-in-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-zoom.svg"),
                "Zoom Tool (Z / Drag zoom box)",
                50,
            )
            .with_group("zoom-tools"),
        }
    }
}

impl ZoomStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for ZoomStudioPlugin {
    fn id(&self) -> &'static str {
        "zoom"
    }
    fn name(&self) -> &'static str {
        "Zoom Plugin"
    }
    fn feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin> {
        Some(&mut self.feature)
    }
    fn feature(&self) -> Option<&dyn FeaturePlugin> {
        Some(&self.feature)
    }
    fn ui(&self) -> Option<&dyn UiPlugin> {
        Some(&self.ui)
    }
}

// 2. Zoom to Selection Action
pub struct ZoomSelectionStudioPlugin {
    feature: ZoomSelectionFeature,
    ui: ToolUiItem,
}

impl Default for ZoomSelectionStudioPlugin {
    fn default() -> Self {
        Self {
            feature: ZoomSelectionFeature,
            ui: ToolUiItem::new(
                "zoom_selection",
                "Zoom Selection",
                "zoom-fit-best-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-zoom-selection.svg"),
                "Zoom Selection (3)",
                51,
            )
            .with_group("zoom-tools"),
        }
    }
}

impl ZoomSelectionStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for ZoomSelectionStudioPlugin {
    fn id(&self) -> &'static str {
        "zoom_selection"
    }
    fn name(&self) -> &'static str {
        "Zoom Selection Plugin"
    }
    fn feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin> {
        Some(&mut self.feature)
    }
    fn feature(&self) -> Option<&dyn FeaturePlugin> {
        Some(&self.feature)
    }
    fn ui(&self) -> Option<&dyn UiPlugin> {
        Some(&self.ui)
    }
}

// 3. Zoom to Fit All Action
pub struct ZoomFitAllStudioPlugin {
    feature: ZoomFitAllFeature,
    ui: ToolUiItem,
}

impl Default for ZoomFitAllStudioPlugin {
    fn default() -> Self {
        Self {
            feature: ZoomFitAllFeature,
            ui: ToolUiItem::new(
                "zoom_fit_all",
                "Zoom Fit All",
                "zoom-fit-best-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-zoom-fit-all.svg"),
                "Zoom Fit All (4)",
                52,
            )
            .with_group("zoom-tools"),
        }
    }
}

impl ZoomFitAllStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for ZoomFitAllStudioPlugin {
    fn id(&self) -> &'static str {
        "zoom_fit_all"
    }
    fn name(&self) -> &'static str {
        "Zoom Fit All Plugin"
    }
    fn feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin> {
        Some(&mut self.feature)
    }
    fn feature(&self) -> Option<&dyn FeaturePlugin> {
        Some(&self.feature)
    }
    fn ui(&self) -> Option<&dyn UiPlugin> {
        Some(&self.ui)
    }
}

// 4. Zoom 1:1 Action
pub struct Zoom100StudioPlugin {
    feature: Zoom100Feature,
    ui: ToolUiItem,
}

impl Default for Zoom100StudioPlugin {
    fn default() -> Self {
        Self {
            feature: Zoom100Feature,
            ui: ToolUiItem::new(
                "zoom_100",
                "Zoom 1:1",
                "zoom-original-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-zoom-100.svg"),
                "Zoom 1:1 (1)",
                53,
            )
            .with_group("zoom-tools"),
        }
    }
}

impl Zoom100StudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for Zoom100StudioPlugin {
    fn id(&self) -> &'static str {
        "zoom_100"
    }
    fn name(&self) -> &'static str {
        "Zoom 100 Plugin"
    }
    fn feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin> {
        Some(&mut self.feature)
    }
    fn feature(&self) -> Option<&dyn FeaturePlugin> {
        Some(&self.feature)
    }
    fn ui(&self) -> Option<&dyn UiPlugin> {
        Some(&self.ui)
    }
}

// 5. Zoom to Page Action
pub struct ZoomPageStudioPlugin {
    feature: ZoomPageFeature,
    ui: ToolUiItem,
}

impl Default for ZoomPageStudioPlugin {
    fn default() -> Self {
        Self {
            feature: ZoomPageFeature,
            ui: ToolUiItem::new(
                "zoom_fit_page",
                "Zoom Page",
                "zoom-fit-best-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-zoom-page.svg"),
                "Zoom Page (2)",
                54,
            )
            .with_group("zoom-tools"),
        }
    }
}

impl ZoomPageStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for ZoomPageStudioPlugin {
    fn id(&self) -> &'static str {
        "zoom_fit_page"
    }
    fn name(&self) -> &'static str {
        "Zoom Page Plugin"
    }
    fn feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin> {
        Some(&mut self.feature)
    }
    fn feature(&self) -> Option<&dyn FeaturePlugin> {
        Some(&self.feature)
    }
    fn ui(&self) -> Option<&dyn UiPlugin> {
        Some(&self.ui)
    }
}
