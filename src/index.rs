use std::collections::HashMap;

use crate::ngram::NGram;

// TODO: more type system abstractions for ITEM_ID/ Term Id or index/ frequency?

// Externally (user) managed identifier for items indexed by an `Index`
// pub struct ItemId(usize);
// impl From<usize> for ItemId {
//     fn from(value: usize) -> Self {
//         ItemId(value)
//     }
// }

// TODO: Vec for small ngram number, HashMap for large

pub struct Index<ItemId, const N: usize> {
    /// List of mappings from Ngram to the indices of search terms that include
    /// it (the Ngram's frequency in each). The search term index indicates the
    /// term position in the `search_terms` vec
    ngrams: Vec<(NGram<N>, Vec<(usize, u32)>)>,
    avg_len: f64,
    /// Index search term index to item id, so that multiple names can point to
    /// the same item. Also store the length (in ngrams) of the search term
    search_terms: Vec<(ItemId, usize)>,
}

pub struct IndexBuilder<ItemId, const N: usize> {
    ngrams: HashMap<NGram<N>, Vec<(usize, u32)>>,
    avg_len: f64,
    search_terms: Vec<(ItemId, usize)>,
}

impl<ItemId, const N: usize> IndexBuilder<ItemId, N> {
    pub fn insert<S: ToString>(&mut self, item_id: ItemId, search_term: S) {
        // Decompose the search term to ngrams
        let ngrams = NGram::<N>::vec_from(search_term);
        let num_ngrams = ngrams.len();

        // Store the item that the term at this index points to (and how long it is)
        let term_index = self.search_terms.len();
        self.search_terms.push((item_id, num_ngrams));

        // Insert all ngrams from the term, counting their frequencies in the term
        for ngram in ngrams {
            let entry = self.ngrams.entry(ngram).or_default();
            if let Some((_, freq)) = entry.iter_mut().find(|(idx, _)| term_index == *idx) {
                *freq += 1;
            } else {
                entry.push((term_index, 1));
            }
        }

        // Update the average length of search terms
        self.avg_len += (num_ngrams as f64 - self.avg_len) / ((term_index + 1) as f64);
    }

    pub fn build(self) -> Index<ItemId, N> {
        let IndexBuilder {
            ngrams,
            avg_len,
            search_terms,
        } = self;

        let mut ngrams = ngrams.into_iter().collect::<Vec<_>>();
        ngrams.sort_by(|(a, _), (b, _)| a.cmp(b));

        Index {
            ngrams,
            avg_len,
            search_terms,
        }
    }
}

impl<const N: usize> Index<usize, N> {
    pub fn from_distinct<I: IntoIterator<Item = S>, S: ToString>(iter: I) -> Self {
        let mut builder = Self::builder();
        for (id, term) in iter.into_iter().enumerate() {
            builder.insert(id, term);
        }
        builder.build()
    }
}

impl<ItemId, const N: usize> Index<ItemId, N> {
    pub fn builder() -> IndexBuilder<ItemId, N> {
        IndexBuilder {
            ngrams: HashMap::new(),
            avg_len: 0f64,
            search_terms: Vec::new(),
        }
    }

    pub fn from_identified<I: IntoIterator<Item = (ItemId, S)>, S: ToString>(iter: I) -> Self {
        let mut builder = Self::builder();
        for (id, term) in iter.into_iter() {
            builder.insert(id, term);
        }
        builder.build()
    }

    pub async fn search(&self, input: &str, max_results: usize) -> Vec<(&ItemId, f64)> {
        // TODO: some thready business?
        self.search_sync(input, max_results)
    }

    pub async fn best_match(&self, input: &str) -> Option<&ItemId> {
        if let Some((id, score)) = self.search(input, 1).await.into_iter().next() {
            if score == 0.0 {
                None
            } else {
                Some(id)
            }
        } else {
            None
        }
    }

    pub fn search_sync(&self, input: &str, max_results: usize) -> Vec<(&ItemId, f64)> {
        let total_terms = self.search_terms.len();
        let mut term_scores = vec![0.0; total_terms];
        for ngram in NGram::<{ N }>::vec_from(input) {
            if let Some((_, terms)) = self.ngrams.iter().find(|(ng, _)| ng.eq(&ngram)) {
                let num_matched_terms = terms.len();
                let idf = f64::ln_1p(
                    ((total_terms - num_matched_terms) as f64 + 0.5)
                        / (num_matched_terms as f64 + 0.5),
                );

                for (term_index, freq) in terms {
                    let term_len = self.search_terms[*term_index].1;
                    if let Some(score) = term_scores.get_mut(*term_index) {
                        *score += crate::scorer::score(term_len, self.avg_len, *freq, idf);
                    }
                }
            }
        }

        let mut results: Vec<(&ItemId, f64)> = term_scores
            .into_iter()
            .enumerate()
            .map(|(term_index, score)| (&self.search_terms[term_index].0, score))
            .collect();
        results.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        results.truncate(max_results);

        results
    }

    pub fn best_match_sync(&self, input: &str) -> Option<&ItemId> {
        if let Some((id, score)) = self.search_sync(input, 1).into_iter().next() {
            if score == 0.0 {
                None
            } else {
                Some(id)
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test using some of the data from `ECMPurpose` (the original use for this search)
    const OPTIONS: &[&str] = &[
        "AEM",
        "ANALYSIS",
        "CAB",
        "CONCESSION",
        "DAR",
        "DATA",
        "DISPOSITION",
        "ESA PO",
        "FPA",
        "MOD",
        "MRB",
        "NTO",
        "OHP/RAD",
        "PROPOSAL/QUOTE",
        "REPAIR",
        "RISK TRANSFER",
        "SAFETY",
        "SAR",
        "SERVICE",
        "SOFTWARE",
        "SOW",
        "SUPPLY",
        "TECH",
        "TIM",
        "TRANSFER",
    ];

    #[test]
    fn test_ngrams() {
        assert_eq!(
            NGram::<3>::new(&['m', 'o', 'd']),
            NGram::<3>::vec_from("mod")[0]
        );
    }

    #[test]
    fn index_from_distinct() {
        let index = Index::<usize, 3>::from_distinct(OPTIONS.iter().cloned());

        println!("Searching for \"dev aem\"...");
        let results = index.search_sync("dev aem", 5);
        results
            .iter()
            .for_each(|&(&i, score)| println!("{}: {}", OPTIONS[i], score));

        let best_match = OPTIONS[*results[0].0];
        assert_eq!(best_match, "AEM");

        let result = *index.best_match_sync("DAR").expect("No result");
        println!("\nBest match for \"DAR\": {}", OPTIONS[result]);
        assert_eq!(OPTIONS[result], "DAR");
    }

    #[test]
    fn index_from_iter() {
        let index = Index::<usize, 3>::from_identified(OPTIONS.iter().cloned().enumerate());

        let results = index.search_sync("dev aem", 1);
        let best_match = OPTIONS[*results[0].0];
        assert_eq!(best_match, "AEM");

        let result = *index.best_match_sync("DAR").expect("No result");
        println!("\nBest match for \"DAR\": {}", OPTIONS[result]);
        assert_eq!(OPTIONS[result], "DAR");
    }

    #[test]
    fn index_manual() {
        // TODO
    }
}
