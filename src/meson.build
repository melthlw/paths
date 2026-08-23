cargo = find_program('cargo', required: true)

cargo_options = [
  '--manifest-path', meson.project_source_root() / 'Cargo.toml',
  '--target-dir', meson.project_build_root() / 'src',
  '--ignore-rust-version',
]

if get_option('buildtype') == 'release'
  cargo_options += ['--release']
  rust_target = 'release'
else
  rust_target = 'debug'
endif

cargo_build = custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: 'gnome-paths',
  console: true,
  command: [
    cargo, 'build',
    cargo_options,
  ]
)

custom_target(
  'install-binary',
  build_by_default: true,
  input: cargo_build,
  output: 'gnome-paths-installed',
  command: [
    'cp',
    meson.project_build_root() / 'src' / rust_target / 'gnome-paths',
    meson.project_build_root() / 'gnome-paths'
  ],
  install: true,
  install_dir: get_option('bindir')
)
