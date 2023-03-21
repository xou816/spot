// A structure for batched queries that I introduced before proper batch management
// Still used to load album lists for instance
// Doesn't know how many elements exist in total ahead of time
#[derive(Clone, Debug)]
pub struct Pagination<T>
where
    T: Clone,
{
    pub data: T,
    // The next offset (of things to load) is set to None whenever we get less than we asked for
    // as it probably means we've reached the end of some list
    pub next_offset: Option<usize>,
    pub batch_size: usize,
}

impl<T> Pagination<T>
where
    T: Clone,
{
    pub fn new(data: T, batch_size: usize) -> Self {
        Self {
            data,
            next_offset: Some(0),
            batch_size,
        }
    }

    pub fn reset_count(&mut self, new_length: usize) {
        self.next_offset = if new_length >= self.batch_size {
            Some(self.batch_size)
        } else {
            None
        }
    }

    pub fn set_loaded_count(&mut self, loaded_count: usize) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = if loaded_count >= self.batch_size {
                Some(offset + self.batch_size)
            } else {
                None
            }
        }
    }

    // If we remove elements from paginated data without refetching from the source,
    // we have to adjust the next offset to load
    pub fn decrement(&mut self) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = Some(offset - 1);
        }
    }

    // Same idea as decrement
    pub fn increment(&mut self) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = Some(offset + 1);
        }
    }
}
