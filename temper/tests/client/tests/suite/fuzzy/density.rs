//! Density over client deliveries and acts (temper/docs/design/client/pages/README.md,
//! A page's parts; temper/docs/design/draft/surfaces.md, 12 and 14).
use jig_client::{Down, Readable, Readout, Up, Watchable, contract as c};
use skein_lib::{Duration, Env, List, Queue, RequestEnvelope, Rng, Time, Token, Wall, Writer};
use std::{
    collections::BTreeSet,
    panic::{AssertUnwindSafe, catch_unwind},
};
use temper_client_domain::{
    self as client, Action, Address, Domain, Event, Limits, Place, Request,
    address::{Filter, Kind},
    forge,
    pages::{self, all_home, home, needs, parts},
};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_world::fixtures;

const SEEDS: u64 = 64;
// Set to Some(the reported seed) to replay only that world's deliveries and acts.
const REPLAY: Option<u64> = None;
const STEPS: usize = 80;
const DAY: u64 = 86_400_000_000_000;
const SECOND: u64 = 1_000_000_000;
const READS: u32 = 128;
const QUESTION: c::ObjectKey = c::ObjectKey::Question { task: 10, message: 1 };

struct World {
    root: Domain,
    env: Env<Limits>,
    rng: Rng,
    projects: u32,
    watches: Vec<(Token, Watchable)>,
    pending: Vec<Up>,
    entries: Vec<c::InboxEntry>,
    approvals: Vec<w::ApprovalRow>,
    accounts: Option<c::Accounts>,
    decision_rows: Vec<(Token, u32, Vec<u64>)>,
    relation_rows: Vec<(Token, u32)>,
    answered: Vec<c::ObjectKey>,
    ways: Vec<u32>,
    revision: u32,
    step: usize,
    observed: BTreeSet<&'static str>,
}

impl World {
    fn event(&mut self, event: Event) {
        let mut out = Queue::uncapped();
        client::step(&mut self.root, &self.env, event, &mut out);
        self.outputs(&out);
        client::take(&mut self.root, &self.env, &mut Queue::uncapped());
        self.check();
    }

    fn up(&mut self, up: Up) {
        self.event(Event::Client { event: jig_client::Event::Up { up } });
    }

    fn outputs(&mut self, out: &Queue<Request>) {
        for request in out {
            let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else {
                continue;
            };
            match down {
                Down::Open { watch, subject } => {
                    self.watches.push((*watch, subject.clone()));
                    self.pending.push(Up::Watched { watch: *watch, event: c::WatchEvent::Opened });
                    if let Watchable::Jig(c::Subject::Tasks { .. }) = subject {
                        self.pending.push(Up::Watched {
                            watch: *watch,
                            event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                                rows: List::with_capacity(0),
                                more: false,
                            }))),
                        });
                    }
                }
                Down::Close { watch } => {
                    self.watches.retain(|(held, _)| held != watch);
                    self.decision_rows.retain(|(held, _, _)| held != watch);
                    self.relation_rows.retain(|(held, _)| held != watch);
                }
                Down::Read { read, what } => {
                    if let Some(result) = self.read(what) {
                        self.pending.push(Up::ReadDone { read: *read, result });
                    }
                }
                Down::Send { attempt, request, .. } => {
                    if let jig_client::Ask::Family { record, .. } = request
                        && let w::Request::Looked(_) = wire::decode_requesting(&record.bytes, &w::CEILINGS)
                            .expect("bounded density fixture")
                            .into_parts()
                            .request
                    {
                        self.pending.push(Up::Answer {
                            attempt: *attempt,
                            envelope: RequestEnvelope::Final,
                            why: None,
                            answer: Some(jig_client::Answered::Family(jig_client::Family {
                                family: wire::FORGE,
                                version: wire::CLIENT_VERSION,
                                bytes: wire::answer(
                                    &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Looked })
                                        .expect("bounded density fixture"),
                                ),
                            })),
                        });
                    }
                    if let jig_client::Ask::Jig(ask) = request
                        && let c::Ask::Answer { on: c::AnswerOn::Question { task, message, .. }, .. } = ask.as_ref()
                    {
                        let object = c::ObjectKey::Question { task: *task, message: *message };
                        self.observed.insert("sent answer");
                        assert!(self.answered.contains(&object), "only an answered card sends");
                        assert!(!self.root.pages().needs().held(object), "the held deadline fired");
                        self.pending.push(Up::Answer {
                            attempt: *attempt,
                            envelope: RequestEnvelope::Final,
                            why: None,
                            answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                                c::DocumentOutcome::QuestionAnswered { task: *task, question: *message, message: 2 },
                            )))),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    fn watch(&self, subject: &Watchable) -> Option<Token> {
        self.watches.iter().rev().find_map(|(watch, held)| (held == subject).then_some(*watch))
    }

    fn deliver(&mut self, subject: &Watchable, delivery: c::Delivery) {
        if let Some(watch) = self.watch(subject) {
            self.up(Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) });
        }
    }

    fn family(&mut self, subject: Option<Watchable>, bytes: Box<[u8]>) {
        if let Some(watch) = subject.and_then(|subject| self.watch(&subject)) {
            self.up(Up::FamilyDelivered {
                watch,
                record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
            });
        }
    }

    fn drain(&mut self) {
        while !self.pending.is_empty() {
            let at =
                usize::try_from(self.rng.below(u64::try_from(self.pending.len()).expect("bounded density fixture")))
                    .expect("bounded density fixture");
            let up = self.pending.swap_remove(at);
            self.up(up);
        }
    }

    fn act(&mut self, act: needs::Act) {
        self.event(Event::Act { action: Action::Pages(pages::Act::Needs(act)) });
    }

    fn go(&mut self, address: Address) {
        self.event(Event::Went { place: Place::Address(address) });
        self.drain();
    }

    fn inbox(&mut self) {
        let entries = List::from_boxed(self.entries.clone().into_boxed_slice());
        self.deliver(
            &Watchable::Jig(c::Subject::Inbox {
                window: self.root.pages().inbox().expect("Home or Needs you owns an inbox").window,
            }),
            c::Delivery::Inbox(Box::new(c::Inbox {
                party: 1,
                cached_needs: entries.len(),
                needs: entries,
                updates: List::with_capacity(0),
                next: None,
                cached_updates: c::Count { value: 0, limited: false },
                results_seen: 0,
                updates_time_complete: true,
                selection: None,
            })),
        );
        self.drain();
    }

    fn accounts(&mut self) {
        let Some(me) = self.root.pages().me(self.root.client()) else { return };
        let party = me.party.number;
        let mut rows = Vec::new();
        for number in 1..=u32::try_from(self.rng.below(4)).expect("zero to three accounts") {
            let kind = match self.rng.below(3) {
                0 => c::AccountKind::SignIn,
                1 => c::AccountKind::Program,
                2 => c::AccountKind::Key,
                _ => unreachable!("three source kinds"),
            };
            let state = if self.rng.below(2) == 0 { c::AccountState::Granted } else { c::AccountState::SignInAgain };
            let expiring = self.rng.below(2) == 0;
            let holder =
                if self.rng.below(2) == 0 { c::AccountHolder::Party(party) } else { c::AccountHolder::Deployment };
            let mut sources = vec![c::AccountSource {
                name: (kind == c::AccountKind::Program).then(|| b"program-login".as_slice().into()),
                place: number,
                number: number * 2,
                kind,
                state,
                usable: state == c::AccountState::Granted,
                expiring,
                attention: false,
                spent_until: None,
            }];
            if self.rng.below(2) == 0 {
                sources.push(c::AccountSource {
                    name: None,
                    place: number,
                    number: number * 2 + 1,
                    kind: c::AccountKind::SignIn,
                    state: c::AccountState::SignInAgain,
                    usable: false,
                    expiring: true,
                    attention: false,
                    spent_until: None,
                });
            }
            rows.push(c::AccountLine {
                list: None,
                number,
                place: number,
                uses: None,
                plan: kind == c::AccountKind::SignIn,
                provider: (self.rng.below(2) == 0).then_some(17),
                provider_name: None,
                manage_usage: None,
                display: text(b"An account"),
                holder,
                sources: List::from_boxed(sources.into_boxed_slice()),
                facts: List::with_capacity(0),
                cut: false,
            });
        }
        let document = c::Accounts {
            program_hosts: List::with_capacity(0),
            accounts: List::from_boxed(rows.into_boxed_slice()),
            signing: List::with_capacity(0),
            last_end: None,
            usable: false,
            settled: true,
            cut: self.rng.below(2) == 0,
            readiness: List::with_capacity(0),
            readiness_cut: false,
        };
        let subject = Watchable::Jig(c::Subject::Accounts);
        if self.watch(&subject).is_some() {
            self.accounts = Some(document.clone());
            self.deliver(&subject, c::Delivery::Accounts(Box::new(document)));
            self.drain();
        }
    }

    fn only(&self) -> u32 {
        let Some(document) = &self.accounts else { return 0 };
        let Some(me) = self.root.pages().me(self.root.client()) else { return 0 };
        let count = document
            .accounts
            .iter()
            .filter(|account| {
                account.holder == c::AccountHolder::Party(me.party.number)
                    && account.sources.get(0).is_some_and(|source| {
                        matches!(source.kind, c::AccountKind::SignIn | c::AccountKind::Program)
                            && (source.state == c::AccountState::SignInAgain || source.expiring)
                    })
            })
            .count();
        u32::try_from(count).expect("zero to three Only you items")
    }

    fn read(&self, what: &Readable) -> Option<Readout> {
        let Readable::Jig(read) = what else { return self.setting_read(what) };
        let value = match read {
            c::Read::Inbox { group, project, kind, .. } => {
                let rows = self
                    .entries
                    .iter()
                    .filter(|row| {
                        *group == c::InboxReadGroup::Needs
                            && project.is_none_or(|project| row.shows_in(project))
                            && kind.is_none_or(|kind| row.group() == kind)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                c::ReadValue::Inbox(Box::new(c::InboxPage {
                    rows: List::from_boxed(rows.into_boxed_slice()),
                    cut: false,
                    before: None,
                    after: None,
                    selection: None,
                }))
            }
            c::Read::Question { task, message } => c::ReadValue::Question(Box::new(c::Question {
                task: chip(*task, self.project_of(*task)),
                from: None,
                message: *message,
                words: text(b"Which approach?\n\n- Small\n- Large"),
                since: Wall::EPOCH,
                acts: offers(c::ActKind::Answer),
                ending: None,
            })),
            c::Read::Proposal { proposal } => c::ReadValue::Proposal(Box::new(c::Proposal {
                projects: List::with_capacity(0),
                key: *proposal,
                project: self.project_of(11),
                reason: text(b"Use the smaller change."),
                action: c::ProposalAction::Batch(List::with_capacity(0)),
                as_holder: true,
                waiting: c::ProposalWaiting { holder: c::HistoryHolder::Person(1), since: Wall::EPOCH },
                funding: List::with_capacity(0),
                acts: offers(c::ActKind::Accept),
            })),
            c::Read::Escalation { task, .. } => {
                let mut task = task_document(*task, self.project_of(*task));
                let hold = c::ChipHold {
                    placed: None,
                    words: Some(text(b"Review the held work.")),
                    circle: None,
                    account: None,
                    source: None,
                    reason: if task.chip.task == 13 {
                        c::HoldCategory::Budget
                    } else {
                        c::HoldCategory::Failures { class: c::HoldFailure::Agent }
                    },
                    revision: 1,
                    before: c::PriorPhase::Waiting,
                    escalation: None,
                    tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
                    hold_wait_since: None,
                };
                task.chip.phase = c::Phase::Held;
                task.chip.details.phase = c::ChipPhase::Held { reason: hold.reason, revision: 1 };
                task.chip.details.details.hold = Some(hold.clone());
                task.hold = Some(Box::new(hold));
                c::ReadValue::Escalation(Box::new(c::Escalation {
                    task: Box::new(task),
                    acts: offers(c::ActKind::Release),
                }))
            }
            c::Read::Task { task } => c::ReadValue::Task(Box::new(task_document(*task, 1))),
            c::Read::Tasks { .. } => c::ReadValue::Tasks(Box::new(c::TaskPage {
                live: List::with_capacity(0),
                live_after: None,
                ended: c::EndedGroup::None,
            })),
            _ => return None,
        };
        Some(Readout::Jig(c::ReadResult::Value(value)))
    }

    fn setting_read(&self, what: &Readable) -> Option<Readout> {
        if let Readable::Family { scope: c::Scope::Project(project), record, .. } = what
            && let w::Read::Setting(setting) =
                wire::decode_reading(&record.bytes, &w::CEILINGS).expect("bounded density fixture").into_parts().read
            && setting.kind() == 7
        {
            let value = w::ProjectWays::new(
                &w::CEILINGS,
                w::ProjectWaysParts {
                    drafts: true,
                    goals: false,
                    changes: true,
                    way: self.ways[usize::try_from(project - 1).expect("bounded density fixture")],
                },
            )
            .expect("bounded density fixture");
            let mut writer = Writer::new(usize::try_from(value.measure()).expect("bounded density fixture"));
            value.encode(&mut writer).expect("bounded density fixture");
            return Some(Readout::Family {
                family: wire::FORGE,
                record: Some(wire::setting_answer(
                    &w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(writer.finish()) })
                        .expect("bounded density fixture"),
                )),
            });
        }
        None
    }

    fn project_of(&self, task: u64) -> u32 {
        1 + u32::try_from(task % u64::from(self.projects)).expect("bounded density fixture")
    }

    fn status(&mut self, project: u32) {
        let landed = self.rng.below(4);
        let seen = self.rng.below(4);
        let previous = (self.rng.below(3) != 0).then_some(2 * SECOND);
        let brief = (self.rng.below(3) != 0).then(|| {
            w::StatusBrief::new(
                &w::CEILINGS,
                w::StatusBriefParts {
                    sentence: b"A change landed.".as_slice().into(),
                    lede: b"The change is ready. More follows.".as_slice().into(),
                    at: self.rng.below(5) * SECOND,
                    landed,
                },
            )
            .expect("bounded density fixture")
        });
        let document = w::ProjectStatus::new(
            &w::CEILINGS,
            w::ProjectStatusParts {
                project,
                landed,
                seen,
                previous,
                last_landed: (landed > 0).then_some(SECOND),
                brief,
                queued: 0,
                red: false,
                branch: b"main".as_slice().into(),
            },
        )
        .expect("bounded density fixture");
        self.family(forge::status::watch(project, &self.env.limits.forge), wire::project_status(&document));
    }

    fn decisions(&mut self, project: u32) {
        let mut rows = List::with_capacity(3);
        for number in 1..=self.rng.below(4) {
            rows.push(
                w::Decision::new(
                    &w::CEILINGS,
                    w::DecisionParts {
                        reach: w::Reach::new(
                            &w::CEILINGS,
                            w::ReachParts {
                                kind: w::ReachKind::Tests,
                                class: w::ReachClass::Decide,
                                reason: w::ReachReason::Kind,
                                under: 0,
                                sets: 0,
                                matched: 0,
                                away: false,
                            },
                        )
                        .expect("density reach"),
                        number: u32::try_from(number).expect("bounded density fixture"),
                        task: 100 + number,
                        at: self.env.wall.as_nanos().saturating_sub(match self.rng.below(3) {
                            0 => DAY,
                            1 => DAY + SECOND,
                            2 => SECOND,
                            _ => unreachable!("three decision ages"),
                        }),
                        question: b"Which approach?".as_slice().into(),
                        took: b"Small".as_slice().into(),
                        why: b"Within the landing rule.".as_slice().into(),
                        overturned: false,
                    },
                )
                .expect("bounded density fixture"),
            )
            .expect("bounded density fixture");
        }
        if let Some(watch) =
            self.watch(&forge::decided::watch(project, &self.env.limits.forge).expect("decision subject fits"))
        {
            self.decision_rows.retain(|(held, _, _)| *held != watch);
            self.decision_rows.push((watch, project, rows.iter().map(w::Decision::at).collect()));
        }
        let document = w::Decisions::new(
            &w::CEILINGS,
            w::DecisionsParts { project, rows, asked: List::with_capacity(0), cut: false },
        )
        .expect("bounded density fixture");
        self.family(forge::decided::watch(project, &self.env.limits.forge), wire::decisions(&document));
        self.drain();
    }

    fn dismissals(&mut self) {
        let mut dismissed = List::with_capacity(self.projects);
        for project in 1..=self.projects {
            if self.rng.below(2) == 0 {
                dismissed
                    .push(w::PersonFact::ToBegin(
                        w::ToBegin::new(&w::CEILINGS, w::ToBeginParts { project }).expect("bounded density fixture"),
                    ))
                    .expect("bounded density fixture");
            }
        }
        self.family(
            forge::dismissed::watch(&self.env.limits.forge),
            wire::dismissals(
                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).expect("bounded density fixture"),
            ),
        );
    }

    fn approval_rows(&mut self) {
        let document = w::Approvals::new(
            &w::CEILINGS,
            w::ApprovalsParts { rows: List::from_boxed(self.approvals.clone().into_boxed_slice()), more: 0 },
        )
        .expect("bounded density fixture");
        self.family(forge::approvals::watch(&self.env.limits.forge), wire::approvals(&document));
    }

    fn settings(&mut self, project: u32) {
        let before = self.open();
        if before.iter().any(|object| self.approvals.iter().any(|row| *object == c::ObjectKey::Task(row.task()))) {
            self.observed.insert("settings with approval");
        }
        self.revision += 1;
        let way = &mut self.ways[usize::try_from(project - 1).expect("bounded density fixture")];
        *way = 1 - *way;
        let mut settings = List::uncapped();
        for project in 1..=self.projects {
            settings
                .push(c::SettingLine {
                    setting: c::Setting::Application { kind: 2, scope: project, number: 1 },
                    name: Some(text(if self.ways[usize::try_from(project - 1).expect("generated project")] == 0 {
                        b"You approve"
                    } else {
                        b"Agents review"
                    })),
                    follows: c::SettingFollows::Change,
                    last: Some(self.revision),
                    application: None,
                })
                .expect("uncapped setting lines");
            settings
                .push(c::SettingLine {
                    setting: c::Setting::Application { kind: 7, scope: project, number: 1 },
                    name: Some(text(
                        if self.ways[usize::try_from(project - 1).expect("bounded density fixture")] == 0 {
                            b"You approve"
                        } else {
                            b"Agents review"
                        },
                    )),
                    follows: c::SettingFollows::Change,
                    last: Some(self.revision),
                    application: None,
                })
                .expect("bounded density fixture");
        }
        assert!(self.watch(&Watchable::Jig(c::Subject::Settings)).is_some(), "the settings revision is delivered");
        self.deliver(
            &Watchable::Jig(c::Subject::Settings),
            c::Delivery::Settings(Box::new(c::SettingsDocument {
                newest: self.revision,
                settings,
                held: List::with_capacity(0),
                pending: List::with_capacity(0),
            })),
        );
        self.drain();
        if let Some(project) = self.root.pages().project() {
            assert_eq!(
                self.root.forge().ways(project).expect("the revised project ways were read").way,
                self.ways[usize::try_from(project - 1).expect("generated project")],
                "the client consumed the settings revision"
            );
        }
        check_settings(&before, &self.open(), &self.approvals);
    }

    fn open(&self) -> Vec<c::ObjectKey> {
        let inbox = self.root.pages().inbox().expect("Home or Needs you owns an inbox");
        self.root.pages().needs().needs(inbox, self.root.forge().approvals()).iter().map(|need| need.object()).collect()
    }

    fn relations(&mut self, project: u32) {
        let subject = forge::follows::watch(project, &self.env.limits.forge).expect("follows subject");
        let Some(watch) = self.watch(&subject) else { return };
        let count = u32::try_from(self.rng.below(3)).expect("at most two relations");
        let dependencies = (1..=count)
            .map(|number| {
                w::DependencyView::new(&w::CEILINGS, w::DependencyViewParts { number, project: 7 }).expect("dependency")
            })
            .collect::<Vec<_>>();
        let panel = w::FollowsPanel::new(
            &w::CEILINGS,
            w::FollowsPanelParts {
                project,
                follows: List::with_capacity(0),
                dependencies: List::from_boxed(dependencies.into_boxed_slice()),
            },
        )
        .expect("relations");
        self.relation_rows.retain(|(held, _)| *held != watch);
        self.relation_rows.push((watch, count));
        self.family(Some(subject), wire::follows_panel(&panel));
    }

    fn showing(&self, project: Option<u32>) -> usize {
        self.decision_rows
            .iter()
            .filter(|(_, found, _)| project.is_none_or(|project| project == *found))
            .flat_map(|(_, _, rows)| rows)
            .filter(|at| self.env.wall.as_nanos() - **at < DAY)
            .count()
    }

    fn check(&mut self) {
        let Some(inbox) = self.root.pages().inbox() else { return };
        let open = self.open();
        let needs_facts = parts::needs_facts(&self.root, self.env.wall);
        let only = self.only();
        assert_eq!(needs_facts.only, only, "Only you counts the person's deciding logins, step {}", self.step);
        if self.accounts.is_some() {
            self.observed.insert(if only == 0 { "no Only you item" } else { "an Only you item" });
        }
        if self.decision_rows.iter().any(|(_, _, rows)| rows.iter().any(|at| self.env.wall.as_nanos() - at >= DAY)) {
            self.observed.insert("expired decisions");
        }
        assert_eq!(
            usize::try_from(needs_facts.open).expect("bounded density fixture"),
            open.len(),
            "Needs you's open count"
        );
        let (facts, brief_lines, decisions, projects) = match *self.root.pages().address() {
            Address::AllHome => {
                let page = all_home::facts(&self.root, self.env.wall);
                let expected = self
                    .relation_rows
                    .iter()
                    .map(|(_, count)| usize::try_from(*count).expect("two relations"))
                    .sum::<usize>();
                assert_eq!(page.follows.len(), expected, "answered relation boxes");
                assert_eq!(usize::try_from(page.parts_facts().follows).expect("bounded boxes"), expected);
                self.observed.insert(if expected > 0 { "relations showing" } else { "no relations" });
                assert_eq!(
                    usize::try_from(page.showing).expect("small decision count"),
                    self.showing(None),
                    "decisions within their day across projects"
                );
                let fresh_projects = page.projects.iter().filter(|project| fresh(&project.status)).count();
                let brief_lines = all_home::brief(&page).map_or(0, |brief| {
                    assert!(brief.lines.iter().all(|line| !line.sentence.is_empty()), "every joined brief has words");
                    brief.lines.len()
                });
                assert_eq!(brief_lines, fresh_projects, "one line per new brief across at most four projects");
                (page.parts_facts(), brief_lines, page.decided.len(), page.projects.len())
            }
            Address::Home { project } => {
                let page = home::facts(&self.root, project, self.env.wall);
                let decisions = page.decided.as_ref().map_or(0, |away| away.rows.len());
                let showing = page.decided.as_ref().map_or(0, |away| away.showing);
                assert_eq!(
                    usize::try_from(showing).expect("small decision count"),
                    if page.to_begin() { 0 } else { self.showing(Some(project)) },
                    "project decisions within their day"
                );
                let brief_lines = page.brief().map_or(0, |brief| {
                    assert!(!brief.lede().is_empty(), "the brief has words");
                    1
                });
                assert_eq!(brief_lines, usize::from(fresh(&page.status)), "only a new brief shows");
                (page.parts_facts(), brief_lines, decisions, 0)
            }
            Address::Inbox { .. } => {
                assert_eq!(
                    usize::try_from(needs_facts.showing).expect("small decision count"),
                    self.showing(inbox.filter.project),
                    "Needs you decisions within their day"
                );
                check_rules(None, &needs_facts);
                check_content(None, &needs_facts, 0, all_home::decided(&self.root, self.env.wall).len(), 0, open.len());
                check_cards(self, &open, needs_facts.open);
                return;
            }
            _ => unreachable!("density walks Home and Needs you"),
        };
        self.observed.insert(if facts.to_begin { "To begin" } else { "everyday Home" });
        self.observed.insert(if facts.open > 3 { "N more" } else { "at most three cards" });
        self.observed.insert(if facts.showing > 0 { "decisions showing" } else { "no decisions" });
        self.observed.insert(if facts.brief { "new brief" } else { "no new brief" });
        assert_eq!(facts.brief, brief_lines > 0, "fresh brief facts, step {}", self.step);
        assert_eq!(needs_facts.known, inbox.known());
        check_rules(Some(&facts), &needs_facts);
        check_content(Some(&facts), &needs_facts, brief_lines, decisions, projects, open.len());
        check_cards(self, &open, facts.open);
    }
}

fn fresh(status: &home::Status) -> bool {
    match status {
        home::Status::Unknown => false,
        home::Status::Shown(document) => document.brief().as_ref().is_some_and(|brief| {
            brief.landed() > document.seen() || document.previous().is_some_and(|previous| brief.at() > previous)
        }),
    }
}

fn check_rules(home: Option<&parts::HomeFacts>, needs: &parts::NeedsFacts) {
    if let Some(facts) = home {
        assert_eq!(
            parts::home(facts),
            parts::HomeParts {
                brief: facts.brief && !facts.to_begin,
                needs: facts.open != 0 && !facts.to_begin,
                decided: facts.showing != 0 && !facts.to_begin,
                projects: facts.across && facts.projects > 1,
                follows: facts.across && facts.follows > 0,
                ask: !facts.to_begin,
            },
            "Home's independent part rules: {facts:?}"
        );
    }
    assert_eq!(
        parts::needs(needs),
        parts::NeedsParts {
            list: needs.known && needs.open != 0,
            done: needs.known && needs.open == 0,
            decided: needs.showing != 0,
            only: needs.known && needs.only != 0,
        },
        "Needs you's independent part rules: {needs:?}"
    );
}

fn check_content(
    home: Option<&parts::HomeFacts>,
    needs: &parts::NeedsFacts,
    brief_lines: usize,
    decisions: usize,
    projects: usize,
    open: usize,
) {
    if let Some(facts) = home {
        let shown = parts::home(facts);
        assert_eq!(shown.brief, brief_lines > 0 && !facts.to_begin, "brief has a line, or no new line is hidden");
        assert_eq!(shown.needs, open > 0 && !facts.to_begin, "Needs you has a card, or no open card is hidden");
        assert_eq!(shown.decided, decisions > 0 && !facts.to_begin, "Decided for you has a row");
        assert_eq!(shown.projects, facts.across && projects >= 2, "Projects has at least two rows");
        assert_eq!(shown.follows, facts.across && facts.follows > 0, "What follows what has a box");
    } else {
        let shown = parts::needs(needs);
        assert_eq!(shown.list, open > 0, "Needs you's list has a card");
        assert_eq!(shown.done, needs.known && open == 0, "Done hides no card");
        assert_eq!(shown.decided, decisions > 0, "Decided for you has a row");
        assert_eq!(shown.only, needs.known && needs.only != 0, "Only you shows exactly while an item needs its person");
    }
}

fn check_cards(world: &World, open: &[c::ObjectKey], count: u32) {
    let inbox = world.root.pages().inbox().expect("Home or Needs you owns an inbox");
    let mut expected = Vec::new();
    if inbox.known() {
        for card in &inbox.cards {
            if matches!(
                card.group,
                c::InboxGroup::Questions
                    | c::InboxGroup::Proposals
                    | c::InboxGroup::Escalations
                    | c::InboxGroup::PersonTasks
            ) && !world.answered.contains(&card.object)
            {
                expected.push(card.object);
            }
        }
        for row in world.root.forge().approvals().rows() {
            if inbox.filter.kind.is_none()
                && inbox.filter.project.is_none_or(|project| project == row.project)
                && inbox.filter.goal.is_none_or(|goal| row.goal == Some(goal))
            {
                expected.push(c::ObjectKey::Task(row.task));
            }
        }
    }
    assert_eq!(open.len(), expected.len(), "each open object is placed once");
    for object in &expected {
        assert_eq!(open.iter().filter(|found| *found == object).count(), 1, "open object {object:?} occurs once");
    }
    assert_eq!(usize::try_from(count).expect("bounded density fixture"), open.len(), "parts' open count");
    if !matches!(world.root.pages().address(), Address::Inbox { .. }) {
        let cards = open.iter().take(3).count();
        let more = usize::try_from(count.saturating_sub(3)).expect("small more count");
        assert_eq!(cards + more, expected.len(), "Home's three cards and N more count every open object");
    }
}

fn check_approval_sizes(world: &mut World, view: &mut temper_client_view::View) {
    use temper_client_view::look;
    let inbox = world.root.pages().inbox().expect("density inbox");
    let visible = match *world.root.pages().address() {
        Address::Inbox { .. } => {
            world.root.pages().needs().shown(inbox, world.root.forge().approvals()).into_iter().collect::<Vec<_>>()
        }
        Address::Home { project } if home::facts(&world.root, project, world.env.wall).to_begin() => Vec::new(),
        Address::AllHome | Address::Home { .. } => {
            world.root.pages().needs().needs(inbox, world.root.forge().approvals()).iter().copied().take(3).collect()
        }
        _ => unreachable!("density pages"),
    };
    let expected: Vec<_> = visible
        .iter()
        .filter_map(|need| match need {
            needs::Need::Approval(row) => Some(row.size),
            needs::Need::Card(_) => None,
        })
        .collect();
    if expected.is_empty() {
        return;
    }
    temper_client_view::build(view, &world.root, world.env.wall, &mut Queue::uncapped());
    let nodes = view.tree().nodes();
    let cards: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter_map(|(start, node)| {
            if !node.styles.as_slice().contains(&look::NEED_CARD) && !node.styles.as_slice().contains(&look::ASK) {
                return None;
            }
            let card = &nodes[start..start + usize::try_from(node.size).expect("card span")];
            card.iter().any(|n| n.name.as_deref() == Some(b"Approve A change on GitHub".as_slice())).then_some(card)
        })
        .collect();
    assert_eq!(cards.len(), expected.len(), "each visible approval has a card");
    for (card, size) in cards.into_iter().zip(expected) {
        let chips: Vec<_> =
            card.iter().enumerate().filter(|(_, n)| n.styles.as_slice().contains(&look::CHIPS)).collect();
        assert_eq!(chips.len(), usize::from(size.is_some()), "exactly one chips row with a size, none without");
        if let Some(size) = size {
            world.observed.insert(if size.cut { "approval with cut size" } else { "approval with size" });
            let (start, row) = chips[0];
            let row = &card[start..start + usize::try_from(row.size).expect("chips span")];
            let actual: Vec<_> = row.iter().flat_map(|n| n.text.iter()).flat_map(|r| r.text.iter().copied()).collect();
            let count = |n| format!("{n}{}", if size.cut && n == u32::MAX { "+" } else { "" });
            let expected = format!(
                "+{} -{} in {} {}",
                count(size.additions),
                count(size.deletions),
                count(size.files),
                if size.files == 1 { "file" } else { "files" }
            );
            assert_eq!(actual, expected.as_bytes(), "approval's size words");
        } else {
            world.observed.insert("approval without size");
        }
    }
}

fn check_settings(before: &[c::ObjectKey], after: &[c::ObjectKey], approvals: &[w::ApprovalRow]) {
    assert_eq!(before, after, "a settings revision keeps the open set and its order");
    for row in approvals {
        let object = c::ObjectKey::Task(row.task());
        if before.contains(&object) {
            assert!(after.contains(&object), "approval asked before a settings move stays open");
        }
    }
}

fn generate(seed: u64) -> World {
    let mut rng = Rng::new(seed);
    let projects = u32::try_from(rng.between(1, 4)).expect("bounded density fixture");
    let mut limits = fixtures::domain_limits();
    limits.client.domain.handles = READS;
    limits.client.domain.reads = READS;
    limits.client.domain.watches = READS;
    limits.client.domain.requests.requests = READS;
    limits.client.domain.text = 512;
    limits.client.domain.long_text = 512;
    limits.client.domain.turn_text = 512;
    let mut world = World {
        root: client::unlinked(&limits, seed),
        env: Env { now: Time::ZERO, wall: Wall::from_nanos(2 * DAY), limits },
        rng,
        projects,
        watches: Vec::new(),
        pending: Vec::new(),
        entries: Vec::new(),
        approvals: Vec::new(),
        accounts: None,
        decision_rows: Vec::new(),
        relation_rows: Vec::new(),
        answered: Vec::new(),
        ways: vec![0; usize::try_from(projects).expect("bounded density fixture")],
        revision: 0,
        step: 0,
        observed: BTreeSet::new(),
    };
    // Start and frame follow the domain's fixtures::start and Across fixture.
    for event in [
        Event::Start { address: Address::AllHome, kept: List::with_capacity(0), with: None },
        Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } },
        Event::Client { event: jig_client::Event::Up { up: Up::Served { served: true, versions: None } } },
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyServed {
                    family: wire::FORGE,
                    versions: Some(c::VersionRange { first: wire::CLIENT_VERSION, last: wire::CLIENT_VERSION }),
                },
            },
        },
    ] {
        world.event(event);
    }
    world.drain();
    world.deliver(&Watchable::Jig(c::Subject::Me), c::Delivery::Me(Box::new(frame(projects))));
    world.drain();
    world.family(
        forge::forges::watch(&limits.forge),
        wire::forges(
            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: List::with_capacity(0), linked: false })
                .expect("bounded density fixture"),
        ),
    );
    world.dismissals();
    world.entries = (10..=13).map(|task| entry(task, world.project_of(task))).collect();
    world.inbox();
    world.approvals = vec![approval(1, &mut world.rng)];
    world.approval_rows();
    assert!(world.open().contains(&c::ObjectKey::Task(201)), "approval asked under You approve is open");
    world.settings(1);
    world.settings(1);
    for project in 1..=projects {
        world.status(project);
        world.decisions(project);
    }
    world.drain();
    world
}

fn sweep(seed: u64, words: &temper_client_view::words::Words) -> BTreeSet<&'static str> {
    let mut world = generate(seed);
    let bounds = fixtures::view_bounds(&world.env.limits, words);
    let mut view = temper_client_view::View::new(&bounds, &world.env.limits, words.clone(), fixtures::FORGE);
    world.observed.insert(match world.projects {
        1 => "one project",
        2 => "two projects",
        3 => "three projects",
        4 => "four projects",
        _ => unreachable!("one to four projects"),
    });
    for step in 0..STEPS {
        world.step = step;
        let project = u32::try_from(world.rng.between(1, u64::from(world.projects))).expect("bounded density fixture");
        match world.rng.below(16) {
            0 => {
                world.observed.insert("All projects Home");
                world.go(Address::AllHome);
            }
            1 => {
                world.observed.insert("project Home");
                world.go(Address::Home { project });
            }
            2 => {
                world.observed.insert("unfiltered Needs you");
                world.go(Address::Inbox { filter: Filter::ALL });
            }
            3 => {
                world.observed.insert("filtered Needs you");
                world.go(Address::Inbox { filter: Filter { project: Some(project), ..Filter::ALL } });
            }
            4 => world.status(project),
            5 => world.decisions(project),
            6 => {
                world.entries.clear();
                for task in 10..=13 {
                    if world.rng.below(3) != 0 {
                        world.entries.push(entry(task, world.project_of(task)));
                    }
                }
                world.inbox();
            }
            7 => {
                world.approvals.clear();
                for project in 1..=world.projects {
                    if world.rng.below(2) == 0 {
                        world.approvals.push(approval(project, &mut world.rng));
                    }
                }
                world.approval_rows();
            }
            8 => world.settings(project),
            9 => {
                if world.open().contains(&QUESTION) && !world.answered.contains(&QUESTION) {
                    world.observed.insert("held answer");
                    world.answered.push(QUESTION);
                    world.act(needs::Act::Choose { object: QUESTION, words: b"Small".as_slice().into() });
                    assert!(world.root.pages().needs().held(QUESTION), "choice is held before sending");
                }
            }
            10 => {
                if world.root.pages().needs().held(QUESTION) {
                    world.observed.insert("undone answer");
                    world.answered.retain(|object| *object != QUESTION);
                    world.act(needs::Act::Undo { object: QUESTION });
                }
            }
            11 => {
                let elapsed = [SECOND, needs::HOLD.as_nanos(), DAY]
                    [usize::try_from(world.rng.below(3)).expect("bounded density fixture")];
                world.env.now = world.env.now.saturating_add(Duration::from_nanos(elapsed));
                world.env.wall = Wall::from_nanos(world.env.wall.as_nanos() + elapsed);
                let mut out = Queue::uncapped();
                client::fire(&mut world.root, &world.env, &mut out);
                world.outputs(&out);
                world.check();
            }
            12 => world.dismissals(),
            13 => world.go(Address::Inbox { filter: Filter { kind: Some(Kind::Question), ..Filter::ALL } }),
            14 => world.accounts(),
            15 => world.relations(project),
            _ => unreachable!("sixteen generator moves"),
        }
        world.drain();
        check_approval_sizes(&mut world, &mut view);
    }
    world.observed
}

#[test]
fn random_facts_and_settings_keep_each_part_and_card() {
    let words =
        temper_client_view::words::Words::english(fixtures::FORGE, 512, temper_client_view::words::Shell::Browser)
            .expect("density words");
    let mut observed = BTreeSet::new();
    for seed in REPLAY.map_or(1..=SEEDS, |seed| seed..=seed) {
        let outcome = catch_unwind(AssertUnwindSafe(|| sweep(seed, &words)));
        assert!(outcome.is_ok(), "density seed {seed}; replay with REPLAY = Some({seed})");
        observed.extend(outcome.expect("seed completed"));
    }
    if REPLAY.is_none() {
        for state in [
            "one project",
            "two projects",
            "three projects",
            "four projects",
            "All projects Home",
            "project Home",
            "unfiltered Needs you",
            "filtered Needs you",
            "To begin",
            "everyday Home",
            "N more",
            "at most three cards",
            "decisions showing",
            "no decisions",
            "new brief",
            "no new brief",
            "relations showing",
            "no relations",
            "held answer",
            "undone answer",
            "sent answer",
            "settings with approval",
            "expired decisions",
            "an Only you item",
            "no Only you item",
            "approval without size",
            "approval with size",
            "approval with cut size",
        ] {
            assert!(observed.contains(state), "density sweep did not reach {state}");
        }
    }
}

fn text(words: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(words.into(), 512).expect("bounded density fixture"))
}
fn offers(act: c::ActKind) -> List<c::Offered> {
    List::from_boxed(Box::new([c::Offered { act, admission: c::Admission::Allowed, effect: None }]))
}
fn frame(projects: u32) -> c::Me {
    let projects = (1..=projects)
        .map(|project| c::MeProject {
            project,
            name: text(format!("Project {project}").as_bytes()),
            role: c::HeldRole::Owner,
            pool: None,
            templates: List::from_boxed(Box::new([chat_template()])),
            creating: None,
            models: List::with_capacity(0),
            left_out: 0,
            defaults: List::with_capacity(0),
            public: None,
        })
        .collect::<Vec<_>>();
    c::Me {
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 256,
        words_bytes: 512,
        party: c::MeParty { number: 1, kind: c::PartyKind::Person, login: text(b"person"), name: text(b"Person") },
        projects: List::from_boxed(projects.into_boxed_slice()),
        needs: c::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: Duration::from_secs(1000),
        deployment_role: Some(c::HeldRole::Owner),
    }
}
// The chat template follows the domain's fixtures::chats::me, so Home's task reads settle.
fn chat_template() -> c::Template {
    c::Template {
        resources: List::with_capacity(0),
        number: 1,
        name: text(b"Chat"),
        executor: c::TemplateExecutor::Charter(1),
        contract: c::TaskContractDocument::Report { words: 512 },
        shallower: 1,
        budget: c::TemplateBudget::AsAllowed { cap: None },
        deadline: c::TemplateDeadline::None,
        tracked: None,
        flags: c::TaskFlags {
            ended_by: c::EndedBy::Result,
            conversation: true,
            listed: false,
            retention: c::Retention::Kept,
        },
        label: 7,
        grace: Some(Duration::from_secs(5)),
        llm: None,
    }
}
fn entry(task: u64, project: u32) -> c::InboxEntry {
    let object = match task {
        10 => c::InboxObject::Question { message: 1 },
        11 => c::InboxObject::Proposal { proposer: c::Proposer::Task(11), number: 1 },
        12 | 13 => c::InboxObject::Escalation { revision: 1 },
        _ => unreachable!("four card kinds"),
    };
    c::InboxEntry {
        task: c::TaskLink::Stored { number: task, status: None },
        object,
        project: Some(project),
        activity: Some(1),
        at: Some(Wall::EPOCH),
        unread: None,
        cursor: c::InboxCursor::Inbox { task, object, at: Some(Wall::EPOCH) },
        reply: None,
        projects: List::with_capacity(0),
    }
}
fn approval(project: u32, rng: &mut Rng) -> w::ApprovalRow {
    let size = match rng.below(3) {
        0 => None,
        kind => {
            let mut counts = [0; 3];
            for count in &mut counts {
                *count = u32::try_from(rng.between(0, 1000)).expect("small size");
            }
            if kind == 2 {
                counts[usize::try_from(rng.below(3)).expect("three counts")] = u32::MAX;
            }
            Some(
                w::ApprovalSize::new(
                    &w::CEILINGS,
                    w::ApprovalSizeParts {
                        files: counts[0],
                        additions: counts[1],
                        deletions: counts[2],
                        cut: kind == 2,
                    },
                )
                .expect("size"),
            )
        }
    };
    w::ApprovalRow::new(
        &w::CEILINGS,
        w::ApprovalRowParts {
            task: 200 + u64::from(project),
            project,
            goal: None,
            forge: 3,
            kind: w::Kind::Github,
            owner: b"acme".as_slice().into(),
            name: b"widget".as_slice().into(),
            pull: 24,
            address: b"https://github.com/acme/widget/pull/24/files".as_slice().into(),
            title: b"A change".as_slice().into(),
            since: SECOND,
            scope: project,
            rule: 4,
            role: 5,
            required: 2,
            counted: 1,
            sets: List::with_capacity(0),
            whole: true,
            yours: w::ApprovalYours::Counts,
            size,
        },
    )
    .expect("bounded density fixture")
}
fn budget() -> c::Budget {
    c::Budget {
        numbers: c::Numbers { budget: 100, spent: 100, spent_below: 0, reserved: 0 },
        left: 0,
        funder: c::Funder::Period { project: 1, period: 1 },
        deadline: None,
    }
}
fn wake() -> c::WakePolicy {
    c::WakePolicy {
        words: c::WakeRule::Immediate,
        notices: c::WakeRule::Never,
        news: c::WakeRule::Never,
        results: c::ResultsWake::Each,
        questions: true,
        answers: true,
        timers: false,
    }
}
fn chip(task: u64, project: u32) -> c::Chip {
    c::Chip {
        task,
        title: text(b"Make a change"),
        executor: c::ChipExecutor::Party { party: 1 },
        phase: c::Phase::Waiting,
        details: Box::new(c::ChipPhaseFacts {
            phase: c::ChipPhase::Waiting { dependencies: List::with_capacity(0), more: false, holds: false },
            details: Box::new(c::ChipDetails {
                dependencies: List::with_capacity(0),
                project,
                executor: c::Executor::Person(c::PersonAddress::Party(1)),
                phase_since: Wall::EPOCH,
                run_since: None,
                attempt: None,
                retry: None,
                wake: wake(),
                person_wait: None,
                hold: None,
                budget: budget(),
                activity: 0,
                resources: List::with_capacity(0),
                resources_cut: false,
                resources_observed: false,
                procedure: None,
                live_delegates: 0,
                closing_connectors: None,
                ending: None,
                ended_at: None,
                observation: None,
            }),
        }),
    }
}
fn task_document(task: u64, project: u32) -> c::Task {
    c::Task {
        settings: None,
        label: 0,
        flags: c::TaskFlags {
            conversation: false,
            listed: true,
            retention: c::Retention::Kept,
            ended_by: c::EndedBy::Result,
        },
        last_activity: Wall::EPOCH,
        made: Wall::EPOCH,
        waits_on: c::WaitsOn::Nothing,
        chip: chip(task, project),
        goal: None,
        priority: None,
        lineage_chips: List::with_capacity(0),
        inputs: List::with_capacity(0),
        project,
        requester: c::Requester::Person(1),
        lineage: List::with_capacity(0),
        spec: c::SpecificationDocument::complete(c::Specification {
            asks: false,
            words: c::Text::new(b"Make a change".as_slice().into(), 512).expect("bounded density fixture"),
            parameters: List::with_capacity(0),
            inputs: List::with_capacity(0),
        }),
        spec_revision: c::Revision { value: 1 },
        person_revision: None,
        taken_by: None,
        instructions: None,
        wake: wake(),
        subscriptions: c::Subscriptions { entries: List::with_capacity(0) },
        result_contract: c::TaskContractDocument::Report { words: 512 },
        dependencies: List::with_capacity(0),
        dependency_facts: List::with_capacity(0),
        references: List::with_capacity(0),
        cut: false,
        resources: List::with_capacity(0),
        budget: budget(),
        authority: c::AuthorityDocument::complete(c::Authority {
            models: None,
            tools: 0,
            grants: List::with_capacity(0),
            delegation: c::Delegation { kinds: List::with_capacity(0), tasks: 0, depth: 0 },
            spend: 100,
            deadline: None,
            notes: 0,
            note_resources: List::with_capacity(0),
        }),
        hold: None,
        proposals: List::with_capacity(0),
        activity: 0,
        changed: Wall::EPOCH,
        acts: offers(c::ActKind::Stop),
        result: None,
        delegates: List::with_capacity(0),
        context: None,
        llm: None,
        kept_account: None,
        run_wait: None,
    }
}
