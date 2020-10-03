pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())
gnome = import('gnome')

gnome.compile_resources('spot',
  'spot.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

conf = configuration_data()
conf.set_quoted('VERSION', meson.project_version())
conf.set_quoted('LOCALEDIR', join_paths(get_option('prefix'), get_option('localedir')))
conf.set_quoted('PKGDATADIR', pkgdatadir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  join_paths(meson.build_root(), 'src', 'config.rs'),
  join_paths(meson.source_root(), 'src', 'config.rs'),
  check: true
)

sources = files(
  'main.rs',
  'config.rs',
  'app/mod.rs',
  'app/state.rs',
  'app/dispatch.rs',
  'app/credentials.rs',
  'app/components/mod.rs',
  'app/components/playback.rs',
  'app/components/login.rs',
  'app/components/player.rs',
  'app/components/playlist.rs',
  'app/components/browser.rs',
  'app/connect/mod.rs',
  'app/connect/playback.rs',
  'app/connect/login.rs',
  'app/connect/playlist.rs',
  'app/connect/browser.rs',
  'app/components/gtypes/mod.rs',
  'app/components/gtypes/song_model.rs',
  'app/components/gtypes/album_model.rs',
  'app/backend/mod.rs',
  'app/backend/player.rs',
  'app/backend/api.rs'
)

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))

cargo = find_program('cargo')
test('Unit test', cargo, args: ['test', '--manifest-path', join_paths(meson.source_root(), 'Cargo.toml')])

cargo_release = custom_target(
  'cargo-build',
  build_by_default: true,
  input: sources,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    cargo_script,
    meson.build_root(),
    meson.source_root(),
    '@OUTPUT@',
    get_option('buildtype'),
    meson.project_name(),
  ]
)
