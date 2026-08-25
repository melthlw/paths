desktop_file = i18n.merge_file(
  input: 'io.gitlab.lewisHeart.Paths.desktop.in',
  output: 'io.gitlab.lewisHeart.Paths.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appstream_file = i18n.merge_file(
  input: 'io.gitlab.lewisHeart.Paths.metainfo.xml.in',
  output: 'io.gitlab.lewisHeart.Paths.metainfo.xml',
  po_dir: '../po',
  install: true,
  install_dir: get_option('datadir') / 'metainfo'
)

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test('Validate metainfo file', appstreamcli, args: ['validate', '--no-net', '--explain', appstream_file])
endif

install_data('io.gitlab.lewisHeart.Paths.gschema.xml',
  install_dir: get_option('datadir') / 'glib-2.0' / 'schemas'
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file', compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()]
  )
endif

install_data(
  'icons/hicolor/scalable/apps/io.gitlab.lewisHeart.Paths.svg',
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'scalable' / 'apps'
)

install_data(
  'icons/hicolor/symbolic/apps/io.gitlab.lewisHeart.Paths-symbolic.svg',
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'symbolic' / 'apps'
)
