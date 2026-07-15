# Lossless parsing

Culinograph parsing is split into two layers.

## Concrete syntax layer

`culinograph_parser::LosslessDocument` tokenizes every source byte. Trivia is not discarded:

- spaces, tabs, and newlines
- line comments
- block comments
- string escaping
- punctuation and delimiters
- Unicode or currently unsupported syntax as `Unknown` tokens

Every token has a byte-based `TextRange`. Balanced braces, brackets, parentheses, and generic argument delimiters are represented as nested concrete syntax nodes. Calling `round_trip()` returns the original source byte-for-byte.

The syntax layer can parse future or extension syntax even when the semantic layer does not understand it, as long as strings, comments, and delimiters are structurally valid.

## Semantic projection

The existing domain AST remains a semantic projection of supported declarations. Use:

```rust
let parsed = culinograph_parser::parse_lossless(source)?;
let exact_source = parsed.syntax.source();
let recipe_model = parsed.semantic;
```

Semantic parsing never acts as a formatter or serializer for existing source.

## Source edits

Editors and the LSP should apply `TextEdit` values to the concrete source:

```rust
use culinograph_parser::{TextEdit, TextRange};

let edit = TextEdit::replace(TextRange::new(start, end), "new value");
let updated = parsed.edit(&[edit])?;
```

Edits must be non-overlapping and use UTF-8 byte offsets. Untouched source ranges remain identical, preserving comments and formatting.

## Error recovery

The concrete lexer preserves unknown tokens, but it reports malformed strings, unterminated block comments, and unbalanced delimiters. Semantic errors are reported separately when projecting supported syntax. Future work can add error nodes and partial semantic projection for incomplete editor buffers.
