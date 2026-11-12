//! A change's way to landing, using the browser's panel (client/ctl/tools.md, 4).
use crate::{
    jig::{
        self,
        call::{Value, number, value},
        digest::Digest,
        render::{self, Text},
        runner::{Called, Out},
        table::{
            Keyed, Kind,
            Occurs::{Optional, Required},
            Tool, flag, positional,
        },
    },
    root::Limits,
};
use alloc::{boxed::Box, collections::BTreeMap};
use jig_client::{Client, Event, Handle, Latest, Readable, Readout, State, Told, Watchable, Why, contract as c};
use skein_lib::{Env, Queue, Time};
use temper_client_domain::{
    forge::{self, change as words},
    owner::Owner,
};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub const READ: Tool = Tool {
    name: b"change show",
    line: b"Read a change's way to landing: its pull request, CI and step",
    prints: b"`change N`, its repository, branch and landing branch, its pull request, step and CI at its head; its questions: the route, those taken of its bound, and each open question with where it stands; and a digest",
    result: b"change",
    forms: &[&[
        positional(b"task", b"TASK", Kind::NumberRange { min: 1, max: u64::MAX }, Required, b"The change task's number"),
        flag(b"project", b"PROJECT", Kind::Project { empty: false }, Optional, b"Its project; the task's when absent"),
        flag(b"since", b"DIGEST", Kind::Digest, Optional, b"Wait until it differs from the read that printed DIGEST"),
        flag(b"until", b"CONDITION", Kind::Word { set: &[b"pull", b"ci:pending", b"ci:passed", b"ci:failed", b"held", b"landed", b"changed", b"cleared"] }, Optional, b"Wait until it holds: pull, ci:pending, ci:passed, ci:failed, held, landed, cleared, or changed"),
        flag(b"timeout", b"DURATION", Kind::Duration, Optional, b"How long a wait lasts; 300 seconds when absent, at most 3600"),
    ]],
    keyed: Keyed::Never,
    area: 6,
    notes: b"While landing-rule approval is outstanding, it prints each requirement,\ncounted people and review heads, the remaining count and any unmapped\nreviews, then points to the pull request for review. Unread and incomplete\nevidence are named separately from zero qualifying approvals.\nOtherwise, until it lands, it prints `next: temper ctl DIR change show TASK\n--until landed`, or, while a task is held, `next: temper ctl DIR\ntask show N`; while verifying or red, `next: temper ctl DIR change show TASK\n--until cleared`; landed, its merge commit. Successful no-change completion\nprints `result: completed, no change` and `next: temper ctl DIR task show N`\nfor its report. An unmet wait ends with `not met: ended`,\n`not met: ended: cancelled` or `not met: ended: failed` (exit 3).\nA wait for landing ends at a hold, `not met: held` (exit 3).\nA task without a panel prints `no change panel for task N` (exit 2).\nThe forge holds its diff, reviews and logs: read them there.\n",
};
#[derive(Debug)]
pub(crate) struct Changes {
    calls: BTreeMap<u32, Watching>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TaskState {
    Unknown,
    Known,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PanelState {
    Expected,
    Missing,
}
#[derive(Debug)]
enum Reading {
    Idle,
    Pending(Handle),
    Retry(Time),
}
impl Reading {
    fn active(&self) -> bool {
        match self {
            Self::Idle => false,
            Self::Pending(_) | Self::Retry(_) => true,
        }
    }
    fn handle(&self) -> Option<Handle> {
        match self {
            Self::Pending(handle) => Some(*handle),
            Self::Idle | Self::Retry(_) => None,
        }
    }
}
#[derive(Debug)]
struct Watching {
    called: Called,
    task: u64,
    project: Option<u32>,
    waiting: bool,
    since: Option<Digest>,
    first: Option<Digest>,
    deadline: Option<Time>,
    panel: Option<Handle>,
    read: Reading,
    task_watch: Option<Handle>,
    activity: Option<u64>,
    final_panel: bool,
    ended: Option<&'static [u8]>,
    panel_state: PanelState,
    run_read: Option<u64>,
    run_watch: Option<(Handle, u64)>,
    holds: Holds,
    task_state: TaskState,
    last: Option<Box<w::ChangePanel>>,
    serial: u32,
    started: bool,
}
/// Task completion and recovery details rendered beside the forge's panel.
#[derive(Debug)]
pub(crate) struct Holds {
    own: Option<Held>,
    completion: Completion,
    run: Option<Box<[u8]>>,
}
#[derive(Debug)]
struct Held {
    reason: Box<[u8]>,
    way_out: Box<[u8]>,
}
impl Holds {
    pub(crate) fn new() -> Self {
        Self { own: None, run: None, completion: Completion::Unknown }
    }
    fn task(&mut self, task: &c::Task) {
        self.own = if task.chip.phase == c::Phase::Held && task.hold.is_some() {
            let mut measure = Text::measure();
            render::holds::way_out(task, &crate::root::Parts, &mut measure);
            let mut out = Text::writing_exact(measure);
            render::holds::way_out(task, &crate::root::Parts, &mut out);
            Some(Held { reason: render::holds::summary(task, &crate::root::Parts), way_out: out.finish() })
        } else {
            None
        };
    }
    fn any(&self, panel: &w::ChangePanel) -> bool {
        self.own.is_some() || self.run.is_some() || panel.hold().is_some()
    }
}
impl Changes {
    pub(crate) fn new() -> Self {
        Self { calls: BTreeMap::new() }
    }
    pub(crate) fn calls(&self) -> usize {
        self.calls.len()
    }
    pub(crate) fn reading(&self) -> bool {
        for watch in self.calls.values() {
            if watch.read.active() {
                return true;
            }
        }
        false
    }
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        let mut next = None;
        for call in self.calls.values() {
            for deadline in [
                call.deadline,
                match call.read {
                    Reading::Retry(at) => Some(at),
                    Reading::Idle | Reading::Pending(_) => None,
                },
            ]
            .into_iter()
            .flatten()
            {
                next = Some(match next {
                    Some(old) => deadline.min(old),
                    None => deadline,
                });
            }
        }
        next
    }
}
pub(crate) fn owns(call: &jig::call::Call) -> bool {
    let name = crate::table::TOOLS.get(usize::from(call.tool)).expect("parsed tool").name;
    name == READ.name
}
pub(crate) fn call(state: &mut Changes, client: &Client, env: &Env<Limits>, called: Called, out: &mut Queue<Out>) {
    if called.value.is_some() || jig::argv::check(crate::table::TOOLS, &called.call, &env.limits.tools).is_err() {
        let printed = render::failure(b"invalid change call", called.json, 1);
        out.push(Out::Printed { number: called.number, printed });
        return;
    }
    let waiting = jig::call::waiting(&READ, &called.call);
    let task = number(&called.call, 0).expect("checked task");
    let since = match value(&called.call, 2) {
        Some(Value::Digest(bytes)) if waiting => Some(Digest::from_bytes(*bytes)),
        _ => None,
    };
    let deadline = match value(&called.call, 4) {
        Some(Value::Duration(duration)) if waiting => {
            Some(env.now.checked_add(*duration).unwrap_or(Time::from_nanos(u64::MAX)))
        }
        _ => None,
    };
    state.calls.insert(
        called.call_number,
        Watching {
            called,
            task,
            project: None,
            waiting,
            since,
            first: None,
            deadline,
            panel: None,
            read: Reading::Idle,
            task_watch: None,
            activity: None,
            final_panel: false,
            ended: None,
            panel_state: PanelState::Expected,
            run_read: None,
            run_watch: None,
            holds: Holds::new(),
            task_state: TaskState::Unknown,
            last: None,
            serial: 16,
            started: false,
        },
    );
    ready(state, client, env, out);
}
fn handle(watch: &mut Watching) -> Handle {
    watch.serial = watch.serial.checked_add(1).expect("bounded wait life");
    Handle {
        asker: Owner::Forge.asker(),
        number: (u64::from(watch.called.call_number) << 32u32) | u64::from(watch.serial),
    }
}
pub(crate) fn ready(state: &mut Changes, client: &Client, env: &Env<Limits>, out: &mut Queue<Out>) {
    let Some(me) = crate::forge::me(client) else { return };
    for _ in 0..state.calls.len() {
        let mut next = None;
        for (&number, watch) in &state.calls {
            if !watch.started {
                next = Some(number);
                break;
            }
        }
        let Some(number) = next else { break };
        let watch = state.calls.get_mut(&number).expect("active change");
        watch.started = true;
        let project = value(&watch.called.call, 1);
        watch.project = match project {
            Some(Value::Project(jig::call::Project::Number(n))) => Some(*n),
            Some(Value::Project(jig::call::Project::Name(name))) => {
                let mut found = None;
                for project in &me.projects {
                    if project.name.bytes() == name.as_ref() {
                        found = Some(project.project);
                        break;
                    }
                }
                found
            }
            _ => None,
        };
        if project.is_some() && watch.project.is_none() {
            let json = watch.called.json;
            complete(state, number, render::failure(b"unknown project", json, 1), out);
            continue;
        }
        if watch.project.is_some() {
            open_panel(watch, env, out);
        }
        read_task(watch, out);
    }
}

fn open_panel(watch: &mut Watching, env: &Env<Limits>, out: &mut Queue<Out>) {
    if watch.waiting && watch.task_watch.is_none() {
        watch_task(watch, out);
    }
    let handle = handle(watch);
    watch.panel = Some(handle);
    out.push(Out::Client {
        event: Event::Watch { handle, subject: subject(watch.task, watch.project.expect("task project"), &env.limits) },
    });
}
pub(crate) fn subject(task: u64, project: u32, limits: &Limits) -> Watchable {
    let limits = forge::wire_limits(&limits.domain.forge).expect("checked forge limits");
    let change = w::ChangeSubject::new(&limits, w::ChangeSubjectParts { task }).expect("task fits");
    let record = w::Watching::new(&limits, w::WatchingParts { watch: w::Watch::Change(change) }).expect("watch fits");
    Watchable::Family {
        scope: c::Scope::Project(project),
        window: c::Window { rows: 0, after: None },
        record: jig_client::Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: wire::watching(&record),
        },
    }
}
fn read_task(watch: &mut Watching, out: &mut Queue<Out>) {
    watch.run_read = None;
    let handle = handle(watch);
    watch.read = Reading::Pending(handle);
    out.push(Out::Client { event: Event::Read { handle, read: Readable::Jig(c::Read::Task { task: watch.task }) } });
}
fn read_run(watch: &mut Watching, run: u64, out: &mut Queue<Out>) {
    if watch.read.active() {
        return;
    }
    let handle = handle(watch);
    watch.read = Reading::Pending(handle);
    watch.run_read = Some(run);
    out.push(Out::Client { event: Event::Read { handle, read: Readable::Jig(c::Read::Task { task: run }) } });
}
fn close(watch: &Watching, out: &mut Queue<Out>) {
    for handle in [watch.panel, watch.task_watch, run_handle(watch)].into_iter().flatten() {
        out.push(Out::Client { event: Event::Unwatch { handle } });
    }
}
fn complete(state: &mut Changes, number: u32, printed: jig::Printed, out: &mut Queue<Out>) {
    let watch = state.calls.remove(&number).expect("active change");
    close(&watch, out);
    out.push(Out::Printed { number: watch.called.number, printed });
}
fn missing(task: u64, json: bool) -> jig::Printed {
    let mut measure = Text::measure();
    measure.put(b"no change panel for task ");
    measure.number(task);
    let mut out = Text::writing_exact(measure);
    out.put(b"no change panel for task ");
    out.number(task);
    render::failure(&out.finish(), json, 2)
}
fn panel_ended(watch: &mut Watching, out: &mut Queue<Out>) {
    watch.panel_state = PanelState::Missing;
    if let Some(handle) = watch.panel.take() {
        out.push(Out::Client { event: Event::Unwatch { handle } });
    }
    if !watch.read.active() || watch.run_read.is_some() {
        read_task(watch, out);
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Completion {
    Unknown,
    Change,
    NoChange,
}
fn completion(task: &c::Task, result: Option<&c::Result>) -> Completion {
    if !terminal(task) || task.chip.phase == c::Phase::Cancelled || task.chip.phase == c::Phase::Failed {
        return Completion::Unknown;
    }
    let result = match result {
        Some(result) => Some(result),
        None => match &task.result {
            Some(c::TaskResult::Document(result)) => Some(result.as_ref()),
            Some(c::TaskResult::Ending(ending)) => return chip_completion(ending),
            None => None,
        },
    };
    let Some(result) = result else { return Completion::Unknown };
    match &result.ending {
        c::Ending::Task(ending) => chip_completion(ending),
        c::Ending::Done => match &result.value {
            Some(value) => result_completion(value),
            None => Completion::Unknown,
        },
        c::Ending::Failed { .. }
        | c::Ending::Cancelled { .. }
        | c::Ending::Withdrawn { .. }
        | c::Ending::Decided { .. }
        | c::Ending::Deleted => Completion::Unknown,
    }
}
fn chip_completion(ending: &c::ChipEnding) -> Completion {
    match ending {
        c::ChipEnding::Done(value) => result_completion(value),
        c::ChipEnding::Failed(_) | c::ChipEnding::Cancelled { .. } => Completion::Unknown,
    }
}
fn result_completion(value: &c::ResultValue) -> Completion {
    match value {
        c::ResultValue::Change { part, .. } if part.connector == wire::FORGE => Completion::Change,
        c::ResultValue::Report(_) => Completion::NoChange,
        c::ResultValue::Change { .. }
        | c::ResultValue::OwnerVerdict { .. }
        | c::ResultValue::Verdict { .. }
        | c::ResultValue::Connector { .. }
        | c::ResultValue::Choice { .. }
        | c::ResultValue::Approved { .. }
        | c::ResultValue::Failure(_)
        | c::ResultValue::Words(_) => Completion::Unknown,
    }
}
fn terminal(task: &c::Task) -> bool {
    match task.chip.phase {
        c::Phase::Ended | c::Phase::Done | c::Phase::Failed | c::Phase::Cancelled => true,
        c::Phase::Waiting
        | c::Phase::Idle
        | c::Phase::Due
        | c::Phase::Preparing
        | c::Phase::Claimed
        | c::Phase::Running
        | c::Phase::Active
        | c::Phase::BackingOff
        | c::Phase::Closing
        | c::Phase::Held => false,
    }
}
fn task_read(
    watch: &mut Watching,
    page: &Readout,
    client: &Client,
    env: &Env<Limits>,
    out: &mut Queue<Out>,
) -> Option<jig::Printed> {
    let Readout::Jig(c::ReadResult::Value(value)) = page else {
        return Some(render::read(
            page,
            client,
            Handle { asker: crate::root::RUNNER.asker(), number: 0 },
            watch.called.json,
            &env.limits.tools,
            &crate::root::Parts,
        ));
    };
    let Some((task, result)) = render::read_task(value) else {
        return Some(render::failure(b"unexpected task document", watch.called.json, 1));
    };
    if task.chip.task != watch.task {
        return Some(render::failure(b"unexpected task document", watch.called.json, 1));
    }
    watch.task_state = TaskState::Known;
    watch.holds.task(task);
    watch.holds.completion = completion(task, result);
    if let Some(printed) = unregistered(watch, task) {
        return Some(printed);
    }
    // The first task read supplies the project; a read remains a read once.
    if watch.project.is_none() {
        watch.project = Some(task.project);
        open_panel(watch, env, out);
    }
    if watch.waiting {
        if terminal(task) {
            if let Some(printed) = task_ended(watch, task, result, env, out) {
                return Some(printed);
            }
        } else {
            watch.activity = Some(task.activity);
            if watch.task_watch.is_none() {
                watch_task(watch, out);
            }
        }
    }
    continue_panel(watch, env, out)
}
fn continue_panel(watch: &mut Watching, env: &Env<Limits>, out: &mut Queue<Out>) -> Option<jig::Printed> {
    if watch.final_panel {
        return None;
    }
    if let Some(panel) = watch.last.as_deref() {
        if let Some(run) = panel.runs() {
            read_run(watch, *run, out);
        } else {
            return delivered(watch, env);
        }
    }
    None
}
fn unregistered(watch: &Watching, task: &c::Task) -> Option<jig::Printed> {
    if watch.panel_state == PanelState::Missing {
        if task.chip.phase == c::Phase::Held
            && let c::TaskContractDocument::Change { connector: wire::FORGE, kind: 1, .. } = task.result_contract
        {
            let reason = render::holds::summary(task, &crate::root::Parts);
            let mut measure = Text::measure();
            measure.put(b"held before registration: ");
            measure.put(&reason);
            let mut out = Text::writing_exact(measure);
            out.put(b"held before registration: ");
            out.put(&reason);
            return Some(render::failure(&out.finish(), watch.called.json, 2));
        }
        if !watch.waiting {
            return Some(missing(watch.task, watch.called.json));
        }
    }
    None
}
fn task_ended(
    watch: &mut Watching,
    task: &c::Task,
    result: Option<&c::Result>,
    env: &Env<Limits>,
    out: &mut Queue<Out>,
) -> Option<jig::Printed> {
    if watch.holds.completion != Completion::Change {
        let why = ending(task, result);
        watch.ended = Some(why);
        if watch.last.is_some() {
            return Some(print(watch.last.as_deref(), watch.called.json, Some(false), why, &env.limits, &watch.holds));
        }
        if watch.panel.is_none() {
            return Some(missing(watch.task, watch.called.json));
        }
    } else if watch.final_panel {
        return if watch.last.is_some() { delivered(watch, env) } else { Some(missing(watch.task, watch.called.json)) };
    } else {
        watch.final_panel = true;
        if watch.panel.is_none() {
            open_panel(watch, env, out);
        }
    }
    None
}
fn watch_task(watch: &mut Watching, out: &mut Queue<Out>) {
    let handle = handle(watch);
    watch.task_watch = Some(handle);
    out.push(Out::Client {
        event: Event::Watch { handle, subject: Watchable::Jig(c::Subject::Task { task: watch.task }) },
    });
}
fn ending(task: &c::Task, result: Option<&c::Result>) -> &'static [u8] {
    let result = result.or(match &task.result {
        Some(c::TaskResult::Document(result)) => Some(result.as_ref()),
        _ => None,
    });
    let phase = if task.chip.phase == c::Phase::Cancelled {
        b"ended: cancelled".as_slice()
    } else if task.chip.phase == c::Phase::Failed {
        b"ended: failed".as_slice()
    } else {
        b"ended".as_slice()
    };
    let Some(result) = result else { return phase };
    match &result.ending {
        c::Ending::Cancelled { .. } => b"ended: cancelled",
        c::Ending::Failed { .. } => b"ended: failed",
        c::Ending::Task(ending) => match ending.as_ref() {
            c::ChipEnding::Cancelled { .. } => b"ended: cancelled",
            c::ChipEnding::Failed(_) => b"ended: failed",
            c::ChipEnding::Done(_) => b"ended",
        },
        c::Ending::Done | c::Ending::Withdrawn { .. } | c::Ending::Decided { .. } | c::Ending::Deleted => phase,
    }
}

fn read_page(
    watch: &mut Watching,
    page: &Readout,
    client: &Client,
    env: &Env<Limits>,
    out: &mut Queue<Out>,
) -> Option<jig::Printed> {
    let Some(run) = watch.run_read.take() else { return task_read(watch, page, client, env, out) };
    let Readout::Jig(c::ReadResult::Value(value)) = page else {
        return Some(render::read(
            page,
            client,
            Handle { asker: crate::root::RUNNER.asker(), number: 0 },
            watch.called.json,
            &env.limits.tools,
            &crate::root::Parts,
        ));
    };
    let Some((task, _)) = render::read_task(value) else {
        return Some(render::failure(b"unexpected task document", watch.called.json, 1));
    };
    if task.chip.task != run {
        return Some(render::failure(b"unexpected task document", watch.called.json, 1));
    }
    let latest = match &watch.last {
        Some(panel) => *panel.runs(),
        None => None,
    };
    if let Some(latest) = latest
        && latest != run
    {
        read_run(watch, latest, out);
        return None;
    }
    watch.holds.run = if latest.is_some() && task.chip.phase == c::Phase::Held {
        Some(render::holds::summary(task, &crate::root::Parts))
    } else {
        None
    };
    if watch.waiting && latest.is_some() && !terminal(task) && watch.run_watch.is_none() {
        let handle = handle(watch);
        watch.run_watch = Some((handle, run));
        out.push(Out::Client {
            event: Event::Watch { handle, subject: Watchable::Jig(c::Subject::Task { task: run }) },
        });
    }
    delivered(watch, env)
}
fn panel_changed(
    watch: &mut Watching,
    client: &Client,
    env: &Env<Limits>,
    handle: Handle,
    out: &mut Queue<Out>,
) -> Option<jig::Printed> {
    let doc = client.document(handle)?;
    if doc.state == State::Opening || doc.state == State::Behind {
        return None;
    }
    let panel = match doc.latest {
        Some(Latest::Family(record)) if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION => {
            wire::decode_change_panel(
                &record.bytes,
                &forge::wire_limits(&env.limits.domain.forge).expect("checked limits"),
            )
        }
        _ => None,
    };
    match panel {
        Some(panel) if panel.task() == watch.task => {
            watch.last = Some(Box::new(panel));
            watch.panel_state = PanelState::Expected;
            watch.holds.run = None;
            if let Some((_, task)) = watch.run_watch
                && Some(task) != *watch.last.as_ref().expect("delivered panel").runs()
            {
                let (handle, _) = watch.run_watch.take().expect("old run watch");
                out.push(Out::Client { event: Event::Unwatch { handle } });
            }
            if let Some(run) = watch.last.as_ref().expect("delivered panel").runs() {
                let run = *run;
                read_run(watch, run, out);
            } else {
                return delivered(watch, env);
            }
        }
        _ => panel_ended(watch, out),
    }
    None
}
fn run_handle(watch: &Watching) -> Option<Handle> {
    let (handle, _) = watch.run_watch?;
    Some(handle)
}
fn watch_failure(why: Why, json: bool) -> jig::Printed {
    let (reason, status): (&[u8], u8) = match why {
        Why::NotAllowed => (b"not_allowed", 2),
        Why::SignedOut => (b"signed out", 1),
        Why::NotServed => (b"version not served", 1),
        Why::Limit => (b"watch table limit", 1),
        Why::Unknown | Why::SubjectEnded => unreachable!("handled"),
    };
    render::failure(reason, json, status)
}
fn read_answer(
    watch: &mut Watching,
    page: &Readout,
    client: &Client,
    env: &Env<Limits>,
    out: &mut Queue<Out>,
) -> Option<jig::Printed> {
    watch.read = Reading::Idle;
    if watch.waiting && *page == Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)) {
        // A read cut by a restart (including a not-ready HTTP answer)
        // keeps its target. The host owns the link deadline and its line.
        watch.read = Reading::Retry(env.now.saturating_add(env.limits.domain.client.domain.backoff.first));
        None
    } else {
        read_page(watch, page, client, env, out)
    }
}
pub(crate) fn told(state: &mut Changes, client: &Client, env: &Env<Limits>, told: &Told, out: &mut Queue<Out>) {
    let handle = match told {
        Told::Changed { handle } | Told::Ended { handle, .. } | Told::Page { handle, .. } => *handle,
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Text { .. } => return,
    };
    let number = u32::try_from(handle.number >> 32u32).expect("high half");
    let Some(watch) = state.calls.get_mut(&number) else { return };
    let mut printed = None;
    match told {
        Told::Page { page, .. } if watch.read.handle() == Some(handle) => {
            printed = read_answer(watch, page, client, env, out);
        }
        Told::Changed { .. } if watch.panel == Some(handle) => {
            printed = panel_changed(watch, client, env, handle, out);
        }
        Told::Changed { .. } if run_handle(watch) == Some(handle) => {
            let Some(doc) = client.document(handle) else { return };
            if doc.state == State::Opening || doc.state == State::Behind {
                return;
            }
            if let Some(Latest::Jig(c::Delivery::Task(task))) = doc.latest {
                let (_, run) = watch.run_watch.expect("run watch");
                if task.chip.task == run {
                    watch.holds.run = if task.chip.phase == c::Phase::Held {
                        Some(render::holds::summary(task, &crate::root::Parts))
                    } else {
                        None
                    };
                    printed = delivered(watch, env);
                } else {
                    printed = Some(render::failure(b"unexpected task document", watch.called.json, 1));
                }
            }
        }
        Told::Ended { why, .. } if run_handle(watch) == Some(handle) => {
            let (_, run) = watch.run_watch.take().expect("run watch");
            if *why == Why::Unknown || *why == Why::SubjectEnded {
                read_run(watch, run, out);
            } else {
                printed = Some(watch_failure(*why, watch.called.json));
            }
        }
        Told::Changed { .. } if watch.task_watch == Some(handle) => {
            let Some(doc) = client.document(handle) else { return };
            if doc.state == State::Opening || doc.state == State::Behind {
                return;
            }
            if let Some(Latest::Jig(c::Delivery::Task(task))) = doc.latest {
                if task.chip.task != watch.task {
                    printed = Some(render::failure(b"unexpected task document", watch.called.json, 1));
                } else if terminal(task) {
                    if let Some(handle) = watch.task_watch.take() {
                        out.push(Out::Client { event: Event::Unwatch { handle } });
                    }
                    read_task(watch, out);
                } else {
                    watch.task_state = TaskState::Known;
                    watch.holds.task(task);
                    if watch.last.is_some() {
                        printed = delivered(watch, env);
                    }
                    if printed.is_none() && watch.panel.is_none() && watch.activity != Some(task.activity) {
                        watch.activity = Some(task.activity);
                        if let Some(handle) = watch.task_watch.take() {
                            out.push(Out::Client { event: Event::Unwatch { handle } });
                        }
                        open_panel(watch, env, out);
                    }
                }
            }
        }
        Told::Ended { why, .. } if watch.panel == Some(handle) || watch.task_watch == Some(handle) => {
            if *why == Why::Unknown || *why == Why::SubjectEnded {
                if watch.panel == Some(handle) {
                    panel_ended(watch, out);
                } else {
                    watch.task_watch = None;
                    read_task(watch, out);
                }
            } else {
                printed = Some(watch_failure(*why, watch.called.json));
            }
        }
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => {}
    }
    if let Some(printed) = printed {
        complete(state, number, printed, out);
    }
}
fn delivered(watch: &mut Watching, env: &Env<Limits>) -> Option<jig::Printed> {
    if watch.task_state == TaskState::Unknown || (watch.read.active() && watch.run_read.is_none()) {
        return None;
    }
    let panel = watch.last.as_deref().expect("delivered panel");
    if !watch.waiting {
        return Some(print(Some(panel), watch.called.json, None, b"", &env.limits, &watch.holds));
    }
    let Some(digest) = digest(panel, &env.limits, &watch.holds) else {
        return Some(render::failure(b"result limit", watch.called.json, 1));
    };
    let condition = match value(&watch.called.call, 3) {
        Some(Value::Word(word)) => {
            let Kind::Word { set } = READ.forms.first().expect("read form").get(3).expect("condition argument").kind
            else {
                unreachable!("condition words")
            };
            Some(*set.get(usize::from(*word)).expect("checked condition"))
        }
        _ => None,
    };
    let holds = match condition {
        Some(b"changed") => Some(watch.first.is_some() && jig::wait::met(watch.since, watch.first, digest, None)),
        Some(b"pull") => Some(panel.pull().is_some()),
        Some(b"ci:pending") => Some(*panel.ci() == w::Ci::Pending),
        Some(b"ci:passed") => Some(*panel.ci() == w::Ci::Passed),
        Some(b"ci:failed") => Some(*panel.ci() == w::Ci::Failed),
        Some(b"held") => Some(watch.holds.any(panel)),
        Some(b"landed") => Some(landed(panel.step()).is_some()),
        Some(b"cleared") => Some(progress(panel.step()) == Progress::Cleared),
        None => None,
        _ => unreachable!("checked condition"),
    };
    if let Some(why) = watch.ended {
        return Some(print(Some(panel), watch.called.json, Some(false), why, &env.limits, &watch.holds));
    }
    let met = jig::wait::met(watch.since, watch.first, digest, holds);
    watch.first = Some(watch.first.unwrap_or(digest));
    if !met && (condition == Some(b"landed") || condition == Some(b"cleared")) && watch.holds.any(panel) {
        return Some(print(Some(panel), watch.called.json, Some(false), b"held", &env.limits, &watch.holds));
    }
    if met
        || watch.final_panel
        || (progress(panel.step()) == Progress::Ended || progress(panel.step()) == Progress::Cleared)
    {
        Some(print(Some(panel), watch.called.json, Some(met), b"ended", &env.limits, &watch.holds))
    } else {
        None
    }
}
pub(crate) fn fire(state: &mut Changes, env: &Env<Limits>, out: &mut Queue<Out>) {
    for _ in 0..state.calls.len() {
        let mut due = None;
        for (&number, watch) in &state.calls {
            if let Some(deadline) = watch.deadline
                && deadline <= env.now
            {
                due = Some(number);
                break;
            }
        }
        let Some(number) = due else { break };
        let watch = state.calls.get(&number).expect("active change");
        let printed =
            print(watch.last.as_deref(), watch.called.json, Some(false), b"timeout", &env.limits, &watch.holds);
        complete(state, number, printed, out);
    }
    for watch in state.calls.values_mut() {
        if let Reading::Retry(at) = watch.read
            && at <= env.now
        {
            let run = watch.run_read;
            watch.read = Reading::Idle;
            match run {
                Some(run) => read_run(watch, run, out),
                None => read_task(watch, out),
            }
        }
    }
}

use skein_json::writer::Encoder;
fn landed(step: &w::Step) -> Option<&[u8; 32]> {
    match step {
        w::Step::Landed(landed) => Some(landed.merge()),
        w::Step::Verifying(value) => Some(value.merge()),
        w::Step::Red(value) => Some(value.merge()),
        w::Step::Cleared(value) => Some(value.merge()),
        w::Step::Ready(_)
        | w::Step::Preparing(_)
        | w::Step::Producing
        | w::Step::Opening
        | w::Step::Recreating
        | w::Step::Reopening
        | w::Step::Checking
        | w::Step::Gating
        | w::Step::Queued(_)
        | w::Step::First
        | w::Step::Updating
        | w::Step::Resolving
        | w::Step::Repairing(_)
        | w::Step::Landing
        | w::Step::Ended => None,
    }
}
fn step(step: &w::Step) -> &'static [u8] {
    // The browser's held-step vocabulary is also the step's lower-case name.
    match step {
        w::Step::First => b"first",
        w::Step::Ended => b"ended",
        w::Step::Ready(_)
        | w::Step::Preparing(_)
        | w::Step::Producing
        | w::Step::Opening
        | w::Step::Recreating
        | w::Step::Reopening
        | w::Step::Checking
        | w::Step::Gating
        | w::Step::Queued(_)
        | w::Step::Updating
        | w::Step::Resolving
        | w::Step::Repairing(_)
        | w::Step::Landing
        | w::Step::Landed(_)
        | w::Step::Verifying(_)
        | w::Step::Red(_)
        | w::Step::Cleared(_) => phrase(words::held_step(step)),
    }
}
fn phrase(word: forge::words::ChangeWord) -> &'static [u8] {
    for (candidate, literals, _) in &forge::words::CHANGES {
        if *candidate == word {
            return literals.first().expect("phrase literal");
        }
    }
    unreachable!("closed change vocabulary")
}
fn ci(ci: &w::Ci) -> &'static [u8] {
    match ci {
        w::Ci::Unknown => b"unknown",
        w::Ci::Pending => b"pending",
        w::Ci::Passed => b"passed",
        w::Ci::Failed => b"failed",
    }
}
fn hold(hold: &w::Hold) -> &[u8] {
    match hold {
        w::Hold::Stalled => b"stalled",
        w::Hold::Repairs => b"repairs",
        w::Hold::Questions => b"questions",
        w::Hold::Resolutions => b"resolutions",
        w::Hold::Updates => b"updates",
        w::Hold::Moved => b"moved",
        w::Hold::Missing => b"missing",
        w::Hold::Closed => b"closed",
        w::Hold::Retargeted => b"retargeted",
        w::Hold::Failed => b"failed",
        w::Hold::Rejected => b"rejected",
        w::Hold::Authority(value) => value.reason(),
    }
}
fn repair(step: &w::Step) -> Option<&'static [u8]> {
    match step {
        w::Step::Repairing(repair) => Some(match repair.reason() {
            w::Reason::Ci => b"ci",
            w::Reason::Gate(_) => b"gate",
            w::Reason::Semantic => b"semantic",
            w::Reason::Review => b"review",
        }),
        w::Step::Ready(_)
        | w::Step::Preparing(_)
        | w::Step::Producing
        | w::Step::Opening
        | w::Step::Recreating
        | w::Step::Reopening
        | w::Step::Checking
        | w::Step::Gating
        | w::Step::Queued(_)
        | w::Step::First
        | w::Step::Updating
        | w::Step::Resolving
        | w::Step::Landing
        | w::Step::Landed(_)
        | w::Step::Verifying(_)
        | w::Step::Red(_)
        | w::Step::Cleared(_)
        | w::Step::Ended => None,
    }
}
fn gate_kind(kind: &w::GateKind) -> &'static [u8] {
    match kind {
        w::GateKind::Review => b"review",
        w::GateKind::Approval => b"approval",
        w::GateKind::Check => b"check",
    }
}
fn verdict(verdict: &w::Verdict) -> &'static [u8] {
    match verdict {
        w::Verdict::None => b"none",
        w::Verdict::Pending => b"pending",
        w::Verdict::Approved => b"approved",
        w::Verdict::Changes => b"changes",
    }
}
fn hex(bytes: &[u8; 32]) -> [u8; 40] {
    let mut digits = [0; 40];
    for (i, digit) in digits.iter_mut().enumerate() {
        let byte = *bytes.get(i / 2).expect("forty digits in 32 bytes");
        let nibble = if i % 2 == 0 { byte >> 4_u8 } else { byte & 15 };
        *digit = *b"0123456789abcdef".get(usize::from(nibble)).expect("hex nibble");
    }
    digits
}
fn used_json(used: &w::Used, out: &mut Encoder) {
    out.object_start();
    out.key(b"used");
    out.unsigned(u64::from(used.used()));
    out.key(b"limit");
    out.unsigned(u64::from(used.limit()));
    out.object_end();
}
fn encode(panel: &w::ChangePanel, digest: Option<Digest>, holds: &Holds, out: &mut Encoder) {
    out.object_start();
    out.key(b"task");
    out.unsigned(panel.task());
    out.key(b"forge");
    out.unsigned(u64::from(panel.forge()));
    for (key, value) in [
        (b"owner".as_slice(), panel.owner()),
        (b"name", panel.name()),
        (b"branch", panel.branch()),
        (b"landing", panel.landing()),
    ] {
        out.key(key);
        out.string(value);
    }
    out.key(b"head");
    match panel.head() {
        Some(head) => out.string(&hex(head)),
        None => out.null(),
    }
    out.key(b"pull");
    match panel.pull() {
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
    encode_progress(panel, out);
    out.key(b"ci_required");
    out.boolean(panel.ci_required());
    out.key(b"ci");
    out.string(ci(panel.ci()));
    encode_gates(panel, out);
    encode_approvals(panel, out);
    for (key, used) in
        [(b"repairs".as_slice(), panel.repairs()), (b"resolutions", panel.resolutions()), (b"updates", panel.updates())]
    {
        out.key(key);
        used_json(used, out);
    }
    encode_questions(panel.questions(), out);
    encode_holds(holds, out);
    if let Some(digest) = digest {
        out.key(b"digest");
        out.string(&digest.hex());
    }
    out.object_end();
}
fn encode_progress(panel: &w::ChangePanel, out: &mut Encoder) {
    out.key(b"step");
    out.string(step(panel.step()));
    out.key(b"merge");
    match landed(panel.step()) {
        Some(merge) => out.string(&hex(merge)),
        None => out.null(),
    }
    if progress(panel.step()) == Progress::Verifying || progress(panel.step()) == Progress::Cleared {
        out.key(b"tiers");
        out.array_start();
        for line in tier_lines(panel.step()) {
            out.string(line);
        }
        out.array_end();
    }
    out.key(b"queue");
    match words::queue(panel.step()) {
        Some(queue) => {
            out.object_start();
            out.key(b"place");
            out.unsigned(u64::from(queue.place));
            out.key(b"ahead");
            out.array_start();
            for task in queue.ahead {
                out.unsigned(*task);
            }
            out.array_end();
            out.object_end();
        }
        None => out.null(),
    }
    out.key(b"repair");
    match repair(panel.step()) {
        Some(reason) => out.string(reason),
        None => out.null(),
    }
    out.key(b"hold");
    match panel.hold() {
        Some(why) => out.string(hold(why)),
        None => out.null(),
    }
    out.key(b"runs");
    match panel.runs() {
        Some(task) => out.unsigned(*task),
        None => out.null(),
    }
}

fn encode_gates(panel: &w::ChangePanel, out: &mut Encoder) {
    out.key(b"gates");
    out.array_start();
    for gate in panel.gates() {
        out.object_start();
        out.key(b"number");
        out.unsigned(gate.number());
        out.key(b"kind");
        out.string(gate_kind(gate.kind()));
        out.key(b"blocks");
        out.boolean(gate.blocks());
        out.key(b"verdict");
        out.string(verdict(gate.verdict()));
        out.object_end();
    }
    out.array_end();
}
fn review_read(read: &w::ReviewRead) -> &'static [u8] {
    match read {
        w::ReviewRead::Unread => b"unread",
        w::ReviewRead::Incomplete => b"incomplete",
        w::ReviewRead::Complete => b"complete",
    }
}
fn freshness(freshness: &w::RuleFreshness) -> &'static [u8] {
    match freshness {
        w::RuleFreshness::Exact => b"exact",
        w::RuleFreshness::Clean => b"clean",
    }
}
fn encode_approvals(panel: &w::ChangePanel, out: &mut Encoder) {
    out.key(b"approvals");
    out.array_start();
    for line in panel.approvals() {
        out.object_start();
        for (key, value) in [
            (b"scope".as_slice(), line.scope()),
            (b"rule", line.number()),
            (b"role", line.role()),
            (b"required", line.required()),
        ] {
            out.key(key);
            out.unsigned(u64::from(value));
        }
        out.key(b"freshness");
        out.string(freshness(line.freshness()));
        out.key(b"read");
        out.string(review_read(line.read()));
        out.key(b"remaining");
        match line.remaining() {
            Some(count) => out.unsigned(u64::from(*count)),
            None => out.null(),
        }
        out.key(b"refused");
        out.boolean(line.refused());
        out.key(b"counted");
        out.array_start();
        for review in line.counted() {
            out.object_start();
            out.key(b"person");
            out.unsigned(review.person());
            out.key(b"head");
            out.string(&hex(review.head()));
            out.object_end();
        }
        out.array_end();
        out.object_end();
    }
    out.array_end();
    out.key(b"unmapped_reviews");
    out.array_start();
    for review in panel.unmapped_reviews() {
        out.object_start();
        out.key(b"account");
        out.unsigned(review.account());
        out.key(b"head");
        out.string(&hex(review.head()));
        out.key(b"verdict");
        out.string(verdict(review.verdict()));
        out.key(b"reason");
        out.string(b"forge account mapped to no person");
        out.object_end();
    }
    out.array_end();
}
fn encode_holds(holds: &Holds, out: &mut Encoder) {
    if holds.completion == Completion::NoChange {
        out.key(b"result");
        out.string(b"completed, no change");
    }
    if let Some(held) = &holds.own {
        out.key(b"task_hold");
        out.object_start();
        out.key(b"reason");
        out.string(&held.reason);
        out.key(b"way_out");
        out.string(&held.way_out);
        out.object_end();
    }
    if let Some(reason) = &holds.run {
        out.key(b"run_hold");
        out.string(reason);
    }
}
fn json(
    panel: Option<&w::ChangePanel>,
    digest: Option<Digest>,
    met: Option<bool>,
    why: &[u8],
    holds: &Holds,
    out: &mut Encoder,
) {
    if let Some(met) = met {
        out.object_start();
        out.key(b"change");
        match panel {
            Some(panel) => encode(panel, digest, holds, out),
            None => out.null(),
        }
        out.key(b"met");
        out.boolean(met);
        if !met {
            out.key(b"why");
            out.string(why);
        }
        out.object_end();
    } else {
        encode(panel.expect("read panel"), digest, holds, out);
    }
}
fn text(
    panel: Option<&w::ChangePanel>,
    digest: Option<Digest>,
    met: Option<bool>,
    why: &[u8],
    holds: &Holds,
    out: &mut Text,
) {
    if let Some(panel) = panel {
        text_heading(panel, out);
        if holds.completion == Completion::NoChange {
            out.put(b"result: completed, no change\n");
        }
        text_state(panel, holds, out);
        text_checks(panel, out);
        text_questions(panel.questions(), out);
        if let Some(digest) = digest {
            out.put(b"digest: ");
            out.put(&digest.hex());
            out.put(b"\n");
        }
        text_next(panel, holds, out);
    }
    match met {
        Some(true) => out.put(b"met\n"),
        Some(false) => {
            out.put(b"not met: ");
            out.put(why);
            out.put(b"\n");
        }
        None => {}
    }
}
fn approval_wait(panel: &w::ChangePanel) -> bool {
    for line in panel.approvals() {
        if *line.read() != w::ReviewRead::Complete || *line.remaining() != Some(0) || line.refused() {
            return true;
        }
    }
    false
}
fn text_heading(panel: &w::ChangePanel, out: &mut Text) {
    out.put(b"change ");
    out.number(panel.task());
    out.put(b" \xc2\xb7 ");
    out.put(panel.owner());
    out.put(b"/");
    out.put(panel.name());
    out.put(b" on forge ");
    out.number(u64::from(panel.forge()));
    out.put(b" \xc2\xb7 ");
    out.put(panel.branch());
    out.put(b" into ");
    out.put(panel.landing());
    out.put(b"\n");
    text_pull(panel, out);
    out.put(b"step: ");
    out.put(step(panel.step()));
    if let w::Step::Landed(landed) = panel.step() {
        out.put(b" \xc2\xb7 merge commit ");
        out.put(&hex(landed.merge()));
    }
    let uncleared = if let w::Step::Cleared(value) = panel.step() { value.uncleared() } else { false };
    if !uncleared {
        for line in tier_lines(panel.step()) {
            out.put(b" \xc2\xb7 ");
            out.put(line);
        }
    }
    out.put(b"\n");
    if progress(panel.step()) == Progress::Verifying || progress(panel.step()) == Progress::Cleared {
        out.put(b"merge commit ");
        out.put(&hex(landed(panel.step()).expect("landed step")));
        out.put(b"\n");
    }
    if uncleared {
        for line in tier_lines(panel.step()) {
            out.put(b"tier: ");
            out.put(line);
            out.put(b"\n");
        }
    }
}

fn text_pull(panel: &w::ChangePanel, out: &mut Text) {
    if let Some(pull) = panel.pull() {
        out.put(b"pull request #");
        out.number(pull.number());
        if !pull.address().is_empty() {
            out.put(b" \xc2\xb7 ");
            out.put(pull.address());
        }
        out.put(b"\n");
    }
}

fn text_state(panel: &w::ChangePanel, holds: &Holds, out: &mut Text) {
    text_way(panel, out);
    if let Some(hold) = panel.hold() {
        out.put(b"held: ");
        if let w::Hold::Authority(value) = hold {
            out.put(value.reason());
        } else {
            out.put(phrase(words::hold(hold)));
        }
        out.put(b" at ");
        out.put(step(panel.step()));
        out.put(b"\n");
    }
    text_holds(panel, holds, out);
    if let Some(queue) = words::queue(panel.step()) {
        out.put(b"queued: place ");
        out.number(u64::from(queue.place));
        for (i, task) in queue.ahead.iter().enumerate() {
            out.put(if i == 0 { b", behind task " } else { b", task " });
            out.number(*task);
        }
        out.put(b"\n");
    }
    if let w::Step::Repairing(_) = panel.step() {
        let (word, arguments) = words::line(panel);
        out.put(b"repair: ");
        for (candidate, literals, _) in &forge::words::CHANGES {
            if *candidate == word {
                crate::forge::literals_text(literals, &arguments, out);
                break;
            }
        }
        out.put(b"\n");
    }
}

fn text_next(panel: &w::ChangePanel, holds: &Holds, out: &mut Text) {
    let ended = holds.completion == Completion::NoChange || *panel.step() == w::Step::Ended;
    if ended || holds.own.is_some() || holds.run.is_some() {
        let task = if ended || holds.own.is_some() { panel.task() } else { panel.runs().expect("held delegate") };
        out.put(b"next: temper ctl DIR task show ");
        out.number(task);
        out.put(b"\n");
    } else if approval_wait(panel) && progress(panel.step()) == Progress::Before {
        out.put(b"next: review the pull request");
        if let Some(pull) = panel.pull() {
            out.put(b" ");
            out.put(pull.address());
        }
        out.put(b" for the required role; only a mapped project person counts\n");
    } else if progress(panel.step()) == Progress::Verifying {
        out.put(b"next: temper ctl DIR change show ");
        out.number(panel.task());
        out.put(b" --until cleared\n");
    } else if landed(panel.step()).is_none() && *panel.step() != w::Step::Ended {
        out.put(b"next: temper ctl DIR change show ");
        out.number(panel.task());
        out.put(b" --until landed\n");
    }
}
fn text_approvals(panel: &w::ChangePanel, out: &mut Text) {
    for line in panel.approvals() {
        out.put(b"approval: scope ");
        out.number(u64::from(line.scope()));
        out.put(b" rule ");
        out.number(u64::from(line.number()));
        out.put(b" role ");
        out.number(u64::from(line.role()));
        out.put(b" ");
        out.put(freshness(line.freshness()));
        out.put(b" required ");
        out.number(u64::from(line.required()));
        out.put(b"; ");
        match line.read() {
            w::ReviewRead::Unread => out.put(b"reviews not read yet"),
            w::ReviewRead::Incomplete => out.put(b"reviews incomplete"),
            w::ReviewRead::Complete => out.put(b"reviews complete"),
        }
        if let Some(remaining) = line.remaining() {
            out.put(b"; counted ");
            out.number(u64::from(line.counted().len()));
            out.put(b"; remaining ");
            out.number(u64::from(*remaining));
        }
        if line.refused() {
            out.put(b"; changes requested");
        }
        out.put(b"\n");
        for review in line.counted() {
            out.put(b"  counted person ");
            out.number(review.person());
            out.put(b" at head ");
            out.put(&hex(review.head()));
            out.put(b"\n");
        }
    }
    for review in panel.unmapped_reviews() {
        out.put(b"uncounted review: forge account ");
        out.number(review.account());
        out.put(b" at head ");
        out.put(&hex(review.head()));
        out.put(b" ");
        out.put(verdict(review.verdict()));
        out.put(b"; mapped to no person\n");
    }
}
fn text_holds(panel: &w::ChangePanel, holds: &Holds, out: &mut Text) {
    if let Some(held) = &holds.own {
        out.put(b"held: task ");
        out.number(panel.task());
        out.put(b" (");
        out.put(&held.reason);
        out.put(b")\nway out: ");
        out.put(&held.way_out);
        out.put(b"\n");
    }
    if let Some(task) = panel.runs() {
        out.put(if holds.run.is_some() { b"held: task " } else { b"running: task " });
        out.number(*task);
        if let Some(reason) = holds.run.as_deref() {
            out.put(b" (");
            out.put(reason);
            out.put(b")");
        }
        out.put(b"\n");
    }
}
fn text_way(panel: &w::ChangePanel, out: &mut Text) {
    match panel.way() {
        w::Way::Turn => {
            out.put(b"way: turn");
            if let Some(task) = panel.in_landing() {
                out.put(b" \xc2\xb7 landing task ");
                out.number(*task);
            }
            out.put(b"\n");
        }
        w::Way::Queue => {}
    }
}
fn text_checks(panel: &w::ChangePanel, out: &mut Text) {
    out.put(b"head ");
    match panel.head() {
        Some(head) => out.put(&hex(head)),
        None => out.put(b"unknown"),
    }
    out.put(b" \xc2\xb7 CI ");
    out.put(if panel.ci_required() { ci(panel.ci()) } else { b"not required" });
    out.put(b"\n");
    if panel.gates().is_empty() {
        out.put(b"gates: none\n");
    }
    for gate in panel.gates() {
        out.put(b"gate ");
        out.number(gate.number());
        out.put(b" \xc2\xb7 ");
        out.put(gate_kind(gate.kind()));
        out.put(if gate.blocks() { b" \xc2\xb7 blocking" } else { b" \xc2\xb7 advisory" });
        out.put(b" \xc2\xb7 ");
        out.put(verdict(gate.verdict()));
        out.put(b"\n");
    }
    text_approvals(panel, out);
    for (i, (label, used)) in [
        (b"repairs ".as_slice(), panel.repairs()),
        (b"resolutions ", panel.resolutions()),
        (b"updates ", panel.updates()),
    ]
    .iter()
    .enumerate()
    {
        if i > 0 {
            out.put(b" \xc2\xb7 ");
        }
        out.put(label);
        out.number(u64::from(used.used()));
        out.put(b" of ");
        out.number(u64::from(used.limit()));
    }
    out.put(b"\n");
}
fn digest(panel: &w::ChangePanel, limits: &Limits, holds: &Holds) -> Option<Digest> {
    let bounds = skein_json::writer::Limits { depth: 8, length: limits.tools.text };
    let mut measure = Encoder::measure(&bounds);
    encode(panel, None, holds, &mut measure);
    let mut out = Encoder::write(measure.measured().ok()?, &bounds);
    encode(panel, None, holds, &mut out);
    Some(Digest::of(&out.finish()))
}
pub(crate) fn print(
    panel: Option<&w::ChangePanel>,
    json_output: bool,
    met: Option<bool>,
    why: &[u8],
    limits: &Limits,
    holds: &Holds,
) -> jig::Printed {
    let digest = match panel {
        Some(panel) => match digest(panel, limits, holds) {
            Some(digest) => Some(digest),
            None => return render::failure(b"result limit", json_output, 1),
        },
        None => None,
    };
    let bytes = if json_output {
        let bounds = skein_json::writer::Limits { depth: 8, length: limits.tools.text };
        let mut measure = Encoder::measure(&bounds);
        json(panel, digest, met, why, holds, &mut measure);
        let Ok(size) = measure.measured() else { return render::failure(b"result limit", true, 1) };
        let mut out = Encoder::write(size, &bounds);
        json(panel, digest, met, why, holds, &mut out);
        out.finish()
    } else {
        let mut measure = Text::measure();
        text(panel, digest, met, why, holds, &mut measure);
        let mut out = Text::writing_exact(measure);
        text(panel, digest, met, why, holds, &mut out);
        out.finish()
    };
    if bytes.len() > usize::try_from(limits.tools.text).expect("text fits") {
        return render::failure(b"result limit", json_output, 1);
    }
    jig::Printed {
        file: jig::FileEnd::Keep,
        status: if met == Some(false) { 3 } else { 0 },
        stdout: bytes,
        stderr: Box::default(),
    }
}

fn tier_lines(step: &w::Step) -> &[Box<[u8]>] {
    match step {
        w::Step::Verifying(value) => value.tiers().as_slice(),
        w::Step::Red(value) => value.tiers().as_slice(),
        w::Step::Cleared(value) => value.tiers().as_slice(),
        w::Step::Ready(_)
        | w::Step::Preparing(_)
        | w::Step::Producing
        | w::Step::Opening
        | w::Step::Recreating
        | w::Step::Reopening
        | w::Step::Checking
        | w::Step::Gating
        | w::Step::Queued(_)
        | w::Step::First
        | w::Step::Updating
        | w::Step::Resolving
        | w::Step::Repairing(_)
        | w::Step::Landing
        | w::Step::Landed(_)
        | w::Step::Ended => &[],
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Progress {
    Before,
    Verifying,
    Cleared,
    Ended,
}
fn progress(step: &w::Step) -> Progress {
    match step {
        w::Step::Verifying(_) | w::Step::Red(_) => Progress::Verifying,
        w::Step::Cleared(_) => Progress::Cleared,
        w::Step::Landed(_) | w::Step::Ended => Progress::Ended,
        w::Step::Ready(_)
        | w::Step::Preparing(_)
        | w::Step::Producing
        | w::Step::Opening
        | w::Step::Recreating
        | w::Step::Reopening
        | w::Step::Checking
        | w::Step::Gating
        | w::Step::Queued(_)
        | w::Step::First
        | w::Step::Updating
        | w::Step::Resolving
        | w::Step::Repairing(_)
        | w::Step::Landing => Progress::Before,
    }
}

fn route(route: &w::Route) -> &'static [u8] {
    match route {
        w::Route::None => b"none",
        w::Route::Up => b"up",
    }
}
fn stands(stands: &w::Stands) -> &'static [u8] {
    match stands {
        w::Stands::Taken => b"taken",
        w::Stands::Asked => b"asked",
        w::Stands::Held => b"held",
    }
}
fn encode_questions(questions: &w::Questions, out: &mut Encoder) {
    out.key(b"questions");
    out.object_start();
    out.key(b"route");
    out.string(route(questions.route()));
    out.key(b"taken");
    out.unsigned(u64::from(questions.taken().used()));
    out.key(b"limit");
    out.unsigned(u64::from(questions.taken().limit()));
    out.key(b"open");
    out.array_start();
    for question in questions.open() {
        out.object_start();
        out.key(b"asker");
        out.unsigned(question.asker());
        out.key(b"number");
        out.unsigned(question.number());
        out.key(b"line");
        out.string(question.line());
        out.key(b"stands");
        out.string(stands(question.stands()));
        out.object_end();
    }
    out.array_end();
    out.object_end();
}
fn text_questions(questions: &w::Questions, out: &mut Text) {
    out.put(b"questions: ");
    out.put(route(questions.route()));
    if *questions.route() != w::Route::None || questions.taken().used() != 0 {
        out.put(b" \xc2\xb7 ");
        out.number(u64::from(questions.taken().used()));
        out.put(b" of ");
        out.number(u64::from(questions.taken().limit()));
        out.put(b" taken");
    }
    out.put(b"\n");
    if !questions.open().is_empty() {
        out.put(
            b"  Answer numbers: use read_task on this change; producer numbers identify the questions relayed below.\n",
        );
    }
    for question in questions.open() {
        out.put(b"  producer question ");
        out.number(question.number());
        out.put(b" of task ");
        out.number(question.asker());
        out.put(b" \xc2\xb7 ");
        match question.stands() {
            w::Stands::Taken => out.put(b"taken"),
            w::Stands::Asked => out.put(b"asked of requester"),
            w::Stands::Held => out.put(b"answered, held for dependencies"),
        }
        out.put(b" \xc2\xb7 ");
        out.put(question.line());
        out.put(b"\n");
    }
}
