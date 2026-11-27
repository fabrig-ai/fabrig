use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use skein_fake_checkout::Checkout;
use skein_fake_llm_domain as provider;
use skein_fake_llm_domain::api::{Capture, Rules, Script, Scripts, Seeds};
use skein_lib::{Duration, ReplyTo, Rng, Time, Token};
use skein_world::domain::{Key, Ledger, Schedule, Span, Stage, Trace};
use skein_world::streams::{Stream, Streams};
use smith_domain_session as agent;
use smith_domain_session::llm::{Block, Decoded, Descriptor, Endpoint, Failure, Prompt, Returned, Usage};

use smith_domain_tools::{self as tools, Authority, Directory, Done, Effect, Fault, Grants, Op};
use smith_tools_world::translate as io;

use crate::fixture;
use crate::shapes::{Reached, completion_tools, recorded_result};
use crate::tickets::{SERVED, Ticketed, Tickets};
use crate::translate;

mod jobs;
mod llm;
mod paths;
mod window_referee;

pub use jobs::{JobCall, check_job_call};
pub use paths::PathCall;
pub use smith_shapes::workspace::TOOLS;
pub use window_referee::{check_prompt_bound, check_prompt_boundary, check_prompt_extends};

/// What the fake opener says when it nudges a session on.
const NUDGE: &[u8] = b"You have not finished: carry on.";

/// The budget [`spec`] asks for: what the calm limits allow at most.
pub const BUDGET: agent::Budget = agent::Budget {
    turns: 16,
    input: 1 << 20,
    output: 1 << 20,
    cache_read: 1 << 20,
    cache_write: 1 << 20,
    time: Duration::from_secs(1800),
};

/// Checked credit envelope around the original retained-payload pressure.
/// The provider and a result batch own their credits successively, so the larger
/// envelope suffices. Owned results, including job and shell output, and concrete
/// delegated results are included; record arrays and caller/emitted copies remain independently
/// measured by the shared heap meter under scenario ceilings.
/// smith/docs/design/domain/tools.md, sections 4, 5 and 6; skein/docs/foundation/programming-model.md, section 6.3.
#[must_use]
pub fn reservation_envelope(limits: &agent::Limits) -> Option<u64> {
    let tools = &limits.tools;
    let search = u64::try_from(std::mem::size_of::<tools::Hit>())
        .ok()?
        .checked_mul(u64::from(tools.search_hits))?
        .checked_add(u64::from(tools.search_bytes))?;
    let edit = u64::try_from(std::mem::size_of::<u32>()).ok()?.checked_mul(u64::from(tools.match_lines))?;
    let shell = u64::from(tools.shell_head).checked_add(u64::from(tools.shell_tail))?;
    let largest =
        [u64::from(tools.read_bytes), tools.list_bytes, search, 0, edit, shell, limits.delegated_result_bytes]
            .into_iter()
            .max()?;
    let batch = largest.checked_mul(u64::from(limits.parallel_tools))?;
    Some(agent::completion_reserve(limits)?.max(batch))
}

/// Counts drawn uniformly from `min..=max`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Count {
    /// Inclusive minimum count required by this contract.
    pub min: u32,
    /// Inclusive maximum count allowed by this contract.
    pub max: u32,
}

/// The two observation points around the loop's reclaim, with no work between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reclaim {
    /// Retired entities remain present and newly resting batches are ineligible.
    Before,
    /// Retired entities are freed and newly resting batches become eligible.
    After,
}

/// Immutable seeded peer behavior and component limits for constructing this world.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Settings {
    /// Seeds the world, which seeds both domains.
    pub seed: u64,
    /// Immutable session limits for the component under test.
    pub agent: agent::Limits,
    /// Model deadlines supplied by the scripted opener in randomized worlds.
    pub progress: agent::Progress,
    /// Neutral fake-provider behavior for this world.
    pub provider: provider::Config,
    /// One-way latency between the agent and the provider.
    pub network: Span,
    /// How long io takes over an operation of the tools, a command's
    /// included.
    pub tool: Span,
    /// The chance, per mille, that io fails an operation of the tools.
    pub tool_errors: u32,
    /// Mount a second, read-only directory at `/ref` in each session's workspace.
    pub reference: bool,
    /// Give the shared menu's README body more draws to exercise dependent calls.
    pub rewrite_heavy: bool,
    /// How many times the opener nudges a session that yields before it
    /// closes it, drawn for each session.
    pub nudges: Count,
    /// How long the opener takes to answer a yield.
    pub think: Span,
    /// The chance, per mille, that the opener closes a session at a moment of
    /// its own, whatever the session is doing then.
    pub abandon: u32,
    /// When such a close comes, after the session opens.
    pub abandon_after: Span,
    /// The chance, per mille, that a cancel loses its race: the call, the
    /// tools' operation or the delegated call it was for ends of itself, and
    /// that is its terminal event. A cancel that wins is told after a network
    /// draw.
    pub cancels_lost: u32,
    /// The chance, per mille, that the opener sends a close twice, the second
    /// a network draw after the first.
    pub double_close: u32,
    /// The chance, per mille, that the opener serves its own tools, a finish
    /// and a lookup, to a session it opens.
    pub serve: u32,
    /// How long the opener takes to answer a call it serves.
    pub serving: Span,
    /// The grid every delivery is rounded up to, so that some come at the
    /// same instant; zero for none.
    pub granule: Duration,
}

impl Settings {
    /// A world where nothing goes wrong: no failures, answers well within
    /// every deadline, room for a few sessions, and an opener that closes a
    /// session when it first yields.
    #[must_use]
    pub const fn calm(seed: u64) -> Settings {
        Settings {
            progress: agent::Progress { head: Duration::from_secs(30), idle: Duration::from_secs(30) },
            seed,
            agent: agent::Limits {
                token_bytes: skein_llm::TOKEN_BYTES,
                sessions: 4,
                spend: 1,
                history_messages: 32,
                history_bytes: 1 << 20,
                budget: BUDGET,

                retries: 3,
                backoff_base: Duration::from_millis(200),
                backoff_max: Duration::from_secs(5),
                parallel_tools: 4,
                tools: TOOLS,
                ..smith_domain_session_fixtures::SMALL
            },
            provider: provider::Config {
                calls: 16,
                query_bytes: 1 << 20,
                script_bytes: 1 << 20,
                answer_bytes: 1 << 20,
                latency_min: Duration::from_millis(100),
                latency_max: Duration::from_millis(2000),
                overloaded: 0,
                rate_limited: 0,
                retry_after: Duration::from_secs(1),
                unavailable: 0,
                too_long: 0,
                unauthorized: 0,
                refused: 0,
                no_calls: 0,
                outside_choice: 0,
                answer_tokens: 1,
                calls_per_answer: 1,
                malformed: 0,
                tool_rounds: 2,
                cache_lifetime: Duration::from_secs(300),
                cache_entries: 8,
                strict: false,
                unscoped_reads: 0,
                conversations: 4,
                capture_answers: 0,
                capture_bytes: 0,
                accounts: 0,
                unserved: 0,
                cut: 0,
            },
            network: Span::millis(1, 20),
            tool: Span::millis(10, 500),
            tool_errors: 0,
            reference: false,
            rewrite_heavy: false,
            nudges: Count { min: 0, max: 0 },
            think: Span::millis(0, 50),
            abandon: 0,
            abandon_after: Span::millis(0, 10_000),
            cancels_lost: 0,
            double_close: 0,
            serve: 0,
            serving: Span::millis(10, 2_000),
            granule: Duration::ZERO,
        }
    }
}

/// A spec that grants every family of tools, with the calm budget, in one
/// writable repository at `/work`, also the working directory. The world
/// gives each session a repository of its own, seeded with the fixture, and
/// names it in the spec as io names its root.
#[must_use]
pub fn spec(prompt: &[u8]) -> agent::Spec {
    let repo = directory(b"host", b"work", Token::new(0), true);
    let grants = Grants { inspect: true, modify: true, shell: true };
    let authority = Authority { cwd: io::names(b"/work").into_vec().pop(), repos: Box::new([repo]), grants };
    agent::Spec {
        tell_parts: false,
        backend: smith_domain_session::record::Backend::Direct,
        account: Some(0),
        effort: None,
        final_turn: None,
        window: 65_536,
        line: 64_512,
        summary: b"Summarise the work so far.".as_slice().into(),
        preamble: b"Earlier window summary:".as_slice().into(),
        affinity: smith_domain_session::llm::Affinity { key: [0; 16], thread: 0 },
        protocol_allowance: 0,
        progress: agent::Progress { head: Duration::from_secs(30), idle: Duration::from_secs(30) },
        endpoint: Endpoint(0),
        model: b"fake-1"[..].into(),
        system: b"You are a coding agent."[..].into(),
        authority,
        delegated: Box::new([]),
        prompt: prompt.into(),
        output: 1024,
        budget: BUDGET,
    }
}

fn directory(host: &[u8], mount: &[u8], root: Token, writable: bool) -> Directory {
    let name = |text: &[u8]| tools::Name::new(text.into()).expect("fixture name");
    Directory { host: Box::new([name(host)]), mount: name(mount), root, writable }
}

/// What the world counted.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Stats {
    /// Calls the agent made.
    pub calls: u32,
    /// Calls that reached the provider.
    pub provider_calls: u32,
    /// Calls that ran out of time.
    pub timeouts: u32,
    /// Calls the agent cancelled.
    pub cancels: u32,
    /// Answers that arrived after their call had ended, and were dropped.
    pub late_answers: u32,
    /// Operations the tools asked of io; those a cancel ended; those that ran
    /// out of time; those io failed.
    pub ops: u32,
    pub op_cancels: u32,
    pub op_timeouts: u32,
    pub op_faults: u32,
    /// Job reads and stops dispatched to io, including cleanup reads.
    pub job_acts: u32,
    /// Completions with a tools-entrance refusal, suppressed write and subsequent read.
    pub refused_write_read: u32,
    /// Calls refused for a job number the session did not hold.
    pub unknown_jobs: u32,
    /// Yielding commands refused because all the session's job cells were occupied.
    pub full_jobs: u32,
    /// Sessions by the most jobs they held at once, from zero through the four-cell limit.
    pub held_peaks: [u32; 5],
    /// Results of calls to the tools that went back to the LLM, each checked
    /// against its call.
    pub results: u32,
    /// Tool calls the session answered as not run, as the LLM stopped
    /// before it could use them.
    pub not_run: u32,
    /// Times a session yielded.
    pub yields: u32,
    /// Messages the opener sent to yielded sessions.
    pub continues: u32,
    /// Closes the opener sent.
    pub closes: u32,
    /// Continues and closes that reached a session after it had ended.
    pub stale: u32,
    /// The most operations and delegated calls one session had in flight at
    /// once.
    pub most_parallel: u32,
    /// The most tool runs the sessions held at the end of an iteration,
    /// before its reclaim point.
    pub most_runs: u32,
    /// Cancels of calls and of operations that lost their race.
    pub cancels_lost: u32,
    pub op_cancels_lost: u32,
    /// What ended a call or an operation whose cancel lost: an answer, a
    /// failure (a deadline included), the operation's own end.
    pub answered_after_cancel: u32,
    pub failed_after_cancel: u32,
    pub done_after_cancel: u32,
    /// Closes that reached a session already closing.
    pub closed_while_closing: u32,
    /// Cancels and withdraws for operations and delegated calls that ended in
    /// the iteration they were sent.
    pub cancels_crossed: u32,
    /// Calls delegated to the opener; withdrawn, or whose answer won the race
    /// with the withdraw; answered as timed out by the opener itself.
    pub delegates: u32,
    pub withdraws: u32,
    pub withdraws_lost: u32,
    pub answered_after_withdraw: u32,
    pub delegate_timeouts: u32,
    /// Finishes the opener refused, and accepted.
    pub finishes_refused: u32,
    pub finishes_accepted: u32,
}

/// The facts the sessions told, by kind, as the loop drained them.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Told {
    /// Number of opted-in reasoning drops observed, without retained content.
    pub reasoning_dropped: u32,
    /// Count of opened facts drained from the component.
    pub opened: u32,
    /// Count of completions started facts drained from the component.
    pub completions_started: u32,
    /// Count of completions answered facts drained from the component.
    pub completions_answered: u32,
    /// Count of completions failed facts drained from the component.
    pub completions_failed: u32,
    /// Count of completions cancelled facts drained from the component.
    pub completions_cancelled: u32,
    /// Count of completions retried facts drained from the component.
    pub completions_retried: u32,
    /// What the session's tools told: kits opened and closed, calls started
    /// (those that asked io for something) and answered.
    pub kits_opened: u32,
    /// Count of kits closed facts drained from the component.
    pub kits_closed: u32,
    /// Count of tools started facts drained from the component.
    pub tools_started: u32,
    /// Count of tools answered facts drained from the component.
    pub tools_answered: u32,
    /// Job ends the session reported outside tool answers.
    pub jobs_ended: u32,
    /// Count of yielded facts drained from the component.
    pub yielded: u32,
    /// Count of used facts drained from the component.
    pub used: u32,
    /// Count of ended facts drained from the component.
    pub ended: u32,
    /// Count of delegates started facts drained from the component.
    pub delegates_started: u32,
    /// Count of delegates answered facts drained from the component.
    pub delegates_answered: u32,
    /// Count of delegates cancelled facts drained from the component.
    pub delegates_cancelled: u32,
    /// Tool calls the completions made, and those that could not be decoded.
    pub calls: u32,
    /// Count of invalid calls facts drained from the component.
    pub invalid_calls: u32,
}

/// A session as its opener saw it.
#[derive(Debug)]
pub struct Session {
    previous_prompt: Option<Prompt>,
    counting: crate::provider::Counting,
    /// Every rendered request, including those the guard refused unsent.
    pub measures: Vec<crate::provider::MeasuredPrompt>,
    line: u64,
    compacting: Option<u32>,
    opening: Option<Box<[agent::llm::Message]>>,
    window: u32,
    ordinary: Option<(Usage, usize)>,
    summary: Option<(u64, u64)>,
    summary_instruction: Box<[u8]>,
    /// The session's name, once it opened.
    pub session: Option<Token>,
    /// What it was opened with: its budget, and the most tokens an answer
    /// may take.
    pub budget: agent::Budget,
    /// Output-token cap supplied to each generated completion.
    pub max_tokens: u32,
    /// Each time it yielded: why, and what the LLM said.
    pub yields: Vec<(agent::Yield, Box<[u8]>)>,
    /// What its `Used` added up to: completions, and their tokens.
    pub turns: u32,
    /// Accepted token usage observed for this fixture session or its terminal.
    pub usage: Usage,
    /// Concrete history emitted in sequence, with historical calls and results.
    pub records: Vec<agent::record::Turn>,
    /// Exact accepted completion usage, independently matched against each emitted Turn.
    accepted: Vec<Usage>,
    /// Provider payload awaiting the corresponding Used acceptance.
    received: Option<Received>,
    /// Original accepted provider payloads before historical call conversion.
    accepted_content: Vec<Box<[Block]>>,
    /// Provider stop determines whether calls executed or remained a yielded tail.
    accepted_stop: Vec<agent::llm::Stop>,
    /// Provider closing witnesses, kept independently of tool timeout race bookkeeping.
    provider_closing: ProviderClosing,
    /// Late Closing winners settle every call as `NotRun` for any provider stop.
    accepted_late: Vec<bool>,
    /// Delegated terminal values, indexed by their immutable origin.
    delegated_results: BTreeMap<(u32, u32), Returned>,
    /// Restored transcript's final sequence and job number, or zero for a fresh session.
    opening_sequence: u32,
    opening_job: u32,
    /// Highest job number the machine has yielded to this kit.
    issued_job: u32,
    /// Highest job number seen in this session's recorded shell answers.
    highest_job: u32,
    accepted_job_calls: Vec<Vec<JobCall>>,
    /// Classified job calls, in the order of their recorded turns.
    pub job_calls: Vec<JobCall>,
    authority: Authority,
    removed_paths: BTreeSet<Vec<u8>>,
    accepted_path_calls: Vec<Vec<PathCall>>,
    /// Classified workspace references, in the order of their recorded turns.
    pub path_calls: Vec<PathCall>,
    held_peak: usize,
    /// Job ends reported by an answer or a terminal fact, never both.
    job_ends: BTreeSet<u32>,
    /// Exact inclusive and own price reports observed before terminal handoff.
    priced: u32,
    /// When its time budget runs out, once it opened.
    expires: Option<Time>,
    /// How it ended, once it has.
    pub ended: Option<Ended>,
    /// Nudges the opener has left to give.
    nudges: u32,
    /// Whether the opener closes it at a moment of its own.
    abandon: bool,
    /// It yielded, and the opener has not continued it yet.
    waiting: bool,
    /// The opener has closed it.
    closed: bool,
    /// Finishes it has called.
    finishes: u32,
}

/// Provider evidence taken at arrival, before the completion can start any calls.
#[derive(Debug)]
struct Received {
    content: Box<[Block]>,
    stop: agent::llm::Stop,
    late: bool,
    job_calls: Vec<JobCall>,
    path_calls: Vec<PathCall>,
}

/// Provider lifecycle evidence; ordinary IO/withdraw race bookkeeping
/// cannot turn a yielded provider completion into a late Closing winner.
#[derive(Default, Debug)]
struct ProviderClosing {
    close_seen: bool,
    cancel_seen: bool,
}

/// How a session ended.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ended {
    /// The single settled session terminal, or none while work remains.
    pub end: agent::End,
    /// Concrete transcript turns emitted by the session, in order.
    pub turns: u32,
    /// Accepted token usage observed for this fixture session or its terminal.
    pub usage: Usage,
}

/// Something on its way, delivered at its time.
enum Delivery {
    /// The opener opens a session.
    Open {
        opener: u64,
        opening: Box<agent::record::Opening>,
    },
    /// The opener continues a yielded session.
    Continue {
        opener: u64,
        content: Box<[u8]>,
    },
    /// The opener closes a session.
    Close {
        opener: u64,
    },
    /// A call arrives at the provider.
    Query {
        call: u64,
        query: provider::api::Query,
    },
    /// The provider's answer arrives back at the agent's side.
    Answer {
        call: u64,
        result: Result<provider::api::Answer, provider::api::Error>,
    },
    /// The agent's side gives up on a call.
    Deadline {
        call: u64,
        phase: smith_domain_session::llm::CompletionPhase,
    },
    /// A cancel that won its race is told.
    Cancelled {
        owner: Token,
    },
    AnswerCancelled {
        owner: Token,
    },
    /// The opener's answer to a delegated call arrives.
    Answered {
        owner: Token,
        text: Box<[u8]>,
        error: bool,
    },
    /// An operation of the tools ends.
    Ran {
        owner: Token,
    },
    /// A held command ends, or its pending read is due, at the unrounded time.
    Job {
        owner: Token,
        at: Time,
    },
}

/// A delegated call in flight, as the opener keeps it.
struct Running {
    /// The answer's delivery, withdrawn if the call is withdrawn.
    delivery: Key,
    /// Immutable position for proving exact concrete delegated result retention.
    origin: agent::record::Origin,
    /// The session that started it.
    session: Token,
    effect: Effect,
    /// A finish the opener accepts, after which it closes the session.
    accepts: bool,
}

/// An operation of the tools in flight, as io keeps it.
struct Pending {
    /// Its end's delivery, moved up when a cancel wins the race.
    delivery: Key,
    /// The session whose tools asked for it, by the repository it is in.
    session: Token,
    /// A store or a command, which only a call that writes asks for.
    writes: bool,
    work: Work,
}

/// What an operation does when it ends.
enum Work {
    /// A file operation, run on the checkout then.
    File(Op),
    /// A command started, finished then.
    Command(io::Started),
    /// A command with a successful spawn whose yield precedes its end and deadline.
    Yielding(jobs::Command),
    /// Nothing more: it ends so (it failed to start, timed out, or was
    /// cancelled).
    Ending(Done),
}

/// A call of the agent in flight, as its protocol layer would keep it.
struct Call {
    owner: Token,
    /// The deadline's delivery, withdrawn when the call ends first.
    deadline: Key,
    /// The session's opener, and the tools it served in the query.
    opener: u64,
    served: Box<[Descriptor]>,
    input: u64,
}

/// Real domain plus scripted typed peers, pending terminals and external observations for this component story.
pub struct World {
    now: Time,
    rng: Rng,
    settings: Settings,
    rendered_result: u32,
    decoder: translate::Decoder,

    agent: agent::Domain,
    agent_stage: Stage<agent::Limits, agent::Event, agent::Request>,

    provider: provider::Domain,
    provider_stage: Stage<provider::Config, provider::Event, provider::Request>,

    /// Deliveries in flight, whose count names openers and calls too.
    wire: Schedule<Delivery>,
    /// The agent's calls in flight, and the call each session has in flight.
    calls: Ledger<u64, Call>,
    calling: BTreeMap<Token, u64>,
    /// The delegated calls in flight, by their tokens; and the session of
    /// each the agent has started or has yet to hear the end of.
    tools: Ledger<Token, Running>,
    runs: BTreeMap<Token, Token>,
    /// The checkout the sessions' tools work on, a repository for each
    /// session, and the opener of each by io's name for its root.
    checkout: Checkout,
    roots: BTreeMap<u64, u64>,
    /// The tools' operations in flight, by their tokens, and every token an
    /// operation has had (a call's, for each operation it asks for).
    ops: Ledger<Token, Pending>,
    /// Commands past their yield, by the same token as their spawn.
    jobs: BTreeMap<Token, jobs::Job>,
    owners: BTreeSet<Token>,
    /// The tools of the calls each session's tools started, as they told,
    /// until their first operation; and whether the call of each operation's
    /// token writes (an edit loads before it stores).
    starting: BTreeMap<Token, VecDeque<tools::Tool>>,
    writing: BTreeMap<Token, bool>,
    /// How many messages of each session's transcript have had their results
    /// counted.
    counted: BTreeMap<Token, usize>,
    /// The sessions whose call, and the delegated calls and operations, whose
    /// cancel lost its race; and the sessions the agent is closing, with a
    /// cancel sent and no end yet.
    cancel_lost: BTreeSet<Token>,
    run_cancel_lost: BTreeSet<Token>,
    op_cancel_lost: BTreeSet<Token>,
    closing: BTreeSet<Token>,
    /// The opener's tickets, as the top level would keep them.
    tickets: Tickets,
    /// Calls the provider has not answered yet.
    serving: Ledger<u64, ()>,
    /// The sessions opened, by the opener's name for each, and the opener's
    /// name for each session by the session's own while it lives.
    sessions: BTreeMap<u64, Session>,
    openers: BTreeMap<Token, u64>,

    finding: Option<String>,
    stats: Stats,
    told: Told,
    trace: Option<Trace>,
}

impl World {
    /// Constructs empty bounded state under the supplied immutable limits; no IO or clocks are consulted.
    #[must_use]
    pub fn new(settings: Settings) -> World {
        Self::with_scripts(settings, Box::new([]))
    }

    /// Construct the same world with explicit application tool turns for focused stories.
    #[must_use]
    pub fn with_scripts(settings: Settings, scripts: Box<[Script]>) -> World {
        Self::with_provider(settings, scripts, Capture::none())
    }

    /// Replays the fake model's captured answers with this world's own faults and timing.
    #[must_use]
    pub fn with_replay(settings: Settings, replay: Capture) -> World {
        Self::with_provider(settings, Box::new([]), replay)
    }

    /// The fake model's retained answers and the classes the world gave its draws.
    #[must_use]
    pub fn capture(&self) -> Capture {
        self.provider.capture()
    }

    fn with_provider(mut settings: Settings, scripts: Box<[Script]>, replay: Capture) -> World {
        assert!(agent::valid(&settings.agent), "the shell refuses invalid limits");
        let mut rng = Rng::new(settings.seed);
        let agent = agent::Domain::new(&settings.agent, rng.next_u64());
        // World delivery, tools and opener draws begin at the third draw.
        let _ = rng.next_u64();
        settings.provider.conversations = settings.provider.conversations.max(settings.agent.sessions);
        let streams = Streams::new(settings.seed);
        let seeds = Seeds::Parts {
            agent: streams.seed_of(Stream::Parties, "agent/model"),
            faults: streams.seed_of(Stream::Faults, "llm"),
            scheduling: streams.seed_of(Stream::Scheduling, "llm"),
        };
        let provider = provider::Domain::drawn(
            &settings.provider,
            seeds,
            Scripts { cued: scripts, asked: Box::new([]), otherwise: Box::new([]) },
            crate::provider::weighted_menu(settings.rewrite_heavy),
            Rules { cued: Box::new([]), asked: Box::new([]) },
            replay,
        )
        .expect("application menu, scripts and replay obey provider admission");
        let mut checkout = Checkout::new();
        fixture::script(&mut checkout);
        World {
            now: Time::ZERO,
            rng,
            settings,
            rendered_result: translate::render_bound(&settings.agent),
            decoder: translate::Decoder::new(&settings.agent.tools),
            agent,
            agent_stage: Stage::uncapped(settings.agent),
            provider,
            provider_stage: Stage::uncapped(settings.provider),
            wire: Schedule::new(),
            calls: Ledger::new("call to the provider"),
            calling: BTreeMap::new(),
            tools: Ledger::new("delegated call"),
            runs: BTreeMap::new(),
            checkout,
            roots: BTreeMap::new(),
            ops: Ledger::new("operation"),
            jobs: BTreeMap::new(),
            owners: BTreeSet::new(),
            starting: BTreeMap::new(),
            writing: BTreeMap::new(),
            counted: BTreeMap::new(),
            cancel_lost: BTreeSet::new(),
            run_cancel_lost: BTreeSet::new(),
            op_cancel_lost: BTreeSet::new(),
            closing: BTreeSet::new(),
            tickets: Tickets::default(),
            serving: Ledger::new("call the provider serves"),
            sessions: BTreeMap::new(),
            openers: BTreeMap::new(),
            finding: None,
            stats: Stats::default(),
            told: Told::default(),
            trace: Some(Trace::default()),
        }
    }

    /// Current injected monotonic time in this world.
    #[must_use]
    pub fn now(&self) -> Time {
        self.now
    }

    /// Observed boundary counters.
    #[must_use]
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Every native fact the sessions told, counted by kind.
    #[must_use]
    pub fn told(&self) -> Told {
        self.told
    }

    /// What crossed between the domains and the world, in order, with times.
    #[must_use]
    pub fn trace(&self) -> &[String] {
        self.trace.as_ref().map_or(&[], Trace::lines)
    }

    /// Keep the referee and recorded turns, but do not format a second textual trace.
    pub fn omit_trace(&mut self) {
        self.trace = None;
    }

    /// Has the opener open a session for `spec` at `at`. Returns the opener's
    /// name for it.
    pub fn submit(&mut self, at: Time, spec: agent::Spec) -> u64 {
        self.submit_open(at, spec, None)
    }

    /// Has the opener resume a settled transcript in a new session's kit.
    pub fn submit_restored(&mut self, at: Time, spec: agent::Spec, transcript: agent::record::Transcript) -> u64 {
        self.submit_open(at, spec, Some(transcript))
    }

    fn submit_open(&mut self, at: Time, mut spec: agent::Spec, transcript: Option<agent::record::Transcript>) -> u64 {
        let opener = self.wire.name();
        if self.rng.chance(self.settings.serve) {
            let mut served = Vec::new();
            for (name, effect, schema) in SERVED {
                let ticket = self.tickets.issue(opener, Ticketed::Tool { name, effect, schema });
                served.push(Descriptor { ticket, effect });
            }
            spec.delegated = served.into();
        }
        let opening = Box::new(agent::record::Opening {
            window_llm: None,
            spec,
            dialect: 2,
            prices: agent::record::Prices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
            budget: 1,
            transcript,
            answered: Box::default(),
            cut: None,
            withheld: Some(b"wait".as_slice().into()),
        });
        self.schedule(at, Delivery::Open { opener, opening });
        opener
    }

    /// The session the opener named `opener`, once the open has been sent.
    #[must_use]
    pub fn session(&self, opener: u64) -> &Session {
        self.sessions.get(&opener).expect("the session was submitted and its open sent")
    }

    /// Every session opened so far, by the opener's name for it.
    pub fn sessions(&self) -> impl Iterator<Item = (u64, &Session)> {
        self.sessions.iter().map(|(opener, session)| (*opener, session))
    }

    /// The completion shapes of every session's recorded turns, in opener and turn order.
    #[must_use]
    pub fn reached(&self) -> Reached {
        let mut reached = Reached::new();
        for session in self.sessions.values() {
            let mut jobs = session.job_calls.iter().peekable();
            let mut paths = session.path_calls.iter().peekable();
            for (turn, content) in session.records.iter().zip(&session.accepted_content) {
                reached.recorded(turn, completion_tools(content), |position| {
                    let paths = paths.next_if(|call| call.sequence == turn.sequence && call.position == position);
                    let job = jobs.next_if(|call| call.sequence == turn.sequence && call.position == position);
                    [paths.and_then(|call| call.path), paths.and_then(|call| call.offset), job.map(|call| call.class)]
                });
            }
        }
        reached
    }

    /// Runs until nothing is left to happen, then checks the invariants of a
    /// settled world. Panics if it takes more than `iterations`.
    pub fn run(&mut self, iterations: u32) {
        self.run_observed(iterations, |_, _| {});
    }

    /// Runs the same checks, sharing only pure argument decodings across a sweep's worlds.
    pub fn run_with_decodings(&mut self, iterations: u32, memo: &mut translate::Decodings) {
        self.decoder.memo = Some(std::mem::take(memo));
        self.run(iterations);
        *memo = self.decoder.memo.take().expect("the run owns the sweep's memo");
    }

    /// Runs the same loop with a scenario observing both sides of each reclaim point.
    pub fn run_observed(&mut self, iterations: u32, mut observe: impl FnMut(&mut Self, Reclaim)) {
        for _ in 0..iterations {
            self.iterate(&mut observe);
            if let Some(why) = &self.finding {
                panic!("{why}");
            }
            if self.has_work_now() {
                continue;
            }
            let Some(next) = self.next_time() else {
                self.assert_settled();
                return;
            };
            assert!(next > self.now, "time moves forward");
            self.now = next;
        }
        panic!("the world did not settle in {iterations} iterations");
    }

    /// One iteration of the loop, as the shell would run it.
    fn iterate(&mut self, observe: &mut impl FnMut(&mut Self, Reclaim)) {
        self.agent_stage.tick(self.now);
        self.provider_stage.tick(self.now);
        self.deliver();

        // The agent resumes what is ready, then takes its events, then fires
        // its alarms, before draining observations and dispatching requests.
        while self.agent.is_ready() {
            self.log("agent ready");
            agent::resume(&mut self.agent, &self.agent_stage.env, &mut self.agent_stage.out);
        }
        while let Some(event) = self.agent_stage.next_event() {
            self.log(EventLine(&event));
            if let agent::Event::Close { session } = &event
                && let Some(opener) = self.openers.get(session)
            {
                self.sessions.get_mut(opener).expect("the addressed live opener").provider_closing.close_seen = true;
            }
            if let agent::Event::Completed { owner, completion } = &event {
                let opener = self.openers[owner];
                let job_calls = self.drawn_jobs(opener, &completion.content);
                let path_calls = self.drawn_paths(opener, &completion.content);
                let session = self.sessions.get_mut(&opener).expect("a provider terminal names its opener");
                let late = session.provider_closing.close_seen || session.provider_closing.cancel_seen;
                session.received = Some(Received {
                    content: completion.content.clone(),
                    stop: completion.stop,
                    late,
                    job_calls,
                    path_calls,
                });
            }
            let ended = ended_run(&event);
            agent::step(&mut self.agent, &self.agent_stage.env, event, &mut self.agent_stage.out);
            if let Some(run) = ended {
                self.runs.remove(&run);
            }
        }
        while self.agent.is_due(self.now) {
            self.log("agent alarm");
            agent::fire(&mut self.agent, &self.agent_stage.env, &mut self.agent_stage.out);
        }
        // The facts, drained as the shell would write them out.
        while let Some(fact) = self.agent.pop_fact() {
            self.tell(&fact);
        }
        while let Some(event) = self.provider_stage.next_event() {
            provider::step(&mut self.provider, &self.provider_stage.env, event, &mut self.provider_stage.out);
        }
        while self.provider.is_due(self.now) {
            provider::fire(&mut self.provider, &self.provider_stage.env, &mut self.provider_stage.out);
        }

        // What the steps asked for, submitted at the end of the iteration.
        while let Some(request) = self.agent_stage.out.pop() {
            self.agent_request(request);
            if self.finding.is_some() {
                return;
            }
        }
        while let Some(request) = self.provider_stage.out.pop() {
            self.provider_request(request);
        }

        // What the iteration held at its most, before the reclaim point
        // frees what ended in it.
        assert!(self.agent.sessions() <= self.settings.agent.sessions, "sessions stay within their slots");
        let runs = self.settings.agent.sessions * self.settings.agent.parallel_tools * 2;
        assert!(self.agent.runs() <= runs, "runs stay within two batches a session");
        assert!(self.agent.kits() <= self.settings.agent.sessions, "a kit at most for each session");
        assert!(self.provider.calls() <= self.settings.provider.calls, "calls stay within their slots");
        self.stats.most_runs = self.stats.most_runs.max(self.agent.runs());
        observe(self, Reclaim::Before);
        self.agent.reclaim();
        self.provider.reclaim();
        observe(self, Reclaim::After);
    }

    /// The domain's boundary counters and readiness, for loop invariant observations.
    #[must_use]
    pub fn domain(&self) -> &agent::Domain {
        &self.agent
    }

    /// Schedules the opener's close through the existing delivery queue at the current time.
    pub fn close(&mut self, opener: u64) {
        self.schedule(self.now, Delivery::Close { opener });
    }

    /// File operations still owed to an opener's session, observed at the machine boundary.
    #[must_use]
    pub fn pending_io(&self, opener: u64) -> Vec<(Token, &Op)> {
        let session = self.sessions[&opener].session;
        self.ops
            .keys()
            .filter_map(|&owner| {
                let pending = self.ops.get(owner).expect("live operation");
                match &pending.work {
                    Work::File(op) if Some(pending.session) == session => Some((owner, op)),
                    Work::File(_) | Work::Command(_) | Work::Yielding(_) | Work::Ending(_) => None,
                }
            })
            .collect()
    }

    /// Delegated calls still owed to an opener's session, in provider positions.
    #[must_use]
    pub fn pending_delegates(&self, opener: u64) -> Vec<(Token, agent::record::Origin)> {
        let session = self.sessions[&opener].session;
        self.tools
            .keys()
            .filter_map(|&owner| {
                let pending = self.tools.get(owner).expect("live delegated call");
                (Some(pending.session) == session).then_some((owner, pending.origin))
            })
            .collect()
    }

    /// Moves a machine terminal through the existing schedule, retaining its work and reply right.
    /// Scenarios keep the chosen time before the request's deadline.
    pub fn deliver_io_at(&mut self, owner: Token, at: Time) {
        assert!(at > self.now, "the machine terminal arrives in a later iteration");
        let key = self.ops.get(owner).expect("pending operation").delivery;
        let delivery = self.wire.withdraw(key).expect("pending machine terminal");
        let key = self.schedule(at, delivery);
        self.ops.get_mut(owner).expect("pending operation").delivery = key;
    }

    fn agent_request(&mut self, request: agent::Request) {
        self.log(RequestLine(&request));
        match request {
            agent::Request::Part { .. } => panic!("parts not requested"),
            agent::Request::Turn { opener, turn } => self.recorded(opener.raw(), turn),
            agent::Request::Priced { opener, spent, own_spent } => {
                let session = self.sessions.get_mut(&opener.raw()).expect("a price names its opener");
                assert!(session.ended.is_none(), "prices precede the terminal");
                assert_eq!(
                    (spent, own_spent),
                    (0, 0),
                    "zero-rate provider and zero-bill fake children have exact zero inclusive and own prices"
                );
                session.priced += 1;
            }
            agent::Request::JobsCancelled { .. }
            | agent::Request::Yield { .. }
            | agent::Request::RateLimitWait { .. }
            | agent::Request::RateLimitWaitDone { .. }
            | agent::Request::RateLimitOutlastsSession { .. } => {}
            agent::Request::Opened { opener, session } => self.opened(opener.raw(), session),
            agent::Request::Yielded { opener, stop, text } => self.yielded(opener.raw(), stop, text),
            agent::Request::Used { opener, usage } => {
                self.used(opener.raw(), usage);
            }
            agent::Request::Ended { opener, end, turns, usage, .. } => {
                self.ended(opener.raw(), Ended { end, turns, usage });
            }
            agent::Request::Complete { owner, prompt, deadlines, .. } => {
                self.complete(owner, prompt, deadlines);
            }
            agent::Request::Cancel { owner } => {
                let opener = self.openers[&owner];
                self.sessions
                    .get_mut(&opener)
                    .expect("provider cancellation names a live session")
                    .provider_closing
                    .cancel_seen = true;
                self.closing.insert(owner);
                // A call that has already ended has its terminal event on the
                // way: the cancel lost the race and changes nothing. One still
                // in flight may end of itself all the same.
                if let Some(&call) = self.calling.get(&owner) {
                    if self.rng.chance(self.settings.cancels_lost) {
                        self.cancel_lost.insert(owner);
                        self.stats.cancels_lost += 1;
                    } else {
                        self.end_call(call);
                        self.send(Delivery::Cancelled { owner });
                        self.stats.cancels += 1;
                    }
                }
            }
            agent::Request::Io { owner, op, deadline } => self.start_op(owner, op, deadline),
            agent::Request::Job { owner, act } => self.job_act(owner, act),
            agent::Request::CancelIo { owner } => self.cancel_op(owner),
            agent::Request::Delegate { owner, opener, call, deadline, origin } => {
                let session = self.sessions.get(&opener.raw()).and_then(|session| session.session);
                let session = session.expect("a session delegates once it has opened");
                assert!(self.runs.insert(owner, session).is_none(), "each delegated call has a token of its own");
                self.serve(owner, opener.raw(), call, deadline, origin);
            }
            agent::Request::Withdraw { owner } => {
                let Some(&session) = self.runs.get(&owner) else {
                    self.stats.cancels_crossed += 1;
                    return;
                };
                self.closing.insert(session);
                if self.tools.contains(owner) && self.rng.chance(self.settings.cancels_lost) {
                    self.run_cancel_lost.insert(owner);
                    self.stats.withdraws_lost += 1;
                } else if let Some(Running { delivery, origin, .. }) = self.tools.take(owner) {
                    let opener = self.openers[&session];
                    self.sessions
                        .get_mut(&opener)
                        .expect("delegation belongs to its opener")
                        .delegated_results
                        .insert((origin.sequence, origin.position), Returned::Withdrawn);
                    self.wire.withdraw(delivery).expect("a served call in flight has its answer on the way");
                    self.send(Delivery::AnswerCancelled { owner });
                    self.stats.withdraws += 1;
                }
            }
        }
    }

    /// The opener learns its session's name, and plans a close of its own if
    /// it is to abandon the session.
    fn opened(&mut self, opener: u64, name: Token) {
        let session = self.sessions.get_mut(&opener).expect("a session opens for an open that was sent");
        assert!(session.session.is_none() && session.ended.is_none(), "a session opens once, before it ends");
        session.session = Some(name);
        session.expires = Some(self.now.saturating_add(session.budget.time));
        self.openers.insert(name, opener);
        if session.abandon {
            let at = self.now.saturating_add(self.draw(self.settings.abandon_after));
            self.schedule(at, Delivery::Close { opener });
        }
    }

    /// The opener answers a yield: it nudges the session on while it has
    /// nudges left, and closes it after.
    fn yielded(&mut self, opener: u64, stop: agent::Yield, text: Box<[u8]>) {
        let session = self.sessions.get_mut(&opener).expect("a session yields to its opener");
        let name = session.session.expect("a session yields once it has opened");
        assert!(session.ended.is_none(), "a session yields before it ends");
        assert!(!session.waiting, "a session yields once for each message");
        session.yields.push((stop, text));
        self.stats.yields += 1;
        assert!(self.idle(name), "a session yields with nothing in flight");
        let session = self.sessions.get_mut(&opener).expect("looked up above");
        if session.closed {
            // A close of the opener's own crossed the yield.
            return;
        }
        let at = self.now.saturating_add(self.draw(self.settings.think));
        let session = self.sessions.get_mut(&opener).expect("looked up above");
        if session.nudges > 0 {
            session.nudges -= 1;
            session.waiting = true;
            self.schedule(at, Delivery::Continue { opener, content: NUDGE.into() });
        } else {
            self.schedule(at, Delivery::Close { opener });
        }
    }

    /// A completion came back: the opener adds up what it used.
    fn used(&mut self, opener: u64, usage: Usage) {
        let session = self.sessions.get_mut(&opener).expect("a session reports to its opener");
        assert!(session.session.is_some() && session.ended.is_none(), "a session uses tokens while it lives");
        session.turns += 1;
        session.usage = add_usage(session.usage, usage);
        session.accepted.push(usage);
        let Received { content, stop, late, job_calls, path_calls } =
            session.received.take().expect("usage accepts a provider payload");
        session.accepted_job_calls.push(job_calls);
        session.accepted_path_calls.push(path_calls);
        session.accepted_content.push(content);
        session.accepted_stop.push(stop);
        session.accepted_late.push(late);
        if session.compacting.is_none() {
            session.ordinary =
                Some((usage, session.previous_prompt.as_ref().expect("accepted request").messages.len() + 1));
            session.summary = None;
        }
    }

    /// Check every concrete handoff against accepted usage and its historical representation.
    #[expect(
        clippy::too_many_lines,
        reason = "one ordered walk pairs provider evidence with recorded calls and results"
    )]
    fn recorded(&mut self, opener: u64, turn: agent::record::Turn) {
        let session = self.sessions.get_mut(&opener).expect("a Turn names its opener");
        assert!(session.ended.is_none(), "concrete history precedes terminal handoff");
        assert_eq!((turn.version, turn.endpoint, turn.dialect), (agent::record::VERSION, Some(Endpoint(0)), 2));
        let index = session.records.len();
        assert_eq!(turn.sequence, session.opening_sequence + u32::try_from(index + 1).expect("bounded sequence"));
        assert_eq!(turn.usage, session.accepted[index], "each Turn records its accepted provider usage");
        assert_eq!(turn.spent, 0, "zero-price exact bill");
        let mut job_calls = std::mem::take(&mut session.accepted_job_calls[index]);
        for job_call in &mut job_calls {
            job_call.sequence = turn.sequence;
        }
        let mut path_calls = std::mem::take(&mut session.accepted_path_calls[index]);
        for path_call in &mut path_calls {
            path_call.sequence = turn.sequence;
        }
        if matches!(turn.kind, agent::record::Kind::Window(_)) {
            window_referee::recorded_window(session, &turn, index);
            session.job_calls.extend(job_calls);
            session.path_calls.extend(path_calls);
            session.records.push(turn);
            return;
        }
        let assistant_index = turn
            .messages
            .iter()
            .position(|message| message.role == agent::llm::Role::Assistant)
            .expect("every settled Turn contains the assistant payload");
        let assistant = &turn.messages[assistant_index];
        assert_resumed_tail(session, index, &turn.messages[..assistant_index]);
        let original = &session.accepted_content[index];
        let mut shape = jobs::RefusalShape::default();
        let mut job_calls = job_calls.into_iter().peekable();
        let mut path_calls = path_calls.into_iter().peekable();
        assert_eq!(assistant.content.len(), original.len(), "every provider block retains its position");
        for (position, (block, original)) in assistant.content.iter().zip(original.iter()).enumerate() {
            match original {
                Block::ToolCall(smith_domain_session::llm::ToolCall { id, name, input, replay, call }) => {
                    assert_eq!(
                        *block,
                        Block::ToolCall(smith_domain_session::llm::ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                            replay: replay.clone(),
                            call: Decoded::Historical,
                        }),
                        "concrete calls preserve exact provider bytes and shed only live authority"
                    );
                    let result = recorded_result(&turn.messages[assistant_index + 1..], id);
                    if let Some(mut path_call) = path_calls.next_if(|call| call.position == position) {
                        path_call.answer = crate::shapes::answer(result);
                        session.path_calls.push(path_call);
                    }
                    if job_calls.peek().is_some_and(|job_call| job_call.position == position) {
                        let mut job_call = job_calls.next().expect("the recorded position is present");
                        job_call.answer = result.cloned();
                        if let Err(finding) = check_job_call(&job_call) {
                            self.finding = Some(format!("session {opener}: {finding}"));
                            return;
                        }
                        session.job_calls.push(job_call);
                    }
                    if session.accepted_stop[index] != agent::llm::Stop::ToolUse {
                        if session.accepted_late[index] {
                            assert_eq!(
                                result,
                                Some(&Returned::NotRun { why: smith_domain_session::llm::NotRun::Stopped }),
                                "late Closing winners settle calls without effects"
                            );
                        } else {
                            assert!(result.is_none(), "a yielded call retains its unanswered tail until a Continue");
                        }
                        continue;
                    }
                    let result = result.expect("every callable ToolUse block has its settled result");
                    shape.observe(call, result, &mut self.stats, self.settings.agent.tools.jobs);
                    match call {
                        Decoded::Delegated { .. } => {
                            let position = u32::try_from(position).expect("bounded provider position");
                            if let Some(expected) = session.delegated_results.get(&(turn.sequence, position)) {
                                assert_eq!(
                                    result, expected,
                                    "exact delegated text/error or withdrawal is retained once"
                                );
                            } else {
                                unstarted_delegate(result, position);
                            }
                        }
                        Decoded::Owned { call } => match result {
                            Returned::Owned { outcome } => {
                                assert!(fixture::fits(call, outcome), "result belongs to its call");
                                jobs::observe_answer(&mut session.highest_job, &mut session.job_ends, outcome);
                            }
                            // The shape referee checked any tools-entrance refusal against this call.
                            Returned::NotRun { .. } | Returned::Invalid { .. } => {}
                            Returned::Text { .. } | Returned::Withdrawn => {
                                panic!("an owned call retains its owned result or never ran")
                            }
                        },
                        Decoded::Invalid { problem, .. } => match result {
                            Returned::Invalid { problem: seen } => assert_eq!(seen, problem),
                            Returned::NotRun { .. } => {}
                            Returned::Text { .. } | Returned::Owned { .. } | Returned::Withdrawn => {
                                panic!("an invalid call retains its exact problem or never ran")
                            }
                        },
                        Decoded::Builtin { .. } | Decoded::Historical => {
                            panic!("the provider must classify every live call")
                        }
                    }
                }
                Block::Opaque { .. } | Block::Text { .. } | Block::Refusal { .. } | Block::ToolResult { .. } => {
                    assert_eq!(block, original, "concrete payloads preserve provider bytes");
                }
            }
        }
        assert_eq!(turn.jobs, session.highest_job, "a turn records the highest job number announced");
        session.records.push(turn);
    }

    /// The session that owns `owner` starts a completion: it must have turns,
    /// input and output tokens and time left, no tokens past their budget,
    /// and an answer that may take no more than the output budget left.
    fn affordable(&self, owner: Token, prompt: &Prompt) {
        let opener = self.openers.get(&owner).expect("a session calls the LLM while it lives");
        let session = &self.sessions[opener];
        let (budget, usage) = (&session.budget, &session.usage);
        assert!(session.turns < budget.turns, "session {opener} starts no turn past its budget");
        assert!(usage.input_tokens.unwrap_or(0) < budget.input, "session {opener} has input tokens left");
        assert!(usage.output_tokens.unwrap_or(0) < budget.output, "session {opener} has output tokens left");
        assert!(
            usage.cache_read_tokens.unwrap_or(0) <= budget.cache_read,
            "session {opener} is within its cache reads"
        );
        assert!(
            usage.cache_write_tokens.unwrap_or(0) <= budget.cache_write,
            "session {opener} is within its cache writes"
        );
        assert!(Some(self.now) < session.expires, "session {opener} has time left");
        let messages = u32::try_from(prompt.messages.len()).expect("a small transcript");
        assert!(
            match prompt.purpose {
                agent::llm::Purpose::Ordinary { .. } => messages < self.settings.agent.history_messages,
                agent::llm::Purpose::Compaction { .. } => messages <= self.settings.agent.history_messages,
            },
            "session {opener}'s transcript has room for the answer"
        );
        let left = budget.output - usage.output_tokens.unwrap_or(0);
        let most = u32::try_from(left).unwrap_or(u32::MAX).min(session.max_tokens);
        assert_eq!(prompt.max_tokens, most, "session {opener}'s answer takes no more than the output budget left");
    }

    fn ended(&mut self, opener: u64, ended: Ended) {
        let session = self.sessions.get(&opener).expect("a session ends for an open that was sent");
        assert!(session.ended.is_none(), "a session ends once");
        match ended.end {
            agent::End::TranscriptRefused { .. } | agent::End::PriceOverflow | agent::End::UsageOverflow => {
                unreachable!("scripted restorations are valid; bounded usage and zero prices cannot overflow")
            }
            agent::End::Busy | agent::End::Invalid => {
                assert!(session.session.is_none(), "a session refused at the entrance never opened");
            }
            agent::End::Closed
            | agent::End::NoSummary { .. }
            | agent::End::Failed { .. }
            | agent::End::Budget { .. } => {
                let name = session.session.expect("a session that ran had opened");
                assert!(self.idle(name), "a session ends once nothing it asked for is in flight");
                assert!(self.jobs.values().all(|job| job.session != name), "a session ends only after its jobs");
                assert!(
                    session.job_ends.iter().copied().eq((session.opening_job..session.highest_job).map(|job| job + 1)),
                    "each announced job ended before its session ends"
                );
                assert_eq!(session.issued_job, session.highest_job, "the shell announced each job its machine yielded");
            }
        }
        if ended.end == agent::End::Closed {
            assert!(session.closed, "a session ends as closed only when its opener closed it");
        }
        if let agent::End::Budget { spent } = ended.end {
            assert!(self.spent(session, spent), "session {opener} ended when its {spent:?} budget was spent");
        }
        assert_eq!((ended.turns, ended.usage), (session.turns, session.usage), "an end adds up what was used");
        if let Some(name) = session.session {
            self.openers.remove(&name);
            self.closing.remove(&name);
        }
        self.stats.held_peaks[session.held_peak] += 1;
        self.tickets.free(opener);
        self.sessions.get_mut(&opener).expect("looked up above").ended = Some(ended);
    }

    /// Whether `session` has run out of its budget in `dimension` now: used it
    /// up, for what gates a completion; gone past it, for the cache.
    fn spent(&self, session: &Session, dimension: agent::Dimension) -> bool {
        let (budget, usage) = (&session.budget, &session.usage);
        match dimension {
            agent::Dimension::Unit => unreachable!("zero prices cannot spend the unit"),
            agent::Dimension::Turns => session.turns >= budget.turns,
            agent::Dimension::Input => usage.input_tokens.unwrap_or(0) >= budget.input,
            agent::Dimension::Output => usage.output_tokens.unwrap_or(0) >= budget.output,
            agent::Dimension::CacheRead => usage.cache_read_tokens.unwrap_or(0) > budget.cache_read,
            agent::Dimension::CacheWrite => usage.cache_write_tokens.unwrap_or(0) > budget.cache_write,
            agent::Dimension::Time => Some(self.now) >= session.expires,
        }
    }

    /// The opener serves the delegated call `call` for the session of `opener`,
    /// by `deadline`: a finish it refuses the first time, as its tests fail,
    /// and accepts after; a lookup, which is slow. It runs the race with the
    /// deadline itself.
    fn serve(&mut self, owner: Token, opener: u64, call: Token, deadline: Time, origin: agent::record::Origin) {
        let session = *self.runs.get(&owner).expect("a run is worked out from the step that started it");
        assert_eq!(
            self.sessions[&opener].accepted_stop.last(),
            Some(&agent::llm::Stop::ToolUse),
            "only callable ToolUse completions delegate effects"
        );
        let Ticketed::Call { tool, arguments: _ } = self.tickets.resolve(call).clone() else {
            panic!("a delegated call's ticket names a call");
        };
        let effect = SERVED.iter().find(|(name, ..)| *name == tool).expect("a call to a served tool").1;
        self.batched(session, effect == Effect::Write);
        let state = self.sessions.get_mut(&opener).expect("the opener serves the sessions it opened");
        let (mut text, mut error, mut accepts): (&[u8], bool, bool) = match tool {
            b"finish" => {
                state.finishes += 1;
                if state.finishes == 1 {
                    (b"not finished: the tests fail", true, false)
                } else {
                    (b"accepted", false, true)
                }
            }
            _ => (b"the forge says: fine", false, false),
        };
        let mut at = self.now.saturating_add(self.draw(self.settings.serving));
        if at > deadline {
            (text, error, accepts, at) = (b"timed out", true, false, deadline);
            self.stats.delegate_timeouts += 1;
        } else if tool == b"finish" {
            if accepts {
                self.stats.finishes_accepted += 1;
            } else {
                self.stats.finishes_refused += 1;
            }
        }
        let delivery = self.schedule(at, Delivery::Answered { owner, text: text.into(), error });
        let running = Running { delivery, origin, session, effect, accepts };
        self.tools.open(owner, running);
        self.stats.delegates += 1;
    }

    /// io starts the tools' operation `op` for `owner`, to end by
    /// `deadline`: after a draw, or at the deadline if that comes first.
    fn start_op(&mut self, owner: Token, op: Op, deadline: Time) {
        // A call's operations, one after another, carry its token.
        self.owners.insert(owner);
        let at = match &op {
            Op::Load { at, .. }
            | Op::List { at, .. }
            | Op::Remove { at, .. }
            | Op::Store { at, .. }
            | Op::Search { at, .. } => at.root,
            Op::Spawn { cwd, .. } => cwd.root,
        };
        let opener = *self.roots.get(&at.raw()).expect("an operation is in a session's repository");
        let session = self.sessions.get(&opener).and_then(|session| session.session);
        let session = session.expect("a session's tools ask io for something once it has opened");
        assert_eq!(
            self.sessions[&opener].accepted_stop.last(),
            Some(&agent::llm::Stop::ToolUse),
            "yielded and late Closing call tails never start owned IO effects"
        );
        let writes = self.writes(owner, session, &op);
        self.batched(session, writes);
        let mut ends = self.now.saturating_add(self.draw(self.settings.tool));
        let mut work = if self.rng.chance(self.settings.tool_errors) {
            self.stats.op_faults += 1;
            Work::Ending(Done::Failed { fault: Fault::Other })
        } else {
            match op {
                Op::Spawn { cwd, command, head, tail, after } => {
                    match io::spawn(&self.checkout, &cwd, &command, &[], (head, tail)) {
                        Ok(mut started) => {
                            // The script names changes relative to its cwd; the fake takes checkout paths.
                            for (path, _) in &mut started.process.program.changes {
                                let root = paths::name(self.checkout.root_path(cwd.root.raw()), &cwd.path);
                                *path = paths::name(&root, path);
                            }
                            if let Some(yield_at) = after.map(|delay| self.now.saturating_add(delay))
                                && yield_at < ends.min(deadline)
                            {
                                let work = Work::Yielding(jobs::Command { started, ends, deadline });
                                ends = yield_at;
                                work
                            } else {
                                Work::Command(started)
                            }
                        }
                        Err(done) => Work::Ending(done),
                    }
                }
                op @ (Op::Load { .. } | Op::List { .. } | Op::Remove { .. } | Op::Store { .. } | Op::Search { .. }) => {
                    Work::File(op)
                }
            }
        };
        // io runs the race with the deadline, and says it lost a moment
        // after the deadline passes: a command killed then tells what it wrote
        // by then (nothing, here).
        if ends > deadline {
            let done = match work {
                Work::Command(started) => io::exited(None, &[], started.head, started.tail),
                Work::File(_) | Work::Ending(_) => Done::TimedOut,
                Work::Yielding(_) => unreachable!("a command only yields before its deadline"),
            };
            let noticed = deadline.saturating_add(self.draw(self.settings.network));
            (work, ends) = (Work::Ending(done), noticed);
            self.stats.op_timeouts += 1;
        }
        let delivery = self.schedule(ends, Delivery::Ran { owner });
        self.ops.open(owner, Pending { delivery, session, writes, work });
        self.stats.ops += 1;
    }

    /// io is asked to cancel the operation of `owner`: it ends cancelled
    /// after a network draw, unless it ends of itself first.
    fn cancel_op(&mut self, owner: Token) {
        assert!(self.owners.contains(&owner), "a cancel names an operation io was asked for");
        if self.cancel_job(owner) {
            return;
        }
        let Some(pending) = self.ops.get(owner) else {
            // It ended in the iteration the cancel was sent.
            self.stats.cancels_crossed += 1;
            return;
        };
        self.closing.insert(pending.session);
        if self.rng.chance(self.settings.cancels_lost) {
            self.op_cancel_lost.insert(owner);
            self.stats.op_cancels_lost += 1;
            return;
        }
        let at = self.now.saturating_add(self.draw(self.settings.network));
        let delivery = self.schedule(at, Delivery::Ran { owner });
        let pending = self.ops.get_mut(owner).expect("looked up above");
        self.wire.withdraw(pending.delivery).expect("an operation in flight has its end on the way");
        (pending.delivery, pending.work) = (delivery, Work::Ending(Done::Cancelled));
        self.stats.op_cancels += 1;
    }

    /// The operation of `owner` ends, and io tells the tools how.
    fn ran(&mut self, owner: Token) {
        let pending = self.ops.end(owner);
        let done = match pending.work {
            Work::File(op) => {
                let removed = match &op {
                    Op::Remove { at, .. } => Some(paths::name(self.checkout.root_path(at.root.raw()), &at.path)),
                    _ => None,
                };
                fixture::prepare_list(&mut self.checkout, &op);
                let done = io::perform(&mut self.checkout, op);
                if let (Done::Removed, Some(path)) = (&done, removed) {
                    let opener = self.openers[&pending.session];
                    self.sessions.get_mut(&opener).expect("the operation names a session").removed_paths.insert(path);
                }
                done
            }
            Work::Command(started) => {
                io::finish(&mut self.checkout, &started.process);
                let program = &started.process.program;
                io::exited(Some(program.exit), &program.output, started.head, started.tail)
            }
            Work::Yielding(command) => self.hold_job(owner, pending.session, command),
            Work::Ending(done) => done,
        };
        if self.op_cancel_lost.remove(&owner) {
            self.stats.done_after_cancel += 1;
        }
        self.agent_stage.push(agent::Event::Done { owner, done });
    }

    /// Checks every result of the session's own tools in `prompt` against
    /// the call it answers, and counts those of its last message once.
    fn check_results(&mut self, owner: Token, prompt: &Prompt) {
        for pair in prompt.messages.windows(2) {
            let calls = pair[0].content.iter().filter_map(|block| {
                let Block::ToolCall(smith_domain_session::llm::ToolCall { call: Decoded::Owned { call }, id, .. }) =
                    block
                else {
                    return None;
                };
                Some((id, call))
            });
            for (id, call) in calls {
                let result = pair[1].content.iter().find_map(|block| {
                    let Block::ToolResult { id: answered, result: Returned::Owned { outcome } } = block else {
                        return None;
                    };
                    (answered == id).then_some(outcome)
                });
                if let Some(outcome) = result {
                    assert!(fixture::fits(call, outcome), "{outcome:?} is not what comes of {call:?}");
                }
            }
        }
        let counted = self.counted.entry(owner).or_default();
        if prompt.messages.len() > *counted {
            *counted = prompt.messages.len();
            let last = prompt.messages.last().expect("a prompt has a message");
            let answers = last
                .content
                .iter()
                .filter(|block| matches!(block, Block::ToolResult { result: Returned::Owned { .. }, .. }));
            self.stats.results += u32::try_from(answers.count()).expect("a small message");
        }
    }

    /// Whether the session `name` has no calls in flight. An unpolled job is not a call.
    fn idle(&self, name: Token) -> bool {
        !self.calling.contains_key(&name)
            && self.tools.values().all(|run| run.session != name)
            && self.ops.values().all(|op| op.session != name)
            && self.jobs.values().all(|job| job.session != name || job.batch_effect().is_none())
    }

    /// Whether the call `owner`'s operation `op` is for, of `session`'s tools,
    /// writes: as the tools told when it started, matched with its first
    /// operation, so that an edit writes as it loads. Once facts have been
    /// lost, the start may not have been told: the operation's own effect.
    fn writes(&mut self, owner: Token, session: Token, op: &Op) -> bool {
        let stores = matches!(op, Op::Remove { .. } | Op::Store { .. } | Op::Spawn { .. });
        if let Some(&writes) = self.writing.get(&owner) {
            assert!(writes || !stores, "only a call that writes stores or runs a command");
            return writes;
        }
        let told = self.starting.get_mut(&session).and_then(VecDeque::pop_front);
        let writes = match told {
            Some(tool) => match tool {
                tools::Tool::Remove | tools::Tool::Write | tools::Tool::Edit | tools::Tool::Shell => true,
                tools::Tool::Read | tools::Tool::List | tools::Tool::Search | tools::Tool::Job => false,
            },
            None => panic!("a call's first operation follows its native start observation"),
        };
        assert!(writes || !stores, "only a call that writes stores or runs a command");
        self.writing.insert(owner, writes);
        writes
    }

    /// Something starts for `session`, an operation of its tools or a
    /// delegated call, which `writes` or only reads: a write runs alone, and
    /// reads run together, as many as the limits allow. (The tools ask io for
    /// one operation at a time for each call, and only a call that writes
    /// stores or runs a command.)
    fn batched(&mut self, session: Token, writes: bool) {
        let runs = self.tools.values().filter(|run| run.session == session).map(|run| run.effect == Effect::Write);
        let ops = self.ops.values().filter(|op| op.session == session).map(|op| op.writes);
        let polls = self.jobs.values().filter(|job| job.session == session).filter_map(jobs::Job::batch_effect);
        let others: Vec<bool> = runs.chain(ops).chain(polls).collect();
        assert!(!others.contains(&true), "nothing runs beside a write");
        if writes {
            assert!(others.is_empty(), "a write runs alone");
        }
        let running = u32::try_from(others.len()).expect("a small batch") + 1;
        assert!(running <= self.settings.agent.parallel_tools, "no more runs at once than the limits allow");
        self.stats.most_parallel = self.stats.most_parallel.max(running);
    }

    /// The provider's requests, carried back the way its protocol layer would.
    fn provider_request(&mut self, request: provider::Request) {
        match request {
            provider::Request::Reply { to, result } => {
                let call = to.into_token().raw();
                self.serving.end(call);
                self.send(Delivery::Answer { call, result: result.map_err(|failure| failure.error) });
            }
            other @ (provider::Request::Piece { .. } | provider::Request::Hold { .. }) => {
                panic!("typed provider expected a terminal reply, got {other:?}")
            }
        }
    }

    /// Hands every delivery that is due to its destination.
    #[expect(clippy::too_many_lines, reason = "one exhaustive delivery table preserves terminal ownership")]
    fn deliver(&mut self) {
        while let Some(delivery) = self.wire.next(self.now) {
            match delivery {
                Delivery::Open { opener, opening } => self.open(opener, opening),
                Delivery::Continue { opener, content } => {
                    let session = self.sessions.get_mut(&opener).expect("the opener continues what it opened");
                    assert!(session.waiting, "the opener continues a session only while it is yielded");
                    session.waiting = false;
                    if session.closed {
                        // It closed the session while it was making up its mind.
                        continue;
                    }
                    let name = session.session.expect("a yielded session has opened");
                    if session.ended.is_some() {
                        self.stats.stale += 1;
                    }
                    self.agent_stage.push(agent::Event::Continue { session: name, content });
                    self.stats.continues += 1;
                }
                Delivery::Close { opener } => {
                    let session = self.sessions.get_mut(&opener).expect("the opener closes what it opened");
                    let again = !session.closed && self.rng.chance(self.settings.double_close);
                    let session = self.sessions.get_mut(&opener).expect("looked up above");
                    session.closed = true;
                    let name = session.session.expect("the opener closes a session once it has opened");
                    if session.ended.is_some() {
                        self.stats.stale += 1;
                    } else if self.closing.contains(&name) {
                        self.stats.closed_while_closing += 1;
                    }
                    self.agent_stage.push(agent::Event::Close { session: name });
                    self.stats.closes += 1;
                    if again {
                        self.send(Delivery::Close { opener });
                    }
                }
                Delivery::Query { call, query } => {
                    self.serving.open(call, ());
                    self.provider_stage.push(provider::Event::Call {
                        reply_to: ReplyTo::new(Token::new(call)),
                        call: Token::new(call),
                        query,
                    });
                    self.stats.provider_calls += 1;
                }
                Delivery::Answer { call, result } => {
                    // The fake refuses a transcript a provider would:
                    // a call without its result, a result without its call.
                    assert!(result != Err(provider::api::Error::InvalidRequest), "the agent sends well-formed queries");
                    if let Some(Call { owner, opener, served, input, .. }) = self.end_call(call) {
                        if self.cancel_lost.remove(&owner) {
                            match result {
                                Ok(_) => self.stats.answered_after_cancel += 1,
                                Err(_) => self.stats.failed_after_cancel += 1,
                            }
                        }
                        let mut result = result;
                        if let Ok(answer) = &mut result {
                            crate::provider::report(&mut answer.usage, input);
                        }
                        let event =
                            translate::outcome(owner, result, &mut self.tickets, opener, &served, &mut self.decoder);
                        self.agent_stage.push(event);
                    } else {
                        self.stats.late_answers += 1;
                    }
                }
                Delivery::Deadline { call, phase } => {
                    let Call { owner, .. } =
                        self.end_call(call).expect("a deadline is withdrawn when its call ends first");
                    if self.cancel_lost.remove(&owner) {
                        self.stats.failed_after_cancel += 1;
                    }
                    self.agent_stage.push(agent::Event::Failed {
                        retry_ok: true,
                        owner,
                        failure: Failure::TimedOut { phase },
                        evidence: smith_domain_session::llm::Evidence::Unknown,
                        detail: Box::default(),
                    });
                    self.stats.timeouts += 1;
                }
                Delivery::Cancelled { owner } => self.agent_stage.push(agent::Event::Cancelled { owner }),
                Delivery::AnswerCancelled { owner } => {
                    self.agent_stage.push(agent::Event::AnswerCancelled { owner, spent: 0 });
                }
                Delivery::Answered { owner, text, error } => {
                    // A withdrawn call's answer is withdrawn.
                    let run = self.tools.end(owner);
                    let opener = self.openers[&run.session];
                    self.sessions.get_mut(&opener).expect("delegation belongs to its opener").delegated_results.insert(
                        (run.origin.sequence, run.origin.position),
                        Returned::Text { text: text.clone(), error, replay: None },
                    );
                    if self.run_cancel_lost.remove(&owner) {
                        self.stats.answered_after_withdraw += 1;
                    }
                    self.agent_stage.push(agent::Event::Answered {
                        owner,
                        text,
                        error,
                        spent: 0,
                        after: agent::After::Complete,
                    });
                    // The opener has its finish, and closes the session.
                    if run.accepts {
                        let opener = *self.openers.get(&run.session).expect("a session lives while its calls run");
                        self.sessions.get_mut(&opener).expect("the opener's session").nudges = 0;
                        self.send(Delivery::Close { opener });
                    }
                }
                Delivery::Ran { owner } => self.ran(owner),
                Delivery::Job { owner, at } => self.job_tick(owner, at),
            }
        }
    }

    /// The opener opens a session, deciding how it will treat it.
    fn open(&mut self, opener: u64, mut opening: Box<agent::record::Opening>) {
        let spec = &mut opening.spec;
        let Count { min, max } = self.settings.nudges;
        let nudges = u32::try_from(self.rng.between(min.into(), max.into())).expect("drawn between two u32s");
        let abandon = self.rng.chance(self.settings.abandon);
        let (opening_sequence, opening_job) = opening
            .transcript
            .as_ref()
            .and_then(|transcript| transcript.turns.last())
            .map_or((0, 0), |turn| (turn.sequence, turn.jobs));
        // A repository of its own, which io names as its root.
        let root = fixture::seed(&mut self.checkout, format!("s{opener}").as_bytes());
        self.roots.insert(root, opener);
        for repo in &mut spec.authority.repos {
            repo.root = io::token(root);
        }
        if self.settings.reference {
            let root = fixture::seed_reference(&mut self.checkout, format!("ref{opener}").as_bytes());
            self.roots.insert(root, opener);
            let mut repos = std::mem::take(&mut spec.authority.repos).into_vec();
            repos.push(directory(b"host-ref", b"ref", io::token(root), false));
            spec.authority.repos = repos.into_boxed_slice();
        }
        let session = Session {
            previous_prompt: None,
            counting: crate::provider::Counting::new(self.settings.seed ^ opener),
            measures: Vec::new(),
            line: spec.line,
            compacting: None,
            opening: None,
            window: spec.window,
            ordinary: None,
            summary: None,
            summary_instruction: spec.summary.clone(),
            session: None,
            budget: spec.budget,
            max_tokens: spec.output,
            yields: Vec::new(),
            turns: 0,
            usage: Usage::ZERO,
            accepted: Vec::new(),
            received: None,
            accepted_content: Vec::new(),
            accepted_stop: Vec::new(),
            provider_closing: ProviderClosing::default(),
            accepted_late: Vec::new(),
            delegated_results: BTreeMap::new(),
            opening_sequence,
            opening_job,
            issued_job: opening_job,
            highest_job: opening_job,
            accepted_job_calls: Vec::new(),
            job_calls: Vec::new(),
            authority: spec.authority.clone(),
            removed_paths: BTreeSet::new(),
            accepted_path_calls: Vec::new(),
            path_calls: Vec::new(),
            held_peak: 0,
            job_ends: BTreeSet::new(),
            records: Vec::new(),
            priced: 0,
            expires: None,
            ended: None,
            nudges,
            abandon,
            waiting: false,
            closed: false,
            finishes: 0,
        };
        assert!(self.sessions.insert(opener, session).is_none(), "openers have distinct names");
        self.agent_stage.push(agent::Event::Open { opener: Token::new(opener), opening });
    }

    /// Ends the agent's call `call` if it is still in flight, withdrawing its
    /// deadline, and returns its owner.
    fn end_call(&mut self, call: u64) -> Option<Call> {
        let ended = self.calls.take(call)?;
        self.calling.remove(&ended.owner);
        self.wire.withdraw(ended.deadline);
        Some(ended)
    }

    fn has_work_now(&self) -> bool {
        self.agent_stage.has_events()
            || self.provider_stage.has_events()
            || self.agent.is_ready()
            || self.agent.is_due(self.now)
            || self.provider.is_due(self.now)
            || self.wire.is_due(self.now)
    }

    fn next_time(&self) -> Option<Time> {
        [self.wire.next_time(), self.agent.next_deadline(), self.provider.next_deadline()].into_iter().flatten().min()
    }

    fn tell(&mut self, fact: &agent::Fact) {
        let fact = fact.kind.clone();
        if let agent::FactKind::Tools { opener, fact } = fact {
            let (tools::FactKind::Opened { session }
            | tools::FactKind::Refused { session, .. }
            | tools::FactKind::Started { session, .. }
            | tools::FactKind::Answered { session, .. }
            | tools::FactKind::JobEnded { session, .. }
            | tools::FactKind::Closing { session, .. }
            | tools::FactKind::Closed { session }) = fact;
            assert!(self.sessions.contains_key(&opener.raw()), "a kit's fact names an opener that opened");
            if let Some(&known) = self.openers.get(&session) {
                assert_eq!(known, opener.raw(), "a kit's fact names the opener of its session");
            }
            if let tools::FactKind::Started { session, tool } = fact
                && tool != tools::Tool::Job
            {
                self.starting.entry(session).or_default().push_back(tool);
            }
        }
        let told = &mut self.told;
        if let agent::FactKind::CompletionAnswered { calls, invalid, .. } = fact {
            told.calls += calls;
            told.invalid_calls += invalid;
        }
        let count = match fact {
            agent::FactKind::CompactionStarted { opener, window, .. } => {
                self.sessions.get_mut(&opener.raw()).expect("known opener").compacting = Some(window);
                return;
            }
            agent::FactKind::BuiltinStarted { .. }
            | agent::FactKind::BuiltinAnswered { .. }
            | agent::FactKind::CompactionEnded { .. } => return,
            agent::FactKind::Opened { .. } => &mut told.opened,
            agent::FactKind::JobEnded { opener, job, end, .. } => {
                let session = self.sessions.get_mut(&opener.raw()).expect("a job terminal names its opener");
                jobs::observe_end(&mut session.job_ends, job, end);
                &mut told.jobs_ended
            }
            agent::FactKind::ReasoningDropped { .. } => &mut told.reasoning_dropped,
            agent::FactKind::CompletionStarted { .. } => &mut told.completions_started,
            agent::FactKind::CompletionAnswered { .. } => &mut told.completions_answered,
            agent::FactKind::CompletionFailed { .. } => &mut told.completions_failed,
            agent::FactKind::CompletionCancelled { .. } => &mut told.completions_cancelled,
            agent::FactKind::CompletionRetried { .. } => &mut told.completions_retried,
            agent::FactKind::Tools { opener: _, fact } => match fact {
                tools::FactKind::Opened { .. } => &mut told.kits_opened,
                tools::FactKind::Closed { .. } => &mut told.kits_closed,
                tools::FactKind::Started { .. } => &mut told.tools_started,
                tools::FactKind::Answered { .. } => &mut told.tools_answered,
                tools::FactKind::Refused { .. }
                | tools::FactKind::Closing { .. }
                | tools::FactKind::JobEnded { .. } => return,
            },
            agent::FactKind::DelegateStarted { .. } => &mut told.delegates_started,
            agent::FactKind::DelegateAnswered { .. } => &mut told.delegates_answered,
            agent::FactKind::DelegateCancelled { .. } | agent::FactKind::DelegateWithheld { .. } => {
                &mut told.delegates_cancelled
            }
            agent::FactKind::Yielded { .. } => &mut told.yielded,
            agent::FactKind::Used { .. } => &mut told.used,
            agent::FactKind::Ended { .. } => &mut told.ended,
        };
        *count += 1;
    }

    /// What every native fact must add up to: what the world
    /// saw cross the boundary.
    fn assert_told(&self) {
        let (told, stats) = (&self.told, &self.stats);
        let count = |n: usize| u32::try_from(n).expect("a small world");
        let opened = count(self.sessions.values().filter(|session| session.session.is_some()).count());
        let turns: u32 = self.sessions.values().map(|session| session.turns).sum();
        assert_eq!((told.opened, told.ended), (opened, count(self.sessions.len())), "an open and an end each");
        assert_eq!(told.completions_started, stats.calls, "a fact for every call");
        let ended = told.completions_answered + told.completions_failed + told.completions_cancelled;
        assert_eq!(ended, stats.calls, "a fact for the end of every call");
        assert_eq!(told.completions_cancelled, stats.cancels, "a fact for every cancelled call");
        assert_eq!(told.used, turns, "a fact for every completion's usage");
        assert_eq!((told.kits_opened, told.kits_closed), (opened, opened), "a kit opened and closed per session");
        assert!(told.tools_started <= stats.ops + stats.job_acts, "a call the tools start asks io for something");
        assert!(told.tools_answered >= told.tools_started, "a fact for the answer to every call started");
        assert!(told.tools_answered >= stats.results, "a fact for every result that went back");
        assert_eq!(told.yielded, stats.yields, "a fact for every yield");
        assert_eq!(told.delegates_started, stats.delegates, "a fact for every delegated call");
        let delegates_ended = told.delegates_answered + told.delegates_cancelled;
        assert_eq!(delegates_ended, stats.delegates, "a fact for the end of every delegated call");
        assert_eq!(told.delegates_cancelled, stats.withdraws, "a fact for every withdrawn call");
    }

    /// The invariants of a world where nothing is left to happen.
    fn assert_settled(&self) {
        assert_eq!(self.agent.sessions(), 0, "every session has ended and been reclaimed");
        assert_eq!(self.agent.runs(), 0, "every run has ended and been reclaimed");
        assert_eq!((self.agent.kits(), self.agent.flights()), (0, 0), "every kit has closed, its calls answered");
        assert!(self.ops.is_empty() && self.op_cancel_lost.is_empty(), "every operation has ended, once");
        assert!(self.jobs.is_empty(), "no held command outlives its session");
        assert_eq!(self.agent.next_deadline(), None, "no alarm outlives its session");
        assert_eq!(self.provider.calls(), 0, "the provider holds no call");
        assert!(self.calls.is_empty() && self.calling.is_empty(), "no call is in flight");
        assert!(self.tools.is_empty() && self.runs.is_empty(), "no tool is running, and every run's end was heard");
        assert!(self.tickets.is_empty(), "every session's tickets were freed when it ended");
        assert!(self.cancel_lost.is_empty() && self.run_cancel_lost.is_empty(), "every lost cancel's race ended");
        assert!(self.serving.is_empty(), "the provider answered every call");
        assert!(
            self.wire.is_empty() && !self.agent_stage.has_events() && !self.provider_stage.has_events(),
            "nothing is on its way"
        );
        for (opener, session) in &self.sessions {
            assert!(session.ended.is_some(), "session {opener} has ended");
            assert_eq!(
                session.records.len(),
                session.accepted.len()
                    - usize::from(matches!(
                        session.ended.as_ref().map(|ended| ended.end.clone()),
                        Some(agent::End::NoSummary { .. })
                    )),
                "every accepted completion except a failed summary emits concrete history"
            );
            if session.session.is_some() {
                assert!(session.priced >= session.turns, "every usage and final terminal is priced");
            }
        }
        self.assert_told();
    }

    fn send(&mut self, delivery: Delivery) {
        let at = self.now.saturating_add(self.draw(self.settings.network));
        self.schedule(at, delivery);
    }

    fn schedule(&mut self, at: Time, delivery: Delivery) -> Key {
        // On a coarse grid, if the world has one, so that deliveries coincide.
        let granule = self.settings.granule.as_nanos();
        let at =
            if granule > 0 { Time::from_nanos(at.as_nanos().div_ceil(granule).saturating_mul(granule)) } else { at };
        self.wire.send(at, delivery)
    }

    fn draw(&mut self, span: Span) -> Duration {
        span.draw(&mut self.rng)
    }

    fn log(&mut self, line: impl fmt::Display) {
        if let Some(trace) = &mut self.trace {
            trace.log(self.now, line);
        }
    }
}

/// The delegated call whose end `event` is, if it is one.
fn ended_run(event: &agent::Event) -> Option<Token> {
    match event {
        agent::Event::Answered { owner, .. }
        | agent::Event::AnswerWithheld { owner }
        | agent::Event::AnswerCancelled { owner, .. } => Some(*owner),
        agent::Event::BuiltinResult { .. }
        | agent::Event::ReadVersion { .. }
        | agent::Event::Open { .. }
        | agent::Event::Continue { .. }
        | agent::Event::CancelJobs { .. }
        | agent::Event::Yield { .. }
        | agent::Event::Close { .. }
        | agent::Event::Cut { .. }
        | agent::Event::Completed { .. }
        | agent::Event::Overflowed { .. }
        | agent::Event::BudgetDenied { .. }
        | agent::Event::UnsentClosed { .. }
        | agent::Event::Narrowed { .. }
        | agent::Event::Failed { .. }
        | agent::Event::Cancelled { .. }
        | agent::Event::Done { .. } => None,
    }
}

struct EventLine<'a>(&'a agent::Event);

impl fmt::Display for EventLine<'_> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str("agent <- ")?;
        match self.0 {
            agent::Event::ReadVersion { session, place, version } => {
                write!(out, "read version {} {place:?} {version:?}", session.raw())
            }
            agent::Event::BuiltinResult { session, id, text, error } => {
                write!(out, "builtin result {} {id:?} {text:?} {error}", session.raw())
            }
            agent::Event::Yield { session } => write!(out, "yield completion {}", session.raw()),
            agent::Event::CancelJobs { session } => write!(out, "cancel jobs {}", session.raw()),
            agent::Event::AnswerWithheld { owner } => write!(out, "answer withheld {}", owner.raw()),
            agent::Event::AnswerCancelled { owner, spent, .. } => {
                write!(out, "answer cancelled {} {spent}", owner.raw())
            }
            agent::Event::Open { opener, opening } => write!(out, "open {} {opening:?}", opener.raw()),
            agent::Event::Answered { owner, text, error, spent, after } => match after {
                agent::After::Complete => write!(out, "answered {} {text:?} {error} {spent}", owner.raw()),
                agent::After::Yield => write!(out, "answered and yield {} {text:?} {error} {spent}", owner.raw()),
            },
            agent::Event::Continue { session, content } => {
                write!(out, "continue {} {:?}", session.raw(), String::from_utf8_lossy(content))
            }
            agent::Event::Cut { session } => write!(out, "cut {}", session.raw()),
            agent::Event::Close { session } => write!(out, "close {}", session.raw()),
            agent::Event::Completed { owner, completion } => {
                write!(out, "completed {} {:?} with {} blocks", owner.raw(), completion.stop, completion.content.len())
            }
            agent::Event::Overflowed { owner, end } => write!(out, "overflowed {} {end:?}", owner.raw()),
            agent::Event::Narrowed { owner, note, choice, names } => {
                write!(out, "narrowed {} {note:?} {choice:?} {names:?}", owner.raw())
            }
            agent::Event::UnsentClosed { owner } => write!(out, "unsent closed {}", owner.raw()),
            agent::Event::BudgetDenied { owner, reason } => write!(out, "budget denied {} {reason:?}", owner.raw()),
            agent::Event::Failed { owner, failure, .. } => write!(out, "failed {} {failure:?}", owner.raw()),
            agent::Event::Cancelled { owner } => write!(out, "cancelled {}", owner.raw()),
            agent::Event::Done { owner, done } => write!(out, "done {} {done:?}", owner.raw()),
        }
    }
}

struct RequestLine<'a>(&'a agent::Request);

impl fmt::Display for RequestLine<'_> {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str("agent -> ")?;
        match self.0 {
            agent::Request::Part { .. } => out.write_str("part"),
            agent::Request::JobsCancelled { opener } => write!(out, "jobs cancelled {}", opener.raw()),
            agent::Request::Turn { opener, turn } => write!(out, "turn {} {turn:?}", opener.raw()),
            agent::Request::Priced { opener, spent, own_spent } => {
                write!(out, "priced {} {spent} own {own_spent}", opener.raw())
            }
            agent::Request::Opened { opener, session } => write!(out, "opened {} as {}", opener.raw(), session.raw()),
            agent::Request::Yielded { opener, stop, text } => {
                write!(out, "yielded {} {stop:?} {:?}", opener.raw(), String::from_utf8_lossy(text))
            }
            agent::Request::Used { opener, usage, .. } => write!(out, "used {} {usage:?}", opener.raw()),
            agent::Request::Ended { opener, end, turns, usage, .. } => {
                write!(out, "ended {} {end:?} after {turns} turns, {usage:?}", opener.raw())
            }
            agent::Request::RateLimitWait { opener, until } => {
                write!(out, "rate limit wait {} until {}", opener.raw(), until.as_nanos())
            }
            agent::Request::RateLimitWaitDone { opener } => write!(out, "rate limit wait done {}", opener.raw()),
            agent::Request::RateLimitOutlastsSession { opener, retry_after } => {
                write!(out, "rate limit outlasts session {} by reset in {retry_after:?}", opener.raw())
            }
            agent::Request::Complete { owner, prompt, deadlines, .. } => {
                let (messages, most) = (prompt.messages.len(), prompt.max_tokens);
                write!(
                    out,
                    "complete {} with {messages} messages, at most {most} tokens, deadlines {deadlines:?}",
                    owner.raw()
                )
            }
            agent::Request::Yield { owner } => write!(out, "yield completion {}", owner.raw()),
            agent::Request::Cancel { owner } => write!(out, "cancel {}", owner.raw()),
            agent::Request::Io { owner, op, deadline } => {
                write!(out, "io {} {op:?} by {}", owner.raw(), deadline.as_nanos())
            }
            agent::Request::Job { owner, act } => write!(out, "job {} {act:?}", owner.raw()),
            agent::Request::CancelIo { owner } => write!(out, "cancel io {}", owner.raw()),
            agent::Request::Delegate { owner, opener, call, deadline, origin } => {
                write!(
                    out,
                    "delegate {} for {} {call:?} origin {origin:?} by {}",
                    owner.raw(),
                    opener.raw(),
                    deadline.as_nanos()
                )
            }
            agent::Request::Withdraw { owner } => write!(out, "withdraw {}", owner.raw()),
        }
    }
}

/// The results answered as not run in the last message of `prompt`.
fn not_run(prompt: &Prompt) -> u32 {
    let Some(last) = prompt.messages.last() else { return 0 };
    let unrun = last.content.iter().filter(|block| {
        matches!(
            block,
            Block::ToolResult { result: Returned::NotRun { why: smith_domain_session::llm::NotRun::Stopped }, .. }
        )
    });
    u32::try_from(unrun.count()).expect("a small message")
}

/// Checks that the results at the end of `prompt` answer the calls of the
/// message before them, in their order.
fn in_call_order(prompt: &Prompt) {
    let [.., asked, answered] = &*prompt.messages else { return };
    let calls: Vec<&[u8]> = asked
        .content
        .iter()
        .filter_map(|block| {
            if let Block::ToolCall(smith_domain_session::llm::ToolCall { id, .. }) = block { Some(&**id) } else { None }
        })
        .collect();
    let results: Vec<&[u8]> = answered
        .content
        .iter()
        .filter_map(|block| if let Block::ToolResult { id, .. } = block { Some(&**id) } else { None })
        .collect();
    assert_eq!(calls, results, "the results go back in call order");
}

/// Every resumed yielded tail settles before the waking assistant.
fn assert_resumed_tail(session: &Session, index: usize, preceding: &[agent::llm::Message]) {
    if index > 0 && session.accepted_stop[index - 1] != agent::llm::Stop::ToolUse && !session.accepted_late[index - 1] {
        for block in &session.accepted_content[index - 1] {
            if let Block::ToolCall(smith_domain_session::llm::ToolCall { id, .. }) = block {
                let result =
                    recorded_result(preceding, id).expect("continuing a yielded call supplies its next-Turn result");
                assert_eq!(
                    *result,
                    Returned::NotRun { why: smith_domain_session::llm::NotRun::Stopped },
                    "yielded calls did not execute and precede the waking assistant"
                );
            }
        }
    }
}

/// Checked raw-prefix addition: the scripted peer cannot overflow these bounded counters.
fn add_usage(left: Usage, right: Usage) -> Usage {
    Usage {
        input_tokens: Some(
            left.input_tokens
                .unwrap_or(0)
                .checked_add(right.input_tokens.unwrap_or(0))
                .expect("bounded scheduled input"),
        ),
        output_tokens: Some(
            left.output_tokens
                .unwrap_or(0)
                .checked_add(right.output_tokens.unwrap_or(0))
                .expect("bounded scheduled output"),
        ),
        cache_read_tokens: Some(
            left.cache_read_tokens
                .unwrap_or(0)
                .checked_add(right.cache_read_tokens.unwrap_or(0))
                .expect("bounded scheduled cache reads"),
        ),
        cache_write_tokens: Some(
            left.cache_write_tokens
                .unwrap_or(0)
                .checked_add(right.cache_write_tokens.unwrap_or(0))
                .expect("bounded scheduled cache writes"),
        ),
        reasoning_tokens: match (left.reasoning_tokens, right.reasoning_tokens) {
            (Some(left), Some(right)) => Some(left.checked_add(right).expect("bounded scheduled reasoning")),
            (None, _) | (_, None) => None,
        },
    }
}

fn unstarted_delegate(result: &Returned, position: u32) {
    match result {
        Returned::NotRun { why: smith_domain_session::llm::NotRun::Stopped } => {}
        Returned::NotRun { why: smith_domain_session::llm::NotRun::AfterFailedWrite { call } } => {
            assert!(u32::from(*call) <= position + 1, "failure precedes unstarted call");
        }
        Returned::Owned { .. }
        | Returned::Text { .. }
        | Returned::Invalid { .. }
        | Returned::Withdrawn
        | Returned::NotRun { why: smith_domain_session::llm::NotRun::OutsideChoice { .. } } => {
            panic!("an unstarted delegated call has no terminal right");
        }
    }
}
