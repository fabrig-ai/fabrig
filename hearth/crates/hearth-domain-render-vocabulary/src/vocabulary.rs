//! Render's boundary records (hearth/docs/design/domain/render/README.md, 5.2).

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Class {
    Configuration,
    Operational,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Outcome {
    Usage,
    Failed,
    TryAgain,
    Negative,
    Done,
}
impl Outcome {
    #[must_use]
    pub fn worst(self, other: Outcome) -> Outcome {
        core::cmp::min(self, other)
    }
}
