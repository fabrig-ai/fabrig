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
        RecordKind::Closer => Stored::Closer(CloserRow {
            task: n,
            goal: n,
            closer: temper_engine_domain_forge_closer::Closer::new(),
            making: None,
            close_changes: Box::new([]),
            landing_changes: Box::new([]),
            done: false,
        }),
        RecordKind::Making => Stored::Making(making::row(number)),
        RecordKind::LandingLog => Stored::LandingLog(LandingLog {
            repository: skein_forge::api::Repository { forge: 1, repository: number },
            landings: Box::new([]),
        }),
        RecordKind::Follow => Stored::Follow(FollowRow {
            project: 5,
            follow: number,
            source: REPO,
            branch: b"main".as_slice().into(),
            heard: None,
            task: None,
            stopped: None,
            queue: Box::new([]),
            skips: Box::new([]),
            parity: None,
            questions: Box::new([]),
            pending: None,
        }),
        RecordKind::Tier => Stored::Tier(tier_row(number)),
        RecordKind::Verification => Stored::Verification(Verification {
            repository: REPO,
            commit: [u8::try_from(number).unwrap(); 32],
            changes: Box::new([]),
            tiers: Box::new([]),
        }),
        RecordKind::Forge => Stored::Forge(records::record(u16::try_from(number).expect("fixture forge"))),
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
                failures: None,
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
        RecordKind::Hold => {
            Stored::Hold(Hold { design_writers: Box::new([]), name: name(number), task: n, writer: None, drift: None })
        }
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
        RecordKind::Dismissed => Stored::Dismissed { party: n, what: PersonFact::ToBegin { project: number } },
        RecordKind::Binding => {
            Stored::Binding { address: Box::new([u8::try_from(number).unwrap()]), subject: Box::new([]) }
        }
        RecordKind::Setting => Stored::Setting(crate::settings::SettingRow {
            key: crate::settings::SettingKey { kind: 1, scope: 0, number, slot: crate::settings::SettingSlot::Change },
            edit: crate::settings::Edit::Remove,
        }),
        RecordKind::Flight => Stored::Flight { task: n, entry: n },
        RecordKind::Writer => {
            Stored::Writer(skein_forge_client::Writer { forge: u16::try_from(number).unwrap(), author: n })
        }
        RecordKind::Change => {
            let mut row = fixtures::opening_row();
            row.task = n;
            Stored::Change(Box::new(row))
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
        RecordKind::ProjectFacts => Stored::ProjectFacts(ProjectFacts {
            project: u32::try_from(n).unwrap(),
            landed: 0,
            last_landed: Wall::EPOCH,
            brief: None,
            running: None,
        }),
        RecordKind::Looked => {
            Stored::Looked(Looked { party: n, project: 0, previous: None, latest: Wall::EPOCH, seen: 0, next_seen: 0 })
        }
        RecordKind::LandedDraft => Stored::LandedDraft(LandedDraft {
            repository: REPO,
            path: format!("{n}.md").into_bytes().into(),
            project: 1,
            branch: b"main".as_slice().into(),
            commit: [1; 32],
            title: Box::new([]),
            writer: 1,
            at: Wall::EPOCH,
            chat: None,
            goal: None,
            ready: true,
        }),
        RecordKind::Away => {
            Stored::Away(Away { party: 1, project: 1, since: Wall::EPOCH, until: None, landed: 0, off: None })
        }
        RecordKind::Decided => Stored::Decided(Decided {
            project: 1,
            number: 1,
            party: 1,
            task: 1,
            at: Wall::EPOCH,
            question: Box::new([]),
            options: Box::new([Box::from(b"a".as_slice()), Box::from(b"b".as_slice())]),
            took: Some(0),
            why: Box::new([]),
            overturned: false,
            reach: temper_protocol_reaches::Reach {
                kind: temper_protocol_reaches::Kind::Wording,
                class: temper_protocol_reaches::Class::Decide,
                reason: temper_protocol_reaches::Reason::Kind,
                under: 0,
                sets: 0,
                matched: 0,
                away: false,
            },
        }),
        RecordKind::Client => panic!("no own table"),
    }
}
#[test]
fn follows_restore_with_per_project_and_queue_bounds() {
    let mut d = domain();
    for number in 1..=LIMITS.follows {
        restore(&mut d, row(RecordKind::Follow, number));
    }
    assert_eq!(d.restore_fit_failure(), None);
    restore(&mut d, row(RecordKind::Follow, LIMITS.follows));
    assert_eq!(d.restore_fit_failure(), None, "replacement at bound");
    restore(&mut d, row(RecordKind::Follow, LIMITS.follows + 1));
    assert_eq!(d.restore_fit_failure().expect("project bound").limit, u64::from(LIMITS.follows));
    let mut d = domain();
    let Stored::Follow(mut full) = row(RecordKind::Follow, 1) else { unreachable!("follow fixture") };
    let entry = FollowEntry {
        before: [1; 32],
        after: [2; 32],
        pull: None,
        commits: Box::new([]),
        more_commits: 0,
        files: None,
        more_files: 0,
        change: None,
        state: FollowState::NotMade,
        word: false,
        remake: false,
    };
    full.queue = vec![entry.clone(); usize::try_from(LIMITS.follow_queue).expect("queue bound")].into_boxed_slice();
    restore(&mut d, Stored::Follow(full.clone()));
    assert_eq!(d.restore_fit_failure(), None);
    full.queue = vec![entry; usize::try_from(LIMITS.follow_queue + 1).expect("one past")].into_boxed_slice();
    restore(&mut d, Stored::Follow(full));
    assert_eq!(d.restore_fit_failure().expect("queue bound").set, RestoreSet::FollowQueue);
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
        (K::Making, LIMITS.makings),
        (K::Landing, LIMITS.landings),
        (
            K::Receipt,
            (LIMITS.repositories * (LIMITS.gates + LIMITS.tiers.tiers) + LIMITS.judge_projects * LIMITS.follows)
                * LIMITS.receipts_per_gate,
        ),
        (K::Tier, LIMITS.repositories * LIMITS.tiers.tiers),
        (K::Verification, LIMITS.changes),
        (K::LandingLog, LIMITS.repositories),
        (K::Application, LIMITS.applications),
        (K::Call, LIMITS.named_calls),
        (K::Repository, LIMITS.repositories),
        (K::Hold, LIMITS.holds),
        (K::Names, LIMITS.tasks + LIMITS.repositories),
        (K::Subscription, LIMITS.subscriptions),
        (K::BranchHead, CLIENT.resources),
        (K::PullState, CLIENT.resources),
        (K::Ci, LIMITS.subscriptions),
        (K::Landed, LIMITS.landings.max(LIMITS.changes)),
        (K::ProposedEffect, LIMITS.tasks),
        (K::Entry, LIMITS.entries),
        (K::Binding, LIMITS.bindings),
        (K::Dismissed, LIMITS.dismissed()),
        (K::Setting, crate::settings::rows(&LIMITS).unwrap()),
        (K::Flight, LIMITS.entries),
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
fn named_branch_names_fit_by_bytes_not_rows() {
    use skein_forge::api::Answer;
    for rows in [CLIENT.rows, CLIENT.rows + 1, 40] {
        let names = vec![Box::from(&b"temper/goal"[..]); usize::try_from(rows).unwrap()].into_boxed_slice();
        let answer = CallPayload::Read(Box::new(Ok(Answer::Branches(names))));
        let key = NamedCall { task: 1, attempt: 1, completion: 1, position: 1 };
        let mut d = domain();
        restore(&mut d, Stored::Call { key, answer: answer.clone() });
        assert_eq!(d.restore_fit_failure(), None);
        assert_eq!(d.named_answer(key), Some(answer));
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
        let names = vec![Box::<[u8]>::default(); usize::try_from(rows).unwrap()].into_boxed_slice();
        let item = skein_forge::api::Summary {
            labels: names.clone(),
            kind: skein_forge::api::Kind::Issue,
            author: 7,
            key: None,
            updated: env().now,
            number: 1,
            title: Box::new([]),
            body: Box::new([]),
            state: skein_forge::api::State::Open,
        };
        for answer in [
            Answer::Item { item, comments: Box::new([]), more: false },
            Answer::Protection(Some(skein_forge::api::Protection {
                branch: Box::new([]),
                contexts: names,
                approvals: 0,
                dismiss_stale: false,
                queue: false,
                signatures: false,
                linear: false,
                code_owners: false,
                bypass: skein_forge::api::Bypass::Never,
            })),
        ] {
            let mut d = domain();
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
    let mut d = restoring_domain(Wall::from_nanos(100));
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
    assert_eq!(direct.application_outcome(Token::new(1)), None, "expired decoded row swept");
    assert_eq!(direct.restore_fit_failure(), None);
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
    let mut d = restoring_domain(Wall::from_nanos(100));
    for token in 0..100 {
        restore(&mut d, Stored::Application { token, until: Wall::from_nanos(100), outcome: adopted_outcome() });
    }
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(d.fit_restored(&LIMITS), Ok(()));
    d.reconcile(&reconcile::empty_tasks());
}
#[test]
fn nonexpired_adopted_history_still_asserts_its_missing_repository() {
    let mut d = restoring_domain(Wall::from_nanos(100));
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
        let limit = if projection { LIMITS.issues } else { LIMITS.changes + LIMITS.tasks };
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
        (RecordKind::Dismissed, Limits { bindings: 0, ..LIMITS }),
        (RecordKind::Call, Limits { named_calls: 0, ..LIMITS }),
        (RecordKind::Change, Limits { changes: 0, ..LIMITS }),
        (RecordKind::Issue, Limits { issues: 0, ..LIMITS }),
        (RecordKind::Verification, Limits { changes: 0, ..LIMITS }),
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
        d.restore_begin(Wall::EPOCH);
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

fn unlinked_row(kind: RecordKind, number: u32) -> Stored {
    let mut record = row(kind, number);
    if let Stored::Repository(row) = &mut record {
        row.provider.forge = 99;
    }
    if let Stored::Names { resources, .. } = &mut record {
        *resources = Box::new([Name { forge: 1, repository: 99, what: What::Repository }]);
    }
    if let Stored::Application { outcome, .. } = &mut record {
        *outcome = adopted_outcome();
    }
    if let Stored::Landing(row) = &mut record {
        row.landing.members = Box::new([policy::Member {
            change: u64::from(number),
            repository: policy::Repository { forge: REPO.forge, repository: REPO.repository },
            pull: 1,
            head: None,
            base: None,
            candidate: None,
            gates: Box::new([]),
            asked: None,
            landed: None,
        }]);
    }
    record
}

#[test]
fn missing_links_scan_retained_keys_after_lower_insertions_and_replacements() {
    use RecordKind as K;
    const FIRST: u32 = 2;
    const LOWER: u32 = 1;
    for (kind, expected_row) in [
        (K::Call, FIRST),
        (K::ProposedEffect, LOWER),
        (K::Repository, LOWER),
        (K::Hold, LOWER),
        (K::Names, LOWER),
        (K::Subscription, LOWER),
        (K::BranchHead, LOWER),
        (K::PullState, LOWER),
        (K::Ci, FIRST),
        (K::Entry, LOWER),
        (K::Change, LOWER),
        (K::Issue, LOWER),
        (K::Release, FIRST),
        (K::Application, LOWER),
        (K::Landing, LOWER),
        (K::Receipt, LOWER),
        (K::Flight, FIRST),
        (K::Writer, FIRST),
    ] {
        let mut d = domain();
        let first = unlinked_row(kind, FIRST);
        let expected = stored_key(&unlinked_row(kind, expected_row));
        restore(&mut d, first);
        restore(&mut d, unlinked_row(kind, LOWER));
        restore(&mut d, unlinked_row(kind, LOWER));
        assert_eq!(d.restore_fit_failure(), None, "{kind:?}");
        let missing = d.missing_link(&reconcile::empty_tasks()).expect("first row has an unmade link");
        assert_eq!(missing.kind, kind);
        assert_eq!(missing.key, expected);
    }
}

#[test]
#[should_panic(expected = "restore_begin precedes every restored row")]
fn restoring_a_row_requires_the_start_observation() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    step(&mut d, &env(), Event::Restore { record: application(1, Wall::EPOCH) }, &mut out);
}

#[test]
fn unknown_tier_events_keep_state_and_hold_the_step() {
    let mut d = domain();
    let before = alloc::format!("{d:?}");
    for event in [
        Event::TierTask { repository: REPO, tier: Box::from(b"tier".as_slice()), number: 1, task: 23 },
        Event::TierRefused { repository: REPO, tier: Box::from(b"tier".as_slice()), number: 1 },
        Event::VetoTier { task: 23, entry: 5 },
    ] {
        assert!(outputs(&mut d, event).is_empty());
    }
    let out = outputs(&mut d, Event::StepTier { task: 23, entry: 5, heard: TierHeard { released: true } });
    let [Request::TierDecision { task, decision, entry, projection: _ }] = out.as_ref() else {
        panic!("one tier decision")
    };
    assert_eq!(
        (*task, decision, *entry),
        (23, &TierDecision::Held { words: b"Tier run is unavailable".as_slice().into() }, None)
    );
    assert_eq!(alloc::format!("{d:?}"), before);
    assert_eq!(d.tier_task(23), None);
    assert!(d.describe_tier_effect(23, 5).is_none());
    assert_eq!(d.change_verification(23), temper_engine_domain_forge_change::Verification::Unfollowed);
}

#[test]
fn tier_families_retain_rows_and_report_their_missing_repository() {
    for kind in [RecordKind::Tier, RecordKind::Verification] {
        let mut d = domain();
        let record = row(kind, 2);
        let key = stored_key(&record);
        restore(&mut d, record.clone());
        if kind == RecordKind::Tier {
            let Stored::Tier(row) = &record else { unreachable!("tier fixture") };
            assert_eq!(d.tiers.get(&(row.repository, row.tier.clone())), Some(row));
        } else {
            let Stored::Verification(row) = &record else { unreachable!("verification fixture") };
            assert_eq!(d.verifications.get(&(row.repository, row.commit)), Some(row));
        }
        assert_eq!(
            d.missing_link(&reconcile::empty_tasks()),
            Some(MissingLink { kind, key, need: Need::Repository(REPO) })
        );
        restore(&mut d, Stored::Repository(adopted()));
        assert_eq!(d.missing_link(&reconcile::empty_tasks()), None);
    }
    let mut d = domain();
    let Stored::Tier(mut tier) = row(RecordKind::Tier, 2) else { unreachable!("tier fixture") };
    tier.task = Some(51);
    restore(&mut d, Stored::Repository(adopted()));
    restore(&mut d, Stored::Tier(tier));
    assert_eq!(d.missing_link(&reconcile::empty_tasks()), None);
}

#[test]
fn receipts_keep_tier_room_when_no_gates_are_configured() {
    let limits = Limits { gates: 0, ..LIMITS };
    let mut d = Domain::new(
        &limits,
        4,
        skein_forge_client::Config { namespace: b"test".as_slice().into(), writers: Box::new([]) },
        &[],
        &[],
        false,
    )
    .unwrap();
    let capacity =
        (LIMITS.repositories * LIMITS.tiers.tiers + LIMITS.judge_projects * LIMITS.follows) * LIMITS.receipts_per_gate;
    let env = Env { limits, ..env() };
    let mut out = Queue::uncapped();
    d.restore_begin(Wall::EPOCH);
    for number in 1..=capacity {
        step(&mut d, &env, Event::Restore { record: row(RecordKind::Receipt, number) }, &mut out);
    }
    assert_eq!(d.restore_fit_failure(), None);
    step(&mut d, &env, Event::Restore { record: row(RecordKind::Receipt, capacity.checked_add(1).unwrap()) }, &mut out);
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Receipt),
            set: RestoreSet::Rows(RecordKind::Receipt),
            held: u64::from(capacity),
            additional: 1,
            limit: u64::from(capacity)
        })
    );
    assert!(out.is_empty());
}

#[test]
fn setting_rule_fields_fit_at_the_bound_and_refuse_one_more_before_insertion() {
    use crate::settings as s;
    for (field, bound, set) in [
        (0_u8, LIMITS.change_policy.gates, RestoreSet::SettingGates),
        (1, LIMITS.roles, RestoreSet::SettingApprovals),
        (2, LIMITS.authority_segments, RestoreSet::SettingSegments),
    ] {
        for extra in [0, 1] {
            let count = usize::try_from(bound + extra).unwrap();
            let mut rule = records::setting_rule();
            match field {
                0 => rule.gates = vec![rule.gates[0]; count].into_boxed_slice(),
                1 => rule.approvals = vec![rule.approvals[0]; count].into_boxed_slice(),
                2 => rule.pattern.segments = vec![Box::<[u8]>::default(); count].into_boxed_slice(),
                _ => unreachable!("table field"),
            }
            let key = s::SettingKey { kind: 1, scope: 0, number: 1, slot: s::SettingSlot::Change };
            let edit = s::Edit::Set(s::SizedValue { value: s::Value::Rule(rule), size: 42 });
            let mut d = domain();
            restore(&mut d, Stored::Setting(s::SettingRow { key, edit: edit.clone() }));
            if extra == 0 {
                assert_eq!(d.restore_fit_failure(), None);
                assert_eq!(d.setting_rows.get(&key), Some(&edit));
            } else {
                assert_eq!(
                    d.restore_fit_failure(),
                    Some(&RestoreFit {
                        family: Some(RecordKind::Setting),
                        set,
                        held: u64::from(bound + 1),
                        additional: 0,
                        limit: u64::from(bound),
                    })
                );
                assert!(d.setting_rows.is_empty());
            }
        }
    }
}

fn tier_row(number: u32) -> TierRow {
    TierRow {
        observations: Box::new([]),
        requested: false,
        cancelled: false,
        purpose: None,
        pending: Box::new([]),
        baseline: None,
        branch: TierBranch::Absent,
        probe: None,
        flight: None,
        held: None,
        result: None,
        received: false,
        goals: Box::new([]),
        goal: None,
        started: None,
        repository: skein_forge::api::Repository { forge: 1, repository: number },
        tier: Box::from(b"tier".as_slice()),
        record: temper_engine_domain_forge_tiers::Record {
            sequence: 0,
            judged: None,
            failing: temper_engine_domain_forge_tiers::Failures::Tests(Box::new([])),
            flakes: Box::new([]),
            run: None,
        },
        task: None,
    }
}

#[test]
fn landing_logs_restore_and_report_their_missing_repository() {
    let mut d = domain();
    let record = Stored::LandingLog(LandingLog {
        repository: REPO,
        landings: Box::new([LoggedLanding {
            commit: [7; 32],
            branch: b"main".as_slice().into(),
            at: Wall::from_nanos(123),
            changes: Box::new([LoggedChange {
                signed: Box::new([]),
                task: 51,
                pull: None,
                title: b"A lasting title".as_slice().into(),
            }]),
        }]),
    });
    restore(&mut d, record.clone());
    let Stored::LandingLog(expected) = record else { unreachable!("fixture") };
    assert_eq!(d.landing_log(REPO), Some(&expected));
    assert_eq!(
        d.missing_link(&reconcile::empty_tasks()),
        Some(MissingLink {
            kind: RecordKind::LandingLog,
            key: Key::LandingLog { repository: REPO },
            need: Need::Repository(REPO),
        })
    );
    restore(&mut d, Stored::Repository(adopted()));
    assert_eq!(d.missing_link(&reconcile::empty_tasks()), None);
}

#[test]
fn landing_log_fields_fit_at_the_limit_and_refuse_one_more() {
    let landing = LoggedLanding {
        commit: [7; 32],
        branch: vec![b'x'; usize::try_from(LIMITS.name_bytes).unwrap()].into(),
        at: Wall::EPOCH,
        changes: vec![
            LoggedChange { signed: Box::new([]), task: 1, pull: None, title: Box::new([]) };
            usize::try_from(LIMITS.landing_members).unwrap()
        ]
        .into(),
    };
    for (field, bound, set) in [
        (0_u8, LIMITS.landings_kept, RestoreSet::LandingsKept),
        (1, LIMITS.landing_members, RestoreSet::LandingMembers),
        (2, LIMITS.name_bytes, RestoreSet::NameBytes),
    ] {
        for extra in [0, 1] {
            let mut row = LandingLog {
                repository: REPO,
                landings: vec![landing.clone(); usize::try_from(LIMITS.landings_kept).unwrap()].into(),
            };
            let count = usize::try_from(bound + extra).unwrap();
            match field {
                0 => row.landings = vec![landing.clone(); count].into(),
                1 => row.landings[0].changes = vec![landing.changes[0].clone(); count].into(),
                2 => row.landings[0].branch = vec![b'x'; count].into(),
                _ => unreachable!("fixture field"),
            }
            let mut d = domain();
            restore(&mut d, Stored::LandingLog(row.clone()));
            if extra == 0 {
                assert_eq!(d.restore_fit_failure(), None);
                assert_eq!(d.landing_log(REPO), Some(&row));
            } else {
                assert_eq!(
                    d.restore_fit_failure(),
                    Some(&RestoreFit {
                        family: Some(RecordKind::LandingLog),
                        set,
                        held: u64::from(bound + 1),
                        additional: 0,
                        limit: u64::from(bound),
                    })
                );
                assert!(d.landing_log(REPO).is_none());
            }
        }
    }
}

#[test]
fn landings_kept_is_positive_and_fits_the_store_ceiling() {
    for (landings_kept, accepted) in [(0, false), (1, true), (256, true), (257, false)] {
        assert_eq!(valid(&Limits { landings_kept, ..LIMITS }), accepted);
    }
}
