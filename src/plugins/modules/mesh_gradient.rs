use crate::plugins::features::MeshGradientFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct MeshGradientStudioPlugin {
    feature: MeshGradientFeature,
    ui: ToolUiItem,
}

impl Default for MeshGradientStudioPlugin {
    fn default() -> Self {
        Self {
            feature: MeshGradientFeature::new(),
            ui: ToolUiItem::new(
                "mesh_gradient",
                "Mesh Gradient",
                "view-grid-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/tool-mesh.svg"),
                "Mesh Gradient (U)",
                37,
            )
            .with_group("fill-tools"),
        }
    }
}

impl MeshGradientStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for MeshGradientStudioPlugin {
    fn id(&self) -> &'static str {
        "mesh_gradient"
    }

    fn name(&self) -> &'static str {
        "Mesh Gradient Plugin"
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
