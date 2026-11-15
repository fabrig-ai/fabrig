//! The seventeen filters of hearth/docs/design/protocol/format/templates.md, 5.

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Filter {
    Default,
    Join,
    Split,
    First,
    Last,
    Length,
    Replace,
    Items,
    Keys,
    Values,
    Sort,
    Tojson,
    ShellQuote,
    SystemdEscape,
    SystemdEscapePath,
    Basename,
    Dirname,
}
impl Filter {
    /// The arguments popped after the input (hearth/docs/design/protocol/format/templates.md, 5).
    #[must_use]
    pub const fn arity(self) -> u32 {
        match self {
            Self::Default | Self::Join | Self::Split => 1,
            Self::Replace => 2,
            Self::First
            | Self::Last
            | Self::Length
            | Self::Items
            | Self::Keys
            | Self::Values
            | Self::Sort
            | Self::Tojson
            | Self::ShellQuote
            | Self::SystemdEscape
            | Self::SystemdEscapePath
            | Self::Basename
            | Self::Dirname => 0,
        }
    }
}
