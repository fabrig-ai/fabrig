//! Live run and restart correlation held by the core (domain/engine/README.md, 3, and domain/engine/executors.md, 2).

use crate::{CallKey, Core, RunChoice, TurnRecord};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Time, Token, Wall, length::len_u64};

use crate::{runs::transcript_text::TranscriptText, translate};

/// The core's decision before a due agent run is prepared or claimed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RunAdmission {
    /// Current policy, funds, account and writes allow preparation.
    Allow(Box<crate::RunAllowance>),
    /// A current account, application credential or host wait.
    Wait(crate::RunWait),
    /// The task child must hold the run for the named reason.
    Hold(tasks::Hold),
}

/// Bounded recent opaque conversation while a due task's store pages are read.
/// Each retained turn counts its four-byte length beside its opaque body
/// (domain/engine/executors.md, 2).
#[derive(Hash, Debug)]
pub struct Transcript {
    pub window: Option<(u64, u32)>,
    /// An upgrade closed the loaded window or crossed its claim.
    pub closed: bool,
    /// A loaded turn was past its store page room (domain/engine/executors.md, 2).
    pub cut: bool,
    /// The preparation has received its final store page.
    pub loaded: bool,
    pub previous_attempt: u64,
    pub last: Option<(u64, u32)>,
    pub from: u64,
    pub bytes: u64,
    pub kept: u64,
    pub turns: Queue<Box<[u8]>>,
    pub texts: Queue<TranscriptText>,
    pub text_bytes: u64,
    pub text_kept: u64,
}

impl Core {
    /// The bytes the run preparations hold or are about to hold, for a
    /// composing root's byte budget (skein's `programming-model.md`,
    /// 6.1): each preparation counts at its resume allowance from its start
    /// until its load is in, then at the turn bodies and text it keeps.
    #[must_use]
    pub fn transcript_bytes(&self) -> u64 {
        let mut bytes = 0_u64;
        for (_, transcript) in &self.transcripts {
            if transcript.loaded {
                bytes = bytes.checked_add(transcript.text_kept).expect("retained transcript text bytes fit u64");
                for body in &transcript.turns {
                    bytes = bytes.checked_add(len_u64(body.len())).expect("retained transcript body bytes fit u64");
                }
            } else {
                bytes = bytes
                    .checked_add(u64::from(self.settings.resume_bytes))
                    .expect("transcript resume allowances fit u64")
                    .checked_add(u64::from(self.settings.tail_bytes))
                    .expect("transcript allowances fit u64");
            }
        }
        bytes
    }

    pub(crate) fn run_credentials(&self, task: u64) -> &[u32] {
        if let Some(credentials) = self.workspace_credentials.get(&task) {
            return credentials;
        }
        match self.proofs.get(&task) {
            Some(proof) => &proof.credentials,
            None => &[],
        }
    }
    pub(crate) fn credentials_ready(&self, task: u64, now: Time) -> bool {
        for number in self.run_credentials(task) {
            if !self.accounts.credential_usable(*number, now) {
                return false;
            }
        }
        true
    }
    pub(crate) fn run_credentials_check(&self, task: u64, now: Time) -> Box<[bool]> {
        let mut usable = List::with_capacity(self.accounts.credential_limit());
        for number in self.run_credentials(task) {
            usable.push(self.accounts.credential_usable(*number, now)).expect("credential bound");
        }
        usable.into_boxed()
    }
    /// Decide a run against current policy, funds, account usability and
    /// connector-translated workspace writes.
    #[must_use]
    pub fn run_admission(&self, task: &tasks::RunContext, wall: Wall, writes: Box<[authority::Write]>) -> RunAdmission {
        let Some(task) = self.tasks.run_context(task.task) else { return RunAdmission::Wait(crate::RunWait::Host) };
        let choice = self.run_choice(task.task);
        let mut findings = Queue::uncapped();
        let checked = self.run_check(
            (task.task, task.project, &task.authority, task.numbers),
            choice.as_ref().ok(),
            wall,
            writes,
            &mut findings,
        );
        match checked {
            authority::RunAnswer::Allow { spend, accounts } => {
                let mut choice = choice.expect("authority requires a usable main binding");
                choice.accounts = accounts;
                RunAdmission::Allow(Box::new(crate::RunAllowance { choice, spend }))
            }
            authority::RunAnswer::Wait | authority::RunAnswer::Propose | authority::RunAnswer::Refuse => denied(
                &mut findings,
                match choice {
                    Err(wait) => wait,
                    Ok(_) => crate::RunWait::Credential,
                },
            ),
        }
    }

    /// `check_run` for a task's current numbers and no writes but `writes`,
    /// under `choice`'s accounts, or none if there is no choice.
    fn run_check(
        &self,
        (number, project, task_authority, task_numbers): (u64, u32, &tasks::Authority, tasks::Numbers),
        choice: Option<&RunChoice>,
        wall: Wall,
        writes: Box<[authority::Write]>,
        findings: &mut Queue<authority::Finding>,
    ) -> authority::RunAnswer {
        let mut accounts = List::with_capacity(jig_domain_tasks::MODELS);
        if let Some(choice) = choice {
            for binding in &choice.accounts {
                accounts
                    .push(authority::AccountCheck { binding: *binding, usable: true, host_eligible: true })
                    .expect("provider bound");
            }
        }
        let mut current = List::with_capacity(self.authority.limits().writes);
        for mut write in writes {
            write.effect = crate::effects::resources::effect(self, project, write.effect);
            if current.push(write).is_err() {
                findings.push(authority::Finding::Oversized);
                return authority::RunAnswer::Refuse;
            }
        }
        let writes = current.into_boxed();
        let numbers = translate::authority_numbers(self.tasks.claim_numbers(number).unwrap_or(task_numbers));
        authority::check_run(
            &self.authority,
            &authority::RunAsk {
                project,
                authority: translate::tasks_authority(task_authority),
                numbers,
                budget: authority::left(numbers).min(self.authority.rules().maximum_run_spend),
                reserve: self.run_reserve(number),
                wall,
                main_provider: match choice {
                    Some(choice) => choice.accounts.first().expect("choice includes main provider").provider,
                    None => 0,
                },
                accounts: accounts.into_boxed(),
                credentials: self.run_credentials_check(number, self.run_now),
                writes,
            },
            findings,
        )
    }

    /// Whether a due agent's admission for no writes is anything but a wait
    /// (domain/engine/executors.md, 1), asking the cheaper checks first.
    pub(crate) fn agent_ready(&self, task: u64) -> bool {
        let Some(row) = self.tasks.task(task) else { return false };
        let current = (task, row.project, &row.authority, row.numbers);
        let mut findings = Queue::uncapped();
        match self.run_check(current, None, self.run_wall, Box::new([]), &mut findings) {
            authority::RunAnswer::Allow { .. } => unreachable!("no accounts cannot allow a run"),
            authority::RunAnswer::Wait | authority::RunAnswer::Propose | authority::RunAnswer::Refuse => {}
        }
        // Every finding but Account holds under any choice.
        match denied(&mut findings, crate::RunWait::Host) {
            RunAdmission::Hold(_) => return true,
            RunAdmission::Wait(_) => {}
            RunAdmission::Allow(_) => unreachable!("denied admission cannot allow a run"),
        }
        // A choice binds at most one account per provider and fits authority's
        // checked account limit, so it cannot add an oversized hold here.
        if !self.credentials_ready(task, self.run_now) {
            return false;
        }
        let Ok(choice) = self.run_choice(task) else { return false };
        match self.run_check(current, Some(&choice), self.run_wall, Box::new([]), &mut findings) {
            authority::RunAnswer::Allow { .. } => true,
            authority::RunAnswer::Wait | authority::RunAnswer::Propose | authority::RunAnswer::Refuse => {
                match denied(&mut findings, crate::RunWait::Credential) {
                    RunAdmission::Hold(_) | RunAdmission::Allow(_) => true,
                    RunAdmission::Wait(_) => false,
                }
            }
        }
    }

    /// Put back the durable evidence when a pending claim cannot commit.
    pub(crate) fn restore_replaced_proof(&mut self, task: u64) {
        drop(self.proofs.remove(&task));
        if let Some(proof) = self.replaced.remove(&task) {
            self.proofs.insert(task, proof).expect("replacement releases proof room");
        }
    }

    /// Only unusable window history closes a window (jig/docs/design/domain/engine/executors.md, 2).
    pub(crate) fn close_window(&mut self, task: u64) {
        if let Some(proof) = self.proofs.get_mut(&task)
            && let Some(window) = &mut proof.window
        {
            window.closed = true;
        }
    }

    /// Begin a bounded transcript load for a due task's previous attempt.
    pub fn begin_transcript(&mut self, task: u64, previous_attempt: u64, window: Option<(u64, u32)>) {
        self.start_input();
        let closed = match self.proofs.get(&task) {
            Some(proof) => {
                proof.crossed
                    || match proof.window {
                        Some(window) => window.closed,
                        None => false,
                    }
            }
            None => false,
        };
        let from = match self.proofs.get(&task) {
            Some(proof) => proof.conversation,
            None => 1,
        };
        assert!(
            self.transcripts
                .insert(
                    task,
                    Transcript {
                        window,
                        closed,
                        cut: false,
                        loaded: false,
                        previous_attempt,
                        last: None,
                        from,
                        bytes: 0,
                        kept: 0,
                        texts: Queue::with_capacity(self.settings.tail_bytes),
                        text_bytes: 0,
                        text_kept: 0,
                        turns: Queue::with_capacity(
                            self.settings.resume_bytes.checked_div(4).expect("nonzero length size")
                        ),
                    },
                )
                .is_ok(),
            "one transcript preparation per live task"
        );
    }

    /// Admit one restored turn in store order, retaining only the bounded
    /// tail while still counting the full current window for resume policy.
    /// Text is UTF-8 supplied by the host protocol; `None` keeps an unreadable marker.
    #[must_use]
    pub fn append_transcript(&mut self, task: u64, turn: TurnRecord, text: Option<Box<[u8]>>) -> bool {
        let length = len_u64(turn.transcript.len());
        self.append_loaded_transcript(task, turn, text, length, false)
    }

    /// Admit a window turn's fields and length without its body, retaining its
    /// fixed text in the tail and preventing resume (domain/engine/executors.md, 2).
    #[must_use]
    pub fn append_cut_transcript(&mut self, task: u64, turn: TurnRecord, bytes: u64) -> bool {
        self.append_loaded_transcript(task, turn, Some(Box::from(crate::CUT_TURN_TEXT)), bytes, true)
    }

    fn append_loaded_transcript(
        &mut self,
        task: u64,
        turn: TurnRecord,
        text: Option<Box<[u8]>>,
        length: u64,
        cut: bool,
    ) -> bool {
        self.start_input();
        let standing = match self.tasks.task(task) {
            Some(row) => row.flags.ended_by == tasks::EndedBy::Standing,
            None => false,
        };
        let transcript = self.transcripts.get_mut(&task).expect("load belongs to prepared task");
        if turn.task != task || turn.attempt > transcript.previous_attempt || turn.attempt == 0 || turn.turn == 0 {
            return false;
        }
        if let Some(previous) = transcript.last
            && (turn.attempt, turn.turn) <= previous
        {
            return false;
        }
        let Some(start) = transcript.window else { return false };
        if (turn.attempt, turn.turn) < start
            || (transcript.last.is_none() && ((turn.attempt, turn.turn) != start || !turn.opens_window))
        {
            return false;
        }
        let Some(framed) = length.checked_add(4) else { return false };
        let Some(total) = transcript.bytes.checked_add(framed) else { return false };
        transcript.bytes = total;
        transcript.cut |= cut;
        transcript.last = Some((turn.attempt, turn.turn));
        // A standing retry carries only the attempt it retries (domain/tasks/standing.md, 5.4).
        if !standing || turn.attempt == transcript.previous_attempt {
            transcript.append_text(text, self.settings.tail_bytes);
        }
        if transcript.closed || transcript.cut {
            return true;
        }
        let bound = u64::from(self.settings.resume_bytes);
        if bound < 4 {
            return true;
        }
        let body_bound = bound.checked_sub(4).expect("room for one length");
        let body = if length > body_bound {
            let start = turn
                .transcript
                .len()
                .checked_sub(usize::try_from(body_bound).expect("u32 bound fits usize"))
                .expect("oversize turn has tail");
            Box::<[u8]>::from(turn.transcript.get(start..).expect("tail starts inside stored turn"))
        } else {
            turn.transcript
        };
        let body_len = len_u64(body.len()).checked_add(4).expect("bounded framed turn");
        for _ in 0..transcript.turns.len() {
            if transcript.kept.checked_add(body_len).expect("two bounded tails") <= bound {
                break;
            }
            let old = transcript.turns.pop().expect("overfull transcript has an older turn");
            transcript.kept = transcript
                .kept
                .checked_sub(len_u64(old.len()).checked_add(4).expect("bounded old frame"))
                .expect("old turn was counted");
        }
        transcript.turns.push(body);
        transcript.kept = transcript.kept.checked_add(body_len).expect("bounded transcript tail");
        true
    }

    /// A completed load includes the window the core asked the store to read.
    #[must_use]
    pub fn transcript_readable(&self, task: u64) -> bool {
        let transcript = self.transcripts.get(&task).expect("prepared transcript state");
        transcript.window.is_none() || transcript.last.is_some()
    }

    /// Whether the current window exceeded the deployment's resume bound.
    #[must_use]
    pub fn transcript_oversized(&self, task: u64) -> bool {
        let transcript = self.transcripts.get(&task).expect("prepared transcript state");
        transcript.cut || transcript.bytes > u64::from(self.settings.resume_bytes)
    }

    /// Select whole prior turns only while their cumulative bytes fit the
    /// deployment's resume bound. A longer window starts fresh.
    #[must_use]
    pub fn resumed_turns(&self, transcript: Transcript) -> Box<[Box<[u8]>]> {
        let mut turns = List::with_capacity(transcript.turns.len());
        if !transcript.closed && !transcript.cut && transcript.bytes <= u64::from(self.settings.resume_bytes) {
            let mut kept = transcript.turns;
            for _ in 0..kept.len() {
                turns.push(kept.pop().expect("counted transcript turn")).expect("bounded transcript turns");
            }
        }
        turns.into_boxed()
    }
}

#[derive(Hash, Debug)]
pub struct RestoringProof {
    pub attempt: u64,
    pub turn: u32,
    pub run_spent: u64,
    pub last_answer: Option<u64>,
}

/// One configured model and its deployment-unit prices (domain/agent/charter.md, 5).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Model {
    pub provider: u32,
    pub dialect: u32,
    pub endpoint: u32,
    /// The endpoint's backend, when it takes program sources (configuration.md, 3).
    pub program: Option<Box<[u8]>>,
    pub name: Box<[u8]>,
    /// The input tokens a conversation's window may hold on this model.
    pub window: u32,
    /// Prompt cache life in milliseconds; zero for none (domain/tasks/standing.md, 5.3).
    pub cache_life_ms: u32,
    /// The most output tokens one completion may have, thinking included.
    pub output: u32,
    pub input_price: u64,
    pub cached_price: u64,
    /// Cache writes, when priced apart from fresh input.
    pub cache_write_price: Option<u64>,
    pub output_price: u64,
    pub price_unit: u32,
    /// The efforts the model declares, by name in its order; a task's LLM choice names one by
    /// index, and none for a model that declares none (domain/tasks/README.md, 3).
    pub efforts: Box<[Box<[u8]>]>,
    /// On a configured model and a charter's, the default a task given no LLM choice takes,
    /// one of `efforts` by name, or `None` for its first (configuration.md, 3).
    /// On a run's charter, its task's choice's (domain/engine/executors.md, 1), or `None` for the LLM
    /// provider's default when the model declares no efforts.
    pub effort: Option<Box<[u8]>>,
}

impl Model {
    /// An unoccupied catalogue position (domain/engine/settings.md, 11).
    #[must_use]
    pub fn hole() -> Self {
        Self {
            provider: 0,
            dialect: 0,
            endpoint: 0,
            program: None,
            name: Box::default(),
            window: 0,
            cache_life_ms: 0,
            output: 0,
            input_price: 0,
            cached_price: 0,
            cache_write_price: None,
            output_price: 0,
            price_unit: 0,
            efforts: Box::default(),
            effort: None,
        }
    }
    #[must_use]
    pub fn is_hole(&self) -> bool {
        self.name.is_empty()
    }

    /// The default effort a task given no LLM choice takes (domain/tasks/README.md, 3).
    #[must_use]
    pub fn default_effort(&self) -> Option<&[u8]> {
        match &self.effort {
            Some(effort) => Some(effort.as_ref()),
            None => match self.efforts.first() {
                Some(effort) => Some(effort.as_ref()),
                None => None,
            },
        }
    }
}

/// The most efforts a configured model declares (configuration.md, 3).
pub const EFFORTS: u32 = 16;
/// The most bytes in an effort's name: the bound smith's charter holds an effort to
/// (smith's protocol/charter.md, sections 2 and 6).
pub const EFFORT_BYTES: u32 = 32;
/// Byte bounds on a configured model's name and description (protocol/limits.md).
pub use jig_domain_people::{MODEL_DESCRIPTION_BYTES, MODEL_NAME_BYTES};

/// Root-owned charter policy, independent of Smith's run vocabulary.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent charter grants and run behavior")]
pub struct RunPolicy {
    pub instructions: Box<[u8]>,
    pub waiting: skein_lib::Duration,
    pub resume: bool,
    pub turns: u32,
    pub time: skein_lib::Duration,
    /// The turns and time of each run kept for finishing once the rest of
    /// its budget is spent.
    pub reserve_turns: u32,
    pub reserve_time: skein_lib::Duration,
    pub model: Model,
    pub alternatives: Box<[Model]>,
    pub inspect: bool,
    pub modify: bool,
    pub shell: bool,
    pub agents: bool,
    pub call_timeout: skein_lib::Duration,
    /// The host families its runs are offered, a mask of the configured `ToolFamilies` as a
    /// task's authority's tools are; a family it or the task's authority leaves out is not
    /// offered (domain/engine/executors.md, 1).
    pub tools: jig_domain_tasks::Tools,
}

impl RunPolicy {
    /// Every family: the identity under a task's authority, and what a decoder whose
    /// record carries no mask fills in.
    pub const EVERY_FAMILY: jig_domain_tasks::Tools = jig_domain_tasks::Tools(u64::MAX);
}

/// The root's complete task-specific charter, kept behind one bounded
/// assignment cell so delivery queues carry a small fixed-size value. Its settings
/// revision is fixed when preparation begins (domain/engine/settings.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RunCharter {
    /// The settings revision recorded as this run began preparation.
    pub revision: u32,
    pub ended_by: tasks::EndedBy,
    /// Selected grant for main then each retained alternative; none sources have no grant.
    pub sources: Box<[Option<u32>]>,
    pub accounts: Box<[authority::AccountBinding]>,
    pub placement: jig_domain_fleet::PlacementConstraint,
    pub policy: RunPolicy,
    pub contract: tasks::Contract,
    pub authority: tasks::Authority,
    pub budget: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RoutedCall {
    Escalation { key: CallKey, task: u64, revision: u64, passed_up: bool },
    Propose { key: CallKey, proposal: u64 },
    Decide { key: CallKey, proposal: u64 },
    Withdraw { key: CallKey, proposal: u64, moved: Option<(u64, people::Moved)> },
    Accepting { key: CallKey, proposer: u64, proposal: u64, message: u64 },
    Message(CallKey),
    Introduce(CallKey),
    Subscribe { key: CallKey, subscription: u64 },
    Unsubscribe(CallKey),
    Control { key: CallKey, target: u64, revised: bool },
}

#[derive(Hash, Debug)]
pub struct RetainedResult {
    pub task: u64,
    pub kind: tasks::ResultKind,
    pub words: Box<[u8]>,
}

#[derive(Hash, Debug)]
pub struct PendingRelay {
    pub previous: Option<u64>,
    pub message: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PersonProposalRoute {
    Deciding { request: Token, proposer: u64, proposal: u64, by: u64, moved: Option<people::Moved> },
    Accepting { request: Token, person: u64, proposer: u64, proposal: u64, message: u64 },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PersonTaskRoute {
    Leave(u64),
    Compound { project: u32, operations: u32, first_message: u64 },
    PoolSet { project: u32, person: u64 },
    Take(u64),
    HandBack(u64),
    Answer(u64),
    Cancel(u64),
    Stop(u64),
    Release(u64),
    Lift(u64),
    Amended { task: u64, released: bool },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GoalRoute {
    Offering { proposal: u64, to: u32, stubs: Box<[tasks::Stub]> },
    Withdrawing { proposer: u64, proposal: u64, moved: Option<people::Moved> },
    Accepting { proposer: u64, proposal: u64, by: u64 },
    Deciding { proposer: u64, proposal: u64, by: u64 },
}

/// A completion's maximum price, rounded up once (domain/authority.md, 7).
#[must_use]
pub fn most_costly(model: &Model) -> Option<u64> {
    let rate = model.input_price.max(model.cached_price).max(model.cache_write_price.unwrap_or(model.input_price));
    let input = u128::from(model.window).checked_mul(u128::from(rate))?;
    let output = u128::from(model.output).checked_mul(u128::from(model.output_price))?;
    let sum = input.checked_add(output)?;
    let unit = u128::from(model.price_unit);
    let rounded = sum.checked_div(unit)?.checked_add(u128::from(sum.checked_rem(unit)? != 0))?;
    u64::try_from(rounded).ok()
}

/// Reserve turns and one completion more (domain/authority.md, 7).
#[must_use]
pub fn reserve_floor(policy: &RunPolicy) -> Option<u64> {
    most_costly(&policy.model)?.checked_mul(u64::from(policy.reserve_turns).checked_add(1)?)
}

impl Core {
    /// The spend a run of `task` reserves: its reserve turns and one completion more of its LLM
    /// choice's main model (domain/authority.md, 7 and 8.3).
    pub(crate) fn run_reserve(&self, task: u64) -> u64 {
        // No spend holds a reserve beyond the numbers' range, so no run of it starts.
        reserve_floor(&self.chosen_policy(task)).unwrap_or(u64::MAX)
    }
}

/// Map admission findings with the run's deadline, funding and resource precedence.
pub(crate) fn denied(findings: &mut Queue<authority::Finding>, wait: crate::RunWait) -> RunAdmission {
    let mut account = false;
    let mut hold = None;
    for _ in 0..findings.len() {
        match findings.pop().expect("authority finding count") {
            authority::Finding::Account => account = true,
            authority::Finding::Deadline => hold = Some(tasks::Hold::Deadline),
            authority::Finding::RunBudget
            | authority::Finding::RunCap
            | authority::Finding::Arithmetic
            | authority::Finding::Spend { .. }
            | authority::Finding::Price { .. } => {
                if hold != Some(tasks::Hold::Deadline) {
                    hold = Some(tasks::Hold::Budget);
                }
            }
            authority::Finding::ResourceUnavailable => {
                if hold.is_none() || hold == Some(tasks::Hold::Effects) || hold == Some(tasks::Hold::ResourceAccess) {
                    hold = Some(tasks::Hold::ResourceUnavailable);
                }
            }
            authority::Finding::ResourceAccess => {
                if hold.is_none() || hold == Some(tasks::Hold::Effects) {
                    hold = Some(tasks::Hold::ResourceAccess);
                }
            }
            authority::Finding::Oversized
            | authority::Finding::UnknownProject
            | authority::Finding::UnknownRole
            | authority::Finding::Authority { .. }
            | authority::Finding::Executor { .. }
            | authority::Finding::Tasks { .. }
            | authority::Finding::Writer
            | authority::Finding::Tool
            | authority::Finding::Grant { .. }
            | authority::Finding::Reference
            | authority::Finding::Scope { .. }
            | authority::Finding::Required { .. }
            | authority::Finding::Failed { .. }
            | authority::Finding::Unguarded { .. }
            | authority::Finding::Unpermitted
            | authority::Finding::Undecidable
            | authority::Finding::PeriodSpend => {
                if hold.is_none() {
                    hold = Some(tasks::Hold::Effects);
                }
            }
        }
    }
    match hold {
        Some(why) => RunAdmission::Hold(why),
        None if account => RunAdmission::Wait(wait),
        None => RunAdmission::Hold(tasks::Hold::Effects),
    }
}
