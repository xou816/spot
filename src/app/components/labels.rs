use gettextrs::*;

lazy_static! {
    // translators: This is part of a contextual menu attached to a single track; this entry allows viewing the album containing a specific track.
    pub static ref VIEW_ALBUM: String = gettext("View album");

    // translators: This is part of a contextual menu attached to a single track; the full text is "More from <artist>".
    pub static ref MORE_FROM: String = gettext("More from");

    // translators: This is part of a contextual menu attached to a single track; the intent is to copy the link (public URL) to a specific track.
    pub static ref COPY_LINK: String = gettext("Copy link");

    // translators: This is part of a contextual menu attached to a single track; this entry adds a track at the end of the play queue.
    pub static ref ADD_TO_QUEUE: String = gettext("Add to queue");

    // translators: This is part of a contextual menu attached to a single track; this entry removes a track from the play queue.
    pub static ref REMOVE_FROM_QUEUE: String = gettext("Remove from queue");
}
