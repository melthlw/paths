use serde::{Deserialize, Serialize};

/// Type of anchor node on a Bézier path
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Cusp / Corner: control handles are independent in both angle and length
    Corner,
    /// Smooth: control handles remain collinear (180° opposite), independent lengths
    Smooth,
    /// Symmetric: control handles remain collinear and equal in length
    Symmetric,
    /// Auto-Smooth: control handles are automatically calculated based on neighbor nodes
    Auto,
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Corner
    }
}

/// Handle visibility mode for the path node editor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandleDisplayMode {
    /// Show handles only for selected node(s)
    SelectedOnly,
    /// Show handles for all nodes of any selected path
    AllInSelectedPath,
    /// Always show all handles on canvas
    Always,
}

impl Default for HandleDisplayMode {
    fn default() -> Self {
        Self::SelectedOnly
    }
}

/// Global settings and visual preferences for the Path Node Editor
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PathEditorConfig {
    /// Visual size of anchor nodes in screen pixels (e.g. 6.0 .. 16.0)
    pub node_size: f32,
    /// Visual radius/size of Bézier handle endpoints in screen pixels
    pub handle_size: f32,
    /// Mouse/pointer hit tolerance radius in screen pixels
    pub hit_tolerance: f32,
    /// Handle display mode
    pub handle_display_mode: HandleDisplayMode,
    /// Differentiate node shapes visually (Square = Corner, Diamond = Smooth, Circle = Symmetric/Auto)
    pub show_distinct_node_shapes: bool,
    /// Enable direct Bézier segment dragging without touching handles first
    pub enable_direct_segment_drag: bool,
    /// Show direction arrows along path segments
    pub show_path_direction: bool,
    /// Highlight curve segments under mouse hover
    pub highlight_hovered_segment: bool,
    /// Angle snapping step in degrees for handles when holding Shift (e.g. 15.0, 30.0, 45.0, 90.0, 0.0 = off)
    pub angle_snapping_step: f32,
    /// Show path outline in distinct highlight color
    pub show_path_outline: bool,
    /// Default node type when inserting new nodes
    pub default_node_type: NodeType,
}

impl Default for PathEditorConfig {
    fn default() -> Self {
        Self {
            node_size: 9.0,
            handle_size: 6.0,
            hit_tolerance: 12.0,
            handle_display_mode: HandleDisplayMode::SelectedOnly,
            show_distinct_node_shapes: true,
            enable_direct_segment_drag: true,
            show_path_direction: false,
            highlight_hovered_segment: true,
            angle_snapping_step: 15.0,
            show_path_outline: true,
            default_node_type: NodeType::Smooth,
        }
    }
}
