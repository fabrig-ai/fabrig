//! The shell's focus and caret (skein/docs/design/ui-screen.md, 5.4).

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Focus {
    pub node: Option<skein_ui::NodeId>,
    /// A byte offset at a character's boundary in the focused field's value.
    pub caret: u32,
    /// The body's first row shown.
    pub scroll: u32,
    /// The last focus request honoured.
    pub honoured: u64,
}
