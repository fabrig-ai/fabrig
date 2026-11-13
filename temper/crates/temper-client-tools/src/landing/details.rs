//! Train, round and prefix evidence (temper/docs/design/domain/landing.md, 14).
use super::{Encoder, Text, hex, optional_hex, w};

pub(super) fn step(value: &w::LandingStep) -> &'static [u8] {
    match value {
        w::LandingStep::Waiting => b"waiting",
        w::LandingStep::Preparing => b"preparing",
        w::LandingStep::Proving => b"proving",
        w::LandingStep::Probing => b"probing",
        w::LandingStep::Proved => b"proved",
        w::LandingStep::Turn => b"turn",
        w::LandingStep::Landed => b"landed",
        w::LandingStep::Held => b"held",
        w::LandingStep::Cancelled => b"cancelled",
        w::LandingStep::Judging => b"judging",
        w::LandingStep::Leaving => b"leaving",
        w::LandingStep::Recording => b"recording",
        w::LandingStep::Telling => b"telling",
    }
}
fn combined(value: &w::Combined) -> &'static [u8] {
    match value {
        w::Combined::Merge => b"merge",
        w::Combined::Squash => b"squash",
    }
}
fn verdict(value: &w::Ci) -> &'static [u8] {
    match value {
        w::Ci::Unknown => b"unknown",
        w::Ci::Pending => b"pending",
        w::Ci::Passed => b"passed",
        w::Ci::Failed => b"failed",
    }
}
pub(super) fn train_text(value: &w::Train, out: &mut Text) {
    out.put(b"train: ");
    out.put(value.owner());
    out.put(b"/");
    out.put(value.name());
    out.put(b" on forge ");
    out.number(u64::from(value.forge()));
    out.put(b" into ");
    out.put(value.branch());
}
pub(super) fn round_text(value: &w::LandingRound, out: &mut Text) {
    out.put(b"round ");
    out.number(u64::from(value.number()));
    out.put(b": ");
    out.put(value.branch());
    if let Some(pull) = value.pull() {
        out.put(b" \xc2\xb7 pull request #");
        out.number(pull.number());
        out.put(b" \xc2\xb7 ");
        out.put(pull.address());
    }
    if let Some(how) = value.combined() {
        out.put(b" \xc2\xb7 ");
        out.put(combined(how));
    }
    out.put(b" \xc2\xb7 step: ");
    out.put(step(value.step()));
    if *value.step() == w::LandingStep::Probing
        && let Some(search) = value.search()
        && let Some(prefix) = search.probe()
    {
        out.put(b" at ");
        out.put(&hex(prefix));
    }
    out.put(b"\n");
    if let Some(search) = value.search() {
        out.put(b"search: green member ");
        bound_text(search.green(), out);
        out.put(b" \xc2\xb7 red member ");
        bound_text(search.red(), out);
        out.put(b"\n");
    }
    if let Some(upstream) = value.upstream() {
        out.put(b"upstream: ");
        out.put(&hex(upstream));
        out.put(b"\n");
    }
    if !value.failure().is_empty() {
        out.put(b"preparation fault: ");
        out.put(value.failure());
        out.put(b"\n");
    }
}
fn bound_text(value: &w::SearchBound, out: &mut Text) {
    out.number(u64::from(value.member()));
    out.put(b" at ");
    out.put(&hex(value.prefix()));
}
pub(super) fn member_text(value: &w::Member, out: &mut Text) {
    if let Some(head) = value.head() {
        out.put(b"  round head: ");
        out.put(&hex(head));
        out.put(b"\n");
    }
    if let Some(judgement) = value.judgement() {
        out.put(b"  judgement at ");
        out.put(&hex(judgement.prefix()));
        out.put(b": ");
        out.put(verdict(judgement.verdict()));
        out.put(b"\n");
        for gate in judgement.gates() {
            out.put(b"    gate ");
            out.number(gate.number());
            out.put(if gate.blocking() { b", blocking: " } else { b", advisory: " });
            out.put(verdict(gate.verdict()));
            out.put(b"\n");
        }
    }
    if let Some(left) = value.left() {
        out.put(b"  left: ");
        left_text(left, out);
        out.put(b"\n");
    }
}
fn left_text(value: &w::LandingLeave, out: &mut Text) {
    match value {
        w::LandingLeave::Conflict => out.put(b"conflict"),
        w::LandingLeave::Cancelled => out.put(b"cancelled"),
        w::LandingLeave::Held => out.put(b"held"),
        w::LandingLeave::Blamed(value) => {
            out.put(b"blamed by gate ");
            out.number(u64::from(value.gate()));
            out.put(b" in run ");
            out.number(value.run());
            out.put(b", red at ");
            out.put(&hex(value.red()));
            out.put(b", green at ");
            out.put(&hex(value.green()));
        }
        w::LandingLeave::HeadMoved(value) => {
            out.put(b"head moved to ");
            out.put(&hex(value.head()));
        }
        w::LandingLeave::Refused(value) => {
            out.put(b"judgement refused by gate ");
            out.number(value.gate());
        }
        w::LandingLeave::Failed(value) => {
            out.put(b"gate ");
            out.number(u64::from(value.gate()));
            out.put(b" failed in run ");
            out.number(value.run());
        }
    }
}
pub(super) fn hold_text(value: &w::LandingHold, out: &mut Text) {
    match value {
        w::LandingHold::Faults(_) => (),
        w::LandingHold::Prepare => out.put(b"held: preparation failed\n"),
        w::LandingHold::Record(value) => {
            out.put(b"held: recording member ");
            out.number(u64::from(value.member()));
            out.put(b" failed\n");
        }
        w::LandingHold::Timeout(value) => {
            out.put(b"held: gate ");
            out.number(u64::from(value.gate()));
            out.put(b" of member ");
            out.number(u64::from(value.member()));
            out.put(b", run ");
            out.number(value.run());
            out.put(b" timed out\n");
        }
        w::LandingHold::Turn => out.put(b"held: turn refused\n"),
    }
}
pub(super) fn train_json(value: Option<&w::Train>, out: &mut Encoder) {
    let Some(value) = value else {
        out.null();
        return;
    };
    out.object_start();
    out.key(b"forge");
    out.unsigned(u64::from(value.forge()));
    out.key(b"repository");
    out.unsigned(u64::from(value.repository()));
    for (key, text) in [(b"owner".as_slice(), value.owner()), (b"name", value.name()), (b"branch", value.branch())] {
        out.key(key);
        out.string(text);
    }
    out.object_end();
}
pub(super) fn round_json(value: Option<&w::LandingRound>, out: &mut Encoder) {
    let Some(value) = value else {
        out.null();
        return;
    };
    out.object_start();
    out.key(b"number");
    out.unsigned(u64::from(value.number()));
    out.key(b"branch");
    out.string(value.branch());
    out.key(b"pull");
    match value.pull() {
        Some(pull) => {
            out.object_start();
            out.key(b"number");
            out.unsigned(pull.number());
            out.key(b"address");
            out.string(pull.address());
            out.object_end();
        }
        None => out.null(),
    }
    out.key(b"combined");
    match value.combined() {
        Some(value) => out.string(combined(value)),
        None => out.null(),
    }
    out.key(b"step");
    out.string(step(value.step()));
    out.key(b"upstream");
    optional_hex(value.upstream().as_ref(), out);
    out.key(b"failure");
    out.string(value.failure());
    if let Some(search) = value.search() {
        out.key(b"search");
        out.object_start();
        out.key(b"green");
        bound_json(search.green(), out);
        out.key(b"red");
        bound_json(search.red(), out);
        out.key(b"probe");
        optional_hex(search.probe().as_ref(), out);
        out.object_end();
    }
    out.object_end();
}
fn bound_json(value: &w::SearchBound, out: &mut Encoder) {
    out.object_start();
    out.key(b"member");
    out.unsigned(u64::from(value.member()));
    out.key(b"prefix");
    out.string(&hex(value.prefix()));
    out.object_end();
}
pub(super) fn member_json(value: &w::Member, out: &mut Encoder) {
    out.key(b"head");
    optional_hex(value.head().as_ref(), out);
    out.key(b"left");
    match value.left() {
        Some(value) => left_json(value, out),
        None => out.null(),
    }
    out.key(b"judgement");
    match value.judgement() {
        Some(value) => {
            out.object_start();
            out.key(b"prefix");
            out.string(&hex(value.prefix()));
            out.key(b"verdict");
            out.string(verdict(value.verdict()));
            out.key(b"gates");
            out.array_start();
            for gate in value.gates() {
                out.object_start();
                out.key(b"number");
                out.unsigned(gate.number());
                out.key(b"blocking");
                out.boolean(gate.blocking());
                out.key(b"verdict");
                out.string(verdict(gate.verdict()));
                out.object_end();
            }
            out.array_end();
            out.object_end();
        }
        None => out.null(),
    }
}
fn left_json(value: &w::LandingLeave, out: &mut Encoder) {
    match value {
        w::LandingLeave::Conflict => out.string(b"conflict"),
        w::LandingLeave::Cancelled => out.string(b"cancelled"),
        w::LandingLeave::Held => out.string(b"held"),
        w::LandingLeave::Blamed(value) => {
            out.object_start();
            out.key(b"blamed");
            out.object_start();
            out.key(b"gate");
            out.unsigned(u64::from(value.gate()));
            out.key(b"run");
            out.unsigned(value.run());
            out.key(b"red");
            out.string(&hex(value.red()));
            out.key(b"green");
            out.string(&hex(value.green()));
            out.object_end();
            out.object_end();
        }
        w::LandingLeave::HeadMoved(value) => {
            out.object_start();
            out.key(b"head_moved");
            out.string(&hex(value.head()));
            out.object_end();
        }
        w::LandingLeave::Refused(value) => {
            out.object_start();
            out.key(b"refused");
            out.unsigned(value.gate());
            out.object_end();
        }
        w::LandingLeave::Failed(value) => {
            out.object_start();
            out.key(b"failed");
            out.object_start();
            out.key(b"gate");
            out.unsigned(u64::from(value.gate()));
            out.key(b"run");
            out.unsigned(value.run());
            out.object_end();
            out.object_end();
        }
    }
}
pub(super) fn hold_json(value: &w::LandingHold, out: &mut Encoder) {
    match value {
        w::LandingHold::Prepare => out.string(b"prepare"),
        w::LandingHold::Turn => out.string(b"turn"),
        w::LandingHold::Record(value) => {
            out.object_start();
            out.key(b"record");
            out.unsigned(u64::from(value.member()));
            out.object_end();
        }
        w::LandingHold::Faults(value) => {
            out.object_start();
            out.key(b"faults");
            out.object_start();
            out.key(b"member");
            out.unsigned(u64::from(value.member()));
            out.key(b"gate");
            out.unsigned(u64::from(value.gate()));
            out.object_end();
            out.object_end();
        }
        w::LandingHold::Timeout(value) => {
            out.object_start();
            out.key(b"timeout");
            out.object_start();
            out.key(b"member");
            out.unsigned(u64::from(value.member()));
            out.key(b"gate");
            out.unsigned(u64::from(value.gate()));
            out.key(b"run");
            out.unsigned(value.run());
            out.object_end();
            out.object_end();
        }
    }
}
pub(super) fn live(phase: &w::LandingPhase) -> bool {
    match phase {
        w::LandingPhase::Landed | w::LandingPhase::Cancelled => false,
        w::LandingPhase::Waiting
        | w::LandingPhase::Preparing
        | w::LandingPhase::Proving
        | w::LandingPhase::Proved
        | w::LandingPhase::Turn
        | w::LandingPhase::Held => true,
    }
}
