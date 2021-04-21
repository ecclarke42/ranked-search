/// Score a document using Okapi BM25
pub fn score(len: usize, avg_len: f64, freq: u32, idf: f64) -> f64 {
    let k1 = 1.2;
    let b = 0.75;

    let norm = (len as f64) / avg_len;
    let freq = freq as f64;

    let score = idf * ((freq * (k1 + 1.0)) / (freq + (k1 * (1.0 - b + (b * norm)))));

    if score < 0.0 {
        0.0
    } else {
        score
    }
}
