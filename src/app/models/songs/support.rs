use std::collections::HashMap;
use std::convert::TryInto;

use crate::app::models::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListChange {
    Inserted(ChangeRange),
    Swapped(u32, u32),
    Removed(ChangeRange),
    Resized { from: u32, to: u32 },
}

impl ListChange {
    pub fn swapped(a: impl TryInto<u32>, b: impl TryInto<u32>) -> Option<Self> {
        let a = a.try_into().ok()?;
        let b = b.try_into().ok()?;
        if b != a {
            Some(Self::Swapped(a, b))
        } else {
            None
        }
    }

    pub fn inserted(a: impl TryInto<u32>, b: impl TryInto<u32>) -> Option<Self> {
        ChangeRange::new(a, b).map(Self::Inserted)
    }

    pub fn removed(a: impl TryInto<u32>, b: impl TryInto<u32>) -> Option<Self> {
        ChangeRange::new(a, b).map(Self::Removed)
    }

    pub fn into_tuple(self) -> (u32, u32, u32) {
        let tuples = match self {
            Self::Inserted(ChangeRange(a, b)) => (a, 0, b - a + 1),
            Self::Removed(ChangeRange(a, b)) => (a, b - a + 1, 0),
            Self::Swapped(a, b) => (u32::min(a, b), 2, 2),
            Self::Resized { from, to } => (0, from, to),
        };
        debug!("changes: {:?}", &tuples);
        tuples
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChangeRange(pub u32, pub u32);

impl ChangeRange {
    pub fn new(a: impl TryInto<u32>, b: impl TryInto<u32>) -> Option<Self> {
        let a = a.try_into().ok()?;
        let b = b.try_into().ok()?;
        if b.saturating_sub(a) > 0 {
            Some(Self(a, b))
        } else {
            None
        }
    }

    pub fn union(self, b: Self) -> Self {
        let Self(a0, a1) = self;
        let Self(b0, b1) = b;
        let start = u32::min(a0, b0);
        let end = b1 - b0 + a1 - a0 + start + 1;
        Self(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct SongList {
    total: usize,
    total_loaded: usize,
    batch_size: usize,
    last_batch_key: usize,
    batches: HashMap<usize, Vec<String>>,
    indexed_songs: HashMap<String, SongModel>,
}

impl SongList {
    pub fn new_sized(batch_size: usize) -> Self {
        Self {
            total: 0,
            total_loaded: 0,
            batch_size,
            last_batch_key: 0,
            batches: Default::default(),
            indexed_songs: Default::default(),
        }
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn iter(&self) -> impl Iterator<Item = &SongModel> {
        let indexed_songs = &self.indexed_songs;
        self.iter_ids_from(0)
            .filter_map(move |(_, id)| indexed_songs.get(id))
    }

    pub fn partial_len(&self) -> usize {
        self.total_loaded
    }

    fn estimated_len(&self, up_to_batch_index: usize) -> usize {
        let batches = &self.batches;
        let batch_size = self.batch_size;
        let batch_count = (0..up_to_batch_index)
            .into_iter()
            .filter(move |i| batches.contains_key(i))
            .count();
        batch_size * batch_count
    }

    pub fn len(&self) -> usize {
        self.total
    }

    fn iter_ids_from(&self, i: usize) -> impl Iterator<Item = (usize, &'_ String)> {
        let batch_size = self.batch_size;
        let index = i / batch_size;
        self.iter_range(index, self.last_batch_key)
            .skip(i % batch_size)
    }

    pub fn find_index(&self, song_id: &str) -> Option<usize> {
        self.iter_ids_from(0)
            .find(|(_, id)| &id[..] == song_id)
            .map(|(pos, _)| pos)
    }

    fn iter_range(&self, a: usize, b: usize) -> impl Iterator<Item = (usize, &'_ String)> {
        let batch_size = self.batch_size;
        let batches = &self.batches;
        (a..=b)
            .into_iter()
            .filter_map(move |i| batches.get_key_value(&i))
            .flat_map(move |(k, b)| {
                b.iter()
                    .enumerate()
                    .map(move |(i, id)| (i + *k * batch_size, id))
            })
    }

    fn batches_add(batches: &mut HashMap<usize, Vec<String>>, batch_size: usize, id: &str) {
        let index = batches.len().saturating_sub(1);
        let count = batches
            .get(&index)
            .map(|b| b.len() % batch_size)
            .unwrap_or(0);
        if count == 0 {
            batches.insert(batches.len(), vec![id.to_string()]);
        } else {
            batches.get_mut(&index).unwrap().push(id.to_string());
        }
    }

    pub fn clear(&mut self) -> Option<ListChange> {
        let last = self.partial_len().saturating_sub(1);
        *self = Self::new_sized(self.batch_size);
        ListChange::removed(0, last)
    }

    pub fn remove(&mut self, ids: &[String]) -> Option<ListChange> {
        let len = self.total_loaded;
        let mut batches = HashMap::<usize, Vec<String>>::default();
        self.iter_ids_from(0)
            .filter(|(_, s)| !ids.contains(s))
            .for_each(|(_, next)| {
                Self::batches_add(&mut batches, self.batch_size, next);
            });
        self.last_batch_key = batches.len().saturating_sub(1);
        self.batches = batches;
        let removed = ids.len();
        self.total = self.total.saturating_sub(removed);
        self.total_loaded = self.total_loaded.saturating_sub(removed);
        Some(ListChange::Resized {
            from: len as u32,
            to: self.total_loaded as u32,
        })
    }

    pub fn append(&mut self, songs: Vec<SongDescription>) -> Option<ListChange> {
        let songs_len = songs.len();
        let insertion_start = self.estimated_len(self.last_batch_key + 1);
        self.total = self.total.saturating_add(songs_len);
        self.total_loaded = self.total_loaded.saturating_add(songs_len);
        for song in songs {
            Self::batches_add(&mut self.batches, self.batch_size, &song.id);
            self.indexed_songs
                .insert(song.id.clone(), SongModel::new(song));
        }
        self.last_batch_key = self.batches.len().saturating_sub(1);
        ListChange::inserted(
            insertion_start,
            (insertion_start + songs_len).saturating_sub(1),
        )
    }

    pub fn add(&mut self, song_batch: SongBatch) -> Option<ListChange> {
        let range = if song_batch.batch.batch_size != self.batch_size {
            song_batch
                .resize(self.batch_size)
                .into_iter()
                .map(|new_batch| {
                    debug!("adding batch {:?}", &new_batch.batch);
                    self.add_one(new_batch)
                })
                .reduce(|acc, cur| Some(acc?.union(cur?)).or(acc).or(cur))
                .unwrap_or(None)
        } else {
            self.add_one(song_batch)
        };
        range.map(ListChange::Inserted)
    }

    fn add_one(&mut self, SongBatch { songs, batch }: SongBatch) -> Option<ChangeRange> {
        assert_eq!(batch.batch_size, self.batch_size);

        let index = batch.offset / batch.batch_size;

        if self.batches.contains_key(&index) {
            warn!("batch already loaded");
            return None;
        }

        let insertion_start = self.estimated_len(index);
        let len = songs.len();
        let ids = songs
            .into_iter()
            .map(|song| {
                let song_id = song.id.clone();
                self.indexed_songs
                    .insert(song_id.clone(), SongModel::new(song));
                song_id
            })
            .collect();

        self.batches.insert(index, ids);
        self.total = batch.total;
        self.total_loaded += len;
        self.last_batch_key = usize::max(self.last_batch_key, index);

        ChangeRange::new(insertion_start, (insertion_start + len).saturating_sub(1))
    }

    fn index_mut(&mut self, i: usize) -> Option<&mut String> {
        let batch_size = self.batch_size;
        let i_batch = i / batch_size;
        self.batches
            .get_mut(&i_batch)
            .and_then(|s| s.get_mut(i % batch_size))
    }

    pub fn swap(&mut self, a: usize, b: usize) -> Option<ListChange> {
        if a == b {
            return None;
        }
        let a_value = self.index_mut(a).map(std::mem::take);
        let a_value = a_value.as_ref();
        let new_a_value = self
            .index_mut(b)
            .and_then(|v| Some(std::mem::replace(v, a_value?.clone())))
            .or_else(|| a_value.cloned());
        let a_mut = self.index_mut(a);
        if let (Some(a_mut), Some(a_value)) = (a_mut, new_a_value) {
            *a_mut = a_value;
        }
        ListChange::swapped(a, b)
    }

    pub fn index(&self, i: usize) -> Option<&SongModel> {
        let batch_size = self.batch_size;
        let batch_id = i / batch_size;
        let indexed_songs = &self.indexed_songs;
        self.batches
            .get(&batch_id)
            .and_then(|batch| batch.get(i % batch_size))
            .and_then(move |id| indexed_songs.get(id))
    }

    pub fn index_continuous(&self, i: usize) -> Option<&SongModel> {
        let batch_size = self.batch_size;
        let bi = i / batch_size;
        let batch = (0..=self.last_batch_key)
            .into_iter()
            .filter_map(move |i| self.batches.get(&i))
            .nth(bi)?;
        batch
            .get(i % batch_size)
            .and_then(move |id| self.indexed_songs.get(id))
    }

    pub fn needed_batch_for(&self, i: usize) -> Option<Batch> {
        let total = self.total;
        let batch_size = self.batch_size;
        let batch_id = i / batch_size;
        if self.batches.contains_key(&batch_id) {
            None
        } else {
            Some(Batch {
                batch_size,
                total,
                offset: batch_id * batch_size,
            })
        }
    }

    pub fn song_batch_for(&self, i: usize) -> Option<SongBatch> {
        let total = self.total;
        let batch_size = self.batch_size;
        let batch_id = i / batch_size;
        let indexed_songs = &self.indexed_songs;
        self.batches.get(&batch_id).map(|songs| SongBatch {
            songs: songs
                .iter()
                .filter_map(move |id| Some(indexed_songs.get(id)?.into_description()))
                .collect(),
            batch: Batch {
                batch_size,
                total,
                offset: batch_id * batch_size,
            },
        })
    }

    pub fn last_batch(&self) -> Option<Batch> {
        if self.total_loaded == 0 {
            None
        } else {
            Some(Batch {
                batch_size: self.batch_size,
                total: self.total,
                offset: self.last_batch_key * self.batch_size,
            })
        }
    }

    pub fn get(&self, id: &str) -> Option<&SongModel> {
        self.indexed_songs.get(id)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    impl SongList {
        fn new_from_initial_batch(initial: SongBatch) -> Self {
            let mut s = Self::new_sized(initial.batch.batch_size);
            s.add(initial);
            s
        }
    }

    fn song(id: &str) -> SongDescription {
        SongDescription {
            id: id.to_string(),
            uri: "".to_string(),
            title: "Title".to_string(),
            artists: vec![],
            album: AlbumRef {
                id: "".to_string(),
                name: "".to_string(),
            },
            duration: 1000,
            art: None,
            track_number: None,
        }
    }

    fn batch(id: usize) -> SongBatch {
        let offset = id * 2;
        SongBatch {
            batch: Batch {
                offset,
                batch_size: 2,
                total: 10,
            },
            songs: vec![
                song(&format!("song{}", offset)),
                song(&format!("song{}", offset + 1)),
            ],
        }
    }

    #[test]
    fn test_iter() {
        let list = SongList::new_from_initial_batch(batch(0));

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().description().id, "song0");
        assert_eq!(list_iter.next().unwrap().description().id, "song1");
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_index() {
        let list = SongList::new_from_initial_batch(batch(0));

        let song1 = list.index(1);
        assert!(song1.is_some());

        let song3 = list.index(3);
        assert!(song3.is_none());
    }

    #[test]
    fn test_add() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.add(batch(1));

        let song3 = list.index(3);
        assert!(song3.is_some());
        let list_iter = list.iter();
        assert_eq!(list_iter.count(), 4);
    }

    #[test]
    fn test_add_with_range() {
        let mut list = SongList::new_from_initial_batch(batch(0));

        let range = list.add(batch(1));
        assert_eq!(range, Some(ListChange::Inserted(ChangeRange(2, 3))));
        assert_eq!(list.partial_len(), 4);

        let range = list.add(batch(3));
        assert_eq!(range, Some(ListChange::Inserted(ChangeRange(4, 5))));
        assert_eq!(list.partial_len(), 6);

        let range = list.add(batch(2));
        assert_eq!(range, Some(ListChange::Inserted(ChangeRange(4, 5))));
        assert_eq!(list.partial_len(), 8);

        let range = list.add(batch(2));
        assert_eq!(range, None);
        assert_eq!(list.partial_len(), 8);
    }

    #[test]
    fn test_find_non_contiguous() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.add(batch(3));

        let index = list.find_index("song6");

        assert_eq!(index, Some(6));
    }

    #[test]
    fn test_iter_non_contiguous() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.add(batch(2));

        assert_eq!(list.partial_len(), 4);

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().description().id, "song0");
        assert_eq!(list_iter.next().unwrap().description().id, "song1");
        assert_eq!(list_iter.next().unwrap().description().id, "song4");
        assert_eq!(list_iter.next().unwrap().description().id, "song5");
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_remove() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.add(batch(1));

        list.remove(&["song0".to_string()]);

        assert_eq!(list.partial_len(), 3);

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().description().id, "song1");
        assert_eq!(list_iter.next().unwrap().description().id, "song2");
        assert_eq!(list_iter.next().unwrap().description().id, "song3");
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_batch_for() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.add(batch(1));
        list.add(batch(2));
        list.add(batch(3));

        assert_eq!(list.partial_len(), 8);

        let batch = list.song_batch_for(3);
        assert_eq!(batch.unwrap().batch.offset, 2);
    }

    #[test]
    fn test_append() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.append(vec![song("song2")]);
        list.append(vec![song("song3")]);
        list.append(vec![song("song4")]);

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().description().id, "song0");
        assert_eq!(list_iter.next().unwrap().description().id, "song1");
        assert_eq!(list_iter.next().unwrap().description().id, "song2");
        assert_eq!(list_iter.next().unwrap().description().id, "song3");
        assert_eq!(list_iter.next().unwrap().description().id, "song4");
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_swap() {
        let mut list = SongList::new_sized(10);
        list.append(vec![song("song0"), song("song1"), song("song2")]);

        list.swap(0, 3); // should be a no-op
        list.swap(2, 3); // should be a no-op
        list.swap(0, 2);
        list.swap(0, 1);
        list.swap(2, 2); // should be no-op
        list.swap(2, 3); // should be no-op

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().description().id, "song1");
        assert_eq!(list_iter.next().unwrap().description().id, "song2");
        assert_eq!(list_iter.next().unwrap().description().id, "song0");
        assert!(list_iter.next().is_none());
    }
}
