pub mod boolean;
pub mod brush;
pub mod circle;
pub mod color_palette;
pub mod eyedropper;
pub mod gradient;
pub mod image;
pub mod measure;
pub mod mesh_gradient;
pub mod page;
pub mod paint_bucket;
pub mod path_editor;
pub mod pattern;
pub mod pen;
pub mod rectangle;
pub mod select;
pub mod spiral;
pub mod star;
pub mod text;
pub mod triangle;
pub mod zoom;

pub use boolean::{
    BooleanCutStudioPlugin, BooleanDifferenceStudioPlugin, BooleanDivisionStudioPlugin,
    BooleanExclusionStudioPlugin, BooleanIntersectionStudioPlugin, BooleanUnionStudioPlugin,
};
pub use brush::BrushStudioPlugin;
pub use circle::CircleStudioPlugin;
pub use color_palette::ColorPaletteStudioPlugin;
pub use eyedropper::EyedropperStudioPlugin;
pub use gradient::GradientStudioPlugin;
pub use image::ImageStudioPlugin;
pub use measure::MeasureStudioPlugin;
pub use mesh_gradient::MeshGradientStudioPlugin;
pub use page::PageStudioPlugin;
pub use paint_bucket::PaintBucketStudioPlugin;
pub use path_editor::PathEditorStudioPlugin;
pub use pattern::PatternStudioPlugin;
pub use pen::PenStudioPlugin;
pub use rectangle::RectangleStudioPlugin;
pub use select::SelectStudioPlugin;
pub use spiral::SpiralStudioPlugin;
pub use star::StarStudioPlugin;
pub use text::TextStudioPlugin;
pub use triangle::TriangleStudioPlugin;
pub use zoom::{
    Zoom100StudioPlugin, ZoomFitAllStudioPlugin, ZoomPageStudioPlugin, ZoomSelectionStudioPlugin,
    ZoomStudioPlugin,
};
