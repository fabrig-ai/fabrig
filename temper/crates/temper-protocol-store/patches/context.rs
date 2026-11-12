//! The owner's per-record inputs to written patch functions (data/codec/patches.md, 5): the command wrote it once; it is the owner's.

use super::kept::temper_store::p0005::old;
use alloc::boxed::Box;
use skein_codec::{
    patch::{Halt, Old},
    value::Node,
};

#[derive(Debug)]
pub struct Context {
    pub landing_branch: Option<Box<[u8]>>,
}

impl Context {
    /// Stored inputs identifying the project and, when retained, a member's repository.
    pub fn landing_project(old: &Old, at: Node) -> Result<(u32, Option<skein_forge::api::Repository>), Halt> {
        let project =
            u32::try_from(old.uint(old.field(at, old::landing_state::PROJECT)?)?).map_err(|_| Halt::Misread(at))?;
        let mut members = old.items(old.field(at, old::landing_state::MEMBERS)?)?;
        let repository = if let Some(member) = old.next_item(&mut members) {
            let repository = old.field(member, old::landing_member::REPOSITORY)?;
            Some(skein_forge::api::Repository {
                forge: u16::try_from(old.uint(old.field(repository, old::repository::FORGE)?)?)
                    .map_err(|_| Halt::Misread(repository))?,
                repository: u32::try_from(old.uint(old.field(repository, old::repository::REPOSITORY)?)?)
                    .map_err(|_| Halt::Misread(repository))?,
            })
        } else {
            None
        };
        Ok((project, repository))
    }
}

// Default supplies no row evidence (skein/docs/design/data/codec/patches.md, 5).
// The driver replaces these inputs before each record. The derive is refused
// crate-wide, so this implementation is written out.
#[expect(clippy::derivable_impls, reason = "the derive is refused crate-wide; see above")]
impl Default for Context {
    fn default() -> Self {
        Self { landing_branch: None }
    }
}
