use std::convert::Into;
pub use super::gtypes::*;

impl Into<AlbumModel> for AlbumDescription {
    fn into(self) -> AlbumModel {
       AlbumModel::new(
            &self.artist,
            &self.title,
            &self.art,
            &self.id
        )
   }
}

#[derive(Clone, Debug)]
pub struct AlbumDescription {
    pub title: String,
    pub artist: String,
    pub uri: String,
    pub art: String,
    pub songs: Vec<SongDescription>,
    pub id: String
}

impl PartialEq for AlbumDescription {
    fn eq(&self, other: &AlbumDescription) -> bool {
        self.id == other.id
    }
}

impl Eq for AlbumDescription {}

#[derive(Clone, Debug)]
pub struct SongDescription {
    pub title: String,
    pub artist: String,
    pub uri: String,
    pub duration: u32,
    pub art: Option<String>
}

impl SongDescription {
    pub fn new(title: &str, artist: &str, uri: &str, duration: u32, art: Option<String>) -> Self {
        Self { title: title.to_string(), artist: artist.to_string(), uri: uri.to_string(), duration, art }
    }
}
#[derive(Clone, Debug)]
pub struct ArtistDescription {
    pub name: String,
    pub albums: Vec<AlbumDescription>
}
