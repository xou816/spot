# Spot

Gtk/Rust native Spotify client for the GNOME desktop. **Only works with premium accounts!**

Based on [librespot](https://github.com/librespot-org/librespot/).

![Preview](./demo.gif)

## Installing

| Package | Maintainer | Repo |
| ------- | ---------- | ---- |
| <a href='https://flathub.org/apps/details/dev.alextren.Spot'><img width='130' alt='Download on Flathub' src='https://flathub.org/assets/badges/flathub-badge-en.png'/></a> | xou816 | https://github.com/flathub/dev.alextren.Spot |
| <a href='https://snapcraft.io/spot'><img width='130' alt="Get it from the Snap Store" src="https://snapcraft.io/static/images/badges/en/snap-store-black.svg"></a> | popey | https://github.com/popey/spot-snap |
| <a href='https://aur.archlinux.org/packages/spot-client/'><img alt="AUR version" src="https://img.shields.io/aur/version/spot-client"></a> | dpeukert | https://gitlab.com/dpeukert/pkgbuilds/tree/main/spot-client |

It is recommended to install a libsecret compliant keyring application, such as GNOME Keyring (aka seahorse).

### Usage notes

Spot caches images and HTTP responses in `~/.cache/spot`.

Spot can also be configured via `gsettings` if you want to change the audio backend, the song bitrate, etc.

## Features

**Only works with premium accounts!**

- minimal playback control (play/pause, prev/next, seeking)
- library browser (saved albums and playlists)
- album and artist search
- artist view
- credentials management with Secret Service
- MPRIS integration

### Planned

- playlist management (creation and edition)
- proper play queue implementation
- translate app
- liked tracks
- GNOME search provider?
- smarter search?
- recommendations?

## Building

### With GNOME Builder and flatpak

Pre-requisite: install the `org.freedesktop.Sdk.Extension.rust-stable` SDK extension with flatpak. Builder might do this for you automatically, but it will install an older version; make sure  the version installed matches the version of the Freedesktop SDK GNOME uses (at the time of writing: 20.08).

Open the project in GNOME Builder and make the `dev.alextren.Spot.development.json` configuration active. Then build :)

### Manually

Requires Rust (stable), GTK3, and a couple other things. Also requires libhandy1: it is not packaged on all distros at the moment, you might have to build it yourself.

**Build** dependencies on Ubuntu 20.04 for instance: ```build-essential pkg-config meson libssl-dev libglib2.0-dev-bin libgtk-3-dev libasound2-dev libpulse-dev```. 

Then, with meson:

```
meson target -Dbuildtype=debug -Doffline=false --prefix="$HOME/.local"
ninja install -C target
```

This will install a `.desktop` file among other things, and the spot executable will be put in `.local/bin` (you might want to add it to your path).

To build an optimized release build, use `-Dbuildtype=release` instead.

### Regenerating potfiles

```
ninja spot-pot -C target
ninja spot-update-po -C target
```

### Debugging

Spot uses [isahc](https://github.com/sagebind/isahc), which uses libcurl, therefore you can set the `https_proxy` env variable to help with debugging. In debug mode, Spot skips SSL certificate verification.