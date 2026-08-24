use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};

/// Studio Plugin representation of the Color Palette Toolbar.
/// `is_core()` returns false to declare this as a modular, non-system (external) plugin.
pub struct ColorPaletteStudioPlugin {}

impl Default for ColorPaletteStudioPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl ColorPaletteStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for ColorPaletteStudioPlugin {
    fn id(&self) -> &'static str {
        "color_palette_toolbar"
    }

    fn name(&self) -> &'static str {
        "Color Palette Toolbar"
    }

    /// External / Non-system plugin: Visible and toggleable in plugin preferences
    fn is_core(&self) -> bool {
        false
    }

    fn feature_mut(&mut self) -> Option<&mut dyn FeaturePlugin> {
        None
    }

    fn feature(&self) -> Option<&dyn FeaturePlugin> {
        None
    }

    fn ui(&self) -> Option<&dyn UiPlugin> {
        None
    }
}
