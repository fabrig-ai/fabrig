/// Facts a party keeps (temper/docs/design/domain/forge/attention.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum PersonFact {
    /// "Skip" on the project's "To begin".
    ToBegin { project: u32 },
    /// The person started `goal` from `chat`'s document.
    GoalFrom { project: u32, chat: u64, goal: u64, at: u64 },
    /// Keep asking for this project and subject, at the client's clock.
    KeepAsking { project: u32, subject: temper_protocol_reaches::Subject, at: u64 },
}

/// A dismissal's answer (temper/docs/design/domain/forge/attention.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DismissAnswer {
    Dismissed,
    TooLarge,
}

/// The subject `dismissed`, outside the task numbers.
pub const DISMISSED_SUBJECT: u64 = 1 << 61;

use crate::{Answered, Domain, Key, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Token};

pub(crate) fn dismiss(
    d: &mut Domain,
    request: Token,
    token: Token,
    party: u64,
    what: PersonFact,
    out: &mut Queue<Request>,
) {
    let replaced = match what {
        PersonFact::KeepAsking { project, subject, .. } => {
            let first = PersonFact::KeepAsking { project, subject, at: 0 };
            match d.dismissed.range((party, first)..).next() {
                Some(((owner, held), ())) => match *held {
                    PersonFact::KeepAsking { project: held_project, subject: held_subject, .. }
                        if *owner == party && held_project == project && held_subject == subject =>
                    {
                        Some(*held)
                    }
                    PersonFact::ToBegin { .. } | PersonFact::GoalFrom { .. } | PersonFact::KeepAsking { .. } => None,
                },
                None => None,
            }
        }
        PersonFact::ToBegin { .. } | PersonFact::GoalFrom { .. } => None,
    };
    let answer = if d.dismissed.contains_key(&(party, what)) {
        DismissAnswer::Dismissed
    } else if let Some(previous) = replaced {
        d.dismissed.remove(&(party, previous));
        out.push(Request::Erase { key: Key::Dismissed { party, what: previous } });
        d.dismissed.insert((party, what), ()).expect("replacement has room");
        out.push(Request::Save { record: Stored::Dismissed { party, what }, release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([DISMISSED_SUBJECT]) });
        DismissAnswer::Dismissed
    } else if d.dismissed.len() == d.dismissed.capacity() || party_full(d, party) {
        DismissAnswer::TooLarge
    } else {
        d.dismissed.insert((party, what), ()).expect("dismissal room checked");
        out.push(Request::Save { record: Stored::Dismissed { party, what }, release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([DISMISSED_SUBJECT]) });
        DismissAnswer::Dismissed
    };
    d.connect.answer(request, token, Answered::Dismiss(answer), out);
}

pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut erases = List::with_capacity(d.dismissed.len());
    for ((party, what), ()) in &d.dismissed {
        let held = match what {
            PersonFact::ToBegin { project }
            | PersonFact::GoalFrom { project, .. }
            | PersonFact::KeepAsking { project, .. } => project,
        };
        if *held == project {
            erases.push((*party, *what)).expect("bounded dismissal rows");
        }
    }
    if !erases.is_empty() {
        for (party, what) in erases.into_boxed() {
            d.dismissed.remove(&(party, what));
            out.push(Request::Erase { key: Key::Dismissed { party, what } });
        }
        out.push(Request::Marked { tasks: Box::new([DISMISSED_SUBJECT]) });
    }
}

fn party_full(d: &Domain, party: u64) -> bool {
    let mut count = 0_u32;
    for ((owner, _), ()) in d.dismissed.range((party, PersonFact::ToBegin { project: 0 })..) {
        if *owner != party {
            break;
        }
        count = count.saturating_add(1);
        if count == temper_client_protocol_wire::DISMISSED {
            return true;
        }
    }
    false
}
