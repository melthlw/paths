use crate::plugins::features::PatternFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct PatternStudioPlugin {
    feature: PatternFeature,
    ui: ToolUiItem,
}

impl Default for PatternStudioPlugin {
    fn default() -> Self {
        Self {
            feature: PatternFeature::new(),
            ui: ToolUiItem::new(
                "pattern",
                "Pattern",
                "tool-pattern-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-pattern-symbolic.svg"),
                "Pattern Tool (Shift+P)",
                38,
            )
            .with_group("fill-tools"),
        }
    }
}

impl PatternStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for PatternStudioPlugin {
    fn id(&self) -> &'static str {
        "pattern"
    }

    fn name(&self) -> &'static str {
        "Pattern Tool Plugin"
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
