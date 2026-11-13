//! Round transitions, per member (temper/docs/design/domain/landing-history.md, 2.2, 3.2–3.3).
use crate::{Domain, LandingRow, Request, history};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Wall};
use temper_engine_domain_forge_landing as policy;

fn step(row: &LandingRow, member: &policy::Member, what: history::What) -> history::Step {
    history::Step {
        what,
        landing: Some(history::InLanding {
            task: row.landing.task,
            round: row.landing.preparation.max(1),
            unit: member.unit,
        }),
        head: None,
        base: None,
        operation: history::Operation::Of(row.landing.task),
        attempt: u32::from(row.landing.preparation.max(1)),
        source_at: None,
    }
}

fn round(d: &mut Domain, wall: Wall, row: &LandingRow, cause: history::RoundCause, out: &mut Queue<Request>) {
    for member in &row.landing.members {
        if member.base.is_none() {
            continue;
        }
        let mut event = step(row, member, history::What::RoundEnded { cause });
        event.base = member.base;
        d.write_event(wall, member.change, event, out);
    }
}

fn formed(d: &mut Domain, wall: Wall, row: &LandingRow, out: &mut Queue<Request>) {
    let mut members = List::uncapped();
    for member in &row.landing.members {
        if member.left.is_none() && member.base.is_some() {
            members.push(history::Member {
                change: member.change,
                unit: member.unit,
                head: member.head.expect("round head"),
            }).expect("admitted round members");
        }
    }
    let members = members.into_boxed();
    let mut place = 0_u16;
    for member in &row.landing.members {
        if member.left.is_some() || member.base.is_none() {
            continue;
        }
        place = place.checked_add(1).expect("bounded stack");
        if row.landing.preparation == 1 {
            let mut event = step(row, member, history::What::Joined {
                line: history::Line::Landing,
                place: Some(u32::from(place)),
                priority: None,
            });
            event.head = member.head;
            d.write_event(wall, member.change, event, out);
        }
        let mut event = step(row, member, history::What::Formed { members: members.clone(), place });
        event.head = member.head;
        event.base = member.base;
        d.write_event(wall, member.change, event, out);
    }
}

fn departed(d: &mut Domain, wall: Wall, before: &LandingRow, after: &LandingRow, out: &mut Queue<Request>) {
    for (old, member) in before.landing.members.iter().zip(&after.landing.members) {
        if old.left.is_some() {
            continue;
        }
        let Some(cause) = member.left else { continue };
        let with = match cause {
            policy::Leave::Conflict => Some(history::With::Branch),
            policy::Leave::EarlierConflict => Some(history::With::Earlier),
            policy::Leave::HeadMoved { .. } | policy::Leave::Cancelled | policy::Leave::Held
            | policy::Leave::Refused { .. } | policy::Leave::Failed { .. } | policy::Leave::Blamed { .. }
            | policy::Leave::Unit { .. } | policy::Leave::Dependency { .. } => None,
        };
        if let Some(with) = with {
            let mut event = step(before, old, history::What::Conflict { with, files: 0 });
            event.operation = history::Operation::Own;
            event.head = old.head;
            event.base = old.base;
            d.write_event(wall, member.change, event, out);
        }
        let mut event = step(before, old, history::What::Departed {
            line: history::Line::Landing,
            cause: history::Departure::Landing(cause),
        });
        // Membership starts in the first round, even when it ends in a later one.
        event.attempt = 1;
        event.head = old.head;
        d.write_event(wall, member.change, event, out);
    }
}

fn held(d: &mut Domain, wall: Wall, row: &LandingRow, reason: &[u8], out: &mut Queue<Request>) {
    for member in &row.landing.members {
        if member.left.is_some() {
            continue;
        }
        let open = d.change(member.change).and_then(landing_hold);
        if open.is_some() {
            continue;
        }
        let mut event = step(row, member, history::What::Held {
            on: history::On::Landing,
            reason: Box::from(reason),
            by: Some(history::By::Task(row.landing.task)),
        });
        event.operation = history::Operation::Own;
        if let Some(number) = d.write_event(wall, member.change, event, out) {
            hold_start(d, member.change, Some(number), out);
        }
    }
}
fn landing_hold(row: &crate::ChangeRow) -> Option<u64> {
    row.history?.landing_held
}
fn hold_start(d: &mut Domain, change: u64, number: Option<u64>, out: &mut Queue<Request>) {
    let row = d.changes.get_mut(&change).expect("member change");
    row.history.as_mut().expect("written history").landing_held = number;
    let row = row.clone();
    crate::domain::save_change(d, out, row);
}
fn released(d: &mut Domain, wall: Wall, row: &LandingRow, reason: &[u8], out: &mut Queue<Request>) {
    for member in &row.landing.members {
        let Some(number) = d.change(member.change).and_then(landing_hold) else { continue };
        let mut event = step(row, member, history::What::Released {
            on: history::On::Landing,
            reason: Box::from(reason),
            by: None,
        });
        event.operation = history::Operation::Of(number);
        event.attempt = 0;
        d.write_event(wall, member.change, event, out);
        hold_start(d, member.change, None, out);
    }
}

fn ended(d: &mut Domain, wall: Wall, before: &LandingRow, after: &LandingRow, outcome: history::LandingOutcome, out: &mut Queue<Request>) {
    let mut any = false;
    for member in &after.landing.members {
        if member.left.is_none() {
            any = true;
            d.write_event(wall, member.change, step(after, member, history::What::LandingEnded { outcome }), out);
        }
    }
    if !any {
        // The last departure is the last notified leave, or the newly set leave.
        let mut last = None;
        for (old, member) in before.landing.members.iter().zip(&after.landing.members) {
            if member.left.is_some() && (!old.leave_told || old.left.is_none()) {
                last = Some(member);
            }
        }
        if let Some(member) = last.or_else(|| after.landing.members.iter().rev().find(|member| member.base.is_some())).or(after.landing.members.last()) {
            d.write_event(wall, member.change, step(after, member, history::What::LandingEnded { outcome }), out);
        }
    }
}

pub(crate) fn transition(d: &mut Domain, wall: Wall, before: &LandingRow, after: &LandingRow, out: &mut Queue<Request>) {
    departed(d, wall, before, after, out);
    if after.landing.preparation != before.landing.preparation {
        formed(d, wall, after, out);
    }
    if before.landing.round.ending.is_none() && let Some(ending) = after.landing.round.ending {
        let cause = match ending {
            policy::RoundEnd::Stale => history::RoundCause::Stale,
            policy::RoundEnd::Leave | policy::RoundEnd::Conflict => history::RoundCause::Left,
            policy::RoundEnd::Cut => history::RoundCause::Cut,
        };
        round(d, wall, before, cause, out);
    }
    let mut turned = false;
    for (old, member) in before.landing.members.iter().zip(&after.landing.members) {
        turned |= old.landed.is_none() && member.landed.is_some() && member.candidate.is_some();
    }
    if turned {
        round(d, wall, before, history::RoundCause::Turned, out);
    }
    if let policy::Phase::Held { .. } = after.landing.phase {
        let reason = d.landing_hold_reason(after).unwrap_or(Box::new([]));
        held(d, wall, after, reason.get(..usize::try_from(history::REASON_BYTES).expect("reason bound")).unwrap_or(&reason), out);
    } else if let policy::Phase::Held { .. } = before.landing.phase {
        let reason = d.landing_hold_reason(before).unwrap_or(Box::new([]));
        released(d, wall, before, reason.get(..usize::try_from(history::REASON_BYTES).expect("reason bound")).unwrap_or(&reason), out);
    }
    if after.landing.phase == policy::Phase::Cancelled && before.landing.phase != policy::Phase::Cancelled && before.landing.phase != policy::Phase::Landed {
        if before.landing.round.ending.is_none() {
            round(d, wall, before, history::RoundCause::Cancelled, out);
        }
        ended(d, wall, before, after, history::LandingOutcome::Cancelled, out);
    } else if after.landing.phase == policy::Phase::Landed && before.landing.phase != policy::Phase::Landed {
        ended(d, wall, before, after, history::LandingOutcome::Done, out);
    }
}
