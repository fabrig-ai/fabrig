//! Fixed scenario journal limits (temper/docs/design/domain/testing.md, 5.1).

// One decision's writes for the walking, basic forge, two-slot and stock settings worlds.
pub const WALKING_WRITES: u32 = 2_118;

// Writes for the two-project settings and period worlds and the ending world without a child.
pub const EXTENDED_WALKING_WRITES: u32 = 2_120;

// Writes for the eight-task forge tree and the QA dependency and kept-change worlds.
pub const TREE_WRITES: u32 = 8_192;
