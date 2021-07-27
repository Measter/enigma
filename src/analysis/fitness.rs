use std::convert::TryInto;

const EPSILON: f32 = 3e-10;

// why you still unstable!?!
struct ArrWindows<'a, T, const N: usize>(&'a [T]);
impl<'a, T, const N: usize> Iterator for ArrWindows<'a, T, N> {
    type Item = &'a [T; N];
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.get(..N)?;
        self.0 = &self.0.get(1..)?;
        next.try_into().ok()
    }
}

pub trait FitnessFunction {
    fn score(&self, text: &str) -> f32;
}

// This one implements the SingleCharacterFitness, BigramFitness, TrigramFitness, and QuadgramFitness
// types.
pub struct NgramFitness<const N: usize> {
    ngrams: Vec<f32>,
}

impl<const N: usize> NgramFitness<N> {
    pub fn new<'a>(ngrams: impl IntoIterator<Item = &'a str>) -> Self {
        let num_ngrams = Self::index(&[b'Z'; N]) + 1;
        let mut store = vec![EPSILON.log10(); num_ngrams];

        for line in ngrams {
            let (key, value) = line.split_once(',').expect("Invalid ngram entry");
            let valid_key = key.chars().all(|c| c.is_ascii_uppercase()) && key.chars().count() == N;
            if !valid_key {
                panic!("Invalid ngram key: {:?}", key);
            }

            let i = Self::index(key.as_bytes()[..N].try_into().unwrap());
            let value = value.parse().expect("Invalid ngram value");

            store[i] = value;
        }

        Self { ngrams: store }
    }

    fn index(v: &[u8; N]) -> usize {
        (0..)
            .map(|i| i * 5)
            .zip(v.iter().rev())
            .map(|(s, v)| ((*v - b'A') as usize) << s)
            .sum()
    }
}

impl<const N: usize> FitnessFunction for NgramFitness<N> {
    fn score(&self, text: &str) -> f32 {
        let valid_text = text.chars().all(|c| c.is_ascii_uppercase());
        if !valid_text {
            panic!("Invalid text input: {:?}", text);
        }

        ArrWindows(text.as_bytes())
            .map(Self::index)
            .map(|i| self.ngrams[i])
            .sum()
    }
}

pub struct IoCFitness {}

impl IoCFitness {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for IoCFitness {
    fn default() -> Self {
        Self::new()
    }
}

impl FitnessFunction for IoCFitness {
    fn score(&self, text: &str) -> f32 {
        let valid_text = text.chars().all(|c| c.is_ascii_uppercase());
        if !valid_text {
            panic!("Invalid text input: {:?}", text);
        }

        let mut histogram = [0_u32; 26];
        text.chars()
            .map(|c| (c as u8 - b'A') as usize)
            .for_each(|i| histogram[i] += 1);

        let total: u32 = histogram.iter().map(|&v| v * (v - 1)).sum();

        let n = text.chars().count() as f32;
        total as f32 / (n * (n - 1.))
    }
}

// This one hasn't been tested, but I'm fairly sure it's correct.
pub struct KnownPlainTextFitness {
    plaintext: String,
}

impl KnownPlainTextFitness {
    pub fn exact_message(str: &str) -> Self {
        assert!(str.chars().all(|c| c.is_ascii_uppercase()));
        assert!(!str.is_empty());

        Self {
            plaintext: str.into(),
        }
    }

    pub fn from_words(words: &[(&str, usize)]) -> Self {
        assert!(words
            .iter()
            .flat_map(|(w, _)| w.chars())
            .all(|c| c.is_ascii_uppercase()));

        let len = words
            .iter()
            .map(|&(w, pos)| pos + w.len())
            .max()
            .expect("You need words for a plaintext attack");

        // Because we know all the characters are ASCII, and that ASCII are single-byte
        // encoded, we'll just build the string as a vector of bytes and glob in the
        // words.

        let mut plaintext = vec![0; len];

        for &(word, pos) in words {
            plaintext[pos..pos + word.len()].copy_from_slice(word.as_bytes());
        }

        Self {
            plaintext: String::from_utf8(plaintext).unwrap(),
        }
    }
}

impl FitnessFunction for KnownPlainTextFitness {
    fn score(&self, text: &str) -> f32 {
        self.plaintext
            .as_bytes()
            .iter()
            .zip(text.as_bytes())
            .map(|(a, b)| (a == b) as u32)
            .sum::<u32>() as _
    }
}
