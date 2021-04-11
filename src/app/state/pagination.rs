#[derive(Clone, Debug)]
pub struct Pagination<T>
where
    T: Clone,
{
    pub data: T,
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

    pub fn decrement(&mut self) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = Some(offset - 1);
        }
    }

    pub fn increment(&mut self) {
        if let Some(offset) = self.next_offset.take() {
            self.next_offset = Some(offset + 1);
        }
    }
}
