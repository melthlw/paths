pub mod brush_store;
pub mod color;
pub mod document;
pub mod element;
pub mod events;
pub mod geometry;
pub mod grid;
pub mod i18n;
pub mod io;
pub mod layer;
pub mod libraries_store;
pub mod modifier;
pub mod modifier_store;
pub mod page;
pub mod path_editor_config;
pub mod pattern_store;
pub mod renderer;
pub mod ruler;
pub mod settings;
pub mod shortcuts;
pub mod snap;
pub mod spatial_index;
pub mod svg_export;
pub mod svg_import;
pub mod units;

pub use color::Color;
pub use spatial_index::SpatialIndex;
pub use document::{Document, TiledCloneParams, TransformOptions};
pub use element::{
    ArcMode, BlendMode, BrushMode, BrushStroke, BrushStyle, CornerRadii, CornerStyle, Element,
    ElementId, FillLayer, FillStyle, Gradient, GradientStop, GradientType, ImageElement,
    MarkerShape, MeshGradient, PathElement, PathNode, PatternType, RectElement, ShapeOrigin,
    StrokeCap, StrokeJoin, StrokeLayer, StrokeStyle, TextAlign, TextElement, dist_to_segment,
    get_system_font_families,
};
pub use events::{KeyEvent, PointerButton, PointerEvent};
pub use geometry::{
    Point, Rect, TransformHandle, Viewport, calculate_resize_scales, hit_transform_handle,
};
pub use grid::{GridConfig, GridStyle};

pub use i18n::{
    Language, get_language, gettext, init as init_i18n, on_language_change_local, set_language,
};
pub use io::{load_document_from_file, save_document_to_file};
pub use page::PageId;
pub use path_editor_config::{HandleDisplayMode, NodeType, PathEditorConfig};

pub use pattern_store::{create_custom_pattern_shader, scan_user_patterns};
pub use renderer::{
    ARTBOARD_HEIGHT, ARTBOARD_WIDTH, ExportConfig, ExportFormat, ExportScope, RenderOptions,
    SkiaRenderer, export_document,
};
pub use ruler::{Guide, GuideOrientation, RulerConfig};

pub use settings::AppSettings;
pub use shortcuts::{KeyCombo, ShortcutAction, ShortcutCategory, ShortcutManager, ShortcutPreset};
pub use snap::{SnapConfig, SnapEngine, SnapGuide};
pub use svg_export::export_document_to_svg;

pub use svg_import::parse_svg;
pub use units::{Unit, eval_math_expression};
