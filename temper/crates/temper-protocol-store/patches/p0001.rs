//! Patch 0001: the command wrote the skeleton; its functions are the author's (data/codec/patches.md, 5).

use super::Context;
use skein_codec::patch::{Build, Halt, Old, Site};
use skein_codec::value::Node;

/// Site 0, `written Route`, temper-store 11 to 12.
///
/// - `add Route.retired`
/// - `drop Route.foreman`
pub fn route(_old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {
    Err(Halt::NoSite(Site::new(1, 0)))
}

/// Site 1, `written Stands`, temper-store 11 to 12.
///
/// - `add Stands.retired`
/// - `drop Stands.foreman`
pub fn stands(_old: &Old, _new: &mut Build, _at: Node, _cx: &mut Context) -> Result<(), Halt> {
    Err(Halt::NoSite(Site::new(1, 1)))
}
