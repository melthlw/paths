cargo = find_program('cargo', required: true)

if get_option('buildtype') == 'release'
  cargo_options = ['--release']
  rust_target = 'release'
else
  cargo_options = []
  rust_target = 'debug'
endif

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: 'gnome-paths',
  install: true,
  install_dir: get_option('bindir'),
  command: [
    'sh', '-c',
    'cargo build --manifest-path "$1" --target-dir "$2" --ignore-rust-version ' + ' '.join(cargo_options) + ' && cp "$2/$3/gnome-paths" "$4"',
    '--',
    meson.project_source_root() / 'Cargo.toml',
    meson.project_build_root() / 'src',
    rust_target,
    '@OUTPUT@'
  ]
)

test(
  'cargo-test',
  cargo,
  args: ['test', '--manifest-path', meson.project_source_root() / 'Cargo.toml'],
  timeout: 0,
)
