//! Bounded words from the task's wait records (jig/docs/design/domain/tasks/lifecycle-holds.md, 1).

use super::Agenda;
use crate::{
    Bound, Figures, Wait, WaitKind, WaitObject,
    documents::{TaskRunWait, TaskSourceState, TaskWaitRefused, TaskWaitWhy},
};
use alloc::boxed::Box;
use jig_domain_fleet::HostKind;
use jig_domain_people::EntryKind;
use jig_domain_tasks::{AccountWhy, EscalationHolder, Hold, WAIT_WORDS};
use skein_lib::{Decimal, Duration, List, Wall, bytes, usize_of};

const CUT: &[u8] = b" [cut]";

pub(super) fn source(figures: Figures, agenda: &Agenda) -> Box<[u8]> {
    let mut out = Words { bytes: List::with_capacity(WAIT_WORDS), cut: false };
    let deadline = if agenda.first.why == Hold::Deadline { agenda.deadline } else { None };
    if let Some(deadline) = deadline {
        out.put(b"Deadline ");
        out.at(deadline);
        out.put(b" passed while waiting for ");
    }
    let mut separator = b"".as_slice();
    for wait in agenda.waits.values() {
        out.put(separator);
        let figure = match super::clock(wait, figures) {
            Some((figure, _)) => Some(figure),
            None => None,
        };
        if deadline.is_none()
            && let Some(figure) = figure
        {
            out.put(b"Waited ");
            out.span(figure);
            out.put(b" for ");
        }
        out.put(kind(wait.kind));
        out.put(b" since ");
        out.at(wait.since);
        object(wait, &mut out);
        if deadline.is_none()
            && figure.is_none()
            && let Bound::At { at, .. } = wait.bound
        {
            out.put(b", bound ");
            out.at(at);
        }
        if out.cut {
            break;
        }
        separator = b"; ";
    }
    out.put(b".");
    out.finish()
}

const fn kind(kind: WaitKind) -> &'static [u8] {
    match kind {
        WaitKind::Party => b"party",
        WaitKind::Question => b"question",
        WaitKind::Dependency => b"dependency",
        WaitKind::Resource => b"resource",
        WaitKind::Account => b"account",
        WaitKind::Host => b"host",
        WaitKind::Delegate => b"delegate",
        WaitKind::News => b"news",
        WaitKind::Step => b"step",
        WaitKind::Room => b"room",
        WaitKind::Turn => b"turn",
        WaitKind::SignOff => b"sign-off",
    }
}

fn object(wait: &Wait, out: &mut Words) {
    match &wait.object {
        WaitObject::News { subscriptions, timer } => {
            if !subscriptions.is_empty() {
                out.put(b": subscriptions ");
                for (index, number) in subscriptions.iter().enumerate() {
                    if index != 0 {
                        out.put(if index.saturating_add(1) == usize_of(subscriptions.len()) {
                            b" and "
                        } else {
                            b", "
                        });
                    }
                    out.number(*number);
                }
            }
            if let Some(at) = timer {
                out.put(if subscriptions.is_empty() { b": a timer at " } else { b", a timer at " });
                out.at(*at);
            }
        }
        WaitObject::Run(wait) => {
            out.put(b": ");
            run(*wait, out);
        }
        WaitObject::Task(task) => out.named(b": task ", *task),
        WaitObject::Question { number, answerer } => {
            out.named(b": question ", *number);
            out.named(b" for task ", *answerer);
        }
        WaitObject::Entry { kind, at } => {
            match kind {
                EntryKind::Proposal { number, .. } => out.named(b": proposal ", *number),
                EntryKind::Escalation { revision } => out.named(b": escalation ", *revision),
                EntryKind::Question { message } => out.named(b": question ", *message),
                EntryKind::PersonTask => out.named(b": person task ", wait.task),
                EntryKind::Reply { message } | EntryKind::ChatReply { message } => out.named(b": reply ", *message),
            }
            match at {
                EscalationHolder::Task(number) => out.named(b" at task ", *number),
                EscalationHolder::Person(number) => out.named(b" at person ", *number),
                EscalationHolder::Role { role, project } => {
                    out.named(b" at role ", u64::from(*role));
                    out.named(b" in project ", u64::from(*project));
                }
            }
        }
        WaitObject::Resource { name, holder } => {
            out.named(b": connector ", u64::from(name.connector));
            for part in &name.path {
                out.put(b"/");
                if bytes::is_utf8(part) {
                    out.put(part);
                } else {
                    out.put(b"[non-text name]");
                }
            }
            if let Some(task) = holder {
                out.named(b", held by task ", *task);
            }
        }
        WaitObject::Procedure { step, things } => {
            out.named(b": procedure step ", u64::from(*step));
            for thing in things {
                out.named(b", object kind ", u64::from(thing.what));
                out.named(b" number ", thing.number);
                if let Some(owner) = thing.owner {
                    out.named(b" of task ", owner);
                }
            }
        }
    }
}

fn run(wait: TaskRunWait, out: &mut Words) {
    match wait {
        TaskRunWait::Credential => out.put(b"credential"),
        TaskRunWait::Host => out.put(b"no host has room or is connected"),
        TaskRunWait::Worker { worker, until } => {
            out.named(b"worker ", u64::from(worker));
            out.put(b" to return, until ");
            out.at(until);
        }
        TaskRunWait::Refused(TaskWaitRefused::Bounds { kind, .. }) => {
            out.put(host(kind));
            out.put(b" cannot take these run bounds");
        }
        TaskRunWait::Refused(TaskWaitRefused::Hello { kind, at, .. }) => {
            out.put(host(kind));
            out.put(b" refused at ");
            out.at(at);
        }
        TaskRunWait::Account { account, source, why } => {
            if let Some(account) = account {
                out.named(b"account ", u64::from(account));
            } else {
                out.put(b"no account");
            }
            if let Some(source) = source {
                out.named(b" source ", u64::from(source));
            }
            out.put(b": ");
            out.put(match why {
                TaskWaitWhy::Refreshing => b"refreshing",
                TaskWaitWhy::Host => b"host",
                TaskWaitWhy::Empty => b"empty",
                TaskWaitWhy::Untooled { worker } => return out.named(b"untooled on worker ", u64::from(worker)),
                TaskWaitWhy::Invalid(why) => match why {
                    AccountWhy::Unknown => b"unknown",
                    AccountWhy::Provider => b"provider",
                    AccountWhy::Uses => b"uses",
                    AccountWhy::Limited => b"limited",
                },
                TaskWaitWhy::Source(state) => return source_state(state, out),
            });
        }
    }
}

fn source_state(state: TaskSourceState, out: &mut Words) {
    out.put(match state {
        TaskSourceState::Moving => b"moving",
        TaskSourceState::Asking => b"asking",
        TaskSourceState::Granted => b"granted",
        TaskSourceState::Later { .. } => b"later",
        TaskSourceState::Limited { .. } => b"limited",
        TaskSourceState::Refused => b"refused",
        TaskSourceState::NeedsSignIn => b"needs sign-in",
        TaskSourceState::Misconfigured => b"misconfigured",
        TaskSourceState::Unloaded { .. } => b"unloaded",
        TaskSourceState::Cancelled => b"cancelled",
        TaskSourceState::Closing => b"closing",
    });
    if let TaskSourceState::Later { until: Some(at) } | TaskSourceState::Limited { until: Some(at) } = state {
        out.put(b" until ");
        out.at(at);
    }
}

const fn host(kind: HostKind) -> &'static [u8] {
    match kind {
        HostKind::Worker => b"worker",
        HostKind::Engine => b"engine host",
    }
}

struct Words {
    bytes: List<u8>,
    cut: bool,
}

impl Words {
    fn put(&mut self, word: &[u8]) {
        if self.cut {
            return;
        }
        let room = usize_of(self.bytes.room());
        self.cut = word.len() > room;
        for byte in bytes::utf8_prefix(word, room) {
            self.bytes.push(*byte).expect("bounded hold words");
        }
    }

    fn named(&mut self, name: &[u8], number: u64) {
        self.put(name);
        self.number(number);
    }

    fn number(&mut self, number: u64) {
        self.put(Decimal::of(number).as_bytes());
    }

    fn at(&mut self, wall: Wall) {
        let utc = wall.utc_second();
        self.put(utc.get(..16).expect("UTC minute"));
        self.put(b"Z");
    }

    fn span(&mut self, figure: Duration) {
        let nanos = figure.as_nanos();
        for (unit, suffix) in [
            (3_600_000_000_000, b" h".as_slice()),
            (60_000_000_000, b" min"),
            (1_000_000_000, b" s"),
            (1_000_000, b" ms"),
            (1, b" ns"),
        ] {
            if nanos.rem_euclid(unit) == 0 {
                self.number(nanos.div_euclid(unit));
                self.put(suffix);
                return;
            }
        }
    }

    fn finish(mut self) -> Box<[u8]> {
        if self.cut {
            let len = bytes::utf8_prefix(self.bytes.as_slice(), usize_of(WAIT_WORDS).saturating_sub(CUT.len())).len();
            self.bytes.truncate(u32::try_from(len).expect("bounded hold words"));
            self.cut = false;
            self.put(CUT);
        }
        self.bytes.into_boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_words_end_at_a_character_boundary_and_mark_the_cut() {
        let mut out = Words { bytes: List::with_capacity(WAIT_WORDS), cut: false };
        for _ in 0..WAIT_WORDS.div_euclid(3) {
            out.put("€".as_bytes());
        }
        out.put("€".as_bytes());
        let words = out.finish();
        assert!(words.len() <= usize_of(WAIT_WORDS));
        assert!(bytes::is_utf8(&words), "character boundary");
        let head = words.strip_suffix(CUT).expect("marked cut");
        assert_eq!(head.len(), usize_of(WAIT_WORDS - 6).div_euclid(3) * 3);
    }
}
