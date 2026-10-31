//! Task acknowledgements, holdings, and keyed terminals
//! (jig/docs/design/domain/engine.md, 4.4).

use crate::flight::{Flight, Internal};
use crate::routing::children::append_people;
use crate::routing::runs::call_decision;
use crate::{
    Ask, CallPart, Core, CoreRecord, Held, Key, Limits, Now, PersonTaskRoute, Record, Request, SentRoute, Write,
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo};

pub(super) fn lifted(core: &mut Core, env: &Env<Limits>, reply_to: ReplyTo, released: u32, out: &mut Queue<Request>) {
    let token = reply_to.into_token();
    let Some(Flight::PersonTask { request, route: PersonTaskRoute::Lift(hold) }) = core.land(token) else {
        return;
    };
    append_people(
        core,
        env,
        people::Event::Decided { request, outcome: people::Outcome::Lifted { hold, released } },
        out,
    );
}

pub(super) fn released(core: &mut Core, env: &Env<Limits>, revision: u32, out: &mut Queue<Request>) {
    core.release_revision(env, revision, out);
}

pub(super) fn erase(key: tasks::Key, out: &mut Queue<Request>) {
    out.push(Request::Write(Write::Erase(Key::Tasks(key))));
}

pub(super) fn relay(core: &mut Core, env: &Env<Limits>, task: u64, out: &mut Queue<Request>) {
    let Some(relay) = crate::relays::next(core, task, env.limits.tasks.run_inbox) else { return };
    out.push(Request::Held(Box::new(relay)));
}

pub(super) fn resolve_uncertain(core: &mut Core, entry: u64, again: bool, out: &mut Queue<Request>) {
    for connector in &core.connectors {
        out.push(Request::Ask {
            connector: *connector,
            ask: Ask::Effect(crate::connector::Ask::ResolveUncertain { entry, again }),
        });
    }
}

pub(super) fn sent(
    core: &mut Core,
    env: &Env<Limits>,
    reply_to: ReplyTo,
    task: u64,
    word: tasks::Word,
    out: &mut Queue<Request>,
) {
    let request = reply_to.into_token();
    match core.sent(request, task, &word, env.limits.tasks.inbox_messages, env.limits.tasks.tasks) {
        SentRoute::Call { request, key, message } => {
            call_decision(core, out, ReplyTo::new(request), key, CallPart::Sent { message });
        }
        SentRoute::Person(event) => append_people(core, env, *event, out),
    }
}

pub(super) fn stop(task: u64, attempt: u64, out: &mut Queue<Request>) {
    out.push(Request::Held(Box::new(Held::StopRun { task, attempt })));
}

pub(super) fn restore_refused(core: &mut Core) {
    core.start_collect_failure();
    assert!(core.start_failure().is_some(), "tasks refusal names its fit");
}

pub(super) fn taken(task: u64, holdings: Box<[tasks::Holding]>, out: &mut Queue<Request>) {
    for holding in holdings {
        match holding {
            tasks::Holding::Write { resource, .. } => {
                out.push(Request::Ask { connector: resource.connector, ask: Ask::Hold { task, resource } });
            }
            tasks::Holding::Slot { .. } => {}
        }
    }
}

pub(super) fn end_topic(task: u64, subscription: u64, connector: u16, out: &mut Queue<Request>) {
    out.push(Request::Ask { connector, ask: Ask::EndTopic { task, subscription } });
}

pub(super) fn writer_waiting(
    core: &mut Core,
    env: &Env<Limits>,
    task: u64,
    pending: &mut Queue<tasks::Request>,
    out: &mut Queue<Request>,
) {
    if core.claiming.remove(&task).is_some() {
        core.restore_replaced_proof(task);
        out.push(Request::Now(Box::new(Now::DropAssignment { task })));
        let mut follow = Queue::uncapped();
        crate::routing::step::step_tasks(core, env, tasks::Event::PreparationFailed { task }, &mut follow);
        for _ in 0..follow.len() {
            pending.push(follow.pop().expect("writer wait output count"));
        }
    }
}

pub(super) fn acknowledged(core: &mut Core, reply_to: ReplyTo, task: u64, attempt: u64, out: &mut Queue<Request>) {
    let token = reply_to.into_token();
    match core.land(token) {
        Some(Flight::Internal(Internal::Acknowledge)) => {}
        Some(_) | None => out.push(Request::Now(Box::new(Now::DropPayload { payload: token }))),
    }
    if let Some(proof) = core.proofs.get(&task) {
        if let Some(terminal) = &proof.terminal {
            out.push(Request::Write(Write::Save(Record::Core(CoreRecord::Terminal(terminal.clone())))));
        }
        out.push(Request::Write(Write::Save(Record::Core(CoreRecord::RunProof(proof.clone())))));
    }
    out.push(Request::Held(Box::new(Held::TaskTerminalAcknowledged { task, attempt })));
    for &connector in core.connectors.as_ref() {
        out.push(Request::Ask { connector, ask: Ask::Lost { task, attempt } });
    }
}

pub(super) fn turn_acknowledged(
    reply_to: ReplyTo,
    task: u64,
    attempt: u64,
    turn: u32,
    accepted: tasks::Accepted,
    out: &mut Queue<Request>,
) {
    out.push(Request::Now(Box::new(Now::AcceptedTurn {
        payload: reply_to.into_token(),
        task,
        attempt,
        turn,
        accepted,
    })));
}
