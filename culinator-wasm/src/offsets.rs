//! Translating Rust byte offsets into JavaScript string indices.
//!
//! `culinator-parser` reports spans as UTF-8 byte offsets, which is the only
//! sensible unit on the Rust side. JavaScript strings are UTF-16, and
//! `String.prototype.slice` counts UTF-16 code units — so the moment a document
//! contains any non-ASCII character, every offset after it refers to a
//! different position in the two languages.
//!
//! This was not hypothetical. `fully_loaded_guacamole.cg` has two em-dashes in
//! its comments (3 bytes each, 1 UTF-16 unit each, so 2 units of drift apiece).
//! The editor's "delete this step" reported a span four units past the real
//! one, sliced `"ation rest does rest {…"`, left an orphaned `oper` behind, and
//! broke the recipe. Any span-based edit had the same flaw.
//!
//! Converting here rather than in TypeScript keeps the fix in the one place
//! that has both the source text and the offsets, and means every consumer gets
//! indices it can hand straight to `slice`.

/// Byte offset -> UTF-16 code-unit offset, precomputed for one source string.
pub struct Utf16Offsets {
    /// Indexed by byte offset; entries inside a multi-byte character hold the
    /// offset of the character's start, so a stray interior index degrades to
    /// the nearest boundary instead of panicking.
    prefix: Vec<u32>,
}

impl Utf16Offsets {
    pub fn new(source: &str) -> Self {
        let mut prefix = vec![0u32; source.len() + 1];
        let mut byte = 0usize;
        let mut utf16 = 0u32;
        for character in source.chars() {
            let bytes = character.len_utf8();
            for offset in 0..bytes {
                prefix[byte + offset] = utf16;
            }
            byte += bytes;
            utf16 += character.len_utf16() as u32;
            prefix[byte] = utf16;
        }
        Self { prefix }
    }

    /// The UTF-16 index a JavaScript caller should use for this byte offset.
    pub fn at(&self, byte: usize) -> usize {
        self.prefix
            .get(byte)
            .copied()
            .unwrap_or_else(|| self.prefix.last().copied().unwrap_or(0)) as usize
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ascii_offsets_are_unchanged() {
        let map = Utf16Offsets::new("hello");
        assert_eq!(map.at(0), 0);
        assert_eq!(map.at(5), 5);
    }

    #[test]
    fn multibyte_characters_shift_later_offsets() {
        // "a—b": the em-dash is 3 UTF-8 bytes but 1 UTF-16 unit.
        let source = "a—b";
        assert_eq!(source.len(), 5);
        let map = Utf16Offsets::new(source);
        assert_eq!(map.at(0), 0);
        assert_eq!(map.at(1), 1);
        assert_eq!(map.at(4), 2, "byte 4 is 'b', UTF-16 index 2");
        assert_eq!(map.at(5), 3);
        // And the mapped indices really do slice the same text in JS terms.
        let utf16: Vec<u16> = source.encode_utf16().collect();
        assert_eq!(
            String::from_utf16(&utf16[map.at(4)..map.at(5)]).unwrap(),
            "b"
        );
    }

    #[test]
    fn astral_characters_count_as_two_units() {
        // An emoji is 4 UTF-8 bytes and a surrogate pair in UTF-16.
        let map = Utf16Offsets::new("🥕x");
        assert_eq!(map.at(4), 2);
        assert_eq!(map.at(5), 3);
    }

    #[test]
    fn out_of_range_offsets_clamp() {
        let map = Utf16Offsets::new("abc");
        assert_eq!(map.at(99), 3);
    }
}
