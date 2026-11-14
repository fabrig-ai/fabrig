//! Resume a linear probe as a group (temper/docs/design/domain/after-landing.md, 5.2 and 10).
use super::{Context, kept::temper_store::p0014::old};
use skein_codec::patch::{Build, Halt, Old};
use skein_codec::value::Node;

/// All landings before the selected linear probe have already passed these tests.
pub fn tier_probe_low(old: &Old, new: &mut Build, at: Node, _cx: &mut Context) -> Result<(), Halt> {
    new.put_uint(old.uint(old.field(at, old::tier_probe::LANDING)?)?)
}

/// The containing run supplies the last landing; a standalone probe has only its own landing.
pub fn tier_probe_high(old: &Old, new: &mut Build, at: Node, cx: &mut Context) -> Result<(), Halt> {
    if let Some(part) = old.parent(at)?
        && let Some(run) = old.parent(part)?
    {
        let count = old.count(old.field(run, old::tier_run::LANDINGS)?)?;
        return new.put_uint(u64::from(count.saturating_sub(1)));
    }
    tier_probe_low(old, new, at, cx)
}
