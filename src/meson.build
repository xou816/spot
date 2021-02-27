pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())
gnome = import('gnome')
dependency('libhandy-1')

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

# find . -name "*.rs" -printf "'%p',\n"
sources = files(
'./api/cache.rs',
'./api/api_models.rs',
'./api/client.rs',
'./api/cached_client.rs',
'./api/mod.rs',
'./app/dispatch.rs',
'./app/components/details/details.rs',
'./app/components/details/details_model.rs',
'./app/components/details/mod.rs',
'./app/components/artist/mod.rs',
'./app/components/utils.rs',
'./app/components/playback/playback_info.rs',
'./app/components/playback/playback_control.rs',
'./app/components/playback/mod.rs',
'./app/components/playlist_details/playlist_details.rs',
'./app/components/playlist_details/playlist_details_model.rs',
'./app/components/playlist_details/mod.rs',
'./app/components/album/album.rs',
'./app/components/album/mod.rs',
'./app/components/artist_details/artist_details.rs',
'./app/components/artist_details/artist_details_model.rs',
'./app/components/artist_details/mod.rs',
'./app/components/login/login.rs',
'./app/components/login/mod.rs',
'./app/components/login/login_model.rs',
'./app/components/window/mod.rs',
'./app/components/saved_playlists/saved_playlists_model.rs',
'./app/components/saved_playlists/mod.rs',
'./app/components/saved_playlists/saved_playlists.rs',
'./app/components/library/library_model.rs',
'./app/components/library/library.rs',
'./app/components/library/mod.rs',
'./app/components/player_notifier.rs',
'./app/components/now_playing/now_playing.rs',
'./app/components/now_playing/mod.rs',
'./app/components/now_playing/now_playing_model.rs',
'./app/components/user_menu/user_menu_model.rs',
'./app/components/user_menu/mod.rs',
'./app/components/user_menu/user_menu.rs',
'./app/components/playlist/song.rs',
'./app/components/playlist/playlist.rs',
'./app/components/playlist/mod.rs',
'./app/components/navigation/navigation.rs',
'./app/components/navigation/factory.rs',
'./app/components/navigation/home.rs',
'./app/components/navigation/mod.rs',
'./app/components/navigation/navigation_model.rs',
'./app/components/notification/mod.rs',
'./app/components/search/search_bar_model.rs',
'./app/components/search/search_model.rs',
'./app/components/search/search_bar.rs',
'./app/components/search/mod.rs',
'./app/components/search/search.rs',
'./app/components/mod.rs',
'./app/loader.rs',
'./app/list_store.rs',
'./app/dbus/mpris.rs',
'./app/dbus/mod.rs',
'./app/dbus/types.rs',
'./app/credentials.rs',
'./app/models.rs',
'./app/backend/player.rs',
'./app/backend/mod.rs',
'./app/state/app_model.rs',
'./app/state/browser_state.rs',
'./app/state/screen_states.rs',
'./app/state/playback_state.rs',
'./app/state/app_state.rs',
'./app/state/mod.rs',
'./app/gtypes/album_model.rs',
'./app/gtypes/song_model.rs',
'./app/gtypes/artist_model.rs',
'./app/gtypes/mod.rs',
'./app/mod.rs',
'./config.rs',
'./main.rs',
)

cargo_script = find_program(join_paths(meson.source_root(), 'build-aux/cargo.sh'))
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
    meson.source_root(),
    meson.build_root(),
    meson.project_name(),
    '@OUTPUT@',
    get_option('buildtype'),
    get_option('offline') ? 'true' : 'false'
  ]
)

cargo_test = find_program(join_paths(meson.source_root(), 'build-aux/test.sh'))
test('Unit tests',
  cargo_test,
  args: [
    meson.source_root(),
    meson.build_root(),
    get_option('offline') ? 'true' : 'false'
  ]
)
