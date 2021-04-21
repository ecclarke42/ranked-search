#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct NGram<const N: usize>([char; N]);

impl<const N: usize> NGram<N> {
    // Panics if chars is not of length N
    pub fn new(chars: &[char]) -> Self {
        let mut arr = [char::default(); N];
        arr.copy_from_slice(chars);
        NGram(arr)
    }

    pub fn vec_from<T: ToString>(input: T) -> Vec<Self> {
        input
            .to_string()
            .to_lowercase()
            .chars()
            .collect::<Vec<char>>()
            .windows(N)
            .map(NGram::new)
            .collect::<Vec<_>>()
    }
}
