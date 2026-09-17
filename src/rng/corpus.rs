//! A finite corpus of saved output read as 32-bit words.
//!
//! The corpus is a byte string whose length is a multiple of four, each four
//! bytes one little-endian word.  Unlike a generator it can run out.  When a
//! read passes the end, [`Corpus`] records the word index at which that
//! happened and returns 0 for that read and every later one; it never repeats
//! its input or substitutes generated words.  A caller that sees
//! [`Corpus::exhausted_at`] set must discard every statistic computed from the
//! reads after that point.

use super::Rng;

/// Saved 32-bit words, read in order.
#[derive(Debug, Clone)]
pub struct Corpus {
    words: Vec<u32>,
    position: usize,
    exhausted_at: Option<usize>,
}

/// A byte string that is not a whole number of words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialWord {
    /// Length of the byte string.
    pub bytes: usize,
}

impl std::fmt::Display for PartialWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} bytes is not a whole number of 32-bit words",
            self.bytes
        )
    }
}

impl std::error::Error for PartialWord {}

impl Corpus {
    /// Little-endian words from `bytes`.
    ///
    /// # Errors
    /// [`PartialWord`] when the length is not a multiple of four.
    pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, PartialWord> {
        if !bytes.len().is_multiple_of(4) {
            return Err(PartialWord { bytes: bytes.len() });
        }
        let words = bytes
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        Ok(Self {
            words,
            position: 0,
            exhausted_at: None,
        })
    }

    /// Number of words in the corpus.
    #[must_use]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// `true` for a corpus with no words.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Index of the next word to be read; past the end once exhausted.
    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    /// The index of the first read past the end, if any read has passed it.
    #[must_use]
    pub fn exhausted_at(&self) -> Option<usize> {
        self.exhausted_at
    }
}

impl Rng for Corpus {
    fn next_u32(&mut self) -> u32 {
        let word = self.words.get(self.position).copied();
        if word.is_none() && self.exhausted_at.is_none() {
            self.exhausted_at = Some(self.position);
        }
        self.position += 1;
        word.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{Corpus, PartialWord};
    use crate::rng::Rng;

    #[test]
    fn words_are_little_endian_and_exhaustion_is_recorded() {
        let mut c = Corpus::from_le_bytes(&[1, 0, 0, 0, 0x78, 0x56, 0x34, 0x12]).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c.next_u32(), 1);
        assert_eq!(c.exhausted_at(), None);
        assert_eq!(c.next_u32(), 0x1234_5678);
        assert_eq!(c.exhausted_at(), None);
        assert_eq!(c.next_u32(), 0);
        assert_eq!(c.exhausted_at(), Some(2));
        assert_eq!(c.next_u32(), 0);
        assert_eq!(c.exhausted_at(), Some(2));
        assert_eq!(c.position(), 4);
    }

    /// Reading in pieces gives the same words as reading at once, and a
    /// request for no words is not exhaustion.
    #[test]
    fn split_reads_match_and_empty_requests_do_not_exhaust() {
        let bytes: Vec<u8> = (0u8..40).collect();
        let whole: Vec<u32> = Corpus::from_le_bytes(&bytes).unwrap().collect_u32s(10);
        let mut c = Corpus::from_le_bytes(&bytes).unwrap();
        let mut split = c.collect_u32s(3);
        split.extend(c.collect_u32s(0));
        split.extend(c.collect_u32s(7));
        assert_eq!(whole, split);
        assert_eq!(c.exhausted_at(), None);
        assert!(Corpus::from_le_bytes(&[]).unwrap().is_empty());
    }

    #[test]
    fn partial_words_are_rejected() {
        assert_eq!(
            Corpus::from_le_bytes(&[0; 5]).unwrap_err(),
            PartialWord { bytes: 5 }
        );
    }
}
