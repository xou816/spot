gnome = import('gnome')
gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)

dependency('libadwaita-1')
dependency('gtk4')
dependency('glib-2.0')
dependency('openssl')
dependency('alsa')
dependency('libpulse')

conf = configuration_data()

conf.set_quoted('LOCALEDIR', get_option('prefix') / get_option('localedir'))

pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()
conf.set_quoted('PKGDATADIR', pkgdatadir)

if get_option('buildtype') == 'debug'
  conf.set_quoted('APPID', 'dev.alextren.Spot.Devel')
  conf.set('VERSION', meson.project_version() + '-dev')
else
  conf.set_quoted('APPID', 'dev.alextren.Spot')
  conf.set('VERSION', meson.project_version())
endif

gnome.compile_resources('spot',
  'spot.gresource.xml',
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir
)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: conf
)

# Copy the config.rs output to the source directory.
run_command(
  'cp',
  meson.current_build_dir() / 'config.rs',
  meson.current_source_dir() / 'config.rs',
  check: true
)

cargo = find_program('cargo', required: true)

if get_option('buildtype') == 'release'
  rust_target = 'release'
  cargo_profile_option = '--release'
else
  rust_target = 'debug'
  cargo_profile_option = '--verbose'
endif

cargo_manifest = meson.project_source_root() / 'Cargo.toml'
cargo_output = 'src' / rust_target / meson.project_name()

env = environment()
env.set('CARGO_HOME', meson.project_source_root() / 'cargo')

cargo_options = [
  '--manifest-path', cargo_manifest,
  '--features', get_option('features'),
  '--target-dir', meson.project_build_root() / 'src',
  cargo_profile_option
]

if get_option('offline')
  cargo_options += ['--offline']
endif

cargo_release = custom_target(
  'cargo-build',
  build_always_stale: true,
  output: 'bin',
  console: true,
  install: false,
  command: [
    cargo, 'build', cargo_options,
  ],
  env: env
)

final_bin = custom_target(
  'copy-cargo-build',
  build_by_default: true,
  build_always_stale: true,
  input: cargo_release,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: get_option('bindir'),
  command: [
    'cp', meson.project_build_root() / cargo_output, '@OUTPUT@'
  ]
)

test('Unit tests',
  cargo,
  args: [
    'test',
    '--manifest-path', cargo_manifest,
    '--target-dir', meson.project_build_root() / 'src',
    cargo_profile_option
  ],
  timeout: 180,
  env: env
)

test('Clippy',
  cargo,
  args: [
    'clippy',
    '--manifest-path', cargo_manifest,
    '--target-dir', meson.project_build_root() / 'src',
    '--',
    '-D', 'warnings',
    '-A', 'clippy::module_inception',
    '-A', 'clippy::new_without_default',
    '-A', 'clippy::enum-variant-names'
  ],
  timeout: 180,
  env: env
)
