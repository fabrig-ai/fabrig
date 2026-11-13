//! Rules and wake rows on the role card (client/pages/team.md, 1).
use super::{paragraph, text};
use crate::{
    bindings::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{Decimal, bytes::concat};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address,
    pages::{
        Act,
        team::{
            self,
            wakes::{Interest, Row},
        },
    },
};

pub(super) fn rules(b: &mut Builder, w: &Words, project: u32, rules: &team::Rules, bindings: &mut Bindings) {
    b.open(Role::Region);
    b.style(look::TEAM_RULES);
    b.open(Role::Paragraph);
    b.style(look::TEAM_KICKER);
    let prefix = w.say(Word::TeamRules, &[]);
    let proposal = w.say(Word::TeamChangeProposal, &[]);
    b.name(&concat(&[&prefix, &proposal]));
    b.text(Run::Plain, &prefix);
    crate::bindings::link(
        b,
        &proposal,
        bindings.insert(Act::Go { address: Address::Role { project, charter: rules.charter } }),
    );
    b.text(Run::Plain, &proposal);
    b.close();
    b.close();
    text(b, Role::Paragraph, &rules.asked);
    b.close();
}

pub(super) fn draw(
    b: &mut Builder,
    w: &Words,
    facts: &team::Card,
    cut: bool,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Region);
    b.style(look::TEAM_WAKES);
    paragraph(b, &w.say(Word::TeamWakes, &[]), look::TEAM_KICKER);
    for row in &facts.wakes {
        let (left, class) = words(w, row, times);
        b.open(Role::Slot);
        b.style(look::TEAM_WAKE_ROW);
        b.open(Role::Text);
        b.name(&left);
        match row.interest {
            Interest::Task { target, held, result } => {
                b.text(Run::Plain, &w.say(Word::WakeTask, &[]));
                let number = Decimal::of(target);
                crate::bindings::link(
                    b,
                    number.as_bytes(),
                    bindings.insert(Act::Go { address: Address::Task { task: target } }),
                );
                b.text(Run::Plain, number.as_bytes());
                b.close();
                b.text(Run::Plain, &w.say(task_suffix(held, result), &[]));
            }
            Interest::Timer { .. } | Interest::Topics { .. } => b.text(Run::Plain, &left),
        }
        b.close();
        if let Some(class) = class {
            b.open(Role::Text);
            b.style(look::TEAM_WAKE_CLASS);
            b.name(&class);
            b.text(Run::Code, &class);
            b.close();
        }
        b.close();
    }
    if cut {
        paragraph(b, &w.say(Word::TeamWakesCut, &[]), look::SETTINGS_EMPTY);
    }
    b.close();
}
fn task_suffix(held: bool, result: bool) -> Word {
    if held && result {
        Word::WakeTaskBoth
    } else if held {
        Word::WakeTaskHeld
    } else if result {
        Word::WakeTaskResult
    } else {
        Word::WakeTaskNeither
    }
}
/// Words stay in the view; the domain rows carry no composed text.
pub(crate) fn words(w: &Words, row: &Row, times: &mut Times) -> (Box<[u8]>, Option<Box<[u8]>>) {
    let left = match row.interest {
        Interest::Timer { at, period, wakes } => {
            if !wakes {
                return (w.say(Word::WakeTimerKept, &[]), None);
            }
            let class = match period {
                Some(span) if span.as_nanos() == 604_800_000_000_000 => w.wake_weekly(at),
                Some(span) if span.as_nanos() == 86_400_000_000_000 => {
                    w.say(Word::WakeDaily, &[Hole::Name(w.turn_clock(at))])
                }
                Some(span) => w.say(Word::WakeEvery, &[Hole::Name(w.wake_span(span))]),
                None => w.say(Word::WakeOnce, &[Hole::Name(times.say(w, TimeForm::Clock, at))]),
            };
            return (w.say(Word::WakeTimer, &[]), Some(class));
        }
        Interest::Task { target, held, result } => concat(&[
            &w.say(Word::WakeTask, &[]),
            Decimal::of(target).as_bytes(),
            &w.say(task_suffix(held, result), &[]),
        ]),
        Interest::Topics { connector: temper_client_protocol_wire::FORGE, count: 1 } => w.say(Word::WakeForge, &[]),
        Interest::Topics { connector: temper_client_protocol_wire::FORGE, count } => {
            w.say(Word::WakeForgeTopics, &[Hole::Number(u64::from(count))])
        }
        Interest::Topics { connector, .. } => w.say(Word::WakeConnector, &[Hole::Number(u64::from(connector))]),
    };
    let class = match row.class {
        c::WakeRule::Immediate => w.say(Word::WakeAtOnce, &[]),
        c::WakeRule::Never => w.say(Word::WakeNever, &[]),
        c::WakeRule::Paced { span } => w.say(Word::WakeEvery, &[Hole::Name(w.wake_span(span))]),
        c::WakeRule::Batch { count, age } => {
            w.say(Word::WakeBatch, &[Hole::Number(u64::from(count)), Hole::Name(w.wake_span(age))])
        }
    };
    (left, Some(class))
}
