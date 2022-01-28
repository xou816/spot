mod main;
pub use main::*;

mod songs;
pub use songs::*;

mod album_model;
pub use album_model::*;

mod artist_model;
pub use artist_model::*;

impl From<&AlbumDescription> for AlbumModel {
    fn from(album: &AlbumDescription) -> Self {
        AlbumModel::new(
            &album.artists_name(),
            &album.title,
            album.year(),
            album.art.as_ref(),
            &album.id,
        )
    }
}

impl From<AlbumDescription> for AlbumModel {
    fn from(album: AlbumDescription) -> Self {
        Self::from(&album)
    }
}

impl From<&PlaylistDescription> for AlbumModel {
    fn from(playlist: &PlaylistDescription) -> Self {
        AlbumModel::new(
            &playlist.owner.display_name,
            &playlist.title,
            // Playlists do not have their released date since they are expected to be updated anytime.
            None,
            playlist.art.as_ref(),
            &playlist.id,
        )
    }
}

impl From<PlaylistDescription> for AlbumModel {
    fn from(playlist: PlaylistDescription) -> Self {
        Self::from(&playlist)
    }
}

impl From<SongDescription> for SongModel {
    fn from(song: SongDescription) -> Self {
        SongModel::new(song)
    }
}

impl From<&SongDescription> for SongModel {
    fn from(song: &SongDescription) -> Self {
        SongModel::new(song.clone())
    }
}
