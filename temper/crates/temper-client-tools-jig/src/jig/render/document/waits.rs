//! A task's supplied wait records (client/ctl/results.md, 1 and 2).
use super::{Cut, Me, Text, acts, clock, llm};
use crate::jig::render::Parts;
use jig_client::contract::{self as c, waits::Bound};
use skein_json::writer::Encoder;
use skein_lib::Wall;

const fn kind(value: c::WaitKind) -> &'static [u8] {
    match value {
        c::WaitKind::Party => b"party",
        c::WaitKind::Question => b"question",
        c::WaitKind::Dependency => b"dependency",
        c::WaitKind::Resource => b"resource",
        c::WaitKind::Account => b"account",
        c::WaitKind::Host => b"host",
        c::WaitKind::Delegate => b"delegate",
        c::WaitKind::News => b"news",
        c::WaitKind::Step => b"step",
        c::WaitKind::Room => b"room",
        c::WaitKind::Turn => b"turn",
        c::WaitKind::SignOff => b"sign-off",
    }
}

pub(super) fn entry_kind(value: c::WaitedFor) -> &'static [u8] {
    match value {
        c::WaitedFor::Question => b"question",
        c::WaitedFor::Proposal => b"proposal",
        c::WaitedFor::Escalation => b"escalation",
        c::WaitedFor::PersonTask => b"person task",
    }
}

pub(super) fn summary(wait: Option<&c::Wait>, nearest: Option<c::Nearest>, out: &mut Text) {
    let Some(wait) = wait else { return };
    since(wait.since, out);
    if let Some(near) = nearest {
        out.put(b" \xc2\xb7 ");
        if near.kind != wait.kind || wait.bound != (Bound::At { at: near.at, then: near.then }) {
            out.put(kind(near.kind));
            out.put(b" ");
        }
        at_bound(near.at, near.then, out);
    }
}

pub(super) fn text(task: &c::Task, me: Option<&Me>, vocabulary: &dyn Parts, out: &mut Text) {
    if task.wait_count == 0 {
        return;
    }
    out.put(b"waits: ");
    out.number(u64::from(task.wait_count));
    out.put(b"\n");
    for wait in &task.waits {
        out.put(b"  ");
        object_text(wait, me, vocabulary, out);
        since(wait.since, out);
        out.put(b" \xc2\xb7 ");
        match wait.bound {
            Bound::At { at, then } => at_bound(at, then, out),
            Bound::None(reason) => no_bound(reason, out),
        }
        if let Some(hears) = wait.hears {
            out.put(b" \xc2\xb7 hears ");
            holder(hears, me, out);
        }
        out.put(b"\n");
    }
    let more = task.wait_count.saturating_sub(task.waits.len());
    if more != 0 {
        out.put(b"  \xe2\x80\xa6 ");
        out.number(u64::from(more));
        out.put(b" more\n");
    }
}

fn since(at: Wall, out: &mut Text) {
    out.put(b" \xc2\xb7 since ");
    clock::deadline(at, out);
}
fn at_bound(at: Wall, then: c::AtBound, out: &mut Text) {
    out.put(match then {
        c::AtBound::Hold => b"holds at ",
        c::AtBound::PassUp => b"passes up at ",
        c::AtBound::Overdue => b"is overdue at ",
        c::AtBound::GiveUp => b"gives up at ",
    });
    clock::deadline(at, out);
}
fn no_bound(reason: c::Unbounded, out: &mut Text) {
    out.put(b"no bound: ");
    match reason {
        c::Unbounded::NoDeadline => out.put(b"no deadline"),
        c::Unbounded::Standing => out.put(b"standing"),
        c::Unbounded::Behind(task) => {
            out.put(b"behind ");
            task_text(task, out);
        }
        c::Unbounded::OperatorHold => out.put(b"an operator's hold"),
    }
}
fn holder(value: c::EscalationHolder, me: Option<&Me>, out: &mut Text) {
    match value {
        c::EscalationHolder::Task(task) => task_text(task, out),
        c::EscalationHolder::Person(party) => acts::party(party, me, out),
        c::EscalationHolder::Role { project, role } => acts::recipient(&c::Recipient::Role { project, role }, me, out),
    }
}
fn task_text(task: u64, out: &mut Text) {
    out.put(b"task ");
    out.number(task);
}
fn object_text(wait: &c::Wait, me: Option<&Me>, vocabulary: &dyn Parts, out: &mut Text) {
    if let (c::WaitKind::Account, c::WaitObject::Run(run @ c::TaskRunWait::Account { .. }))
    | (c::WaitKind::Host, c::WaitObject::Run(run @ c::TaskRunWait::Host)) = (wait.kind, &wait.object)
    {
        llm::run_wait_text(*run, out);
        return;
    }
    out.put(kind(wait.kind));
    match &wait.object {
        c::WaitObject::Entry { kind, number, at } => {
            out.put(b" ");
            out.put(entry_kind(*kind));
            out.put(b" ");
            out.number(*number);
            out.put(b" at ");
            holder(*at, me, out);
        }
        c::WaitObject::Question { number, answerer } => {
            out.put(b" ");
            out.number(*number);
            out.put(b" to ");
            task_text(*answerer, out);
        }
        c::WaitObject::Task(task) => {
            out.put(b" ");
            task_text(*task, out);
        }
        c::WaitObject::Resource { name, holder } => {
            out.put(b" ");
            vocabulary.owner_part(
                &c::Part {
                    connector: name.connector,
                    kind: c::PartKind::Resource,
                    key: c::PartKey::Path(name.path.clone()),
                },
                out,
            );
            if let Some(task) = holder {
                out.put(b" held by ");
                task_text(*task, out);
            }
        }
        c::WaitObject::Run(run) => {
            out.put(b" ");
            llm::run_wait_text(*run, out);
        }
        c::WaitObject::News { subscriptions, timer } => news(subscriptions, *timer, out),
        c::WaitObject::Procedure { step, things } => procedure(*step, things, vocabulary, out),
    }
}
fn news(subscriptions: &skein_lib::List<u64>, timer: Option<Wall>, out: &mut Text) {
    if !subscriptions.is_empty() {
        out.put(b" subscriptions ");
        let mut separator = b"".as_slice();
        for number in subscriptions {
            out.put(separator);
            out.number(*number);
            separator = b", ";
        }
    }
    if let Some(at) = timer {
        out.put(if subscriptions.is_empty() { b" timer at " } else { b" and timer at " });
        clock::deadline(at, out);
    } else if subscriptions.is_empty() {
        out.put(b" nothing armed");
    }
}
fn procedure(step: u32, things: &skein_lib::List<c::Thing>, vocabulary: &dyn Parts, out: &mut Text) {
    if let Some(name) = vocabulary.wait_step(step) {
        out.put(b" at ");
        out.put(name);
    } else {
        out.put(b" ");
        out.number(u64::from(step));
    }
    let mut separator = b": ".as_slice();
    for thing in things {
        out.put(separator);
        if let Some((name, number_separator)) = vocabulary.wait_thing(thing.what) {
            out.put(name);
            out.put(number_separator);
        } else {
            out.put(b"thing ");
            out.number(u64::from(thing.what));
            out.put(b" ");
        }
        out.number(thing.number);
        if let Some(owner) = thing.owner {
            out.put(b" of ");
            task_text(owner, out);
        }
        separator = b", ";
    }
}

pub(super) fn encode_task(task: &c::Task, out: &mut Encoder, budget: &mut Cut) {
    encode_summary(task.wait.as_ref(), task.nearest, out, budget);
    out.key(b"waits");
    out.array_start();
    for wait in &task.waits {
        encode_wait(wait, out, budget);
    }
    out.array_end();
    out.key(b"wait_count");
    out.unsigned(u64::from(task.wait_count));
}
pub(super) fn encode_summary(wait: Option<&c::Wait>, nearest: Option<c::Nearest>, out: &mut Encoder, budget: &mut Cut) {
    out.key(b"wait");
    match wait {
        Some(wait) => encode_wait(wait, out, budget),
        None => out.null(),
    }
    out.key(b"nearest");
    match nearest {
        Some(value) => {
            out.object_start();
            encode_at(value.at, value.then, out);
            out.key(b"kind");
            out.string(kind(value.kind));
            out.object_end();
        }
        None => out.null(),
    }
}
fn encode_wait(wait: &c::Wait, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    out.key(b"task");
    out.unsigned(wait.task);
    out.key(b"kind");
    out.string(kind(wait.kind));
    out.key(b"object");
    encode_object(&wait.object, out, budget);
    out.key(b"since");
    out.unsigned(wait.since.as_nanos());
    out.key(b"bound");
    out.object_start();
    match wait.bound {
        Bound::At { at, then } => encode_at(at, then, out),
        Bound::None(reason) => {
            out.key(b"none");
            encode_unbounded(reason, out);
        }
    }
    out.object_end();
    out.key(b"hears");
    match &wait.hears {
        Some(holder) => super::encode_owner_escalation_holder(holder, out, budget),
        None => out.null(),
    }
    out.object_end();
}
fn encode_at(at: Wall, then: c::AtBound, out: &mut Encoder) {
    out.key(b"at");
    out.unsigned(at.as_nanos());
    out.key(b"then");
    out.string(match then {
        c::AtBound::Hold => b"hold",
        c::AtBound::PassUp => b"pass_up",
        c::AtBound::Overdue => b"overdue",
        c::AtBound::GiveUp => b"give_up",
    });
}
fn encode_unbounded(value: c::Unbounded, out: &mut Encoder) {
    match value {
        c::Unbounded::NoDeadline => out.string(b"no_deadline"),
        c::Unbounded::Standing => out.string(b"standing"),
        c::Unbounded::OperatorHold => out.string(b"operator_hold"),
        c::Unbounded::Behind(task) => {
            out.object_start();
            out.key(b"behind");
            out.unsigned(task);
            out.object_end();
        }
    }
}
fn encode_object(object: &c::WaitObject, out: &mut Encoder, budget: &mut Cut) {
    out.object_start();
    match object {
        c::WaitObject::Entry { kind, number, at } => {
            out.key(b"entry");
            out.object_start();
            out.key(b"kind");
            super::templates::encode_waited_for(*kind, out, budget);
            out.key(b"number");
            out.unsigned(*number);
            out.key(b"at");
            super::encode_owner_escalation_holder(at, out, budget);
            out.object_end();
        }
        c::WaitObject::Question { number, answerer } => {
            out.key(b"question");
            out.object_start();
            out.key(b"number");
            out.unsigned(*number);
            out.key(b"answerer");
            out.unsigned(*answerer);
            out.object_end();
        }
        c::WaitObject::Task(task) => {
            out.key(b"task");
            out.unsigned(*task);
        }
        c::WaitObject::Resource { name, holder } => {
            out.key(b"resource");
            encode_resource(name, *holder, out);
        }
        c::WaitObject::Run(run) => {
            out.key(b"run");
            llm::encode_run_wait(Some(*run), out);
        }
        c::WaitObject::News { subscriptions, timer } => {
            out.key(b"news");
            out.object_start();
            out.key(b"subscriptions");
            out.array_start();
            for number in subscriptions {
                out.unsigned(*number);
            }
            out.array_end();
            out.key(b"timer");
            match timer {
                Some(at) => out.unsigned(at.as_nanos()),
                None => out.null(),
            }
            out.object_end();
        }
        c::WaitObject::Procedure { step, things } => {
            out.key(b"procedure");
            encode_procedure(*step, things, out);
        }
    }
    out.object_end();
}
fn encode_resource(name: &c::ResourceName, holder: Option<u64>, out: &mut Encoder) {
    out.object_start();
    out.key(b"name");
    out.object_start();
    super::parts::encode_resource_name(name.connector, &name.path, out);
    out.object_end();
    out.key(b"holder");
    optional_number(holder, out);
    out.object_end();
}
fn encode_procedure(step: u32, things: &skein_lib::List<c::Thing>, out: &mut Encoder) {
    out.object_start();
    out.key(b"step");
    out.unsigned(u64::from(step));
    out.key(b"things");
    out.array_start();
    for thing in things {
        out.object_start();
        out.key(b"what");
        out.unsigned(u64::from(thing.what));
        out.key(b"number");
        out.unsigned(thing.number);
        out.key(b"owner");
        optional_number(thing.owner, out);
        out.object_end();
    }
    out.array_end();
    out.object_end();
}
fn optional_number(value: Option<u64>, out: &mut Encoder) {
    match value {
        Some(value) => out.unsigned(value),
        None => out.null(),
    }
}
