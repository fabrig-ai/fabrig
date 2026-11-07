//! A provider's list meets accounts, settings and the commit barrier (domain/engine.md, 17.12).
use crate::settings_support::*;
use jig_core_world::{
    Fault,
    client_peer::ClientPeer,
    lists::{listed, set_list},
    peers,
    world::{Boot, Cut, Setup, World},
};
use jig_domain_accounts as accounts;
use jig_domain_core::{self as core, documents, reads, settings as s};
use jig_domain_fleet as fleet;
use jig_domain_link::{self as link, Hello};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token, Wall};

const PROVIDER: u32 = 1;
const CHANNEL: Token = Token::new(7);
const ACCOUNT: u32 = 1;
const SOURCE: u32 = 0;
const NEW: u32 = jig_domain_tasks::MODELS; // from_list allocates the highest free setting number.
const DAY: Duration = Duration::from_secs(86_400);
const ME: Token = Token::new(45);
const ACCOUNTS: Token = Token::new(46);

fn configured() -> (root::Config, root::Limits) {
    let (mut config, mut limits) = configuration();
    set_list(&mut config, PROVIDER, &[b"*"], &[]);
    config.core.settings.endpoints[0].providers = Box::new([PROVIDER]);
    config.core.workers_in_process = false;
    config.core.defaults.charters[0].rest = true;
    limits.core.settings.alternatives = 7;
    peers::journal(&mut limits);
    (config, limits)
}
fn start(config: root::Config, limits: &root::Limits) -> (World, u64) {
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits: *limits,
        rows: Some(Box::new([])),
        boot: Boot::Empty,
        ..Setup::default()
    });
    assert!(world.domain.ready(), "{:?}", world.domain.start_failure());
    world.client_peer = Some(ClientPeer::default());
    let signed = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::Period { project: 1, period: 1, budget: 100, next: None }));
    (world, signed)
}
fn setup() -> (World, u64) {
    let (config, limits) = configured();
    let (mut world, signed) = start(config, &limits);
    host(&mut world);
    add_key(&mut world, signed);
    (world, signed)
}
fn host(world: &mut World) {
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Hello {
        worker: 0,
        channel: CHANNEL,
        hello: Hello {
            jobs: Box::new([]),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(1),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
}
fn add_key(world: &mut World, signed: u64) {
    world.send(root::Event::Core(core::Event::Key {
        reply_to: ReplyTo::new(Token::new(SEED)),
        sign_in: signed,
        key: [20; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::AddKey { provider: PROVIDER, join: None, digest: [7; 32] },
        }),
        carried: core::Carried { number: 1_u64 << 63, digest: [7; 32] },
    }));
}
fn again(world: &mut World) {
    world.send(root::Event::Core(core::Event::Account(accounts::Event::Change {
        request: Token::new(90),
        holder: accounts::Holder::Party(OWNER),
        change: accounts::Change::ReadLists { provider: PROVIDER },
    })));
}
fn read(world: &mut World, names: &[&[u8]]) {
    again(world);
    world.answer_list(listed(names));
}
fn record(world: &World) -> accounts::ListRecord {
    *world.domain.accounts().list(ACCOUNT).expect("list record")
}
fn entry(world: &World, number: u32) -> s::CatalogueModel {
    let Some(s::Value::Model(value)) = world.domain.settings().value(s::Setting::Model(number)) else {
        panic!("model")
    };
    value.as_ref().clone()
}
fn make(world: &mut World, signed: u64, key: u8, model: Option<u8>) -> u64 {
    let mut member = jig_test_profile::fixtures::people_member(b"catalogue task", 20);
    member.llm = model.map(|model| people::LlmChoice {
        account: Some(ACCOUNT),
        main: people::ModelEffort { model, effort: None },
        subagents: Box::new([]),
    });
    let reply = ask(world, signed, key, people::Ask::Create { project: 1, batch: Box::new([member]) });
    let people::Reply::Outcome(people::Outcome::Created { first, .. }) = reply else { panic!("{reply:?}") };
    first
}
fn take(world: &mut World) -> Vec<(u64, root::Delivery)> {
    std::mem::take(&mut world.client_peer.as_mut().expect("list story fixture").delivered)
}
fn watch(world: &mut World, signed: u64) {
    for (watcher, subject) in [(ME, documents::Watched::Frame), (ACCOUNTS, documents::Watched::Accounts)] {
        world.send(root::Event::Core(core::Event::Watch {
            watcher,
            sign_in: signed,
            subject,
            window: documents::Window { rows: 8, after: None },
        }));
        take(world);
        world.send(root::Event::Core(core::Event::Delivered { watcher }));
    }
}

#[test]
fn a_new_name_and_its_account_commit_as_one_decision() {
    let (mut world, signed) = setup();
    watch(&mut world, signed);
    let placed = world.list_reads.last().expect("key places a read");
    assert_eq!((placed.channel, placed.endpoint, placed.grant.account), (CHANNEL, 1, SOURCE));
    world.store.fault(Fault::Hold { commits: 1 });
    world.answer_list(listed(&[b"scripted", b"new-model"]));
    assert!(
        world.client_peer.as_ref().expect("list story fixture").delivered.is_empty(),
        "watches wait for commit acknowledgement"
    );
    world.release_commits();
    let revision = revision(&world, 2, false);
    assert_eq!(revision.head.by, s::Decider::List { provider: PROVIDER });
    assert!(!revision.head.moved);
    assert_eq!(revision.head.proposal, None);
    let value = entry(&world, NEW);
    assert_eq!(value.source, s::ModelSource::List);
    assert!(!value.picker);
    assert_eq!(value.first_read, record(&world).read);
    let writes = world
        .commits
        .iter()
        .find(|writes| {
            writes.iter().any(|write| {
                matches!(
                    write,
                    root::Write::Save(root::Record::Core(core::Record::Accounts(accounts::Stored::Account {
                        list: Some(_),
                        ..
                    })))
                )
            })
        })
        .expect("account commit");
    assert!(writes.iter().any(|write| matches!(write, root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Settings(s::Record::Revision { revision, .. })))) if revision.head.number == 2)), "account and revision share a commit");
    assert_eq!(record(&world).names, 1 | (1_u64 << 63));
}

#[test]
fn list_answers_mark_only_the_documents_they_change() {
    enum Answer {
        Names(&'static [&'static [u8]]),
        Failure,
        Alarm,
    }
    for (label, good, answer, me, accounts, reads, newest) in [
        ("settings", true, Answer::Names(&[b"scripted", b"new-model"]), 1, 1, 1, 2),
        ("names narrow", true, Answer::Names(&[]), 1, 1, 1, 1),
        ("first empty list", false, Answer::Names(&[]), 1, 1, 0, 1),
        ("repeat", true, Answer::Names(&[b"scripted"]), 0, 1, 1, 1),
        ("first failure", false, Answer::Failure, 0, 1, 0, 1),
        ("failure", true, Answer::Failure, 0, 1, 1, 1),
        ("day", true, Answer::Alarm, 0, 0, 1, 1),
    ] {
        let (mut world, signed) = setup();
        if good {
            world.answer_list(listed(&[b"scripted"]));
        }
        watch(&mut world, signed);
        let placed = world.list_reads.len();
        let answered = match answer {
            Answer::Names(names) => {
                read(&mut world, names);
                true
            }
            Answer::Failure => {
                again(&mut world);
                world.answer_list(link::ListOutcome::Failed(link::ListFailure::Deadline));
                true
            }
            Answer::Alarm => {
                let (now, wall) = world.clocks();
                world.set_clock(now.saturating_add(DAY), Wall::from_nanos(wall.as_nanos() + DAY.as_nanos()));
                world.send(root::Event::Timer(core::Timer::Account));
                false
            }
        };
        if answered {
            let saved = world
                .store
                .rows
                .get(&root::Key::Core(core::Key::Accounts(accounts::Key::Account { number: ACCOUNT })))
                .expect("saved account");
            assert!(
                matches!(saved, root::Record::Core(core::Record::Accounts(accounts::Stored::Account { list: Some(list), .. })) if *list == record(&world)),
                "{label}: answer saves its record"
            );
        }
        let deliveries = take(&mut world);
        let count = |watcher| {
            deliveries.iter().filter(|(_, delivery)| matches!(delivery, root::Delivery::Core(core::Held::Delivery { watcher: got, .. }) if *got == watcher)).count()
        };
        assert_eq!((count(ME), count(ACCOUNTS)), (me, accounts), "{label}");
        assert_eq!(world.list_reads.len() - placed, reads, "{label}");
        assert_eq!(world.domain.settings().newest(), newest, "{label}");
    }
}

#[test]
fn every_list_failure_saves_only_the_attempt() {
    for (failure, expected) in [
        (link::ListFailure::Refused { status: 403 }, accounts::ReadFailure::Refused { status: 403 }),
        (link::ListFailure::Unreachable, accounts::ReadFailure::Unreachable),
        (link::ListFailure::Unreadable, accounts::ReadFailure::Unreadable),
        (link::ListFailure::Deadline, accounts::ReadFailure::Deadline),
        (link::ListFailure::Agent, accounts::ReadFailure::Agent),
    ] {
        let (mut world, _) = setup();
        world.answer_list(listed(&[b"scripted"]));
        let good = record(&world);
        let (now, wall) = world.clocks();
        world.set_clock(
            now.saturating_add(Duration::from_secs(1)),
            Wall::from_nanos(wall.as_nanos() + Duration::from_secs(1).as_nanos()),
        );
        again(&mut world);
        world.answer_list(link::ListOutcome::Failed(failure));
        assert_eq!(
            record(&world),
            accounts::ListRecord { tried: Some(world.env().wall), failure: Some(expected), ..good }
        );
        assert_eq!(world.domain.settings().newest(), 1);
    }
}

#[test]
fn a_list_revision_keeps_running_work_on_its_resolution() {
    let (mut world, signed) = setup();
    let task = make(&mut world, signed, 80, None);
    let held = world.domain.core_mut().charter_of(task).clone();
    world.answer_list(listed(&[b"scripted", b"new-model"]));
    assert_eq!(world.domain.core_mut().charter_of(task), &held);
    assert_eq!(jig_core_world::fixtures::live_row(&world, task).settings.task, 1);
    let new = make(&mut world, signed, 81, Some(63));
    let next = world.domain.core_mut().charter_of(new);
    assert!(next.policy.alternatives.iter().any(|model| model.name.as_ref() == b"new-model"));
    assert_eq!(world.assigned.as_ref().expect("list story fixture").run.policy.model.name.as_ref(), b"new-model");
    assert_eq!(jig_core_world::fixtures::live_row(&world, new).settings.task, 2);
    assert!(world.cancellations.is_empty(), "Stays takes no moves");
}

#[test]
fn a_full_revision_budget_saves_room_without_a_revision() {
    for (revisions, versions, refused) in [(1, 2, true), (2, 1, true), (2, 2, false)] {
        let (config, mut limits) = configured();
        limits.core.settings.revisions = revisions;
        limits.core.settings.versions = versions;
        peers::journal(&mut limits);
        let (mut world, signed) = start(config, &limits);
        host(&mut world);
        add_key(&mut world, signed);
        make(&mut world, signed, 80, None);
        let item = run_charter(&world, b"second resolution");
        changed(&change(&mut world, signed, 72, 1, Box::new([item])), 2);
        make(&mut world, signed, 81, None);
        world.answer_list(listed(&[b"scripted", b"new-model"]));
        assert_eq!(world.domain.settings().newest(), if refused { 2 } else { 3 });
        assert_eq!(record(&world).failure, if refused { Some(accounts::ReadFailure::Room) } else { None });
        assert!(world.domain.ready());
        if refused {
            assert_eq!(record(&world).read, None);
            assert_eq!(record(&world).names, 0);
        }
    }
}

#[test]
fn a_gone_name_returns_without_becoming_new_again() {
    let (mut world, signed) = setup();
    world.answer_list(listed(&[b"scripted", b"new-model"]));
    let first = entry(&world, NEW).first_read;
    let task = make(&mut world, signed, 80, Some(63));
    let held = world.domain.core_mut().charter_of(task).clone();
    read(&mut world, &[b"scripted"]);
    assert!(entry(&world, NEW).withdrawn);
    assert!(
        !world
            .domain
            .core_mut()
            .charter(1)
            .expect("list story fixture")
            .policy
            .alternatives
            .iter()
            .any(|model| model.name.as_ref() == b"new-model")
    );
    assert!(!frame(&mut world, signed).projects.as_slice()[0].offer.as_slice().iter().any(|row| row.model == 63));
    assert_eq!(world.domain.core_mut().charter_of(task), &held);
    assert_eq!(world.assigned.as_ref().expect("held assignment").run.policy.model.name.as_ref(), b"new-model");
    read(&mut world, &[b"scripted", b"new-model"]);
    assert!(!entry(&world, NEW).withdrawn);
    assert_eq!(entry(&world, NEW).first_read, first);
}

#[test]
fn a_full_catalogue_counts_the_names_left_out() {
    let (mut world, _) = setup();
    let names: Vec<_> = (0..64).map(|number| format!("new-{number}").into_bytes()).collect();
    let rows: Vec<&[u8]> = names.iter().map(Vec::as_slice).collect();
    world.answer_list(listed(&rows));
    assert_eq!(record(&world).left_out, 1); // One of the 64 numbers is the stock model.
    assert_eq!(record(&world).names.count_ones(), 63);
}

#[test]
fn a_read_waits_for_a_worker_and_resends_after_loss() {
    let (config, limits) = configured();
    let (mut world, signed) = start(config, &limits);
    add_key(&mut world, signed);
    assert!(world.list_reads.is_empty());
    assert_eq!(world.domain.accounts().list(ACCOUNT), None);
    host(&mut world);
    let first = world.list_reads.last().expect("list story fixture").read;
    world.send(root::Event::Core(core::Event::Fleet(fleet::Event::Lost { channel: CHANNEL })));
    host(&mut world);
    assert_eq!(world.list_reads.len(), 2);
    assert_eq!(world.list_reads.last().expect("list story fixture").read, first);
    world.answer_list(listed(&[b"new-model"]));
    assert_eq!(world.domain.settings().newest(), 2);
}

#[test]
fn a_restart_keeps_both_records_or_neither() {
    for committed in [false, true] {
        let (mut world, _) = setup();
        if !committed {
            world.cut = Some(Cut::Submitted(world.store.applied + 1));
        }
        world.answer_list(listed(&[b"scripted", b"new-model"]));
        restart(&mut world, configured().0);
        assert_eq!(world.domain.settings().newest(), if committed { 2 } else { 1 });
        assert_eq!(world.domain.accounts().list(ACCOUNT).is_some(), committed);
        let count = world.list_reads.len();
        host(&mut world);
        world.send(root::Event::Core(core::Event::Account(accounts::Event::Component(
            accounts::FromComponent::Granted {
                account: SOURCE,
                generation: 2,
                form: link::Form::Credential(link::Scheme::Key),
                valid: accounts::KEY_VALID,
            },
        ))));
        assert_eq!(world.list_reads.len() - count, usize::from(!committed), "only an unfinished read is due again");
        if committed {
            assert_eq!(entry(&world, NEW).first_read, record(&world).read);
        }
    }
}

#[test]
fn a_list_revision_withdraws_a_pending_settings_proposal() {
    let (mut world, signed) = setup();
    let item = run_charter(&world, b"proposed instructions");
    let proposal = proposed(&propose(&mut world, signed, 70, 1, Box::new([item])));
    world.answer_list(listed(&[b"scripted", b"new-model"]));
    let reads::Answer::Ended(history) = crate::settings_support::read(
        &mut world,
        signed,
        reads::Read::Proposal { proposer: people::Proposer::Party(OWNER), proposal },
    ) else {
        panic!("withdrawn history")
    };
    let reads::EndedValue::PersonProposal { proposal: row, decision: Some(decision) } = history.value else {
        panic!("withdrawn proposal")
    };
    assert!(matches!(row.state, jig_domain_tasks::PersonProposalState::Withdrawn { .. }));
    assert_eq!(decision.choice, people::ProposalChoice::Withdrawn);
    assert!(updates(&mut world, signed).iter().any(|entry| entry.object
        == documents::InboxObject::Proposal { proposer: people::Proposer::Party(OWNER), number: proposal }));
}

#[test]
fn an_answer_for_a_removed_account_changes_nothing() {
    let (mut world, signed) = setup();
    let _reply = ask(
        &mut world,
        signed,
        71,
        people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::Remove { account: ACCOUNT },
        },
    );
    let before = world.store.rows.clone();
    world.answer_list(listed(&[b"new-model"]));
    assert_eq!(world.store.rows, before);
    assert_eq!(world.domain.settings().newest(), 1);
}
