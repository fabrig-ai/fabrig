//! Skein's patch set for kept token records (codec.md, 12), empty before the baseline.

use skein_codec::patch::{Build, Halt, Old, Site};
use skein_codec::value::Node;

/// The owner's per-record inputs to written patches (codec.md, 11.5).
#[derive(Debug)]
pub struct Context {}

#[expect(clippy::derivable_impls, reason = "programming-model.md, 10.2 lists the permitted derives")]
impl Default for Context {
    fn default() -> Self {
        Self {}
    }
}

/// Skein's patch texts in number order, empty before the baseline.
pub const PATCHES: &[&str] = &[];

/// Kept families whose texts a host loads before planning patches.
pub mod kept {
    /// Kept token record versions, absent before the baseline.
    pub mod skein_oauth {
        /// The family's kept schema texts in version order.
        pub const TEXTS: &[&str] = &[];
    }
}

/// Declared refusals of written patch functions, empty with the set.
pub mod refusal {}

/// Answers a host's written stop with no matching site in this empty set.
pub fn written(site: Site, _old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {
    Err(Halt::NoSite(site))
}
