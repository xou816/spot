use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;

use crate::app::models::SongDescription;

pub struct PlayQueue {
    rng: SmallRng,
    indexed_songs: HashMap<String, SongDescription>,
    running_order: Vec<String>,
    running_order_shuffled: Option<Vec<String>>,
}

impl PlayQueue {
    pub fn new(tracks: &[SongDescription]) -> Self {
        let running_order = tracks.iter().map(|t| t.id.clone()).collect();
        PlayQueue {
            rng: SmallRng::from_entropy(),
            indexed_songs: Self::index_tracks(tracks),
            running_order,
            running_order_shuffled: None,
        }
    }

    fn index_tracks(tracks: &[SongDescription]) -> HashMap<String, SongDescription> {
        let mut map: HashMap<String, SongDescription> = HashMap::with_capacity(tracks.len());
        for track in tracks.iter() {
            map.insert(track.id.clone(), track.clone());
        }
        map
    }

    fn is_shuffled(&self) -> bool {
        self.running_order_shuffled.is_some()
    }

    pub fn set_tracks(&mut self, tracks: &[SongDescription], keep_index: Option<usize>) {
        self.indexed_songs = Self::index_tracks(tracks);
        if self.is_shuffled() {
            self.shuffle(keep_index);
        }
    }

    pub fn song(&self, id: &str) -> Option<&SongDescription> {
        self.indexed_songs.get(id)
    }

    pub fn songs<'i, 's: 'i>(&'s self) -> impl Iterator<Item = &'i SongDescription> + 'i {
        let iter = self.running_order_shuffled.as_ref().unwrap_or(&self.running_order);
        let indexed = &self.indexed_songs;
        iter.iter().filter_map(move |id| indexed.get(id))
    }

    pub fn shuffle(&mut self, keep_index: Option<usize>) {
        let mut shuffled = self.running_order.clone();
        let mut final_list = if let Some(index) = keep_index {
            vec![shuffled.remove(index)]
        } else {
            vec![]
        };
        shuffled.shuffle(&mut self.rng);
        final_list.append(&mut shuffled);
        self.running_order_shuffled = Some(final_list);
    }

    pub fn toggle_shuffle(&mut self, keep_index: Option<usize>) {
        if !self.is_shuffled() {
            self.shuffle(keep_index);
        }
    }
}

pub struct PlaybackState {
    pub is_playing: bool,
    pub current_song_id: Option<String>,
    pub playlist: PlayQueue,
}
