#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary boundary fixtures allocate values at the measured limit and one past it"
)]
use super::*;
use crate::reconcile::{MissingLink, Need};
use temper_engine_domain_forge_landing as policy;

fn name(number: u32) -> Name {
    Name { forge: 1, repository: 2, what: What::Pull(u64::from(number)) }
}
#[expect(clippy::too_many_lines, reason = "one fixture per owned restore table")]
fn row(kind: RecordKind, number: u32) -> Stored {
    let n = u64::from(number);
    match kind {
        RecordKind::Landing => Stored::Landing(LandingRow {
            landing: policy::Landing {
                task: n,
                project: 5,
                phase: policy::Phase::Waiting,
                members: Box::new([]),
                preparation: 0,
                stale_proofs: 0,
            },
            asks: Box::new([]),
            entries: Box::new([]),
        }),
        RecordKind::Receipt => Stored::Receipt {
            key: ReceiptKey {
                project: number,
                repository: REPO,
                check: Check::Gate(b"test".as_slice().into()),
                key: [0; 32],
            },
            receipt: Receipt {
                passed: true,
                run: ReceiptRun { id: n, number: n, url: Box::new([]) },
                candidate: [0; 32],
                ended: Wall::EPOCH,
            },
        },
        RecordKind::Application => application(n, Wall::from_nanos(101)),
        RecordKind::Call => Stored::Call {
            key: NamedCall { task: n, attempt: 1, completion: 1, position: 1 },
            answer: CallPayload::Refused(skein_forge::api::Error::Refused),
        },
        RecordKind::Repository => {
            let mut row = adopted();
            row.provider.repository = number;
            Stored::Repository(row)
        }
        RecordKind::Hold => Stored::Hold(Hold { name: name(number), task: n, writer: None, drift: None }),
        RecordKind::Names => Stored::Names { task: n, resources: Box::new([]) },
        RecordKind::Subscription => Stored::Subscription(Subscriber {
            task: n,
            number: 1,
            topic: Topic::Ci { repository: REPO, head: [0; 32] },
            own_change: None,
            goal_tasks: None,
            paths: Box::new([]),
        }),
        RecordKind::BranchHead => Stored::BranchHead(BranchHead { name: name(number), commit: [0; 32], owned: true }),
        RecordKind::PullState => Stored::PullState(PullState {
            name: name(number),
            head: [0; 32],
            ci: skein_forge::api::Ci::Passed,
            state: skein_forge::api::State::Open,
        }),
        RecordKind::Ci => Stored::Ci(CiState {
            repository: skein_forge::api::Repository { forge: 1, repository: number },
            head: [0; 32],
            status: skein_forge::api::Ci::Passed,
        }),
        RecordKind::Landed => Stored::Landed { commit: [u8::try_from(number).unwrap(); 32], task: n },
        RecordKind::ProposedEffect => {
            Stored::ProposedEffect(ProposedEffect { number: n, entry: status(n, n), resource: What::Repository })
        }
        RecordKind::Entry => Stored::Entry { entry: status(n, n), held: true },
        RecordKind::Binding => {
            Stored::Binding { address: Box::new([u8::try_from(number).unwrap()]), subject: Box::new([]) }
        }
        RecordKind::Flight => Stored::Flight { task: n, entry: n },
        RecordKind::DemoAct => Stored::DemoAct(DemoAct {
            key: [u8::try_from(number).unwrap(); 16],
            act: DemoActKind::Close { project: 5, task: n },
        }),
        RecordKind::Writer => {
            Stored::Writer(skein_forge_client::Writer { forge: u16::try_from(number).unwrap(), author: n })
        }
        RecordKind::Change => {
            let mut row = fixtures::opening_row();
            row.task = n;
            Stored::Change(row)
        }
        RecordKind::Issue => Stored::Issue(IssueRow {
            goal: n,
            repository: REPO,
            state: issues::Projected::default(),
            number: None,
            pending: None,
            before: None,
            failed: false,
            desired: None,
        }),
        RecordKind::Release => Stored::Release(ReleaseRow {
            task: n,
            root: n,
            ending: ReleaseEnding::Done,
            close_pull: None,
            comment: None,
            pending: None,
            failed: false,
            effects_settled: true,
            releasing: true,
        }),
        RecordKind::Client | RecordKind::Demo => panic!("no own table"),
    }
}
fn application(token: u64, until: Wall) -> Stored {
    Stored::Application { token, until, outcome: ApplicationOutcome::ForgeRefused(AdoptionRefusal::Refused) }
}
fn restore(d: &mut Domain, row: Stored) {
    assert!(outputs(d, Event::Restore { record: row }).is_empty());
}

#[test]
fn owned_tables_fit_at_limit_replace_and_refuse_one_more() {
    use RecordKind as K;
    for (kind, limit) in [
        (K::Landing, LIMITS.landings),
        (K::Receipt, LIMITS.repositories * LIMITS.gates * LIMITS.receipts_per_gate),
        (K::Application, LIMITS.applications),
        (K::Call, LIMITS.named_calls),
        (K::Repository, LIMITS.repositories),
        (K::Hold, LIMITS.holds),
        (K::Names, LIMITS.tasks),
        (K::Subscription, LIMITS.subscriptions),
        (K::BranchHead, CLIENT.resources),
        (K::PullState, CLIENT.resources),
        (K::Ci, LIMITS.subscriptions),
        (K::Landed, LIMITS.landings.max(LIMITS.changes)),
        (K::ProposedEffect, LIMITS.tasks),
        (K::Entry, LIMITS.entries),
        (K::Binding, LIMITS.bindings),
        (K::Flight, LIMITS.entries),
        (K::DemoAct, demo_act_bound(&LIMITS).unwrap()),
        (K::Writer, CLIENT.repositories),
        (K::Change, LIMITS.changes),
        (K::Issue, LIMITS.issues),
        (K::Release, LIMITS.tasks),
    ] {
        let mut d = domain();
        // Constructor writer 1 is one of the restored writer keys.
        for number in 1..=limit {
            restore(&mut d, row(kind, number));
        }
        assert_eq!(d.restore_fit_failure(), None, "{kind:?} fits exactly");
        restore(&mut d, row(kind, limit));
        assert_eq!(d.restore_fit_failure(), None, "{kind:?} replaces at capacity");
        restore(&mut d, row(kind, limit + 1));
        assert_eq!(
            d.restore_fit_failure(),
            Some(&RestoreFit {
                family: Some(kind),
                set: RestoreSet::Rows(kind),
                held: u64::from(limit),
                additional: 1,
                limit: u64::from(limit)
            }),
            "{kind:?}"
        );
    }
}

#[test]
fn fields_refuse_measured_bytes_and_normal_admission_still_judges_identity() {
    for (address, subject, set, held, limit) in [
        (254, 255, None, 0, 0),
        (255, 0, Some(RestoreSet::BindingAddressBytes), 255, 254),
        (1, 256, Some(RestoreSet::BindingSubjectBytes), 256, 255),
    ] {
        let mut d = domain();
        restore(
            &mut d,
            Stored::Binding {
                address: vec![b'A'; address].into_boxed_slice(),
                subject: vec![0; subject].into_boxed_slice(),
            },
        );
        match set {
            None => assert_eq!(d.restore_fit_failure(), None),
            Some(set) => assert_eq!(
                d.restore_fit_failure(),
                Some(&RestoreFit { family: Some(RecordKind::Binding), set, held, additional: 0, limit })
            ),
        }
    }
    for (task, attempt) in [(0, 1), (1, 0), (0, 0)] {
        let mut d = domain();
        let key = NamedCall { task, attempt, completion: 1, position: 1 };
        let answer = CallPayload::Refused(skein_forge::api::Error::Refused);
        assert_eq!(d.keep_named_answer(&LIMITS, key, answer.clone()).unwrap_err(), answer);
        restore(&mut d, Stored::Call { key, answer: answer.clone() });
        assert_eq!(d.named_answer(key), Some(answer));
        assert_eq!(d.restore_fit_failure(), None);
    }
}

#[test]
fn named_payload_bytes_and_nested_rows_have_exact_boundaries() {
    use skein_forge::api::Answer;
    for bytes in [CLIENT.answer_bytes, CLIENT.answer_bytes + 1] {
        let mut d = domain();
        let answer = Answer::File {
            bytes: vec![0; usize::try_from(bytes).unwrap()].into_boxed_slice(),
            head: [0; 32],
            path: Box::new([]),
            truncated: false,
        };
        restore(
            &mut d,
            Stored::Call {
                key: NamedCall { task: 1, attempt: 1, completion: 1, position: 1 },
                answer: CallPayload::Read(Box::new(Ok(answer))),
            },
        );
        if bytes == CLIENT.answer_bytes {
            assert_eq!(d.restore_fit_failure(), None);
        } else {
            assert_eq!(
                d.restore_fit_failure(),
                Some(&RestoreFit {
                    family: Some(RecordKind::Call),
                    set: RestoreSet::NamedAnswerBytes,
                    held: u64::from(bytes),
                    additional: 0,
                    limit: u64::from(CLIENT.answer_bytes)
                })
            );
        }
    }
    for rows in [CLIENT.rows, CLIENT.rows + 1] {
        let mut d = domain();
        let answer = Answer::Branches(vec![Box::<[u8]>::default(); usize::try_from(rows).unwrap()].into_boxed_slice());
        restore(
            &mut d,
            Stored::Call {
                key: NamedCall { task: 1, attempt: 1, completion: 1, position: 1 },
                answer: CallPayload::Read(Box::new(Ok(answer))),
            },
        );
        if rows == CLIENT.rows {
            assert_eq!(d.restore_fit_failure(), None);
        } else {
            assert_eq!(d.restore_fit_failure().unwrap().set, RestoreSet::NamedAnswerRows);
            assert_eq!(d.restore_fit_failure().unwrap().held, u64::from(rows));
        }
    }
    for rows in [CLIENT.rows, CLIENT.rows + 1] {
        let mut d = domain();
        let mut pull = fixtures::open_pull();
        pull.reviewers = vec![1; usize::try_from(rows).unwrap()].into_boxed_slice();
        restore(
            &mut d,
            Stored::Call {
                key: NamedCall { task: 1, attempt: 1, completion: 1, position: 1 },
                answer: CallPayload::Read(Box::new(Ok(Answer::Pull(pull)))),
            },
        );
        if rows == CLIENT.rows {
            assert_eq!(d.restore_fit_failure(), None);
        } else {
            assert_eq!(d.restore_fit_failure().unwrap().set, RestoreSet::NamedAnswerRows);
            assert_eq!(d.restore_fit_failure().unwrap().held, u64::from(rows));
        }
    }
}

#[test]
fn references_refuse_before_mutation_and_first_failure_stops_recovery() {
    let mut d = domain();
    let capacity = crate::reconcile::capacity(&LIMITS).unwrap();
    // A single-reference replacement isolates this entrance from all retained table bounds.
    for _ in 0..capacity {
        restore(&mut d, row(RecordKind::Repository, 1));
    }
    assert_eq!(d.restore_fit_failure(), None);
    let mut changed = adopted();
    changed.owner = b"must not replace".as_slice().into();
    changed.provider.repository = 1;
    restore(&mut d, Stored::Repository(changed));
    let first = RestoreFit {
        family: Some(RecordKind::Repository),
        set: RestoreSet::References,
        held: u64::from(capacity),
        additional: 1,
        limit: u64::from(capacity),
    };
    assert_eq!(d.restore_fit_failure(), Some(&first));
    assert_eq!(
        d.repositories.get(&skein_forge::api::Repository { forge: 1, repository: 1 }).unwrap().owner,
        adopted().owner
    );
    for event in [
        Event::Restore { record: row(RecordKind::Binding, 1) },
        Event::Restored { clock: skein_forge_client::RecoveryClock::Wall },
        Event::ReadAfresh,
        Event::SettleOutbox,
        Event::Settle,
    ] {
        assert!(outputs(&mut d, event).is_empty());
    }
    d.restore_begin(Wall::EPOCH);
    assert_eq!(d.fit_restored(&LIMITS), Err(first));
    let mut out = Queue::uncapped();
    resume(&mut d, &env(), &mut out);
    fire(&mut d, &env(), &mut out);
    assert!(out.is_empty());
    assert_eq!(d.restore_fit_failure(), Some(&first));
}

#[test]
fn application_expiry_uses_one_explicit_start_observation() {
    let mut d = domain();
    d.restore_begin(Wall::from_nanos(100));
    for token in 0..100 {
        restore(&mut d, application(token, Wall::from_nanos(100)));
    }
    for token in 100..104 {
        restore(&mut d, application(token, Wall::from_nanos(101)));
    }
    d.restore_begin(Wall::from_nanos(200));
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(d.application_outcome(Token::new(0)), None);
    assert!(d.application_outcome(Token::new(100)).is_some());
    restore(&mut d, application(104, Wall::from_nanos(101)));
    assert_eq!(d.restore_fit_failure().unwrap().set, RestoreSet::Rows(RecordKind::Application));
    let mut direct = domain();
    restore(&mut direct, application(1, Wall::EPOCH));
    assert!(direct.application_outcome(Token::new(1)).is_some(), "explicit compatibility default");
}

#[test]
fn watcher_union_fits_distinct_names_and_refuses_conversion() {
    for count in [CLIENT.resources, CLIENT.resources + 1] {
        let mut d = domain();
        for number in 0..count {
            let task = u64::from(number / LIMITS.resources_per_task);
            let mut resources = Vec::new();
            for n in (number / LIMITS.resources_per_task) * LIMITS.resources_per_task..=number {
                resources.push(name(n));
            }
            restore(&mut d, Stored::Names { task, resources: resources.into_boxed_slice() });
        }
        let result = d.fit_restored(&LIMITS);
        if count == CLIENT.resources {
            assert_eq!(result, Ok(()));
        } else {
            assert_eq!(
                result,
                Err(RestoreFit {
                    family: Some(RecordKind::Names),
                    set: RestoreSet::Watches,
                    held: u64::from(CLIENT.resources),
                    additional: 1,
                    limit: u64::from(CLIENT.resources)
                })
            );
        }
    }
    for bytes in [LIMITS.name_bytes, LIMITS.name_bytes + 1] {
        let mut d = domain();
        restore(
            &mut d,
            Stored::Names {
                task: 1,
                resources: Box::new([Name {
                    what: What::Branch(Box::new([vec![b'x'; usize::try_from(bytes).unwrap()].into_boxed_slice()])),
                    ..name(1)
                }]),
            },
        );
        if bytes == LIMITS.name_bytes {
            assert_eq!(d.fit_restored(&LIMITS), Ok(()));
        } else {
            assert_eq!(
                d.fit_restored(&LIMITS),
                Err(RestoreFit {
                    family: Some(RecordKind::Names),
                    set: RestoreSet::NameBytes,
                    held: u64::from(bytes),
                    additional: 0,
                    limit: u64::from(LIMITS.name_bytes)
                })
            );
        }
    }
    let mut d = domain();
    for task in 0..LIMITS.tasks {
        restore(&mut d, Stored::Names { task: u64::from(task), resources: Box::new([name(1)]) });
    }
    assert_eq!(d.fit_restored(&LIMITS), Ok(()), "shared names consume one distinct watcher");
}

fn adopted_outcome() -> ApplicationOutcome {
    ApplicationOutcome::Adopted(AdoptedRepository {
        repository: REPO,
        owner: b"org".as_slice().into(),
        name: b"repo".as_slice().into(),
        role: Role::Owned,
        home: true,
        kinds: adopted().kinds,
        protection: Protection::Absent,
    })
}
#[test]
fn expired_adopted_history_neither_counts_nor_links_an_absent_repository() {
    let mut d = domain();
    d.restore_begin(Wall::from_nanos(100));
    for token in 0..100 {
        restore(&mut d, Stored::Application { token, until: Wall::from_nanos(100), outcome: adopted_outcome() });
    }
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(d.fit_restored(&LIMITS), Ok(()));
    d.reconcile(&reconcile::empty_tasks());
}
#[test]
fn nonexpired_adopted_history_still_asserts_its_missing_repository() {
    let mut d = domain();
    d.restore_begin(Wall::from_nanos(100));
    restore(&mut d, Stored::Application { token: 1, until: Wall::from_nanos(101), outcome: adopted_outcome() });
    assert_eq!(
        d.missing_link(&reconcile::empty_tasks()),
        Some(MissingLink { kind: RecordKind::Application, key: Key::Application(1), need: Need::Repository(REPO) })
    );
}
#[test]
fn a_fitting_zero_call_still_asserts_its_missing_task_link() {
    let mut d = domain();
    restore(&mut d, row(RecordKind::Call, 0));
    assert_eq!(
        d.missing_link(&reconcile::empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::Call,
            key: Key::Call(NamedCall { task: 0, attempt: 1, completion: 1, position: 1 }),
            need: Need::Task { task: 0, retired: None }
        })
    );
}
#[test]
fn independent_timer_room_is_checked_before_insertion() {
    for projection in [false, true] {
        let mut d = domain();
        let limit = if projection { LIMITS.issues } else { LIMITS.changes };
        for number in 100..100 + limit {
            if projection {
                d.projection_after(u64::from(number), Wall::EPOCH);
            } else {
                d.wait_change(&env(), u64::from(number), Some(Wall::EPOCH));
            }
        }
        let mut record = row(if projection { RecordKind::Issue } else { RecordKind::Change }, 1);
        if let Stored::Issue(row) = &mut record {
            row.desired = Some(issues::GoalView {
                phase: issues::Phase::Done,
                goal: 1,
                repository: 2,
                title: "".into(),
                goal_text: "".into(),
                plan: Box::new([]),
                milestones: Box::new([]),
                finished: Some("".into()),
            });
        }
        restore(&mut d, record);
        assert_eq!(
            d.restore_fit_failure().unwrap().set,
            if projection { RestoreSet::ProjectionTimers } else { RestoreSet::ChangeTimers }
        );
        assert_eq!(d.restore_fit_failure().unwrap().held, u64::from(limit));
        if projection {
            assert!(d.issues.is_empty());
        } else {
            assert!(d.changes.is_empty());
        }
    }
}

fn landing(members: u32, gates: u32) -> LandingRow {
    let member = policy::Member {
        change: 1,
        repository: policy::Repository { forge: 1, repository: 2 },
        pull: 1,
        head: None,
        base: None,
        candidate: None,
        gates: vec![
            policy::GateProof { proof: policy::Proof::Unkeyed, posted: policy::Posted::Nothing, faults: 0 };
            usize::try_from(gates).unwrap()
        ]
        .into_boxed_slice(),
        asked: None,
        landed: None,
    };
    LandingRow {
        landing: policy::Landing {
            task: 1,
            project: 5,
            phase: policy::Phase::Waiting,
            members: vec![member; usize::try_from(members).unwrap()].into_boxed_slice(),
            preparation: 0,
            stale_proofs: 0,
        },
        asks: Box::new([]),
        entries: Box::new([]),
    }
}
#[test]
fn landing_geometry_fits_before_reconstruction_and_index_conversion() {
    let mut d = domain();
    for task in 1..=LIMITS.landings {
        let mut row = landing(LIMITS.landing_members, LIMITS.gates);
        row.landing.task = u64::from(task);
        row.asks = vec![None; usize::try_from(LIMITS.landing_members).unwrap()].into_boxed_slice();
        row.entries = vec![
            LandingEntry { entry: 1, member: 0, gate: 0, operation: Operation::Pending };
            usize::try_from(LIMITS.entries).unwrap()
        ]
        .into_boxed_slice();
        restore(&mut d, Stored::Landing(row));
    }
    assert_eq!(d.fit_restored(&LIMITS), Ok(()));
    for (row, set, held, limit) in [
        (landing(3, 2), RestoreSet::LandingMembers, 3, 2),
        (landing(2, 3), RestoreSet::LandingGates, 3, 2),
        (LandingRow { asks: vec![None; 3].into_boxed_slice(), ..landing(2, 2) }, RestoreSet::LandingAsks, 3, 2),
        (
            LandingRow {
                entries: vec![LandingEntry { entry: 1, member: 0, gate: 0, operation: Operation::Pending }; 9]
                    .into_boxed_slice(),
                ..landing(2, 2)
            },
            RestoreSet::LandingEntries,
            9,
            8,
        ),
    ] {
        let mut d = domain();
        restore(&mut d, Stored::Landing(row));
        assert_eq!(
            d.restore_fit_failure(),
            Some(&RestoreFit { family: Some(RecordKind::Landing), set, held, additional: 0, limit })
        );
        assert!(d.landing.rows.is_empty());
    }
}
#[test]
fn a_zero_capacity_table_refuses_its_first_row() {
    for (kind, limits) in [
        (RecordKind::Binding, Limits { bindings: 0, ..LIMITS }),
        (RecordKind::Call, Limits { named_calls: 0, ..LIMITS }),
        (RecordKind::Change, Limits { changes: 0, ..LIMITS }),
        (RecordKind::Issue, Limits { issues: 0, ..LIMITS }),
        (RecordKind::Receipt, Limits { gates: 0, ..LIMITS }),
    ] {
        let mut d = Domain::new(
            &limits,
            4,
            skein_forge_client::Config { namespace: b"test".as_slice().into(), writers: Box::new([]) },
            &[],
            &[],
            false,
        )
        .unwrap();
        let mut out = Queue::uncapped();
        step(&mut d, &Env { limits, ..env() }, Event::Restore { record: row(kind, 1) }, &mut out);
        assert_eq!(
            d.restore_fit_failure(),
            Some(&RestoreFit { family: Some(kind), set: RestoreSet::Rows(kind), held: 0, additional: 1, limit: 0 })
        );
    }
}

#[test]
fn noncanonical_binding_bytes_restore_unchanged_while_bind_normalizes() {
    let mut d = domain();
    restore(
        &mut d,
        Stored::Binding { address: b"UPPER@EXAMPLE.COM".as_slice().into(), subject: b"old".as_slice().into() },
    );
    let sent = outputs(
        &mut d,
        Event::Bind {
            request: Token::new(1),
            address: b"Upper@Example.com".as_slice().into(),
            subject: b"new".as_slice().into(),
        },
    );
    assert!(sent.contains(&Request::Save {
        record: Stored::Binding { address: b"upper@example.com".as_slice().into(), subject: b"new".as_slice().into() },
        read_afresh: None,
        release: None
    }));
}
