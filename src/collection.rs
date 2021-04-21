use super::{Index, IndexBuilder};

/// `Collection` builds on the `Index` to manage a group of items with associated
/// search terms, returning the items themselves during search. This removes the
/// need for the user to manually manage `ItemId`s.
pub struct Collection<T, const N: usize> {
    index: Index<usize, N>,
    items: Vec<T>,
}

pub struct CollectionBuilder<T, const N: usize> {
    index_builder: IndexBuilder<usize, N>,
    items: Vec<T>,
}

impl<T, const N: usize> CollectionBuilder<T, N> {
    pub fn insert<U: IntoIterator<Item = S>, S: ToString>(&mut self, item: T, names: U) -> usize {
        let id = self.items.len();
        self.items.push(item);
        for name in names.into_iter() {
            self.index_builder.insert(id, name)
        }
        id
    }

    // TODO: when would anyone use this?
    pub fn add_name<S: ToString>(&mut self, item_index: usize, name: S) {
        self.index_builder.insert(item_index, name);
    }

    pub fn build(self) -> Collection<T, N> {
        let CollectionBuilder {
            index_builder,
            items,
        } = self;

        let index = index_builder.build();

        Collection { index, items }
    }
}

impl<T, const N: usize> Collection<T, N> {
    pub fn builder() -> CollectionBuilder<T, N> {
        CollectionBuilder {
            index_builder: Index::builder(),
            items: Vec::new(),
        }
    }

    /// Construct a searchable collection from items of type T with any given
    /// number of names (Iterator U)
    pub fn from_items<I, U, S>(iter: I) -> Self
    where
        I: IntoIterator<Item = (T, U)>,
        U: IntoIterator<Item = S>,
        S: ToString,
    {
        let mut index = Index::builder();
        let mut items = Vec::new();
        for (id, (item, names)) in iter.into_iter().enumerate() {
            items.push(item);
            for name in names.into_iter() {
                index.insert(id, name);
            }
        }

        Self {
            index: index.build(),
            items,
        }
    }

    /// NOTE: the returned Vec is not guaranteed to be of length
    pub async fn search(&self, input: &str, max_results: usize) -> Vec<(&T, f64)> {
        self.index
            .search(input, max_results)
            .await
            .into_iter()
            .filter_map(|(id, score)| self.items.get(*id).map(|item| (item, score)))
            .collect()
    }

    pub async fn best_match(&self, input: &str) -> Option<&T> {
        self.index
            .best_match(input)
            .await
            .map(|id| self.items.get(*id))
            .flatten()
    }

    pub fn search_sync(&self, input: &str, max_results: usize) -> Vec<(&T, f64)> {
        self.index
            .search_sync(input, max_results)
            .into_iter()
            .filter_map(|(id, score)| self.items.get(*id).map(|item| (item, score)))
            .collect()
    }

    pub fn best_match_sync(&self, input: &str) -> Option<&T> {
        self.index
            .best_match_sync(input)
            .map(|id| self.items.get(*id))
            .flatten()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    struct Item {
        data: u32,
    }

    #[test]
    fn test_collection() {
        let collection = Collection::<Item, 3>::from_items(vec![
            (Item { data: 0 }, vec!["first", "one"]),
            (Item { data: 1 }, vec!["two"]),
            (Item { data: 2 }, vec!["three", "third"]),
        ]);

        assert_eq!(
            collection.best_match_sync("first").expect("missing!").data,
            0
        )
    }

    #[test]
    fn test_collection_builder() {
        let collection = {
            let mut builder = Collection::<_, 3>::builder();
            builder.insert(Item { data: 0 }, vec!["first", "one"]);
            builder.insert(Item { data: 1 }, vec!["two"]);
            builder.insert(Item { data: 2 }, vec!["three", "third"]);
            builder.build()
        };

        assert_eq!(
            collection.best_match_sync("first").expect("missing!").data,
            0
        )
    }
}
