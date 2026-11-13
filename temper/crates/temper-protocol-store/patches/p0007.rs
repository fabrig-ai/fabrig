//! Per-member prefixes and effects from a v17 round (domain/landing.md, 12).
use super::{Context, kept::temper_store::p0007::old};
use skein_codec::patch::{Build, Halt, Old};
use skein_codec::value::Node;

/// Carry the round's single member into the batch's per-member record.
pub fn landing_state_members(old: &Old, new: &mut Build, at: Node, _cx: &mut Context) -> Result<(), Halt> {
    let members = old.field(at, old::landing_state::MEMBERS)?;
    let round = old.some(old.field(at, old::landing_state::ROUND)?)?;
    let mut items = old.items(members)?;
    new.open()?;
    for index in 0..old.count(members)? {
        let member = old.next_item(&mut items).expect("counted members");
        new.open()?;
        for field in [
            old::landing_member::CHANGE,
            old::landing_member::REPOSITORY,
            old::landing_member::PULL,
            old::landing_member::HEAD,
            old::landing_member::BASE,
            old::landing_member::CANDIDATE,
            old::landing_member::GATES,
            old::landing_member::ASKED,
            old::landing_member::LANDED,
            old::landing_member::LEFT,
        ] {
            new.carry(old, old.field(member, field)?)?;
        }
        new.put_uint(0)?;
        match round {
            Some(round) => new.carry(old, old.field(round, old::landing_round::BRANCH)?)?,
            None => new.put_content(b"")?,
        }
        match round {
            Some(round) if index == 0 => {
                new.carry(old, old.field(round, old::landing_round::COMBINED)?)?;
                new.carry(old, old.field(round, old::landing_round::RECORDED)?)?;
                new.carry(old, old.field(round, old::landing_round::TOLD)?)?;
                new.carry(old, old.field(round, old::landing_round::JUDGING)?)?;
            }
            Some(_) | None => {
                new.put_none()?;
                new.put_bool(false)?;
                new.put_bool(false)?;
                new.put_bool(false)?;
            }
        }
        new.put_bool(old.some(old.field(member, old::landing_member::LEFT)?)?.is_some())?;
        new.close()?;
    }
    new.close()
}
