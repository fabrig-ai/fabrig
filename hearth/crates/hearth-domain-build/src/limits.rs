//! The plan's bound (skein/docs/foundation/programming-model.md, 6.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// The most layers a plan builds: the recipe's three.
    pub layers: u32,
    /// The most steps a layer runs.
    pub steps: u32,
}
pub const SHIPPED: Limits = Limits { layers: 3, steps: crate::plan::MAX_STEPS };
