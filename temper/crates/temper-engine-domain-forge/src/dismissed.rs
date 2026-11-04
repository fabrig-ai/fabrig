/// What a party has dismissed (temper/docs/design/domain/forge.md, 5.5).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Dismissal {
    /// "Skip" on the project's "To begin".
    ToBegin { project: u32 },
}

/// A dismissal's answer (temper/docs/design/domain/forge.md, 5.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    what: Dismissal,
    out: &mut Queue<Request>,
) {
    let answer = if d.dismissed.contains_key(&(party, what)) {
        DismissAnswer::Dismissed
    } else if d.dismissed.len() == d.dismissed.capacity() {
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
        let Dismissal::ToBegin { project: held } = what;
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
