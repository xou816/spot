use rand::{rngs::SmallRng, RngCore, SeedableRng};

// A random, resizable mapping (i-th element to play => j-th track) used to handle shuffled playlists
// It's lazy: initially we don't compute what index i maps to
// It's resizable: if our playlist grows or shrinks, we have to keep the generated mappings stable
// (we don't want to reshuffle!)
#[derive(Debug)]
pub struct LazyRandomIndex {
    rng: SmallRng,
    indices: Vec<usize>,
    // How many mapping were generated
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

    // Resets the mapping, but make sure some index `first` will be mapped from 0
    // This is used to "pin" the position of a track when switching in and out of shuffle mode
    // See tests below for an example
    pub fn reset_picking_first(&mut self, first: usize) {
        self.generated = 0;
        if let Some(index) = self.indices.iter().position(|i| *i == first) {
            self.pick_next(index);
        }
    }

    // Grow or shrink
    pub fn resize(&mut self, size: usize) {
        if size >= self.indices.len() {
            self.grow(size);
        } else {
            self.shrink(size);
        }
    }

    // Resize the underlying Vec, but we don't generate mappings yet
    // We don't update the `generated` count: for now whatever is beyond that index is just the non-shuffled index
    pub fn grow(&mut self, size: usize) {
        let current_size = self.indices.len();
        self.indices.extend(current_size..size);
    }

    pub fn shrink(&mut self, size: usize) {
        self.generated = usize::min(size, self.generated);
        self.indices.truncate(size);
    }

    // Get the index (for instance in a playlist) of the i-th next element to play
    pub fn get(&self, i: usize) -> Option<usize> {
        if i >= self.generated || i >= self.indices.len() {
            None
        } else {
            Some(self.indices[i])
        }
    }

    // Generate all mappings until the mapping for i has been generated
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

    // Generate the next mapping
    pub fn next(&mut self) -> Option<usize> {
        if self.indices.len() < self.generated {
            return None;
        }

        let last = self.generated;
        // Pick a random index in the range [k, n[ where
        // k is the next index to map
        // n is the size of our targeted list

        // The element at that index will be swapped with whatever is in pos k
        // That is, we never mess with the already picked indices, we want this to be stable:
        // a0, a1, ..., ak-1, ak, ..., an
        // <-already mapped->
        //                    <-not mapped->
        // We just pick something between k and n and make it ak

        // Example gen with size 3
        // [0, 1, 2], generated = 0
        // [1, 0, 2], generated = 1, we swapped 0 and 1
        // [1, 0, 2], generated = 2, we swapped 1 and 1 (no-op)
        // [1, 0, 2], generated = 3, no-op again (no choice, only one element left to place)
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

        // It's controlled randomness for the test :)
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
        // Check that we have all our indices mapped
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

        // After growing the index from 5 to 10, we want to check
        // that the previously generated mappings are unaffected.
        assert_eq!(values, same_values);
    }

    #[test]
    fn test_reset() {
        let mut index = LazyRandomIndex::from(rng_for_test());

        // Example use
        // Assume we have 5 songs in our shuffled playlist, and we generate those 5 few random mappings
        index.grow(5);
        index.next_until(5);

        // We exit shuffle mode at some point

        // Shuffle is toggled again: we want index 2 to come up first in the shuffled playlist
        // because it is what's currently playing.
        index.reset_picking_first(2);
        assert_eq!(index.get(0), Some(2));
    }
}
