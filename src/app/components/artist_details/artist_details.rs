use gtk::prelude::*;
use gladis::Gladis;


#[derive(Clone, Gladis)]
struct ArtistDetailsWidget {
    pub artist_name: gtk::Label,
    pub artist_albums: gtk::FlowBox
}

impl ArtistDetailsWidget {

    fn new() -> Self {
        Self::from_resource(resource!("/components/artist_details.ui")).unwrap()
    }
}

pub struct ArtistDetails {

}
