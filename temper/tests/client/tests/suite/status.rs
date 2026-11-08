//! Home's entry looks cross the real roots (temper/docs/design/domain/testing.md,
//! 2.1; temper/docs/design/domain/forge.md, 5.6; temper/docs/design/client/README.md, 3.5).
use jig_client::{Handle, Key, Latest};
use jig_client_kit::harness::{App, ClientId, Duty, Front};
use skein_lib::{Duration, Env, Queue, Time, Wall};
use temper_client_domain::{
    Action, Address, Event, Request,
    forge::dismissed::{self, PersonFact},
    forge::status,
    owner::Owner,
    pages::{
        Act, all_home,
        home::{self, Facts, Status},
    },
};
use temper_client_protocol_wire::v1 as w;
use temper_client_world::{
    app::{Root, Temper},
    fixtures,
    front::Joint,
};
use temper_engine_domain::{Key as StoreKey, Record};
use temper_engine_domain_forge as forge;

const CLIENT: ClientId = ClientId(0);
const PROJECT: u32 = 1;
const OTHER_PROJECT: u32 = 2;
const PUMP_ROUNDS: usize = 32;
// Bounds immediate forge progression; no real clock or poll wait is needed.
const LANDING_ROUNDS: usize = 200;
const SECOND_DEVICE: ClientId = ClientId(1);
const OTHER_CLIENT: ClientId = ClientId(2);
// The owner and its two sign-ins occupy the first three people identifiers.
const OTHER_PARTY: u64 = 4;
const OTHER_SUBJECT: u64 = 8;
const WATCH: Handle = Handle { asker: Owner::Forge.asker(), number: 2202 };
const OFFLINE: Duration = Duration::from_millis(30);
// temper/docs/design/domain/forge.md, 5.7: the root's status procedure.
const STATUS_PROCEDURE: u32 = 5;

struct Probe {
    root: Root,
    out: Queue<Request>,
    looks: Vec<u32>,
}

struct Story {
    front: Joint,
    probes: Vec<Probe>,
    env: Env<temper_client_domain::Limits>,
    refuse_status: Option<u32>,
    status_ends: Vec<jig_client::contract::WatchEnd>,
}

impl Story {
    fn new() -> Self {
        let mut limits = fixtures::domain_limits();
        // A new page opens before the previous page's handles finish retiring.
        // The session also holds trials and a running trial task.
        limits.client.domain.handles = 18;
        limits.client.domain.watches = 18;
        Self {
            front: Joint::with_status_people(fixtures::SEED, limits),
            probes: Vec::new(),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            refuse_status: None,
            status_ends: Vec::new(),
        }
    }

    fn client(&mut self, party: u64, address: Address) -> ClientId {
        let who = ClientId(self.probes.len());
        self.front.bind(who, party);
        let root = Root::Person(Box::new(temper_client_domain::unlinked(&self.env.limits, fixtures::SEED)));
        self.probes.push(Probe { root, out: Queue::uncapped(), looks: Vec::new() });
        self.event(who, Temper::start(address, Vec::new(), 0));
        who
    }

    fn with_landing() -> Self {
        let mut story = Self::new();
        story.front = Joint::with_status_landing(fixtures::SEED, story.env.limits);
        story
    }

    fn land(&mut self) {
        let goal = self.front.begin_status_landing();
        self.pump();
        for _ in 0..LANDING_ROUNDS {
            self.front.approve_status_landing(goal);
            self.after(Duration::from_millis(10));
            if self.front.engine.task(goal).is_none() {
                let Record::Forge { row } = self
                    .front
                    .engine
                    .store
                    .rows
                    .get(&StoreKey::Forge(forge::Key::ProjectFacts { project: PROJECT }))
                    .expect("the landing committed project facts")
                else {
                    panic!("forge facts")
                };
                let forge::Stored::ProjectFacts(row) = row.as_ref() else { panic!("project facts row") };
                assert_eq!(row.landed, 1, "the real landing counted once");
                assert!(
                    self.front.engine.store.rows.values().any(|record| {
                        matches!(record, Record::Forge { row } if matches!(row.as_ref(),
                        forge::Stored::LandingLog(log) if log.landings.iter()
                            .any(|landing| landing.changes.iter().any(|change| change.task == goal))))
                    }),
                    "the real landing log holds the change"
                );
                return;
            }
        }
        panic!("the change did not land");
    }

    fn event(&mut self, who: ClientId, event: temper_client_world::app::Event) {
        let probe = &mut self.probes[who.0];
        Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
        self.pump();
    }

    fn pump(&mut self) {
        for _ in 0..PUMP_ROUNDS {
            let mut worked = false;
            for (index, probe) in self.probes.iter_mut().enumerate() {
                Temper::take(&mut probe.root, &self.env, &mut probe.out);
                while let Some(request) = probe.out.pop() {
                    worked = true;
                    match Temper::duty(request) {
                        Duty::Wire(mut request) => {
                            for project in [PROJECT, OTHER_PROJECT] {
                                if let Request::Unlinked {
                                    request:
                                        jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { request, .. } },
                                } = &request
                                    && Some(request) == status::ask(project, &self.env.limits.forge).as_ref()
                                {
                                    probe.looks.push(project);
                                }
                                if self.refuse_status == Some(project)
                                    && let Request::Unlinked {
                                        request:
                                            jig_client::UnlinkedRequest::Down {
                                                down: jig_client::Down::Open { subject, .. },
                                            },
                                    } = &mut request
                                    && Some(&*subject) == status::watch(project, &self.env.limits.forge).as_ref()
                                    && let jig_client::Watchable::Family { scope, .. } = subject
                                {
                                    // The real engine refuses this watch's unowned scope.
                                    *scope = jig_client::contract::Scope::Project(u32::MAX);
                                }
                            }
                            self.front.wire(ClientId(index), self.env.now, self.env.wall, request);
                        }
                        Duty::Show { address, replace } => {
                            self.front.shows.push((address.clone(), replace));
                            Temper::step(&mut probe.root, &self.env, Temper::went(address), &mut probe.out);
                        }
                        Duty::Store(_) | Duty::SignIn { .. } | Duty::SignOut | Duty::Fact => {}
                    }
                }
            }
            let callbacks = self.front.poll(self.env.now, self.env.wall);
            if !worked && callbacks.is_empty() {
                return;
            }
            for (who, event) in callbacks {
                self.front.received(who, &event);
                if let Some(Event::Client {
                    event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { end, .. } },
                }) = event.root()
                {
                    self.status_ends.push(*end);
                }
                let probe = &mut self.probes[who.0];
                Temper::step(&mut probe.root, &self.env, event, &mut probe.out);
            }
        }
        panic!("status clients and engine did not settle");
    }

    fn enter(&mut self) {
        assert_eq!(self.client(fixtures::PARTY, fixtures::inbox()), CLIENT);
        // Keep the real dismissal before this Home's first entry, so To begin is already skipped.
        self.event(
            CLIENT,
            Event::Client {
                event: jig_client::Event::Send {
                    key: Key { asker: Owner::Forge.asker(), number: 2201 },
                    name: None,
                    request: dismissed::ask(PersonFact::ToBegin { project: PROJECT }, &self.env.limits.forge)
                        .expect("skip request"),
                },
            }
            .into(),
        );
        self.go(CLIENT, Address::Home { project: PROJECT });
    }

    fn go(&mut self, who: ClientId, address: Address) {
        self.event(who, Temper::went(address));
    }

    fn after(&mut self, elapsed: Duration) {
        self.env.now = self.env.now.saturating_add(elapsed);
        self.env.wall = Wall::from_nanos(self.env.now.as_nanos());
        for probe in &mut self.probes {
            Temper::fire(&mut probe.root, &self.env, &mut probe.out);
        }
        self.pump();
    }

    fn facts(&self) -> Facts {
        let root = self.probes[CLIENT.0].root.person();
        assert_eq!(root.pages().address(), &Address::Home { project: PROJECT });
        home::facts(root, PROJECT, skein_lib::Wall::EPOCH)
    }

    fn shown(&self) -> w::ProjectStatus {
        let Status::Shown(document) = self.facts().status else {
            panic!(
                "Home's status is shown at {:?}, link {:?}, row {:?}: {:?}",
                self.env.now,
                self.probes[CLIENT.0].root.person().client().link(),
                self.looked(fixtures::PARTY),
                self.facts()
            )
        };
        *document
    }

    fn looked(&self, party: u64) -> Option<forge::Looked> {
        self.looked_on(party, PROJECT)
    }

    fn looked_on(&self, party: u64, project: u32) -> Option<forge::Looked> {
        self.front.engine.store.rows.get(&StoreKey::Forge(forge::Key::Looked { party, project })).map(|record| {
            let Record::Forge { row } = record else { panic!("forge record") };
            let forge::Stored::Looked(row) = row.as_ref() else { panic!("looked row") };
            row.clone()
        })
    }

    fn view(&self) -> temper_client_world::app::View {
        let (_, mut view) = Temper::new(fixtures::SEED);
        let temper_client_world::app::View::Person(person) = &mut view else { panic!("person view") };
        temper_client_view::build(person, self.probes[CLIENT.0].root.person(), self.env.wall, &mut Queue::uncapped());
        view
    }

    fn words(&self) -> String {
        let view = self.view();
        let text: Vec<u8> = Temper::tree(&view)
            .nodes()
            .iter()
            .flat_map(|node| node.text.iter())
            .flat_map(|run| run.text.iter().copied())
            .collect();
        String::from_utf8(text).expect("Home words")
    }

    fn rows(&self) -> Vec<(StoreKey, Record)> {
        self.front
            .engine
            .store
            .rows
            .iter()
            .filter_map(|(key, record)| {
                if matches!(key, StoreKey::Forge(forge::Key::ProjectFacts { .. } | forge::Key::Looked { .. })) {
                    Some((key.clone(), record.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    fn watch(&mut self, who: ClientId) {
        // Observers use no settings, accounts or inbox; keep the shared engine watch table for Home's facts.
        for handle in [
            self.probes[who.0].root.person().forge().settings_handle(),
            temper_client_domain::pages::accounts::handle(),
            temper_client_domain::pages::needs::handle(),
        ] {
            self.event(who, Event::Client { event: jig_client::Event::Unwatch { handle } }.into());
        }
        self.event(
            who,
            Event::Client {
                event: jig_client::Event::Watch {
                    handle: WATCH,
                    subject: status::watch(PROJECT, &self.env.limits.forge).expect("status watch"),
                },
            }
            .into(),
        );
    }

    fn watched(&self, who: ClientId) -> w::ProjectStatus {
        let client = self.probes[who.0].root.person().client();
        let document = client.document(WATCH).expect("status handle");
        assert_eq!(document.state, jig_client::State::Live, "status watch stays live");
        let Some(Latest::Family(record)) = document.latest else { panic!("status document delivered to {who:?}") };
        status::document(record, self.env.limits.forge).expect("decoded status")
    }

    fn look(&mut self, who: ClientId, number: u64) {
        self.event(
            who,
            Event::Client {
                event: jig_client::Event::Send {
                    key: Key { asker: Owner::Forge.asker(), number },
                    name: None,
                    request: status::ask(PROJECT, &self.env.limits.forge).expect("look request"),
                },
            }
            .into(),
        );
    }
}

fn assert_only_plan(story: &Story) {
    use jig_client::contract as c;
    let document = temper_client_domain::pages::accounts::document(story.probes[CLIENT.0].root.person().client())
        .expect("person's account facts");
    assert_eq!(document.accounts.len(), 1, "only the person's plan can pay");
    let account = document.accounts.get(0).expect("the person's only account");
    assert_eq!(account.holder, c::AccountHolder::Party(fixtures::PARTY));
    assert_eq!(account.provider, Some(1));
    assert!(account.plan);
    assert_eq!(account.sources.len(), 1);
    let source = account.sources.get(0).expect("the plan's only source");
    assert_eq!(source.kind, c::AccountKind::SignIn);
    assert!(source.usable, "the plan stays usable");
    assert!(
        story.front.engine.store.rows.values().all(|record| !matches!(
            record,
            Record::Core(jig_domain_core::Record::Accounts(jig_domain_accounts::Stored::Account {
                holder: jig_domain_accounts::Holder::Deployment,
                ..
            }))
        )),
        "the engine keeps no deployment account"
    );
}

fn sign_in_plan(story: &mut Story) -> u32 {
    use jig_domain_accounts as a;
    use temper_client_domain::pages::{accounts, connect_model};
    story.client(fixtures::PARTY, Address::ConnectModel { way: None });
    story.event(
        CLIENT,
        Event::Act {
            action: Action::Pages(Act::ConnectModel(connect_model::Act::SignIn {
                name: [114; 16],
                way: accounts::Way::SignIn { provider: 1, plan: true },
            })),
        }
        .into(),
    );
    let source = temper_client_world::front::account_acts::signing(&story.front);
    story.front.engine.protocol.sign_in_sources.insert(source);
    for event in [
        a::Event::Visited { source, address: b"https://issuer/visit".as_slice().into() },
        a::Event::SignedIn {
            source,
            subject: b"plan-holder".as_slice().into(),
            display: b"Plan holder".as_slice().into(),
            declined: false,
            said: a::Said::default(),
        },
        // The scripted issuer's grant spans both sittings, as the person's session does.
        a::Event::Component(a::FromComponent::Granted {
            account: source,
            generation: 2,
            valid: Duration::from_secs(2 * 60 * 60),
            form: jig_domain_link::Form::Credential(jig_domain_link::Scheme::Bearer),
        }),
    ] {
        story.front.engine.send(temper_engine_domain::Event::Account(event));
        story.pump();
    }
    temper_client_world::front::account_acts::answer_trial(&mut story.front, &mut story.probes[CLIENT.0].root);
    story.pump();
    source
}

#[test]
fn a_plans_project_has_its_brief_on_a_return_past_the_sitting() {
    use jig_domain_tasks as tasks;
    const REPORT: &[u8] = b"1 change landed on the local forge.\n\nThe README now records the local status change.";
    let mut story = Story::new();
    // The stock's creation offers use the ordinary client's document envelope.
    story.env.limits.client = temper_client_tools::limits::client().client;
    story.front = Joint::with_plan_status(fixtures::SEED, story.env.limits);
    story.front.engine.forge.as_mut().expect("fake forge").grant(
        b"acme/shop",
        2,
        skein_fake_forge_domain::api::Permission::None,
    );
    fixtures::adopt_change_repository(&mut story.front);
    let source = sign_in_plan(&mut story);
    assert_only_plan(&story);
    let holdings = story
        .front
        .engine
        .store
        .rows
        .values()
        .find_map(|record| {
            if let Record::Core(jig_domain_core::Record::People(jig_domain_people::Stored::Roles {
                project: PROJECT,
                holdings,
            })) = record
            {
                Some(holdings)
            } else {
                None
            }
        })
        .expect("project membership");
    assert_eq!(
        holdings.as_ref(),
        &[jig_domain_people::Holding { person: fixtures::PARTY, role: jig_domain_people::Role::Owner }]
    );
    story.go(CLIENT, Address::Home { project: PROJECT });
    story.event(CLIENT, Event::Act { action: Action::Pages(Act::SkipToBegin) }.into());
    assert_eq!(story.shown().previous(), &None);
    assert_eq!(story.facts().brief(), None);
    assert!(!story.words().contains("Written by the status agent"));
    story.go(CLIENT, fixtures::inbox());
    story.land();
    story.after(Duration::from_secs(29 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert_eq!(story.facts().brief(), None);
    assert!(!story.words().contains("Written by the status agent"));
    assert!(story.front.assignments.iter().all(|run| run.charter != 9), "no status run within the sitting");
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(31 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    let run = story
        .front
        .assignments
        .iter()
        .find(|run| run.charter == 9)
        .expect("the stock period funds the status worker beyond the owner's open pool")
        .clone();
    assert_eq!(story.front.hosted_attempt(run.task), Some(run.attempt), "the worker received its assignment");
    assert!(run.grants.iter().any(|grant| grant.account == source), "the status worker borrows the person's plan");
    let task = story.front.engine.task(run.task).expect("admitted status task");
    let tasks::Party::Task(parent) = task.requester else { panic!("status procedure parent") };
    assert_eq!(
        story.front.engine.task(parent).expect("status procedure").executor,
        tasks::Executor::Procedure { connector: fixtures::FORGE, code: STATUS_PROCEDURE }
    );
    story.front.engine.answer(
        fixtures::QUESTION_WORKER,
        run.task,
        run.attempt,
        0,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: REPORT.into() }, cancel_delegates: false },
        temper_engine_domain::Parts::none(),
    );
    story.pump();
    let brief = story.facts().brief().expect("production procedure published the brief").clone();
    assert_eq!(brief.sentence(), b"1 change landed on the local forge.");
    assert_eq!(brief.lede(), b"The README now records the local status change.");
    let words = story.words();
    for expected in [
        "1 change landed on the local forge.",
        "The README now records the local status change.",
        "Written by the status agent",
    ] {
        assert!(words.contains(expected), "Home shows {expected}: {words}");
    }
    assert_only_plan(&story);
}

#[test]
fn entering_home_looks_once_and_shows_status() {
    let mut story = Story::new();
    story.enter();
    let row = story.looked(fixtures::PARTY).expect("committed look");
    assert_eq!(row.previous, None);
    assert_eq!(row.latest, story.env.wall);
    assert_eq!((row.seen, row.next_seen), (0, 0));
    assert_eq!(story.shown().previous(), &None);
    assert_eq!(story.facts().landed_since(), 0);
    assert_eq!(story.facts().brief(), None);
    assert_eq!(story.front.status_looks(CLIENT), 1);
    story.pump();
    assert_eq!(story.front.status_looks(CLIENT), 1, "the settled Home sends no second look");
}

impl Story {
    fn across() -> Self {
        let mut story = Self::new();
        story.front = fixtures::across_people(story.env.limits);
        story
    }

    fn across_landings() -> Self {
        use jig_domain_authority::Executor;
        use temper_engine_domain_world::{store::Store, walking};
        let mut story = Self::new();
        let (mut config, mut bounds) = fixtures::across_configuration();
        bounds.tasks.charters = 2; // The ordinary charter and the status agent.
        bounds.tasks.charter_documents = 4;
        bounds.tasks.executor_kinds = 2;
        bounds.authority.executors = 2;
        bounds.documents.executors = 2;
        let mut status = walking::charter();
        status.number = 9;
        config.core.defaults.charters = Box::new([walking::charter(), status]);
        let kinds: Box<[Executor]> = Box::new([Executor::Charter(1), Executor::Charter(9)]);
        config.core.defaults.rules.ceiling.delegation.kinds.clone_from(&kinds);
        for project in &mut config.core.seed_projects {
            project.policy.policy.ceiling.delegation.kinds.clone_from(&kinds);
        }
        let bootstrap = temper_engine_domain_world::oracle::config::copy(&config);
        let mut retained = temper_engine_domain_world::fixtures::restored_over(Store::new(), bootstrap, &bounds);
        // Committed landing history, as in the refused-run story; both briefs below come from the procedure.
        let rows = [PROJECT, OTHER_PROJECT]
            .into_iter()
            .flat_map(|project| {
                [
                    Record::Forge {
                        row: Box::new(forge::Stored::ProjectFacts(forge::ProjectFacts {
                            project,
                            landed: u64::from(project),
                            last_landed: Wall::EPOCH,
                            brief: None,
                            running: None,
                        })),
                    },
                    Record::Forge {
                        row: Box::new(forge::Stored::Looked(forge::Looked {
                            party: fixtures::PARTY,
                            project,
                            previous: None,
                            latest: Wall::EPOCH,
                            seen: 0,
                            next_seen: 0,
                        })),
                    },
                ]
            })
            .collect();
        retained.seed(rows, Box::new([]));
        let restart = temper_engine_domain_world::oracle::config::copy(&config);
        story.front = Joint::with_engine(fixtures::SEED, &story.env.limits, retained, [config, restart], &bounds, 4);
        for project in [PROJECT, OTHER_PROJECT] {
            story.front.engine.send(temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Period {
                project,
                period: 1,
                budget: 1000,
                next: None,
            }));
            story.front.engine.settle();
        }
        story.after(Duration::from_secs(31 * 60));
        story
    }
}

#[test]
fn all_home_looks_per_entry_and_joins_both_status_procedures_briefs() {
    use jig_domain_tasks as tasks;
    let mut story = Story::across_landings();
    story.client(fixtures::PARTY, Address::AllHome);
    let runs: Vec<_> = story
        .front
        .assignments
        .iter()
        .filter(|run| run.charter == 9)
        .map(|run| (story.front.engine.task(run.task).expect("status agent").project, run.clone()))
        .collect();
    assert_eq!(runs.len(), 2, "both projects' status procedures assigned their agent");
    for (project, report) in [
        (PROJECT, b"One change landed.\n\nThe first project is ready. More detail stays on its Home.".as_slice()),
        (
            OTHER_PROJECT,
            b"Two changes landed.\n\nThe second project is ready! More detail stays on its Home.".as_slice(),
        ),
    ] {
        let (_, run) = runs.iter().find(|(held, _)| *held == project).expect("project's status agent");
        let tasks::Party::Task(parent) = story.front.engine.task(run.task).expect("status agent").requester else {
            panic!("status procedure parent")
        };
        assert_eq!(
            story.front.engine.task(parent).expect("status procedure").executor,
            tasks::Executor::Procedure { connector: fixtures::FORGE, code: STATUS_PROCEDURE }
        );
        story.front.engine.answer(
            fixtures::QUESTION_WORKER,
            run.task,
            run.attempt,
            0,
            tasks::End::Finished {
                result: tasks::TaskResult::Report { words: report.into() },
                cancel_delegates: false,
            },
            temper_engine_domain::Parts::none(),
        );
        story.pump();
        story.after(Duration::from_secs(1));
    }
    let facts = all_home::facts(story.probes[CLIENT.0].root.person(), story.env.wall);
    assert_eq!(facts.projects.iter().map(|row| row.project).collect::<Vec<_>>(), [PROJECT, OTHER_PROJECT]);
    for row in &facts.projects {
        let Status::Shown(document) = &row.status else { panic!("project {} has its status", row.project) };
        assert_eq!((document.project(), document.landed(), document.seen()), (row.project, u64::from(row.project), 0));
    }
    let intro = all_home::intro(&facts);
    assert_eq!(intro.landed, all_home::Landed::Across { count: 3, projects: 2 });
    let brief = intro.brief.expect("the procedure briefs join across projects");
    assert_eq!(
        brief.lines.iter().map(|line| (line.project, line.name.as_ref(), line.sentence.as_ref())).collect::<Vec<_>>(),
        [
            (OTHER_PROJECT, b"Project 2".as_slice(), b"The second project is ready!".as_slice()),
            (PROJECT, b"Project 1".as_slice(), b"The first project is ready.".as_slice()),
        ]
    );
    let first: Vec<_> = [PROJECT, OTHER_PROJECT]
        .into_iter()
        .map(|project| {
            assert_eq!(story.probes[CLIENT.0].looks.iter().filter(|sent| **sent == project).count(), 1);
            story.looked_on(fixtures::PARTY, project).expect("committed entry look")
        })
        .collect();
    story.pump();
    assert_eq!(story.probes[CLIENT.0].looks.len(), 2, "the settled entry sends no duplicate look");
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(29 * 60));
    story.go(CLIENT, Address::AllHome);
    for before in first {
        let after = story.looked_on(fixtures::PARTY, before.project).expect("return look committed");
        assert_eq!(story.probes[CLIENT.0].looks.iter().filter(|project| **project == before.project).count(), 2);
        assert_eq!((after.previous, after.seen), (before.previous, before.seen));
        assert_eq!(after.latest, story.env.wall);
        assert!(after.latest > before.latest);
    }
}

#[test]
fn a_refused_projects_status_leaves_the_other_project_shown() {
    let mut story = Story::across();
    story.refuse_status = Some(OTHER_PROJECT);
    story.client(fixtures::PARTY, Address::AllHome);
    assert!(
        story.status_ends.contains(&jig_client::contract::WatchEnd::NotAllowed),
        "the real engine refused the watch"
    );
    let facts = all_home::facts(story.probes[CLIENT.0].root.person(), story.env.wall);
    assert_eq!(facts.projects.len(), 2);
    assert!(matches!(&facts.projects[0].status, Status::Shown(document) if document.project() == PROJECT));
    assert_eq!((facts.projects[1].project, &facts.projects[1].status), (OTHER_PROJECT, &Status::Unknown));
}

#[test]
fn all_home_asks_in_the_open_project_without_starting_a_chat() {
    const WORDS: &[u8] = b"Explain the second project's landing";
    let mut story = Story::across();
    story.client(fixtures::PARTY, Address::Home { project: OTHER_PROJECT });
    story.go(CLIENT, Address::AllHome);
    let pages = story.probes[CLIENT.0].root.person().pages();
    assert_eq!(pages.project(), Some(OTHER_PROJECT));
    let before = story.front.engine.store.rows.clone();
    story.event(CLIENT, Event::Act { action: Action::Pages(Act::AskLine { words: WORDS.into() }) }.into());
    assert_eq!(story.probes[CLIENT.0].root.person().pages().ask_line(), WORDS);
    story.event(CLIENT, Event::Act { action: Action::Pages(Act::AskWith { words: WORDS.into() }) }.into());
    assert_eq!(story.front.shows.last().map(|(address, _)| address), Some(&Address::Ask { project: OTHER_PROJECT }));
    let pages = story.probes[CLIENT.0].root.person().pages();
    assert_eq!(pages.address(), &Address::Ask { project: OTHER_PROJECT });
    let composer = pages.chats().expect("Ask's unsent composer");
    assert_eq!(composer.words.as_ref(), WORDS);
    assert!(composer.start.is_none());
    assert!(story.front.assignments.is_empty(), "AskWith starts no chat");
    assert_eq!(story.front.engine.store.rows, before, "only navigation and composer words change");
}

#[test]
fn a_second_entry_within_the_sitting_keeps_its_last_looked() {
    let mut story = Story::new();
    story.enter();
    let first = story.looked(fixtures::PARTY).expect("first look");
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(31 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert_eq!(*story.shown().previous(), Some(first.latest.as_nanos()));
    let before = story.facts();
    let sitting = story.looked(fixtures::PARTY).expect("new sitting");
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(29 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    let after = story.looked(fixtures::PARTY).expect("same sitting");
    assert_eq!(after.previous, sitting.previous, "29 minutes keeps the previous sitting's last look");
    assert_eq!(after.seen, sitting.seen);
    assert_eq!(after.latest, story.env.wall, "this sitting's last look advances");
    assert_eq!(story.shown().previous(), &Some(first.latest.as_nanos()));
    assert_eq!(story.facts().landed_since(), before.landed_since());
    assert_eq!(story.front.status_looks(CLIENT), 3, "one look per entry");
}

#[test]
fn coming_back_after_a_landing_says_what_landed() {
    let mut story = Story::with_landing();
    story.enter();
    let first = story.looked(fixtures::PARTY).expect("first sitting");
    assert_eq!(story.facts().landed_since(), 0);
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(60));
    story.land();
    assert_eq!(story.looked(fixtures::PARTY), Some(first.clone()), "landing while away does not look");
    story.after(Duration::from_secs(31 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    let document = story.shown();
    assert_eq!(document.previous(), &Some(first.latest.as_nanos()));
    assert_eq!((document.landed(), document.seen()), (1, 0));
    assert_eq!(story.facts().landed_since(), 1, "seen is the count at the previous sitting's last look");
    let second = story.looked(fixtures::PARTY).expect("second sitting");
    assert_eq!((second.seen, second.next_seen), (0, 1));
    assert_eq!(story.front.status_looks(CLIENT), 2);
}

#[test]
fn no_look_goes_while_to_begin_shows() {
    let mut story = Story::new();
    story.client(fixtures::PARTY, Address::Home { project: PROJECT });
    assert!(story.facts().to_begin());
    assert_eq!(story.facts().status, Status::Unknown);
    assert_eq!(story.looked(fixtures::PARTY), None);
    assert_eq!(story.front.status_looks(CLIENT), 0);
    story.after(Duration::from_secs(31 * 60));
    assert!(story.facts().to_begin(), "To begin remains until done or skipped");
    assert_eq!(story.facts().status, Status::Unknown);
    assert_eq!(story.looked(fixtures::PARTY), None);
    assert_eq!(story.front.status_looks(CLIENT), 0, "time passing sends no look");
}

#[test]
fn a_skip_sends_it() {
    let mut story = Story::new();
    story.client(fixtures::PARTY, Address::Home { project: PROJECT });
    assert!(story.facts().to_begin());
    assert_eq!(story.front.status_looks(CLIENT), 0);
    story.event(CLIENT, Event::Act { action: Action::Pages(Act::SkipToBegin) }.into());
    assert!(!story.facts().to_begin());
    assert_eq!(story.shown().previous(), &None);
    assert_eq!(story.facts().landed_since(), 0);
    let row = story.looked(fixtures::PARTY).expect("Skip sends the entry's look");
    assert_eq!(row.latest, story.env.wall);
    assert_eq!(story.front.status_looks(CLIENT), 1);
    story.pump();
    assert_eq!(story.front.status_looks(CLIENT), 1, "Skip sends this entry's look only once");
}

#[test]
fn a_look_reaches_another_device_of_the_party() {
    let mut story = Story::new();
    story.enter();
    let first = story.looked(fixtures::PARTY).expect("owner's first look");
    story.go(CLIENT, Address::Forges);
    // Observers need no project page or its standing needs watch.
    let observing = Address::Welcome { step: temper_client_domain::address::Step::Welcome };
    assert_eq!(story.client(fixtures::PARTY, observing.clone()), SECOND_DEVICE);
    story.watch(SECOND_DEVICE);
    story.front.bind_dismissal_person(OTHER_CLIENT, OTHER_PARTY, OTHER_SUBJECT);
    story.front.add_dismissal_member(CLIENT, OTHER_PARTY, PROJECT);
    assert_eq!(story.client(OTHER_PARTY, observing), OTHER_CLIENT);
    story.watch(OTHER_CLIENT);
    story.after(Duration::from_secs(60));
    story.look(OTHER_CLIENT, 2203);
    let other_first = story.looked(OTHER_PARTY).expect("member's first look");
    story.after(Duration::from_secs(31 * 60));
    story.look(OTHER_CLIENT, 2204);
    let other = story.looked(OTHER_PARTY).expect("member's second sitting");
    assert_eq!(story.watched(OTHER_CLIENT).previous(), &Some(other_first.latest.as_nanos()));
    assert_eq!(story.watched(SECOND_DEVICE).previous(), &None, "the owner's sitting has not moved");
    // The source needs no Home page's watches to publish its next entry look.
    story.look(CLIENT, 2205);
    assert_eq!(story.looked(fixtures::PARTY).expect("owner's second sitting").previous, Some(first.latest));
    assert_eq!(
        story.watched(SECOND_DEVICE).previous(),
        &Some(first.latest.as_nanos()),
        "the first device's look marks the second device's open watch"
    );
    assert_eq!(
        story.watched(OTHER_CLIENT).previous(),
        &Some(other_first.latest.as_nanos()),
        "another party's watch keeps that party's previous sitting"
    );
    assert_eq!(story.looked(OTHER_PARTY), Some(other), "the owner's look changes no other party's row");
    assert_eq!(story.front.status_looks(SECOND_DEVICE), 0, "the watching device sent no look");
}

#[test]
fn a_restart_keeps_the_rows() {
    let mut story = Story::with_landing();
    story.client(fixtures::PARTY, Address::Home { project: PROJECT });
    story.event(CLIENT, Event::Act { action: Action::Pages(Act::SkipToBegin) }.into());
    story.go(CLIENT, fixtures::inbox());
    story.land();
    story.after(Duration::from_secs(31 * 60));
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert_eq!(story.facts().landed_since(), 1);
    let document = story.shown();
    let looked = story.looked(fixtures::PARTY).expect("look before restart");
    let rows = story.rows();
    assert_eq!(rows.len(), 2, "one ProjectFacts and one Looked row");
    story.front.restart_dismissal_people(OFFLINE);
    assert_eq!(story.rows(), rows, "the committed rows survive the engine restart");
    story.front.reload(CLIENT, story.env.now);
    story.probes[CLIENT.0] = Probe {
        root: Root::Person(Box::new(temper_client_domain::unlinked(&story.env.limits, fixtures::SEED))),
        out: Queue::uncapped(),
        looks: Vec::new(),
    };
    story.event(CLIENT, Temper::start(fixtures::inbox(), Vec::new(), 0));
    story.after(OFFLINE);
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert!(!story.facts().to_begin(), "the Skip dismissal survives the engine and client restarts");
    let view = story.view();
    assert!(
        !Temper::tree(&view)
            .nodes()
            .iter()
            .any(|node| node.role == skein_ui::Role::Button && node.name.as_deref() == Some(b"Skip")),
        "a fresh Home has no Skip button"
    );
    let words = story.words();
    assert!(words.contains("One change landed since you last looked. Nothing needs you."), "{words}");
    assert!(words.contains("Landing: "), "{words}");
    assert!(!words.contains("nothing has landed yet"), "{words}");
    assert_eq!(story.shown(), document, "a fresh Home reads the restored rows");
    assert_eq!(story.facts().landed_since(), 1);
    let after = story.looked(fixtures::PARTY).expect("look after restart");
    assert_eq!(after.previous, looked.previous);
    assert_eq!((after.seen, after.next_seen), (looked.seen, looked.next_seen));
    assert_eq!(after.latest, story.env.wall);
}

#[test]
fn a_refused_status_run_leaves_home_saying_when_the_last_change_landed() {
    use temper_engine_domain_world::{store::Store, walking};
    let mut story = Story::new();
    let mut limits = story.front.engine.env.limits;
    limits.people.sign_in_lifetime = Duration::from_secs(7200);
    let mut period = 0;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(fixtures::SEED);
        let policy = &mut config.core.seed_projects[0].policy;
        policy.policy.period_spend = policy.policy.roles[0].period_spend;
        period = policy.policy.period_spend;
        config.core.defaults.rules.period_spend = period;
        policy.permissions = Box::new([]);
        policy.templates = Box::new([]);
        config.core.defaults.policies.insert(1, policy.clone()).expect("policy 1");
        config.core.connectors = Box::new([fixtures::FORGE]);
        config
    });
    let bootstrap = temper_engine_domain_world::oracle::config::copy(&configs[0]);
    let mut retained =
        temper_engine_domain_world::fixtures::restored_over(Store::with_limits(&limits), bootstrap, &limits);
    retained.seed(
        Box::new([Record::Forge {
            row: Box::new(forge::Stored::ProjectFacts(forge::ProjectFacts {
                project: PROJECT,
                landed: 1,
                last_landed: Wall::EPOCH,
                brief: None,
                running: None,
            })),
        }]),
        Box::new([]),
    );
    story.front = Joint::with_engine(fixtures::SEED, &story.env.limits, retained, configs, &limits, 1);
    story.front.engine.send(temper_engine_domain::Event::Party(temper_engine_domain::PartyInput::Period {
        project: PROJECT,
        period: 2,
        budget: period,
        next: None,
    }));
    story.front.engine.settle();
    story.enter();
    assert_eq!((story.shown().landed(), story.shown().seen()), (1, 1));
    story.go(CLIENT, fixtures::inbox());
    story.after(Duration::from_secs(31 * 60));
    let before = story.front.engine.transactions.len();
    story.go(CLIENT, Address::Home { project: PROJECT });
    assert!(
        story.front.engine.transactions[before..]
            .iter()
            .flatten()
            .any(|write| matches!(write, temper_engine_domain::Write::Save(Record::Forge { row })
            if matches!(row.as_ref(), forge::Stored::ProjectFacts(facts) if facts.running.is_none()))),
        "the new sitting attempted a status run and saved its refusal"
    );
    assert!(story.shown().previous().is_some(), "return advances the sitting");
    assert_eq!((story.shown().landed(), story.shown().seen()), (1, 1));
    assert_eq!(story.shown().brief(), &None);
    let record = story
        .front
        .engine
        .store
        .rows
        .get(&StoreKey::Forge(forge::Key::ProjectFacts { project: PROJECT }))
        .expect("facts");
    let Record::Forge { row } = record else { panic!("forge record") };
    let forge::Stored::ProjectFacts(facts) = row.as_ref() else { panic!("facts record") };
    assert_eq!(facts.running, None, "the unfunded run was refused");
    let (_, mut view) = Temper::new(fixtures::SEED);
    let temper_client_world::app::View::Person(person) = &mut view else { panic!("person view") };
    temper_client_view::build(person, story.probes[CLIENT.0].root.person(), story.env.wall, &mut Queue::uncapped());
    let tree = Temper::tree(&view);
    assert!(tree.nodes().iter().any(|node| node.role == skein_ui::Role::Heading { level: 1 }
        && node.name.as_deref() == Some(b"The last change landed 31 minutes ago. Nothing needs you.")));
    let text: Vec<u8> =
        tree.nodes().iter().flat_map(|node| node.text.iter()).flat_map(|run| run.text.iter().copied()).collect();
    let text = String::from_utf8(text).expect("Home words");
    for forbidden in ["Written by the status agent", "brief", "refused", "failed", "unfunded"] {
        assert!(!text.contains(forbidden), "Home names no status-run failure: {text}");
    }
}
