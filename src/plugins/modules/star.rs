use crate::plugins::features::StarFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct StarStudioPlugin {
    feature: StarFeature,
    ui: ToolUiItem,
}

impl Default for StarStudioPlugin {
    fn default() -> Self {
        Self {
            feature: StarFeature::new(),
            ui: ToolUiItem::new(
                "star",
                "Star",
                "starred-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-star.svg"),
                "Star (S)",
                16,
            )
            .with_group("shapes"),
        }
    }
}

impl StarStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for StarStudioPlugin {
    fn id(&self) -> &'static str {
        "star"
    }

    fn name(&self) -> &'static str {
        "Star Plugin"
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
