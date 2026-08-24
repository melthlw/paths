use crate::plugins::features::SelectFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct SelectStudioPlugin {
    feature: SelectFeature,
    ui: ToolUiItem,
}

impl Default for SelectStudioPlugin {
    fn default() -> Self {
        Self {
            feature: SelectFeature::new(),
            ui: ToolUiItem::new(
                "select",
                "Select & Move",
                "tool-select-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-select-symbolic.svg"),
                "Select & Move (V)",
                10,
            ),
        }
    }
}

impl SelectStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for SelectStudioPlugin {
    fn id(&self) -> &'static str {
        "select"
    }

    fn name(&self) -> &'static str {
        "Select Plugin"
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
