use crate::plugins::features::RectangleFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct RectangleStudioPlugin {
    feature: RectangleFeature,
    ui: ToolUiItem,
}

impl Default for RectangleStudioPlugin {
    fn default() -> Self {
        Self {
            feature: RectangleFeature::new(),
            ui: ToolUiItem::new(
                "rectangle",
                "Rectangle & Shapes",
                "tool-square-symbolic",
                Some("/io/gitlab/lewisHeart/GnomePaths/icons/tool-square-symbolic.svg"),
                "Rectangle & Shapes (R)",
                14,
            )
            .with_group("shapes"),
        }
    }
}

impl RectangleStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for RectangleStudioPlugin {
    fn id(&self) -> &'static str {
        "rectangle"
    }

    fn name(&self) -> &'static str {
        "Rectangle Plugin"
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
