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

## v0.3.0 — Gradients, Meshes & Pattern Engine
> **Focus**: Advanced fill rendering, interactive meshes, and asset ecosystem.

### Gradients & Mesh
- [ ] **Interactive Gradient Toolbar & On-Canvas HUD**:
  - [ ] On-canvas interactive controls for linear, radial, and sweep/angular gradients.
  - [ ] Precise addition, removal, and interpolation of color stops.
- [ ] **Fully Functional Mesh Gradients**:
  - [ ] Dedicated toolbar to configure mesh grid dimensions (rows × columns).
  - [ ] Visual editing of mesh nodes, Bézier patch curvature handles, and per-vertex color assignment.

### Patterns & Assets
- [ ] **Advanced Pattern Fill & Stroke Engine**:
  - [ ] Dynamic repeating and tiling patterns for fills and strokes.
  - [ ] On-canvas interactive scaling, rotation, and offset adjustments.
  - [ ] **Pattern Asset Management**: Load custom patterns from user asset folders (`~/.config/gnome-paths/patterns/`) and embed them into project files.

---

## v0.4.0 — Artistic Brushes, Vectorization & Typography
> **Focus**: Expressive freehand drawing tools, text-on-path, and professional typography.

### Vector Pen & Brushes
- [ ] **Advanced Vector Brush & Pen Engine**:
  - [ ] Real-time stroke smoothing with stabilizer (*Streamline / Lazy Mouse*).
  - [ ] Customizable stroke profiles: taper in, pressure/velocity dynamic thickness, and taper out.
  - [ ] Create custom brushes directly from clipboard path data (*Clipboard to Brush*).
  - [ ] **Brush Asset Presets**: Built-in library of brush presets with support for importing user asset collections.

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
