/// Byte offsets of a source to lines and UTF-16 columns and back, as LSP positions count them by default.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of each line's start.
    starts: Vec<usize>,
    text: String,
}

impl LineIndex {
    /// The index of `text`.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        Self {
            starts,
            text: text.to_string(),
        }
    }

    /// Line (0-based) and UTF-16 column of byte `offset`; past the end is the end.
    #[must_use]
    pub fn position(&self, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.text.len());
        let line = self.starts.partition_point(|&s| s <= offset) - 1;
        let start = self.starts[line];
        let col: usize = self.text[start..offset].chars().map(char::len_utf16).sum();
        (to_u32(line), to_u32(col))
    }

    /// Byte offset of `line` and UTF-16 column `col`; a column past the line's end is its end, a line past the last
    /// the text's end.
    #[must_use]
    pub fn offset(&self, line: u32, col: u32) -> usize {
        let Some(&start) = self.starts.get(line as usize) else {
            return self.text.len();
        };
        let end = self
            .starts
            .get(line as usize + 1)
            .map_or(self.text.len(), |&e| e - 1);
        let mut units = 0;
        for (i, c) in self.text[start..end].char_indices() {
            if units >= col as usize {
                return start + i;
            }
            units += c.len_utf16();
        }
        end
    }
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    #[test]
    fn utf16_columns_both_ways() {
        let text = "ab\n°C x\n\nµs";
        let index = LineIndex::new(text);
        let x = text.find('x').unwrap();
        assert_eq!(index.position(x), (1, 3));
        assert_eq!(index.offset(1, 3), x);
        assert_eq!(index.position(text.len()), (3, 2));
        assert_eq!(index.offset(2, 5), text.find("\n\n").unwrap() + 1);
        assert_eq!(index.offset(9, 0), text.len());
    }
}
