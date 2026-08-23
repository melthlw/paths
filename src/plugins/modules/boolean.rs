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
                "insert-object-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-union.svg"),
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
                "insert-object-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-difference.svg"),
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
                "insert-object-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-intersection.svg"),
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
                "insert-object-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-exclusion.svg"),
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
                "insert-object-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-division.svg"),
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
                "insert-object-symbolic",
                Some("/io/github/lewis/GnomePaths/icons/bool-cut.svg"),
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
