use crate::plugins::features::EyedropperFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct EyedropperStudioPlugin {
    feature: EyedropperFeature,
    ui: ToolUiItem,
}

impl Default for EyedropperStudioPlugin {
    fn default() -> Self {
        Self {
            feature: EyedropperFeature::new(),
            ui: ToolUiItem::new(
                "eyedropper",
                "Eyedropper",
                "color-select-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-eyedropper.svg"),
                "Eyedropper (I)",
                40,
            ),
        }
    }
}

impl EyedropperStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for EyedropperStudioPlugin {
    fn id(&self) -> &'static str {
        "eyedropper"
    }

    fn name(&self) -> &'static str {
        "Eyedropper Plugin"
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
