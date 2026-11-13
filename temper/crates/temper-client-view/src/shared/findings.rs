//! Finding refusals shared with Tell (temper/docs/design/client/pages/needs-you.md, 1).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use skein_lib::{Decimal, bytes};
use skein_ui::phrases::Hole;
use temper_client_protocol_wire::v1 as w;

#[must_use]
pub fn finding_refusal(words: &Words, refusal: &w::FindingRefusal) -> Box<[u8]> {
    match refusal {
        w::FindingRefusal::NoArea(areas) => {
            let mut names = Box::<[u8]>::default();
            for area in areas.areas() {
                names = if names.is_empty() { area.clone() } else { bytes::concat(&[&names, b", ", area]) };
            }
            words.say(Word::FindingNoSuchArea, &[Hole::Name(names)])
        }
        w::FindingRefusal::Full(full) => words.say(Word::FindingFull, &[Hole::Number(u64::from(full.count()))]),
        w::FindingRefusal::NotYours(held) => {
            let (prefix, number) = match held.holder() {
                w::Holder::Task(task) => (Word::FindingTask, task.number()),
                w::Holder::Party(party) => (Word::FindingParty, party.number()),
                w::Holder::Role(role) => (Word::FindingRole, u64::from(role.number())),
            };
            let holder = bytes::concat(&[&words.say(prefix, &[]), Decimal::of(number).as_bytes()]);
            words.say(Word::FindingOtherOwner, &[Hole::Name(holder)])
        }
        w::FindingRefusal::NoFinding => words.say(Word::FindingGone, &[]),
        w::FindingRefusal::NoGoal => words.say(Word::FindingNoGoal, &[]),
        w::FindingRefusal::Ended => words.say(Word::FindingEnded, &[]),
        w::FindingRefusal::Words(field) => field.field().into(),
    }
}
