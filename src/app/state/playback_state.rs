use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};

use crate::app::models::SongDescription;

pub struct ShuffledSongs {
    rng: SmallRng,
    internal_playlist: Vec<SongDescription>,
    playlist: Vec<SongDescription>,
    shuffled: bool,
}

impl ShuffledSongs {
    pub fn new(tracks: Vec<SongDescription>) -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            internal_playlist: tracks,
            playlist: vec![],
            shuffled: false,
        }
    }

    pub fn update(&mut self, tracks: Vec<SongDescription>, keep_index: Option<usize>) {
        self.internal_playlist = tracks;
        if self.shuffled {
            self.shuffle(keep_index);
        }
    }

    pub fn songs(&self) -> &Vec<SongDescription> {
        if self.shuffled {
            &self.playlist
        } else {
            &self.internal_playlist
        }
    }

    pub fn shuffle(&mut self, keep_index: Option<usize>) {
        let mut shuffled = self.internal_playlist.clone();
        let mut final_list = if let Some(index) = keep_index {
            vec![shuffled.remove(index)]
        } else {
            vec![]
        };
        shuffled.shuffle(&mut self.rng);
        final_list.append(&mut shuffled);
        self.playlist = final_list;
    }

    pub fn toggle_shuffle(&mut self, keep_index: Option<usize>) {
        if !self.shuffled {
            self.shuffle(keep_index);
        }
        self.shuffled = !self.shuffled;
    }
}

pub struct PlaybackState {
    pub is_playing: bool,
    pub current_song_id: Option<String>,
    pub playlist: ShuffledSongs,
}