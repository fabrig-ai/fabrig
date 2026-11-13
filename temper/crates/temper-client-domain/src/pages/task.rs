//! One task's borrowed documents, first activity page and live run handle.
//! See temper/docs/design/client/pages/goal.md, 2, and temper/docs/design/client/pages/task.md, 4.
use super::{
    Act, Request,
    activity::{Asking, READ_FIRST, READ_MOST, signed_out},
    inbox::Card,
};
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Key, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{Duration, Env, List, Queue, Time, Wall};
/// A task's state as a goal's page says it (client/pages/goal.md, 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// jig's word for its phase.
    Phase(c::Phase),
    /// Waiting on one listed dependency: this task.
    WaitingFor(u64),
    /// Waiting on `count` listed dependencies, and on more when the list was cut.
    WaitingForTasks { count: u32, more: bool },
    /// Waiting on no dependency, for its holdings.
    WaitingForHoldings,
    /// Waiting for the supplied number of live delegates.
    WaitingForDelegates(u32),
    /// Waiting for an answer from the reader.
    WaitingForYou,
    /// Waiting for an answer from another person or role.
    WaitingForParty(c::Whom),
    /// Waiting for news.
    WaitingForNews,
    /// Running without more specific observed work.
    Running,
    /// Held by the reader's own pause.
    Paused,
    /// Observed writing.
    Writing,
    /// Observed using this tool.
    Tool(Box<[u8]>),
    /// Ended, with its owner's ending.
    Ended(c::TaskStatus),
}
/// A task's state from its chip.
#[must_use]
pub fn row_state(chip: &c::Chip) -> State {
    if chip.phase == c::Phase::Running {
        if let Some(observation) = &chip.details.details.observation {
            match &observation.work {
                c::ChipWork::Writing => return State::Writing,
                c::ChipWork::Tool { tool, .. } => return State::Tool(tool.clone()),
                c::ChipWork::Preparing
                | c::ChipWork::Starting
                | c::ChipWork::Active
                | c::ChipWork::Waiting
                | c::ChipWork::Ending => {}
            }
        }
        return State::Running;
    }
    match chip.details.as_ref() {
        c::ChipPhaseFacts { phase: c::ChipPhase::Waiting { dependencies, more, holds }, .. } => {
            if dependencies.len() == 1 && !more {
                State::WaitingFor(*dependencies.as_slice().first().expect("one dependency"))
            } else if !dependencies.is_empty() || *more {
                State::WaitingForTasks { count: dependencies.len(), more: *more }
            } else if *holds {
                State::WaitingForHoldings
            } else {
                State::Phase(chip.phase)
            }
        }
        c::ChipPhaseFacts { phase: c::ChipPhase::Ended, details } => match &details.ending {
            Some(c::ChipEnding::Done(_)) => State::Ended(c::TaskStatus::Done),
            Some(c::ChipEnding::Failed(_)) => State::Ended(c::TaskStatus::Failed),
            Some(c::ChipEnding::Cancelled { .. }) => State::Ended(c::TaskStatus::Cancelled),
            None => State::Phase(chip.phase),
        },
        c::ChipPhaseFacts {
            phase:
                c::ChipPhase::Active(_) | c::ChipPhase::Person { .. } | c::ChipPhase::Closing(_) | c::ChipPhase::Held { .. },
            ..
        } => State::Phase(chip.phase),
    }
}
/// A goal's header state from its exact wait, falling back to its chip.
#[must_use]
pub fn header_state(task: &c::Task, me: Option<&c::Me>) -> State {
    if let Some(me) = me
        && super::chat::paused(&task.chip, me)
    {
        return State::Paused;
    }
    match task.chip.phase {
        c::Phase::Ended | c::Phase::Done | c::Phase::Failed | c::Phase::Cancelled => return row_state(&task.chip),
        c::Phase::Waiting
        | c::Phase::Idle
        | c::Phase::Due
        | c::Phase::Preparing
        | c::Phase::Claimed
        | c::Phase::Running
        | c::Phase::Active
        | c::Phase::BackingOff
        | c::Phase::Closing
        | c::Phase::Held => {}
    }
    match &task.waits_on {
        c::WaitsOn::Dependencies { pending, count } => {
            if *count == 1 && pending.len() == 1 {
                State::WaitingFor(*pending.as_slice().first().expect("one pending task"))
            } else if *count != 0 {
                State::WaitingForTasks { count: *count, more: false }
            } else {
                row_state(&task.chip)
            }
        }
        c::WaitsOn::Delegates => State::WaitingForDelegates(task.chip.details.details.live_delegates),
        c::WaitsOn::Party { whom, .. } => {
            if let Some(me) = me
                && reader(*whom, task.project, me)
            {
                State::WaitingForYou
            } else {
                State::WaitingForParty(*whom)
            }
        }
        c::WaitsOn::News => State::WaitingForNews,
        c::WaitsOn::Nothing => row_state(&task.chip),
    }
}
/// A task's run heading (client/pages/task.md, 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Run {
    /// A run is out: claimed, running, or its run closing.
    Live { attempt: u64, since: Option<Wall> },
    /// The task lives and no run is out: its last run's attempt.
    Waiting { attempt: u64, since: Option<Wall> },
    /// The task has ended: its last run's attempt.
    Ended { attempt: u64 },
    /// The task lives and has had no run.
    NoRun,
}
/// None when the task ended without a run, or its chip is not an owner's.
#[must_use]
pub fn run(task: &c::Task) -> Option<Run> {
    let c::ChipPhaseFacts { phase, details } = task.chip.details.as_ref();
    match phase {
        c::ChipPhase::Active(c::RunStage::Claimed { attempt } | c::RunStage::Running { attempt })
        | c::ChipPhase::Closing(c::ClosingStage::Run { attempt }) => {
            Some(Run::Live { attempt: *attempt, since: details.run_since })
        }
        c::ChipPhase::Ended => match details.attempt {
            Some(attempt) => Some(Run::Ended { attempt }),
            None => None,
        },
        c::ChipPhase::Active(_)
        | c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_)
        | c::ChipPhase::Held { .. } => Some(match details.attempt {
            Some(attempt) => Run::Waiting { attempt, since: details.run_since },
            None => Run::NoRun,
        }),
    }
}
// Jig's facts::reader is private; keep its person and project-role identity rule.
fn reader(whom: c::Whom, project: u32, me: &c::Me) -> bool {
    match whom {
        c::Whom::Person(person) => person == me.party.number,
        c::Whom::Role { project: held_project, role } => {
            if held_project != project {
                return false;
            }
            for held in &me.projects {
                let number = match held.role {
                    c::HeldRole::Owner => 0,
                    c::HeldRole::Maintainer => 1,
                    c::HeldRole::Member => 2,
                    c::HeldRole::Observer => 3,
                    c::HeldRole::Policy { role } => role,
                };
                if held.project == project && number == role {
                    return true;
                }
            }
            false
        }
    }
}
/// A dependency's state from its link; none when a stored link has no status.
#[must_use]
pub fn link_state(link: &c::TaskLink) -> Option<State> {
    match link {
        c::TaskLink::Live(chip) | c::TaskLink::Retained { chip, .. } => Some(row_state(chip)),
        c::TaskLink::Stored { status: Some(status), .. } => Some(State::Ended(*status)),
        c::TaskLink::Stored { status: None, .. } => None,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Unknown,
    Tracked,
    /// An agent's task with no priority, not the chat label (client/pages/task.md, 2).
    Agent,
    HeaderAndPlan,
}
/// A standing role's composer uses Talk whenever its document supplies that act
/// (client/pages/task.md, 4), including its refusal rather than falling back to steering.
#[must_use]
pub fn talks(task: &c::Task) -> bool {
    task.flags.ended_by == c::EndedBy::Standing
        && jig_client::facts::offer(task.acts.as_slice(), c::ActKind::Talk).is_some()
}
/// Leave belongs to the reader's open talk and must be admitted by the document.
#[must_use]
pub fn can_leave(task: &c::Task, me: Option<&c::Me>) -> bool {
    super::chat::allowed(task, c::ActKind::Leave)
        && match (&task.talk, me) {
            (Some(open), Some(me)) => open.party == me.party.number,
            (Some(_), None) | (None, Some(_) | None) => false,
        }
}
/// A task page's request kinds, one in flight per kind (client/pages/task.md, 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sending {
    Message,
    Leave,
    Stop,
    Resume,
    Cancel,
    Amend,
}
/// The page's open amendment; its draft is 3.7's (client/pages/task.md, 4).
#[derive(Debug)]
pub struct Amending {
    pub draft: crate::amend::Draft,
    pub words: Box<[u8]>,
    pub cut: bool,
    pub tracking: Option<c::Tracking>,
    pub no_change: bool,
    pub done: bool,
    pub outcome: Option<jig_client::Outcome>,
}
impl Amending {
    fn of(task: &c::Task, shape: Shape) -> Self {
        let mut draft = crate::amend::Draft::of(task);
        let words = task.spec.specification.words.bytes.clone();
        let cut = task.spec.specification.words.cut || draft.words(words.clone()).is_err();
        Self {
            draft,
            words: if cut { Box::default() } else { words.into_bytes() },
            cut,
            tracking: if shape == Shape::Tracked {
                Some(match task.priority {
                    Some(priority) => c::Tracking::Tracked { priority },
                    None => c::Tracking::Untracked,
                })
            } else {
                None
            },
            no_change: false,
            done: false,
            outcome: None,
        }
    }
}
/// A goal's or procedure's ended delegates, one read out at a time (client/pages/goal.md, 2, client/pages/task.md, 3).
#[derive(Debug)]
pub struct Ended {
    pub handle: Handle,
    pub page: Option<Box<c::TaskPage>>,
    pub reading: bool,
    pub stale: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadEnd {
    Busy,
    Unreachable,
    Limit,
    /// Jig ends old reads when a successful sign-in installs its credential.
    SignIn,
    SignedOut,
    Unknown,
    NotAllowed,
    NotServed,
    Gone,
    TooLarge,
    TooLinked {
        joins: u32,
    },
    Malformed,
}
impl ReadEnd {
    #[must_use]
    pub const fn transient(self) -> bool {
        match self {
            Self::Busy | Self::Unreachable | Self::Limit | Self::SignIn => true,
            Self::SignedOut
            | Self::Unknown
            | Self::NotAllowed
            | Self::NotServed
            | Self::Gone
            | Self::TooLarge
            | Self::TooLinked { .. }
            | Self::Malformed => false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentRead {
    Reading,
    Ready,
    Ended { why: ReadEnd, retry_at: Option<Time> },
}

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads document state while backoff stays private")]
pub struct Task {
    pub number: u64,
    pub document: Handle,
    pub document_read: DocumentRead,
    read_wait: Duration,
    long_text: u32,
    asking: Asking,
    /// The document that already permitted an ending-triggered panel reopen (client/forge/served.md, 2).
    pub(crate) panel_reopened_at: Option<u64>,
    pub tree: Handle,
    pub activity: Handle,
    pub run: Option<Handle>,
    current_call: Option<Box<c::CurrentCall>>,
    pub ended: Option<Ended>,
    pub goal: Option<super::goal::Goal>,
    pub shape: Shape,
    pub latest: Option<Box<c::ActivityPage>>,
    pub thread: jig_client::thread::Thread,
    pub final_task: Option<Box<c::Task>>,
    pub next: u64,
    pub through: Option<u64>,
    pub cards: List<Card>,
    pub words: Box<[u8]>,
    pub beyond: u64,
    pub composer_focus: u64,
    pub pending: List<(Key, Sending)>,
    pub outcome: Option<jig_client::Outcome>,
    pub amending: Option<Amending>,
}
impl Task {
    /// The supplied call under way (temper/docs/design/client/pages/task.md, 2).
    #[must_use]
    pub fn current_call(&self) -> Option<&c::CurrentCall> {
        self.current_call.as_deref()
    }

    pub(crate) fn new(number: u64, first: u64, limits: &Limits, out: &mut Queue<Request>) -> Self {
        let document = handle(first);
        let tree = handle(first.checked_add(1).expect("task handles fit"));
        let activity = handle(first.checked_add(2).expect("task handles fit"));
        for (handle, subject) in [
            (document, c::Subject::Task { task: number }),
            (tree, c::Subject::Tree { task: number, window: limits.client.domain.rows }),
        ] {
            out.push(Request::Client(jig_client::Event::Watch { handle, subject: Watchable::Jig(subject) }));
        }
        let mut thread = jig_client::thread::Thread::new(number, &super::chat::thread_limits(limits));
        thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
        let mut asking = Asking::new();
        asking.ask(activity, c::Read::Activity { task: number, before: None }, out);
        Self {
            number,
            document,
            document_read: DocumentRead::Reading,
            read_wait: READ_FIRST,
            long_text: limits.client.domain.long_text,
            asking,
            panel_reopened_at: None,
            tree,
            activity,
            run: None,
            current_call: None,
            ended: None,
            goal: None,
            shape: Shape::Unknown,
            latest: None,
            thread,
            final_task: None,
            next: first.checked_add(3).expect("task handles fit"),
            through: None,
            cards: List::with_capacity(limits.client.domain.reads),
            words: Box::default(),
            beyond: 0,
            composer_focus: 0,
            pending: List::with_capacity(6),
            outcome: None,
            amending: None,
        }
    }
    #[must_use]
    pub fn document<'a>(&'a self, client: &'a Client) -> Option<&'a c::Task> {
        if let Some(task) = &self.final_task {
            return Some(task);
        }
        match client.document(self.document)?.latest? {
            Latest::Jig(c::Delivery::Task(task)) => Some(task),
            Latest::Jig(_) | Latest::Family(_) => None,
        }
    }
    /// A procedure's connector panel (jig's client/domain/pages.md, 5.2).
    #[must_use]
    pub fn panel(&self, client: &Client, forge: u16) -> Option<crate::forge::panel::Subject> {
        let task = self.document(client)?;
        match task.chip.executor {
            c::ChipExecutor::Procedure { connector, .. } if connector == forge => {
                Some(crate::forge::panel::Subject { task: task.chip.task, project: task.project })
            }
            c::ChipExecutor::Procedure { .. }
            | c::ChipExecutor::Job { .. }
            | c::ChipExecutor::Agent { .. }
            | c::ChipExecutor::Party { .. }
            | c::ChipExecutor::Role { .. } => None,
        }
    }
    #[expect(
        clippy::too_many_lines,
        reason = "composer dispatch lists every page act, including Ask how and role talks"
    )]
    pub(crate) fn act(
        &mut self,
        env: &Env<Limits>,
        client: &Client,
        act: Act,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if self.shape != Shape::Agent && self.shape != Shape::Tracked {
            return;
        }
        match act {
            Act::MessageWords { words } => {
                (self.words, self.beyond) = super::composer::held(words, env.limits.client.domain.long_text);
            }
            Act::Send => {
                let _sent = self.send(env, client, Sending::Message, serial, out);
                self.composer_focus = self.composer_focus.checked_add(1).expect("focus count fits");
            }
            Act::Older if self.shape == Shape::Agent => {
                if let Some(older) = self.thread.older() {
                    self.asking.ask(self.activity, older, out);
                }
            }
            Act::Newer | Act::Latest if self.shape == Shape::Agent => {
                if let Some(newer) = self.thread.newer() {
                    self.asking.ask(self.activity, newer, out);
                }
            }
            Act::Older | Act::Newer | Act::Latest => {}
            Act::ChangeProposal { .. }
            | Act::Amend
            | Act::AmendWords { .. }
            | Act::AmendTracked
            | Act::AmendPriority { .. }
            | Act::DiscardAmendment
            | Act::SendAmendment
            | Act::AmendAgain
            | Act::MakeDefault
            | Act::AdoptFirst
            | Act::CopyAgain { .. }
            | Act::AddOpen { .. }
            | Act::AddChoose { .. }
            | Act::AddWrite { .. }
            | Act::AddClose
            | Act::AddSubmit
            | Act::CopyOffered
            | Act::DefaultName { .. }
            | Act::OpenProject { .. }
            | Act::Project { .. }
            | Act::OpenStart { .. }
            | Act::OpenLanded { .. }
            | Act::StartDraft { .. }
            | Act::StartName { .. }
            | Act::StartWay { .. }
            | Act::StartAsks { .. }
            | Act::StartCancel
            | Act::StartGoal
            | Act::Switcher(_)
            | Act::Away(_)
            | Act::Terminal(_)
            | Act::Told(_)
            | Act::Remembered(_)
            | Act::Models(_)
            | Act::HandArea(_)
            | Act::StopAsking(_)
            | Act::Needs(_)
            | Act::Finding(_)
            | Act::TellWords { .. }
            | Act::Tell { .. }
            | Act::Choice(_)
            | Act::SignOut
            | Act::Accounts(_)
            | Act::ConnectModel(_)
            | Act::AskLine { .. }
            | Act::AskWith { .. }
            | Act::ChatWords { .. }
            | Act::AskHow
            | Act::StartChat
            | Act::WayTab { .. }
            | Act::WayStop { .. }
            | Act::UseWay { .. }
            | Act::KeepWay { .. }
            | Act::SkipToBegin
            | Act::RetryPlayground
            | Act::Playground
            | Act::Suggest { .. }
            | Act::LeaveTalk
            | Act::Pause
            | Act::Resume
            | Act::Stop
            | Act::Close
            | Act::DeleteProject { .. }
            | Act::StartOver
            | Act::DelegateFate { .. }
            | Act::ClosedChats
            | Act::MoreGoals
            | Act::GoalsFilter { .. }
            | Act::ChatFilter { .. }
            | Act::Filter { .. }
            | Act::OpenResult { .. }
            | Act::Decide { .. }
            | Act::CardWords { .. }
            | Act::Again { .. }
            | Act::Confirm
            | Act::Cancel
            | Act::AddDraft
            | Act::Mode { .. }
            | Act::ShowSection { .. }
            | Act::Go { .. } => unreachable!("only composer and older acts routed here"),
        }
    }
    /// The task page's amendment acts (client/pages/task.md, 4).
    #[expect(
        clippy::too_many_lines,
        reason = "amendment dispatch lists every page act, including start goal and Home’s repository sheet"
    )]
    pub(crate) fn amend(
        &mut self,
        env: &Env<Limits>,
        client: &Client,
        act: Act,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        match act {
            Act::Amend | Act::AmendAgain => {
                if let Some(open) = &self.amending
                    && !open.done
                    && act == Act::Amend
                {
                    return;
                }
                if let Some(task) = self.document(client)
                    && (self.shape == Shape::Tracked || self.shape == Shape::Agent)
                    && super::chat::allowed(task, c::ActKind::Amend)
                {
                    self.amending = Some(Amending::of(task, self.shape));
                }
            }
            Act::DiscardAmendment => {
                if let Some(open) = &self.amending
                    && open.done
                {
                    return;
                }
                self.amending = None;
            }
            Act::AmendWords { words } => {
                if let Some(open) = &mut self.amending
                    && !open.done
                    && !open.cut
                    && let Ok(text) = c::Text::new(words, env.limits.client.domain.text)
                {
                    open.words = text.clone().into_bytes();
                    open.draft.words(text).expect("whole specification");
                    open.no_change = false;
                }
            }
            Act::AmendTracked => {
                if let Some(open) = &mut self.amending
                    && !open.done
                    && let Some(tracking) = open.tracking
                {
                    let tracking = match tracking {
                        c::Tracking::Tracked { .. } => c::Tracking::Untracked,
                        c::Tracking::Untracked => c::Tracking::Tracked { priority: 0 },
                    };
                    open.tracking = Some(tracking);
                    open.draft.track(tracking);
                    open.no_change = false;
                }
            }
            Act::AmendPriority { priority } => {
                if let Some(open) = &mut self.amending
                    && !open.done
                    && let Some(c::Tracking::Tracked { .. }) = open.tracking
                {
                    let tracking = c::Tracking::Tracked { priority };
                    open.tracking = Some(tracking);
                    open.draft.track(tracking);
                    open.no_change = false;
                }
            }
            Act::SendAmendment => {
                let _sent = self.send(env, client, Sending::Amend, serial, out);
            }
            Act::MakeDefault
            | Act::AdoptFirst
            | Act::CopyAgain { .. }
            | Act::AddOpen { .. }
            | Act::AddChoose { .. }
            | Act::AddWrite { .. }
            | Act::AddClose
            | Act::AddSubmit
            | Act::CopyOffered
            | Act::DefaultName { .. }
            | Act::OpenProject { .. }
            | Act::Project { .. }
            | Act::OpenStart { .. }
            | Act::OpenLanded { .. }
            | Act::StartDraft { .. }
            | Act::StartName { .. }
            | Act::StartWay { .. }
            | Act::StartAsks { .. }
            | Act::StartCancel
            | Act::StartGoal
            | Act::Switcher(_)
            | Act::Away(_)
            | Act::Terminal(_)
            | Act::Told(_)
            | Act::Remembered(_)
            | Act::Models(_)
            | Act::HandArea(_)
            | Act::StopAsking(_)
            | Act::Needs(_)
            | Act::Finding(_)
            | Act::TellWords { .. }
            | Act::Tell { .. }
            | Act::Choice(_)
            | Act::SignOut
            | Act::Accounts(_)
            | Act::ConnectModel(_)
            | Act::AskLine { .. }
            | Act::AskWith { .. }
            | Act::ChatWords { .. }
            | Act::AskHow
            | Act::StartChat
            | Act::WayTab { .. }
            | Act::WayStop { .. }
            | Act::UseWay { .. }
            | Act::KeepWay { .. }
            | Act::SkipToBegin
            | Act::RetryPlayground
            | Act::Playground
            | Act::Suggest { .. }
            | Act::ChangeProposal { .. }
            | Act::MessageWords { .. }
            | Act::Send
            | Act::LeaveTalk
            | Act::Pause
            | Act::Resume
            | Act::Stop
            | Act::Close
            | Act::DeleteProject { .. }
            | Act::StartOver
            | Act::DelegateFate { .. }
            | Act::Older
            | Act::Newer
            | Act::Latest
            | Act::ClosedChats
            | Act::MoreGoals
            | Act::GoalsFilter { .. }
            | Act::ChatFilter { .. }
            | Act::Filter { .. }
            | Act::OpenResult { .. }
            | Act::Decide { .. }
            | Act::CardWords { .. }
            | Act::Again { .. }
            | Act::Confirm
            | Act::Cancel
            | Act::AddDraft
            | Act::Mode { .. }
            | Act::ShowSection { .. }
            | Act::Go { .. } => unreachable!("only amendment acts routed here"),
        }
    }
    pub(crate) fn send(
        &mut self,
        env: &Env<Limits>,
        client: &Client,
        kind: Sending,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) -> Option<Key> {
        let task = self.document(client)?;
        let reader = match latest(client, super::me_handle()) {
            Some(Latest::Jig(c::Delivery::Me(me))) => Some(me.as_ref()),
            Some(Latest::Jig(_) | Latest::Family(_)) | None => None,
        };
        let paused = match self.shape {
            Shape::Tracked => super::goals::state(&task.chip, false, false, 0) == super::goals::State::Paused,
            Shape::Agent => match reader {
                Some(me) => super::chat::paused(&task.chip, me),
                None => false,
            },
            Shape::Unknown | Shape::HeaderAndPlan => false,
        };
        let ask = match kind {
            Sending::Message
                if (self.shape == Shape::Agent || self.shape == Shape::Tracked)
                    && task.chip.details.phase != c::ChipPhase::Ended
                    && !self.words.is_empty()
                    && super::chat::allowed(task, if talks(task) { c::ActKind::Talk } else { c::ActKind::Message }) =>
            {
                let me = reader?;
                if super::composer::over(&self.words, self.beyond, me.words_bytes) > 0 {
                    return None;
                }
                let Ok(words) = c::Text::new(self.words.clone(), env.limits.client.domain.long_text) else {
                    return None;
                };
                if talks(task) {
                    c::Ask::Talk { project: task.project, task: self.number, words }
                } else {
                    c::Ask::Message { project: task.project, task: self.number, words }
                }
            }
            Sending::Leave if self.shape == Shape::Agent && can_leave(task, reader) => {
                c::Ask::Leave { project: task.project, task: self.number }
            }
            Sending::Stop | Sending::Resume => control(task, kind, paused)?,
            Sending::Cancel if super::chat::allowed(task, c::ActKind::Cancel) => c::Ask::Cancel {
                project: task.project,
                task: self.number,
                reason: c::Text::new(
                    Box::from(if self.shape == Shape::Tracked || self.shape == Shape::Agent {
                        b"Closed from its page".as_slice()
                    } else {
                        b"Cancelled from its page".as_slice()
                    }),
                    u32::MAX,
                )
                .expect("page cancellation reason fits"),
                grace: None,
            },
            Sending::Amend if super::chat::allowed(task, c::ActKind::Amend) => {
                let Some(open) = &self.amending else { return None };
                if open.done || open.draft.moved(task) {
                    return None;
                }
                let Some(drafted) = open.draft.amendment() else {
                    self.amending.as_mut().expect("open amendment").no_change = true;
                    return None;
                };
                c::Ask::Amend {
                    project: task.project,
                    task: self.number,
                    revision: drafted.revision,
                    change: Box::new(drafted.change),
                    reason: c::Text::new(Box::from(*b"Amended from its page"), u32::MAX).expect("page reason fits"),
                }
            }
            Sending::Message | Sending::Leave | Sending::Cancel | Sending::Amend => return None,
        };
        for (_, pending) in &self.pending {
            if *pending == kind {
                return None;
            }
        }
        *serial = serial.checked_add(1).expect("task keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        if let c::Ask::Message { words, .. } | c::Ask::Talk { words, .. } = &ask {
            let me = reader?;
            if !self.thread.sending(key, me.party.number, words.clone(), 0) {
                self.outcome = Some(jig_client::Outcome::Limit);
                return None;
            }
            self.words = Box::default();
            self.beyond = 0;
        }
        self.pending.push((key, kind)).expect("one request per task act fits");
        if kind == Sending::Amend {
            self.amending.as_mut().expect("open amendment").outcome = None;
        } else {
            self.outcome = None;
        }
        out.push(Request::Client(jig_client::Event::Send {
            key,
            name: None,
            request: jig_client::Ask::Jig(Box::new(ask)),
        }));
        Some(key)
    }
    /// Goal outcomes and leaving a role talk belong to the frame toast (client/pages/task.md, 4).
    pub(crate) fn toasted(&self, told: &Told) -> Option<super::needs::Toasted> {
        if self.shape != Shape::Tracked && self.shape != Shape::Agent {
            return None;
        }
        let Told::Answered { key, outcome } = told else { return None };
        let kind = self.pending_kind(*key)?;
        let done = match outcome {
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(_))) => true,
            jig_client::Outcome::Answered(_)
            | jig_client::Outcome::Full { .. }
            | jig_client::Outcome::NotKept
            | jig_client::Outcome::Held
            | jig_client::Outcome::Unknown
            | jig_client::Outcome::Limit
            | jig_client::Outcome::Unconfirmed
            | jig_client::Outcome::Unsent
            | jig_client::Outcome::Lost => false,
        };
        if kind == Sending::Leave && done {
            return Some(super::needs::Toasted::LeftTalk);
        }
        if self.shape != Shape::Tracked {
            return None;
        }
        match kind {
            Sending::Stop if done => Some(super::needs::Toasted::Paused),
            Sending::Resume if done => Some(super::needs::Toasted::Resumed),
            Sending::Leave | Sending::Stop | Sending::Resume | Sending::Cancel | Sending::Message if !done => {
                Some(super::needs::Toasted::Refused { outcome: outcome.clone() })
            }
            Sending::Leave | Sending::Stop | Sending::Resume | Sending::Cancel | Sending::Message | Sending::Amend => {
                None
            }
        }
    }
    fn pending_kind(&self, key: Key) -> Option<Sending> {
        for (pending, kind) in &self.pending {
            if *pending == key {
                return Some(*kind);
            }
        }
        None
    }
    pub(crate) fn leave(&self, out: &mut Queue<Request>) -> u64 {
        for handle in [Some(self.document), Some(self.tree), Some(self.activity), self.run].into_iter().flatten() {
            unwatch(handle, out);
        }
        if let Some(ended) = &self.ended {
            unwatch(ended.handle, out);
        }
        for card in &self.cards {
            unwatch(card.handle, out);
        }
        if let Some(goal) = &self.goal {
            goal.leave(out);
        }
        self.next
    }
    fn answered(&mut self, key: Key, outcome: jig_client::Outcome) {
        if self.pending_kind(key) == Some(Sending::Message) {
            // Goals restore durable words from their latest activity page (5.8).
            if self.shape == Shape::Tracked {
                self.thread.words.refused(key);
            } else if let jig_client::Outcome::Answered(jig_client::Answered::Jig(answer)) = &outcome {
                self.thread.answered(key, answer);
            } else {
                self.thread.words.refused(key);
            }
        }
        let kind = self.pending_kind(key);
        // A message delivered is said by its words, "Sent", as a chat's
        // (client/pages/task.md, 4); a refused or unknown one by the outcome.
        let mut said = false;
        if kind == Some(Sending::Message)
            && let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(_))) = &outcome
        {
            said = true;
        }
        if (kind == Some(Sending::Leave) || kind == Some(Sending::Stop) || kind == Some(Sending::Resume))
            && let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(_))) = &outcome
        {
            said = true;
        }
        if kind == Some(Sending::Amend) {
            if let Some(open) = &mut self.amending {
                if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(_))) = &outcome {
                    open.done = true;
                }
                open.outcome = Some(outcome);
            }
        } else if !said && self.shape != Shape::Tracked {
            self.outcome = Some(outcome);
        }
        let mut kept = List::with_capacity(self.pending.capacity());
        for row in &self.pending {
            if row.0 != key {
                kept.push(*row).expect("remaining pending fits");
            }
        }
        self.pending = kept;
    }
    pub(crate) fn told(&mut self, now: Time, client: &Client, told: Told, out: &mut Queue<Request>) -> bool {
        if let Some(goal) = &mut self.goal
            && goal.told(now, client, &told, out)
        {
            return true;
        }
        if let Told::Page { handle, page: Readout::Jig(c::ReadResult::Value(value)) } = &told {
            for index in 0..self.cards.len() {
                let card = self.cards.get_mut(index).expect("bounded held card");
                if card.handle == *handle {
                    if let c::ReadValue::Task(task) = value {
                        card.project = Some(task.project);
                    }
                    card.page = Some(value.clone());
                    if self.shape == Shape::Agent && self.run.is_none() && self.thread.loaded >= self.thread.committed {
                        self.thread.end_run();
                    }
                    return true;
                }
            }
        }
        match told {
            Told::Answered { key, outcome } if self.pending_kind(key).is_some() => {
                self.answered(key, outcome);
            }
            Told::Pending { key } if self.pending_kind(key).is_some() => {}

            Told::Changed { handle } if handle == self.document => {
                if let Some(Latest::Jig(c::Delivery::Task(task))) = latest(client, handle) {
                    self.document_read = DocumentRead::Ready;
                    self.changed(task, out);
                }
            }
            Told::Ended { handle, why } if handle == self.document => {
                self.document_ended(now, client, why, out);
            }
            Told::Page { handle, page } if handle == self.document => {
                self.document_page(now, client, page, out);
            }
            Told::Page { handle, page } if Some(handle) == self.ended_handle() => {
                let ended = self.ended.as_mut().expect("ended children handle");
                ended.reading = false;
                if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(page))) = page {
                    ended.page = Some(page);
                }
                if ended.stale {
                    self.read_ended(out);
                }
            }
            Told::Page { handle, page } if handle == self.activity => {
                self.activity_page(now, client, page, out);
            }
            Told::Changed { handle } if handle == self.tree => {
                self.read_ended(out);
                match latest(client, handle) {
                    Some(Latest::Jig(c::Delivery::Tree(tree))) => {
                        self.held(&tree.root, out);
                        for row in &tree.rows {
                            self.held(&row.chip, out);
                        }
                    }

                    Some(Latest::Jig(_) | Latest::Family(_)) | None => {}
                }
            }
            Told::Text { handle } if handle == self.tree => {}
            Told::Changed { handle } | Told::Text { handle } if Some(handle) == self.run => {
                if self.shape == Shape::Agent
                    && let Some(Latest::Jig(c::Delivery::RunState(run))) = latest(client, handle)
                {
                    self.thread.run(run, client.text(handle).or(run.writing.as_ref()));
                    self.current_call = match &run.call {
                        Some(call) => Some(Box::new(call.clone())),
                        None => None,
                    };
                }
            }
            Told::Ended { handle, .. } if handle == self.tree || Some(handle) == self.run => {}
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => return false,
        }
        if self.shape == Shape::Agent && self.run.is_none() && self.thread.loaded >= self.thread.committed {
            self.thread.end_run();
        }
        true
    }
    fn document_ended(&mut self, now: Time, client: &Client, why: jig_client::Why, out: &mut Queue<Request>) {
        let why = match why {
            jig_client::Why::SubjectEnded | jig_client::Why::Unknown => {
                self.document_read = DocumentRead::Reading;
                read(self.document, c::Read::Task { task: self.number }, out);
                return;
            }
            jig_client::Why::NotAllowed => ReadEnd::NotAllowed,
            jig_client::Why::SignedOut => signed_out(client),
            jig_client::Why::NotServed => ReadEnd::NotServed,
            jig_client::Why::Limit => ReadEnd::Limit,
        };
        self.read_ended_at(now, why);
    }
    fn document_page(&mut self, now: Time, client: &Client, page: Readout, out: &mut Queue<Request>) {
        let why = super::activity::read_end(client, &page);
        if let Some(task) = super::chat::page_task(page, self.number) {
            self.document_read = DocumentRead::Ready;
            self.changed(&task, out);
            self.final_task = Some(task);
        } else {
            self.read_ended_at(now, why);
        }
    }
    fn read_ended_at(&mut self, now: Time, why: ReadEnd) {
        let retry_at = if why.transient() { Some(now.saturating_add(self.read_wait)) } else { None };
        self.document_read = DocumentRead::Ended { why, retry_at };
        if retry_at.is_some() {
            self.read_wait = self.read_wait.saturating_mul(2).min(READ_MOST);
        }
    }
    pub(crate) fn link(&mut self, link: jig_client::Link) {
        let why = match link {
            jig_client::Link::SignedOut => ReadEnd::SignedOut,
            jig_client::Link::OutOfDate => ReadEnd::NotServed,
            jig_client::Link::Starting | jig_client::Link::Live | jig_client::Link::Offline => return,
        };
        self.document_read = DocumentRead::Ended { why, retry_at: None };
    }
    pub(crate) fn fire(&mut self, now: Time, out: &mut Queue<Request>) {
        if let DocumentRead::Ended { retry_at: Some(at), .. } = self.document_read
            && at <= now
        {
            self.document_read = DocumentRead::Reading;
            read(self.document, c::Read::Task { task: self.number }, out);
        }
        self.asking.fire(now, self.activity, out);
        if let Some(goal) = &mut self.goal {
            goal.fire(now, out);
        }
    }
    #[must_use]
    pub(crate) const fn next_deadline(&self) -> Option<Time> {
        let document = match self.document_read {
            DocumentRead::Ended { retry_at, .. } => retry_at,
            DocumentRead::Reading | DocumentRead::Ready => None,
        };
        let goal = match &self.goal {
            Some(goal) => goal.next_deadline(),
            None => None,
        };
        super::activity::earliest(super::activity::earliest(document, self.asking.next_deadline()), goal)
    }
    fn activity_page(&mut self, now: Time, client: &Client, page: Readout, out: &mut Queue<Request>) {
        let why = super::activity::read_end(client, &page);
        if let Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(page))) = page {
            self.asking.answered();
            if self.shape == Shape::Agent {
                let empty_with_more = page.rows.is_empty() && page.before.is_some();
                if let Some(reading) = self.thread.page(*page) {
                    self.asking.ask(self.activity, reading, out);
                }
                if empty_with_more && let Some(older) = self.thread.older() {
                    self.asking.ask(self.activity, older, out);
                }
            } else {
                self.latest = Some(page);
            }
        } else if let Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut)) = page
            && client.link().state == jig_client::Link::Starting
            && super::awaits_first_read(&self.thread)
            && self.latest.is_none()
        {
            self.asking.ask(self.activity, c::Read::Activity { task: self.number, before: None }, out);
        } else if why.transient() {
            self.asking.ended(now);
        } else {
            self.asking.answered();
            self.thread.failed();
        }
    }
    /// Tracked pages read needs through their inbox, without task cards (client/pages/task.md, 4).
    pub(crate) fn held(&mut self, chip: &c::Chip, out: &mut Queue<Request>) {
        if self.shape == Shape::Tracked || chip.phase != c::Phase::Held {
            return;
        }
        let object = c::ObjectKey::Task(chip.task);
        for index in 0..self.cards.len() {
            let card = self.cards.get_mut(index).expect("bounded held card");
            if card.object != object {
                continue;
            }
            // A card read for an earlier hold is read again for this one, so a
            // task held anew offers Release anew (client/pages/task.md, 4).
            if card.key.is_none()
                && let (Some(was), Some(now)) = (hold_revision(card.page.as_ref()), chip_hold(chip))
                && was < now
            {
                unwatch(card.handle, out);
                card.handle = handle(self.next);
                self.next = self.next.checked_add(1).expect("task handles fit");
                card.page = None;
                card.words = None;
                card.outcome = None;
                read(card.handle, c::Read::Task { task: chip.task }, out);
            }
            return;
        }
        if self.cards.len() == self.cards.capacity() {
            return;
        }
        let handle = handle(self.next);
        self.next = self.next.checked_add(1).expect("task handles fit");
        self.cards
            .push(Card {
                whole: None,
                object,
                group: c::InboxGroup::Escalations,
                project: None,
                position: None,
                unread: None,
                handle,
                page: None,
                words: None,
                key: None,
                outcome: None,
            })
            .expect("held cards bounded");
        read(handle, c::Read::Task { task: chip.task }, out);
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "the ended-handle accessor uses a stateless projection of its optional state"
    )]
    fn ended_handle(&self) -> Option<Handle> {
        self.ended.as_ref().map(|ended| ended.handle)
    }
    fn read_ended(&mut self, out: &mut Queue<Request>) {
        let Some(ended) = &mut self.ended else { return };
        if ended.reading {
            ended.stale = true;
            return;
        }
        ended.reading = true;
        ended.stale = false;
        read(
            ended.handle,
            c::Read::Tasks {
                query: c::TaskQuery {
                    project: None,
                    requester: None,
                    root: None,
                    parent: Some(self.number),
                    label: None,
                    phase: Some(c::PhaseClass::Ended),
                    executor: None,
                    tracked: None,
                    order: c::QueryOrder::Ended,
                },
                live: None,
                ended: None,
            },
            out,
        );
    }
    fn changed(&mut self, task: &c::Task, out: &mut Queue<Request>) {
        let reads_ended = match task.chip.executor {
            c::ChipExecutor::Procedure { .. } => true,
            c::ChipExecutor::Agent { .. } => task.priority.is_some(),
            c::ChipExecutor::Job { .. } | c::ChipExecutor::Party { .. } | c::ChipExecutor::Role { .. } => false,
        };
        if reads_ended {
            if self.ended.is_none() {
                let handle = handle(self.next);
                self.next = self.next.checked_add(1).expect("task handles fit");
                self.ended = Some(Ended { handle, page: None, reading: false, stale: false });
            }
            self.read_ended(out);
        }
        self.shape = match task.chip.executor {
            c::ChipExecutor::Agent { .. } if task.priority.is_some() => Shape::Tracked,
            c::ChipExecutor::Agent { .. } => Shape::Agent,
            c::ChipExecutor::Job { .. }
            | c::ChipExecutor::Procedure { .. }
            | c::ChipExecutor::Party { .. }
            | c::ChipExecutor::Role { .. } => Shape::HeaderAndPlan,
        };
        if self.shape == Shape::Tracked {
            for card in &self.cards {
                unwatch(card.handle, out);
            }
            self.cards.clear();
            if let Some(goal) = &mut self.goal {
                goal.changed(task, &mut self.next, out);
            } else {
                self.goal = Some(super::goal::Goal::new(task, self.long_text, &mut self.next, out));
            }
        } else if let Some(goal) = self.goal.take() {
            goal.leave(out);
        }
        if self.shape == Shape::Agent {
            if let Some(page) = self.latest.take()
                && let Some(reading) = self.thread.page(*page)
            {
                self.asking.ask(self.activity, reading, out);
            }
            if let Some(reading) = self.thread.document(task) {
                self.asking.ask(self.activity, reading, out);
            }
        } else {
            if let Some(through) = self.through
                && through != task.activity
            {
                self.asking.ask(self.activity, c::Read::Activity { task: self.number, before: None }, out);
            }
            self.through = Some(task.activity);
        }
        let running = match task.chip.details.as_ref() {
            c::ChipPhaseFacts {
                phase:
                    c::ChipPhase::Active(c::RunStage::Claimed { .. } | c::RunStage::Running { .. })
                    | c::ChipPhase::Closing(c::ClosingStage::Run { .. }),
                ..
            } => true,
            c::ChipPhaseFacts { .. } => false,
        };
        if running && self.run.is_none() {
            let handle = handle(self.next);
            self.next = self.next.checked_add(1).expect("task handles fit");
            self.run = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Run { task: self.number, turn: None, offset: 0 }),
            }));
        } else if !running && let Some(handle) = self.run.take() {
            self.current_call = None;
            unwatch(handle, out);
        }
    }
    pub(crate) fn chat(&self, client: &Client, me: Option<&c::Me>) -> Option<Address> {
        let task = self.document(client)?;
        let template = jig_client::shapes::template(me?, task.project, jig_client::shapes::CHAT)?;
        if task.label == template.label { Some(Address::Chat { task: self.number }) } else { None }
    }
}
fn handle(number: u64) -> Handle {
    Handle { asker: Owner::Pages.asker(), number }
}
fn unwatch(handle: Handle, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Unwatch { handle }));
}
fn read(handle: Handle, read: c::Read, out: &mut Queue<Request>) {
    out.push(Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(read) }));
}
/// The hold's revision a chip says, if it is held.
pub(super) fn chip_hold(chip: &c::Chip) -> Option<u64> {
    match chip.details.as_ref() {
        c::ChipPhaseFacts { phase: c::ChipPhase::Held { revision, .. }, .. } => Some(*revision),
        c::ChipPhaseFacts { .. } => None,
    }
}
/// The hold's revision a held card's read task carries, if any.
fn hold_revision(page: Option<&c::ReadValue>) -> Option<u64> {
    let Some(c::ReadValue::Task(task)) = page else { return None };
    Some(task.hold.as_ref()?.revision().value)
}
fn latest(client: &Client, handle: Handle) -> Option<&Latest> {
    client.document(handle)?.latest
}

/// Pause and Resume use the same offered acts and hold revision on task pages and Team.
pub(super) fn control(task: &c::Task, kind: Sending, paused: bool) -> Option<c::Ask> {
    match kind {
        Sending::Stop if super::chat::allowed(task, c::ActKind::Stop) => {
            Some(c::Ask::Stop { project: task.project, task: task.chip.task })
        }
        Sending::Resume if paused && super::chat::allowed(task, c::ActKind::Release) => Some(c::Ask::Release {
            project: task.project,
            task: task.chip.task,
            revision: c::Revision { value: chip_hold(&task.chip)? },
            again: None,
        }),
        Sending::Leave | Sending::Message | Sending::Stop | Sending::Resume | Sending::Cancel | Sending::Amend => None,
    }
}
