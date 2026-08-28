# Roadmap — Paths

Development roadmap structured by semantic versioning (SemVer) milestones for **Paths**, a modern vector graphics and illustration design studio for GNOME.

---

## 📌 Versioning Strategy

This project adheres to **Semantic Versioning (SemVer 2.0.0)** (`MAJOR.MINOR.PATCH`):

- **Pre-1.0 Development (`0.y.z`)**: 
  - **`MINOR` increments (`0.x.0`)**: Represent feature milestones (new tools, canvas engines, UI inspectors, modifiers).
  - **`PATCH` increments (`0.x.y`)**: Represent bug fix updates, performance tweaks, and UI refinements within a minor feature cycle.
- **Production Stable (`1.0.0+`)**:
  - **`MAJOR`**: Breaking changes to file formats, core plugin APIs, or architecture.
  - **`MINOR`**: Backward-compatible new features and tool additions.
  - **`PATCH`**: Backward-compatible bug fixes and minor improvements.

---

## 🚀 Released Milestones

### v0.2.0 — Session, Clones & UX Core `[Released]`
> **Focus**: State persistence, robust object manipulation, and linked clone management.

#### Interface & System
- [x] **Session & Window State Persistence**:
  - [x] Save and restore UI state (window geometry, sidebar panel widths and visibility, active zoom/pan coordinates) via `GSettings` schema.
  - [x] Persist preferences across sessions (canvas background, grid/snap, themes, icon sizes, and language with System Default automatic detection).

#### Manipulation & Objects
- [x] **Linked Clones Decoupling**:
  - [x] Decouple spatial transformations (position, rotation, scale) so cloned instances can be moved independently while keeping the master path and styling synchronized.
  - [x] Counter-adjust unselected clones when masters are translated, aligned, or dragged on canvas.
- [x] **Clone Management Panel**:
  - [x] Inspect and list all instances linked to each master element in a dedicated Studio Inspector panel.
  - [x] Quick action to unlink single or all clones and convert them into independent vector objects.
  - [x] Full integration into the Studio modular dock, section splitting, and detached floating windows.

---

### v0.2.1 — Node Usability & Precision Tool Cursors `[Released]`
> **Focus**: Vector node visual beautification, direct manipulation engine, and dedicated tool cursors.

#### Vector Node System & Aesthetics
- [x] **Node System Beautification & High-Contrast Design**:
  - [x] Antialiased anchor points with subtle drop shadows and contrast halos for crisp visibility across dark/light canvas backgrounds and colorful artwork.
  - [x] Distinct geometric shapes per node type: modern rounded rectangles for Corner/Cusp nodes, smooth antialiased circles for Smooth nodes, and center-cored circles for Symmetric/Auto nodes.
  - [x] Elegant Bézier control handles with solid white cores, accent borders, drop shadows, and high-contrast dual-stroke connection lines.
  - [x] Glowing outer selection rings, hover glow rings for nodes/handles, curve segment glowing highlights, and (+) node insert indicator.
  - [x] Distinct directional start point indicator on open paths.

#### Usability & Direct Manipulation
- [x] **Direct Manipulation & Intuitive Interactions**:
  - [x] Direct curve segment dragging to bend and deform Bézier curves in real time.
  - [x] Double-click on node to toggle between Smooth (tangent handles) and Corner (retracted handles).
  - [x] Double-click on segment to insert a new node cleanly using De Casteljau subdivision without distorting the curve.
  - [x] `Alt + Drag` on control handle to break symmetry and independently adjust tangent angles (converts to Corner/Cusp).
  - [x] `Shift + Drag` on handle for 45° angle snapping.
  - [x] `Delete` / `Backspace` key to cleanly remove selected nodes.

#### Canvas Tool Cursors
- [x] **Dedicated High-DPI Tool Cursors**:
  - [x] Skia-rendered hardware cursor generator and cache with pixel-perfect hotspots.
  - [x] Contextual cursors for active tools: Pen (`tool:pen`, `tool:pen_add`, `tool:pen_remove`, `tool:pen_close`), Node Editor (`tool:node`, `tool:node_add`, `tool:node_curve`), Rectangle, Circle, Star, Spiral, Triangle, Brush, Eraser, Text, Gradient, Eyedropper, Measure, Page, Rotate, Zoom.

---

### v0.2.2 — Asset Libraries, Design System & Icon Geometry `[Released]`
> **Focus**: Integrated multi-category asset libraries panel, live dynamic language switching, design system presets, and compound path parsing.

#### Asset Libraries (Studio Modular Panel)
- [x] **New "Libraries" Inspector Tab (Closed by default)**:
  - [x] **Live Search & Filter**: Real-time cross-library search bar (`SearchEntry`) for immediate discovery of assets and styles.
  - [x] **Category Pill Switcher**: Smooth animated category transitions between Swatches, Patterns, Icons, Shapes, Strokes, and Typography.
  - [x] **Color Swatches & Palettes**:
    - [x] Curated palettes: GNOME Adwaita Core, Tailwind Modern, Cyberpunk & Neon, and Pastel & Soft.
    - [x] Antialiased rounded swatch tiles with tooltips and one-click fill application to active selection or canvas fill.
  - [x] **Pattern & Texture Presets**:
    - [x] Visual cards for Technical Grid, Halftone Dots, Diagonal Stripes, Checkerboard, Hexagonal Honeycomb, Brick Wall, and Seigaiha Scales.
    - [x] One-click pattern fill layer attachment to selected vector objects.
  - [x] **Vector Symbolic Icons Library**:
    - [x] Rich categorized vector icon collection (Home, User, Search, Settings, Heart, Star, Check, Close, Folder, Cloud, Trash, Edit, Play, Pause, Camera, Lock, Globe, Code, Audio Volume, etc.).
    - [x] Click-to-insert vector icons directly onto canvas as editable Bézier paths.
  - [x] **Geometric Shapes & Badges Library**:
    - [x] Ready-to-use vector shapes: Shield, 8-Point Badge, Ribbon, Speech Bubble, Lightning Bolt, Hexagon, Octagon, Price Tag, Directional Arrow, and Diamond.
    - [x] Click-to-insert vector shapes with instant node editing and styling support.
  - [x] **Stroke Styles & Dash Presets**:
    - [x] Solid, Dashed, Dotted, and stroke preset library.
    - [x] One-click stroke layer application to selected elements.
  - [x] **Typography & Hierarchy Presets**:
    - [x] Display Hero (48pt Bold), Heading 1 (32pt Bold), Heading 2 (24pt SemiBold), Body (16pt Regular), Monospace Code (14pt), and Caption (11pt).
    - [x] Instant font family, size, and weight application to selected text elements.

#### Icon Precision & Compound Subpath Parsing
- [x] **Compound SVG Subpath Separation**:
  - [x] Multi-contour SVG path parser (`parse_svg_path_to_elements` and `parse_svg_path_data_subpaths`) leveraging Skia Safe to decompose compound SVG icons (`M ... Z M ... Z`) into clean distinct contours.
  - [x] Eliminate bridging artifacts and diagonal distortion in library icons and dropped SVG assets.
  - [x] Full support for cutouts/holes with opposite winding and EvenOdd fill rule in compound paths (`PathElement`).
  - [x] Complete standalone bundle in `data/resources/icons/` with all UI icons and symlink aliases (`ln -s`).

#### Localization & UX
- [x] **Prominent Language Selection Radio Group**:
  - [x] Native `gtk4::CheckButton` radio group indicator with accent checkmark for "Padrão do Sistema" and selected languages.
- [x] **Live Dynamic Language Switching**:
  - [x] Instant UI rebuild without application restart via `on_language_change_local` event bus.

---

### v0.2.3 — Transform Modes, Fill Inspector Layout & Studio Polish `[Released]`
> **Focus**: Interactive transform modes, two-row fill inspector layout, toolbar contextual interaction, and zero-warning codebase cleanup.

#### Transform Modes & Proportional Scaling
- [x] **Interactive Transform Mode Toggles**:
  - [x] **Scale Stroke Width**: Proportional scaling of stroke width (`stroke_width`) and multi-stroke layer widths when resizing objects via selection handles or width/height inspector controls (`TransformOptions.scale_stroke_width`).
  - [x] **Scale Corner Radii**: Proportional scaling of rectangle corner radii (`corner_radius` and per-corner `CornerRadii`) when resizing rounded rectangles (`TransformOptions.scale_corner_radii`).
  - [x] **Move & Scale Gradients**: Synchronized translation and proportional scaling of linear, radial, and mesh gradient handles with object transforms (`TransformOptions.move_gradients`).
  - [x] **Move & Scale Patterns**: Synchronized translation and proportional scaling of geometric pattern texture scale with object transforms (`TransformOptions.move_patterns`).
  - [x] Dedicated toggle buttons in Select Tool options bar (`btn_scale_stroke`, `btn_scale_corners`, `btn_move_gradients`, `btn_move_patterns`).

#### Appearance Inspector & Layout
- [x] **Two-Row Fill Layer Architecture**:
  - [x] Streamlined multi-row layout for complex fills (Linear, Radial, Pattern, Mesh):
    - *Row 1 (Controls & Actions)*: Layer grip `[::]`, fill style dropdown, angle/options dropdown, opacity percentage, visibility toggle, and delete layer.
    - *Row 2 (Colors & Swatches)*: Dual color swatches with hex code inputs with adaptive wrapping to eliminate horizontal clipping in narrow inspector sidebars.
  - [x] Compact single-row layout preserved for solid color fills.

#### Toolbar & Interaction Polish
- [x] **Secondary Click (Right-Click) Grip Menu**:
  - [x] Restrict toolbar position menu (Top, Bottom, Left, Right) to secondary click (right-click / button 3) on the drag grip, preventing accidental popup opening during left-click dock navigation.
- [x] **Pencil vs. Vector Pen Symbolic Differentiation**:
  - [x] Clear symbolic iconography separating Vector Pen and Freehand Pencil/Brush in the main toolbox and options bar.

#### Code Quality & Compiler Cleanliness
- [x] **Zero-Warning Codebase Cleanliness**:
  - [x] Eliminated all dead code warnings and unused methods across 19 modules without resorting to `#![allow(dead_code)]`.
  - [x] Wrapped all dynamic plugin loading symbols in explicit `unsafe` blocks for Rust 2024 compliance.

---

### v0.3.0 — Gradients, Meshes & Pattern Engine `[Released]`
> **Focus**: Advanced fill rendering, interactive meshes, pattern geometry, and universal selection.

#### Universal Selection Across Tools
- [x] **Universal Canvas Selection**:
  - [x] Direct clicking on any existing element in creation tools (Rectangle, Circle, Star, Triangle, Spiral, Brush) selects the clicked element immediately.
  - [x] Holding `Ctrl` in any tool temporarily activates the selection marquee, element dragging, and transform handles without tool switching.
  - [x] Shift+click additive multi-selection support preserved across all tools.

#### Pattern Engine Geometry
- [x] **Authentic Pattern Geometry**:
  - [x] *Honeycomb*: 120° shared-wall hexagonal tessellation with exact aspect ratio (\(H = W \cdot \sqrt{3}\)) and continuous polygon borders without horizontal gaps.
  - [x] *Brick Wall*: Interlocking 50% staggered courses with alternating vertical mortar joints.
  - [x] *Seigaiha Scales*: Concentric 180° semicircular arcs centered at grid transition points.
  - [x] Synchronized SVG export and Skia renderers for all patterns.
- [x] **Custom Pattern Management**:
  - [x] On-canvas interactive scaling, rotation, and offset handles for pattern fills.
  - [x] Load custom SVG/raster patterns from user asset folder (`~/.config/gnome-paths/libraries/patterns/`).

#### Gradients & Mesh
- [x] **Multi-Stop Linear & Radial Gradients**:
  - [x] Color stop addition, deletion, opacity, and interpolation in Appearance Inspector.
- [x] **Mesh Gradient Grid**:
  - [x] Interactive mesh gradient grids with patch deformation and per-vertex color assignment.
- [x] **On-Canvas Interactive Gradient Tool**:
  - [x] On-canvas interactive gizmo handles for linear, radial, and sweep gradients.
- [x] **Dedicated Mesh Dimensions Toolbar**:
  - [x] Dynamic row × column dimension spinbuttons and patch curvature handle editing.

---

### v0.4.0 — Artistic Brushes, Vectorization & Tool Grouping `[Released]`
> **Focus**: Expressive freehand drawing tools, dedicated Studio toolbar, and tool palette grouping.

#### Vector Pen & Brushes
- [x] **Advanced Vector Brush & Pencil Engine**:
  - [x] Real-time stroke smoothing with stabilizer filter (moving average / Chaikin corner smoothing).
  - [x] Customizable stroke profiles: start taper, end taper, pressure/velocity dynamic thickness tapering.
  - [x] **Dual Mode Architecture**:
    - [x] *Pencil Mode*: Real-time Catmull-Rom cubic Bézier curve fitting to generate clean, node-editable vector paths (`Element::Path`).
    - [x] *Brush Mode*: Rich expressive freehand strokes (`BrushStroke`) supporting 6 styles: Solid Round, Pencil, Calligraphy (45° chisel ribbon), Inking Pen, Highlighter Marker (multiplicative blending), and Soft Airbrush.
  - [x] Instant stroke-to-path conversion (`to_path_element()` and `convert_selected_to_path`).
  - [x] Auto-close path when closing loop within threshold distance.
- [x] **Dedicated Pencil & Brush Tool Options Bar**:
  - [x] Segmented mode switch (Pencil / Brush), style dropdown, stroke width, stabilizer spinbutton, calligraphy angle, pressure dynamics, taper start/end, auto-close, and cap styles.
- [x] **Toolbox Grouping & Flyout Popovers**:
  - [x] Grouped **Vector Pen & Pencil Brush** (`pen-brush`) with Vector Pen as primary default.
  - [x] Grouped Shapes (`shapes`), Fill Tools (`fill-tools`), Zoom Tools (`zoom-tools`), and Booleans (`boolean`).
  - [x] Dynamic tool icon switching and flyout popover on long-press / right-click.
- [x] **Clipboard to Brush & Asset Presets**:
  - [x] Create custom brushes directly from clipboard path data (*Clipboard to Brush*).
  - [x] Built-in and user-imported brush presets folder (`~/.config/paths/brushes/`).

#### Zoom 1:1 Standardization
- [x] **Zoom 1:1 Action & Official Badge**:
  - [x] Synchronized `Zoom 1:1 (1)` tool action with `ctx.viewport.reset()`, matching the floating HUD button.
  - [x] Standardized Adwaita `zoom-original-symbolic` badge icon with centered numeral `1`.

#### Typography & Text
- [x] **Text on Path**:
  - [x] Draw or attach text dynamically along open or closed vector paths.
  - [x] Controls for path offset, glyph orientation, side inversion, and repeat along path.
- [x] **Advanced Typography Panel**:
  - [x] Full OpenType feature support (standard and context ligatures, stylistic alternates, fractions, and tabular numerals).
  - [x] Fine-grained controls for kerning, tracking/letter-spacing, leading, and text alignment.
- [x] **Text Stroke & Outline**:
  - [x] Independent stroke rendering for text elements with custom color, width, dashes, and join styles.

### v0.4.5 — 3D Vector Extrusion Engine & Vector FX Suite `[Released]`
> **Focus**: Parametric 3D vector extrusion, ambient occlusion lighting, canvas interactive gizmos, GTK visual dimmer controls, and zoom-adaptive sub-pixel sweep rendering.

#### 3D Extrusion & Vector FX Engine
- [x] **3D Vector Extrusion Engine (`Extrude3DModifier`)**:
  - [x] Full 3D extrusion support across all element types (Paths, Rectangles, Text, Polygons, Brushes, Groups).
  - [x] Projection modes: Isometric, Cabinet 45°, and Perspective.
  - [x] Ambient occlusion lighting shading with 1-click side extrusion color swatches (`Auto`, `Dark Metal`, `Gold`, `Ruby`, `Cyan`).
  - [x] Advanced 3D parametric controls: Taper (pyramidal cone scaling), Twist (3D rotational extrude swirl), Bevel Edge Radius (3D vector bevel highlights), and Specular Gloss lighting reflections.
  - [x] 1-Click 3D Presets Bar (`Extrude`, `Pyramid`, `Twist 3D`, `Bevel`).
- [x] **Canvas Interactive 3D Handles & Control Gizmos**:
  - [x] Live cyan direction vector ray + interactive grab handle on canvas to rotate 3D angle and depth directly with mouse drag.
- [x] **GTK Inspector 3D Angle Dimmer Knob Widget**:
  - [x] Custom 46px circular 360° angle dimmer knob widget (`gtk4::DrawingArea` with `GestureDrag`) for intuitive visual rotation in GTK inspector.
- [x] **Screen-Space Zoom-Adaptive Sub-Pixel Sweep Engine**:
  - [x] Dynamic scale-aware sub-pixel sweep resolution (< 0.2 screen pixels per step) leveraging `canvas.local_to_device_as_3x3()`.
  - [x] Guarantees 100% smooth, anti-aliased 3D vector edges with zero staircasing or pixel steps at any canvas zoom level (even 10,000% zoom).
- [x] **Single Skia Path Render Engine**:
  - [x] Merges all sub-pixel extrusion volume steps into 1 single `skia::Path` before drawing, executing 1 single Skia GPU/CPU draw call for 144+ FPS buttery smooth live dragging.
- [x] **Text Glyphs to Vector Path Outline Conversion (`TextElement::to_skia_path`)**:
  - [x] Skia Font glyph outline extractor (`font.get_path`) converting text strings into true vector bezier outlines for 3D extrusion of text letters.
- [x] **Twist & Swirl Distortion Modifier (`TwistModifier`)**:
  - [x] Rotational swirl path node distortion around element bounds.

---

## 🛠️ Upcoming Milestones

### v0.5.0 — Masks, Bitmap Processing & Modifiers `[In Progress]`
> **Focus**: Non-destructive workflows, raster image processing, and parametric modifiers.

#### Masks & Images
- [ ] **Advanced Dynamic Clipping & Alpha Masks**:
  - [ ] Non-destructive clipping masks with isolated editing for masked contents.
  - [ ] Support for opacity/luminance masks driven by gradients and grayscale values.
- [x] **Image Frame Tool & Adwaita HIG Visuals**:
  - [x] Interactive creation of image frames on canvas with custom precision cursor (`tool-image-symbolic.svg` / `Shift+I`).
  - [x] Refined GNOME / Adwaita Dark `#242427` empty-state placeholder card with 1.5px subtle border and vector photo frame silhouette.
  - [x] Tool options HUD bar with image replacement, aspect ratio restoration, and live opacity slider.
- [x] **Raster Image (Bitmap) Non-Destructive Processing & Inspector**:
  - [x] Controls for dimensions, aspect ratio lock, and native resolution (DPI) readouts.
  - [x] Skia non-destructive GPU-accelerated adjustments: brightness, contrast, saturation, hue rotation, Gaussian blur, and color spectrum inversion.
  - [x] Monochrome grayscale and vintage sepia tone filters with 1-click quick presets (Normal, Vibrant, B&W High, Vintage, Warm, Cool).
  - [x] Automatic contextual switching in the Studio Inspector for active image selections.
- [ ] **Trace Bitmap (Image Vectorization)**:
  - [ ] Auto-trace raster images (PNG, JPEG) into editable vector paths with threshold, detail, and color quantization controls.

#### Parametric Modifiers
- [x] **Live Modifiers Panel (Live Path Effects)**:
  - [x] *Array Modifier (Linear, Radial, Grid)*: Parametric duplication with progressive scale, rotation, and spacing.
  - [x] *3D Extrusion Modifier*: Parametric 3D vector volume with lighting, taper, twist, and bevel.
  - [x] *Twist & Swirl Modifier*: Rotational Bézier node deformation.
  - [x] *Envelope Warp Modifier*: 4-point mesh distortion applied dynamically over vector objects.
  - [x] *Dynamic Chamfer & Rounding*: Real-time corner rounding without destructive edits to the underlying path geometry.

#### Interface & System
- [ ] **Tabbed Document System**:
  - [ ] Tab bar supporting multiple files open simultaneously.
  - [ ] Individual close buttons, unsaved changes detection with confirmation dialogs.
  - [ ] Tab reordering via Drag & Drop and keyboard shortcuts (`Ctrl+Tab`, `Ctrl+W`).

---

### v0.6.0 — Plugins, Color Profiles & Advanced Export `[Planned]`
> **Focus**: Extensibility ABI, professional printing color management, and multi-format export.

#### Plugin Ecosystem & ABI
- [ ] **Dynamic Plugin Architecture**:
  - [ ] Safe C/Rust FFI bindings for dynamic shared library (`.so`) plugins.
  - [ ] Plugin Manager panel in Studio preferences for enabling/disabling user plugins.
- [ ] **Color Management (CMYK & ICC Profiles)**:
  - [ ] Support for ICC color profiles and CMYK soft-proofing for print design workflows.

#### Advanced Export Engine
- [ ] **Multi-Format Vector & Document Export**:
  - [ ] PDF vector export with embedded fonts and CMYK layer support.
  - [ ] EPS and DXF (CAD) export engines.
  - [ ] Batch export slice generator (export assets at 1x, 2x, 3x, SVG, PNG, WebP simultaneously).

---

### v0.7.0 — GPU Rendering, Performance & Multi-Threading `[Planned]`
> **Focus**: Skia GPU acceleration, multi-core rendering pipeline, and stress optimization.

- [x] **Skia Hardware Acceleration**: Vulkan / OpenGL backend initialization for ultra-smooth 120 FPS panning/zooming on complex illustrations.
- [ ] **Tiled Parallel Canvas Renderer**: Multi-threaded scene rendering for complex artwork with over 100,000 nodes.
- [ ] **Async Undo/Redo Engine**: Zero-lag history tree management for large document undo stacks.

---

### v1.0.0 — Production Release, Packaging & Ecosystem `[Planned]`
> **Focus**: Official production release, sandboxing, accessibility, and desktop ecosystem integration.

- [ ] **Flathub & Flatpak Packaging**: Official Flatpak distribution with full XDG Desktop Portals integration (File Chooser, Color Picker, Print).
- [ ] **Skia 2D Stress Testing**: Complete validation and benchmark suite for vector rendering stability.
- [ ] **SVG 2.0 Interoperability**: 100% bidirectional import/export fidelity with Inkscape, Figma, and Adobe Illustrator.
- [ ] **Accessibility (a11y)**: Complete keyboard navigation, focus indicators, and AT-SPI screen reader support (Orca).
- [ ] **Official User Manual & Documentation**: Interactive onboarding guide and comprehensive shortcut reference.
