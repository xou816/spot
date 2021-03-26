pub struct Pagination<T> {
    pub data: T,
    pub next_offset: Option<u32>,
    pub batch_size: u32,
}

impl<T> Pagination<T> {
    pub fn new(data: T, batch_size: u32) -> Self {
        Self {
            data,
            next_offset: Some(0),
            batch_size,
        }
    }

    pub fn reset_count(&mut self, new_length: u32) {
        self.next_offset = if new_length >= self.batch_size {
            Some(self.batch_size)
        } else {
            None
        }
    }

    pub fn set_loaded_count(&mut self, loaded_count: u32) {
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
