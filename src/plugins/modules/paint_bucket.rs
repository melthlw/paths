use crate::plugins::features::PaintBucketFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct PaintBucketStudioPlugin {
    feature: PaintBucketFeature,
    ui: ToolUiItem,
}

impl Default for PaintBucketStudioPlugin {
    fn default() -> Self {
        Self {
            feature: PaintBucketFeature::new(),
            ui: ToolUiItem::new(
                "paint_bucket",
                "Paint Bucket",
                "format-fill-color-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-paint-bucket.svg"),
                "Paint Bucket (K)",
                35,
            )
            .with_group("fill-tools"),
        }
    }
}

impl PaintBucketStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for PaintBucketStudioPlugin {
    fn id(&self) -> &'static str {
        "paint_bucket"
    }

    fn name(&self) -> &'static str {
        "Paint Bucket Plugin"
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
