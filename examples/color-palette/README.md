# Paths — Example Plugin: Color Palette Toolbar

This directory contains the source code for the **Color Palette Toolbar** plugin for Paths.

---

## 🎨 Key Features

- **Fill & Stroke Mode Toggle**: Quick one-click toggle button to switch the target between Fill color and Stroke/Outline color.
- **Built-in Palette Presets**:
  - **GNOME Adwaita**: Standard GNOME HIG color spectrum (Blue, Cyan, Green, Yellow, Orange, Red, Purple, Pink, Slates, Grays).
  - **Document Colors**: Dynamic palette that automatically extracts and displays all unique colors currently used across the active vector document.
  - **Pastel Soft**: Harmonious soft pastel palette.
  - **Vibrant Neon**: High-contrast electric neon tones.
  - **Monochrome**: Precise 11-step grayscale gradient spectrum.
  - **Material 500**: Vibrant design color wheel.
  - **Custom Palette**: User-defined collection of custom color swatches.
- **Custom Swatch Creation & Color Picker**: Add any custom color to the active palette via an integrated color picker popover with RGBA/Hex entry.

---

## 🛠️ Building the Plugin (`.so`)

To build the shared dynamic library:

```bash
cd examples/color-palette
cargo build --release
```

The compiled dynamic library will be located at:
```
target/release/libcolor_palette_plugin.so
```

---

## 📦 Installation in Paths

### Option 1: Via the User Interface
1. Launch **Paths**.
2. Open **Main Menu (Hamburger) → Preferences → Plugins**.
3. Under the *Plugin Management* section, click **"Install plugin"** and select the built `libcolor_palette_plugin.so` file.

### Option 2: Via the Command Line
Copy the compiled shared library directly into the user plugins directory:
```bash
mkdir -p ~/.local/share/gnome-paths/plugins
cp target/release/libcolor_palette_plugin.so ~/.local/share/gnome-paths/plugins/
```
Restart Paths to load the plugin.

