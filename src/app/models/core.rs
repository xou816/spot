use super::main::*;
use std::collections::HashMap;

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
            total: usize::MAX,
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
    batch_size: usize,
    max_batch: usize,
    batches: HashMap<usize, Vec<String>>,
    indexed_songs: HashMap<String, SongDescription>,
}

impl SongList {
    pub fn new(initial: SongBatch) -> Self {
        let mut s = Self {
            total: initial.batch.total,
            batch_size: initial.batch.batch_size,
            max_batch: Default::default(),
            batches: Default::default(),
            indexed_songs: Default::default(),
        };
        s.add(initial);
        s
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ SongDescription> {
        self.iter_from(0)
    }

    pub fn iter_from(&self, i: usize) -> impl Iterator<Item = &'_ SongDescription> {
        let indexed_songs = &self.indexed_songs;
        self.iter_ids_from(i)
            .filter_map(move |id| indexed_songs.get(id))
    }

    pub fn len(&self) -> usize {
        self.iter_ids_from(0).count()
    }

    pub fn iter_ids_from(&self, i: usize) -> impl Iterator<Item = &'_ String> {
        let batch_size = self.batch_size;
        let index = i / batch_size;
        let batches = &self.batches;
        (index..=self.max_batch)
            .into_iter()
            .map(move |i| batches.get(&i))
            .take_while(|b| b.is_some())
            .flat_map(|b| b.unwrap().iter())
            .skip(i % batch_size)
    }

    pub fn remove(&mut self, ids: &[String]) {
        let mut batches = HashMap::<usize, Vec<String>>::default();
        self.iter_ids_from(0)
            .filter(|s| !ids.contains(s))
            .for_each(|next| {
                let index = batches.len().saturating_sub(1);
                let count = batches
                    .get(&index)
                    .map(|b| b.len() % self.batch_size)
                    .unwrap_or(0);
                if count == 0 {
                    batches.insert(batches.len(), vec![next.clone()]);
                } else {
                    batches.get_mut(&index).unwrap().push(next.clone());
                }
            });
        self.max_batch = batches.len().saturating_sub(1);
        self.batches = batches;
    }

    pub fn add(&mut self, SongBatch { songs, batch }: SongBatch) {
        assert_eq!(batch.batch_size, self.batch_size);
        assert_eq!(batch.total, self.total);
        let index = batch.offset / batch.batch_size;
        let ids = songs.iter().map(|s| s.id.clone()).collect();
        self.batches.insert(index, ids);
        self.max_batch = usize::max(self.max_batch, index);
        for song in songs {
            self.indexed_songs.insert(song.id.clone(), song);
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
        let list = SongList::new(batch(0));

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().id, "song0");
        assert_eq!(list_iter.next().unwrap().id, "song1");
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_index() {
        let list = SongList::new(batch(0));

        let song1 = list.index(1);
        assert!(song1.is_some());

        let song3 = list.index(3);
        assert!(song3.is_none());
    }

    #[test]
    fn test_add() {
        let mut list = SongList::new(batch(0));
        list.add(batch(1));

        let song3 = list.index(3);
        assert!(song3.is_some());
        let mut list_iter = list.iter();
        assert!(list_iter.next().is_some());
        assert!(list_iter.next().is_some());
        assert!(list_iter.next().is_some());
        assert!(list_iter.next().is_some());
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_iter_non_contiguous() {
        let mut list = SongList::new(batch(0));
        list.add(batch(2));

        assert_eq!(list.len(), 2);

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().id, "song0");
        assert_eq!(list_iter.next().unwrap().id, "song1");
        assert!(list_iter.next().is_none());

        let mut list_iter = list.iter_from(4);
        assert_eq!(list_iter.next().unwrap().id, "song4");
        assert_eq!(list_iter.next().unwrap().id, "song5");
        assert!(list_iter.next().is_none());
    }

    #[test]
    fn test_status() {
        let list = SongList::new(batch(0));
        assert_eq!(list.status(0), SongIndexStatus::Present);
        assert_eq!(list.status(6), SongIndexStatus::Absent);
        assert_eq!(list.status(10), SongIndexStatus::OutOfBounds);
    }

    #[test]
    fn test_remove() {
        let mut list = SongList::new(batch(0));
        list.add(batch(1));

        list.remove(&["song0".to_string()]);

        assert_eq!(list.len(), 3);

        let mut list_iter = list.iter();
        assert_eq!(list_iter.next().unwrap().id, "song1");
        assert_eq!(list_iter.next().unwrap().id, "song2");
        assert_eq!(list_iter.next().unwrap().id, "song3");
        assert!(list_iter.next().is_none());
    }
}
