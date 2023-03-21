// The underlying data structure for a list of songs
mod support;

// A GObject wrapper around that list
mod song_list_model;
pub use song_list_model::*;

mod song_model;
pub use song_model::*;
