//! Chat admission, running and settlement transitions (domain/host.md, sections 2 and 8–9).
//! Each startup transition moves its grants and survey by value. Follow derives
//! the pending head request and the answer's store/child settlement barrier.
use super::{
    Agent, AgentFailure, Box, Chat, ChatState, Child, Config, DeliveryRecord, DeliveryState, Domain, Env, ExitStatus,
    Fact, Failure, GitOp, GitResult, HeadSurvey, Limits, List, Live, Map, Notice, Queue, Reconcile, Request, RunStart,
    Starting, State, StoreFailure, Token, Undelivered, agent, command, deliver_to_agent, host, mem, observe, parent,
    reconcile_next,
};
use crate::credentials::Grants;

pub(super) fn receive_line(domain: &mut Domain, env: &Env<Limits>, text: Box<[u8]>, out: &mut Queue<Request>) {
    match domain.config.mode {
        crate::Mode::Headless { .. } => return,
        crate::Mode::Chat { .. } => {}
    }
    if text.len() > usize::try_from(env.limits.line_bytes).expect("u32 fits usize") {
        out.push(Request::Notice { notice: Notice::NotDelivered { line: None, why: Undelivered::TooLong } });
        return;
    }
    domain.lines.receive(text).expect("input is held at the inbox plus the completed line");
    dispatch_line(domain, env, out);
}

pub(super) fn dispatch_line(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.chat.state_pending() || domain.lines.is_empty() {
        return;
    }
    match &domain.chat.phase {
        State::Idle | State::Running { .. } => {}
        State::Loading(_) | State::Starting(_) | State::Ending(_) | State::Answered { .. } | State::Done => return,
    }
    if domain.lines.received() {
        let Some(next_message) = domain.chat.state.next_message.checked_add(1) else {
            stop_names(domain, out);
            return;
        };
        domain.chat.state.next_message = next_message;
        domain.lines.name(Token::new(next_message));
    } else if domain.chat.phase.is_running() {
        return;
    }
    if domain.chat.phase.is_idle() {
        let Some(activation) = domain.chat.state.activation.checked_add(1) else {
            stop_names(domain, out);
            return;
        };
        domain.chat.state.activation = activation;
        domain.chat.phase = State::Starting(Starting {
            grants: Grants {
                values: List::with_capacity(env.limits.smith.accounts),
                awaiting: u32::try_from(domain.config.accounts.len()).expect("validated account count"),
            },
            grant_failed: None,
            saved: false,
            survey: HeadSurvey::NotStarted,
            heads: Map::with_capacity(env.limits.smith.run.directories),
        });
        for account in &domain.config.accounts {
            out.push(Request::Credential { account: *account });
        }
    }
    domain.chat.metadata_started();
    let fresh = domain.chat.phase.is_starting() && !domain.config.resume;
    out.push(Request::SaveState { state: domain.chat.state, fresh });
}

pub(super) fn stop_names(domain: &mut Domain, out: &mut Queue<Request>) {
    domain.chat.phase = State::Done;
    out.push(Request::Notice { notice: Notice::Failure(Failure::NamesExhausted) });
    out.push(Request::Exit { status: crate::end::exit_status(domain.ending.cause()) });
}

pub(crate) fn select_chat(
    domain: &mut Domain,
    env: &Env<Limits>,
    name: Box<[u8]>,
    create: bool,
    out: &mut Queue<Request>,
) {
    let refusal = if name.len() > usize::try_from(env.limits.chat_bytes).expect("bounded chat name") {
        Some(crate::ChatRefusal::Name)
    } else if !domain.chat.phase.is_idle()
        || domain.child.run().is_some()
        || domain.chat.state_pending()
        || !domain.turns.unsaved.is_empty()
        || !domain.lines.is_empty()
        || domain.saving.is_some()
    {
        Some(crate::ChatRefusal::Busy)
    } else {
        None
    };
    if let Some(reason) = refusal {
        out.push(Request::Notice { notice: Notice::ChatRefused { name, reason } });
        return;
    }
    if domain.chat_seed == u64::MAX {
        stop_names(domain, out);
        return;
    }
    domain.chat.phase = State::Loading(crate::chat::Load::Ready);
    domain.chat.request_load();
    out.push(Request::SelectChat { name, create });
}

pub(super) fn chat_selected(
    domain: &mut Domain,
    name: Box<[u8]>,
    result: Result<(), crate::ChatRefusal>,
    out: &mut Queue<Request>,
) {
    if domain.ending.store_failed() {
        return;
    }
    assert!(domain.child.run().is_none() && !domain.chat.state_pending(), "selection ends between activations");
    assert!(domain.chat.phase.is_loading() && domain.chat.load_requested(), "selection has one terminal");
    match result {
        Ok(()) => {
            let closed = domain.chat.closed();
            domain.chat_seed = match domain.chat_seed.checked_add(1) {
                Some(seed) => seed,
                None => {
                    stop_names(domain, out);
                    return;
                }
            };
            domain.chat = Chat::new(domain.chat_seed);
            if closed {
                domain.chat.close_input();
            }
            domain.chat.request_load();
            domain.config.chat = name;
            out.push(Request::Load);
        }
        Err(reason) => {
            domain.chat.phase = State::Idle;
            out.push(Request::Notice { notice: Notice::ChatRefused { name, reason } });
            if domain.chat.closed() {
                closed(domain, out);
            }
        }
    }
}

pub(super) fn loaded(
    domain: &mut Domain,
    env: &Env<Limits>,
    state: Option<ChatState>,
    transcript: Option<smith_host_domain::Transcript>,
    deliveries: Box<[DeliveryRecord]>,
    out: &mut Queue<Request>,
) {
    assert!(
        domain.chat.phase.is_loading() && domain.chat.load_requested(),
        "Load has one terminal: phase {:?}, requested {}",
        domain.chat.phase,
        domain.chat.load_requested()
    );
    match state {
        Some(mut saved) => {
            if !domain.config.resume {
                saved.identity = domain.chat.state.identity;
            }
            domain.chat.state = saved;
        }
        None => {
            domain.chat.state =
                ChatState { identity: domain.chat.state.identity, activation: 0, next_message: 0, read: None };
        }
    }
    domain.transcript = transcript;
    domain.records = Map::with_capacity(env.limits.smith.run.answered_calls);
    for record in deliveries {
        match domain.records.insert(record.name, record) {
            Ok(None) => {}
            Ok(Some(_)) | Err(_) => {
                store_failed(domain, env, None, StoreFailure::Read, out);
                return;
            }
        }
    }
    resume_loading(domain, env, out);
}

pub(super) fn resume_loading(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    for (_, record) in &domain.records {
        match &record.state {
            DeliveryState::Intent(_) => {
                domain.reconciling = Some(Reconcile::new(record.clone()));
                reconcile_next(domain, env, out);
                return;
            }
            DeliveryState::Answer(_) => {}
        }
    }
    domain.chat.phase = State::Idle;
    crate::choices::apply(domain, env, out);
    if !domain.chat.phase.is_idle() {
        return;
    }
    if domain.chat.closed() && domain.lines.is_empty() {
        domain.chat.phase = State::Done;
        let status = match domain.config.mode {
            crate::Mode::Headless { .. } => crate::end::exit_status(domain.ending.cause()),
            crate::Mode::Chat { .. } => ExitStatus::Accepted,
        };
        out.push(Request::Exit { status });
    } else {
        dispatch_line(domain, env, out);
        if domain.interactive && domain.chat.phase.is_idle() && domain.lines.is_empty() {
            out.push(Request::Notice { notice: Notice::Waiting });
        }
    }
}

pub(super) fn state_saved(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    assert!(domain.chat.state_pending(), "StateSaved answers one SaveState");
    domain.chat.metadata_saved();
    domain.lines.saved();
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Starting(mut starting) => {
            starting.saved = true;
            State::Starting(starting)
        }
        state @ (State::Loading(_)
        | State::Idle
        | State::Running { .. }
        | State::Ending(_)
        | State::Answered { .. }
        | State::Done) => state,
    };
    if domain.chat.phase.is_running() {
        send_line(domain, env, out);
    }
    maybe_start(domain, env, out);
}

pub(super) fn credential(
    domain: &mut Domain,
    env: &Env<Limits>,
    grant: agent::Grant,
    form: host::Form,
    out: &mut Queue<Request>,
) {
    let grant =
        host::Grant { account: grant.name.account, generation: grant.name.generation, valid: grant.valid, form };
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Starting(starting) => granted(starting, grant, &domain.config),
        state @ (State::Running { .. } | State::Ending(_)) => {
            command(
                &mut domain.agent,
                env.now,
                env.wall,
                parent::Event::Grant { agent: domain.child.run().expect("grant belongs to the admitted child"), grant },
            );
            state
        }
        state @ State::Answered { .. } => state,
        State::Loading(_) | State::Idle | State::Done => unreachable!("grant answers a live account request"),
    };
    maybe_start(domain, env, out);
}

pub(super) fn granted(mut starting: Starting, grant: host::Grant, config: &Config) -> State {
    assert!(config.accounts.contains(&grant.account), "grant names a requested account");
    assert!(starting.grants.awaiting > 0, "one terminal per account request");
    starting.grants.awaiting = starting.grants.awaiting.checked_sub(1).expect("pending account");
    starting.grants.values.push(grant).expect("validated account capacity");
    State::Starting(starting)
}

pub(super) fn no_credential(
    domain: &mut Domain,
    env: &Env<Limits>,
    account: u32,
    reason: crate::CredentialFailure,
    out: &mut Queue<Request>,
) {
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Starting(mut starting) => {
            assert!(starting.grants.awaiting > 0, "one terminal per account request");
            starting.grants.awaiting = starting.grants.awaiting.checked_sub(1).expect("pending account");
            starting.grant_failed = Some((account, reason));
            State::Starting(starting)
        }
        state @ (State::Running { .. } | State::Ending(_)) => {
            out.push(Request::Notice { notice: Notice::Failure(Failure::Credential { account, reason }) });
            state
        }
        state @ State::Answered { .. } => state,
        State::Loading(_) | State::Idle | State::Done => unreachable!("missing grant answers a live account request"),
    };
    maybe_start(domain, env, out);
}

pub(super) fn maybe_start(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Starting(starting) => start_progress(domain, starting, env, out),
        state @ (State::Loading(_)
        | State::Idle
        | State::Running { .. }
        | State::Ending(_)
        | State::Answered { .. }
        | State::Done) => state,
    };
    follow_chat(domain, env, out);
}

pub(super) fn follow_chat(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Starting(mut starting) => {
            let survey = mem::replace(&mut starting.survey, HeadSurvey::Complete);
            starting.survey = match survey {
                HeadSurvey::Ready(directory) => {
                    out.push(Request::Git {
                        owner: Token::new(0),
                        directory,
                        op: GitOp::Head,
                        deadline: env.now.saturating_add(domain.config.budget.time),
                    });
                    HeadSurvey::Reading(directory)
                }
                survey @ (HeadSurvey::NotStarted | HeadSurvey::Reading(_) | HeadSurvey::Complete) => survey,
            };
            State::Starting(starting)
        }
        state @ (State::Loading(_)
        | State::Idle
        | State::Running { .. }
        | State::Ending(_)
        | State::Answered { .. }
        | State::Done) => state,
    };
    match &domain.chat.phase {
        State::Answered { .. } if domain.turns.unsaved.is_empty() => crate::person::finish_answer(domain, out),
        State::Idle if domain.chat.closed() => closed(domain, out),
        State::Loading(_)
        | State::Idle
        | State::Starting(_)
        | State::Running { .. }
        | State::Ending(_)
        | State::Answered { .. }
        | State::Done => {}
    }
}

pub(super) fn start_progress(
    domain: &mut Domain,
    mut starting: Starting,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) -> State {
    if !starting.saved || starting.grants.awaiting > 0 {
        return State::Starting(starting);
    }
    if domain.ending.requested().is_some() {
        crate::person::show_dropped(&mut domain.lines, out);
        out.push(Request::Exit { status: ExitStatus::Interrupted });
        return State::Done;
    }
    if domain.chat.closed() {
        match domain.config.mode {
            crate::Mode::Headless { .. } => {
                out.push(Request::Exit { status: ExitStatus::Interrupted });
                return State::Done;
            }
            crate::Mode::Chat { .. } => {}
        }
    }
    if let Some((account, reason)) = starting.grant_failed {
        crate::person::show_dropped(&mut domain.lines, out);
        out.push(Request::Notice { notice: Notice::Failure(Failure::Credential { account, reason }) });
        return match domain.config.mode {
            crate::Mode::Headless { .. } => {
                out.push(Request::Exit { status: ExitStatus::Credentials { account } });
                State::Done
            }
            crate::Mode::Chat { .. } => State::Idle,
        };
    }
    match starting.survey {
        HeadSurvey::NotStarted => starting = survey_heads(starting, &domain.config, 0),
        HeadSurvey::Ready(_) | HeadSurvey::Reading(_) => return State::Starting(starting),
        HeadSurvey::Complete => {}
    }
    match starting.survey {
        HeadSurvey::Ready(_) | HeadSurvey::Reading(_) => return State::Starting(starting),
        HeadSurvey::Complete => {}
        HeadSurvey::NotStarted => unreachable!("survey chooses a repository or completes"),
    }
    let history = if domain.config.resume { domain.transcript.take() } else { None };
    let mut answered = List::with_capacity(env.limits.smith.run.answered_calls);
    if domain.config.resume {
        for (name, record) in &domain.records {
            match &record.state {
                DeliveryState::Answer(delivery) => answered
                    .push(agent::AnsweredCall {
                        name: *name,
                        tool: Box::from(&b"deliver"[..]),
                        answer: agent::Answered::Delivery(Box::new(delivery.clone())),
                    })
                    .expect("one turn's calls fit the bound"),
                DeliveryState::Intent(_) => unreachable!("intents reconciled before starting"),
            }
        }
    } else {
        domain.records = Map::with_capacity(env.limits.smith.run.answered_calls);
    }

    domain.child = Child::Starting;

    domain.ending = crate::end::End::new();
    domain.told = crate::end::Told::new();

    let messages = domain.lines.carry();
    let mut initial = List::with_capacity(env.limits.smith.run.messages.saturating_add(1));
    for message in &messages {
        initial.push(message.name).expect("bounded carried lines");
        observe_initial_message(domain, message);
    }
    let source = RunStart {
        live_text: domain.config.live_text,
        messages,
        identity: domain.chat.state.identity,
        activation: domain.chat.state.activation,
        charter: domain.charter().0,
        workspace: domain.config.workspace.clone(),
        transcript: history,
        answered: answered.into_boxed(),
        grants: starting.grants.values.into_boxed(),
    };
    match domain.agent.prepare(source) {
        Ok(start) => {
            command(&mut domain.agent, env.now, env.wall, parent::Event::Spawn { client: Token::new(1), start });
        }
        Err(error) => refused_preparation(domain, error, out),
    }
    observe(domain, Fact::Started { activation: domain.chat.state.activation });
    for name in &initial {
        observe(domain, Fact::Message { name: *name });
    }
    State::Running {
        live: Live { heads: starting.heads, own_heads: Map::with_capacity(env.limits.smith.run.directories) },
        deadline: env.now.saturating_add(domain.config.budget.time),
    }
}

pub(super) fn refused_preparation(domain: &mut Domain, error: crate::agent::BridgeError, out: &mut Queue<Request>) {
    domain.child.faulted(AgentFailure::NotStarted);
    let invalid = crate::agent::preparation_refusal(error);
    domain.child.gone();
    crate::person::show_dropped(&mut domain.lines, out);
    out.push(Request::Notice { notice: Notice::Failure(Failure::Unstarted(host::End::Invalid(invalid))) });
    out.push(Request::Exit { status: crate::end::exit_status(domain.ending.cause()) });
}

pub(super) fn observe_initial_message(domain: &mut Domain, message: &host::Message) {
    match &domain.agent {
        Agent::Inline { .. } => domain.run_messages.push(message.clone()),
        Agent::Spawned { .. } => {}
    }
}

pub(super) fn survey_heads(mut starting: Starting, config: &Config, next: u32) -> Starting {
    if let Some(workspace) = &config.workspace {
        for (index, directory) in
            workspace.directories.iter().enumerate().skip(usize::try_from(next).expect("bounded position"))
        {
            if directory.writable && directory.git {
                starting.survey = HeadSurvey::Ready(u32::try_from(index).expect("admitted directory count"));
                return starting;
            }
        }
    }
    starting.survey = HeadSurvey::Complete;
    starting
}

pub(super) fn head_result(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: GitResult,
    out: &mut Queue<Request>,
) {
    assert_eq!(owner, Token::new(0), "start head inspection uses its own owner");
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Starting(starting) => head_read(domain, starting, result, out),
        State::Loading(_)
        | State::Idle
        | State::Running { .. }
        | State::Ending(_)
        | State::Answered { .. }
        | State::Done => unreachable!("head inspection belongs to startup"),
    };
    maybe_start(domain, env, out);
}

pub(super) fn head_read(
    domain: &mut Domain,
    mut starting: Starting,
    result: GitResult,
    out: &mut Queue<Request>,
) -> State {
    let directory = match starting.survey {
        HeadSurvey::Reading(directory) => directory,
        HeadSurvey::NotStarted | HeadSurvey::Ready(_) | HeadSurvey::Complete => {
            unreachable!("head inspection is active")
        }
    };
    match result {
        GitResult::Head { head } if !head.is_empty() && head.len() <= agent::run::Receipt::CAPACITY => {
            starting.heads.insert(directory, head).expect("one head per writable repository");
            State::Starting(survey_heads(
                starting,
                &domain.config,
                directory.checked_add(1).expect("admitted position"),
            ))
        }
        GitResult::Head { .. } | GitResult::Failed { .. } => {
            crate::person::show_dropped(&mut domain.lines, out);
            let directory = domain
                .config
                .workspace
                .as_ref()
                .expect("head survey has a workspace")
                .directories
                .get(usize::try_from(directory).expect("directory position fits usize"))
                .expect("head survey names a declared directory")
                .name
                .clone();
            out.push(Request::Notice { notice: Notice::Failure(Failure::HeadUnavailable { directory }) });
            match domain.config.mode {
                crate::Mode::Headless { .. } => {
                    out.push(Request::Exit { status: ExitStatus::Refused });
                    State::Done
                }
                crate::Mode::Chat { .. } => State::Idle,
            }
        }
        GitResult::Status { .. }
        | GitResult::Inspected { .. }
        | GitResult::Markers { .. }
        | GitResult::Committed { .. }
        | GitResult::Pushed
        | GitResult::Stale
        | GitResult::NoEffect { .. }
        | GitResult::Uncertain { .. } => unreachable!("start head inspection has one terminal"),
    }
}

pub(super) fn send_line(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if !domain.chat.phase.is_running() {
        return;
    }
    let Some(run) = domain.child.run() else { return };
    for message in domain.lines.relay(false) {
        let name = message.name;
        observe_initial_message(domain, &message);
        command(
            &mut domain.agent,
            env.now,
            env.wall,
            parent::Event::Message {
                finish_requested: message.finish_requested,
                agent: run,
                name,
                label: message.label,
                text: message.text,
            },
        );
        observe(domain, Fact::Message { name });
    }
    dispatch_line(domain, env, out);
}

pub(super) fn turn_saved(domain: &mut Domain, env: &Env<Limits>, number: u32, out: &mut Queue<Request>) {
    if domain.ending.store_failed() {
        return;
    }
    let (pending, sequence) = domain.turns.unsaved.pop().expect("TurnSaved answers a pending SaveTurn");
    assert_eq!(number, pending, "turns become durable in order");
    if let Some(run) = domain.child.run() {
        command(&mut domain.agent, env.now, env.wall, parent::Event::Acknowledge { agent: run, turn: number });
    }
    let mut retired = List::with_capacity(env.limits.smith.run.answered_calls);
    for (name, record) in &domain.records {
        let answered = match record.state {
            DeliveryState::Intent(_) => false,
            DeliveryState::Answer(_) => true,
        };
        if answered && (name.activation != domain.chat.state.activation || name.completion <= sequence) {
            retired.push(*name).expect("bounded saved answers");
        }
    }
    for name in &retired {
        domain.records.remove(name);
    }
    if domain.turns.unsaved.is_empty() {
        if domain.child.failure().is_some() && domain.child.is_gone() {
            domain.chat.phase = State::Done;
            out.push(Request::Exit { status: crate::end::exit_status(domain.ending.cause()) });
        } else {
            crate::person::finish_answer(domain, out);
        }
    }
}

pub(super) fn interrupt(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    match domain.config.mode {
        crate::Mode::Headless { .. } => domain.ending.record(crate::EndCause::Cancelled),
        crate::Mode::Chat { .. } => {}
    }
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Idle => {
            out.push(Request::Exit { status: ExitStatus::Accepted });
            State::Done
        }
        State::Running { live, .. } => {
            domain.ending.cancel(env.now);
            if let Some(run) = domain.child.run() {
                command(&mut domain.agent, env.now, env.wall, parent::Event::Stop { agent: run });
            }
            State::Ending(live)
        }
        State::Starting(starting) => {
            domain.chat.close_input();
            domain.ending.cancel(env.now);
            State::Starting(starting)
        }
        state @ (State::Loading(_) | State::Ending(_) | State::Answered { .. } | State::Done) => {
            domain.chat.close_input();
            state
        }
    };
    follow_chat(domain, env, out);
}

pub(crate) fn closed(domain: &mut Domain, out: &mut Queue<Request>) {
    match domain.config.mode {
        crate::Mode::Headless { .. } => return,
        crate::Mode::Chat { .. } => {}
    }
    if domain.interactive && !domain.chat.closed() {
        let extra = if domain.child.run().is_some() || domain.chat.phase.is_starting() { 1 } else { 2 };
        domain.closing_last = Some(domain.chat.state.activation.saturating_add(extra));
    }
    domain.chat.close_input();
    if domain.chat.phase.is_idle() && domain.lines.is_empty() {
        domain.chat.phase = State::Done;
        out.push(Request::Exit { status: ExitStatus::Accepted });
    }
}

pub(super) fn drop_run(domain: &mut Domain, env: &Env<Limits>) {
    if domain.chat.phase.is_answered() {
        return;
    }
    if let Some(run) = domain.child.run() {
        domain.ending.cancel(env.now);
        domain.ending.record(crate::EndCause::Cancelled);
        end_chat(domain);

        command(&mut domain.agent, env.now, env.wall, parent::Event::Drop { agent: run });
    }
}

pub(super) fn store_failed(
    domain: &mut Domain,
    env: &Env<Limits>,
    turn: Option<u32>,
    reason: StoreFailure,
    out: &mut Queue<Request>,
) {
    if domain.ending.store_failed() {
        return;
    }
    domain.ending.record(crate::EndCause::Host { turn, reason });
    domain.ending.fail_store();
    crate::person::show_dropped(&mut domain.lines, out);
    domain.chat.close_input();
    domain.turns.unsaved = Queue::with_capacity(env.limits.unsaved);
    out.push(Request::Notice { notice: Notice::Failure(Failure::Store { turn, reason }) });
    if let Some(record) = domain.saving.take() {
        let owner = domain.delivery_owner.take();
        match &record.state {
            DeliveryState::Answer(agent::run::Delivery::Delivered(_)) => {
                out.push(Request::Notice { notice: Notice::Failure(Failure::DeliveryUnsaved) });
            }
            DeliveryState::Intent(_)
            | DeliveryState::Answer(
                agent::run::Delivery::Nothing
                | agent::run::Delivery::Refused(_)
                | agent::run::Delivery::Failed(_)
                | agent::run::Delivery::Stale,
            ) => {}
        }
        let delivery = match record.state {
            DeliveryState::Intent(_) => {
                agent::run::Delivery::Failed(agent::run::DeliveryFailure::new(0, agent::run::DeliveryReason::Broken))
            }
            DeliveryState::Answer(delivery) => delivery,
        };
        if let Some(owner) = owner {
            deliver_to_agent(domain, env, owner, delivery);
        }
    }
    if let Some(run) = domain.child.run() {
        command(&mut domain.agent, env.now, env.wall, parent::Event::Drop { agent: run });
        end_chat(domain);
    } else {
        domain.chat.phase = State::Done;
        out.push(Request::Exit { status: crate::end::exit_status(domain.ending.cause()) });
    }
}

pub(super) fn end_chat(domain: &mut Domain) {
    let state = mem::replace(&mut domain.chat.phase, State::Done);
    domain.chat.phase = match state {
        State::Running { live, .. } | State::Ending(live) => State::Ending(live),
        state @ (State::Loading(_) | State::Idle | State::Starting(_) | State::Answered { .. } | State::Done) => state,
    };
}
