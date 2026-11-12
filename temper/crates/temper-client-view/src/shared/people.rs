//! Team's person line (temper/docs/design/client/pages/team.md, 1).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use temper_client_domain::pages::team;

#[must_use]
pub fn person_line(w: &Words, person: &team::Person) -> Option<Box<[u8]>> {
    let waits = person.waits?;
    let whom = if person.you { w.say(Word::TeamWhomYou, &[]) } else { person.name.clone() };
    Some(wait_line(w, &waits, whom))
}

fn wait_line(w: &Words, waits: &team::Waits, whom: Box<[u8]>) -> Box<[u8]> {
    let total = waits.total();
    if total == 0 {
        return w.say(Word::TeamNothing, &[Hole::Name(whom)]);
    }
    let mut kind = None;
    let mut mixed = false;
    for (count, one, many) in [
        (waits.questions, Word::TeamQuestion, Word::TeamQuestions),
        (waits.reviews, Word::TeamReview, Word::TeamReviews),
        (waits.proposals, Word::TeamProposal, Word::TeamProposals),
        (waits.held, Word::TeamHeldTask, Word::TeamHeldTasks),
        (waits.person_tasks, Word::TeamPersonTask, Word::TeamPersonTasks),
    ] {
        if count > 0 {
            mixed |= kind.is_some();
            kind = Some(if total == 1 { one } else { many });
        }
    }
    let kind = if mixed { Word::TeamThings } else { kind.expect("a positive total has a kind") };
    let number = if waits.cut {
        w.say(Word::TeamCountCut, &[Hole::Number(u64::from(total))])
    } else if total == 1 {
        w.say(Word::TeamOne, &[])
    } else {
        match crate::pages::home::number_word(total) {
            Some(word) => w.say(word, &[]),
            None => w.say(Word::TeamCount, &[Hole::Number(u64::from(total))]),
        }
    };
    w.say(
        if total == 1 { Word::TeamWaits } else { Word::TeamWait },
        &[Hole::Name(number), Hole::Name(w.say(kind, &[])), Hole::Name(whom)],
    )
}

