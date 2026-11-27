use super::fixtures;
use crate::store::{self, Store};
use jig_domain_core::engine::store as engine_store;
use jig_protocol_store::{Row, core_v6::Record as _};
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::Time;
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;

#[test]
fn a_fresh_directory_opens_the_engine() {
    const PASSES: u32 = 256; // Calm store completions with the clock held fixed.
    let mut directory = fixtures::directory::Directory::new();
    let mut service = directory.service();
    for _ in 0..PASSES {
        directory.turn(&mut service);
        assert_eq!(service.end(), None, "fresh engine start");
        if service.engine.as_ref().expect("engine").domain.ready() {
            return;
        }
    }
    panic!("fresh engine did not reach readiness");
}

#[test]
fn settings_history_room_follows_effective_capacities() {
    let (mut root, _, mut profile) = fixtures::store_limits();
    // Force the settings envelope to bind instead of the profile's fixed value.
    profile.store.value = 1;
    for (charters, run_bytes) in [(16, 32_768), (16, 65_536), (17, 65_536)] {
        root.tasks.charters = charters;
        root.journal.run_bytes = run_bytes;
        let core = engine::core_limits(&root);
        let revision = jig_protocol_store::sizes::settings_value(&core).unwrap();
        let checked = store::layout(&root, &root.forge, &profile).expect("settings room follows capacities");
        assert_eq!(checked.store.value, revision);
        assert_eq!(checked.store.chunk, skein_kv::chunk_bytes(checked.store.key, revision).unwrap());
        assert!(checked.store.page.bytes >= checked.store.key + revision);
        assert_eq!(checked.store.budget, profile.store.budget);
        assert_eq!(checked.store.page.rows, profile.store.page.rows);
        assert_eq!(checked.store, checked.secrets);
    }
}

#[test]
fn task_values_past_the_codec_ceiling_refuse_before_the_provider_floor() {
    let (mut root, _, profile) = fixtures::store_limits();
    let settings = jig_protocol_store::sizes::settings_value(&engine::core_limits(&root)).unwrap();
    root.tasks.batch_bytes = settings.checked_mul(2).unwrap();
    assert_eq!(
        store::layout(&root, &root.forge, &profile),
        Err(store::Refusal::Map(jig_protocol_store::sizes::SizeRefusal::Ceiling))
    );
}

#[test]
fn the_store_opens_over_memory_and_answers_a_commit() {
    let mut store = fixtures::memory_store();
    fixtures::commit(&mut store, 1, Box::new([]));
    let Some(jig_protocol_store::Event::Committed { number: 1, .. }) = store.host.above.pop() else {
        panic!("first commit")
    };
    assert!(store.host.above.is_empty());
}

#[test]
fn a_reopen_checks_new_limits_only_when_admission_inputs_change() {
    let (root, _, profile) = fixtures::store_limits();
    let store = fixtures::memory_store_for(&root, &profile, &[]);
    let first = store.host.component.checked();
    let reopened = fixtures::reopened_store(store, &root);
    assert_eq!(reopened.host.component.checked(), first);

    let mut changed = root;
    changed.tasks.spec_bytes = root.tasks.spec_bytes.checked_add(1).unwrap();
    let expected = store::limits(&changed, &changed.forge, &profile).unwrap();
    assert_ne!(expected.limits(), first.limits());
    let reopened = fixtures::reopened_store(reopened, &changed);
    assert_eq!(reopened.host.component.checked(), expected);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the directory fixture supplies static schema texts and sets to real opens"
)]
fn prepared_binary_texts_keep_the_open_verdict() {
    use jig_protocol_store::{Event, OpenRefusal, Owner, OwnerSets};

    let mut prepared: Vec<skein_codec::schema::Schema> = Vec::new();
    for family in store::OWNER_SETS.sets.iter().flat_map(|set| set.families) {
        let frames = family.frames.as_ref().map_or(&[][..], |frames| frames.texts);
        for text in core::iter::once(family.current).chain(frames.iter().copied()) {
            if !prepared.iter().any(|schema| schema.text() == text.as_bytes()) {
                prepared.push(skein_codec::schema::parse(text.as_bytes()).expect("binary schema"));
            }
        }
    }
    let prepared = Box::leak(prepared.into_boxed_slice());

    // The application still has the same family and keys but an unmatched text
    // that cannot be admitted. Borrowing its binary schema instead would hide the refusal.
    let mut sets = store::OWNER_SETS.sets.to_vec();
    let application = sets.iter_mut().find(|set| set.owner == Owner::Application).expect("application set");
    let mut families = application.families.to_vec();
    families.first_mut().expect("application family").current = "invalid schema";
    application.families = Box::leak(families.into_boxed_slice());
    let refused = OwnerSets { sets: Box::leak(sets.into_boxed_slice()) };

    for (sets, expected) in
        [(store::OWNER_SETS, Ok(())), (refused, Err(OpenRefusal::Set { owner: Owner::Application }))]
    {
        for schemas in [None, Some(&*prepared)] {
            let settings = fixtures::start_settings(fixtures::engine(), &[(1, 100)]);
            let mut service =
                crate::Service::engine(settings, &fixtures::seeds(), None, fixtures::forge_runtime()).expect("engine");
            service.owner_sets(sets);
            if let Some(schemas) = schemas {
                service.prepared_schemas(schemas);
            }
            let event = fixtures::directory::Directory::new().open(&mut service.engine.as_mut().expect("engine").store);
            let verdict = if let Event::OpenRefused { why, .. } = event {
                Err(why)
            } else {
                let Event::Opened { .. } = event else { panic!("open terminal: {event:?}") };
                Ok(())
            };
            assert_eq!(verdict, expected, "prepared={}", schemas.is_some());
        }
    }
}

#[test]
fn sizes_past_skein_kvs_limits_refuse_the_store() {
    let (root, _, profile) = fixtures::store_limits();
    let admission = store::limits(&root, &root.forge, &profile).expect("small fits");
    let checked = admission.limits();
    let profile = jig_protocol_store::Limits { store: checked.store, ..profile };
    assert!(checked.store.is_usable());
    assert_eq!(checked.store, checked.secrets);
    assert_eq!(checked.core, engine::core_limits(&root));
    assert_eq!(
        checked.commits,
        jig_protocol_store::sizes::Commits { in_flight: root.journal.commits, writes: root.journal.writes }
    );
    for field in ["key", "ops", "commit", "page", "queue", "queued bytes"] {
        let mut short = profile;
        match field {
            "key" => short.store.key = checked.store.key.checked_sub(1).unwrap(),
            "ops" => short.store.ops = checked.store.ops.checked_sub(1).unwrap(),
            "commit" => short.store.commit = checked.store.commit.checked_sub(1).unwrap(),
            "page" => short.store.page.bytes = checked.store.page.bytes.checked_sub(1).unwrap(),
            "queue" => short.store.queued = root.journal.commits.checked_sub(1).unwrap(),
            "queued bytes" => {
                short.store.queued_bytes = u64::from(root.journal.commits)
                    .checked_mul(u64::from(checked.store.commit))
                    .unwrap()
                    .checked_sub(1)
                    .unwrap();
            }
            _ => panic!("named row"),
        }
        assert!(store::limits(&root, &root.forge, &short).is_none(), "{field}");
    }
}

#[test]
fn page_floor_admits_the_exact_map_bound_and_refuses_one_short() {
    let (mut root, _, mut profile) = fixtures::store_limits();
    root.loads.transcript_bytes = 8;
    let admission = store::limits(&root, &root.forge, &profile).expect("map-dominant page");
    let map = admission.limits();
    let map_bytes = map.store.key.checked_add(map.store.value).unwrap();
    assert_eq!(map.store.page.bytes, map_bytes);
    profile.store = map.store;
    let transcript = 8;
    let page = map_bytes;
    {
        root.loads.transcript_bytes = transcript;
        // Model startup's turn-plus-record and decoded read-row load bounds.
        root.loads.bytes = transcript.checked_add(u32::try_from(size_of::<engine::Record>()).unwrap()).unwrap();
        root.loads.read_bytes = u64::from(transcript)
            .checked_add(u64::try_from(size_of::<jig_domain_core::reads::Row>()).unwrap())
            .unwrap()
            .checked_add(u64::try_from(size_of::<jig_domain_core::Record>()).unwrap())
            .unwrap();
        profile.store.page.bytes = page;
        let admission = store::limits(&root, &root.forge, &profile).expect("exact page ceiling");
        let checked = admission.limits();
        assert_eq!(checked.store.page.bytes, page);
        assert_eq!(checked.store, checked.secrets);
        assert_eq!(checked.store.page.rows, profile.store.page.rows);
        assert_eq!(checked.store.chunk, map.store.chunk);
        assert_eq!(checked.store.key, map.store.key);
        assert_eq!(checked.store.value, map.store.value);
        assert_eq!(checked.reads, map.reads);
        assert_eq!(checked.core, engine::core_limits(&root));
        assert_eq!(checked.queue, map.queue);
        assert_eq!(checked.store.queued_bytes, map.store.queued_bytes);
        assert_eq!(checked.store.commit, map.store.commit);
        profile.store.page.bytes = page.checked_sub(1).unwrap();
        assert!(store::limits(&root, &root.forge, &profile).is_none());
    }
}

#[test]
fn shipped_maximum_frame_fits_small_page_and_out_of_codec_admission_refuses() {
    let (mut root, _, profile) = fixtures::store_limits();
    root.loads.transcript_bytes = jig_protocol_store::core_v6::CEILINGS.turn_record_transcript;
    // Model startup's turn-plus-record and decoded read-row load bounds.
    root.loads.bytes =
        root.loads.transcript_bytes.checked_add(u32::try_from(size_of::<engine::Record>()).unwrap()).unwrap();
    root.loads.read_bytes = u64::from(root.loads.transcript_bytes)
        .checked_add(u64::try_from(size_of::<jig_domain_core::reads::Row>()).unwrap())
        .unwrap()
        .checked_add(u64::try_from(size_of::<jig_domain_core::Record>()).unwrap())
        .unwrap();
    let admission = store::limits(&root, &root.forge, &profile).expect("shipped maximum fits Small");
    let checked = admission.limits();
    assert_eq!(checked.store.page.bytes, checked.store.key + checked.store.value);
    assert!(
        checked.store.page.bytes
            >= jig_protocol_store::sizes::frame(&checked.core, root.loads.transcript_bytes).unwrap()
    );
    root.loads.transcript_bytes = jig_protocol_store::core_v6::CEILINGS.turn_record_transcript + 1;
    assert!(store::limits(&root, &root.forge, &profile).is_none());
}

#[test]
fn a_forge_row_committed_is_restored_by_the_forge_interval() {
    let mut store = fixtures::memory_store();
    let row = fixtures::landed_row();
    fixtures::commit(
        &mut store,
        1,
        Box::new([engine::Write::Save(engine::Record::Forge { row: Box::new(row.clone()) })]),
    );
    let Some(jig_protocol_store::Event::Committed { number: 1, .. }) = store.host.above.pop() else {
        panic!("first commit")
    };
    let rows = fixtures::load(&mut store, engine::Range::Forge);
    assert_eq!(rows.as_ref(), [engine::Record::Forge { row: Box::new(row) }]);
}

#[test]
#[expect(clippy::disallowed_methods, reason = "test holds parsed owner texts for the store's static input")]
fn prepared_schemas_preserve_open_and_seeded_rows() {
    const FORGE: u16 = 1;
    let rows = [forge::Stored::Forge(fixtures::forge_record(FORGE)), forge::Stored::ForgeNumber { last: FORGE }];
    let mut seeded = fixtures::directory::Directory::new();
    let writes = rows.iter().cloned().map(|row| engine::Write::Save(engine::Record::Forge { row: Box::new(row) }));
    seeded.seed_with_sets(writes.collect(), None);

    let (root, _, profile) = fixtures::store_limits();
    let limits = store::limits(&root, &root.forge, &profile).expect("checked store limits");
    let schemas: &'static [_] = Box::leak(store::OWNER_SETS.prepare());
    for prepared in [None, Some(schemas)] {
        let mut disk = seeded.fork();
        let mut store = Store::new(&root, &limits, &[], &fixtures::seeds());
        if let Some(schemas) = prepared {
            store.prepared_schemas(schemas);
        }
        disk.open_limits(&mut store);
        for row in &rows {
            let key = forge::stored_key(row);
            let Some(Row::Application { key: found, value }) = disk.get(&mut store, &key) else {
                panic!("seeded row for {key:?}");
            };
            assert_eq!(found, temper_protocol_store::keys::encode(&key, u32::MAX).expect("fixture key"));
            assert_eq!(value, temper_protocol_store::records::encode(row));
        }
    }
}

#[test]
fn the_store_closes_after_its_drivers() {
    let mut store = fixtures::memory_store();
    let root = Fd::new(10);
    assert_eq!(store.adopt(root, 1000), Ok(()));
    assert_eq!(store.adopt(Fd::new(11), 1000), Err(Fd::new(11)));
    store.iterate();
    let openings = take_openings(&mut store, root);
    store.close();
    assert!(!store.closed());
    for (index, submit) in openings.into_iter().enumerate() {
        store.complete(Complete {
            op: submit.op,
            kind: submit.kind,
            result: Ok(Done::Fd(Fd::new(20_i32.checked_add(i32::try_from(index).unwrap()).unwrap()))),
        });
    }
    store.iterate();
    let closes = take_closes(&mut store, &[Fd::new(20), Fd::new(21)]);
    assert!(!store.closed());
    for submit in closes {
        store.complete(Complete { op: submit.op, kind: submit.kind, result: Ok(Done::Nothing) });
    }
    store.iterate();
    store.iterate();
    let submit = take_closes(&mut store, &[root]).into_iter().next().unwrap();
    assert!(!store.closed());
    store.complete(Complete { op: submit.op, kind: submit.kind, result: Ok(Done::Nothing) });
    store.iterate();
    assert!(store.closed());
    assert!(!store.work_pending(Time::ZERO));
    assert_eq!(store.host.next_deadline(), None);
}

fn take_openings(store: &mut Store, root: Fd) -> [Submit; 2] {
    let out = [store.submitted.pop().expect("secrets clone"), store.submitted.pop().expect("transcript clone")];
    for (index, submit) in out.iter().enumerate() {
        let Op::Open { root: found, path, .. } = &submit.kind else { panic!("directory open") };
        assert_eq!(*found, root);
        assert_eq!(path.as_ref(), b".");
        assert_eq!(submit.op.raw() >> 60_u32, 9_u64.checked_add(u64::try_from(index).unwrap()).unwrap());
        assert!(store::owns(submit.op));
    }
    assert!(store.submitted.is_empty());
    out
}

fn take_closes(store: &mut Store, fds: &[Fd]) -> Box<[Submit]> {
    assert_eq!(usize::try_from(store.submitted.len()).unwrap(), fds.len(), "closes for {fds:?}");
    let mut out = skein_lib::List::with_capacity(u32::try_from(fds.len()).unwrap());
    for fd in fds {
        let submit = store.submitted.pop().expect("close submission");
        let Op::Close { fd: found } = submit.kind else { panic!("close operation") };
        assert_eq!(found, *fd);
        assert!(store::owns(submit.op));
        out.push(submit).unwrap();
    }
    assert!(store.submitted.is_empty());
    out.into_boxed()
}

#[test]
fn a_writer_committed_is_reloaded_through_the_real_store() {
    let mut store = fixtures::memory_store();
    let row = temper_engine_domain_forge::Stored::Writer(skein_forge_client::Writer { forge: 1, author: 42 });
    fixtures::commit(
        &mut store,
        1,
        Box::new([engine::Write::Save(engine::Record::Forge { row: Box::new(row.clone()) })]),
    );
    let Some(jig_protocol_store::Event::Committed { number: 1, .. }) = store.host.above.pop() else {
        panic!("writer commit")
    };
    assert_eq!(
        fixtures::load(&mut store, engine::Range::Forge).as_ref(),
        [engine::Record::Forge { row: Box::new(row) }]
    );
}

#[test]
fn the_store_uses_the_roots_effective_core_limits() {
    let (root, parsed, profile) = fixtures::store_limits();
    let effective = engine::core_limits(&root);
    assert_ne!(parsed.load_slots, effective.load_slots);
    let admission = store::limits(&root, &root.forge, &profile).unwrap();
    let checked = admission.limits();
    assert_eq!(checked.core, effective);
    assert_eq!(checked.core.call_answer_bytes, parsed.call_answer_bytes);
}

#[test]
fn settings_history_loads_with_the_roots_effective_envelope() {
    use jig_domain_core::{CoreRecord, Record, reads, settings};
    use skein_lib::{Token, Wall};
    let (root, _, _) = fixtures::store_limits();
    let effective = engine::core_limits(&root);
    let mut store = fixtures::memory_store();
    let revision = settings::Revision {
        head: settings::Head {
            number: 1,
            against: 0,
            at: Wall::EPOCH,
            by: settings::Decider::FirstStart,
            proposal: None,
            moved: false,
        },
        items: Box::new([]),
        applications: Box::default(),
    };
    fixtures::commit(
        &mut store,
        1,
        Box::new([engine::Write::Save(engine::Record::Core(Record::Core(CoreRecord::Settings(
            settings::Record::Revision { history: false, revision: revision.clone() },
        ))))]),
    );
    let Some(jig_protocol_store::Event::Committed { .. }) = store.host.above.pop() else {
        panic!("settings history commit")
    };
    store.host.ask(jig_protocol_store::Request::Read {
        owner: Token::new(42),
        range: reads::Range {
            party: 1,
            scope: reads::Scope::SettingsHistory { first: 1, last: 1, newest: 1, whole: true },
            after: None,
            backwards: false,
            rows: 1,
            bytes: reads::settings::load_bytes(&effective, 1).unwrap(),
        },
    });
    let Some(jig_protocol_store::Event::Read {
        result: engine_store::Answer::Rows(engine_store::Rows::Read { rows, next: None }),
        ..
    }) = store.host.above.pop()
    else {
        panic!("settings history must answer a value, not Busy")
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(reads::settings::record_revision(&rows[0].value), Some(&revision));
}

#[test]
fn every_service_start_keeps_the_shells_wires() {
    use jig_protocol_store as protocol;
    use skein_lib::{Env, Queue, Reader, Token, Wall};
    let service = fixtures::ready();
    let (root, _, profile) = fixtures::store_limits();
    let admission = store::limits(&root, &root.forge, &profile).expect("fixture limits");
    let limits = admission.limits();
    let mut maps = service.engine.expect("started engine").store.host.component.into_stores();
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut maps.store,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.store },
        skein_kv::Request::Get {
            owner: Token::new(1),
            key: protocol::keys::encode_own(&protocol::keys::Own::Wires, limits.store.key).unwrap(),
        },
        &mut above,
        &mut below,
    );
    let Some(skein_kv::Event::Got { value: Some(bytes), .. }) = above.pop() else { panic!("durable wires row") };
    let wires =
        protocol::component_v3::Wires::decode(&protocol::component_v3::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    let spoken = crate::wires();
    assert!(spoken.len() <= 16, "the store's wire count bound");
    assert_eq!(wires.wires.len(), spoken.len());
    for (kept, expected) in wires.wires.iter().zip(&spoken) {
        assert!(expected.name.len() <= 32, "the store's wire name bound");
        assert_eq!(kept.name.as_ref(), expected.name.as_bytes());
        assert_eq!(kept.version, u32::from(expected.highest()));
    }
}

#[test]
fn stalled_file_publication_keeps_targets_and_cancels_within_operation_credit() {
    use skein_lib::{Queue, Wall};
    // Three file drivers can each retain a target and its cancellation.
    const OPERATIONS: u32 = 6;
    let mut store = fixtures::memory_store();
    store.submitted = Queue::with_capacity(OPERATIONS);
    store.adopt(Fd::new(10), 1000).unwrap();
    store.iterate();
    assert_eq!(store.submitted.len(), 2, "both directory opens wait for publication");
    let due = store.host.next_deadline().expect("file requests have deadlines");
    store.host.clock(due, Wall::EPOCH);
    store.iterate();
    assert_eq!(store.submitted.len(), 4, "each stalled open has one cancellation");
    store.iterate();
    assert_eq!(store.submitted.len(), 4, "the next pass adds no duplicate cancellation");
    let openings = [store.submitted.pop().unwrap(), store.submitted.pop().unwrap()];
    let mut cancels = [store.submitted.pop().unwrap(), store.submitted.pop().unwrap()].into_iter();
    for (index, submit) in openings.into_iter().enumerate() {
        let cancel = cancels.next().unwrap();
        let Op::Cancel { target } = cancel.kind else { panic!("file cancellation") };
        assert_eq!(target, submit.op, "cancellation follows its target in the same driver namespace");
        // A late successful open needs a cleanup close while its cancel still waits.
        store.complete(Complete {
            op: submit.op,
            kind: submit.kind,
            result: Ok(Done::Fd(Fd::new(20 + i32::try_from(index).unwrap()))),
        });
        assert_eq!(store.submitted.len(), u32::try_from(index + 1).unwrap());
        store.complete(Complete { op: cancel.op, kind: cancel.kind, result: Err(skein_io::kernel::Error::TooLate) });
    }
    assert_eq!(store.submitted.len(), 2, "cleanup successors fit the same operation credit");
    for _ in 0..store.submitted.len() {
        let submit = store.submitted.pop().unwrap();
        let Op::Close { .. } = submit.kind else { panic!("timed-out opens close their returned descriptors") };
        store.complete(Complete { op: submit.op, kind: submit.kind, result: Ok(Done::Nothing) });
    }
    store.iterate();
    assert!(store.submitted.is_empty(), "cleanup leaves no kernel work");
}
