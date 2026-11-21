use super::*;
use crate::domain::{LostReadState, LostWriterKind, PendingCi, PendingLost, child, goal_branch};
use crate::tests::{adopted, env, fixtures, landing_routes, outputs};
use crate::{Event, Hold, Name, Subscriber, Topic};
use skein_lib::{Queue, Token};

#[expect(clippy::too_many_lines, reason = "one fixture seeds all six restart terms in the same creation order")]
fn setup(count: usize) -> Domain {
    let mut d = landing_routes::landing_fixture();
    let mut environment = env();
    environment.limits.client.pending = 32;
    environment.limits.client.calls = 32;
    d.client = skein_forge_client::Domain::configured(
        &environment.limits.client,
        4,
        skein_forge_client::Config {
            namespace: b"test".as_slice().into(),
            writers: Box::new([skein_forge_client::Writer { forge: 1, author: 7 }]),
        },
    )
    .unwrap();
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    landing_routes::member_landing(&mut d);
    let template = d.landing.rows.get(&61).unwrap().clone();
    d.landing.rows.remove(&61);
    let repository = adopted().provider;
    let mut tasks = List::with_capacity(4);
    for task in [40, 10, 30, 20].iter().take(count) {
        tasks.push(*task).unwrap();
    }
    d.subscriptions
        .insert(
            (99, Topic::Landings { repository, branch: b"main".as_slice().into() }),
            Subscriber {
                task: 99,
                number: 1,
                topic: Topic::Landings { repository, branch: b"main".as_slice().into() },
                own_change: None,
                goal_tasks: Some(tasks.into_boxed()),
                paths: Box::new([]),
            },
        )
        .unwrap();
    for (index, task) in [40, 10, 30, 20].iter().take(count).enumerate() {
        let mut landing = template.clone();
        landing.landing.task = *task;
        d.landing.rows.insert(*task, landing).unwrap();
        let mut change = fixtures::opening_row();
        change.task = *task;
        change.pull = Some(1);
        change.change.last_head = Some([3; 32]);
        d.changes.insert(*task, change).unwrap();
        crate::topics::refresh(&mut d, &environment, *task, &mut Queue::uncapped());
        let owner = Token::new(u64::try_from(index).unwrap());
        d.pending_ci
            .insert(
                owner,
                PendingCi {
                    repository,
                    head: [u8::try_from(index.checked_add(1).unwrap()).unwrap(); 32],
                    refresh: false,
                },
            )
            .unwrap();
        let name = Name {
            forge: repository.forge,
            repository: repository.repository,
            what: crate::What::Branch(Box::new([
                b"temper".as_slice().into(),
                b"1".as_slice().into(),
                bytes::concat(&[b"g", Decimal::of(*task).as_bytes()]),
            ])),
        };
        d.lost
            .insert(
                owner,
                PendingLost {
                    task: *task,
                    attempt: 1,
                    name: name.clone(),
                    comparison: None,
                    baseline: None,
                    state: LostReadState::Asked,
                    kind: LostWriterKind::Written,
                },
            )
            .unwrap();
        d.holds
            .insert(
                name.clone(),
                Hold {
                    name: name.clone(),
                    task: *task,
                    writer: None,
                    beside: None,
                    design_writers: Box::new([]),
                    drift: None,
                },
            )
            .unwrap();
        goal_branch::start(&mut d, &environment, name, None, &mut Queue::uncapped());
    }
    // Ask gathers in creation order, not their tasks' numerical order.
    for task in [40, 10, 30, 20].iter().take(count) {
        crate::landing::step(
            &mut d,
            &environment,
            *task,
            0,
            crate::LandingHeard {
                job: None,
                outcomes: Box::new([]),
                dependencies: Box::new([]),
                held: Box::new([]),
                failure: Box::new([]),
                preparation: None,
                released: false,
                cancelled: false,
                kept: Box::new([]),
            },
            &mut Queue::uncapped(),
        );
    }
    let mut watches = List::with_capacity(4);
    let kinds = [What::Branch(b"main".as_slice().into()), What::Pull(7), What::Issue(8), What::Repository];
    for what in kinds.into_iter().take(count) {
        watches
            .push(skein_forge_client::Watch {
                resource: skein_forge_client::Resource { repository, what },
                participating: false,
            })
            .unwrap();
    }
    child(
        &mut d,
        &environment,
        skein_forge_client::Event::Keep { owner: Token::new(99), watches: watches.into_boxed() },
        &mut Queue::uncapped(),
    );
    d.restart_fresh = Some(false);
    child(&mut d, &environment, skein_forge_client::Event::ReadAfresh, &mut Queue::uncapped());
    d
}

#[test]
fn uncertain_writes_are_counted_by_entry_not_task_and_cut_at_three() {
    use skein_lib::{Duration, Time, Wall};
    for (count, expected) in [
        (1, "1 uncertain write (entry 2 of task 40)"),
        (4, "4 uncertain writes (entry 2 of task 40, entry 5 of task 30, entry 7 of task 10)"),
    ] {
        let mut d = setup(1);
        for (number, task) in [(2, 40), (5, 30), (7, 10), (9, 20)].iter().take(count) {
            let mut entry = crate::tests::status(*number, *task);
            entry.effect.write =
                skein_forge::api::Write::CreateBranch { branch: b"saved".as_slice().into(), commit: [3; 32] };
            entry.start = Some(skein_forge_client::Position { at: Time::ZERO, comment: 0, review: 0, review_page: 1 });
            entry.attempt = Some(skein_forge_client::Attempt {
                sent: Time::ZERO,
                deadline: env().now.saturating_add(Duration::from_secs(600)),
                wall: Wall::EPOCH,
                expires: Wall::from_nanos(700_000_000_000),
            });
            child(&mut d, &env(), skein_forge_client::Event::Make { entry }, &mut Queue::uncapped());
        }
        assert!(d.restart_waits(RestartStage::Settled).is_empty(), "not settling yet");
        child(&mut d, &env(), skein_forge_client::Event::SettleOutbox, &mut Queue::uncapped());
        assert_eq!(d.restart_waits(RestartStage::Settled).as_ref(), expected.as_bytes());
        child(&mut d, &env(), skein_forge_client::Event::Settle, &mut Queue::uncapped());
        assert!(d.restart_waits(RestartStage::Settled).is_empty(), "stopping owes no restart wait");
    }
}

#[test]
fn every_fresh_kind_counts_and_names_its_three_oldest_without_mutation() {
    for (count, expected) in [
        (
            1,
            "1 fresh read (org/repo branch main), 1 landing gather (task 40), 1 CI read (org/repo at 010101010101), 1 goal-file read (task 40), 1 lost writer's read (task 40), 1 goal branch (task 40)",
        ),
        (
            4,
            "4 fresh reads (org/repo branch main, org/repo pull 7, org/repo issue 8), 4 landing gathers (task 40, task 10, task 30), 4 CI reads (org/repo at 010101010101, org/repo at 020202020202, org/repo at 030303030303), 4 goal-file reads (task 40, task 10, task 30), 4 lost writers' reads (task 40, task 10, task 30), 4 goal branches (task 40, task 10, task 30)",
        ),
    ] {
        let d = setup(count);
        let words = d.restart_waits(RestartStage::ReadAfresh);
        assert_eq!(words.as_ref(), expected.as_bytes());
        assert_eq!(
            d.restart_waits(RestartStage::ReadAfresh).as_ref(),
            expected.as_bytes(),
            "observing again owes the same terms"
        );
    }
    let mut d = setup(4);
    let environment = env();
    // Listings have their own table order, after live resources.
    let repository = adopted().provider;
    child(
        &mut d,
        &environment,
        skein_forge_client::Event::Keep {
            owner: Token::new(100),
            watches: Box::new([
                skein_forge_client::Watch {
                    resource: skein_forge_client::Resource { repository, what: What::Pull(7) },
                    participating: false,
                },
                skein_forge_client::Watch {
                    resource: skein_forge_client::Resource { repository, what: What::Issue(8) },
                    participating: false,
                },
            ]),
        },
        &mut Queue::uncapped(),
    );
    assert!(
        d.restart_waits(RestartStage::ReadAfresh)
            .starts_with(b"3 fresh reads (org/repo pull 7, org/repo issue 8, org/repo listing), ")
    );
    d.restart_fresh = None;
    assert!(d.restart_waits(RestartStage::ReadAfresh).is_empty());
    assert!(d.restart_waits(RestartStage::Restored).is_empty());
}
