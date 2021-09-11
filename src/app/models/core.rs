use super::main::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InsertionRange(pub usize, pub usize);

impl InsertionRange {
    fn union(a: Option<Self>, b: Option<Self>) -> Option<Self> {
        match (a, b) {
            (Some(Self(a0, a1)), Some(Self(b0, b1))) => {
                let start = usize::min(a0, b0);
                let end = usize::max(a0 + b0, a1 + b1) - start;
                Some(Self(start, end))
            }
            (Some(a), None) | (None, Some(a)) => Some(a),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Batch {
    pub offset: usize,
    pub batch_size: usize,
    pub total: usize,
}

impl Batch {
    pub fn first_of_size(batch_size: usize) -> Self {
        Self {
            offset: 0,
            batch_size,
            total: 0,
        }
    }

    pub fn next(self) -> Option<Self> {
        let Self {
            offset,
            batch_size,
            total,
        } = self;

        Some(Self {
            offset: offset + batch_size,
            batch_size,
            total,
        })
        .filter(|b| b.offset < total)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SongIndexStatus {
    Present,
    Absent,
    OutOfBounds,
}

#[derive(Clone, Debug)]
pub struct SongList {
    total: usize,
    total_loaded: usize,
    batch_size: usize,
    last_batch_key: usize,
    batches: HashMap<usize, Vec<String>>,
    indexed_songs: HashMap<String, SongDescription>,
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

    pub fn new_from_initial_batch(initial: SongBatch) -> Self {
        let mut s = Self::new_sized(initial.batch.batch_size);
        s.add(initial);
        s
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ SongDescription> {
        self.iter_from(0)
    }

    fn iter_from(&self, i: usize) -> impl Iterator<Item = &'_ SongDescription> {
        let indexed_songs = &self.indexed_songs;
        self.iter_ids_from(i)
            .filter_map(move |id| indexed_songs.get(id))
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

    pub fn iter_ids_from(&self, i: usize) -> impl Iterator<Item = &'_ String> {
        let batch_size = self.batch_size;
        let index = i / batch_size;
        self.iter_range(index, self.last_batch_key)
            .skip(i % batch_size)
    }

    fn iter_range(&self, a: usize, b: usize) -> impl Iterator<Item = &'_ String> {
        let batches = &self.batches;
        (a..=b)
            .into_iter()
            .filter_map(move |i| batches.get(&i))
            .flat_map(|b| b.iter())
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

    pub fn remove(&mut self, ids: &[String]) {
        let mut batches = HashMap::<usize, Vec<String>>::default();
        self.iter_ids_from(0)
            .filter(|s| !ids.contains(s))
            .for_each(|next| {
                Self::batches_add(&mut batches, self.batch_size, next);
            });
        self.last_batch_key = batches.len().saturating_sub(1);
        self.batches = batches;
        self.total = self.total.saturating_sub(ids.len());
        self.total_loaded = self.total_loaded.saturating_sub(ids.len());
    }

    pub fn append(&mut self, songs: Vec<SongDescription>) {
        self.total = self.total.saturating_add(songs.len());
        self.total_loaded = self.total_loaded.saturating_add(songs.len());
        for song in songs {
            Self::batches_add(&mut self.batches, self.batch_size, &song.id);
            self.indexed_songs.insert(song.id.clone(), song);
        }
        self.last_batch_key = self.batches.len().saturating_sub(1);
    }

    pub fn add(&mut self, song_batch: SongBatch) -> Option<InsertionRange> {
        if song_batch.batch.batch_size != self.batch_size {
            song_batch
                .resize(self.batch_size)
                .into_iter()
                .map(|new_batch| self.add_one(new_batch))
                .reduce(InsertionRange::union)
                .unwrap_or(None)
        } else {
            self.add_one(song_batch)
        }
    }

    fn add_one(&mut self, SongBatch { songs, batch }: SongBatch) -> Option<InsertionRange> {
        assert_eq!(batch.batch_size, self.batch_size);

        let index = batch.offset / batch.batch_size;
        if self.batches.contains_key(&index) {
            return None;
        }

        let insertion_start = self.estimated_len(index);
        let len = songs.len();
        let ids = songs
            .into_iter()
            .map(|song| {
                let song_id = song.id.clone();
                self.indexed_songs.insert(song_id.clone(), song);
                song_id
            })
            .collect();

        self.batches.insert(index, ids);
        self.total = batch.total;
        self.total_loaded += len;
        self.last_batch_key = usize::max(self.last_batch_key, index);

        Some(InsertionRange(insertion_start, len))
    }

    fn index_mut(&mut self, i: usize) -> Option<&mut String> {
        let batch_size = self.batch_size;
        let i_batch = i / batch_size;
        self.batches
            .get_mut(&i_batch)
            .and_then(|s| s.get_mut(i % batch_size))
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let a_value = self.index_mut(a).map(|v| std::mem::take(v));
        let a_value = a_value.as_ref();
        let new_a_value = self
            .index_mut(b)
            .and_then(|v| Some(std::mem::replace(v, a_value?.clone())))
            .or_else(|| a_value.cloned());
        let a_mut = self.index_mut(a);
        if let (Some(a_mut), Some(a_value)) = (a_mut, new_a_value) {
            *a_mut = a_value;
        }
    }

    pub fn index(&self, i: usize) -> Option<&SongDescription> {
        let batch_size = self.batch_size;
        let batch_id = i / batch_size;
        let indexed_songs = &self.indexed_songs;
        self.batches
            .get(&batch_id)
            .and_then(|batch| batch.get(i % batch_size))
            .and_then(move |id| indexed_songs.get(id))
    }

    pub fn has_batch_for(&self, i: usize) -> (Batch, bool) {
        let total = self.total;
        let batch_size = self.batch_size;
        let batch_id = i / batch_size;
        (
            Batch {
                batch_size,
                total,
                offset: batch_id * batch_size,
            },
            self.batches.contains_key(&batch_id),
        )
    }

    pub fn song_batch_for(&self, i: usize) -> Option<SongBatch> {
        let total = self.total;
        let batch_size = self.batch_size;
        let batch_id = i / batch_size;
        let indexed_songs = &self.indexed_songs;
        self.batches.get(&batch_id).map(|songs| SongBatch {
            songs: songs
                .iter()
                .filter_map(move |id| indexed_songs.get(id))
                .cloned()
                .collect(),
            batch: Batch {
                batch_size,
                total,
                offset: batch_id * batch_size,
            },
        })
    }

    pub fn last_batch(&self) -> Batch {
        Batch {
            batch_size: self.batch_size,
            total: self.total,
            offset: self.last_batch_key * self.batch_size,
        }
    }

    pub fn get(&self, id: &str) -> Option<&SongDescription> {
        self.indexed_songs.get(id)
    }

    pub fn status(&self, i: usize) -> SongIndexStatus {
        if i >= self.total {
            return SongIndexStatus::OutOfBounds;
        }

        let batch_size = self.batch_size;
        let batch_id = i / batch_size;
        self.batches
            .get(&batch_id)
            .map(|batch| match batch.get(i % batch_size) {
                Some(_) => SongIndexStatus::Present,
                None => SongIndexStatus::OutOfBounds,
            })
            .unwrap_or(SongIndexStatus::Absent)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
        assert_eq!(list_iter.next().unwrap().id, "song0");
        assert_eq!(list_iter.next().unwrap().id, "song1");
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
        assert_eq!(range, Some(InsertionRange(2, 2)));
        assert_eq!(list.partial_len(), 4);

        let range = list.add(batch(3));
        assert_eq!(range, Some(InsertionRange(4, 2)));
        assert_eq!(list.partial_len(), 6);

        let range = list.add(batch(2));
        assert_eq!(range, Some(InsertionRange(4, 2)));
        assert_eq!(list.partial_len(), 8);

        let range = list.add(batch(2));
        assert_eq!(range, None);
        assert_eq!(list.partial_len(), 8);
    }

    #[test]
    fn test_iter_non_contiguous() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.add(batch(2));

        assert_eq!(list.partial_len(), 4);

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().id, "song0");
        assert_eq!(list_iter.next().unwrap().id, "song1");
        assert_eq!(list_iter.next().unwrap().id, "song4");
        assert_eq!(list_iter.next().unwrap().id, "song5");
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_status() {
        let list = SongList::new_from_initial_batch(batch(0));
        assert_eq!(list.status(0), SongIndexStatus::Present);
        assert_eq!(list.status(6), SongIndexStatus::Absent);
        assert_eq!(list.status(10), SongIndexStatus::OutOfBounds);
    }

    #[test]
    fn test_remove() {
        let mut list = SongList::new_from_initial_batch(batch(0));
        list.add(batch(1));

        list.remove(&["song0".to_string()]);

        assert_eq!(list.partial_len(), 3);

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().id, "song1");
        assert_eq!(list_iter.next().unwrap().id, "song2");
        assert_eq!(list_iter.next().unwrap().id, "song3");
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
        assert_eq!(list_iter.next().unwrap().id, "song0");
        assert_eq!(list_iter.next().unwrap().id, "song1");
        assert_eq!(list_iter.next().unwrap().id, "song2");
        assert_eq!(list_iter.next().unwrap().id, "song3");
        assert_eq!(list_iter.next().unwrap().id, "song4");
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
        assert_eq!(list_iter.next().unwrap().id, "song1");
        assert_eq!(list_iter.next().unwrap().id, "song2");
        assert_eq!(list_iter.next().unwrap().id, "song0");
        assert!(list_iter.next().is_none());
    }
}
