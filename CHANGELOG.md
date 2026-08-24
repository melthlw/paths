# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [0.1.0] - 2026-08-22

### Added
- Initial release of GNOME Paths vector graphics editor built with GTK4, Libadwaita, Rust, and Skia 2D GPU.
- Interactive Bezier path node editor supporting cusp, smooth, and symmetric control handles.
- Parametric shape primitives including rectangles with independent corner radii, circles, regular polygons, stars, and spirals.
- Real-time geometric boolean operations: Union, Difference, Intersection, Exclusion, Division, and Slice.
- Multi-stop linear, radial, and 2D on-canvas mesh gradient painting.
- Magnetic smart snapping to grids, object geometry, and artboard centers.
- Multi-format document export in SVG, PNG, PDF, JPG, and WebP.
