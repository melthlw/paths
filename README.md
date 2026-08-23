
<p align="center">
  <img src="data/icons/hicolor/scalable/apps/io.github.lewis.GnomePaths.svg" alt="GNOME Paths Logo" height="128">
</p>

<h1 align="center">GNOME Paths</h1>
<p align="center"><em>Vector graphics and illustration editor for the GNOME desktop.</em></p>

<p align="center">
  🇧🇷 <a href="README.pt-BR.md">Leia em Português</a>
</p>

<p align="center">
  Creating and editing vector graphics on Linux should be fast, responsive, and well-integrated into the desktop environment.
</p>

<p align="center">
  <b>GNOME Paths</b> is a lightweight, hardware-accelerated vector design tool built with <b>GTK4</b>, <b>Libadwaita</b>, <b>Rust</b>, and the <b>Skia 2D</b> graphics engine.
</p>

<div align="center">
  <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 1em;">
    <a href="https://flathub.org/">
      <img width="190" alt="Download on Flathub" src="https://flathub.org/api/badge?locale=en" />
    </a>
  </div>
</div>

<div align="center" style="display:flex; justify-content:center; align-items:center; gap:12px; margin-top: 14px;">
    <a href="https://ko-fi.com/lauel" style="display:flex; align-items:center;">
        <img src="https://ko-fi.com/img/githubbutton_sm.svg"
             alt="Support on Ko-fi"
             style="height:30px; width:auto; display:block;">
    </a>
</div>

---

<p align="center" style="display: flex; justify-content: center; gap: 0.8em; flex-wrap: wrap;">
  <img alt="GTK4" src="https://img.shields.io/badge/GTK-4.18+-blue.svg" />
  <img alt="Libadwaita" src="https://img.shields.io/badge/Libadwaita-1.6+-purple.svg" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021-orange.svg" />
  <img alt="Engine" src="https://img.shields.io/badge/Engine-Skia%20GPU-green.svg" />
  <img alt="License" src="https://img.shields.io/badge/License-GPL--3.0--or--later-blue.svg" />
</p>

---

## Screenshots

<div align="center" style="display: flex; flex-wrap: wrap; justify-content: center; gap: 16px;">
  <img src="screenshots/main-window.png" alt="Main Window" style="max-height:360px; max-width: 48%; object-fit: contain; border-radius: 8px;">
  <img src="screenshots/preferences.png" alt="Preferences Window" style="max-height:360px; max-width: 48%; object-fit: contain; border-radius: 8px;">
</div>

---

## Features

- **Bézier Path Editing**: Node editor with support for cusp, smooth, and symmetric control handles.
- **Parametric Shapes**: Rectangles with independent corner radiuses, circles, stars, polygons, and spirals.
- **Boolean Operations**: Union, Difference, Intersection, Exclusion, Division, and Cut/Slice.
- **Gradients and Mesh**: Linear gradients, radial gradients, and on-canvas editable 2D mesh grids.
- **Multi-Page Artboards**: Manage multiple pages in a single document with individual export options.
- **Export Formats**: SVG, PNG, PDF, JPG, and WebP.
- **Snapping and Guides**: Magnetic snapping to grids, object bounding boxes, and artboard centers.
- **Adaptive Interface**: Follows system dark and light theme preferences with customizable toolbars.

---

## About the Project & Vision

**GNOME Paths** was born out of a love for vector graphics, deeply inspired by the versatility and power of **Inkscape**, with the goal of providing a modern, fast, and responsive experience that feels native to the GNOME desktop.

### Development & Transparency
This project has been extensively coded and iterated with the assistance of **AI pair-programming**, but it is thoughtfully planned, structured, and curated with genuine care and dedication. 

### Future Horizons & Ideas
I'm not entirely sure where this road will take us or how far the project will grow, but there are several ambitious directions under exploration:
- **Workspace-Driven Interface**: A Blender-inspired adaptive interface system where tabs and layouts dynamically adjust depending on the document type and workflow (e.g., Illustration, Precision Pathing, Typography).
- **Vector Animation**: Timeline, keyframing, and motion path support for vector animation.
- **Document Layout & Diagramming**: Advanced multi-page layout and publishing tools for brochures, books, and diagrams.

Many of these are still experimental ideas and concepts that may evolve or take shape as the project matures.

---

## Contributing & Support

Community support is essential to help GNOME Paths grow. If you'd like to get involved, all forms of contribution are warmly appreciated:

- **Bug Reports & Suggestions**: Help us find issues, test edge cases, and propose new workflows on GitLab.
- **Code & Development**: Submit pull requests for performance improvements, tools, or bug fixes.
- **Translations**: Help localize GNOME Paths for your language (see [TRANSLATING.md](TRANSLATING.md)).
- **Financial Support**: If you find the project useful and would like to support ongoing development, consider donating on [Ko-fi](https://ko-fi.com/lauel).

---

## How to Build

### GNOME Builder

1. Install **GNOME Builder** from Flathub.
2. Clone the repository URL `https://gitlab.gnome.org/lewisHeart/gnome-paths.git`.
3. Select the Flatpak runtime configuration.
4. Click **Run** to build and run the application.

### Flatpak CLI

```bash
# Install GNOME 47 SDK and extensions
flatpak install flathub \
  org.gnome.Platform//47 \
  org.gnome.Sdk//47 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08 \
  org.freedesktop.Sdk.Extension.llvm19//24.08

# Build and install
flatpak-builder --user --install --force-clean build-dir io.github.lewis.GnomePaths.json

# Run
flatpak run io.github.lewis.GnomePaths
```

### Meson and Ninja

```bash
meson setup build
ninja -C build
./build/gnome-paths
```

### Cargo

```bash
cargo run --release
```

To run test suites:
```bash
cargo test
```

---

## Translations

Detailed instructions on contributing and managing translations can be found in **[TRANSLATING.md](TRANSLATING.md)**.

---

## Repository and Support

- **GitLab**: [gitlab.gnome.org/lewisHeart/gnome-paths](https://gitlab.gnome.org/lewisHeart/gnome-paths)
- **Ko-fi**: [ko-fi.com/lauel](https://ko-fi.com/lauel)

---

## License

GNOME Paths is licensed under the [GNU General Public License v3.0 or later (GPL-3.0-or-later)](LICENSE).
>>>>>>> b2f4e7b (feat: initialize project structure with core modules, UI components, and asset library)
