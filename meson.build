project(
  'spot',
  version: '0.1.15',
  meson_version: '>= 0.50.0',
  default_options: ['warning_level=2', 'buildtype=release'],
)


subdir('data')
subdir('po')
subdir('src')

meson.add_install_script('build-aux/meson/postinstall.py')

flatpak_cargo_generator = find_program(meson.source_root() / 'build-aux/flatpak-cargo-generator.py')
cargo_sources = custom_target(
  'cargo-update-sources',
  build_by_default: false,
  output: 'cargo-sources.json',
  input: meson.source_root() / 'Cargo.lock',
  command: [
    flatpak_cargo_generator,
    '@INPUT@',
    '-o', '@SOURCE_ROOT@/cargo-sources.json'
  ]
)
