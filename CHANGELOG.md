# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Alt-Based Freeform Lasso & Slicing Node Selection (`src/plugins/features/path_editor.rs`)**:
  - Hold `Alt` in Path/Node editor while dragging on canvas to draw a continuous freeform lasso selection.
  - Closed lasso loop detection via point-in-polygon ray casting (even-odd winding) to select enclosed anchor points.
  - Open polyline crossing detection (line segment intersection and proximity threshold) to slice-select nodes along the drawn stroke.
  - Real-time on-canvas dashed accent preview line for visual feedback during lasso drawing.
- **Analytical Tight Bounding Box Calculation (`src/core/element/path.rs`)**:
  - Analytical 1st-derivative root-solving (`PathElement::tight_bounds()`) for exact quadratic and cubic Bézier curve extremum extents.
  - Computes the exact geometric bounding box of curved paths rather than inflated control handle envelopes.
  - Seamlessly integrated into object selection bounds, clone offset calculations, modifier geometry, and export framing.
- **High-Performance Spatial Indexing & Frustum Culling (`src/core/spatial_index.rs`)**:
  - 2D R-Tree spatial indexing powered by `rstar` crate (`SpatialIndex`), organizing document elements for logarithmic geometric queries.
  - Viewport frustum culling: culls off-canvas elements during real-time panning, zooming, and high-frequency redraws, dramatically reducing draw overhead.
- **Skia Picture Recording Cache (`src/core/renderer/cache.rs`)**:
  - Per-element `skia::Picture` recording and caching (`skia::PictureRecorder`), reusing pre-recorded display lists and skipping redundant curve tessellations, shader evaluations, and modifier pipelines on static elements.
- **Welcome Window & Template Launcher (`src/ui/welcome.rs`)**:
  - Standalone Libadwaita dialog (`PathsWelcomeDialog`) with fixed 860x580 dimensions conforming to GNOME HIG standards.
  - Document preset templates organized by category: Standard Print (A4, A3, Letter), Screen & Web (1080p, 4K, Mobile), Social Media (Instagram, Twitter/X banner, YouTube thumbnail), and Iconography (GNOME 128x128, App Icon 512x512).
  - Recent files tracking with thumbnail cards, file path indicators, last opened timestamps, and quick removal.
  - Direct shortcuts: New from Preset, Open File..., and Browse Templates.
- **Bitmap Image Vectorization & Tracing Engine (`src/core/trace.rs`)**:
  - High-performance Marching Squares contour extraction with 16-state cellular topological tracking and EvenOdd signed area orientation.
  - Three vectorization modes:
    - *Monochrome / Brightness Cutoff*: Precision luminance thresholding with hole cutout support.
    - *Color Quantization*: K-means color clustering generating grouped, stacked colored vector paths.
    - *Edge Detection*: Sobel gradient magnitude filtering for outline vector tracing.
  - Ramer-Douglas-Peucker (RDP) polygon simplification with configurable tolerance.
  - Boundary padding (1-pixel false envelope) guaranteeing full topological loop closure for shapes that touch or bleed off image borders.
  - Smooth cubic Bézier handle fitting with automatic corner angle detection (`NodeType::Corner` vs `NodeType::Smooth`).
- **Interactive Libadwaita Trace Dialog (`src/ui/dialogs/trace_bitmap.rs`)**:
  - Side-by-side interactive split preview with Cairo checkerboard rendering, zoom fit, and path/node count statistics.
  - Debounced real-time recalculation (50ms) for responsive slider dragging.
- **Bidirectional Vector & Raster Workflow (`src/ui/canvas/ops_document.rs`)**:
  - `CanvasWidget::rasterize_selected_to_image()`: Rasterizes active vector selections directly into a transparent PNG bitmap image element with full Undo/Redo (`Ctrl+Z`).
  - `CanvasWidget::apply_traced_elements()`: In-place replacement or stacked overlay of traced vector paths over original bitmap elements.
- **Canvas Context Menu Integration (`src/ui/context_menu/`)**:
  - Right-click on raster image objects provides "Trace Bitmap...".
  - Right-click on vector selections provides "Rasterize to Bitmap".
- **3D Projection Rendering & Perspective Engine (`src/core/modifier.rs`)**:
  - Added Perspective projection mode with focal depth and perspective vanishing distortion to `Extrude3DModifier`.
  - Back-face cap rendering and silhouette contour lines ensuring solid, topologically clean 3D meshes.
- **Style Copy/Paste & Enhanced Clipboard (`src/core/clipboard.rs` / `src/ui/canvas/ops_document.rs`)**:
  - Copy Style (`Ctrl+Alt+C`) and Paste Style (`Ctrl+Alt+V`) transferring fill layers, strokes, and opacity across vector elements.
  - Drag-and-drop file import support on the canvas for instant placement of SVG, PNG, JPG, and WebP files.
- **Studio Inspector Catalog & Dock Improvements (`src/ui/inspector/`)**:
  - Enhanced catalog popover scrolling (`max_content_height(380)` and width 260px) in `catalog.rs` for clear access to all panels.
  - Fixed tab index bounds in `dock.rs` ensuring seamless docking, reordering, and closing for all 9 inspector panels.
- **Localization**:
  - Complete Brazilian Portuguese (`pt_BR`) translations for all new bitmap adjustments, tracing, rasterization, and welcome dialog features.

### Changed
- **Modernized Libadwaita Image Inspector (`src/ui/inspector/image.rs`)**:
  - Refactored UI utilizing Libadwaita `adw::StatusPage` for clean, HIG-compliant empty states with quick action buttons ("Rasterize Selection to Bitmap", "Import Image File...", "Create Image Frame").
  - Replaced standard switches with `adw::SwitchRow` for Invert, Grayscale, and Sepia non-destructive filters.
  - Modernized slider controls (`create_adw_slider_row`) utilizing Libadwaita `adw::ActionRow` with inline expansive sliders and numeric value readout badges.
  - Removed redundant custom CSS overrides from `style.css` in favor of standard Libadwaita styling tokens and spacing.
- **Unified Canvas Status Architecture (`src/ui/canvas/state.rs`)**:
  - Replaced verbose status callback parameters across canvas events with a unified `CanvasStatusSnapshot` struct consolidating zoom level, pan offsets, active tool, selection bounds, and node counts.
- **3D Extrude Geometry Parameters (`src/core/modifier.rs`)**:
  - Refined bevel edge radius scaling and ring level geometry calculations for extruded shapes.


## [0.4.0] - 2026-08-28

### Added
- **Image Frame Tool (`ImageFeature` & `ImageStudioPlugin`)**:
  - Interactive canvas tool for creating and placing image frames via click or drag (`Shift+I`).
  - Native GNOME/Adwaita symbolic icon (`tool-image-symbolic.svg`) and precision custom Skia cursor.
  - Refined Adwaita Dark `#242427` empty-state placeholder graphic with 1.5px border and vector photo frame silhouette.
  - Support for interactive double-click tool activation, direct resizing, and aspect-ratio preservation.
- **Hardware-Accelerated Non-Destructive Image Processing (Skia Color Matrix Engine)**:
  - Real-time Skia color matrix shader calculations supporting:
    - Brightness adjustment (-100% to +100%).
    - Contrast curve scaling (0.0x to 2.5x).
    - Saturation adjustment (0.0x to 2.5x with ITU-R BT.709 luminance vector).
    - Hue chromatic rotation (-180° to +180°).
    - GPU Gaussian Blur filtering (0.0 to 50.0 px).
    - Full color spectrum inversion.
    - Black & White monochrome grayscale conversion.
    - Classic photographic sepia tinting.
- **Image Inspector Panel (`src/ui/inspector/image.rs`)**:
  - Dedicated Adwaita card panel with real-time numeric sliders, switches, and file management.
  - Dimension & native pixel readouts with 1-click aspect ratio restoration.
  - Quick Filter Presets: Normal (Reset), Vibrant, B&W High Contrast, Vintage/Sepia, Warm, and Cool.
  - Contextual inspector switching automatically hiding vector fill/stroke cards when an image frame is selected.
- **Tool Options Toolbar Integration (`src/ui/tool_options/image.rs`)**:
  - Quick access HUD bar for image replacement, aspect ratio restoration, opacity adjustment, and file name badge.

## [0.4.5] - 2026-08-25

### Added
- **3D Vector Extrusion & Vector FX Suite (`Extrude3DModifier`)**:
  - Full 3D extrusion engine supporting 2D vector shapes, text elements, rectangles, polygons, brush strokes, and groups.
  - Projection modes (`Isometric`, `Cabinet 45°`, `Perspective`) with adjustable 3D depth and projection angle.
  - Ambient occlusion lighting shading with 1-click side extrusion color swatches (`Auto`, `Dark Metal`, `Gold`, `Ruby`, `Cyan`).
  - Advanced 3D parametric controls: Taper (pyramidal cone scaling), Twist (3D rotational extrude swirl), Bevel Edge Radius (3D vector bevel highlights), and Specular Gloss lighting reflections.
  - 1-Click 3D Presets Bar (`Extrude`, `Pyramid`, `Twist 3D`, `Bevel`) in inspector card header.
- **Canvas Interactive 3D Handles & Control Gizmos**:
  - Live cyan direction vector ray + interactive grab handle on canvas to rotate 3D angle and depth directly with mouse drag.
- **GTK Inspector 3D Angle Dimmer Knob Widget**:
  - Custom 46px circular 360° angle dimmer knob widget (`gtk4::DrawingArea` with `GestureDrag`) for intuitive visual rotation in GTK inspector.
- **Screen-Space Zoom-Adaptive Sub-Pixel Sweep Engine**:
  - Dynamic scale-aware sub-pixel sweep resolution (< 0.2 screen pixels per step) using `canvas.local_to_device_as_3x3()`.
  - Guarantees 100% smooth, anti-aliased 3D vector edges with zero staircasing or pixel steps at any canvas zoom level (even 10,000% zoom).
- **Single Skia Path Render Engine**:
  - Merges all sub-pixel extrusion volume steps into 1 single `skia::Path` before drawing, executing 1 single Skia GPU/CPU draw call for 144+ FPS buttery smooth live dragging.
- **Text Glyphs to Vector Path Conversion (`TextElement::to_skia_path`)**:
  - Skia Font glyph outline extractor (`font.get_path`) converting text strings into true vector bezier outlines for 3D extrusion of text letters.
- **Twist & Swirl Distortion Modifier (`TwistModifier`)**:
  - Rotational swirl path node distortion around element bounds.

### Changed
- **GTK Inspector Cards Overhaul & Re-entrancy Guards**:
  - Re-architected GTK modifier cards with compact `.linked` header bar (`[Eye | Apply | Save | Up | Down | Trash]`).
  - Added re-entrancy flags (`is_updating`) on `Scale` and `SpinButton` synchronization to eliminate GTK signal deadlocks.
  - Removed status notification rebuilds during live slider dragging to achieve zero-lag, 144+ FPS UI interaction.
- **Deduplicated Asset Catalog Presets**:
  - Cleaned up duplicate array modifier preset files from catalog gallery (`assets/modifiers/`).

## [0.3.0] - 2026-08-25

### Added
- **Dynamic Dock Layout Engine (`DockLayoutManager`)**:
  - Multi-layered edge docking architecture (`Top`, `Bottom`, `Left`, `Right`) for floating HUD bars (Main Toolbar, Tool Options, Color Palette, and modular plugin bars).
  - Sequential automatic margin calculation with generous 12px gap separation preventing bar overlap regardless of docking configuration.
  - Live measurement of GTK widget dimensions (`height()`) respecting theme CSS paddings and borders.
  - Real-time layout recalculation when any bar is moved, hidden, or dynamically activated.
- **Application Namespace & ID Migration**:
  - Full migration of application ID, GSchema, Desktop Entry, AppStream metainfo, Flatpak manifests, and GResource path to `io.gitlab.lewisHeart.GnomePaths`.
- **Advanced Tiled Clones System (Clonagem em Ladrilhos Dinâmicos)**:
  - Parametric grid generation supporting 11 wallpaper symmetries and arrangements: P1 (Simple Translation), P2 (180° Half-Turn), PM (Horizontal Mirror), PMM (Double Mirror), PG (Glide Reflection), CM (Alternating Reflection), PMG (Reflection & Glide), PGG (Double Glide), P4 (90° Rotation), P6 (60° Hexagonal Symmetry), and Radial (Circular Ring Distribution).
  - Granular multi-dimensional transformation studio with row/column increments, percentage offsets, scale growth, angular progression, opacity fading, and pseudo-random jitter.
  - Interactive inspector UI in Libadwaita with ExpanderRows, SpinButtons, live tile counters, and one-click actions: Create Tiled Clones, Unlink All, Clear Clones, and Reset Defaults.
  - Full Undo/Redo history integration for all matrix and radial clone operations.
- **Color Palette Toolbar & Plugin Auto-Restoration**:
  - Integrated color palette toolbar plugin (`tool-pattern-symbolic.svg`) with custom swatch management.
  - Automatic restoration of saved plugin enablement state on boot from `AppSettings::enabled_plugins()`.
- **RefCell Borrow Safety & UI Stability**:
  - Replaced direct `RefCell` borrows in Inspector Appearance updates with `try_borrow` and `try_borrow_mut` guards to prevent double-borrow panics during interactive updates.
  - In-place row updates preventing widget destruction and premature popover dismissal while dragging HSV color pickers.
- **GNOME 50 Platform Alignment**:
  - Upgraded Flatpak manifests and build configuration to GNOME 50 runtime and SDK (`org.gnome.Platform//50`, `org.gnome.Sdk//50`).
  - Offline prebuilt Skia binaries source integration for seamless Flatpak compilation.
- **Test Suite Expansion**:
  - 77 automated unit tests covering tiled clone mathematics, geometric symmetries, radial dispersion, and undo/redo state integrity.

### Changed
- Redesigned Clones panel into an interactive studio with live linked instance tracking, jump-to-master navigation, and instant canvas selection.
- Enforced lower position for the small color control box when docked vertically on Left/Right canvas edges.
- Cleaned and deduplicated gettext translation catalogs (`po/en.po`, `po/pt_BR.po`, and `po/gnome-paths.pot`).

## [0.2.3] - 2026-08-24

### Added
- Multi-page artboard document architecture with individual and batch page export.
- Linked clones system with automatic transformation and style synchronization from master elements.
- Asset library panel and reusable component catalog in the inspector.
- Centralized GSettings configuration persistence for window geometry, canvas state, and tool options.
- Granular transform controls for independent scaling and translation of gradients, patterns, and stroke widths.
- Native dynamic plugin engine supporting modular shared libraries (.so).
- Lossless SVG roundtrip with embedded project metadata and compound path reconstruction.
- Import support for raster bitmap assets in PNG, JPG, and WebP formats.
- Complete 68-test automated verification suite covering geometry, units, SVG I/O, and document lifecycles.

### Changed
- Standardized all UI icons to symbolic 16x16 and 20x20 SVGs complying with GNOME Human Interface Guidelines.
- Redesigned floating HUD toolbar indicators and restructured inspector appearance controls.
- Upgraded package definition and dependencies to Rust 2024 edition.
- Migrated primary repository, issue tracker, and project documentation to GitLab.

## [0.1.0] - 2026-08-02

### Added
- Initial release of Paths vector graphics editor built with GTK4, Libadwaita, Rust, and Skia 2D GPU.
- Interactive Bezier path node editor supporting cusp, smooth, and symmetric control handles.
- Parametric shape primitives including rectangles with independent corner radii, circles, regular polygons, stars, and spirals.
- Real-time geometric boolean operations: Union, Difference, Intersection, Exclusion, Division, and Slice.
- Multi-stop linear, radial, and 2D on-canvas mesh gradient painting.
- Magnetic smart snapping to grids, object geometry, and artboard centers.
- Multi-format document export in SVG, PNG, PDF, JPG, and WebP.
