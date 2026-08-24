use crate::plugins::features::PageFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct PageStudioPlugin {
    feature: PageFeature,
    ui: ToolUiItem,
}

impl Default for PageStudioPlugin {
    fn default() -> Self {
        Self {
            feature: PageFeature::new(),
            ui: ToolUiItem::new(
                "page",
                "Page Tool",
                "tool-page-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-page-symbolic.svg"),
                "Page Tool (Shift+P)",
                99,
            ),
        }
    }
}

impl PageStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for PageStudioPlugin {
    fn id(&self) -> &'static str {
        "page"
    }

    fn name(&self) -> &'static str {
        "Page Plugin"
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
