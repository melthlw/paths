<p align="center">
  <img src="data/icons/hicolor/scalable/apps/io.github.lewis.GnomePaths.svg" alt="GNOME Paths Logo" height="128">
</p>

<h1 align="center">GNOME Paths</h1>

<p align="center">
  <b>Modern, hardware-accelerated vector graphics and illustration design studio for GNOME.</b>
</p>

<p align="center">
  <a href="README.pt-BR.md">Versao em Portugues</a> • 
  <a href="README.md">English Version</a>
</p>

<p align="center">
  <img alt="GTK4" src="https://img.shields.io/badge/GTK-4.18+-3584e4.svg?style=flat-square&logo=gnome" />
  <img alt="Libadwaita" src="https://img.shields.io/badge/Libadwaita-1.6+-9141ac.svg?style=flat-square" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-2021%20%2F%202024-e66100.svg?style=flat-square&logo=rust" />
  <img alt="Engine" src="https://img.shields.io/badge/Engine-Skia%202D%20GPU-26a269.svg?style=flat-square" />
  <img alt="License" src="https://img.shields.io/badge/License-GPL--3.0--or--later-1c71d8.svg?style=flat-square" />
</p>

<div align="center" style="margin-top: 14px; margin-bottom: 24px;">
  <a href="https://ko-fi.com/lauel">
    <img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Support on Ko-fi" style="height:32px; width:auto; display:inline-block;">
  </a>
</div>

---

## Screenshots

<div align="center">
  <p><b>Main Canvas and Vector Illustration Workspace</b></p>
  <img src="screenshots/main-window.png" alt="GNOME Paths Workspace" style="max-width: 100%; border-radius: 10px; box-shadow: 0 8px 24px rgba(0,0,0,0.4);" />
</div>

<br/>

<div align="center">
  <p><b>Preferences and Customization Dialog</b></p>
  <img src="screenshots/preferences.png" alt="GNOME Paths Preferences" style="max-width: 85%; border-radius: 10px; box-shadow: 0 8px 24px rgba(0,0,0,0.4);" />
</div>

---

## Features and Capabilities

<table>
  <tr>
    <td width="50%">
      <h3>Advanced Bezier and Node Editing</h3>
      <p>Precision path control with cusp, smooth, and symmetric node types. Interactive tangent handles, node insertion/deletion, segment conversion (line/curve), and continuous smoothing via de Casteljau algorithm.</p>
    </td>
    <td width="50%">
      <h3>Interface Customization and Workspace</h3>
      <p>Flexible layout: floating or docked HUD toolbars with custom tool ordering, collapsible sidebars (Property Inspector, Layers, and Libraries), and native GNOME dark and light theme integration.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>Multi-Page</h3>
      <p>Multi-page environment supporting multiple independent artboards in a single document. Individual dimension controls, page reordering, overview navigation, and selective or batch export.</p>
    </td>
    <td width="50%">
      <h3>Linked Clones</h3>
      <p>Instantiate linked object clones with automatic transformation and style propagation from master elements, supporting selective unlinking for non-destructive design workflows.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>Boolean Path Operations</h3>
      <p>Real-time geometric path combinations: <b>Union</b>, <b>Difference</b>, <b>Intersection</b>, <b>Exclusion</b>, <b>Division</b>, and <b>Slice/Cut</b>.</p>
    </td>
    <td width="50%">
      <h3>Modular Native Plugin Ecosystem</h3>
      <p>Dynamic native shared library architecture (<code>.so</code>) enabling custom tools, new geometric primitives, and deep editor integrations.</p>
    </td>
  </tr>
</table>

---

## Supported Export and Import Formats

| Format | Export | Import | Key Capabilities |
| :--- | :---: | :---: | :--- |
| **SVG** | Yes | Yes | Lossless project data, standard W3C vector curves, gradients and metadata |
| **PNG** | Yes | Yes | High-DPI rasterization with alpha transparency and bitmap layer import |
| **JPG / JPEG** | Yes | Yes | Compressed raster images for textures, background canvas, and references |
| **WebP** | Yes | Yes | Modern lightweight web graphics import and export |
| **PDF** | Yes | No | Vector print-ready document pages |

---

## How to Build and Run

### Prerequisites

Required development packages:
- **Rust** (stable toolchain)
- **GTK4** (`>= 4.18`)
- **Libadwaita** (`>= 1.6`)
- **Clang / LLVM** (required for `skia-safe` binding generation)

### 1. Cargo (Local Development)

```bash
git clone https://gitlab.com/lewisHeart/gnome-paths.git
cd gnome-paths

cargo run --release
```

Run test suite:
```bash
cargo test
```

### 2. GNOME Builder (Flatpak)

1. Open **GNOME Builder**.
2. Clone repository `https://gitlab.com/lewisHeart/gnome-paths.git`.
3. Select the **GNOME 47** Flatpak runtime configuration.
4. Click **Run**.

### 3. Flatpak Builder CLI

```bash
# Install GNOME 47 runtime and Rust extensions
flatpak install flathub \
  org.gnome.Platform//47 \
  org.gnome.Sdk//47 \
  org.freedesktop.Sdk.Extension.rust-stable//24.08 \
  org.freedesktop.Sdk.Extension.llvm19//24.08

# Build and install Flatpak package
flatpak-builder --user --install --force-clean build-dir io.github.lewis.GnomePaths.json

# Run application
flatpak run io.github.lewis.GnomePaths
```

### 4. Meson and Ninja

```bash
meson setup build
ninja -C build
./build/gnome-paths
```

---

## About the Project and Vision

**GNOME Paths** was born out of a passion for vector graphics, deeply inspired by the versatility and power of **Inkscape**, with the ambition of delivering a modern, lightweight, GPU-accelerated tool tailored specifically for the GNOME desktop environment.

### Development and Transparency
This project is actively developed and iterated with the assistance of **AI pair-programming**, while being carefully architected, structured, and curated by human design. 

We champion transparency and open collaboration. If you prefer traditional hand-crafted development workflows without AI tools, you are warmly encouraged to contribute via pull requests, code reviews, native plugin creation, design suggestions, or bug reports.

### Potential Future Ideas
- **Dynamic Workspace Modes**: Adaptive UI layouts that reorganize tools based on workflow (Illustration, Editorial, Pixel/Raster, Typography).
- **Vector Animation and Timeline**: Motion interpolation, keyframes, and path animation.
- **Node Graphs (Procedural Modifiers)**: Non-destructive procedural path operations and modifiers.
- **Advanced Editorial Layouts**: Master pages, flowing text columns, and multi-column book layout.

---

## Contributing and Community

- **Bug Reports and Feedback**: Open an issue on our [GitLab Issue Tracker](https://gitlab.com/lewisHeart/gnome-paths/-/issues).
- **Translations**: Help translate GNOME Paths into your language! See **[TRANSLATING.md](TRANSLATING.md)**.
- **Plugins**: Check out the example plugin template in [`examples/plugin-template`](examples/plugin-template/).
- **Support the Project**: If you enjoy using GNOME Paths, you can support development on [Ko-fi](https://ko-fi.com/lauel).

---

## License

GNOME Paths is free and open-source software licensed under the **[GNU General Public License v3.0 or later (GPL-3.0-or-later)](LICENSE)**.
