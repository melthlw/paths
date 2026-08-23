use crate::plugins::features::CircleFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct CircleStudioPlugin {
    feature: CircleFeature,
    ui: ToolUiItem,
}

impl Default for CircleStudioPlugin {
    fn default() -> Self {
        Self {
            feature: CircleFeature::new(),
            ui: ToolUiItem::new(
                "circle",
                "Circle",
                "media-record-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-circle.svg"),
                "Circle (C)",
                15,
            )
            .with_group("shapes"),
        }
    }
}

impl CircleStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for CircleStudioPlugin {
    fn id(&self) -> &'static str {
        "circle"
    }

    fn name(&self) -> &'static str {
        "Circle Plugin"
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
