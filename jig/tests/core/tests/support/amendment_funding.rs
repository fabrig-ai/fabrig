//! Compare the translated task offer with the direct party request, never its funding outcome.
use jig_client_domain::contract as c;
use jig_core_world::{
    fixtures::{ask, delegate, live_row, member},
    world::{Setup, World},
};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, List, Rng, Token, Wall};

fn client_limits() -> jig_client_protocol_link::Limits {
    jig_client_protocol_link::Limits::new(
        jig_client_domain::Limits {
            owners: &[],
            watches: 4,
            handles: 8,
            turn_bytes: 4096,
            ask_bytes: 16384,
            requests: skein_lib::RequestLimits {
                requests: 4,
                bytes: 65536,
                out: 16,
                first: Duration::from_nanos(10),
                most: Duration::from_nanos(80),
                margin: Duration::from_secs(1),
            },
            fields: 4,
            reads: 4,
            parts: 8,
            delegates: 2,
            document_bytes: 65_536,
            records: 2,
            kept: 4,
            projects: 2,
            text: 128,
            long_text: 4096,
            turn_text: 4096,
            segments: 4,
            rows: 32,
            backoff: jig_client_domain::Backoff { first: Duration::from_nanos(10), most: Duration::from_nanos(80) },
            authority: 32,
            accounts: jig_client_domain::AccountCounts { accounts: 8, sources: 8, sign_ins: 8 },
            heartbeat: Duration::from_nanos(100),
        },
        4,
    )
    .expect("valid receiving limits")
}

pub(crate) fn document(world: &mut World, task: u64) -> c::Task {
    if world.reads.is_none() {
        world.reads = Some(jig_core_world::reads::Parties::new(world.store.applied));
    }
    let doc = jig_core_world::stories::instructions::task_document(world, task, 9010);
    let limits = client_limits();
    let jig_domain_core::settings::Value::Model(model) =
        world.domain.settings().value(jig_domain_core::settings::Setting::Model(1)).expect("scripted model")
    else {
        panic!("model setting")
    };
    let models = jig_protocol_engine_clients::translate::llm::catalogue(core::slice::from_ref(model.as_ref()));
    let wire = jig_protocol_engine_clients::translate::task::document(&doc, &models, limits.wire())
        .expect("whole task translates");
    jig_client_protocol_link::translate::documents::task(wire, &limits).expect("whole task decodes")
}

pub(crate) fn change(authority: c::Authority) -> c::Amendment {
    c::Amendment {
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: Some(authority),
        instructions: None,
        tracked: None,
        llm: None,
    }
}

pub(crate) fn offer(doc: &c::Task, change: &c::Amendment) -> c::Admission {
    let offer = &doc.acts.iter().find(|row| row.act == c::ActKind::Amend).expect("amending offered").admission;
    c::admission::amendment(offer, change, &doc.authority)
}

fn pattern(value: &c::Pattern) -> people::Pattern {
    people::Pattern {
        segments: value.segments.clone().into_boxed(),
        last: match &value.last {
            c::PatternEnd::Exact(value) => people::Last::Exact(value.clone()),
            c::PatternEnd::Open(value) => people::Last::Open(value.clone()),
        },
    }
}

// Drafts keep their models: omitting the selector retains them. Other fields copy
// literally, with no policy read here.
fn person(value: &c::Authority) -> people::Authority {
    people::Authority {
        tools: value.tools,
        models: None,
        spend: value.spend,
        deadline: value.deadline,
        notes: value.notes,
        note_resources: value
            .note_resources
            .iter()
            .map(|row| people::ResourceScope { connector: row.connector, pattern: pattern(&row.pattern) })
            .collect(),
        grants: value
            .grants
            .iter()
            .map(|row| people::Grant { connector: row.connector, kind: row.kind, pattern: pattern(&row.pattern) })
            .collect(),
        delegation: people::Delegation {
            tasks: value.delegation.tasks,
            depth: value.delegation.depth,
            kinds: value
                .delegation
                .kinds
                .iter()
                .map(|kind| match kind {
                    c::AuthorityExecutor::Charter(number) => people::Executor::Charter(*number),
                    c::AuthorityExecutor::Procedure(number) => people::Executor::Procedure(*number),
                    c::AuthorityExecutor::Role(number) => people::Executor::Role(*number),
                    c::AuthorityExecutor::Job(number) => people::Executor::Job(*number),
                })
                .collect(),
        },
    }
}

pub(crate) fn request(world: &mut World, task: u64, authority: &c::Authority, key: u8) {
    let revision = live_row(world, task).revisions.spec;
    ask(
        world,
        key,
        people::Ask::Amend {
            project: 1,
            task,
            revision,
            release: None,
            amendment: people::Amendment {
                authority: Some(person(authority)),
                ..jig_test_profile::fixtures::person_unchanged(b"funded amendment")
            },
        },
    );
}

fn prepare(world: &mut World, child_state: u64, spent: u64) -> u64 {
    let parent = world.assignments[0];
    let reserved = child_state == 1;
    let child = if child_state != 0 {
        let mut child = member(world, tasks::Executor::Agent { charter: 1 }, b"reserved work");
        child.authority.budget.spend = 10;
        child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        Some(delegate(world, parent, 1, child))
    } else {
        None
    };
    world.await_timer(parent.0);
    world.send(root::Event::Answer {
        read: None,
        refused: Box::new([]),
        channel: Token::new(7),
        task: parent.0,
        attempt: parent.1,
        cumulative: spent,
        end: tasks::End::Parked,
    });
    if let Some(child) = child {
        let run =
            world.assignments.iter().find(|(task, _)| *task == child).expect("child assigned after parent parked");
        world.send(root::Event::Answer {
            read: None,
            refused: Box::new([]),
            channel: Token::new(7),
            task: child,
            attempt: run.1,
            cumulative: if reserved { 0 } else { 7 },
            end: if reserved {
                tasks::End::Parked
            } else {
                tasks::End::Finished {
                    result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
                    cancel_delegates: false,
                }
            },
        });
        if !reserved {
            let run =
                world.assignments.iter().rev().find(|(task, _)| *task == parent.0).expect("parent woken by the result");
            world.send(root::Event::Answer {
                read: Some(live_row(world, parent.0).last_message),
                refused: Box::new([]),
                channel: Token::new(7),
                task: run.0,
                attempt: run.1,
                cumulative: 0,
                end: tasks::End::Parked,
            });
        }
    }
    // No run reservation obscures the funding judgement while the drafts are compared.
    world.send(root::Event::Core(jig_domain_core::Event::Fleet(jig_domain_fleet::Event::Lost {
        channel: Token::new(7),
    })));
    parent.0
}

fn draft(before: &c::Authority, original: &c::Authority, left: u64, case: u8) -> c::Authority {
    let mut draft = before.clone();
    match case {
        0 => draft.spend += 10,
        1 => draft.spend += left + 1,
        2 => draft.spend = 80,
        3 => draft.deadline = None,
        4 => draft.grants = original.grants.clone(),
        5 => draft.delegation.kinds = original.delegation.kinds.clone(),
        6 => {
            draft.tools = 0;
            draft.grants = List::with_capacity(0);
        }
        7 => {
            draft.spend += 1;
            draft.delegation.kinds = List::with_capacity(0);
        }
        8 => draft.spend = 501,
        9 => draft.spend = 1001,
        _ => unreachable!("ten draft variants"),
    }
    draft
}

fn compare(world: &mut World, task: u64, doc: &c::Task, draft: &c::Authority, key: u8) {
    let predicted = offer(doc, &change(draft.clone()));
    request(world, task, draft, key);
    let Some(people::Reply::Outcome(outcome)) = world.people_answers.last() else { panic!("direct answer") };
    let actual = match outcome {
        people::Outcome::Amended { .. } => 0,
        people::Outcome::Refused(people::Refusal::Authority {
            proposable: Some(people::Accepter::Role(_)), ..
        }) => 1,
        people::Outcome::Refused(_) => 2,
        other => panic!("amendment outcome: {other:?}"),
    };
    let expected = match predicted {
        c::Admission::Allowed => 0,
        c::Admission::Proposed { .. } => 1,
        c::Admission::Refused(_) => 2,
        c::Admission::Bounded { .. } => panic!("concrete client judgement"),
    };
    assert_eq!(actual, expected, "key {key}: {predicted:?}, {outcome:?}");
}

pub(crate) fn sweep(seed: u64) {
    let mut rng = Rng::new(seed);
    for own in [true, false] {
        let mut world = World::new(Setup { seed, ..Setup::default() });
        let child_state = rng.below(3);
        eprintln!("replay funding seed {seed}, own {own}, child {child_state}");
        let task = prepare(&mut world, child_state, rng.below(30));
        let mut doc = document(&mut world, task);
        let original = doc.authority.whole().expect("whole authority");
        let mut narrowed = original.clone();
        narrowed.spend = 100;
        narrowed.deadline = Some(Wall::from_nanos(1_000_000_000_000));
        narrowed.grants = List::with_capacity(0);
        narrowed.delegation.kinds = List::with_capacity(0);
        request(&mut world, task, &narrowed, 101);
        assert!(
            matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::Amended { .. }))),
            "seed {seed}, own {own}, child {child_state}, baseline: {:?}, numbers {:?}, phase {:?}",
            world.people_answers.last(),
            live_row(&world, task).numbers,
            live_row(&world, task).phase
        );
        let amender = if own { 1 } else { 3 };
        if !own {
            let signed = jig_core_world::fixtures::second_party(&mut world, b"another-owner", people::Role::Owner);
            world.sign_in = Some(signed);
        }
        let left = 20 + rng.below(80);
        ask(
            &mut world,
            102,
            people::Ask::SetPool { project: 1, person: amender, budget: if own { 100 + left } else { left } },
        );
        assert!(matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::PoolSet { .. }))));
        for case in 0..10_u8 {
            doc = document(&mut world, task);
            let after = draft(&doc.authority.authority, &original, left, case);
            compare(&mut world, task, &doc, &after, 110 + case);
        }
    }
}
