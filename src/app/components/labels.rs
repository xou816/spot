use gettextrs::*;

lazy_static! {
    // translators: This is part of a contextual menu attached to a single track; this entry allows viewing the album containing a specific track.
    pub static ref VIEW_ALBUM: String = gettext("View album");

    // translators: This is part of a contextual menu attached to a single track; the intent is to copy the link (public URL) to a specific track.
    pub static ref COPY_LINK: String = gettext("Copy link");

    // translators: This is part of a contextual menu attached to a single track; this entry adds a track at the end of the play queue.
    pub static ref ADD_TO_QUEUE: String = gettext("Add to queue");

    // translators: This is part of a contextual menu attached to a single track; this entry removes a track from the play queue.
    pub static ref REMOVE_FROM_QUEUE: String = gettext("Remove from queue");

    // translators: This is part of a contextual menu attached to a single track; this entry adds a track to the library.
    pub static ref ADD_TO_LIBRARY: String = gettext("Add to library");

    // translators: This is part of a contextual menu attached to a single track; this entry removes a track from the library.
    pub static ref REMOVE_FROM_LIBRARY: String = gettext("Remove from library");
}

pub fn add_to_playlist_label(playlist: &str) -> String {
    // this is just to fool xgettext, it doesn't like macros (or rust for that matter) :(
    if cfg!(debug_assertions) {
        // translators: This is part of a larger text that says "Add to <playlist name>". This text should be as short as possible.
        gettext("Add to {}");
    }
    gettext!("Add to {}", playlist)
}

pub fn n_songs_selected_label(n: usize) -> String {
    // this is just to fool xgettext, it doesn't like macros (or rust for that matter) :(
    if cfg!(debug_assertions) {
        // translators: This shows up when in selection mode. This text should be as short as possible.
        ngettext("{} song selected", "{} songs selected", n as u32);
    }
    ngettext!("{} song selected", "{} songs selected", n as u32, n)
}

pub fn more_from_label(artist: &str) -> String {
    // this is just to fool xgettext, it doesn't like macros (or rust for that matter) :(
    if cfg!(debug_assertions) {
        // translators: This is part of a contextual menu attached to a single track; the full text is "More from <artist>".
        gettext("More from {}");
    }
    gettext!("More from {}", glib::markup_escape_text(artist))
}

pub fn album_by_artist_label(album: &str, artist: &str) -> String {
    // this is just to fool xgettext, it doesn't like macros (or rust for that matter) :(
    if cfg!(debug_assertions) {
        // translators: This is part of a larger label that reads "<Album> by <Artist>"
        gettext("{} by {}");
    }
    gettext!(
        "{} by {}",
        glib::markup_escape_text(album),
        glib::markup_escape_text(artist)
    )
}
