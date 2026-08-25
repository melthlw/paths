# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
- Initial release of GNOME Paths vector graphics editor built with GTK4, Libadwaita, Rust, and Skia 2D GPU.
- Interactive Bezier path node editor supporting cusp, smooth, and symmetric control handles.
- Parametric shape primitives including rectangles with independent corner radii, circles, regular polygons, stars, and spirals.
- Real-time geometric boolean operations: Union, Difference, Intersection, Exclusion, Division, and Slice.
- Multi-stop linear, radial, and 2D on-canvas mesh gradient painting.
- Magnetic smart snapping to grids, object geometry, and artboard centers.
- Multi-format document export in SVG, PNG, PDF, JPG, and WebP.
