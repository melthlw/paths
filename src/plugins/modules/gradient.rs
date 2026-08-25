use crate::plugins::features::GradientFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct GradientStudioPlugin {
    feature: GradientFeature,
    ui: ToolUiItem,
}

impl Default for GradientStudioPlugin {
    fn default() -> Self {
        Self {
            feature: GradientFeature::new(),
            ui: ToolUiItem::new(
                "gradient",
                "Gradient",
                "tool-gradient-symbolic",
                Some("/io/gitlab/lewisHeart/GnomePaths/icons/tool-gradient-symbolic.svg"),
                "Gradient (G)",
                36,
            )
            .with_group("fill-tools"),
        }
    }
}

impl GradientStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for GradientStudioPlugin {
    fn id(&self) -> &'static str {
        "gradient"
    }

    fn name(&self) -> &'static str {
        "Gradient Plugin"
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
