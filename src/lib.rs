//! In memory ranked search tool, inspired by https://github.com/BurntSushi/imdb-rename/

// #![feature(min_const_generics)]

mod collection;
mod index;
mod ngram;
mod scorer;

pub use collection::{Collection, CollectionBuilder};
pub use index::{Index, IndexBuilder};
