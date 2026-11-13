//! Draw current records and agreed versions through both components (bridges.md, 9).
use crate::{joined_world::Joined, translation};
use jig_domain_host as host;
use jig_domain_link::{ListFailure, ListOutcome, SourceName};
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::fixtures::{self, ATTEMPT_TWO as ATTEMPT, RUN_SEVEN as RUN};
use skein_lib::{Duration, Rng, Token};

fn upward(seed: u64) -> Vec<(host::LinkUp, Box<[wire::Part]>)> {
    let mut records = fixtures::up_records();
    let mut rng = Rng::new(seed);
    for (record, _) in &mut records {
        match record {
            host::LinkUp::Turn { turn, .. } => {
                turn.spent = rng.between(0, 10);
            }
            host::LinkUp::Call { input, .. } => {
                *input = vec![u8::try_from(rng.below(256)).expect("bounded translation fixture"); 4].into_boxed_slice();
            }
            host::LinkUp::Answer {
                answer: host::RunAnswer { ending: host::Ending::Ended { outcome, .. }, .. },
                ..
            } => {
                *outcome = translation::golden(
                    smith_charter::patches::kept::smith_charter::TEXTS.last().expect("bounded translation fixture"),
                    "record_run_result_smallest",
                );
            }
            _ => {}
        }
    }
    records.extend([
        (host::LinkUp::Logins { logins: Box::new([]) }, Box::new([]) as Box<[wire::Part]>),
        (
            host::LinkUp::SignedOut {
                name: SourceName::new(b"alice").expect("bounded translation fixture"),
                result: jig_domain_link::SignOutResult::SignedOut,
            },
            Box::new([]),
        ),
        (host::LinkUp::ListRead { read: seed, outcome: ListOutcome::Failed(ListFailure::Agent) }, Box::new([])),
        (host::LinkUp::Draining, Box::new([])),
        (
            host::LinkUp::JobAnswer {
                run: RUN,
                attempt: ATTEMPT,
                answer: host::JobEnding::Finished {
                    result: vec![u8::try_from(rng.below(256)).expect("bounded translation fixture"); 4]
                        .into_boxed_slice(),
                    work: host::Work { left: None, saved: None },
                },
            },
            Box::new([]),
        ),
    ]);
    records
}
fn exercise(world: &mut Joined, seed: u64) {
    for (record, parts) in upward(seed) {
        world.worker_up(record, parts);
        world.run();
    }
    let mut assignment = fixtures::assignment();
    assignment.assignment.charter = translation::charter(1);
    assignment.turns = Box::new([]);
    world.engine_down(
        host::LinkDown::Assign { assignment },
        fixtures::assignment_parts(),
        crate::joined_world::values(),
    );
    world.run();
    let name = SourceName::new(b"alice").expect("bounded translation fixture");
    let grant = fixtures::grant(jig_domain_link::Form::Credential(jig_domain_link::Scheme::Bearer));
    for down in [
        host::LinkDown::Message {
            run: RUN,
            attempt: ATTEMPT,
            name: Token::new(seed),
            sender: b"sender".as_slice().into(),
            words: vec![b'w'; usize::try_from(seed % 8).expect("bounded translation fixture")].into_boxed_slice(),
            finish_requested: seed % 2 == 0,
        },
        fixtures::relay_answer(b"call"),
        host::LinkDown::TurnKept { run: RUN, attempt: ATTEMPT, turn: 1 },
        host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT },
        host::LinkDown::Cancel { run: RUN, attempt: ATTEMPT },
        host::LinkDown::Sources { sources: Box::new([name]) },
        host::LinkDown::Check { name },
        host::LinkDown::SignOut { name },
        host::LinkDown::AssignJob {
            job: host::JobAssignment {
                run: RUN,
                attempt: ATTEMPT,
                kind: jig_domain_link::JobKind(1),
                spec: vec![b's'; usize::try_from(seed % 8).expect("bounded translation fixture")].into_boxed_slice(),
                deadline: Duration::from_secs(seed),
                workspace: None,
                grants: Box::new([]),
            },
        },
    ] {
        world.engine_down(down, Box::new([]), Box::new([]));
        world.run();
    }
    for down in [
        host::LinkDown::Grant { run: RUN, attempt: ATTEMPT, grant },
        host::LinkDown::ReadList { read: seed, endpoint: 1, grant },
    ] {
        world.engine_down(down, Box::new([]), Box::new([Some(crate::joined_world::credential(engine_scheme()))]));
        world.run();
    }
    world.engine_busy();
    world.run();
}
fn engine_scheme() -> jig_protocol_engine_workers::Scheme {
    jig_protocol_engine_workers::Scheme::Bearer
}
#[test]
fn fuzzy_every_kind_at_drawn_agreed_versions_arrives_unchanged_or_is_refused() {
    let mut kinds = std::collections::BTreeSet::new();
    let mut refusals = 0;
    for seed in 0..12 {
        let mut rng = Rng::new(seed);
        let version = u16::try_from(rng.between(u64::from(wire::wire::FLOOR), u64::from(wire::VERSION)))
            .expect("bounded translation fixture");
        let charter = u16::try_from(rng.between(4, 5)).expect("bounded translation fixture");
        let mut current = translation::world(12, 5, seed);
        let mut capped = translation::world(version, charter, seed);
        exercise(&mut current, seed);
        exercise(&mut capped, seed);
        assert_eq!(format!("{:?}", capped.engine_root), format!("{:?}", current.engine_root), "seed {seed}");
        assert_eq!(format!("{:?}", capped.worker_root), format!("{:?}", current.worker_root), "seed {seed}");
        kinds.extend(capped.engine_records());
        kinds.extend(capped.worker_records());
        let contract = u16::try_from(rng.between(1, 2)).expect("drawn contract");
        let mut assignment = fixtures::assignment();
        let expected = translation::charter(contract);
        assignment.assignment.charter = expected.clone();
        assignment.turns = Box::new([]);
        let frames = capped.assignment_bytes().len();
        capped.engine_down(
            host::LinkDown::Assign { assignment },
            fixtures::assignment_parts(),
            crate::joined_world::values(),
        );
        capped.run();
        if contract == 2 && charter == 4 {
            refusals += 1;
            assert_eq!(capped.assignment_bytes().len(), frames);
            assert!(matches!(
                capped.engine_root.last(),
                Some(jig_protocol_engine_workers::Request::Up {
                    up: host::LinkUp::Answer {
                        answer: host::RunAnswer { ending: host::Ending::Refused(host::Refusal::Untranslated(_)), .. },
                        ..
                    },
                    ..
                })
            ));
        } else {
            let Some(jig_protocol_worker_link::Request::Down { down: host::LinkDown::Assign { assignment }, .. }) =
                capped.worker_root.last()
            else {
                panic!("assignment")
            };
            assert_eq!(assignment.assignment.charter, expected);
        }
        current.finish();
        capped.finish();
    }
    assert_eq!(kinds.len(), wire::KINDS.len());
    assert!(refusals > 0, "draws exercise refusals too");
}
