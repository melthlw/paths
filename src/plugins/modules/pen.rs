use crate::plugins::features::PenFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct PenStudioPlugin {
    feature: PenFeature,
    ui: ToolUiItem,
}

impl Default for PenStudioPlugin {
    fn default() -> Self {
        Self {
            feature: PenFeature::new(),
            ui: ToolUiItem::new(
                "pen",
                "Vector Pen",
                "document-edit-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-vector-pen.svg"),
                "Vector Pen (P)",
                16,
            ),
        }
    }
}

impl PenStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for PenStudioPlugin {
    fn id(&self) -> &'static str {
        "pen"
    }

    fn name(&self) -> &'static str {
        "Vector Pen Plugin"
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
