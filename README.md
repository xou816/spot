# Spot

Gtk/Rust native Spotify client for the Gnome desktop.

Based on [librespot](https://github.com/librespot-org/librespot/).

![Preview](./demo.gif)

## Features

**Only works with premium accounts**

- minimal playback control (play/pause, prev/next, seeking)
- library browser (saved albums)
- album search
- artist view
- credentials management with Secret Service over DBus

## Building

### With Gnome Builder

Should be as simple as opening the project and hitting run :)

### Manually

Requires Rust (stable), GTK3, and a couple other things. See for instance this [Dockerfile](.github/actions/test/Dockerfile) for dependencies required in Fedora 33. 

Then, with meson:

```
meson target --prefix="$HOME/.local"
ninja -C target
```