#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary Rust fixtures build and compare encoded job pages"
)]
use super::fixtures;
use crate::{jobs, keys, records};
use jig_protocol_store::{JobPage, JobRoom, JobRow, JobWrite};
use temper_engine_domain_forge_vocabulary::{ChangeRow, Key, Stored};

fn dispatch(job: &str, page: &JobPage) -> Result<jig_protocol_store::JobAnswer, jig_protocol_store::JobRefusal> {
    jobs::page(
        job,
        page,
        &jobs::Present::default(),
        &crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap(),
    )
}
fn row(task: u64) -> JobRow {
    let mut change = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: false });
    change.task = task;
    change.change.task = task;
    JobRow {
        key: keys::encode(&Key::Change(task), u32::MAX).unwrap(),
        value: records::encode(&Stored::Change(Box::new(change))),
    }
}
fn page(rows: Box<[JobRow]>, writes: u32, bytes: u32) -> JobPage {
    JobPage { end: rows.is_empty(), rows, layout: None, room: JobRoom { writes, bytes: u64::from(bytes) } }
}

#[test]
fn change_parts_job_moves_rows_and_consumes_only_whole_changes_within_the_edge() {
    let input = page(Box::new([row(12), row(13)]), 6, u32::MAX);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let answer = dispatch("change-parts", &input).unwrap();
    assert_eq!(answer.through, None);
    assert_eq!(answer.writes.len(), 6);
    let mut total = 0;
    for (writes, old) in answer.writes.as_chunks::<3>().0.iter().zip(&input.rows) {
        let [
            JobWrite::Put { key: cold_key, value: cold },
            JobWrite::Put { key: hot_key, value: hot },
            JobWrite::Erase { key },
        ] = writes
        else {
            panic!("two parts and erase")
        };
        let Stored::ChangeCold(cold) = records::row(cold_key, cold, &bounds).unwrap() else { panic!("cold") };
        let Stored::ChangeHot(hot) = records::row(hot_key, hot, &bounds).unwrap() else { panic!("hot") };
        let Stored::Change(whole) = records::row(&old.key, &old.value, &bounds).unwrap() else { panic!("whole") };
        assert_eq!(ChangeRow::join(*cold, *hot), *whole);
        assert_eq!(*key, old.key);
        for write in writes {
            total += match write {
                JobWrite::Put { key, value } => {
                    skein_kv::put_bytes(u32::try_from(key.len()).unwrap(), u32::try_from(value.len()).unwrap()).unwrap()
                }
                JobWrite::Erase { key } => skein_kv::erase_bytes(u32::try_from(key.len()).unwrap()).unwrap(),
            };
        }
    }
    for (writes, bytes) in [(3, u32::MAX), (5, u32::MAX), (6, total - 1)] {
        let cut = dispatch("change-parts", &page(Box::new([row(12), row(13)]), writes, bytes)).unwrap();
        assert_eq!(cut.writes.len(), 3);
        assert_eq!(cut.through.as_deref(), Some(input.rows[0].key.as_ref()));
    }
    let exact = dispatch("change-parts", &page(Box::new([row(12), row(13)]), 6, total)).unwrap();
    assert_eq!(exact.writes.len(), answer.writes.len());
    assert_eq!(exact.through, None);
    let empty = dispatch("change-parts", &page(Box::new([]), 0, 0)).unwrap();
    assert!(empty.writes.is_empty());
    assert_eq!(empty.through, None);
}

#[test]
fn change_parts_job_refuses_the_unreadable_key_and_a_row_that_cannot_fit() {
    let mut bad = row(12);
    bad.value = Box::new([0]);
    for input in
        [page(Box::new([bad]), 3, u32::MAX), page(Box::new([row(12)]), 2, u32::MAX), page(Box::new([row(12)]), 3, 0)]
    {
        let why = dispatch("change-parts", &input).unwrap_err();
        assert_eq!(why.key, Some(input.rows[0].key.clone()));
    }
    let mut wrong_key = row(12);
    wrong_key.key = keys::encode(&Key::Change(13), u32::MAX).unwrap();
    assert_eq!(
        dispatch(
            "change-parts",
            &page(Box::new([JobRow { key: wrong_key.key.clone(), value: wrong_key.value.clone() }]), 3, u32::MAX)
        )
        .unwrap_err()
        .key,
        Some(wrong_key.key)
    );
}

use skein_forge::api::{self as api, Repository};
use skein_lib::Wall;
use temper_engine_domain_forge_vocabulary as d;

pub(super) const JOB: &str = "deleted-repository-forge-rows";
pub(super) const A: Repository = Repository { forge: 1, repository: 7 };
pub(super) const B: Repository = Repository { forge: 1, repository: 20 };

pub(super) fn encoded(stored: &Stored) -> JobRow {
    JobRow { key: keys::encode(&d::stored_key(stored), u32::MAX).unwrap(), value: records::encode(stored) }
}
fn present() -> jobs::Present {
    jobs::Present { repositories: Box::new([A]), forges: Box::new([(1, false)]), last: 1 }
}
fn deleted(
    input: &JobPage,
    present: &jobs::Present,
) -> Result<jig_protocol_store::JobAnswer, jig_protocol_store::JobRefusal> {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    jobs::page(JOB, input, present, &bounds)
}
fn name(repository: Repository) -> d::Name {
    d::Name { forge: repository.forge, repository: repository.repository, what: d::What::Repository }
}
pub(super) fn application(repository: Repository) -> Stored {
    let Stored::Repository(adopted) = fixtures::adopted(d::Protection::Absent) else { panic!("adopted fixture") };
    Stored::Application {
        token: 1,
        until: Wall::EPOCH,
        outcome: d::ApplicationOutcome::Adopted(d::AdoptedRepository {
            repository,
            owner: adopted.owner,
            name: adopted.name,
            role: adopted.role,
            home: adopted.home,
            kinds: adopted.kinds,
            protection: adopted.protection,
        }),
    }
}
fn dropped_rows(repository: Repository) -> Vec<Stored> {
    let mut branch = name(repository);
    branch.what = d::What::Branch(Box::new([Box::from(b"main".as_slice())]));
    let mut pull = name(repository);
    pull.what = d::What::Pull(3);
    let mut live = fixtures::live(skein_forge_client::What::Pull(3));
    live.watch.resource.repository = repository;
    let mut tier = fixtures::tier_row();
    tier.repository = repository;
    vec![
        Stored::BranchHead(d::BranchHead { name: branch, commit: [1; 32], owned: true }),
        Stored::PullState(d::PullState { name: pull, head: [1; 32], ci: api::Ci::Passed, state: api::State::Closed }),
        Stored::Ci(d::CiState { repository, head: [1; 32], status: api::Ci::Passed }),
        Stored::Client(skein_forge_client::Stored::Live(live)),
        Stored::Receipt {
            key: d::ReceiptKey { project: 1, repository, check: d::Check::Gate(b"ci".as_slice().into()), key: [1; 32] },
            receipt: d::Receipt {
                logs: Box::new([]),
                failures: None,
                passed: true,
                run: d::ReceiptRun { id: 1, number: 1, url: Box::new([]) },
                candidate: [1; 32],
                ended: Wall::EPOCH,
            },
        },
        Stored::Tier(tier),
        Stored::Verification(d::Verification {
            repository,
            commit: [1; 32],
            changes: Box::new([]),
            tiers: Box::new([]),
        }),
        Stored::LandingLog(d::LandingLog { repository, landings: Box::new([]) }),
        application(repository),
    ]
}
pub(super) fn names(task: u64, resources: &[Repository]) -> Stored {
    Stored::Names { task, resources: resources.iter().copied().map(name).collect() }
}
fn input(rows: &[Stored]) -> JobPage {
    let mut rows: Vec<_> = rows.iter().map(encoded).collect();
    rows.sort_by(|a, b| a.key.cmp(&b.key));
    page(rows.into(), u32::MAX, u32::MAX)
}

#[test]
fn deleted_repository_job_drops_each_family_and_counts_names_resources() {
    let mut rows = dropped_rows(B);
    rows.extend([names(1, &[A, B]), names(2, &[B])]);
    let input = input(&rows);
    let answer = deleted(&input, &present()).unwrap();
    assert_eq!(answer.through, None);
    assert_eq!(answer.counts.as_ref(), [11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2]);
    assert_eq!(answer.writes.len(), 11);
    for (write, row) in answer.writes.iter().zip(&input.rows) {
        if row.key == encoded(&names(1, &[A, B])).key {
            let JobWrite::Put { key, value } = write else { panic!("partial Names put") };
            let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
            assert_eq!(records::row(key, value, &bounds), Ok(names(1, &[A])));
        } else {
            let JobWrite::Erase { key } = write else { panic!("orphan erased") };
            assert_eq!(key, &row.key);
        }
    }
}

#[test]
fn deleted_repository_job_keeps_present_repositories_and_unlinked_outcomes() {
    let mut rows = dropped_rows(A);
    rows.extend(refused_rows(A).into_iter().map(|(_, row)| row));
    rows.push(names(1, &[A]));
    rows.push(Stored::Application { token: 2, until: Wall::EPOCH, outcome: d::ApplicationOutcome::MakeKept });
    let answer = deleted(&input(&rows), &present()).unwrap();
    assert!(answer.writes.is_empty());
    assert_eq!(answer.counts.as_ref(), [0; 11]);
    assert_eq!(answer.through, None);
    let empty = deleted(&page(Box::new([]), 0, 0), &present()).unwrap();
    assert!(empty.writes.is_empty());
    assert_eq!(empty.counts.as_ref(), [0; 11]);
}

#[expect(clippy::too_many_lines, reason = "one fixture row per refused family, including a two-member landing")]
fn refused_rows(repository: Repository) -> Vec<(&'static str, Stored)> {
    use temper_engine_domain_forge_landing as l;
    let mut issue = fixtures::issue_row();
    issue.repository = repository;
    let mut change = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: false });
    change.repository = repository;
    let (cold, _) = change.split();
    let mut entry = fixtures::entry(api::Write::Close { number: 3 }, skein_forge_client::Condition::None);
    entry.repository = repository;
    vec![
        (
            "ProposedEffect",
            Stored::ProposedEffect(d::ProposedEffect { number: 1, resource: d::What::Pull(3), entry: entry.clone() }),
        ),
        (
            "Hold",
            Stored::Hold(d::Hold {
                name: name(repository),
                task: 1,
                writer: None,
                drift: None,
                design_writers: Box::new([]),
            }),
        ),
        (
            "Subscription",
            Stored::Subscription(d::Subscriber {
                task: 1,
                number: 1,
                topic: d::Topic::Pull { repository, number: 3 },
                own_change: None,
                goal_tasks: None,
                paths: Box::new([]),
            }),
        ),
        ("Entry", Stored::Entry { entry, held: true }),
        (
            "Client",
            Stored::Client(skein_forge_client::Stored::Repository(skein_forge_client::RepositoryRecord {
                repository,
                mark: None,
                clock: skein_lib::Time::ZERO,
            })),
        ),
        ("Issue", Stored::Issue(issue)),
        (
            "Release",
            Stored::Release(d::ReleaseRow {
                task: 1,
                root: 1,
                ending: d::ReleaseEnding::Done,
                close_pull: None,
                comment: None,
                pending: Some((1, Some(name(repository)))),
                failed: false,
                effects_settled: false,
                releasing: false,
            }),
        ),
        (
            "Landing",
            Stored::Landing(d::LandingRow {
                job: None,
                asks: Box::new([None, None]),
                entries: Box::new([]),
                landing: l::Landing {
                    last_left: None,
                    round: l::Round::new(b"landing".as_slice().into()),
                    task: 1,
                    project: 1,
                    phase: l::Phase::Waiting,
                    preparation: 0,
                    stale_proofs: 0,
                    branch: b"main".as_slice().into(),
                    members: [A, repository]
                        .into_iter()
                        .enumerate()
                        .map(|(at, repository)| l::Member {
                            round: None,
                            unit: u16::try_from(at).unwrap(),
                            branch: b"landing".as_slice().into(),
                            combined: None,
                            recorded: false,
                            told: false,
                            judging: false,
                            leave_told: false,
                            left: None,
                            change: 1,
                            repository: l::Repository { forge: repository.forge, repository: repository.repository },
                            pull: 3,
                            head: None,
                            base: None,
                            candidate: None,
                            gates: Box::new([]),
                            asked: None,
                            landed: None,
                        })
                        .collect(),
                },
            }),
        ),
        (
            "Follow",
            Stored::Follow(d::FollowRow {
                project: 1,
                follow: 1,
                source: repository,
                branch: b"main".as_slice().into(),
                heard: None,
                task: None,
                stopped: None,
                questions: Box::new([]),
                pending: None,
                queue: Box::new([]),
                skips: Box::new([]),
                parity: None,
            }),
        ),
        ("ChangeCold", Stored::ChangeCold(Box::new(cold))),
    ]
}

#[test]
fn deleted_repository_job_refuses_every_pending_family_with_its_key_and_repository() {
    for (family, row) in refused_rows(B) {
        let input = input(&[row]);
        let why = deleted(&input, &present()).unwrap_err();
        assert_eq!(why.key, Some(input.rows[0].key.clone()), "{family}");
        assert_eq!(&*why.why, format!("{family} names deleted repository 1/20"));
    }
}

#[test]
fn deleted_repository_job_keeps_ended_forge_history_but_not_client_live_or_names() {
    for facts in [
        jobs::Present { repositories: Box::new([A]), forges: Box::new([(1, true)]), last: 1 },
        jobs::Present { repositories: Box::new([A]), forges: Box::new([]), last: 1 },
    ] {
        let mut rows = dropped_rows(B);
        rows.extend(
            refused_rows(B)
                .into_iter()
                .filter(|(family, _)| ["Issue", "Landing", "Follow", "ChangeCold"].contains(family))
                .map(|(_, row)| row),
        );
        rows.push(names(1, &[B]));
        let answer = deleted(&input(&rows), &facts).unwrap();
        assert_eq!(answer.counts.as_ref(), [2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1]);
        assert_eq!(answer.writes.len(), 2);
    }
    for forge in [0, 2] {
        let facts = jobs::Present { repositories: Box::new([A]), forges: Box::new([]), last: 1 };
        let repository = Repository { forge, ..B };
        let rows = [dropped_rows(repository).remove(1), application(repository)];
        let answer = deleted(&input(&rows), &facts).unwrap();
        assert_eq!(answer.counts.as_ref(), [2, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0]);
    }
}

#[test]
fn deleted_repository_job_pages_whole_names_rows_within_write_and_byte_room() {
    let rows = [names(1, &[A, B]), names(2, &[B, B])];
    let mut input = input(&rows);
    let full = deleted(&input, &present()).unwrap();
    let first = full.writes[0].bytes();
    let total = first + full.writes[1].bytes();
    for (writes, bytes) in [(1, u64::MAX), (2, total - 1), (1, first)] {
        input.room = JobRoom { writes, bytes };
        let answer = deleted(&input, &present()).unwrap();
        assert_eq!(answer.through, Some(input.rows[0].key.clone()));
        assert_eq!(answer.writes.len(), 1);
        let JobWrite::Put { key, value } = &answer.writes[0] else { panic!("Names put") };
        let JobWrite::Put { key: expected_key, value: expected_value } = &full.writes[0] else { panic!("Names put") };
        assert_eq!((key, value), (expected_key, expected_value));
        assert_eq!(answer.counts.as_ref(), [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    }
    input.room = JobRoom { writes: 2, bytes: total };
    assert_eq!(deleted(&input, &present()).unwrap().counts.as_ref(), [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);
    for (writes, bytes) in [(0, u64::MAX), (1, first - 1)] {
        input.room = JobRoom { writes, bytes };
        let why = deleted(&input, &present()).unwrap_err();
        assert_eq!(why.key, Some(input.rows[0].key.clone()));
        assert_eq!(&*why.why, "row exceeds the deleted repository job edge");
    }
}

#[test]
fn deleted_repository_job_refuses_unreadable_rows_and_keys() {
    let mut rows = dropped_rows(B);
    rows.extend(refused_rows(B).into_iter().map(|(_, row)| row));
    rows.push(names(1, &[A, B]));
    for row in rows {
        let mut bad = encoded(&row);
        bad.value = Box::new([0]);
        let why = deleted(
            &page(Box::new([JobRow { key: bad.key.clone(), value: bad.value.clone() }]), 1, u32::MAX),
            &present(),
        )
        .unwrap_err();
        assert_eq!(why.key, Some(bad.key));
        assert!(why.why.starts_with("unreadable "), "{why:?}");
    }
    let mut row = encoded(&names(1, &[B]));
    for key in [Box::from([4]), encoded(&names(2, &[B])).key] {
        row.key = key.clone();
        assert_eq!(
            deleted(
                &page(Box::new([JobRow { key: row.key.clone(), value: row.value.clone() }]), 1, u32::MAX),
                &present()
            )
            .unwrap_err()
            .key,
            Some(key)
        );
    }
}
