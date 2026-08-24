use crate::plugins::features::TextFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct TextStudioPlugin {
    feature: TextFeature,
    ui: ToolUiItem,
}

impl Default for TextStudioPlugin {
    fn default() -> Self {
        Self {
            feature: TextFeature::new(),
            ui: ToolUiItem::new(
                "text",
                "Text",
                "tool-text-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-text-symbolic.svg"),
                "Text (T)",
                20,
            ),
        }
    }
}

impl TextStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for TextStudioPlugin {
    fn id(&self) -> &'static str {
        "text"
    }

    fn name(&self) -> &'static str {
        "Text Plugin"
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
