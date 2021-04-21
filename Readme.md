# `ranked-search`

A simple implementation of an [n-gram](https://en.wikipedia.org/wiki/N-gram)-based ranked search tool in Rust.

This exposes an `Index` for searching and a simplified `Collection` to search for specific objects.

## Prior Work

This library was intended as a simplification of the search function in [`imdb-rename`](https://github.com/BurntSushi/imdb-rename)'s indexer.

I could publish this on crates.io, but it seems like there are a lot more (and more mature) existing n-gram libraries [already on the site](https://crates.io/search?q=ngram). This mostly served as a learning exercise for me.
