use crate::plugins::features::SpiralFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct SpiralStudioPlugin {
    feature: SpiralFeature,
    ui: ToolUiItem,
}

impl Default for SpiralStudioPlugin {
    fn default() -> Self {
        Self {
            feature: SpiralFeature::new(),
            ui: ToolUiItem::new(
                "spiral",
                "Spiral",
                "media-record-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-spiral.svg"),
                "Spiral (W)",
                18,
            )
            .with_group("shapes"),
        }
    }
}

impl SpiralStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for SpiralStudioPlugin {
    fn id(&self) -> &'static str {
        "spiral"
    }

    fn name(&self) -> &'static str {
        "Spiral Plugin"
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
