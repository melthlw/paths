# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] - 2026-08-25

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
