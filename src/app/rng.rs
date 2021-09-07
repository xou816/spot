use rand::{rngs::SmallRng, RngCore, SeedableRng};

#[derive(Debug)]
pub struct LazyRandomIndex {
    rng: SmallRng,
    indices: Vec<usize>,
    generated: usize,
}

impl Default for LazyRandomIndex {
    fn default() -> Self {
        Self::from(SmallRng::from_entropy())
    }
}

impl LazyRandomIndex {
    fn from(rng: SmallRng) -> Self {
        Self {
            rng,
            indices: Default::default(),
            generated: 0,
        }
    }

    pub fn reset_picking_first(&mut self, first: usize) {
        self.generated = 0;
        if let Some(index) = self.indices.iter().position(|i| *i == first) {
            self.pick_next(index);
        }
    }

    pub fn resize(&mut self, size: usize) {
        if size >= self.indices.len() {
            self.grow(size);
        } else {
            self.shrink(size);
        }
    }

    pub fn grow(&mut self, size: usize) {
        let current_size = self.indices.len();
        self.indices.extend(current_size..size);
    }

    pub fn shrink(&mut self, size: usize) {
        self.generated = usize::min(size, self.generated);
        self.indices.truncate(size);
    }

    pub fn get(&self, i: usize) -> Option<usize> {
        if i >= self.generated || i >= self.indices.len() {
            None
        } else {
            Some(self.indices[i])
        }
    }

    pub fn next_until(&mut self, i: usize) -> Option<usize> {
        if i >= self.indices.len() {
            return None;
        }

        loop {
            if self.generated > i {
                break Some(self.indices[i]);
            }
            self.next();
        }
    }

    pub fn next(&mut self) -> Option<usize> {
        if self.indices.len() < self.generated {
            return None;
        }

        let last = self.generated;
        let next = (self.rng.next_u64() as usize) % (self.indices.len() - last) + last;
        Some(self.pick_next(next))
    }

    fn pick_next(&mut self, next: usize) -> usize {
        let last = self.generated;
        self.indices.swap(last, next);
        self.generated += 1;
        self.indices[last]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn rng_for_test() -> SmallRng {
        SmallRng::seed_from_u64(0)
    }

    fn get_sequence(n: usize) -> Vec<usize> {
        let mut rng = rng_for_test();
        (0..n)
            .into_iter()
            .map(|_| rng.next_u64() as usize)
            .collect()
    }

    #[test]
    fn test_initial() {
        let seq = get_sequence(10);
        let first = Some(seq[0] % 10);

        let mut index = LazyRandomIndex::from(rng_for_test());
        index.grow(10);

        let next = index.next();
        assert_eq!(next, first);
        assert_eq!(index.get(0), first);
    }

    #[test]
    fn test_sample_all() {
        let mut index = LazyRandomIndex::from(rng_for_test());
        index.grow(2);
        index.grow(5);

        let mut values = (0..5)
            .into_iter()
            .filter_map(|_| index.next())
            .collect::<Vec<usize>>();
        let sorted = &mut values[..];
        sorted.sort();
        assert_eq!(sorted, &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_after_grow() {
        let mut index = LazyRandomIndex::from(rng_for_test());

        index.grow(5);
        index.next_until(2);
        let values = &[index.get(0), index.get(1), index.get(2)];

        index.grow(10);
        let same_values = &[index.get(0), index.get(1), index.get(2)];

        assert_eq!(values, same_values);
    }

    #[test]
    fn test_reset() {
        let mut index = LazyRandomIndex::from(rng_for_test());

        index.grow(5);
        index.next_until(5);

        index.reset_picking_first(2);
        assert_eq!(index.get(0), Some(2));
    }
}
