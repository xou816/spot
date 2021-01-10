project(
  'spot',
  version: '0.1.0',
  meson_version: '>= 0.50.0',
  default_options: ['warning_level=2', 'buildtype=release'],
)


subdir('data')
subdir('src')

meson.add_install_script('build-aux/meson/postinstall.py')
