use crate::core::document::BooleanOperation;
use crate::plugins::features::BooleanFeature;
use crate::plugins::traits::{FeaturePlugin, StudioPlugin, UiPlugin};
use crate::plugins::ui::ToolUiItem;

pub struct BooleanUnionStudioPlugin {
    feature: BooleanFeature,
    ui: ToolUiItem,
}

impl Default for BooleanUnionStudioPlugin {
    fn default() -> Self {
        Self {
            feature: BooleanFeature::new(BooleanOperation::Union, "boolean-union", "Union"),
            ui: ToolUiItem::new(
                "boolean-union",
                "Union",
                "bool-union-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-union-symbolic.svg"),
                "Union (Ctrl++)",
                30,
            )
            .with_group("boolean"),
        }
    }
}

impl BooleanUnionStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for BooleanUnionStudioPlugin {
    fn id(&self) -> &'static str {
        "boolean-union"
    }
    fn name(&self) -> &'static str {
        "Boolean Union Plugin"
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

pub struct BooleanDifferenceStudioPlugin {
    feature: BooleanFeature,
    ui: ToolUiItem,
}

impl Default for BooleanDifferenceStudioPlugin {
    fn default() -> Self {
        Self {
            feature: BooleanFeature::new(
                BooleanOperation::Difference,
                "boolean-difference",
                "Difference",
            ),
            ui: ToolUiItem::new(
                "boolean-difference",
                "Difference",
                "bool-difference-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-difference-symbolic.svg"),
                "Difference (Ctrl+-)",
                31,
            )
            .with_group("boolean"),
        }
    }
}

impl BooleanDifferenceStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for BooleanDifferenceStudioPlugin {
    fn id(&self) -> &'static str {
        "boolean-difference"
    }
    fn name(&self) -> &'static str {
        "Boolean Difference Plugin"
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

pub struct BooleanIntersectionStudioPlugin {
    feature: BooleanFeature,
    ui: ToolUiItem,
}

impl Default for BooleanIntersectionStudioPlugin {
    fn default() -> Self {
        Self {
            feature: BooleanFeature::new(
                BooleanOperation::Intersection,
                "boolean-intersection",
                "Intersection",
            ),
            ui: ToolUiItem::new(
                "boolean-intersection",
                "Intersection",
                "bool-intersection-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-intersection-symbolic.svg"),
                "Intersection (Ctrl+*)",
                32,
            )
            .with_group("boolean"),
        }
    }
}

impl BooleanIntersectionStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for BooleanIntersectionStudioPlugin {
    fn id(&self) -> &'static str {
        "boolean-intersection"
    }
    fn name(&self) -> &'static str {
        "Boolean Intersection Plugin"
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

pub struct BooleanExclusionStudioPlugin {
    feature: BooleanFeature,
    ui: ToolUiItem,
}

impl Default for BooleanExclusionStudioPlugin {
    fn default() -> Self {
        Self {
            feature: BooleanFeature::new(
                BooleanOperation::Exclusion,
                "boolean-exclusion",
                "Exclusion",
            ),
            ui: ToolUiItem::new(
                "boolean-exclusion",
                "Exclusion",
                "bool-exclusion-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-exclusion-symbolic.svg"),
                "Exclusion (Ctrl+^)",
                33,
            )
            .with_group("boolean"),
        }
    }
}

impl BooleanExclusionStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for BooleanExclusionStudioPlugin {
    fn id(&self) -> &'static str {
        "boolean-exclusion"
    }
    fn name(&self) -> &'static str {
        "Boolean Exclusion Plugin"
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

pub struct BooleanDivisionStudioPlugin {
    feature: BooleanFeature,
    ui: ToolUiItem,
}

impl Default for BooleanDivisionStudioPlugin {
    fn default() -> Self {
        Self {
            feature: BooleanFeature::new(BooleanOperation::Division, "boolean-division", "Division"),
            ui: ToolUiItem::new(
                "boolean-division",
                "Division",
                "bool-division-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-division-symbolic.svg"),
                "Division (Ctrl+/)",
                34,
            )
            .with_group("boolean"),
        }
    }
}

impl BooleanDivisionStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for BooleanDivisionStudioPlugin {
    fn id(&self) -> &'static str {
        "boolean-division"
    }
    fn name(&self) -> &'static str {
        "Boolean Division Plugin"
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

pub struct BooleanCutStudioPlugin {
    feature: BooleanFeature,
    ui: ToolUiItem,
}

impl Default for BooleanCutStudioPlugin {
    fn default() -> Self {
        Self {
            feature: BooleanFeature::new(BooleanOperation::Cut, "boolean-cut", "Cut / Slice"),
            ui: ToolUiItem::new(
                "boolean-cut",
                "Cut / Slice",
                "bool-cut-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-cut-symbolic.svg"),
                "Cut / Slice (Ctrl+Alt+/)",
                35,
            )
            .with_group("boolean"),
        }
    }
}

impl BooleanCutStudioPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

impl StudioPlugin for BooleanCutStudioPlugin {
    fn id(&self) -> &'static str {
        "boolean-cut"
    }
    fn name(&self) -> &'static str {
        "Boolean Cut Plugin"
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
