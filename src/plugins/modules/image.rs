use crate::plugins::features::ImageFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct ImageStudioPlugin {
    feature: ImageFeature,
    ui: ToolUiItem,
}

impl Default for ImageStudioPlugin {
    fn default() -> Self {
        Self {
            feature: ImageFeature::new(),
            ui: ToolUiItem::new(
                "image",
                "Image Frame",
                "tool-image-symbolic",
                Some("/io/gitlab/lewisHeart/GnomePaths/icons/tool-image-symbolic.svg"),
                "Image Frame (Shift+I)",
                21,
            ),
        }
    }
}

impl ImageStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for ImageStudioPlugin {
    fn id(&self) -> &'static str {
        "image"
    }

    fn name(&self) -> &'static str {
        "Image Frame Plugin"
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
