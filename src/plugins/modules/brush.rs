use crate::plugins::features::BrushFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct BrushStudioPlugin {
    feature: BrushFeature,
    ui: ToolUiItem,
}

impl Default for BrushStudioPlugin {
    fn default() -> Self {
        Self {
            feature: BrushFeature::new(),
            ui: ToolUiItem::new(
                "brush",
                "Pencil & Brush",
                "tool-pen-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-pen-symbolic.svg"),
                "Pencil & Brush (B)",
                18,
            )
            .with_group("pen-brush"),
        }
    }
}

impl BrushStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for BrushStudioPlugin {
    fn id(&self) -> &'static str {
        "brush"
    }

    fn name(&self) -> &'static str {
        "Brush Plugin"
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
