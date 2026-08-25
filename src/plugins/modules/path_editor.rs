use crate::plugins::features::PathEditorFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct PathEditorStudioPlugin {
    feature: PathEditorFeature,
    ui: ToolUiItem,
}

impl Default for PathEditorStudioPlugin {
    fn default() -> Self {
        Self {
            feature: PathEditorFeature::new(),
            ui: ToolUiItem::new(
                "path_editor",
                "Path Node Editor",
                "tool-path-editor-symbolic",
                Some("/io/gitlab/lewisHeart/GnomePaths/icons/tool-path-editor-symbolic.svg"),
                "Path Node Editor (A)",
                12,
            ),
        }
    }
}

impl PathEditorStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for PathEditorStudioPlugin {
    fn id(&self) -> &'static str {
        "path_editor"
    }

    fn name(&self) -> &'static str {
        "Path Node Editor Plugin"
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
