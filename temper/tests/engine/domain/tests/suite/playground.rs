//! The playground on the real root, connector, core and typed store
//! (temper/docs/design/draft/playground-reset.md, 6.1, 6.2, 8 and 9).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Key, Record, Write};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{
    cuts, direct::Driver, fixtures, forge_stand::ForgeStand, forges, protocol, store::Store, walking,
};

const SEEDS: [u64; 2] = [71, 9762];
const START: u8 = 81;
const END: u8 = 90;

fn limits() -> engine::Limits {
    let mut limits = fixtures::forge_limits();
    // Deployment, playground and a second project; retained start/adoption/reset answers.
    limits.people.projects = 3;
    limits.authority.projects = 2;
    limits.people.act_grants = 1;
    limits.people.pending = 4;
    limits.people.requests = 16;
    limits.journal.deliveries += 4;
    limits.journal.held += 12;
    limits
}

fn driver(seed: u64, real: bool) -> Driver {
    let limits = limits();
    let mut store = fixtures::first_start_over(Store::with_limits(&limits), walking::playground_config(seed), &limits);
    if real {
        store.seed(forges::rows(&[forges::record(1, forge::Kind::Forgejo, forge::Mode::Managed)]), Box::new([]));
    }
    let mut d = Driver::cold(store, walking::playground_config(seed), &limits);
    d.period_sent = true;
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]));
    d.settle();
    d.sign_in();
    d.settle();
    d.delivered.clear();
    d
}

fn application(d: &mut Driver, ask: forge::Ask, number: u8, carry: bool) {
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in: session(d),
        key: [number; 16],
        digest: [number; 32],
        scope: people::ApplicationScope::Deployment,
        request: engine::Application::Forge(ask),
        carried: carry.then_some(core::Carried { number: u64::from(number), digest: [7; 32] }),
    }));
}

fn session(d: &Driver) -> u64 {
    d.store
        .rows
        .values()
        .find_map(|row| {
            if let Record::Core(core::Record::People(people::Stored::SignIn { number, .. })) = row {
                Some(*number)
            } else {
                None
            }
        })
        .expect("durable sign-in")
}

fn answer(d: &Driver, number: u8) -> Vec<Option<engine::ApplicationAnswer>> {
    d.delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::Application { to, outcome, .. } = delivery
                && *to == ReplyTo::new(Token::new(u64::from(number)))
            {
                Some(outcome.clone())
            } else {
                None
            }
        })
        .collect()
}

fn playground(d: &Driver) -> forge::Playground {
    let Some(Record::Forge { row }) = d.store.rows.get(&Key::Forge(forge::Key::Playground)) else {
        panic!("durable playground row")
    };
    let forge::Stored::Playground(row) = row.as_ref() else { panic!("playground record") };
    *row
}

fn running(seed: u64) -> Driver {
    let mut d = driver(seed, false);
    application(&mut d, forge::Ask::StartPlayground, START, true);
    d.pump();
    assert!(answer(&d, START).is_empty(), "start waits for the fake forge's connection");
    let row = playground(&d);
    assert!(
        d.immediate
            .iter()
            .any(|request| matches!(request, engine::Request::Register(record) if record.number == row.forge))
    );
    assert!(
        d.immediate
            .iter()
            .any(|request| matches!(request, engine::Request::SetUpGroup { forge, .. } if *forge == row.forge))
    );
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: row.forge,
        state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    d.settle();
    assert_eq!(answer(&d, START), [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running))]);
    // Serve publishes the connected forge's application credential to the core.
    d.send(engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) });
    d.settle();
    d
}

#[test]
fn a_start_makes_the_fake_forge_project_account_and_adoption() {
    for seed in SEEDS {
        let mut d = running(seed);
        let row = playground(&d);
        assert_eq!(row.state, forge::PlaygroundState::Running);
        let project = row.project.expect("playground project");
        let account = row.account.expect("scripted account");
        assert!(cuts::forge_rows(&d.store).iter().any(|record| matches!(record,
            forge::Stored::Forge(record) if record.number == row.forge && record.playground && record.mode == forge::Mode::Fake)));
        assert!(d.store.rows.values().any(|record| matches!(record,
            Record::Core(core::Record::People(people::Stored::Policy { project: number, name, .. }))
                if *number == project && name.as_ref() == "Playground")));
        assert!(cuts::acted(&d.store).iter().any(|(_, act, outcome)| matches!((act, outcome),
            (people::Act::MakeProject { name, policy: 1, .. }, people::Outcome::ProjectMade { project: made })
                if *made == project && name.as_ref() == "Playground")));
        assert!(d.store.rows.values().any(|record| matches!(record,
            Record::Core(core::Record::Accounts(accounts::Stored::Account {
                number, provider: 1, holder: accounts::Holder::Deployment, uses: Some(uses), ..
            })) if *number == account && uses.as_ref() == [accounts::Use::Party(1), accounts::Use::Project(project)])));
        assert!(cuts::forge_rows(&d.store).iter().any(|record| matches!(record,
            forge::Stored::Repository(record) if record.project == project && record.provider.forge == row.forge
                && record.owner.as_ref() == b"playground" && record.name.as_ref() == b"widget")));
        let kept = d.protocol.accounts.len();
        let calls = d.forge.as_ref().expect("fake forge").calls;
        let acts = cuts::acted(&d.store);
        let header = d.store.header();
        application(&mut d, forge::Ask::StartPlayground, START + 1, true);
        d.settle();
        assert_eq!(
            answer(&d, START + 1),
            [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running))]
        );
        assert_eq!(playground(&d), row);
        assert_eq!(cuts::acted(&d.store), acts);
        assert_eq!(d.store.header().projects, header.projects);
        assert_eq!(d.protocol.accounts.len(), kept);
        assert_eq!(d.forge.as_ref().expect("fake forge").calls, calls);
        assert!(
            d.delivered.iter().any(
                |delivery| matches!(delivery, Delivery::CarriedUnused { number } if *number == u64::from(START + 1))
            )
        );
        another_project_cannot_select(&mut d, account);
    }
}

fn ask(d: &mut Driver, sign_in: u64, key: u8, ask: people::Ask) {
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in,
        key: [key; 16],
        ask,
    }));
    d.pump();
}

fn other_project(d: &mut Driver, owner: u64) -> u32 {
    ask(d, owner, 83, people::Ask::MakeProject { name: "Other project".into(), policy: 1 });
    let project = d
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::ProjectMade { project }), ..
            } = delivery
            {
                Some(*project)
            } else {
                None
            }
        })
        .expect("other project made");
    assert_ne!(Some(project), playground(d).project);
    project
}

fn another_project_cannot_select(d: &mut Driver, account: u32) {
    let owner = session(d);
    let project = other_project(d, owner);
    d.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(84)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: 8_u64.to_be_bytes().into() },
            login: b"other".as_slice().into(),
            name: b"Other owner".as_slice().into(),
        },
    }));
    d.pump();
    let (person, sign_in) = d
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply { to, reply: people::Reply::SignedIn { person, sign_in, .. } } = delivery
                && *to == ReplyTo::new(Token::new(84))
            {
                Some((*person, *sign_in))
            } else {
                None
            }
        })
        .expect("other project's owner signed in");
    ask(
        d,
        owner,
        86,
        people::Ask::SetRoles { project, holdings: Box::new([people::Holding { person, role: people::Role::Owner }]) },
    );
    d.send(engine::Event::Party(engine::PartyInput::Period { project, period: 1, budget: 1000, next: None }));
    d.pump();
    let people::Ask::Create { mut batch, .. } = walking::chat(1, b"Work in the other project") else {
        panic!("chat fixture")
    };
    batch[0].llm = Some(people::LlmChoice {
        account: Some(account),
        main: people::ModelEffort { model: 0, effort: None },
        subagents: Box::new([]),
    });
    ask(d, sign_in, 87, people::Ask::Create { project, batch });
    assert!(
        d.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { to, reply: people::Reply::Outcome(people::Outcome::Refused(
            people::Refusal::Account { account: refused, why: people::AccountWhy::Uses })) }
            if *to == ReplyTo::new(Token::new(87)) && *refused == account)),
        "another project's task cannot select the scripted account: {:?}",
        d.delivered
    );
    assert!(!d.delivered.iter().any(|delivery| matches!(delivery, Delivery::Assigned { .. })));
}

#[test]
fn a_start_beside_a_real_forge_or_while_resetting_is_not_offered() {
    for seed in SEEDS {
        let mut d = driver(seed, true);
        application(&mut d, forge::Ask::StartPlayground, START, true);
        d.settle();
        assert_eq!(
            answer(&d, START),
            [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::NotOffered))]
        );
        assert!(d.root.playground().is_none());
        assert!(cuts::acted(&d.store).is_empty());
        let mut d = running(seed);
        application(&mut d, forge::Ask::EndPlayground, END, false);
        d.pump();
        assert!(matches!(playground(&d).state, forge::PlaygroundState::Resetting(_)));
        application(&mut d, forge::Ask::StartPlayground, START + 1, true);
        d.pump();
        assert_eq!(
            answer(&d, START + 1),
            [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::NotOffered))]
        );
    }
}

fn serve_answer(d: &mut Driver, act: forge::KeptActKind) {
    let forge = playground(d).forge;
    let event = match act {
        forge::KeptActKind::RemoveGroup => forge::Event::GroupRemoved { forge },
        forge::KeptActKind::EraseSecrets => forge::Event::SecretsErased { forge },
        forge::KeptActKind::Cancel { .. } => panic!("the core answers cancellation"),
    };
    d.send(engine::Event::Forge(Box::new(event)));
}

struct Snapshot {
    store: Store,
    writes: Vec<Write>,
}

// Observe every durable decision before its terminal reaches the old process.
fn pump_recording(d: &mut Driver, snapshots: &mut Vec<Snapshot>) {
    for _ in 0..200 {
        d.advance(false);
        if let Some((_, writes)) = d.store.pending.front() {
            let writes = writes.to_vec();
            let number = d.store.apply();
            if writes.contains(&Write::Erase(Key::Forge(forge::Key::Playground))) {
                assert!(answer(d, END).is_empty(), "Reset cannot leave before the erasure's commit terminal");
                assert!(!d.store.rows.contains_key(&Key::Forge(forge::Key::Playground)));
            }
            snapshots.push(Snapshot { store: d.store.fork(), writes });
            d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        }
    }
    assert!(d.events.is_empty() && d.store.pending.is_empty(), "reset's immediate work drained");
}

fn finish(d: &mut Driver, snapshots: &mut Vec<Snapshot>) {
    pump_recording(d, snapshots);
    for act in [forge::KeptActKind::RemoveGroup, forge::KeptActKind::EraseSecrets] {
        if cuts::kept_acts(&d.store).iter().any(|row| row.act == act) {
            serve_answer(d, act);
            pump_recording(d, snapshots);
        }
    }
    d.settle();
}

fn assert_empty(d: &Driver) {
    assert!(d.root.playground().is_none());
    assert!(d.root.forges().rows.is_empty());
    // Nothing in this deployment names a surviving task. Even repository names have no keeper.
    for (key, record) in &d.store.rows {
        let kept = match record {
            Record::Forge { row } => matches!(row.as_ref(), forge::Stored::ForgeNumbers { .. }),
            Record::Core(core::Record::Core(row)) => {
                matches!(row, core::CoreRecord::Deployment(_) | core::CoreRecord::Settings(_))
            }
            Record::Core(core::Record::People(row)) => matches!(
                row,
                people::Stored::Acted { .. }
                    | people::Stored::Answer { .. }
                    | people::Stored::Person { .. }
                    | people::Stored::SignIn { .. }
                    | people::Stored::ReadResults { .. }
                    | people::Stored::Roles { project: people::DEPLOYMENT, .. }
            ),
            Record::Core(core::Record::Accounts(row)) => {
                matches!(row, accounts::Stored::Next { .. } | accounts::Stored::Issued { .. })
            }
            Record::Core(core::Record::Tasks(row)) => match row {
                tasks::Stored::ClosedLedger(ledger) => ledger.closed,
                tasks::Stored::Ledger(ledger) => matches!(
                    ledger.funder,
                    tasks::Funder::Period { project: people::DEPLOYMENT, .. }
                        | tasks::Funder::Pool { project: people::DEPLOYMENT, .. }
                ),
                tasks::Stored::PlacedHold(_)
                | tasks::Stored::Update(_)
                | tasks::Stored::Result(_)
                | tasks::Stored::EndedByParty(_)
                | tasks::Stored::EndedByProject(_)
                | tasks::Stored::EndedDelegate(_)
                | tasks::Stored::Delegate(_)
                | tasks::Stored::Activity(_)
                | tasks::Stored::Milestone(_)
                | tasks::Stored::PersonProposal(_)
                | tasks::Stored::DecidedProposal(_)
                | tasks::Stored::History(_)
                | tasks::Stored::Live(_)
                | tasks::Stored::Ended(_)
                | tasks::Stored::Stub(_)
                | tasks::Stored::Writer(_)
                | tasks::Stored::Pool(_) => false,
            },
            Record::Core(core::Record::Notes(_)) => false,
        };
        assert!(kept, "row outside the keep list: {key:?}: {record:?}");
    }
}

fn live_chat(d: &mut Driver) -> engine::Assignment {
    let project = playground(d).project.expect("playground project");
    d.send(engine::Event::Party(engine::PartyInput::Period { project, period: 1, budget: 1000, next: None }));
    d.hello(Token::new(7));
    d.settle();
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(85)),
        sign_in: session(d),
        key: [85; 16],
        ask: walking::chat(project, b"Try the playground"),
    }));
    d.settle();
    let chat = d.last_assignment();
    assert_eq!(chat.run.accounts[0].account, playground(d).account.expect("scripted account"));
    assert_eq!(d.task(chat.task).expect("live chat").project, project);
    let read = chat.messages.last().map(|word| word.number);
    d.send(protocol::turn(Token::new(7), chat.task, chat.attempt, 1, 3, read, b"Working in the playground"));
    d.settle();
    chat
}

fn park(d: &mut Driver, chat: &engine::Assignment) {
    let mut event =
        d.answer_event(Token::new(7), chat.task, chat.attempt, 3, tasks::End::Parked, engine::Parts::none());
    let engine::Event::Worker(engine::WorkerInput::Link {
        link: engine::Link::Up { up: jig_domain_host::LinkUp::Answer { answer, .. }, .. },
        ..
    }) = &mut event
    else {
        panic!("worker answer")
    };
    answer.read = chat.messages.last().map(|word| Token::new(word.number));
    answer.turns = 1;
    d.send(event);
    d.pump();
}

#[test]
fn the_reset_deletes_the_project_removes_the_forge_and_the_account_then_the_row() {
    for seed in SEEDS {
        let mut d = running(seed);
        let row = playground(&d);
        let chat = live_chat(&mut d);
        application(&mut d, forge::Ask::EndPlayground, END, false);
        d.pump();
        assert!(answer(&d, END).is_empty(), "Reset waits for its row's erasure");
        let grace = d
            .task(chat.task)
            .expect("chat in its deletion grace")
            .grace
            .as_ref()
            .expect("project deletion cancels the chat");
        assert_eq!(grace.by, tasks::Party::Deployment { project: row.project.expect("project") });
        assert_eq!(grace.reason.as_ref(), b"Project deleted");
        d.fire_at(Duration::from_secs(5));
        d.pump();
        park(&mut d, &chat);
        let mut snapshots = Vec::new();
        finish(&mut d, &mut snapshots);
        assert_eq!(answer(&d, END), [Some(engine::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset))]);
        assert_empty(&d);
        assert!(
            d.store.rows.values().any(|record| matches!(record,
            Record::Core(core::Record::Tasks(tasks::Stored::ClosedLedger(ledger)))
                if matches!(ledger.funder, tasks::Funder::Pool { project, person: 1, .. }
                    if Some(project) == row.project) && ledger.numbers.spent_below == 3)),
            "the person's settled spend survives deletion"
        );
        assert!(
            d.transactions
                .iter()
                .flatten()
                .any(|write| matches!(write, Write::DropTranscript { task } if *task == chat.task))
        );
        assert!(cuts::acted(&d.store).iter().any(|(_, act, outcome)| matches!((act, outcome),
            (people::Act::RemoveProject { project }, people::Outcome::ProjectRemoved { project: removed })
                if Some(*project) == row.project && project == removed)));
        assert!(cuts::acted(&d.store).iter().any(|(_, act, outcome)| matches!((act, outcome),
            (people::Act::RemoveAccount { account }, people::Outcome::AccountRemoved { .. })
                if Some(*account) == row.account)));
        nothing_is_read(&mut d, row.project.expect("project"), &chat);
    }
}

fn nothing_is_read(d: &mut Driver, project: u32, chat: &engine::Assignment) {
    for scope in [
        core::reads::Scope::ProjectRows { project },
        core::reads::Scope::TaskRows { task: chat.task },
        core::reads::Scope::Notes { project },
    ] {
        let result = d.store.read(Box::new(core::reads::Range {
            party: 0,
            scope,
            after: None,
            backwards: false,
            rows: 16,
            bytes: u64::MAX,
        }));
        assert!(
            matches!(result, core::reads::Loaded::Page { rows, next: None } if rows.is_empty()),
            "the real typed store has no task, project or note rows"
        );
    }
    let turn = d.store.read(Box::new(core::reads::Range {
        party: 1,
        scope: core::reads::Scope::Turn { task: chat.task, attempt: chat.attempt, turn: 1 },
        after: None,
        backwards: false,
        rows: 1,
        bytes: u64::MAX,
    }));
    let absent = match &turn {
        core::reads::Loaded::Gone => true,
        core::reads::Loaded::Page { rows, next } => rows.is_empty() && next.is_none(),
        core::reads::Loaded::Busy | core::reads::Loaded::TooLinked { .. } | core::reads::Loaded::TurnProblem { .. } => {
            false
        }
    };
    assert!(absent, "the erased transcript cannot be read: {turn:?}");
    for (number, read) in
        [(120, core::reads::Read::Project { project }), (121, core::reads::Read::Task { task: chat.task })]
    {
        d.send(engine::Event::Party(engine::PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(number)),
            sign_in: session(d),
            read,
        }));
        d.settle();
        assert!(d.delivered.iter().any(|delivery| matches!(delivery,
            Delivery::Read { reply: to, answer: core::reads::Answer::None(core::reads::Why::Unknown) }
                if *to == ReplyTo::new(Token::new(number)))));
    }
}

fn comparable(d: &Driver) -> std::collections::BTreeMap<Key, Record> {
    let mut rows = d.store.rows.clone();
    // End's answer is kept in memory (T6). A crashed caller has no completion;
    // only the uncut caller saves the core's application-answer metadata.
    rows.remove(&Key::Core(core::Key::People(people::Key::Answer(people::RequestKey { person: 1, key: [END; 16] }))));
    let Some(Record::Core(core::Record::Core(core::CoreRecord::Deployment(header)))) =
        rows.get_mut(&Key::Core(core::Key::Core(core::CoreKey::Deployment)))
    else {
        panic!("deployment header")
    };
    // A cold start adds its journal start mark and may replay a retained answer.
    header.commits = 0;
    rows
}

#[test]
fn a_reset_cut_after_each_step_and_act_resumes_to_the_same_end() {
    for seed in SEEDS {
        let mut uncut = running(seed);
        application(&mut uncut, forge::Ask::EndPlayground, END, false);
        let mut snapshots = Vec::new();
        finish(&mut uncut, &mut snapshots);
        assert_empty(&uncut);
        assert_eq!(answer(&uncut, END), [Some(engine::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset))]);
        let expected = comparable(&uncut);
        // Every reset step and each forge kept act has a durable cut in this walk.
        for step in [forge::ResetStep::Project, forge::ResetStep::Forge, forge::ResetStep::Account] {
            assert!(snapshots.iter().any(|snapshot| snapshot.writes.iter().any(|write| matches!(write,
                Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Playground(row)
                    if row.state == forge::PlaygroundState::Resetting(step))))));
        }
        for act in cuts::saved_acts(&uncut) {
            assert!(
                snapshots
                    .iter()
                    .any(|snapshot| snapshot.writes.contains(&Write::Erase(Key::Forge(forge::Key::KeptAct(act.key)))))
            );
        }
        assert_eq!(cuts::saved_acts(&uncut).len(), 2, "RemoveGroup and EraseSecrets");
        for project_act in [true, false] {
            assert!(
                snapshots.iter().any(|snapshot| snapshot.writes.iter().any(|write| matches!(write,
                Write::Save(Record::Core(core::Record::People(people::Stored::Acted { act, .. })))
                    if if project_act { matches!(act, people::Act::RemoveProject { .. }) }
                       else { matches!(act, people::Act::RemoveAccount { .. }) }))),
                "cut after each core act's durable answer"
            );
        }
        for (cut, snapshot) in snapshots.into_iter().enumerate() {
            let kept = cuts::kept_acts(&snapshot.store);
            let mut resumed =
                cuts::cold(snapshot.store, walking::playground_config(seed), &limits(), seed, uncut.env.wall);
            for act in kept {
                assert!(cuts::kept_acts(&resumed.store).contains(&act), "resend keeps the act's identity");
            }
            finish(&mut resumed, &mut Vec::new());
            assert_empty(&resumed);
            assert_eq!(comparable(&resumed), expected, "seed {seed}, cut {cut}: {:?}", snapshot.writes);
            eprintln!("seed {seed}: resumed durable reset cut {cut}");
        }
    }
}

#[test]
fn a_start_that_finds_the_playground_running_resets_it() {
    for seed in SEEDS {
        let d = running(seed);
        let mut resumed = cuts::cold(d.store, walking::playground_config(seed), &limits(), seed, d.env.wall);
        assert!(resumed.transactions.iter().flatten().any(|write| matches!(write,
            Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Playground(row)
                if row.state == forge::PlaygroundState::Resetting(forge::ResetStep::Project)))));
        finish(&mut resumed, &mut Vec::new());
        assert_empty(&resumed);
    }
}

#[test]
fn the_starting_person_cannot_select_the_scripted_account_in_another_project() {
    let mut leaks = Vec::new();
    for seed in SEEDS {
        let mut d = running(seed);
        let account = playground(&d).account.expect("scripted account");
        let owner = session(&d);
        let project = other_project(&mut d, owner);
        d.send(engine::Event::Party(engine::PartyInput::Period { project, period: 1, budget: 1000, next: None }));
        d.hello(Token::new(7));
        d.pump();
        let people::Ask::Create { mut batch, .. } = walking::chat(1, b"Work outside the playground") else {
            panic!("chat fixture")
        };
        batch[0].llm = Some(people::LlmChoice {
            account: Some(account),
            main: people::ModelEffort { model: 0, effort: None },
            subagents: Box::new([]),
        });
        ask(&mut d, owner, 87, people::Ask::Create { project, batch });
        let refused = d.delivered.iter().any(|delivery| {
            matches!(delivery,
            Delivery::WebReply { to, reply: people::Reply::Outcome(people::Outcome::Refused(
                people::Refusal::Account { account: refused, why: people::AccountWhy::Uses })) }
                if *to == ReplyTo::new(Token::new(87)) && *refused == account)
        });
        if !refused {
            let selected: Vec<_> = d
                .delivered
                .iter()
                .filter_map(|delivery| {
                    if let Delivery::Assigned { assignment, .. } = delivery {
                        Some((assignment.task, assignment.run.accounts.to_vec()))
                    } else {
                        None
                    }
                })
                .collect();
            leaks.push((seed, project, selected));
        }
    }
    assert!(leaks.is_empty(), "another project's task selected the playground account: {leaks:?}");
}
