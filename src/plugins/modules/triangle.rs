use crate::plugins::features::TriangleFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct TriangleStudioPlugin {
    feature: TriangleFeature,
    ui: ToolUiItem,
}

impl Default for TriangleStudioPlugin {
    fn default() -> Self {
        Self {
            feature: TriangleFeature::new(),
            ui: ToolUiItem::new(
                "triangle",
                "Triangle",
                "tool-triangle-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-triangle-symbolic.svg"),
                "Triangle (Y)",
                17,
            )
            .with_group("shapes"),
        }
    }
}

impl TriangleStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for TriangleStudioPlugin {
    fn id(&self) -> &'static str {
        "triangle"
    }

    fn name(&self) -> &'static str {
        "Triangle Plugin"
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
