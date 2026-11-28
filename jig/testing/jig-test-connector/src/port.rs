//! The test connector's side of the engine port (jig/docs/design/domain/connectors.md, 2).

use crate::{Domain, Effect, Event, Limits, Path, Record, RecordKey, Request, SystemRequest};
use alloc::boxed::Box;
use jig::{effects, lifecycle, ops, parts, procedures, projections, requests, resources, settings, topics};
use jig_domain_contract as jig;
use skein_lib::{Decimal, Env, List, Map, Queue, ReplyTo, Token, Writer};

/// Contract answers and requests to the testing application's own peers.
#[derive(Debug)]
pub enum Out {
    Up(jig::Up),
    Save(Record),
    Erase(RecordKey),
    System(SystemRequest),
    LostRead { task: u64, attempt: u64, resource: resources::Name, call: SystemRequest },
    Procedure { task: u64, step: u64, code: u16, context: Box<ops::RunContext> },
    Projection(Box<projections::ProjectionFeed>),
    EffectOwner { effect: Effect, origin: effects::EffectOrigin },
    ApplicationRelease { request: Token, project: u32, release: crate::ApplicationRelease },
    ReadAnswered { reply: ReplyTo, answer: Result<crate::ClientDocument, crate::DocumentMissing> },
    Topic { token: Token, resources: Option<Box<[effects::Effect]>> },
    RestoreRefused { kind: u32, scope: u32, number: u32 },
    RestoreResource { project: u32, name: resources::Name, role: resources::ResourceRole, hold: resources::HoldKind },
}

#[derive(Debug)]
pub(crate) struct Port {
    number: u16,
    inputs: Map<Token, Effect>,
    effect_tasks: Map<Token, u64>,
    outbox: Map<u64, (u64, crate::Purpose)>,
    steps: Map<u64, (u64, u32, crate::steps::StepContext)>,
    interests: Map<u64, (u64, u16)>,
    documents: Map<Token, crate::ClientDocument>,
    parts: Option<Queue<parts::PartsAsk>>,
    releases: Option<Queue<u32>>,
    delay_reports: Queue<bool>,
    reports: Queue<resources::Up>,
    waits: Option<Queue<procedures::Up>>,
}

impl Port {
    pub(crate) fn new(limits: &Limits) -> Self {
        Self {
            number: 1,
            inputs: Map::with_capacity(limits.staged.saturating_add(1)),
            effect_tasks: Map::with_capacity(limits.procedures),
            outbox: Map::with_capacity(limits.entries.saturating_add(1)),
            steps: Map::with_capacity(limits.procedures),
            interests: Map::with_capacity(limits.subscriptions),
            documents: Map::with_capacity(limits.values),
            parts: None,
            releases: None,
            delay_reports: Queue::uncapped(),
            reports: Queue::uncapped(),
            waits: None,
        }
    }
}

impl Domain {
    /// Bind the number used inside qualified contract resource names.
    pub fn address(&mut self, number: jig::Connector) {
        self.port.number = number.0;
    }

    pub fn take_document(&mut self, watcher: Token) -> Option<crate::ClientDocument> {
        self.port.documents.remove(&watcher)
    }

    pub fn script_parts(&mut self) {
        self.port.parts = Some(Queue::uncapped());
    }

    pub fn take_parts(&mut self) -> Option<parts::PartsAsk> {
        self.port.parts.as_mut()?.pop()
    }

    pub fn script_project_releases(&mut self) {
        if self.port.releases.is_none() {
            self.port.releases = Some(Queue::uncapped());
        }
    }

    pub fn take_project_release(&mut self) -> Option<u32> {
        self.port.releases.as_mut()?.pop()
    }

    /// The resource names this connector contributes to a run's workspace preparation.
    pub fn workspace_holdings(
        &mut self,
        project: u32,
        spec: &ops::Spec,
        out: &mut Queue<Out>,
    ) -> Option<Box<[resources::Holding]>> {
        holdings(self, project, spec, out)
    }

    /// Delay selected observations, never the judgement that names the holdings.
    pub fn delay_holding_reports(&mut self, choices: Box<[bool]>) {
        for delayed in choices {
            self.port.delay_reports.push(delayed);
        }
    }

    pub fn take_holding_report(&mut self) -> Option<resources::Up> {
        self.port.reports.pop()
    }

    pub fn script_wait_reports(&mut self) {
        self.port.waits = Some(Queue::uncapped());
    }

    pub fn take_wait_report(&mut self) -> Option<procedures::Up> {
        self.port.waits.as_mut()?.pop()
    }

    /// Join restored topic rows to restored subscription numbers without changing either record.
    pub fn restore_interest(&mut self, task: u64, topic: u16, number: u64) {
        if self.subscriptions.contains_key(&(topic, task)) {
            self.port.interests.insert(number, (task, topic)).expect("one binding per kept subscription");
        }
    }
}

pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Out>) {
    let mut raw = Queue::uncapped();
    if let Event::Ask(ask) = event {
        asked(domain, env, ask, &mut raw, out);
    } else if let Event::Released(released) = event {
        match released {
            jig::Released::Make { entry } => crate::step_system(domain, env, Event::Make { entry }, &mut raw),
            jig::Released::Answered { .. } => {} // the fake sends no deployment acts
        }
    } else if let Event::StageEffect { owner, effect, start } = event {
        domain.port.inputs.insert(owner, effect).expect("admitted effect owner");
        if let Some(origin) = start {
            if let effects::EffectOrigin::Procedure { task, .. } = &origin {
                domain.port.effect_tasks.insert(owner, *task).expect("admitted procedure effect");
            }
            say_effect(out, effects::Up::Start { owner, origin });
        }
    } else if let Event::WriterRead { resource, event } = event {
        if let crate::SystemEvent::Fact { resource: observed, fact, .. }
        | crate::SystemEvent::Settle { resource: observed, fact } = &event
            && !fact.pending
            && observed.segments() == resource.path.as_ref()
        {
            crate::step_system(domain, env, Event::System(event), &mut raw);
            reported(domain, &mut raw, false, out);
            out.push(Out::Up(jig::Up::Resources(resources::Up::ReadAfresh { resource })));
        }
    } else if let Event::Restore { record } = event {
        if let Record::Outbox(entry) = &record {
            kept(domain, entry.number, entry.task, entry.key.origin);
        }
        crate::step_system(domain, env, Event::Restore { record }, &mut raw);
        reported(domain, &mut raw, true, out);
    } else {
        crate::step_system(domain, env, event, &mut raw);
    }
    reported(domain, &mut raw, false, out);
}

pub fn resume(domain: &mut Domain, out: &mut Queue<Out>) {
    let mut raw = Queue::uncapped();
    crate::domain::resume_system(domain, &mut raw);
    reported(domain, &mut raw, false, out);
}

pub fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Out>) {
    let mut raw = Queue::uncapped();
    crate::fire_system(domain, env, &mut raw);
    reported(domain, &mut raw, false, out);
}

fn asked(domain: &mut Domain, env: &Env<Limits>, ask: jig::Ask, raw: &mut Queue<Request>, out: &mut Queue<Out>) {
    let event = match ask {
        jig::Ask::Lifecycle(ask) => match ask {
            lifecycle::Ask::CatchUp => {
                let mut procedures = List::with_capacity(domain.procedures.len());
                for (task, _) in &domain.procedures {
                    procedures.push(*task).expect("one report per procedure");
                }
                Event::CatchUp { procedures: procedures.into_boxed() }
            }
            lifecycle::Ask::Adopt { project, adoption, .. } => Event::Adopt {
                project,
                resource: path(adoption.resource.path),
                role: match adoption.role {
                    lifecycle::ResourceRole::Owned => crate::ResourceRole::Owned,
                    lifecycle::ResourceRole::Fork => crate::ResourceRole::Participating,
                    lifecycle::ResourceRole::Context => crate::ResourceRole::Context,
                },
            },
            lifecycle::Ask::ReleaseProject { project } => {
                if let Some(releases) = &mut domain.port.releases {
                    releases.push(project);
                    return;
                }
                Event::ReleaseProject { project }
            }
        },
        jig::Ask::Resources(ask) => {
            resource_ask(domain, env, ask, raw, out);
            return;
        }
        jig::Ask::Effects(ask) => {
            effect_ask(domain, env, ask, raw, out);
            return;
        }
        jig::Ask::Topics(ask) => {
            topic_ask(domain, env, ask, raw);
            return;
        }
        jig::Ask::Parts(ask) => {
            part_ask(domain, env, ask, raw, out);
            return;
        }
        jig::Ask::Procedures(ask) => match ask {
            procedures::Ask::Start { context, step, chat: _ } => {
                start_procedure(domain, context, step, out);
                return;
            }
            procedures::Ask::StepMade { task, step, first, revisions } => {
                domain.port.steps.remove(&task);
                Event::StepMade { task, step, first, revisions }
            }
            procedures::Ask::StepRefused { task, step, why } => {
                Event::StepRefused { task, step, why: crate::steps::refused(why) }
            }
            procedures::Ask::WaitsRefused { task, why } => Event::WaitsRefused { task, why: wait_refusal(why) },
        },
        jig::Ask::Projections(ask) => {
            match ask {
                projections::Ask::Project { feed } => out.push(Out::Projection(feed)),
                projections::Ask::Forget { .. } => {} // the system peer has no projection cache
            }
            return;
        }
        jig::Ask::Settings(ask) => setting_ask(ask),
        jig::Ask::Requests(ask) => match ask {
            requests::Ask::Decide { request, token, owner, person, scope, role, retention } => Event::Application {
                request,
                token,
                owner,
                person,
                scope: scope_of(scope),
                role: request_role(role),
                retention,
            },
            requests::Ask::Drop { owner } => Event::ApplicationDrop { owner },
            requests::Ask::PartAnswered { request, outcome } => {
                Event::ApplicationCore { request, answer: application_answer(outcome) }
            }
        },
    };
    crate::step_system(domain, env, event, raw);
}

fn start_procedure(domain: &mut Domain, context: Box<ops::RunContext>, step: u64, out: &mut Queue<Out>) {
    match context.executor {
        ops::Executor::Procedure { code, .. } => {
            let task = context.task;
            let snapshot =
                crate::steps::StepContext { task, authority: context.authority.clone(), numbers: context.numbers };
            domain.port.steps.insert(task, (step, code, snapshot)).expect("one step per procedure");
            out.push(Out::Procedure { task, step, code: u16::try_from(code).expect("test procedure code"), context });
        }
        ops::Executor::Job { .. } => {}
        ops::Executor::Agent { .. } | ops::Executor::Person(_) => {
            unreachable!("procedure start has a procedure executor")
        }
    }
}

fn resource_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: resources::Ask,
    raw: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    let up = match ask {
        resources::Ask::Unname { task } => {
            crate::step_system(domain, env, Event::Unnamed { task }, raw);
            domain.port.steps.remove(&task);
            return;
        }
        resources::Ask::Hold { .. } | resources::Ask::RepairRefused { .. } | resources::Ask::DelegateRefused { .. } => {
            return;
        }
        resources::Ask::Lost { task, attempt, resources } => {
            for resource in resources {
                let call = SystemRequest::ReadFact { resource: path(resource.path.clone()), observed: env.wall };
                out.push(Out::LostRead { task, attempt, resource, call });
            }
            return;
        }
        resources::Ask::Holdings { request, project, spec, .. } => {
            resources::Up::Holdings { request, holdings: holdings(domain, project, &spec, out), refused: None }
        }
        resources::Ask::DelegateHoldings { request, members, .. } => {
            resources::Up::DelegateHoldings { request, holdings: batch_holdings(domain, &members, out), refused: None }
        }
        resources::Ask::ProcedureHoldings { task, step, members } => resources::Up::ProcedureHoldings {
            task,
            step,
            holdings: batch_holdings(domain, &members, out),
            refused: None,
        },
    };
    out.push(Out::Up(jig::Up::Resources(up)));
}

fn batch_holdings(
    domain: &mut Domain,
    members: &[resources::HoldingsNeed],
    out: &mut Queue<Out>,
) -> Option<Box<[Box<[resources::Holding]>]>> {
    let mut batch = List::with_capacity(u32::try_from(members.len()).expect("bounded batch"));
    let mut valid = true;
    for member in members {
        if let Some(named) = holdings(domain, member.project, &member.spec, out) {
            batch.push(named).expect("batch room");
        } else {
            valid = false;
        }
    }
    if valid { Some(batch.into_boxed()) } else { None }
}

fn holdings(
    domain: &mut Domain,
    project: u32,
    spec: &ops::Spec,
    out: &mut Queue<Out>,
) -> Option<Box<[resources::Holding]>> {
    let mut holdings = List::with_capacity(u32::try_from(spec.parameters.len()).ok()?);
    for parameter in &spec.parameters {
        match parameter {
            ops::Parameter::Resource { connector, resource, .. } if *connector == domain.port.number => {
                let described = domain.specification_resource(*resource)?;
                let resource = named(domain, &described.path);
                let role = domain.resource_role(project, &described.path).unwrap_or(crate::ResourceRole::Unavailable);
                let report = resource_report(domain, project, &described.path, role, described.hold);
                let holding = match described.hold {
                    crate::Hold::Exclusive { .. } => Some(resources::Holding::Write { resource, kind: 1 }),
                    crate::Hold::Pooled { .. } => Some(resources::Holding::Slot { pool: resource, kind: 2 }),
                    crate::Hold::None => None,
                };
                if domain.port.delay_reports.pop() == Some(true) {
                    domain.port.reports.push(report);
                } else {
                    out.push(Out::Up(jig::Up::Resources(report)));
                }
                if let Some(holding) = holding {
                    holdings.push(holding).ok()?;
                }
            }
            ops::Parameter::Resource { .. } | ops::Parameter::Number { .. } | ops::Parameter::Bytes { .. } => {}
        }
    }
    Some(holdings.into_boxed())
}

fn effect_ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    ask: effects::Ask,
    raw: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    let mut release = None;
    let event = match ask {
        effects::Ask::Describe { owner } => {
            let Some(effect) = domain.port.inputs.remove(&owner) else {
                say_effect(out, effects::Up::Refused { owner });
                return;
            };
            Event::Describe { token: owner, effect }
        }
        effects::Ask::DescribeProposal { owner, proposal } => {
            Event::DescribeProposal { token: owner, number: proposal }
        }
        effects::Ask::Keep { owner, entry, task, key, project: _ } => {
            domain.port.effect_tasks.remove(&owner);
            let origin = match key.origin {
                effects::EffectPurpose::Call { attempt, completion, position } => {
                    crate::Purpose::Call { attempt, completion, position }
                }
                effects::EffectPurpose::Procedure { .. } => crate::Purpose::Procedure,
                effects::EffectPurpose::Projection { .. } => crate::Purpose::Projection,
            };
            kept(domain, entry, task, origin);
            Event::Keep {
                token: owner,
                entry,
                task,
                key: crate::Key { deployment: key.deployment, task: key.task, purpose: key.purpose, origin },
            }
        }
        effects::Ask::KeepProposal { owner, proposal, task, project } => {
            Event::KeepProposal { token: owner, number: proposal, task, project }
        }
        effects::Ask::Drop { owner, .. } => {
            if let Some(task) = domain.port.effect_tasks.remove(&owner) {
                crate::procedures::effect_decided(domain, task, false, raw);
            }
            drop(domain.port.inputs.remove(&owner));
            Event::Drop { token: owner }
        }
        effects::Ask::DropProposal { proposal } => Event::DropProposal { number: proposal },
        effects::Ask::Resolve { entry, again } => Event::Resolve { entry, repeat: again },
        effects::Ask::Close { task, .. } => Event::Close { task },
        effects::Ask::Release { task, .. } => {
            release = Some(task);
            Event::Unnamed { task }
        }
        effects::Ask::Judge { owner, judge, state, resources } => {
            judge_effect(domain, env, owner, judge, state, resources, raw, out);
            return;
        }
    };
    let kept_entry = if let Event::Keep { entry, .. } = &event { Some(*entry) } else { None };
    crate::step_system(domain, env, event, raw);
    if let Some(task) = release {
        reported(domain, raw, false, out);
        say_effect(out, effects::Up::Released { task });
    }
    if let Some(entry) = kept_entry {
        reported(domain, raw, false, out);
        if !domain.outbox.entries.contains_key(&entry) {
            domain.port.outbox.remove(&entry);
        }
    }
}

#[expect(clippy::too_many_arguments, reason = "pass all fields of the judgement to the connector")]
fn judge_effect(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    judge: effects::Judge,
    state: [u8; 32],
    resources: Box<[effects::Name]>,
    raw: &mut Queue<Request>,
    out: &mut Queue<Out>,
) {
    let mut paths = List::with_capacity(u32::try_from(resources.len()).expect("bounded resources"));
    for resource in resources {
        paths.push(path(resource.segments)).expect("one path per resource");
    }
    let bytes = state.get(24..).expect("state suffix fits").try_into().expect("eight state bytes");
    crate::requirements::judge(domain, env, owner, judge.requirement, paths.as_slice(), u64::from_be_bytes(bytes), raw);
    domain.judges.remove(&owner);
    let Some(Request::Verdict { token, verdict }) = raw.pop() else { unreachable!("a judge first gives its verdict") };
    assert_eq!(token, owner, "the verdict names the asked owner");
    let (verdict, at, guarded, state) = match verdict {
        crate::Verdict::Met { state, observed, guarded } => {
            (effects::Verdict::Met, observed, guarded, state_pin(state))
        }
        crate::Verdict::Wait => (effects::Verdict::Wait, env.wall, false, state),
        crate::Verdict::Refuse { .. } => (effects::Verdict::Refuse, env.wall, false, state),
    };
    say_effect(out, effects::Up::Verdict { owner, judge, verdict, at, guarded, state });
}

fn topic_ask(domain: &mut Domain, env: &Env<Limits>, ask: topics::Ask, raw: &mut Queue<Request>) {
    let (subscription, task, topic) = match ask {
        topics::Ask::Subscribed { key, subscription, topic, .. } => {
            let topic = u16::try_from(topic).expect("test topic number");
            crate::step_system(domain, env, Event::Subscribe { task: key.task, topic, wake_at: 2, keep_at: 1 }, raw);
            domain.restore_interest(key.task, topic, subscription);
            return;
        }
        topics::Ask::Unsubscribed { key, subscription, topic, .. } => {
            (subscription, key.task, u16::try_from(topic).expect("test topic number"))
        }
        topics::Ask::Ended { task, subscription } => {
            let Some((_, topic)) = domain.port.interests.get(&subscription) else { return };
            (subscription, task, *topic)
        }
        topics::Ask::Refused { .. } => return,
    };
    domain.port.interests.remove(&subscription);
    for (_, interest) in &domain.port.interests {
        if *interest == (task, topic) {
            return;
        }
    }
    crate::step_system(domain, env, Event::Unsubscribe { task, topic }, raw);
}

fn part_ask(domain: &mut Domain, env: &Env<Limits>, ask: parts::Ask, raw: &mut Queue<Request>, out: &mut Queue<Out>) {
    let event = match ask {
        parts::Ask::Lines(ask) => {
            if let Some(script) = &mut domain.port.parts {
                script.push(ask);
            } else {
                out.push(Out::Up(jig::Up::Parts(parts::Up::Lines {
                    token: ask.token,
                    parts: crate::parts::make(&ask),
                })));
            }
            return;
        }
        parts::Ask::Public { reply, .. } => {
            // The fake has no document for the public application families.
            out.push(Out::Up(jig::Up::Parts(parts::Up::Public { reply, answer: None })));
            return;
        }
        parts::Ask::Read { reply, scope, read, party, window } => {
            crate::step_system(
                domain,
                env,
                Event::ReadDocument { scope: scope_of(scope), read, party, window: window_of(window) },
                raw,
            );
            let Some(Request::ReadAnswered { read: answered, answer }) = raw.pop() else {
                unreachable!("read returns its answer")
            };
            assert_eq!(read, answered, "one answer for this read");
            out.push(Out::ReadAnswered { reply, answer });
            return;
        }
        parts::Ask::Document { watcher, scope, subject, party, window } => {
            Event::Document { watcher, scope: scope_of(scope), subject, party, window: window_of(window) }
        }
        parts::Ask::Plan(_)
        | parts::Ask::Abandon { .. }
        | parts::Ask::Gather { .. }
        | parts::Ask::Cut { .. }
        | parts::Ask::Drop { .. } => return,
        parts::Ask::DropDocument { subject } => Event::DocumentForget { subject },
    };
    crate::step_system(domain, env, event, raw);
}

fn setting_ask(ask: settings::Ask) -> Event {
    match ask {
        settings::Ask::Judge { owner, revision, proposal, items } => {
            Event::JudgeSettings { token: owner, revision, proposal, items: setting_items(items) }
        }
        settings::Ask::Restore { owner, proposal, items } => {
            let mut restored = List::with_capacity(u32::try_from(items.len()).expect("bounded restore items"));
            for item in items {
                let edit = match item.edit {
                    settings::RestoredEdit::Set { revision } => crate::RestoredEdit::Set { revision },
                    settings::RestoredEdit::Remove => crate::RestoredEdit::Remove,
                    settings::RestoredEdit::Reset => crate::RestoredEdit::Reset,
                };
                restored
                    .push(crate::RestoredItem { kind: item.kind, scope: item.scope, number: item.number, edit })
                    .expect("one restored item per ask item");
            }
            Event::RestoreSettings { token: owner, proposal, items: restored.into_boxed() }
        }
        settings::Ask::Keep { revision, proposal, items } => {
            Event::KeepSettings { revision, proposal, items: setting_items(items) }
        }
        settings::Ask::KeepProposal { proposal, items } => {
            Event::KeepProposalSettings { proposal, items: setting_items(items) }
        }
        settings::Ask::DropProposal { proposal } => Event::DropProposalSettings { proposal },
        settings::Ask::ForgetReplaced { before } => Event::ForgetReplaced { before },
        settings::Ask::SaveDefaults { revision } => Event::SaveSettingsDefaults { revision },
    }
}

fn setting_items(items: Box<[settings::Item]>) -> Box<[crate::SettingItem]> {
    let mut translated = List::with_capacity(u32::try_from(items.len()).expect("bounded settings items"));
    for item in items {
        let edit = match item.edit {
            settings::Edit::Set(bytes) => crate::SettingEdit::Set(bytes),
            settings::Edit::Remove => crate::SettingEdit::Remove,
            settings::Edit::Reset => crate::SettingEdit::Reset,
        };
        translated
            .push(crate::SettingItem { kind: item.kind, scope: item.scope, number: item.number, edit })
            .expect("one edit per item");
    }
    translated.into_boxed()
}

fn setting_answers(judged: Box<[crate::SettingJudged]>) -> Box<[settings::Judged]> {
    let mut answers = List::with_capacity(u32::try_from(judged.len()).expect("bounded judgements"));
    for answer in judged {
        answers
            .push(match answer {
                crate::SettingJudged::Accept { size, charters } => settings::Judged::Accept { size, charters },
                crate::SettingJudged::Same => settings::Judged::Same,
                crate::SettingJudged::Refuse { why } => settings::Judged::Refuse { why },
            })
            .expect("one answer per judgement");
    }
    answers.into_boxed()
}

fn setting_defaults(defaults: Box<[crate::SettingDefault]>, dropped: Box<[(u32, u32, u32)]>) -> settings::Up {
    let mut states = List::with_capacity(u32::try_from(defaults.len()).expect("bounded defaults"));
    for default in defaults {
        states
            .push(settings::DefaultState {
                kind: default.kind,
                scope: default.scope,
                number: default.number,
                size: default.size,
                charters: default.charters,
                differs: default.differs,
            })
            .expect("one state per default");
    }
    let mut names = List::with_capacity(u32::try_from(dropped.len()).expect("bounded dropped defaults"));
    for (kind, scope, number) in dropped {
        names.push(settings::DroppedDefault { kind, scope, number }).expect("one name per dropped default");
    }
    settings::Up::Defaults { defaults: states.into_boxed(), dropped: names.into_boxed() }
}

#[expect(clippy::too_many_lines, reason = "the fake system's total translation into contract and local outputs")]
fn reported(domain: &mut Domain, raw: &mut Queue<Request>, restoring: bool, out: &mut Queue<Out>) {
    for _ in 0..raw.len() {
        let up = match raw.pop().expect("fake output count") {
            Request::Save { record } => {
                out.push(Out::Save(record));
                continue;
            }
            Request::Erase { key } => {
                out.push(Out::Erase(key));
                continue;
            }
            Request::System(call) => {
                out.push(Out::System(call));
                continue;
            }
            Request::SettingsJudged { token, judged } => {
                jig::Up::Settings(settings::Up::Judged { owner: token, judged: setting_answers(judged) })
            }
            Request::SettingsRestored { token, judged } => {
                jig::Up::Settings(settings::Up::Restored { owner: token, judged: setting_answers(judged) })
            }
            Request::SettingsDefaults { defaults, dropped } => jig::Up::Settings(setting_defaults(defaults, dropped)),
            Request::RestoreRefused { kind, scope, number } => {
                out.push(Out::RestoreRefused { kind, scope, number });
                continue;
            }
            Request::ProjectReleased { project } => jig::Up::Lifecycle(lifecycle::Up::Released { project }),
            Request::ReadAfreshDone => {
                jig::Up::Lifecycle(lifecycle::Up::Done { stage: lifecycle::RestartStage::ReadAfresh })
            }
            Request::RestartDone => jig::Up::Lifecycle(lifecycle::Up::Done { stage: lifecycle::RestartStage::Settled }),
            Request::Document { watcher, document, .. } => {
                let outcome = match document {
                    Ok(document) => {
                        domain.port.documents.insert(watcher, document).expect("one prepared document per watcher");
                        parts::Made::Ready
                    }
                    Err(why) => parts::Made::Gone {
                        why: match why {
                            crate::DocumentMissing::Unknown => parts::End::Gone,
                            crate::DocumentMissing::NotAllowed => parts::End::NotAllowed,
                        },
                    },
                };
                jig::Up::Parts(parts::Up::Made { watcher, outcome })
            }
            Request::Topic { token, resources } => {
                out.push(Out::Topic { token, resources: Some(topic_reads(domain.port.number, resources)) });
                continue;
            }
            Request::TopicRefused { token } => {
                out.push(Out::Topic { token, resources: None });
                continue;
            }
            Request::News { topic, subscribers } => {
                news(domain, topic, &subscribers, out);
                continue;
            }
            Request::Marked { subjects } => jig::Up::Requests(requests::Up::Marked { subjects }),
            Request::ApplicationDone { request, token } => {
                jig::Up::Requests(requests::Up::Decided { request, outcome: Ok(token) })
            }
            Request::ApplicationAgain { request, why } => jig::Up::Requests(requests::Up::Decided {
                request,
                outcome: Err(match why {
                    crate::application_core::Again::Busy => ops::party::Again::Busy,
                    crate::application_core::Again::NotReady => ops::party::Again::NotReady,
                }),
            }),
            Request::ApplicationRelease { request, project, release } => {
                out.push(Out::ApplicationRelease { request, project, release });
                continue;
            }
            Request::Closed { task } => jig::Up::Effects(effects::Up::Closed { task }),
            Request::Drift { pool: resource, tasks } | Request::DriftResource { tasks, resource } => {
                for task in tasks {
                    out.push(Out::Up(jig::Up::Resources(resources::Up::Drift {
                        task,
                        resource: named(domain, &resource),
                    })));
                }
                continue;
            }
            Request::Step { task, decision } => {
                let (step, code, context) = domain.port.steps.get(&task).expect("one admitted procedure start");
                if let crate::StepDecision::Effect(effect) = decision {
                    out.push(Out::EffectOwner {
                        effect,
                        origin: effects::EffectOrigin::Procedure { task, step: *step, entry: None },
                    });
                    continue;
                }
                jig::Up::Procedures(procedures::Up::Step {
                    task,
                    step: *step,
                    code: *code,
                    decision: crate::steps::translate(context, *step, domain.port.number, decision),
                })
            }
            Request::Waits { task, wait } => {
                let waits = match wait {
                    Some(wait) => List::from_boxed(Box::new([procedures::Reported {
                        kind: ops::WaitKind::Step,
                        object: ops::WaitObject::Procedure { step: wait.step, things: List::with_capacity(0) },
                        since: wait.since,
                        bound: procedures::ReportedBound::At { at: wait.at, then: ops::AtBound::Hold },
                    }])),
                    None => List::with_capacity(0),
                };
                let up = procedures::Up::Waits { task, waits };
                if let Some(script) = &mut domain.port.waits {
                    script.push(up);
                    continue;
                }
                jig::Up::Procedures(up)
            }
            Request::Described { token, description, .. } => jig::Up::Effects(effects::Up::Described {
                owner: token,
                description: Box::new(describe(domain.port.number, description)),
            }),
            Request::EffectBusy { token } => jig::Up::Effects(effects::Up::Busy { owner: token }),
            Request::EffectRefused { token } => jig::Up::Effects(effects::Up::Refused { owner: token }),
            Request::Outcome { entry, outcome } => {
                outbox(domain, entry, outcome, out);
                continue;
            }
            Request::Slots { pool, slots } => {
                jig::Up::Resources(resources::Up::PoolSlots { name: named(domain, &pool), slots })
            }
            Request::Named { project, resources, .. } => {
                for resource in resources {
                    out.push(Out::Up(jig::Up::Resources(resource_report(
                        domain,
                        project,
                        &resource.path,
                        resource.role,
                        resource.hold,
                    ))));
                }
                continue;
            }
            Request::Resource { project, resource } => {
                if restoring {
                    out.push(Out::RestoreResource {
                        project,
                        name: named(domain, &resource.path),
                        role: resource_role(resource.role),
                        hold: report_hold(domain, &resource.path, resource.hold),
                    });
                    continue;
                }
                jig::Up::Resources(resource_report(domain, project, &resource.path, resource.role, resource.hold))
            }
            Request::Adopted { .. }
            | Request::Unknown { .. }
            | Request::Refused { .. }
            | Request::Changed { .. }
            | Request::Verdict { .. }
            | Request::Make { .. }
            | Request::SettingValue { .. }
            | Request::ReadAnswered { .. }
            | Request::Answer { .. }
            | Request::Ready { .. }
            | Request::Section { .. }
            | Request::Workspace { .. } => continue,
        };
        out.push(Out::Up(up));
    }
}

fn path(segments: Box<[Box<[u8]>]>) -> Path {
    Path::new(List::from_boxed(segments), u32::MAX).expect("admitted nonempty path")
}

fn named(domain: &Domain, resource: &Path) -> resources::Name {
    resources::Name { connector: domain.port.number, path: resource.segments().into() }
}

fn resource_role(role: crate::ResourceRole) -> resources::ResourceRole {
    match role {
        crate::ResourceRole::Owned => resources::ResourceRole::Owned,
        crate::ResourceRole::Participating => resources::ResourceRole::Participant,
        crate::ResourceRole::Context => resources::ResourceRole::Context,
        crate::ResourceRole::Unavailable => resources::ResourceRole::Unavailable,
    }
}

fn resource_report(
    domain: &Domain,
    project: u32,
    resource: &Path,
    role: crate::ResourceRole,
    hold: crate::Hold,
) -> resources::Up {
    resources::Up::Resource {
        project,
        name: named(domain, resource),
        role: resource_role(role),
        hold: report_hold(domain, resource, hold),
    }
}

fn report_hold(domain: &Domain, resource: &Path, hold: crate::Hold) -> resources::HoldKind {
    match hold {
        crate::Hold::None => resources::HoldKind::Shared,
        crate::Hold::Exclusive { mode } => resources::HoldKind::Exclusive { wait: mode == crate::HoldMode::Wait },
        crate::Hold::Pooled { mode } => resources::HoldKind::Pooled {
            slots: domain.slots(resource).unwrap_or(0),
            wait: mode == crate::HoldMode::Wait,
        },
    }
}

fn topic_reads(number: u16, paths: Box<[Path]>) -> Box<[effects::Effect]> {
    let mut reads = List::with_capacity(u32::try_from(paths.len()).expect("bounded topic resources"));
    for path in paths {
        reads
            .push(effects::Effect {
                connector: number,
                kind: 0,
                name: effects::Name { segments: path.segments().into() },
                state: [0; 32],
                price: None,
                access: effects::EffectAccess::Context,
                additional: Box::new([]),
                guards: Box::new([]),
            })
            .expect("one read per offered resource");
    }
    reads.into_boxed()
}

fn news(domain: &Domain, topic: u16, subscribers: &[crate::Classed], out: &mut Queue<Out>) {
    for subscriber in subscribers {
        for (number, interest) in &domain.port.interests {
            if *interest == (subscriber.task, topic) {
                let class = match subscriber.class {
                    crate::Class::Wake => topics::NewsClass::Wakes,
                    crate::Class::Keep => topics::NewsClass::Kept,
                };
                let digits = Decimal::of(u64::from(topic));
                let size = b"News on topic ".len().checked_add(digits.as_bytes().len()).expect("bounded topic words");
                let mut words = Writer::new(size);
                words.put(b"News on topic ").expect("topic label fits");
                words.put(digits.as_bytes()).expect("topic number fits");
                out.push(Out::Up(jig::Up::Topics(topics::Up::News {
                    task: subscriber.task,
                    subscription: *number,
                    topic: u64::from(topic),
                    class,
                    words: words.finish(),
                })));
            }
        }
    }
}

fn say_effect(out: &mut Queue<Out>, up: effects::Up) {
    out.push(Out::Up(jig::Up::Effects(up)));
}

fn kept(domain: &mut Domain, entry: u64, task: u64, purpose: crate::Purpose) {
    domain.port.outbox.insert(entry, (task, purpose)).expect("bounded outbox routes");
}

fn outbox(domain: &mut Domain, entry: u64, outcome: crate::Outcome, out: &mut Queue<Out>) {
    let Some(&(task, purpose)) = domain.port.outbox.get(&entry) else { return };
    let outcome = match outcome {
        crate::Outcome::Made { .. } => effects::OutboxOutcome::Made,
        crate::Outcome::Failed => effects::OutboxOutcome::Failed,
        crate::Outcome::Uncertain => effects::OutboxOutcome::Uncertain,
        crate::Outcome::Held => effects::OutboxOutcome::Held { entry },
        crate::Outcome::Withdrawn => effects::OutboxOutcome::Withdrawn,
        crate::Outcome::Skipped => effects::OutboxOutcome::NotMade,
    };
    match outcome {
        effects::OutboxOutcome::Made
        | effects::OutboxOutcome::Failed
        | effects::OutboxOutcome::NotMade
        | effects::OutboxOutcome::Withdrawn => {
            domain.port.outbox.remove(&entry);
            if purpose == crate::Purpose::Procedure {
                let mut raw = Queue::uncapped();
                crate::procedures::effect_decided(domain, task, outcome == effects::OutboxOutcome::Made, &mut raw);
                if let Some(Request::Save { record }) = raw.pop() {
                    out.push(Out::Save(record));
                }
                assert!(raw.is_empty(), "the decision updates only the procedure record");
            }
        }
        effects::OutboxOutcome::Uncertain | effects::OutboxOutcome::Held { .. } => {}
    }
    if purpose == crate::Purpose::Projection {
        out.push(Out::Up(jig::Up::Projections(projections::Up::Outbox { entry, outcome })));
    } else {
        say_effect(out, effects::Up::Outbox { entry, task, outcome });
    }
}

fn describe(number: u16, description: crate::Description) -> effects::EffectDescription {
    let mut resources = List::with_capacity(u32::try_from(description.resources.len()).expect("bounded resources"));
    for resource in description.resources {
        resources.push(effects::Name { segments: resource.segments().into() }).expect("one name per resource");
    }
    let mut additional = List::with_capacity(resources.len().saturating_sub(1));
    let first = resources.as_slice().first().expect("described effect names a resource").clone();
    for resource in resources.as_slice().get(1..).expect("at least one resource") {
        additional
            .push(effects::EffectResource { name: resource.clone(), access: effects::EffectAccess::Owned })
            .expect("additional resources");
    }
    effects::EffectDescription {
        connector: number,
        purpose: description.purpose,
        effect: effects::Effect {
            connector: number,
            kind: description.kind,
            name: first,
            state: state_pin(description.state),
            price: description.price,
            access: effects::EffectAccess::Owned,
            additional: additional.into_boxed(),
            guards: Box::new([]),
        },
    }
}

fn state_pin(state: u64) -> [u8; 32] {
    let mut pin = [0; 32];
    for (byte, state) in pin.iter_mut().skip(24).zip(state.to_be_bytes()) {
        *byte = state;
    }
    pin
}

const fn wait_refusal(why: procedures::ReportRefusal) -> &'static str {
    match why {
        procedures::ReportRefusal::NotLive => "not a live task",
        procedures::ReportRefusal::NotProcedure => "not a procedure",
        procedures::ReportRefusal::WrongConnector => "another connector owns the task",
        procedures::ReportRefusal::TooManyWaits => "more than four waits",
        procedures::ReportRefusal::TooManyThings => "more than four things",
        procedures::ReportRefusal::WrongKindOrObject => "not a procedure's kind and object",
        procedures::ReportRefusal::NoFigure => "the kind has no figure",
    }
}
fn window_of(value: parts::Window) -> crate::ClientWindow {
    crate::ClientWindow {
        rows: value.rows,
        after: match value.after {
            Some(parts::Cursor::Number { number }) => Some(crate::ClientCursor::Number(number)),
            Some(parts::Cursor::Inbox { .. }) => Some(crate::ClientCursor::Unknown),
            None => None,
        },
    }
}

fn scope_of(value: requests::ApplicationScope) -> crate::ClientScope {
    match value {
        requests::ApplicationScope::Project(project) => crate::ClientScope::Project(project),
        requests::ApplicationScope::Deployment => crate::ClientScope::Deployment,
        requests::ApplicationScope::Own => crate::ClientScope::Own,
    }
}

fn request_role(role: Option<ops::Role>) -> Option<u32> {
    Some(role?.number())
}

fn application_answer(outcome: ops::party::Outcome) -> crate::ApplicationCoreAnswer {
    match outcome {
        ops::party::Outcome::Released { .. } => crate::ApplicationCoreAnswer::Released,
        ops::party::Outcome::Refused(why) => {
            crate::ApplicationCoreAnswer::Refused(crate::application_facts::refusal(why))
        }
        ops::party::Outcome::Again(why) => crate::ApplicationCoreAnswer::Again(crate::application_facts::again(why)),
        ops::party::Outcome::WorkerPauseAsked { .. }
        | ops::party::Outcome::Restored { .. }
        | ops::party::Outcome::Compound { .. }
        | ops::party::Outcome::CompoundRefused { .. }
        | ops::party::Outcome::Applied { .. }
        | ops::party::Outcome::SignedOut
        | ops::party::Outcome::NoteEdited { .. }
        | ops::party::Outcome::ServiceMade { .. }
        | ops::party::Outcome::ServiceReissued { .. }
        | ops::party::Outcome::Adopted { .. }
        | ops::party::Outcome::Created { .. }
        | ops::party::Outcome::Proposed { .. }
        | ops::party::Outcome::Withdrawn { .. }
        | ops::party::Outcome::Left { .. }
        | ops::party::Outcome::Stopped { .. }
        | ops::party::Outcome::Cancelled { .. }
        | ops::party::Outcome::PersonTaken { .. }
        | ops::party::Outcome::PersonHandedBack { .. }
        | ops::party::Outcome::PersonAnswered { .. }
        | ops::party::Outcome::Moved { .. }
        | ops::party::Outcome::ProposalDecided { .. }
        | ops::party::Outcome::Said { .. }
        | ops::party::Outcome::QuestionAnswered { .. }
        | ops::party::Outcome::Amended { .. }
        | ops::party::Outcome::RolesSet { .. }
        | ops::party::Outcome::Renamed { .. }
        | ops::party::Outcome::ProjectMade { .. }
        | ops::party::Outcome::ProjectRemoved { .. }
        | ops::party::Outcome::VisibilitySet { .. }
        | ops::party::Outcome::SettingsChanged { .. }
        | ops::party::Outcome::MovedToNewest { .. }
        | ops::party::Outcome::PolicyChanged { .. }
        | ops::party::Outcome::PoolSet { .. }
        | ops::party::Outcome::EscalationDecided { .. }
        | ops::party::Outcome::AccountSignInStarted { .. }
        | ops::party::Outcome::AccountProgramDeclared { .. }
        | ops::party::Outcome::AccountKeyKept { .. }
        | ops::party::Outcome::AccountMovingOut { .. }
        | ops::party::Outcome::AccountFinishing { .. }
        | ops::party::Outcome::AccountMovedIn { .. }
        | ops::party::Outcome::AccountReading { .. }
        | ops::party::Outcome::AccountLoginAsked { .. }
        | ops::party::Outcome::AccountChanged
        | ops::party::Outcome::AccountRemoved { .. }
        | ops::party::Outcome::AccountSignedOut { .. }
        | ops::party::Outcome::Lifted { .. }
        | ops::party::Outcome::Concluded { .. }
        | ops::party::Outcome::MarkedSeen { .. } => unreachable!("release core part returns a release terminal"),
    }
}
