//! Serve the core's start script through its ports (`jig/docs/design/domain/engine/vocabulary.md`, 2.6).

use super::{Action, ApplicationWhy, Failure, Phase, RestoreRange, SweepInput};
use crate::{
    Core, Limits, Record, Request,
    engine::{self, Out, Peer, Resume, application, store},
};
use alloc::boxed::Box;
use jig_domain_contract::{Ask, Connector, Up, lifecycle, resources, settings};
use skein_lib::{Env, Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Wait {
    Dormant,
    Ready,
    Load(Token),
    Restored(Connector),
    Switched,
    Resume,
    Halted,
}

#[derive(Hash, Debug)]
struct SweepPage {
    items: Queue<store::Item>,
    next: Option<store::Cursor>,
}

#[derive(Hash, Debug)]
pub(super) struct State {
    wait: Wait,
    sweep: Option<SweepPage>,
    ranges: Option<Box<[RestoreRange]>>,
}

impl State {
    pub(super) const fn new() -> Self {
        Self { wait: Wait::Dormant, sweep: None, ranges: None }
    }
}

/// Drop the port exchanges when this start will not advance again.
pub(crate) fn stop(core: &mut Core) {
    if core.start_phase().is_some() && core.start.port.wait != Wait::Dormant {
        halt(core);
    }
}

fn halt(core: &mut Core) {
    core.start.port.wait = Wait::Halted;
    core.start.port.sweep = None;
    core.start.port.ranges = None;
    core.start.waiting = None;
    core.start.census = None;
    core.start.fill = None;
}

pub(crate) fn store(core: &mut Core, env: &Env<Limits>, input: store::In, out: &mut Queue<Out>) {
    match input {
        store::In::Opened { pending } => {
            if !core.stopping {
                let action = core.start_begin(pending, env.wall);
                drive(core, env, action, out);
            }
        }
        store::In::Loaded { load, result } if core.start.port.wait == Wait::Load(load) => {
            core.start.port.wait = Wait::Ready;
            match result {
                store::Answer::Rows(store::Rows::Start { items, next }) => page(core, env, items, next, out),
                store::Answer::Rows(
                    store::Rows::Records { .. } | store::Rows::Read { .. } | store::Rows::Turns { .. },
                )
                | store::Answer::Cut { .. }
                | store::Answer::Failed(_) => halt(core),
            }
        }
        store::In::Switched => {
            if core.start.port.wait == Wait::Switched {
                finish(core, env, Phase::Switch, out);
            }
        }
        store::In::Fill { .. } | store::In::Census(..) if core.start.port.wait == Wait::Halted => {}
        input @ (store::In::Fill { .. } | store::In::Census(..) | store::In::Loaded { .. }) => {
            let requests = crate::routing::step_store(core, env, input);
            engine::append(requests, out);
        }
    }
}

pub(crate) fn connector(core: &mut Core, env: &Env<Limits>, number: Connector, up: Up, out: &mut Queue<Out>) {
    let restoring = core.start.port.wait == Wait::Restored(number);
    match up {
        Up::Lifecycle(lifecycle::Up::Done { stage: lifecycle::RestartStage::Restored }) => {
            if restoring {
                finish(core, env, Phase::RestoreConnector { connector: number.0 }, out);
            }
        }
        Up::Settings(settings::Up::Unfit { kind, scope, number }) => {
            if restoring {
                let action =
                    core.start_refuse(Failure::Application { kind, scope, number, why: ApplicationWhy::Shape });
                drive(core, env, action, out);
            }
        }
        Up::Resources(resources::Up::Resource { project, name, role, .. }) if restoring => {
            if name.connector == number.0 {
                let _restored = core.restore_resource_role(env, project, name, role);
            }
        }
        Up::Lifecycle(lifecycle::Up::Adopted { project, name, role, home }) if restoring => {
            if name.connector == number.0 {
                crate::effects::adopted(core, &env.limits, project, name, role, home);
            }
        }
        Up::Settings(settings::Up::Defaults { defaults, dropped }) if restoring => {
            crate::settings::application::defaults(core, number.0, defaults, dropped);
            core.start_collect_failure();
            if let Some(failure) = core.start_failure() {
                drive(core, env, Action::Refused(failure), out);
            }
        }
        Up::Resources(resources::Up::Resource { .. })
        | Up::Lifecycle(lifecycle::Up::Adopted { .. })
        | Up::Settings(settings::Up::Defaults { .. })
            if core.start_phase().is_some() && core.start.port.wait != Wait::Dormant => {}
        Up::Lifecycle(lifecycle::Up::Done {
            stage: lifecycle::RestartStage::ReadAfresh | lifecycle::RestartStage::Settled,
        }) if !core.start_opened() && core.start.port.wait != Wait::Dormant => {}
        up @ (Up::Lifecycle(
            lifecycle::Up::Done { stage: lifecycle::RestartStage::ReadAfresh | lifecycle::RestartStage::Settled }
            | lifecycle::Up::Released { .. }
            | lifecycle::Up::Adopted { .. },
        )
        | Up::Resources(_)
        | Up::Effects(_)
        | Up::Topics(_)
        | Up::Parts(_)
        | Up::Procedures(_)
        | Up::Projections(_)
        | Up::Settings(
            settings::Up::Defaults { .. } | settings::Up::Judged { .. } | settings::Up::Restored { .. },
        )
        | Up::Requests(_)
        | Up::Act { .. }) => {
            let requests = crate::routing::step_connector(core, env, number, up);
            engine::append(requests, out);
        }
    }
}

pub(crate) fn resume(core: &mut Core, env: &Env<Limits>, resume: Resume, out: &mut Queue<Out>) {
    if core.start.port.wait != Wait::Resume {
        return;
    }
    let continuation = core.start.waiting.as_ref().expect("held start continuation");
    if resume.0.raw() != continuation.sequence {
        return;
    }
    let continuation = continuation.clone();
    let previous = core.start_phase();
    let mut requests = Queue::uncapped();
    core.start.port.wait = Wait::Ready;
    let action = core.start_resume(env, continuation, &mut requests);
    if previous == Some(Phase::Sweep) {
        let Action::Step(Phase::Sweep) = action else {
            drive(core, env, action, out);
            return;
        };
        engine::append(requests, out);
        sweep(core, env, out);
    } else if core.start_phase() == Some(Phase::AdoptRuns) {
        // The continuation activates the restored tasks as it enters this phase.
        report(out, application::Out::Start { phase: Some(Phase::AdoptRuns) });
        append_started(&action, requests, out);
        match action {
            Action::Step(Phase::AdoptRuns) => finish(core, env, Phase::AdoptRuns, out),
            action @ (Action::Refused(_)
            | Action::Idle
            | Action::Commit(_)
            | Action::Step(
                Phase::LoadCore
                | Phase::LoadInbox
                | Phase::RestoreConnector { .. }
                | Phase::Link
                | Phase::Sweep
                | Phase::Switch
                | Phase::Wires
                | Phase::Open,
            )) => {
                drive(core, env, action, out);
            }
        }
    } else {
        append_started(&action, requests, out);
        drive(core, env, action, out);
    }
}

fn report(out: &mut Queue<Out>, report: application::Out) {
    out.push(Out::Now(Peer::Application(report)));
}

fn done(core: &mut Core, env: &Env<Limits>, phase: Phase, out: &mut Queue<Out>) -> Action {
    let mut requests = Queue::uncapped();
    let action = core.start_done(env, phase, &mut requests);
    append_started(&action, requests, out);
    action
}

fn append_started(action: &Action, requests: Queue<Request>, out: &mut Queue<Out>) {
    match action {
        Action::Refused(_) => {}
        Action::Step(_) | Action::Idle | Action::Commit(_) => engine::append(requests, out),
    }
}

fn finish(core: &mut Core, env: &Env<Limits>, phase: Phase, out: &mut Queue<Out>) {
    let action = done(core, env, phase, out);
    drive(core, env, action, out);
}

fn drive(core: &mut Core, env: &Env<Limits>, mut action: Action, out: &mut Queue<Out>) {
    core.start.port.wait = Wait::Ready;
    // Only AdoptRuns and Open can complete two consecutive phases without waiting.
    for _ in 0_u8..3 {
        let phase = match action {
            Action::Step(phase) => phase,
            Action::Commit(continuation) => {
                core.start.port.wait = Wait::Resume;
                out.push(Out::Held(Peer::Resume(Resume(Token::new(continuation.sequence)))));
                return;
            }
            Action::Refused(failure) => {
                refused(core, failure, out);
                return;
            }
            Action::Idle => {
                if core.start_opened() {
                    report(out, application::Out::Start { phase: None });
                }
                return;
            }
        };
        report(out, application::Out::Start { phase: Some(phase) });
        match phase {
            Phase::LoadCore | Phase::LoadInbox | Phase::Sweep => {
                begin_walk(core, env, phase, out);
                return;
            }
            Phase::RestoreConnector { connector } => {
                let connector = Connector(connector);
                core.start.port.wait = Wait::Restored(connector);
                out.push(Out::Ask(connector, Ask::Lifecycle(lifecycle::Ask::Restore)));
                return;
            }
            Phase::Switch => {
                core.start.port.wait = Wait::Switched;
                out.push(Out::Held(Peer::Store(store::Out::Switch)));
                return;
            }
            Phase::Wires => {
                action = done(core, env, phase, out);
                match &action {
                    Action::Commit(_) => out.push(Out::Write(store::Write::Wires)),
                    Action::Step(_) | Action::Refused(_) | Action::Idle => {}
                }
            }
            Phase::Link | Phase::AdoptRuns | Phase::Open => action = done(core, env, phase, out),
        }
    }
    unreachable!("start reaches a port or opens after its immediate phases");
}

fn refused(core: &mut Core, failure: Failure, out: &mut Queue<Out>) {
    let phase = core.start_phase().expect("a refusal names the phase it stopped in");
    halt(core);
    report(out, application::Out::Refused { phase, failure });
}

fn begin_walk(core: &mut Core, env: &Env<Limits>, phase: Phase, out: &mut Queue<Out>) {
    let ranges: Box<[RestoreRange]> = match phase {
        Phase::LoadCore => Box::new(RestoreRange::LIVE),
        Phase::LoadInbox => Box::new([RestoreRange::Census(core.census(&env.limits))]),
        Phase::Sweep => Box::new([RestoreRange::People]),
        Phase::RestoreConnector { .. }
        | Phase::Link
        | Phase::Switch
        | Phase::Wires
        | Phase::AdoptRuns
        | Phase::Open => {
            unreachable!("only the three store walks load start pages")
        }
    };
    if let Some(failure) = core.start_failure() {
        refused(core, failure, out);
        return;
    }
    core.start.port.ranges = Some(ranges);
    load(core, None, out);
}

fn load(core: &mut Core, after: Option<store::Cursor>, out: &mut Queue<Out>) {
    let ranges = core.start.port.ranges.as_ref().expect("a page has its walk's ranges").clone();
    let load = core.loads.name();
    core.start.port.wait = Wait::Load(load);
    out.push(Out::Held(Peer::Store(store::Out::Load { load, range: store::Range::Start { ranges, after } })));
}

fn page(
    core: &mut Core,
    env: &Env<Limits>,
    items: Box<[store::Item]>,
    next: Option<store::Cursor>,
    out: &mut Queue<Out>,
) {
    let phase = core.start_phase().expect("a start load belongs to a phase");
    match phase {
        Phase::LoadCore | Phase::LoadInbox => {
            for item in items {
                match item {
                    store::Item::Record(record) => match core.restore(env, record) {
                        super::Restore::Kept => {}
                        super::Restore::Deployment { commits } => report(out, application::Out::Numbered { commits }),
                        super::Restore::Refused => {
                            core.start_collect_failure();
                            let failure = core.start_failure().expect("restoration names what cannot fit");
                            drive(core, env, Action::Refused(failure), out);
                            return;
                        }
                        super::Restore::Swept => unreachable!("live and inbox rows do not enter the sweep"),
                    },
                    store::Item::Missing { record, target } => core.start_missing(record, target),
                }
            }
            if let Some(next) = next {
                load(core, Some(next), out);
            } else {
                core.start.port.ranges = None;
                finish(core, env, phase, out);
            }
        }
        Phase::Sweep => {
            let mut queued = Queue::uncapped();
            for item in items {
                queued.push(item);
            }
            core.start.port.sweep = Some(SweepPage { items: queued, next });
            sweep(core, env, out);
        }
        Phase::RestoreConnector { .. }
        | Phase::Link
        | Phase::Switch
        | Phase::Wires
        | Phase::AdoptRuns
        | Phase::Open => {
            unreachable!("only the three store walks load start pages")
        }
    }
}

fn sweep(core: &mut Core, env: &Env<Limits>, out: &mut Queue<Out>) {
    let mut page = core.start.port.sweep.take().expect("a sweep continues from its last page");
    for _ in 0..page.items.len() {
        let item = page.items.pop().expect("sweep page item count");
        // Missing joins belong to the inbox census, and the sweep walks only people rows.
        let row = match item {
            store::Item::Record(Record::People(row)) => row,
            store::Item::Record(Record::Core(_) | Record::Tasks(_) | Record::Accounts(_) | Record::Notes(_))
            | store::Item::Missing { .. } => {
                halt(core);
                return;
            }
        };
        core.start.sweep.push(row);
        let mut requests = Queue::uncapped();
        let action = core.start_sweep(env, SweepInput::Next, &mut requests);
        append_started(&action, requests, out);
        match action {
            Action::Idle => {}
            Action::Commit(_) => {
                core.start.port.sweep = Some(page);
                drive(core, env, action, out);
                return;
            }
            Action::Refused(_) => {
                drive(core, env, action, out);
                return;
            }
            Action::Step(_) => unreachable!("a row cannot end the sweep"),
        }
    }
    if let Some(next) = page.next {
        load(core, Some(next), out);
    } else {
        core.start.port.ranges = None;
        let mut requests = Queue::uncapped();
        let action = core.start_sweep(env, SweepInput::Finished, &mut requests);
        append_started(&action, requests, out);
        drive(core, env, action, out);
    }
}
