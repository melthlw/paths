# Roadmap — GNOME Paths

Development roadmap structured by version milestones.

---

## v0.2.0 — Session, Clones & UX Core
> **Focus**: State persistence, robust object manipulation, and linked clone management.

### Interface & System
- [x] **Session & Window State Persistence**:
  - [x] Save and restore UI state (window geometry, sidebar panel widths and visibility, active zoom/pan coordinates) via `GSettings` schema.
  - [x] Persist preferences across sessions (canvas background, grid/snap, themes, icon sizes, and language with System Default automatic detection).

### Manipulation & Objects
- [x] **Linked Clones Decoupling**:
  - [x] Decouple spatial transformations (position, rotation, scale) so cloned instances can be moved independently while keeping the master path and styling synchronized.
  - [x] Counter-adjust unselected clones when masters are translated, aligned, or dragged on canvas.
- [x] **Clone Management Panel**:
  - [x] Inspect and list all instances linked to each master element in a dedicated Studio Inspector panel.
  - [x] Quick action to unlink single or all clones and convert them into independent vector objects.
  - [x] Full integration into the Studio modular dock, section splitting, and detached floating windows.

---

## v0.2.1 — Node Usability & Precision Tool Cursors
> **Focus**: Vector node visual beautification, direct manipulation engine, and dedicated tool cursors.

### Vector Node System & Aesthetics
- [x] **Node System Beautification & High-Contrast Design**:
  - [x] Antialiased anchor points with subtle drop shadows and contrast halos for crisp visibility across dark/light canvas backgrounds and colorful artwork.
  - [x] Distinct geometric shapes per node type: modern rounded rectangles for Corner/Cusp nodes, smooth antialiased circles for Smooth nodes, and center-cored circles for Symmetric/Auto nodes.
  - [x] Elegant Bézier control handles with solid white cores, accent borders, drop shadows, and high-contrast dual-stroke connection lines.
  - [x] Glowing outer selection rings, hover glow rings for nodes/handles, curve segment glowing highlights, and (+) node insert indicator.
  - [x] Distinct directional start point indicator on open paths.

### Usability & Direct Manipulation
- [x] **Direct Manipulation & Intuitive Interactions**:
  - [x] Direct curve segment dragging to bend and deform Bézier curves in real time.
  - [x] Double-click on node to toggle between Smooth (tangent handles) and Corner (retracted handles).
  - [x] Double-click on segment to insert a new node cleanly using De Casteljau subdivision without distorting the curve.
  - [x] `Alt + Drag` on control handle to break symmetry and independently adjust tangent angles (converts to Corner/Cusp).
  - [x] `Shift + Drag` on handle for 45° angle snapping.
  - [x] `Delete` / `Backspace` key to cleanly remove selected nodes.

### Canvas Tool Cursors
- [x] **Dedicated High-DPI Tool Cursors**:
  - [x] Skia-rendered hardware cursor generator and cache with pixel-perfect hotspots.
  - [x] Contextual cursors for active tools: Pen (`tool:pen`, `tool:pen_add`, `tool:pen_remove`, `tool:pen_close`), Node Editor (`tool:node`, `tool:node_add`, `tool:node_curve`), Rectangle, Circle, Star, Spiral, Triangle, Brush, Eraser, Text, Gradient, Eyedropper, Measure, Page, Rotate, Zoom.

---

## v0.2.2 — Asset Libraries, Design System & Icon Geometry
> **Focus**: Integrated multi-category asset libraries panel, live dynamic language switching, design system presets, and compound path parsing.

### Asset Libraries (Studio Modular Panel)
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
  - [ ] **Effects & Shadows**:
    - [ ] Soft Elevation, Floating Card, Glassmorphism, and Neon Ambient Glow blur presets.

### Icon Precision & Compound Subpath Parsing
- [x] **Compound SVG Subpath Separation**:
  - [x] Multi-contour SVG path parser (`parse_svg_path_to_elements` and `parse_svg_path_data_subpaths`) leveraging Skia Safe to decompose compound SVG icons (`M ... Z M ... Z`) into clean distinct contours.
  - [x] Eliminate bridging artifacts and diagonal distortion in library icons and dropped SVG assets.
  - [x] Full support for cutouts/holes with opposite winding and EvenOdd fill rule in compound paths (`PathElement`).
  - [x] Complete standalone bundle in `data/resources/icons/` with all UI icons and symlink aliases (`ln -s`).

### Localization & UX
- [x] **Prominent Language Selection Radio Group**:
  - [x] Native `gtk4::CheckButton` radio group indicator with accent checkmark for "Padrão do Sistema" and selected languages.
- [x] **Live Dynamic Language Switching**:
  - [x] Instant UI rebuild without application restart via `on_language_change_local` event bus.

---

## v0.3.0 — Gradients, Meshes & Pattern Engine
> **Focus**: Advanced fill rendering, interactive meshes, pattern geometry, and universal selection.

### Universal Selection Across Tools
- [x] **Universal Canvas Selection**:
  - [x] Direct clicking on any existing element in creation tools (Rectangle, Circle, Star, Triangle, Spiral, Brush) selects the clicked element immediately.
  - [x] Holding `Ctrl` in any tool temporarily activates the selection marquee, element dragging, and transform handles without tool switching.
  - [x] Shift+click additive multi-selection support preserved across all tools.

### Pattern Engine Geometry
- [x] **Authentic Pattern Geometry**:
  - [x] *Honeycomb (Colmeia)*: 120° shared-wall hexagonal tessellation with exact aspect ratio ($H = W \cdot \sqrt{3}$) and continuous polygon borders without horizontal gaps.
  - [x] *Brick Wall (Parede de Tijolos)*: Interlocking 50% staggered courses with alternating vertical mortar joints.
  - [x] *Seigaiha Scales (Escamas / Ondas)*: Concentric 180° semicircular arcs centered at grid transition points.
  - [x] Synchronized SVG export and Skia renderers for all patterns.
- [ ] **Custom Pattern Management**:
  - [ ] On-canvas interactive scaling, rotation, and offset handles for pattern fills.
  - [ ] Load custom SVG/raster patterns from user asset folder (`~/.config/gnome-paths/patterns/`).

### Gradients & Mesh
- [x] **Multi-Stop Linear & Radial Gradients**:
  - [x] Color stop addition, deletion, opacity, and interpolation in Appearance Inspector.
- [x] **Mesh Gradient Grid**:
  - [x] Interactive mesh gradient grids with patch deformation and per-vertex color assignment.
- [ ] **On-Canvas Interactive Gradient Tool**:
  - [ ] On-canvas interactive gizmo handles for linear, radial, and sweep gradients.
- [ ] **Dedicated Mesh Dimensions Toolbar**:
  - [ ] Dynamic row × column dimension spinbuttons and patch curvature handle editing.

---

## v0.4.0 — Artistic Brushes, Vectorization & Tool Grouping
> **Focus**: Expressive freehand drawing tools, dedicated Studio toolbar, and tool palette grouping.

### Vector Pen & Brushes
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
- [ ] **Clipboard to Brush & Asset Presets**:
  - [ ] Create custom brushes directly from clipboard path data (*Clipboard to Brush*).
  - [ ] Built-in and user-imported brush presets folder (`~/.config/gnome-paths/brushes/`).

### Zoom 1:1 Standardization
- [x] **Zoom 1:1 Action & Official Badge**:
  - [x] Synchronized `Zoom 1:1 (1)` tool action with `ctx.viewport.reset()`, matching the floating HUD button.
  - [x] Standardized Adwaita `zoom-original-symbolic` badge icon with centered numeral `1`.

### Typography & Text
- [ ] **Text on Path**:
  - [ ] Draw or attach text dynamically along open or closed vector paths.
  - [ ] Controls for path offset, glyph orientation, side inversion, and repeat along path.
- [ ] **Advanced Typography Panel**:
  - [ ] Full OpenType feature support (standard and context ligatures, stylistic alternates, fractions, and tabular numerals).
  - [ ] Fine-grained controls for kerning, tracking/letter-spacing, leading, and text alignment.
- [ ] **Text Stroke & Outline**:
  - [ ] Independent stroke rendering for text elements with custom color, width, dashes, and join styles.

---

## v0.5.0 — Masks, Bitmap Processing & Modifiers
> **Focus**: Non-destructive workflows, raster image processing, and parametric modifiers.

### Masks & Images
- [ ] **Advanced Dynamic Clipping & Alpha Masks**:
  - [ ] Non-destructive clipping masks with isolated editing for masked contents.
  - [ ] Support for opacity/luminance masks driven by gradients and grayscale values.
- [ ] **Raster Image (Bitmap) Toolbar & Inspector**:
  - [ ] Controls for dimensions, aspect ratio lock, resolution (DPI), and rotation of imported images.
  - [ ] Skia non-destructive image adjustments: brightness, contrast, saturation, sharpness, and gamma correction.
- [ ] **Trace Bitmap (Image Vectorization)**:
  - [ ] Auto-trace raster images (PNG, JPEG) into editable vector paths with threshold, detail, and color quantization controls.

### Parametric Modifiers
- [ ] **Live Modifiers Panel (Live Path Effects)**:
  - [ ] *Array Modifier (Linear, Radial, Grid)*: Parametric duplication with progressive scale, rotation, and spacing.
  - [ ] *Envelope Warp Modifier*: 4-point mesh distortion applied dynamically over vector objects.
  - [ ] *Dynamic Chamfer & Rounding*: Real-time corner rounding without destructive edits to the underlying path geometry.

### Interface & System
- [ ] **Tabbed Document System**:
  - [ ] Tab bar supporting multiple files open simultaneously.
  - [ ] Individual close buttons, unsaved changes detection with confirmation dialogs.
  - [ ] Tab reordering via Drag & Drop and keyboard shortcuts (`Ctrl+Tab`, `Ctrl+W`).

---

## v1.0.0 — Stability, Packaging & Distribution
> **Focus**: Official release preparation, sandboxing, and desktop ecosystem integration.

- [ ] **Flathub & Flatpak Packaging**: Official Flatpak distribution with full XDG Desktop Portals integration.
- [ ] **Skia 2D Stress Testing**: Performance validation on complex, high-node-count vector artwork.
- [ ] **SVG 2.0 Interoperability**: Full bidirectional import/export fidelity with Inkscape and Adobe Illustrator.
- [ ] **Accessibility (a11y)**: Complete keyboard navigation and AT-SPI screen reader support (Orca).

