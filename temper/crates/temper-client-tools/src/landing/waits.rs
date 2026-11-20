//! A landing's wait and engine-relative ages (domain/landing-waits.md, 2, 4).
use super::{Encoder, Text, optional_number, w};

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
    out.put(b"\n");
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
    out.object_start();
    out.key(b"for");
    out.string(match wait {
        Some(wait) => name(&wait.on),
        None => b"nothing",
    });
    out.key(b"change");
    optional_number(
        match wait {
            Some(wait) => wait.change.as_ref(),
            None => None,
        },
        out,
    );
    out.key(b"gate");
    match wait {
        Some(wait) => match wait.gate.as_ref() {
            Some(gate) => out.string(gate),
            None => out.null(),
        },
        None => out.null(),
    }
    out.key(b"run");
    optional_number(
        match wait {
            Some(wait) => wait.run.as_ref(),
            None => None,
        },
        out,
    );
    out.key(b"landing");
    optional_number(
        match wait {
            Some(wait) => wait.landing.as_ref(),
            None => None,
        },
        out,
    );
    out.object_end();
}
