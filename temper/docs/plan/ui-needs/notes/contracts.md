# ui-needs: the shapes that cross tasks

The run's binding calls. A task builds to what is here; a later batch adds to this note and changes
nothing above it.

## A question's words

`temper-client-domain`'s module `question` (created by `question-parser`), by
`temper/docs/design/domain/agent.md`, 8.2:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    /// What the button says, and the answer's words: no prefix, no " (recommended)".
    pub words: Box<[u8]>,
    pub recommended: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub question: Box<[u8]>,
    /// Empty where there is none.
    pub context: Box<[u8]>,
    pub saw: Option<Box<[u8]>>,
    /// Empty on the fallback: answered by a reply alone.
    pub choices: Box<[Choice]>,
}

impl Parsed {
    /// The choice drawn primary: the recommended one, else the first.
    #[must_use]
    pub fn primary(&self) -> Option<usize>;
}

/// `cut` is the read's `question.words.cut`.
#[must_use]
pub fn parse(words: &[u8], cut: bool) -> Parsed;
```
