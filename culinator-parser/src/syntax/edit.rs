use super::{SyntaxError, TextRange};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}

impl TextEdit {
    pub fn replace(range: TextRange, replacement: impl Into<String>) -> Self {
        Self {
            range,
            replacement: replacement.into(),
        }
    }
    pub fn insert(offset: usize, text: impl Into<String>) -> Self {
        Self::replace(TextRange::new(offset, offset), text)
    }
    pub fn delete(range: TextRange) -> Self {
        Self::replace(range, "")
    }
}

pub fn apply_text_edits(source: &str, edits: &[TextEdit]) -> Result<String, SyntaxError> {
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| (edit.range.start, edit.range.end));
    let mut cursor = 0;
    let mut output = String::with_capacity(source.len());
    for edit in ordered {
        if edit.range.end > source.len() || edit.range.start > edit.range.end {
            return Err(SyntaxError::InvalidEditRange {
                start: edit.range.start,
                end: edit.range.end,
                source_len: source.len(),
            });
        }
        if edit.range.start < cursor {
            return Err(SyntaxError::OverlappingEdits(edit.range.start));
        }
        output.push_str(&source[cursor..edit.range.start]);
        output.push_str(&edit.replacement);
        cursor = edit.range.end;
    }
    output.push_str(&source[cursor..]);
    Ok(output)
}

#[cfg(test)]
mod test;
