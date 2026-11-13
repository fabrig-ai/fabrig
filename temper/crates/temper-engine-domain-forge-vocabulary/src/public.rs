//! Visitor projections from retained forge facts (temper/docs/design/client/forge/README.md, 9).
use alloc::boxed::Box;
use skein_lib::Wall;

/// A task handed by jig, with the public goal of its tree (client/forge/README.md, 9.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PublicTask {
    pub task: u64,
    pub goal: Option<u64>,
}
/// The core's public ask, with tasks ascending (client/forge/README.md, 9.1).
#[derive(Debug)]
pub struct PublicAsk<'a> {
    pub kind: u16,
    pub version: u16,
    pub project: u32,
    pub task: Option<u64>,
    pub public: &'a [PublicTask],
    pub now: Wall,
}
/// Tasks in first-named order and a record referring to their places.
#[derive(Debug)]
pub struct PublicMade {
    pub tasks: Box<[u64]>,
    pub record: Box<[u8]>,
}
impl PublicAsk<'_> {
    #[expect(clippy::manual_find, reason = "the strict subset uses an explicit search without closures")]
    pub fn handed(&self, task: u64) -> Option<&PublicTask> {
        for row in self.public {
            if row.task == task {
                return Some(row);
            }
        }
        None
    }
}
