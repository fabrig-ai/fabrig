//! The retired foreman's stored state (domain/forge/change/questions.md, 1).
use super::Context;
use super::kept::temper_store::p0001::{new, old};
use skein_codec::patch::{Build, Halt, Old};
use skein_codec::value::Node;

/// A retired foreman keeps no task; all other stands retain their payloads.
pub fn stands(old: &Old, new: &mut Build, at: Node, _cx: &mut Context) -> Result<(), Halt> {
    if old.tag(at)? == old::stands::FOREMAN {
        return new.put_variant(new::stands::RETIRED);
    }
    new.carry(old, at)
}
