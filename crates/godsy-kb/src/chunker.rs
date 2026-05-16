use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    pub ordinal: u32,
    pub text: String,
}

/// Split `text` into character-windowed chunks of roughly `size` chars with
/// `overlap` chars carried into the next chunk. Operates on chars, not bytes,
/// so it is safe for non-ASCII input. Empty input produces no chunks.
#[must_use]
pub fn chunk_text(text: &str, size: usize, overlap: usize) -> Vec<Chunk> {
    let trimmed = text.trim();
    if trimmed.is_empty() || size == 0 {
        return Vec::new();
    }
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() <= size {
        return vec![Chunk { ordinal: 0, text: trimmed.to_string() }];
    }
    let stride = size.saturating_sub(overlap).max(1);
    let mut out = Vec::new();
    let mut start = 0_usize;
    let mut ordinal = 0_u32;
    while start < chars.len() {
        let end = (start + size).min(chars.len());
        let slice: String = chars[start..end].iter().collect();
        out.push(Chunk { ordinal, text: slice });
        ordinal += 1;
        if end == chars.len() {
            break;
        }
        start += stride;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_produces_no_chunks() {
        assert!(chunk_text("", 100, 10).is_empty());
        assert!(chunk_text("   \n\t  ", 100, 10).is_empty());
    }

    #[test]
    fn short_input_yields_single_chunk() {
        let c = chunk_text("hello world", 100, 10);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].text, "hello world");
    }

    #[test]
    fn long_input_yields_overlapping_chunks() {
        let s = "a".repeat(250);
        let c = chunk_text(&s, 100, 20);
        assert_eq!(c.len(), 3);
        assert_eq!(c[0].text.chars().count(), 100);
        assert_eq!(c[1].text.chars().count(), 100);
        assert_eq!(c[2].text.chars().count(), 250 - 80 * 2);
    }

    #[test]
    fn handles_non_ascii_safely() {
        let s: String = "ñ".repeat(50);
        let c = chunk_text(&s, 20, 5);
        assert!(!c.is_empty());
        for ch in &c {
            assert!(ch.text.chars().all(|x| x == 'ñ'));
        }
    }
}
