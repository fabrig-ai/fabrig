/// What a party has dismissed (temper/docs/design/domain/forge.md, 5.5).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Dismissal {
    /// "Skip" on the project's "To begin".
    ToBegin { project: u32 },
}
