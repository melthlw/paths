use crate::plugins::features::MeasureFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct MeasureStudioPlugin {
    feature: MeasureFeature,
    ui: ToolUiItem,
}

impl Default for MeasureStudioPlugin {
    fn default() -> Self {
        Self {
            feature: MeasureFeature::new(),
            ui: ToolUiItem::new(
                "measure",
                "Ruler / Measure",
                "tool-measure-symbolic",
                Some("/io/gitlab/lewisHeart/GnomePaths/icons/tool-measure-symbolic.svg"),
                "Ruler / Measure (M)",
                41,
            ),
        }
    }
}

impl MeasureStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for MeasureStudioPlugin {
    fn id(&self) -> &'static str {
        "measure"
    }

    fn name(&self) -> &'static str {
        "Measure Plugin"
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
