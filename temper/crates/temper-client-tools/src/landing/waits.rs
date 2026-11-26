//! A landing's wait and engine-relative ages (domain/landing-waits.md, 2, 4).
use super::{Encoder, Text, optional_number, w};
use crate::jig::render::document::clock;
use skein_lib::Wall;

pub(crate) fn age(now: u64, at: u64, out: &mut Text) {
    let seconds = now.saturating_sub(at).checked_div(1_000_000_000).expect("nonzero second");
    for (unit, next, suffix, next_suffix) in
        [(86400, 3600, b"d".as_slice(), b"h".as_slice()), (3600, 60, b"h", b"m"), (60, 1, b"m", b"s")]
    {
        if seconds >= unit {
            out.number(seconds.checked_div(unit).expect("nonzero unit"));
            out.put(suffix);
            out.put(b" ");
            out.number(seconds.checked_rem(unit).expect("nonzero unit").checked_div(next).expect("nonzero unit"));
            out.put(next_suffix);
            return;
        }
    }
    out.number(seconds);
    out.put(b"s");
}
fn name(on: &w::LandingWaitFor) -> &'static [u8] {
    match on {
        w::LandingWaitFor::Nothing => b"nothing",
        w::LandingWaitFor::Release => b"release",
        w::LandingWaitFor::Ready => b"ready",
        w::LandingWaitFor::Turn => b"turn",
        w::LandingWaitFor::Prepare => b"prepare",
        w::LandingWaitFor::Prefixes => b"prefixes",
        w::LandingWaitFor::Open => b"open",
        w::LandingWaitFor::Close => b"close",
        w::LandingWaitFor::Delete => b"delete",
        w::LandingWaitFor::Post => b"post",
        w::LandingWaitFor::Run => b"run",
        w::LandingWaitFor::Receipt => b"receipt",
        w::LandingWaitFor::Judgement => b"judgement",
        w::LandingWaitFor::Merge => b"merge",
        w::LandingWaitFor::Record => b"record",
        w::LandingWaitFor::Ancestry => b"ancestry",
        w::LandingWaitFor::Read => b"read",
        w::LandingWaitFor::Room => b"room",
    }
}
pub(super) fn text(wait: Option<&w::LandingWait>, out: &mut Text) {
    out.put(b"waits for: ");
    let Some(wait) = wait else {
        out.put(b"nothing; it steps next\n");
        return;
    };
    match wait.on {
        w::LandingWaitFor::Nothing => out.put(b"nothing; it steps next"),
        w::LandingWaitFor::Release => out.put(b"its release"),
        w::LandingWaitFor::Ready => {
            change(wait, out);
            out.put(b" to be ready");
        }
        w::LandingWaitFor::Turn => {
            out.put(b"the turn");
            if let Some(landing) = wait.landing.as_ref() {
                out.put(b", held by landing ");
                out.number(*landing);
            }
        }
        w::LandingWaitFor::Prepare => out.put(b"the round's prepare job"),
        w::LandingWaitFor::Prefixes => out.put(b"the prepared prefixes on the members' branches"),
        w::LandingWaitFor::Open => out.put(b"the round's pull request to open"),
        w::LandingWaitFor::Close => out.put(b"the round's pull request to close"),
        w::LandingWaitFor::Delete => {
            change(wait, out);
            out.put(b"'s branch in the landing to be deleted");
        }
        w::LandingWaitFor::Post => {
            gate(wait, b"'s status at ", out);
            prefix(wait, out);
        }
        w::LandingWaitFor::Run => {
            gate(wait, b"'s run ", out);
            if let Some(run) = wait.run.as_ref() {
                out.number(*run);
                out.put(b" ");
            }
            out.put(b"at ");
            prefix(wait, out);
        }
        w::LandingWaitFor::Receipt => {
            gate(wait, b"'s receipt for ", out);
            prefix(wait, out);
        }
        w::LandingWaitFor::Judgement => {
            change(wait, out);
            out.put(b"'s judgement at its prefix");
        }
        w::LandingWaitFor::Merge => out.put(b"the turn's merge"),
        w::LandingWaitFor::Record => {
            out.put(b"the record of ");
            change(wait, out);
            out.put(b"'s pull request as merged");
        }
        w::LandingWaitFor::Ancestry => {
            out.put(b"whether the branch holds ");
            change(wait, out);
            out.put(b"'s head");
        }
        w::LandingWaitFor::Read => out.put(b"a fresh read of its facts from the forge"),
        w::LandingWaitFor::Room => out.put(b"room in the forge's outbox"),
    }
    timing(wait, out);
    out.put(b"\n");
}
pub(super) fn timing(wait: &w::LandingWait, out: &mut Text) {
    if let Some(since) = wait.since {
        out.put(b" \xc2\xb7 since ");
        clock::deadline(Wall::from_nanos(since), out);
    }
    if let Some(bound) = &wait.bound {
        out.put(b" \xc2\xb7 ");
        match bound {
            w::LandingWaitBound::At(at) => {
                out.put(match at.then {
                    w::LandingWaitThen::Hold => b"holds at ",
                    w::LandingWaitThen::PassUp => b"passes up at ",
                    w::LandingWaitThen::Overdue => b"is overdue at ",
                    w::LandingWaitThen::GiveUp => b"gives up at ",
                });
                clock::deadline(Wall::from_nanos(at.at), out);
            }
            w::LandingWaitBound::None(bound) => {
                out.put(b"no bound: ");
                match &bound.reason {
                    w::LandingWaitUnbounded::NoDeadline => out.put(b"no deadline"),
                    w::LandingWaitUnbounded::Standing => out.put(b"standing"),
                    w::LandingWaitUnbounded::Behind(behind) => {
                        out.put(b"behind task ");
                        out.number(behind.task);
                    }
                    w::LandingWaitUnbounded::OperatorHold => out.put(b"an operator's hold"),
                }
            }
        }
    }
}
fn gate(wait: &w::LandingWait, words: &[u8], out: &mut Text) {
    out.put(b"gate ");
    if let Some(gate) = wait.gate.as_ref() {
        out.put(gate);
    }
    out.put(words);
}
fn prefix(wait: &w::LandingWait, out: &mut Text) {
    change(wait, out);
    out.put(b"'s prefix");
}
fn change(wait: &w::LandingWait, out: &mut Text) {
    out.put(b"change ");
    if let Some(change) = wait.change.as_ref() {
        out.number(*change);
    }
}
pub(super) fn json(wait: Option<&w::LandingWait>, live: bool, out: &mut Encoder) {
    if !live {
        out.null();
        return;
    }
    let (on, change, gate, run, landing, since, bound) = match wait {
        Some(wait) => (
            name(&wait.on),
            wait.change.as_ref(),
            wait.gate.as_ref(),
            wait.run.as_ref(),
            wait.landing.as_ref(),
            wait.since.as_ref(),
            wait.bound.as_ref(),
        ),
        None => (b"nothing".as_slice(), None, None, None, None, None, None),
    };
    out.object_start();
    out.key(b"for");
    out.string(on);
    out.key(b"change");
    optional_number(change, out);
    out.key(b"gate");
    match gate {
        Some(gate) => out.string(gate),
        None => out.null(),
    }
    out.key(b"run");
    optional_number(run, out);
    out.key(b"landing");
    optional_number(landing, out);
    out.key(b"since");
    optional_number(since, out);
    out.key(b"bound");
    bound_json(bound, out);
    out.object_end();
}

fn bound_json(bound: Option<&w::LandingWaitBound>, out: &mut Encoder) {
    let Some(bound) = bound else {
        out.null();
        return;
    };
    out.object_start();
    match bound {
        w::LandingWaitBound::At(at) => {
            out.key(b"at");
            out.unsigned(at.at);
            out.key(b"then");
            out.string(match at.then {
                w::LandingWaitThen::Hold => b"hold",
                w::LandingWaitThen::PassUp => b"pass_up",
                w::LandingWaitThen::Overdue => b"overdue",
                w::LandingWaitThen::GiveUp => b"give_up",
            });
        }
        w::LandingWaitBound::None(bound) => {
            out.key(b"none");
            match &bound.reason {
                w::LandingWaitUnbounded::NoDeadline => out.string(b"no_deadline"),
                w::LandingWaitUnbounded::Standing => out.string(b"standing"),
                w::LandingWaitUnbounded::OperatorHold => out.string(b"operator_hold"),
                w::LandingWaitUnbounded::Behind(behind) => {
                    out.object_start();
                    out.key(b"behind");
                    out.unsigned(behind.task);
                    out.object_end();
                }
            }
        }
    }
    out.object_end();
}
