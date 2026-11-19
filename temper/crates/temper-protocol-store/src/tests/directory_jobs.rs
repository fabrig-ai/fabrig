//! Current jobs run over a directory store (jig protocol/store/jobs.md, 2).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary Rust disk fixtures"
)]
use jig_domain_core_vocabulary::{CoreRecord, Deployment, Record};
use jig_protocol_store::{self as store, Event, Get, OwnerSet, OwnerSets, Request, Row, Store, Write};
use skein_lib::Token;
#[path = "directory_disk.rs"]
pub(super) mod disk;
use disk::Disk;

pub(super) const SETS: OwnerSets =
    OwnerSets { sets: &[store::SKEIN_SET, store::SMITH_SET, jig_protocol_accounts::JIG_SET, crate::OWNER_SET] };
pub(super) fn component(disk: &Disk, sets: OwnerSets) -> Store {
    let app = crate::application(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    let mut store = Store::on_directory(&disk.limits(), app, &[], disk.seeded());
    store.owner_sets(sets);
    store
}
pub(super) fn header(number: u64) -> Record {
    Record::Core(CoreRecord::Deployment(Deployment {
        id: [7; 16],
        restart_wait_ms: None,
        tasks: 1,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: 1,
        projects: 0,
        applications: 0,
        commits: number,
    }))
}
fn without_deleted_repository_job() -> OwnerSets {
    const APPLICATION: OwnerSet = OwnerSet { jobs: crate::OWNER_SET.jobs.split_at(4).0, ..crate::OWNER_SET };
    OwnerSets { sets: &[store::SKEIN_SET, store::SMITH_SET, jig_protocol_accounts::JIG_SET, APPLICATION] }
}

// The floor has neither the registered change parts nor person briefs. Keep every
// other current declaration and range, without leaking an owner set at each open.
const PARTS: usize = 3;
const fn after_floor(from: &[u8]) -> bool {
    from.len() == 1
        && (from[0] == crate::keys::CHANGE_COLD
            || from[0] == crate::keys::CHANGE_HOT
            || from[0] == crate::keys::PERSON_BRIEF)
}
const fn floor_types() -> [store::ValueType; crate::OWNER_SET.families[0].types.len() - PARTS] {
    let types = crate::OWNER_SET.families[0].types;
    let mut kept = [types[0]; crate::OWNER_SET.families[0].types.len() - PARTS];
    let mut read = 0;
    let mut written = 0;
    while read < types.len() {
        if !after_floor(types[read].keys[0].from) {
            kept[written] = types[read];
            written = written.checked_add(1).unwrap();
        }
        read = read.checked_add(1).unwrap();
    }
    assert!(written == kept.len());
    kept
}
const fn floor_keys() -> [store::Keys; crate::OWNER_SET.families[0].keys.len() - PARTS] {
    let keys = crate::OWNER_SET.families[0].keys;
    let mut kept = [keys[0]; crate::OWNER_SET.families[0].keys.len() - PARTS];
    let mut read = 0;
    let mut written = 0;
    while read < keys.len() {
        if !after_floor(keys[read].from) {
            kept[written] = keys[read];
            written = written.checked_add(1).unwrap();
        }
        read = read.checked_add(1).unwrap();
    }
    assert!(written == kept.len());
    kept
}
fn floor_sets() -> OwnerSets {
    const FAMILY: store::KeptFamily = store::KeptFamily {
        current: super::fixtures::layouts::FLOOR,
        types: &floor_types(),
        keys: &floor_keys(),
        ..crate::OWNER_SET.families[0]
    };
    const APPLICATION: OwnerSet =
        OwnerSet { jobs: &[], families: &[FAMILY, crate::OWNER_SET.families[1]], ..crate::OWNER_SET };
    OwnerSets { sets: &[store::SKEIN_SET, store::SMITH_SET, jig_protocol_accounts::JIG_SET, APPLICATION] }
}

fn seed_floor(disk: &mut Disk, rows: Vec<store::JobRow>) -> OwnerSets {
    let sets = floor_sets();
    let mut before = component(disk, sets);
    let events = disk.open(&mut before);
    let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("first open: {events:?}")
    };
    let events = disk.request(&mut before, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
    let [Event::Committed { .. }] = events.as_slice() else { panic!("commit: {events:?}") };
    let writes = core::iter::once(Write::Save(header(2)))
        .chain(rows.into_iter().map(|row| Write::Put { key: row.key, value: row.value }))
        .collect();
    let events = disk.request(&mut before, Request::Commit { number: 2, writes });
    let [Event::Committed { .. }] = events.as_slice() else { panic!("commit: {events:?}") };
    drop(before);
    disk.restart();
    sets
}

#[test]
fn the_four_id_jobs_open_floor_rows_before_deleted_repositories_are_read() {
    use super::{fixtures, test_ids};
    use temper_engine_domain_forge_vocabulary as d;
    let mut disk = Disk::new();
    let mut floor = fixtures::layouts::Floor::new();
    let (long, expected) = test_ids::long_tier(&mut floor);
    let short = [test_ids::landing(), test_ids::receipt_row(), test_ids::verification()];
    let mut rows: Vec<_> = short
        .iter()
        .zip(["landing", "receipt", "verification"])
        .map(|(stored, variant)| floor.row(stored, variant))
        .collect();
    rows.push(long);
    rows.push(floor.row(&fixtures::adopted(d::Protection::Absent), "repository"));
    seed_floor(&mut disk, rows);
    let mut current = component(&disk, SETS);
    let bounds = crate::bounds(&fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    let mut events = alloc::collections::VecDeque::from(disk.open(&mut current));
    let mut ran = Vec::new();
    let mut seen = Vec::new();
    let mut opened = false;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: store::Owner::Application, job, page } => {
                if !page.rows.is_empty() {
                    seen.push(job.clone());
                }
                let present = crate::jobs::read_present(&mut current, &bounds).unwrap();
                let answer = crate::jobs::page(&job, &page, &present, &bounds);
                events.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::Ran { of: store::Owner::Application, job, counts, .. } => {
                if job.as_ref() == "test-ids-tier" {
                    assert_eq!(counts.iter().map(|count| count.count).collect::<Vec<_>>(), [1, 1, 1]);
                } else {
                    assert!(counts.iter().all(|count| count.count == 0));
                }
                ran.push(job);
            }
            Event::Opened { pending: true, .. } => opened = true,
            Event::Census { .. } => {}
            other => panic!("floor open: {other:?}; {:?}", current.start_open_failure()),
        }
    }
    assert!(opened);
    assert_eq!(
        ran.iter().map(AsRef::as_ref).collect::<Vec<&str>>(),
        [
            "test-ids-landing",
            "test-ids-receipt",
            "test-ids-tier",
            "test-ids-verification",
            "deleted-repository-forge-rows"
        ]
    );
    // The floor predates landing search by test, so its layout is derived, not written.
    assert!(!seen.iter().any(|job| job.as_ref() == "test-ids-landing"));
    for row in short.into_iter().chain([d::Stored::Tier(expected)]) {
        let row = super::jobs::encoded(&row);
        let events =
            disk.request(&mut current, Request::Get { owner: Token::new(1), key: Get::Application(row.key.clone()) });
        let [Event::Got { row: Some(Row::Application { key, value }), .. }] = events.as_slice() else {
            panic!("readback: {events:?}")
        };
        assert_eq!((key, value), (&row.key, &row.value));
        crate::v25::Row::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(value)).unwrap();
    }
}

#[test]
fn an_in_flight_long_id_refuses_open_without_changing_the_active_store() {
    use super::{fixtures, test_ids};
    let mut disk = Disk::new();
    let mut tier = test_ids::tier();
    tier.record.run = fixtures::tier_row().record.run;
    let mut floor = fixtures::layouts::Floor::new();
    let mut row = floor.row(&temper_engine_domain_forge_vocabulary::Stored::Tier(tier), "tier");
    row.value = fixtures::layouts::replace_id(&row.value, b"t::a", &[b't'; 2000]);
    let key = row.key.clone();
    let original = row.value.clone();
    let sets = seed_floor(&mut disk, vec![row]);
    let mut current = component(&disk, SETS);
    let mut events = alloc::collections::VecDeque::from(disk.open(&mut current));
    let bounds = crate::bounds(&fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    let mut refused = false;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, job, page, .. } => {
                let answer = crate::jobs::page(&job, &page, &crate::jobs::Present::default(), &bounds);
                events.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::Ran { .. } => {}
            Event::OpenRefused { why: store::OpenRefusal::Job { job, key: bad, why, .. }, .. } => {
                assert_eq!(&*job, "test-ids-tier");
                assert_eq!(bad.as_ref(), Some(&key));
                assert_eq!(&*why, "a tier run in flight holds a failing test's id past 1,024 bytes");
                refused = true;
            }
            other => panic!("refused start: {other:?}"),
        }
    }
    assert!(refused);
    drop(current);
    disk.restart();
    let mut before = component(&disk, sets);
    let events = disk.open(&mut before);
    let [Event::Opened { last: 2, pending: false, .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("active store: {events:?}")
    };
    let events = disk.request(&mut before, Request::Get { owner: Token::new(1), key: Get::Application(key) });
    let [Event::Got { row: Some(Row::Application { value, .. }), .. }] = events.as_slice() else {
        panic!("old row: {events:?}")
    };
    assert_eq!(value.get(2..), original.get(2..), "the active value remains at the floor's encoding");
}

fn pull(repository: skein_forge::api::Repository, number: u64) -> temper_engine_domain_forge_vocabulary::Stored {
    use temper_engine_domain_forge_vocabulary::{Name, PullState, Stored, What};
    Stored::PullState(PullState {
        name: Name { forge: repository.forge, repository: repository.repository, what: What::Pull(number) },
        head: [1; 32],
        ci: skein_forge::api::Ci::Passed,
        state: skein_forge::api::State::Closed,
    })
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one open story checks pages, stored facts, readback and the durable job marker"
)]
fn deleted_repository_job_runs_over_directory_pages_once_and_preserves_present_and_ended_rows() {
    use super::jobs::{A, B, JOB, application, encoded, names};
    use skein_forge::api::Repository;
    use temper_engine_domain_forge_vocabulary as d;
    let mut disk = Disk::new();
    let mut before = component(&disk, without_deleted_repository_job());
    let events = disk.open(&mut before);
    let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("first open: {events:?}")
    };
    let orphans = disk.limits().store.ops + 3;
    let mut rows: Vec<_> = (0..u64::from(orphans)).map(|n| pull(B, n)).collect();
    rows.extend([application(B), names(1, &[A, B])]);
    let erased = rows[..usize::try_from(orphans + 1).unwrap()].to_vec();
    let d::Stored::Repository(mut repository) = super::fixtures::adopted(d::Protection::Absent) else {
        panic!("repository")
    };
    repository.provider = A;
    let mut kept_forge = super::fixtures::forge_record();
    kept_forge.number = 1;
    let mut ended_forge = kept_forge.clone();
    ended_forge.number = 2;
    ended_forge.state = d::ForgeState::Ended { secrets_erased: true, box_removed: true };
    let mut ended_application = application(Repository { forge: 3, ..B });
    let d::Stored::Application { token, .. } = &mut ended_application else { panic!("application") };
    *token = 2;
    let kept = [
        d::Stored::Repository(repository),
        d::Stored::Forge(kept_forge),
        d::Stored::Forge(ended_forge),
        d::Stored::ForgeNumber { last: 3 },
        pull(A, 1),
        pull(A, 2),
        pull(Repository { forge: 2, ..B }, 1),
        ended_application,
    ];
    rows.extend(kept.clone());
    // Commit the registry and the set's job markers before the fixture rows.
    let events = disk.request(&mut before, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
    let [Event::Committed { .. }] = events.as_slice() else { panic!("seed: {events:?}") };
    let mut number = 1;
    for chunk in rows.chunks(usize::try_from(disk.limits().store.ops / 2).unwrap()) {
        number += 1;
        let mut writes = vec![Write::Save(header(number))];
        for row in chunk {
            let row = encoded(row);
            writes.push(Write::Put { key: row.key, value: row.value });
        }
        let events = disk.request(&mut before, Request::Commit { number, writes: writes.into() });
        let [Event::Committed { .. }] = events.as_slice() else { panic!("plant: {events:?}") };
    }
    drop(before);
    disk.restart();
    let mut current = component(&disk, SETS);
    let bounds = crate::bounds(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    let mut events = alloc::collections::VecDeque::from(disk.open(&mut current));
    let mut pages = 0_u32;
    let mut ran = 0_u32;
    let mut opened = false;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: store::Owner::Application, job, page } => {
                assert_eq!(&*job, JOB);
                pages += 1;
                assert!(page.rows.len() <= usize::try_from(disk.limits().store.ops).unwrap());
                let present = crate::jobs::read_present(&mut current, &bounds).unwrap();
                assert_eq!(present.repositories.as_ref(), [A]);
                assert_eq!(present.forges.as_ref(), [(1, false), (2, true)]);
                assert_eq!(present.last, 3);
                let answer = crate::jobs::page(&job, &page, &present, &bounds);
                events.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::Ran { of: store::Owner::Application, job, counts, .. } => {
                assert_eq!(&*job, JOB);
                ran += 1;
                let names: Vec<_> = counts.iter().map(|c| &*c.name).collect();
                assert_eq!(names, crate::OWNER_SET.jobs[4].counts);
                let counts: Vec<_> = counts.iter().map(|c| c.count).collect();
                assert_eq!(counts, [u64::from(orphans + 2), 0, u64::from(orphans), 0, 0, 0, 0, 0, 0, 1, 1]);
            }
            Event::Opened { pending: true, .. } => opened = true,
            Event::Census { .. } => {}
            other => panic!("open: {other:?}; {:?}", current.start_open_failure()),
        }
    }
    assert!(opened);
    assert!(pages > 2, "more than one data page, then the empty end page");
    assert_eq!(ran, 1);
    for row in erased {
        let events =
            disk.request(&mut current, Request::Get { owner: Token::new(1), key: Get::Application(encoded(&row).key) });
        let [Event::Got { row: None, .. }] = events.as_slice() else { panic!("orphan gone: {events:?}") };
    }
    for row in kept.into_iter().chain([names(1, &[A])]) {
        let row = encoded(&row);
        let events =
            disk.request(&mut current, Request::Get { owner: Token::new(1), key: Get::Application(row.key.clone()) });
        let [Event::Got { row: Some(Row::Application { key, value }), .. }] = events.as_slice() else {
            panic!("kept: {events:?}")
        };
        assert_eq!((key, value), (&row.key, &row.value));
    }
    let events = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = events.as_slice() else { panic!("switch: {events:?}") };
    drop(current);
    disk.restart();
    let mut again = component(&disk, SETS);
    let events = disk.open(&mut again);
    let [Event::Opened { .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("second open has no Job or Ran: {events:?}")
    };
}

#[test]
fn deleted_repository_job_refuses_unreadable_stored_facts_by_key() {
    use super::jobs::{B, JOB, encoded};
    use temper_engine_domain_forge_vocabulary as d;
    let repository = encoded(&super::fixtures::adopted(d::Protection::Absent));
    let mut forge = super::fixtures::forge_record();
    forge.number = 2;
    let forge = encoded(&d::Stored::Forge(forge));
    let mut number = encoded(&d::Stored::ForgeNumber { last: 2 });
    number.value = [&number.value[..], &[0]].concat().into_boxed_slice();
    for (family, bad) in [
        ("Repository", store::JobRow { key: Box::new([2]), value: repository.value }),
        ("Forge", store::JobRow { key: crate::keys::encode(&d::Key::Forge(1), 1024).unwrap(), value: forge.value }),
        ("ForgeNumber", number),
    ] {
        let mut disk = Disk::new();
        let mut before = component(&disk, without_deleted_repository_job());
        let events = disk.open(&mut before);
        let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else {
            panic!("open: {events:?}")
        };
        let events =
            disk.request(&mut before, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
        let [Event::Committed { .. }] = events.as_slice() else { panic!("seed: {events:?}") };
        let orphan = encoded(&pull(B, 1));
        let events = disk.request(
            &mut before,
            Request::Commit {
                number: 2,
                writes: Box::new([
                    Write::Save(header(2)),
                    Write::Put { key: bad.key.clone(), value: bad.value },
                    Write::Put { key: orphan.key, value: orphan.value },
                ]),
            },
        );
        let [Event::Committed { .. }] = events.as_slice() else { panic!("plant: {events:?}") };
        drop(before);
        disk.restart();
        let mut current = component(&disk, SETS);
        let events = disk.open(&mut current);
        let [Event::Job { owner, of: store::Owner::Application, job, .. }] = events.as_slice() else {
            panic!("job: {events:?}")
        };
        assert_eq!(&**job, JOB);
        let bounds = crate::bounds(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
        let why = crate::jobs::read_present(&mut current, &bounds).unwrap_err();
        assert_eq!(why.key.as_ref(), Some(&bad.key), "{family}");
        assert_eq!(&*why.why, format!("unreadable {family} row"));
        let events = disk.request(&mut current, Request::Job { owner: *owner, answer: Err(why) });
        let [Event::OpenRefused { why: store::OpenRefusal::Job { owner: of, job, key, .. }, .. }] = events.as_slice()
        else {
            panic!("refusal: {events:?}")
        };
        assert_eq!(of, &store::Owner::Application);
        assert_eq!(&**job, JOB);
        assert_eq!(key.as_ref(), Some(&bad.key));
    }
}
