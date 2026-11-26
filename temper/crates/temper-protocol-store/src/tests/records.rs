use super::fixtures;
use crate::{records, v25};
use skein_codec::Record;
use temper_engine_domain_forge as forge;

#[test]
fn a_limit_hold_keeps_its_task_revision_figures_time_and_entrance() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let row = forge::Stored::LimitHold(forge::LimitHold {
        task: 7,
        member: b"forge.entries".as_slice().into(),
        configured: 8,
        needed: 9,
        revision: 13,
        at: skein_lib::Wall::from_nanos(42),
        order: 3,
        room: 1,
        set: forge::LimitSet::Entries,
    });
    let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
    let bytes = records::encode(&row);
    assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
    let wrong = crate::keys::encode(&forge::Key::LimitHold(8), u32::MAX).unwrap();
    assert_eq!(records::row(&wrong, &bytes, &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn a_changes_follows_round_trip_and_restore_with_the_configured_merge_bound() {
    use skein_lib::{Env, Queue, Time, Wall};
    use temper_engine_domain_forge_change::State;
    let mut row = fixtures::change_row(State::Producing { requested: false });
    let mut sha1 = [0_u8; 32];
    sha1[..20].fill(0xaa);
    let follows = forge::ChangeFollows { follow: 3, commits: Box::new([sha1, [0xbb; 32]]) };
    row.follows = Some(follows.clone());
    let task = row.task;
    let stored = fixtures::change_cold(row.clone());
    let hot = fixtures::change_hot(row);
    let mut limits = fixtures::LIMITS;
    limits.follow_merge = 2;
    let bounds = crate::bounds(&limits, limits.client.op_bytes).expect("two landings fit");
    let bytes = records::encode(&stored);
    let decoded = records::decode(&bytes, &bounds).expect("stored follows");
    assert_eq!(decoded, stored);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut restarted = forge::Domain::new(
        &limits,
        91,
        forge::Config { namespace: Box::new([7; 16]), writers: Box::new([]), push: None },
        &[],
        &[],
        false,
    )
    .unwrap();
    restarted.restore_begin(env.wall);
    forge::step(&mut restarted, &env, forge::Event::Restore { record: decoded }, &mut Queue::uncapped());
    let hot = records::decode(&records::encode(&hot), &bounds).unwrap();
    forge::step(&mut restarted, &env, forge::Event::Restore { record: hot }, &mut Queue::uncapped());
    restarted.fit_restored(&limits).unwrap();
    assert_eq!(restarted.change(task).unwrap().follows, Some(follows));
    limits.follow_merge = 1;
    let smaller = crate::bounds(&limits, limits.client.op_bytes).expect("smaller merge bound");
    assert_eq!(records::decode(&bytes, &smaller), Err(crate::Undecodable::Malformed));
}

#[test]
fn each_family_of_names_holds_and_topics_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics() {
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{name}");
    }
}
#[test]
fn a_holds_head_beside_and_waiting_write_round_trip() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let name = forge::Name { forge: 1, repository: 20, what: forge::What::Branch(Box::new([b"g".as_slice().into()])) };
    for waiting in [None, Some([9; 32])] {
        let row = forge::Stored::Hold(forge::Hold {
            name: name.clone(),
            task: 7,
            writer: None,
            design_writers: Box::new([]),
            drift: None,
            beside: Some(forge::HeadBeside { head: [8; 32], waiting }),
        });
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn a_landing_other_than_the_default_branch_round_trips() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let forge::Stored::Repository(mut row) = fixtures::adopted(forge::Protection::Absent) else {
        panic!("adopted repository")
    };
    row.landing = b"staging".as_slice().into();
    assert_eq!(row.settings.default_branch.as_ref(), b"main");
    let stored = forge::Stored::Repository(row);
    assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
}

#[test]
fn an_empty_stored_landing_reads_as_the_default_branch() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let forge::Stored::Repository(mut row) = fixtures::adopted(forge::Protection::Absent) else {
        panic!("adopted repository")
    };
    row.settings.default_branch = b"trunk".as_slice().into();
    row.landing = Box::new([]);
    let before = records::encode(&forge::Stored::Repository(row.clone()));
    row.landing = row.settings.default_branch.clone();
    let expected = forge::Stored::Repository(row);
    let decoded = records::decode(&before, &bounds).expect("empty landing defaults");
    assert_eq!(decoded, expected);
    let after = records::encode(&decoded);
    assert_ne!(after, before, "the default branch is now explicit");
    assert_eq!(records::decode(&after, &bounds), Ok(expected));
}

#[test]
fn a_repository_keeps_refusals_of_both_causes() {
    use skein_lib::Wall;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let forge::Stored::Repository(mut repository) = fixtures::adopted(forge::Protection::Absent) else {
        panic!("adopted repository")
    };
    repository.refusals = Box::new([
        forge::TrainRefusal {
            branch: Box::from(b"goal".as_slice()),
            since: Wall::from_nanos(33),
            cause: forge::FormingRefusal::Landings,
        },
        forge::TrainRefusal {
            branch: Box::from(b"main".as_slice()),
            since: Wall::from_nanos(48),
            cause: forge::FormingRefusal::Jig {
                number: 40,
                why: Box::from(b"Live".as_slice()),
                retry: Wall::from_nanos(1010),
            },
        },
    ]);
    let stored = forge::Stored::Repository(repository);
    assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
}

#[test]
fn a_landing_past_name_bytes_fails_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let forge::Stored::Repository(mut row) = fixtures::adopted(forge::Protection::Absent) else {
        panic!("adopted repository")
    };
    let length = usize::try_from(fixtures::LIMITS.name_bytes).unwrap().checked_add(1).unwrap();
    row.landing = skein_lib::bytes::zeroed(length);
    let bytes = records::encode(&forge::Stored::Repository(row));
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn a_holds_design_writers_and_their_fresh_read_progress_round_trip() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for rereading in [false, true] {
        let row = forge::Stored::Hold(forge::Hold {
            name: forge::Name {
                forge: 1,
                repository: 2,
                what: forge::What::Branch(Box::new([b"g1".as_slice().into()])),
            },
            task: 1,
            writer: None,
            beside: None,
            design_writers: Box::new([forge::DesignWriter { task: 1, attempt: 2, rereading }]),
            drift: None,
        });
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}

#[test]
fn a_truncated_row_fails_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics() {
        let bytes = records::encode(&row);
        for length in 0..bytes.len() {
            let prefix = bytes.get(..length).expect("prefix within row");
            assert_eq!(records::decode(prefix, &bounds), Err(crate::Undecodable::Malformed), "{name}, {length}");
        }
    }
}
#[test]
fn a_branch_past_name_bytes_fails_to_decode() {
    let mut segments = skein_lib::List::with_capacity(129);
    for _ in 0_u32..129 {
        segments.push(1_u8).expect("bounded bytes");
    }
    let row = forge::Stored::Hold(forge::Hold {
        beside: None,
        design_writers: Box::new([]),
        name: forge::Name { forge: 1, repository: 2, what: forge::What::Branch(Box::new([segments.into_boxed()])) },
        task: 1,
        writer: None,
        drift: None,
    });
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
}
#[test]
fn bounds_take_each_limit_from_its_source() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let limits = bounds.limits();
    for (name, value, expected) in [
        // Two four-byte lengths per segment, at most name_bytes empty segments.
        ("segments", limits.segments_segments, 1024),
        ("host", limits.adopted_host, 128),
        ("workflows", limits.adopted_workflows, 16),
        ("workflow name", limits.adopted_workflows_item, 255),
        ("resources", limits.names_resources, 4),
        ("goal tasks", limits.subscriber_goal_tasks_some, 8),
        ("open questions", limits.questions_open, 5),
    ] {
        assert_eq!(value, expected, "{name}");
    }
    assert_eq!(bounds.forge(), &fixtures::LIMITS);
}
#[test]
fn bounds_refuse_sources_past_schema_ceilings() {
    let mut larger = fixtures::LIMITS;
    larger.name_bytes = v25::CEILINGS.adopted_host.checked_add(1).expect("one past ceiling");
    assert_eq!(crate::bounds(&larger, fixtures::LIMITS.client.op_bytes), None);
}

#[test]
fn each_write_of_an_entry_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for (name, write) in fixtures::writes() {
        for condition in fixtures::conditions() {
            for held in [false, true] {
                let row = forge::Stored::Entry { entry: fixtures::entry(write.clone(), condition.clone()), held };
                let bytes = records::encode(&row);
                assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{name}, {condition:?}, {held}");
            }
        }
    }
}
#[test]
fn a_proposed_effect_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let row = forge::Stored::ProposedEffect(forge::ProposedEffect {
        number: 1,
        resource: forge::What::Branch(Box::new([Box::new([0_u8, 255])])),
        entry: fixtures::entry(
            skein_forge::api::Write::Merge { number: 2, head: [3; 32], how: skein_forge::api::MergeHow::Style },
            skein_forge_client::Condition::Merge { base: Box::new([4]) },
        ),
    });
    let bytes = records::encode(&row);
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}
#[test]
fn outbox_decoding_refuses_fields_past_their_bounds() {
    use skein_forge::api::Write;
    use skein_forge_client::Condition;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let rows = [
        ("reviewer count", Write::SetReviewers { number: 1, reviewers: Box::new([1; 9]) }),
        ("operation bytes", Write::Post { number: 1, key: Box::new([]), body: Box::new([1; 513]) }),
    ];
    for (name, write) in rows {
        let row = forge::Stored::Entry { entry: fixtures::entry(write, Condition::None), held: false };
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed), "{name}");
    }
}

#[test]
fn each_named_call_payload_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for payload in &fixtures::payloads() {
        let row = fixtures::named_call(payload.clone());
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
    for answer in fixtures::answers_without_item_pieces() {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}
#[test]
fn a_base_moved_outcome_round_trips_with_or_without_the_reported_commit() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for base in [Some([6; 32]), None] {
        let row = fixtures::named_call(forge::CallPayload::Effect {
            entry: 8,
            deadline: skein_lib::Wall::from_nanos(123),
            outcome: Some(skein_forge_client::Outcome::Failed(skein_forge::api::Error::BaseMoved { base })),
        });
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}

#[test]
fn error_tags_are_numbered_as_the_schema_lists_them() {
    for (error, tag) in fixtures::errors() {
        let bytes = fixtures::encoded(&records::encode_error(error));
        assert_eq!(bytes[0], tag, "{error:?}");
    }
}
#[test]
fn an_answer_piece_past_its_limit_fails_to_decode() {
    let answer = skein_forge::api::Answer::Branches(Box::new([Box::new([0_u8; 4125])]));
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
}
#[test]
fn a_full_answer_at_the_fixture_limits_fits_its_pieces() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for count in [8, 9, 40] {
        for extra in [0, 1] {
            let bytes = fixtures::LIMITS.client.answer_bytes + extra;
            let name_len = bytes / count - 16; // Each name has a 16-byte measure slot.
            let mut branches = skein_lib::List::with_capacity(count);
            for index in 0..count {
                let length = name_len + if index == 0 { bytes % count } else { 0 };
                branches.push(skein_lib::bytes::zeroed(usize::try_from(length).unwrap())).expect("fixture rows");
            }
            let answer = skein_forge::api::Answer::Branches(branches.into_boxed());
            let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
            let encoded = records::encode(&row);
            let decoded = records::decode(&encoded, &bounds);
            assert_eq!(decoded.is_ok(), extra == 0, "{count} names, {bytes} measured bytes");
            if extra == 0 {
                assert_eq!(decoded, Ok(row));
            }
        }
    }
}

#[test]
fn each_item_answer_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for answer in fixtures::item_answers() {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}
#[test]
fn malformed_answer_item_pieces_fail_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let cases: [(&str, Box<[u8]>); 3] = [
        ("unfinished length", Box::new([0, 0, 0])),
        ("item shorter than length", Box::new([0, 0, 0, 1])),
        ("empty summary", Box::new([0, 0, 0, 0])),
    ];
    for (name, piece) in cases {
        let row = fixtures::item_piece_row(piece);
        assert_eq!(
            records::decode(&fixtures::encoded_row(&row), &bounds),
            Err(crate::Undecodable::Malformed),
            "{name}"
        );
    }
}
#[test]
fn answer_item_counts_past_rows_fail_to_decode() {
    let mut items = skein_lib::List::with_capacity(9);
    for _ in 0_u32..9 {
        items.push(fixtures::summary()).expect("fixture room");
    }
    let answer = skein_forge::api::Answer::Items {
        cut: false,
        items: items.into_boxed(),
        more: false,
        now: skein_lib::Time::from_nanos(1),
    };
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn each_remaining_answer_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for answer in fixtures::remaining_answers() {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}
#[test]
fn listing_answers_preserve_cut_and_refuse_records_without_it() {
    use skein_forge::api::Answer;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for cut in [false, true] {
        for answer in [
            Answer::Reviews { reviews: Box::new([fixtures::review()]), more: true, cut },
            Answer::Remarks { remarks: Box::new([]), more: true, cut },
            Answer::PullFiles { head: [1; 32], files: Box::new([fixtures::file()]), more: true, cut },
        ] {
            let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
            let bytes = records::encode(&row);
            assert_eq!(records::decode(&bytes, &bounds), Ok(row));
            // A pre-cut named-call record ended at more: the cut byte is last.
            let old = bytes.get(..bytes.len().saturating_sub(1)).unwrap();
            assert_eq!(records::decode(old, &bounds), Err(crate::Undecodable::Malformed));
        }
    }
}

#[test]
fn fixed_answer_lists_past_their_counts_fail_to_decode() {
    use skein_forge::api::{Answer, Collaborator, Permission};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let mut collaborators = skein_lib::List::with_capacity(9);
    for user in 0_u64..9 {
        collaborators.push(Collaborator { user, permission: Permission::Write }).expect("fixture room");
    }
    let mut paths = skein_lib::List::with_capacity(9);
    for _ in 0u32..9 {
        paths.push(Box::from(b"x".as_slice())).unwrap();
    }
    let mut names = skein_lib::List::with_capacity(9);
    for _ in 0_u32..9 {
        names.push(Box::<[u8]>::default()).expect("fixture names");
    }
    let names = names.into_boxed();
    let mut item = fixtures::summary();
    item.labels = names.clone();
    let cases = [
        ("labels", Answer::Item { item, comments: Box::new([]), more: false }),
        (
            "contexts",
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
        ),
        ("collaborators", Answer::Collaborators { collaborators: collaborators.into_boxed(), more: true }),
        (
            "compared paths",
            Answer::Compare {
                before: [1; 32],
                after: [2; 32],
                files: paths.into_boxed(),
                ahead: 22,
                behind: 3,
                changed: 12,
            },
        ),
    ];
    for (name, answer) in cases {
        let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed), "{name}");
    }
}

#[test]
fn each_client_row_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for row in &fixtures::client_rows() {
        let bytes = records::encode(row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row.clone()));
    }
}
#[test]
fn client_list_counts_take_their_domain_bounds() {
    use skein_forge_client::{Delivery, Echo, Stored, What};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let mut echoes = fixtures::live(What::Issue(1));
    echoes.echoes = Box::new([Echo::Comment(1); 9]);
    let mut comments = fixtures::live(What::Issue(1));
    comments.comments = Box::new([Delivery { id: 1, revision: 1 }; 17]);
    let mut reviews = fixtures::live(What::Issue(1));
    reviews.reviews = Box::new([Delivery { id: 1, revision: 1 }; 17]);
    for (name, value) in [("echoes use rows", echoes), ("comments use inbox", comments), ("reviews use inbox", reviews)]
    {
        let row = forge::Stored::Client(Stored::Live(value));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed), "{name}");
    }
}
#[test]
fn a_cached_summary_key_uses_the_answer_byte_bound() {
    use skein_forge_client::{Stored, What};
    let mut live = fixtures::live(What::Issue(1));
    let mut summary = fixtures::summary();
    summary.key = Some(Box::new([1_u8; 600]));
    live.cached.item = Some(summary);
    let row = forge::Stored::Client(Stored::Live(live));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}

#[test]
fn a_change_in_each_state_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for state in &fixtures::change_states() {
        for repairs in [None, Some([28; 32])] {
            let mut row = fixtures::change_row(state.clone());
            row.repairs = repairs;
            for row in [fixtures::change_cold(row.clone()), fixtures::change_hot(row)] {
                let meter = skein_world::domain::heap::Meter::new();
                meter.start();
                let cloned = row.clone();
                let held = meter.end().held();
                assert_eq!(forge::stored_bytes(&cloned), Some(held));
                drop(cloned);
                let bytes = records::encode(&row);
                assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{state:?}");
            }
        }
    }
}
#[test]
fn a_changes_last_checked_head_round_trips_some_and_none() {
    use temper_engine_domain_forge_change::State;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for checked in [Some([29; 32]), None] {
        let mut row = fixtures::change_row(State::Producing { requested: true });
        row.change.last_checked = checked;
        row.change.last_checked_by = match checked {
            Some(_) => Some(41),
            None => None,
        };
        let row = fixtures::change_hot(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn a_changes_execution_clock_round_trips_for_each_last_reading() {
    use skein_lib::{Duration, Wall};
    use temper_engine_domain_forge_change::{Clock, Execution, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for seen in [Execution::Unknown, Execution::Waiting, Execution::Running] {
        let mut row = fixtures::change_row(State::Checking { head: [15; 32] });
        row.change.clock = Some(Clock {
            since: row.change.since,
            head: [15; 32],
            run: 7,
            attempt: 2,
            ran: Duration::from_nanos(13),
            waited: Duration::from_nanos(17),
            unknown: Duration::from_nanos(23),
            at: Wall::from_nanos(53),
            seen,
        });
        let row = fixtures::change_hot(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn review_revisions_round_trip_and_older_reports_default_to_zero() {
    use temper_engine_domain_forge_change::State;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let mut row = fixtures::change_row(State::Gating { head: [15; 32], asked: Box::new([21]) });
    row.spec_revision = 7;
    row.gate_revision = 5;
    row.verdicts[0].revision = u64::MAX;
    let stored = fixtures::change_hot(row.clone());
    assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored.clone()));
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family
        .current
        .replace("status: ChangeStatus revision: u64 = 0", "status: ChangeStatus")
        .replace(" spec_revision: u64 = 0 gate_revision: u64 = 0", "");
    assert_ne!(before, family.current);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let mut bytes = records::encode(&stored);
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", None).unwrap(),
            &[],
            target,
            target.root(b"Row", None).unwrap(),
            &[],
        )
        .unwrap();
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(131_072);
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    row.spec_revision = 0;
    row.gate_revision = 0;
    row.verdicts[0].revision = 0;
    assert_eq!(records::decode(&bytes, &bounds), Ok(fixtures::change_hot(row)));
}

/// Registered additions default during layout conversion, without a version move.
#[test]
fn a_change_written_without_a_checked_delegate_decodes_with_none() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family.current.replace(" last_checked_by: option u64 = none", "");
    assert_ne!(before, family.current);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
    row.change.last_checked = Some([29; 32]);
    row.change.last_checked_by = Some(41);
    let mut bytes = records::encode(&fixtures::change_hot(row.clone()));
    let mut work = skein_codec::layout::Workspace::new();
    // Removing the delegate and defaulting it to none cannot grow the populated row.
    let mut out = skein_lib::Buffer::with_capacity(u32::try_from(bytes.len()).unwrap());
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", Some(b"change_hot")).unwrap(),
            &[],
            target,
            target.root(b"Row", Some(b"change_hot")).unwrap(),
            &[],
        )
        .unwrap();
        assert!(work.fit(&plan));
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    row.change.last_checked_by = None;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    assert_eq!(records::decode(&bytes, &bounds), Ok(fixtures::change_hot(row)));
}

/// Registered additions default during layout conversion, without a version move.
#[test]
fn a_change_written_without_an_execution_clock_decodes_with_none() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family.current.replace(" clock: option ExecutionClock = none", "");
    assert_ne!(before, family.current);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let stored = fixtures::change_hot(fixtures::change_row(temper_engine_domain_forge_change::State::Checking {
        head: [15; 32],
    }));
    let mut bytes = records::encode(&stored);
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", None).unwrap(),
            &[],
            target,
            target.root(b"Row", None).unwrap(),
            &[],
        )
        .unwrap();
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(131_072);
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    assert_eq!(records::decode(&bytes, &bounds), Ok(stored));
}

#[test]
fn an_ended_changes_hold_round_trips_in_its_state() {
    use temper_engine_domain_forge_change::{Hold, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for ended_held in [None, Some(Hold::Repairs), Some(Hold::Resolutions), Some(Hold::Rejected), Some(Hold::Stalled)] {
        let mut row = fixtures::change_row(State::Ended);
        row.change.ended_held = ended_held;
        let row = fixtures::change_hot(row);
        let bytes = records::encode(&row);
        let wire = v25::Row::decode(&v25::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("encoded row");
        let v25::Value::ChangeHot(wire) = wire.value else { panic!("change row") };
        assert_eq!(wire.change.state.live, v25::ChangeLive::Ended);
        assert_eq!(wire.change.state.holds.len(), u32::from(ended_held.is_some()));
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}

#[test]
fn a_changes_role_choices_round_trip_with_or_without_models() {
    use jig_domain_tasks::{LlmChoice, ModelEffort};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let producing = LlmChoice {
        later: None,
        main: ModelEffort { model: 3, effort: Some(1) },
        subagents: Box::new([ModelEffort { model: 0, effort: Some(0) }, ModelEffort { model: 2, effort: None }]),
        account: Some(17),
    };
    let review = LlmChoice {
        later: None,
        main: ModelEffort { model: 4, effort: Some(2) },
        subagents: Box::new([ModelEffort { model: 1, effort: None }, ModelEffort { model: 3, effort: Some(1) }]),
        account: None,
    };
    for (producing_llm, review_llm) in [(None, None), (Some(producing), Some(review))] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: false });
        row.repairs = Some([28; 32]);
        row.producing_llm = producing_llm;
        row.review_llm = review_llm;
        let row = fixtures::change_cold(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}
#[test]
fn verifying_changes_keep_the_first_observation_at_wall_time_extremes() {
    use temper_engine_domain_forge_change::{Hold, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for nanos in [0, 123_456_789, u64::MAX] {
        let since = skein_lib::Wall::from_nanos(nanos);
        for state in [State::Landed { merge: [1; 32], since }, State::Red { merge: [2; 32], since }] {
            for saved in [state.clone(), State::Held { was: Box::new(state), why: Hold::Failed }] {
                let row = fixtures::change_hot(fixtures::change_row(saved));
                assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
            }
        }
    }
}

#[test]
fn terminal_change_ci_round_trips_with_or_without_an_observation() {
    use temper_engine_domain_forge_change::{Ci, State, Status};
    const HEAD: skein_forge::api::Commit = [15; 32]; // change_row's last_head.
    const MERGE: skein_forge::api::Commit = [1; 32];
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for state in [
        State::Landed { merge: MERGE, since: skein_lib::Wall::from_nanos(17) },
        State::Ended,
        State::NoChange { words: b"No widget UI exists".as_slice().into() },
    ] {
        for ci in [
            None,
            Some(Ci { head: HEAD, status: Status::Unknown }),
            Some(Ci { head: HEAD, status: Status::Pending }),
            Some(Ci { head: HEAD, status: Status::Passed }),
            Some(Ci { head: HEAD, status: Status::Failed }),
        ] {
            let mut change = fixtures::change_row(state.clone());
            change.ci = ci;
            let row = fixtures::change_hot(change);
            let bytes = records::encode(&row);
            assert_eq!(records::decode(&bytes, &bounds), Ok(row));
        }
    }
}
#[test]
fn a_negative_priority_round_trips_unchanged() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for priority in [-1_i32, i32::MIN, 0_i32, i32::MAX] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Landed {
            merge: [1; 32],
            since: skein_lib::Wall::from_nanos(17),
        });
        row.priority = priority;
        let row = fixtures::change_hot(row);
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{priority}");
    }
}
#[test]
fn eight_wrapping_holds_keep_their_order_at_the_schema_ceiling() {
    use temper_engine_domain_forge_change::{Hold, State};
    let mut state = State::Checking { head: [1; 32] };
    for why in [
        Hold::Stalled,
        Hold::Repairs,
        Hold::Resolutions,
        Hold::Updates,
        Hold::BranchMoved,
        Hold::BranchMissing,
        Hold::PullClosed,
        Hold::Retargeted,
    ] {
        state = State::Held { was: Box::new(state), why };
    }
    let row = fixtures::change_hot(fixtures::change_row(state));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}
#[test]
fn each_change_delegate_round_trips_unchanged() {
    use temper_engine_domain_forge_change::{Delegate, Repair, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for delegate in [
        Delegate::Produce,
        Delegate::Repair(Repair::Ci),
        Delegate::Repair(Repair::Gate(1)),
        Delegate::Repair(Repair::Semantic),
        Delegate::Repair(Repair::Landing { gate: 3 }),
        Delegate::Repair(Repair::Review { head: [7; 32] }),
        Delegate::Resolve { base: [2; 32] },
        Delegate::Gate { number: 3, head: [4; 32] },
    ] {
        let mut row = fixtures::change_row(State::Checking { head: [5; 32] });
        row.delegate = Some((6, delegate));
        let row = fixtures::change_hot(row);
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{delegate:?}");
    }
}

/// The merge delegate and its retained fields survive storage (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_change_row_with_a_merge_delegate_and_a_catch_up_round_trips() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for from in [Some([30; 32]), None] {
        let row = fixtures::merge_change_row(from);
        for row in [fixtures::change_cold(row.clone()), fixtures::change_hot(row)] {
            assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "from {from:?}");
        }
    }
}

#[test]
fn an_issue_row_with_its_desired_view_round_trips_unchanged() {
    use temper_engine_domain_forge_issues::{Phase, Projected};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for phase in
        [Phase::Waiting, Phase::Active, Phase::Settling, Phase::Held, Phase::Done, Phase::Failed, Phase::Cancelled]
    {
        let mut row = fixtures::issue_row();
        row.desired.as_mut().expect("full view").phase = phase;
        let row = forge::Stored::Issue(row);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "{phase:?}");
    }
    let mut without = fixtures::issue_row();
    without.before = None;
    without.desired = None;
    without.state = Projected::default();
    without.number = None;
    without.pending = None;
    without.failed = false;
    let row = forge::Stored::Issue(without);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "absent fields");
    let mut unfinished = fixtures::issue_row();
    unfinished.desired.as_mut().expect("full view").finished = None;
    let row = forge::Stored::Issue(unfinished);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "no final words");
}
#[test]
fn every_family_encodes_and_decodes_to_itself() {
    use skein_forge::api::Write;
    use skein_forge_client::{Condition, Stored, What};
    let entry = fixtures::entry(Write::Post { number: 1, key: Box::new([2]), body: Box::new([3]) }, Condition::None);
    let remaining = [
        ("call", fixtures::named_call(forge::CallPayload::Refused(skein_forge::api::Error::Busy))),
        (
            "proposal",
            forge::Stored::ProposedEffect(forge::ProposedEffect {
                number: 4,
                resource: forge::What::Issue(5),
                entry: entry.clone(),
            }),
        ),
        ("entry", forge::Stored::Entry { entry, held: true }),
        ("client", forge::Stored::Client(Stored::Live(fixtures::live(What::Issue(6))))),
        (
            "change",
            fixtures::change_hot(fixtures::change_row(temper_engine_domain_forge_change::State::Landed {
                merge: [7; 32],
                since: skein_lib::Wall::from_nanos(17),
            })),
        ),
        ("issue", forge::Stored::Issue(fixtures::issue_row())),
    ];
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for (name, row) in fixtures::names_holds_and_topics().into_iter().chain(remaining) {
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{name}");
    }
}
#[test]
fn issue_fields_use_their_domain_limits_at_and_past_each_bound() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for (name, field, limit) in [
        ("title", 0_u8, 128_usize),
        ("goal", 1, 512),
        ("plan item", 2, 512),
        ("milestone", 3, 256),
        ("finished", 4, 256),
    ] {
        for extra in [0_usize, 1] {
            let mut row = fixtures::issue_row();
            let view = row.desired.as_mut().expect("full view");
            let text = "x".repeat(limit + extra).into_boxed_str();
            match field {
                0 => view.title = text,
                1 => view.goal_text = text,
                2 => view.plan[0].text = text,
                3 => view.milestones[0].text = text,
                4 => view.finished = Some(text),
                _ => panic!("table field"),
            }
            let row = forge::Stored::Issue(row);
            let decoded = records::decode(&records::encode(&row), &bounds);
            if extra == 0 {
                assert_eq!(decoded, Ok(row), "{name} at limit");
            } else {
                assert_eq!(decoded, Err(crate::Undecodable::Malformed), "{name} past limit");
            }
        }
    }
}

fn application(outcome: forge::ApplicationOutcome) -> forge::Stored {
    forge::Stored::Application { token: u64::MAX, until: skein_lib::Wall::from_nanos(u64::MAX), outcome }
}
fn application_refusals() -> Box<[jig_domain_people::Refusal]> {
    use jig_domain_people as p;
    // The refusal fixtures include three ceiling cases under all three proposal verdicts.
    const REFUSAL_CASES: u32 = 202;
    let mut rows = skein_lib::List::with_capacity(REFUSAL_CASES);
    rows.push(p::Refusal::NameTaken).expect("bounded application cases");
    for rule in [p::NameRule::Empty, p::NameRule::Long, p::NameRule::Control, p::NameRule::Space, p::NameRule::Digits] {
        rows.push(p::Refusal::NotWellFormed { at: 0, why: p::Malformed::Name(rule) })
            .expect("bounded application cases");
    }
    for why in [p::AccountWhy::Unknown, p::AccountWhy::Provider, p::AccountWhy::Uses, p::AccountWhy::Limited] {
        rows.push(p::Refusal::Account { account: 43, why }).expect("account refusal rows fit");
    }
    for why in [
        p::AccountMoveWhy::NotSignedIn,
        p::AccountMoveWhy::NotOurs,
        p::AccountMoveWhy::Held,
        p::AccountMoveWhy::Cancelled,
        p::AccountMoveWhy::Moving,
    ] {
        rows.push(p::Refusal::AccountMove { why }).expect("account move refusal rows fit");
    }
    for row in [
        p::Refusal::Circle { tasks: Box::new([u64::MAX, 42]), cut: true },
        p::Refusal::Kept { pending: Box::new([42, u64::MAX]) },
        p::Refusal::Dependency { dependency: u64::MAX, how: p::DependencyStatus::Done },
        p::Refusal::Dependency { dependency: u64::MAX, how: p::DependencyStatus::Failed },
        p::Refusal::Dependency { dependency: u64::MAX, how: p::DependencyStatus::Cancelled },
        p::Refusal::Role { held: None },
        p::Refusal::Taken { by: u64::MAX },
        p::Refusal::NotWaiting,
        p::Refusal::NoFurther,
        p::Refusal::Unknown,
        p::Refusal::NotOffered,
        p::Refusal::KeyConflict,
    ] {
        rows.push(row).expect("refusal rows fit");
    }
    for held in
        [p::Role::Owner, p::Role::Maintainer, p::Role::Member, p::Role::Observer, p::Role::Policy { role: u32::MAX }]
    {
        rows.push(p::Refusal::Role { held: Some(held) }).expect("bounded application cases");
    }
    for lacking in [
        p::Lack::Ceiling {
            holder: p::CeilingHolder::Rules,
            dimension: p::CeilingDimension::PeriodSpend,
            value: Some(1000),
            asked: Some(1001),
        },
        p::Lack::Ceiling {
            holder: p::CeilingHolder::Policy,
            dimension: p::CeilingDimension::Tools,
            value: None,
            asked: None,
        },
        p::Lack::Ceiling {
            holder: p::CeilingHolder::Role(0),
            dimension: p::CeilingDimension::PeriodSpend,
            value: Some(500),
            asked: Some(499),
        },
        p::Lack::HoldOwner { hold: u64::MAX, owner: p::HoldOwner::Party { person: u64::MAX } },
        p::Lack::HoldOwner { hold: u64::MAX, owner: p::HoldOwner::Task { task: u64::MAX } },
        p::Lack::Approver,
        p::Lack::Budget { needs: u64::MAX, left: 42 },
        p::Lack::Grant { kind: u16::MAX },
        p::Lack::Funding,
        p::Lack::TaskKind { executor: None },
        p::Lack::Depth { most: u32::MAX },
        p::Lack::Deadline,
        p::Lack::Tasks { left: u32::MAX },
        p::Lack::Tool,
        p::Lack::Scope,
        p::Lack::Reference,
        p::Lack::ResourceAccess,
        p::Lack::ResourceUnavailable,
        p::Lack::Shape,
        p::Lack::Again { required: true },
        p::Lack::Account,
        p::Lack::Writer,
        p::Lack::Required { connector: u16::MAX, kind: 9 },
        p::Lack::Failed { connector: 8, kind: 7 },
        p::Lack::Unguarded { connector: 6, kind: 5 },
    ] {
        for proposable in [None, Some(p::Accepter::Nobody), Some(p::Accepter::Role(u32::MAX))] {
            rows.push(p::Refusal::Authority { lacking, proposable }).expect("bounded application cases");
        }
    }
    application_limit_refusals(&mut rows);
    application_refusal_lists(&mut rows);
    application_moved_and_ended(&mut rows);
    rows.into_boxed()
}
fn application_limit_refusals(rows: &mut skein_lib::List<jig_domain_people::Refusal>) {
    use jig_domain_people as p;
    for which in [
        p::LimitKind::Projects,
        p::LimitKind::PlacedHolds,
        p::LimitKind::ProjectPlacedHolds,
        p::LimitKind::PartyPlacedHolds,
        p::LimitKind::Exemptions,
        p::LimitKind::Words,
        p::LimitKind::Identity,
        p::LimitKind::Roles,
        p::LimitKind::Arithmetic,
        p::LimitKind::Duplicate,
        p::LimitKind::Empty,
        p::LimitKind::Batch,
        p::LimitKind::ProjectLiveTasks,
        p::LimitKind::LiveTasks,
        p::LimitKind::Tree,
        p::LimitKind::Depth,
        p::LimitKind::Delegates,
        p::LimitKind::Subscription,
        p::LimitKind::Dependencies,
        p::LimitKind::Cycle,
        p::LimitKind::Spec,
        p::LimitKind::Contract,
        p::LimitKind::Authority,
        p::LimitKind::Inputs,
        p::LimitKind::Attempts,
        p::LimitKind::LiveDelegates,
        p::LimitKind::Restore,
        p::LimitKind::Read,
        p::LimitKind::Turn,
        p::LimitKind::HoldKind,
        p::LimitKind::HoldTaken,
        p::LimitKind::Holds,
        p::LimitKind::InboxFull,
        p::LimitKind::Notes,
        p::LimitKind::Revision,
        p::LimitKind::Question,
        p::LimitKind::Priority,
        p::LimitKind::Policy,
        p::LimitKind::Resource,
        p::LimitKind::Service,
    ] {
        rows.push(p::Refusal::Limit { which }).expect("bounded application cases");
    }
}
fn application_refusal_lists(rows: &mut skein_lib::List<jig_domain_people::Refusal>) {
    use jig_domain_people as p;
    for (count, circle) in [
        (jig_protocol_store::people_v17::CEILINGS.refusal_circle_tasks, true),
        (jig_protocol_store::people_v17::CEILINGS.refusal_kept_pending, false),
    ] {
        let mut tasks = skein_lib::List::with_capacity(count);
        for task in 0..count {
            tasks.push(u64::from(task)).expect("ceiling-sized refusal list");
        }
        rows.push(if circle {
            p::Refusal::Circle { tasks: tasks.into_boxed(), cut: false }
        } else {
            p::Refusal::Kept { pending: tasks.into_boxed() }
        })
        .expect("bounded application cases");
    }
}
fn application_moved_and_ended(rows: &mut skein_lib::List<jig_domain_people::Refusal>) {
    use jig_domain_people as p;
    rows.push(p::Refusal::Moved { now: p::Moved::PlacedHold { revision: u64::MAX } })
        .expect("bounded application cases");
    for bound in [
        p::Bound::Reasoning,
        p::Bound::Metadata,
        p::Bound::Items,
        p::Bound::Answer,
        p::Bound::Tools,
        p::Bound::HistoryItems,
        p::Bound::History,
        p::Bound::Window,
        p::Bound::Output,
    ] {
        for value in [0, u64::MAX] {
            rows.push(p::Refusal::Moved {
                now: p::Moved::Hold {
                    revision: u64::MAX,
                    reason: p::HoldReason::Exceeded(p::Exceeded { bound, value }),
                },
            })
            .expect("bounded application cases");
        }
    }
    for reason in [
        p::HoldReason::Circle,
        p::HoldReason::Failures { class: p::FailureClass::Invalid },
        p::HoldReason::Stalled,
        p::HoldReason::Dependency { task: u64::MAX },
        p::HoldReason::Stopped,
        p::HoldReason::Placed { hold: u64::MAX },
        p::HoldReason::Drift,
        p::HoldReason::Effects,
        p::HoldReason::EffectFailed,
        p::HoldReason::Uncertain { entry: u64::MAX },
        p::HoldReason::Budget,
        p::HoldReason::Deadline,
        p::HoldReason::Waited,
        p::HoldReason::HoldsWaited,
        p::HoldReason::ResourceUnavailable,
        p::HoldReason::ResourceAccess,
        p::HoldReason::PoolLost { pool: u64::MAX },
        p::HoldReason::Procedure,
    ] {
        rows.push(p::Refusal::Moved { now: p::Moved::Hold { revision: u64::MAX, reason } })
            .expect("bounded application cases");
    }
    for class in [
        p::FailureClass::Transient,
        p::FailureClass::Permanent,
        p::FailureClass::Run,
        p::FailureClass::Agent,
        p::FailureClass::Lost,
        p::FailureClass::Infrastructure,
    ] {
        rows.push(p::Refusal::Moved { now: p::Moved::Hold { revision: 4, reason: p::HoldReason::Failures { class } } })
            .expect("bounded application cases");
    }
    for now in [
        p::Moved::Spec { revision: u64::MAX },
        p::Moved::Person { revision: u64::MAX },
        p::Moved::Note { revision: u32::MAX },
    ] {
        rows.push(p::Refusal::Moved { now }).expect("bounded application cases");
    }
    for how in [
        p::Ending::Done,
        p::Ending::Failed,
        p::Ending::Deleted,
        p::Ending::Decided { by: 7, at: skein_lib::Wall::from_nanos(8) },
    ] {
        rows.push(p::Refusal::Ended { how }).expect("bounded application cases");
    }
    for by in [p::Party::Person(u64::MAX), p::Party::Task(u64::MAX), p::Party::Deployment { project: u32::MAX }] {
        rows.push(p::Refusal::Ended { how: p::Ending::Cancelled { by, at: skein_lib::Wall::from_nanos(u64::MAX) } })
            .expect("bounded application cases");
    }
}
fn application_outcomes() -> Box<[forge::ApplicationOutcome]> {
    let mut outcomes = skein_lib::List::with_capacity(224);
    for refusal in application_refusals() {
        outcomes.push(forge::ApplicationOutcome::CoreRefused(refusal)).expect("bounded application cases");
    }
    for reason in [
        forge::AdoptionRefusal::NoSuchForge,
        forge::AdoptionRefusal::NotFound,
        forge::AdoptionRefusal::NoAccess,
        forge::AdoptionRefusal::Refused,
        forge::AdoptionRefusal::InFlight,
        forge::AdoptionRefusal::TooLarge,
        forge::AdoptionRefusal::Unreachable,
        forge::AdoptionRefusal::InvalidAnswer,
        forge::AdoptionRefusal::AnotherProject,
        forge::AdoptionRefusal::HomeHeld,
        forge::AdoptionRefusal::PrefixOccupied,
    ] {
        outcomes.push(forge::ApplicationOutcome::ForgeRefused(reason)).expect("bounded application cases");
    }
    for protection in [
        forge::Protection::Unknown,
        forge::Protection::Absent,
        forge::Protection::Rule(skein_forge::api::Protection {
            queue: false,
            signatures: false,
            linear: false,
            code_owners: false,
            bypass: skein_forge::api::Bypass::Never,

            branch: Box::from(b"main".as_slice()),
            contexts: Box::new([Box::from(b"check".as_slice())]),
            approvals: 2,
            dismiss_stale: true,
        }),
    ] {
        let forge::Stored::Repository(row) = fixtures::adopted(protection) else { panic!("repository fixture") };
        outcomes
            .push(forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
                landing: b"staging".as_slice().into(),
                repository: row.provider,
                owner: row.owner,
                name: row.name,
                role: row.role,
                home: row.home,
                kinds: row.kinds,
                protection: row.protection,
            }))
            .expect("bounded application cases");
    }
    outcomes.into_boxed()
}
#[test]
fn opened_application_outcomes_preserve_tasks_and_refusal_details() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let repositories = Box::new([
        forge::DraftRepository { owner: b"one".as_slice().into(), name: b"first".as_slice().into() },
        forge::DraftRepository { owner: b"two".as_slice().into(), name: b"second".as_slice().into() },
    ]);
    let outcomes = [
        forge::ApplicationOutcome::Opened { task: u64::MAX },
        forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::Several(repositories)),
        forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::NoRepository),
        forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::NotARepository),
        forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::UnderPrefix {
            owner: b"one".as_slice().into(),
            name: b"first".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
        }),
        forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::NamePast { bound: 1024 }),
    ];
    for outcome in outcomes {
        let row = application(outcome);
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
        assert_eq!(records::row(&key, &records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn every_application_outcome_preserves_its_complete_record() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for outcome in application_outcomes() {
        let row = application(outcome);
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).expect("key");
        let bytes = records::encode(&row);
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row.clone()), "{row:?}");
        assert_eq!(bytes[2], 14, "the Application family tag");
    }
    // Settings judgement words cannot occur in a forge application refusal.
    let limits =
        jig_protocol_store::people_v17::Limits { refusal_judged_why: 0, ..jig_protocol_store::people_v17::CEILINGS };
    assert!(
        jig_protocol_store::people_v17::Refusal::encoded_bound(&limits).expect("bounded facts")
            <= u64::from(v25::CEILINGS.core_refused_refusal)
    );
}
#[test]
fn appended_people_refusals_round_trip_in_application_answers() {
    use jig_domain_people::Refusal;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for refusal in [Refusal::NoTerminal, Refusal::LoginUnderWay, Refusal::NotLending] {
        let row = application(forge::ApplicationOutcome::CoreRefused(refusal));
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn application_decoding_refuses_malformed_refusal_facts() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for refusal in [Box::from([255]), Box::from([2]), Box::from([9, 0])] {
        let value = v25::CoreRefused { refusal };
        let application =
            v25::Application { token: 9, until: 10, outcome: v25::ApplicationOutcome::CoreRefused(value) };
        let row = v25::Row { value: v25::Value::Application(application) };
        assert_eq!(records::decode(&fixtures::encoded_row(&row), &bounds), Err(crate::Undecodable::Malformed));
    }
}

#[test]
fn an_application_adopted_name_past_its_bound_fails_to_decode() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let forge::Stored::Repository(row) = fixtures::adopted(forge::Protection::Absent) else {
        panic!("repository fixture")
    };
    let outcome = forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
        landing: b"main".as_slice().into(),
        repository: row.provider,
        owner: Box::new([0; 129]),
        name: row.name,
        role: row.role,
        home: row.home,
        kinds: row.kinds,
        protection: row.protection,
    });
    assert_eq!(records::decode(&records::encode(&application(outcome)), &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn a_job_steps_name_past_its_limit_fails_to_decode() {
    use skein_forge::api::{Answer, JobAttempt, Step};
    let answer = Answer::Job {
        attempt: JobAttempt { head: [1; 32], run: 7, job: 8, attempt: 2 },
        step: Some(Step { number: 1, name: Box::new([b'x'; 4097]) }),
        log: Box::new([]),
        truncated: false,
    };
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("Fixture limits fit");
    assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn forge_writers_round_trip_with_the_current_golden_bytes() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for (forge, author) in [(0, 1), (1, 0), (1, 42), (u16::MAX, u64::MAX)] {
        let row = forge::Stored::Writer(skein_forge_client::Writer { forge, author });
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).expect("writer key");
        let bytes = records::encode(&row);
        assert_eq!(bytes.len(), 13); // Row version, family, forge, author.
        assert_eq!(&bytes[..3], &[0, 25, 15]);
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
        if forge == u16::MAX {
            let mut golden = [255; 11];
            golden[0] = 15;
            assert_eq!(&bytes[2..], &golden);
        }
    }
    let zero = forge::Stored::Writer(skein_forge_client::Writer { forge: 1, author: 0 });
    let bytes = records::encode(&zero);
    let other = crate::keys::encode(&forge::Key::Writer(2), u32::MAX).unwrap();
    assert_eq!(records::row(&other, &bytes, &bounds), Err(crate::Undecodable::Malformed));
    assert_eq!(records::decode(&bytes[..bytes.len() - 1], &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn binding_rows_round_trip_at_the_field_bounds() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    // protocol/sign-in.md, 3.1 and 5: address <=254, subject <=255 bytes.
    let mut longest_address = [b'a'; 254];
    longest_address[252] = b'@';
    longest_address[253] = b'b';
    for (address, subject) in [
        (Box::from(b"a@b".as_slice()), Box::from(b"".as_slice())),
        (Box::from(b"sam@example.com".as_slice()), Box::from(b"google-subject".as_slice())),
        (Box::from(longest_address), Box::from([b's'; 255])),
    ] {
        let row = forge::Stored::Binding { address, subject };
        let bytes = records::encode(&row);
        assert_eq!(&bytes[..3], &[0, 25, 16]);
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
        let other =
            crate::keys::encode(&forge::Key::Binding(b"other@example.com".as_slice().into()), u32::MAX).unwrap();
        assert_eq!(records::row(&other, &bytes, &bounds), Err(crate::Undecodable::Malformed));
    }
}
#[test]
fn binding_decoder_preserves_bounded_noncanonical_addresses() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for address in [b"Sam@example.com".as_slice(), b"no-at", b"@b", b"a@@b", b"a@ b", b"", b"a\0b", &[0; 254]] {
        let row = forge::Stored::Binding { address: address.into(), subject: b"subject".as_slice().into() };
        let bytes = records::encode(&row);
        let key = crate::keys::encode(&forge::stored_key(&row), 511).unwrap();
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
        assert_eq!(records::decode(&bytes[..bytes.len() - 1], &bounds), Err(crate::Undecodable::Malformed));
        let mut trailing = skein_lib::Writer::new(bytes.len() + 1);
        trailing.put(&bytes).unwrap();
        trailing.put(&[0]).unwrap();
        assert_eq!(records::decode(&trailing.finish(), &bounds), Err(crate::Undecodable::Malformed));
    }
}
#[test]
fn binding_schema_refuses_fields_one_past_their_bounds() {
    use crate::v25 as w;
    for (address, subject) in [
        (Box::from([b'a'; 255]), b"subject".as_slice().into()),
        (b"sam@example.com".as_slice().into(), Box::from([b's'; 256])),
    ] {
        w::Binding { address, subject }.encode(&w::CEILINGS).unwrap_err();
    }
}

#[test]
fn embedded_people_refusals_changed_at_store_nine() {
    // The people v12 generated Unknown golden was [9]; v13 inserts NameTaken
    // before every existing refusal. All 38 existing Refusal goldens change.
    let after =
        jig_protocol_store::people_v17::Refusal::Unknown.encode(&jig_protocol_store::people_v17::CEILINGS).unwrap();
    assert_eq!(after.as_ref(), &[10]);
    assert_eq!(crate::STORE_VERSION, 25);
}

#[test]
fn release_comments_decode_only_within_the_deployment_byte_bound() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for (bytes, fits) in [(fixtures::LIMITS.client.op_bytes, true), (fixtures::LIMITS.client.op_bytes + 1, false)] {
        let mut comment = skein_lib::List::with_capacity(bytes);
        for _ in 0..bytes {
            comment.push(b'x').expect("fixture bytes");
        }
        let row = forge::Stored::Release(forge::ReleaseRow {
            room: None,
            task: 1,
            root: 1,
            ending: forge::ReleaseEnding::Cancelled,
            close_pull: Some(3),
            comment: Some(comment.into_boxed()),
            pending: None,
            failed: false,
            effects_settled: true,
            releasing: true,
        });
        assert_eq!(records::decode(&records::encode(&row), &bounds).is_ok(), fits, "{bytes}");
    }
}

#[test]
fn release_room_places_round_trip() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for room in [None, Some(1), Some(u64::MAX)] {
        let row = forge::Stored::Release(forge::ReleaseRow {
            room,
            task: 1,
            root: 1,
            ending: forge::ReleaseEnding::Cancelled,
            close_pull: Some(3),
            comment: Some(Box::from(b"cancelled".as_slice())),
            pending: None,
            failed: false,
            effects_settled: true,
            releasing: true,
        });
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn flight_rows_keep_both_key_fields_and_refuse_a_disagreeing_key() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for (task, entry) in [(0, 0), (7, 11), (u64::MAX, u64::MAX)] {
        let row = forge::Stored::Flight { task, entry };
        let bytes = records::encode(&row);
        let key = crate::keys::encode(&forge::Key::Flight { task, entry }, u32::MAX).expect("fixed key");
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
        let other = crate::keys::encode(&forge::Key::Flight { task: task ^ 1, entry }, u32::MAX).expect("fixed key");
        assert_eq!(records::row(&other, &bytes, &bounds), Err(crate::Undecodable::Malformed));
    }
}

#[test]
fn review_failure_counts_take_the_gate_bound() {
    use temper_engine_domain_forge_change::{GateFailure, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for count in [fixtures::LIMITS.change_policy.gates, fixtures::LIMITS.change_policy.gates + 1] {
        let mut row = fixtures::change_row(State::Checking { head: [1; 32] });
        let mut failures = skein_lib::List::with_capacity(count);
        for number in 0..count {
            failures.push(GateFailure { number: u64::from(number), count: 2 }).expect("named count");
        }
        row.change.gate_failures = failures.into_boxed();
        let row = fixtures::change_hot(row);
        let bytes = records::encode(&row);
        if count == fixtures::LIMITS.change_policy.gates {
            assert_eq!(records::decode(&bytes, &bounds), Ok(row));
        } else {
            assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
        }
    }
}

#[test]
fn change_record_words_use_the_task_bound_instead_of_operation_bytes() {
    const SPEC_BYTES: u32 = 2048;
    let mut change = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: false });
    change.title = Box::new([b't'; 256]);
    change.body = Box::new([b'x'; 2048]);
    let row = fixtures::change_cold(change);
    let bytes = records::encode(&row);
    let bounds = crate::bounds(&fixtures::LIMITS, SPEC_BYTES).unwrap();
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    let smaller = crate::bounds(&fixtures::LIMITS, SPEC_BYTES - 1).unwrap();
    assert_eq!(records::decode(&bytes, &smaller), Err(crate::Undecodable::Malformed));
}

#[test]
fn an_authority_hold_keeps_its_cause_and_release_across_restart() {
    use temper_engine_domain_forge_change::{AuthorityHold, AuthorityLack, AuthorityRecovery, Hold, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for lack in [AuthorityLack::Tasks, AuthorityLack::Depth, AuthorityLack::Spend] {
        for released in [false, true] {
            let why = Hold::Authority(AuthorityHold {
                recovery: AuthorityRecovery { task: u64::MAX, spec: u64::MAX, hold: u64::MAX, tasks: u64::MAX },
                lack,
                had: u64::MAX,
                released,
            });
            let mut row = fixtures::change_row(State::Held {
                was: Box::new(State::Gating { head: [1; 32], asked: Box::new([]) }),
                why,
            });
            row.change.released = Some(why);
            let row = fixtures::change_hot(row);
            assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        }
    }
}

#[test]
fn a_changes_result_evidence_survives_restart_with_its_cut_counts() {
    let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Landed {
        merge: [1; 32],
        since: skein_lib::Wall::from_nanos(17),
    });
    row.result = forge::ChangeResult {
        head: Some([3; 32]),
        checks: forge::ResultText { bytes: b"check passed".as_slice().into(), cut: 0 },
        failed: forge::ResultText { bytes: b"check".as_slice().into(), cut: 0 },
        repairs: forge::ResultText { bytes: b"CI failed: check".as_slice().into(), cut: 31 },
        repair_count: 8,
        reviews: forge::ResultText { bytes: b"failed at 123".as_slice().into(), cut: 77 },
    };
    let row = fixtures::change_hot(row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

#[test]
fn a_directory_answer_round_trips_with_its_entries() {
    use skein_forge::api::{Answer, Entry, EntryKind};
    let mut limits = fixtures::LIMITS;
    limits.client.rows = 1; // Directory entries use the existing byte budget alone.
    let bounds = crate::bounds(&limits, limits.client.op_bytes).expect("fixture bounds");
    for path in [b"".as_slice(), b"src"] {
        for truncated in [false, true] {
            let answer = Answer::Directory {
                head: [15; 32],
                path: path.into(),
                entries: Box::new([
                    Entry { name: b"main.rs".as_slice().into(), kind: EntryKind::File },
                    Entry { name: b"lib".as_slice().into(), kind: EntryKind::Directory },
                    Entry { name: b"vendor".as_slice().into(), kind: EntryKind::Submodule },
                ]),
                truncated,
            };
            let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(answer))));
            assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        }
    }
}

#[test]
fn a_turn_change_ready_or_preparing_round_trips_with_its_landing() {
    use temper_engine_domain_forge_change::{State, Way};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for state in [
        State::Ready { head: [1; 32], ready_since: skein_lib::Wall::from_nanos(2) },
        State::Preparing { from: [1; 32], main: [3; 32], ready_since: skein_lib::Wall::from_nanos(2) },
    ] {
        for landing in [None, Some(8)] {
            let mut row = fixtures::change_row(state.clone());
            row.change.way = Way::Turn;
            if landing.is_some() {
                row.previous = Some(forge::PreviousLink { landing, ask: None });
            }
            let row = fixtures::change_hot(row);
            let bytes = records::encode(&row);
            assert_eq!(records::decode(&bytes, &bounds), Ok(row));
        }
    }
}

#[test]
fn a_change_round_trips_its_previous_link_and_ask() {
    use temper_engine_domain_forge_change::{Ask, Below, Landed, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let mut row = fixtures::change_row(State::Ready { head: [1; 32], ready_since: skein_lib::Wall::from_nanos(2) });
    row.previous = Some(forge::PreviousLink {
        landing: Some(8),
        ask: Some(Ask::Repair {
            gate: 3,
            candidate: [4; 32],
            below: Some(Below {
                green: [5; 32],
                changes: [Some(Landed { change: 6, pull: 7 }), None, None, None],
                by_test: true,
                top: false,
            }),
        }),
    });
    let stored = fixtures::change_hot(row);
    assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
}

#[test]
fn a_change_ask_queue_round_trips_and_defaults_at_the_previous_layout() {
    use temper_engine_domain_forge_change::{Ask, Handed, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let mut row = fixtures::change_row(State::Ready { head: [1; 32], ready_since: skein_lib::Wall::from_nanos(2) });
    row.asks = Box::new([
        Handed { number: 2, landing: 8, round: 1, ask: Ask::Resolve { base: [2; 32] } },
        Handed { number: 4, landing: 8, round: 2, ask: Ask::Landed { prefix: [3; 32] } },
    ]);
    row.handed = 4;
    let stored = fixtures::change_hot(row.clone());
    let mut bytes = records::encode(&stored);
    assert_eq!(records::decode(&bytes, &bounds), Ok(stored));

    // The layout at the base has both legacy fields but not the queue or its count.
    let current = crate::OWNER_SET.families.first().expect("temper-store family").current;
    let before = current
        .replace("record LandingHanded { number: u32 landing: u64 round: u16 ask: LandingAsk }\n", "")
        .replace(" asks: list 16 LandingHanded = [] handed: u32 = 0", "");
    assert_ne!(current, before);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(current.as_bytes()).unwrap();
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", None).unwrap(),
            &[],
            target,
            target.root(b"Row", None).unwrap(),
            &[],
        )
        .unwrap();
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(131_072);
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    row.asks = Box::new([]);
    row.handed = 0;
    assert_eq!(records::decode(&bytes, &bounds), Ok(fixtures::change_hot(row)));
}

#[test]
fn a_change_stored_before_handed_over_asks_reads_with_none() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family.current.replace(" landing_ask: option LandingAsk = none", "");
    assert_ne!(before, family.current);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Ready {
        head: [1; 32],
        ready_since: skein_lib::Wall::from_nanos(2),
    });
    row.previous = Some(forge::PreviousLink {
        landing: None,
        ask: Some(temper_engine_domain_forge_change::Ask::Landed { prefix: [3; 32] }),
    });
    let mut bytes = records::encode(&fixtures::change_hot(row.clone()));
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", None).unwrap(),
            &[],
            target,
            target.root(b"Row", None).unwrap(),
            &[],
        )
        .unwrap();
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(131_072);
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    row.previous = None;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    assert_eq!(records::decode(&bytes, &bounds), Ok(fixtures::change_hot(row)));
}

#[test]
fn turn_writes_round_trip_with_their_parameters() {
    use skein_forge::api as a;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for write in [
        a::Write::Merge { number: 7, head: [3; 32], how: a::MergeHow::FastForwardOnly },
        a::Write::Merge { number: 7, head: [3; 32], how: a::MergeHow::Recorded { commit: [0xc1; 32] } },
        a::Write::Update { number: 7, style: a::UpdateStyle::Rebase },
        a::Write::Status {
            commit: [3; 32],
            context: Box::from(b"temper/gate/gate".as_slice()),
            check: a::Check::Passed,
            description: Box::from(b"key 0123456789abcdef passed".as_slice()),
            link: Box::from(b"https://forge/run/7".as_slice()),
        },
        a::Write::Dispatch {
            workflow: Box::from(b"gate.yml".as_slice()),
            branch: Box::from(b"temper/51".as_slice()),
            inputs: Box::new([a::Input {
                name: Box::from(b"temper_key".as_slice()),
                value: Box::from(b"0123456789abcdef".as_slice()),
            }]),
        },
        a::Write::Rerun { attempt: a::JobAttempt { head: [3; 32], run: 7, job: 8, attempt: 2 } },
    ] {
        let row =
            forge::Stored::Entry { entry: fixtures::entry(write, skein_forge_client::Condition::None), held: false };
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn landing_rows_preserve_phases_proofs_asks_and_effects() {
    use skein_lib::Wall;
    use temper_engine_domain_forge_change as c;
    use temper_engine_domain_forge_landing as l;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let at = Wall::from_nanos(17);
    for phase in [
        l::Phase::Waiting,
        l::Phase::Preparing,
        l::Phase::Proving,
        l::Phase::Proved { at },
        l::Phase::Turn { at },
        l::Phase::Landed,
        l::Phase::Cancelled,
        l::Phase::Held { was: Box::new(l::Phase::Proving), why: l::Hold::Faults { member: 0, gate: 0 } },
    ] {
        for proof in [
            l::Proof::Unkeyed,
            l::Proof::Keyed { key: [3; 32] },
            l::Proof::Dispatched { key: [3; 32], try_: 2 },
            l::Proof::Passed { key: [3; 32] },
            l::Proof::Failed { key: [3; 32] },
        ] {
            for ask in [
                None,
                Some(c::Ask::Prepare { head: [1; 32], main: [2; 32] }),
                Some(c::Ask::Repair { gate: 0, candidate: [1; 32], below: None }),
            ] {
                let row = forge::Stored::Landing(forge::LandingRow {
                    read_hold: None,
                    last_event: None,
                    turn_since: None,
                    waiting: None,
                    follows: None,
                    flake_reruns: Box::new([]),
                    room: Some(11),
                    job: Some(forge::history::JobRecord {
                        step: forge::history::JobStep {
                            task: 71,
                            attempt: 2,
                            job: forge::history::Job::Prepare,
                            outcome: Some(forge::history::JobOutcome::Done),
                            head: None,
                        },
                        round: 1,
                        members: Box::new([0]),
                        head: None,
                        written: false,
                    }),
                    landing: l::Landing {
                        last_left: Some(0),
                        searched: None,
                        round: temper_engine_domain_forge_landing::Round::new(Box::from(b"temper/1/l61".as_slice())),

                        task: 61,
                        project: 5,
                        phase: phase.clone(),
                        members: Box::new([l::Member {
                            round: Some(1),
                            unit: 0,
                            branch: Box::from(b"temper/1/l61".as_slice()),
                            combined: None,
                            recorded: false,
                            told: false,
                            judging: false,
                            leave_told: false,
                            left: None,
                            change: 51,
                            repository: l::Repository { forge: 1, repository: 2 },
                            pull: 7,
                            head: Some([1; 32]),
                            base: Some([2; 32]),
                            candidate: Some([1; 32]),
                            gates: Box::new([l::GateProof { proof, posted: l::Posted::Pending, faults: 2 }]),
                            asked: Some(1),
                            landed: None,
                        }]),
                        preparation: 1,
                        stale_proofs: 3,
                        last_prepare: None,
                        branch: Box::from(b"temper/g1".as_slice()),
                    },
                    previous_asks: Box::new([ask]),
                    entries: Box::new([forge::LandingEntry {
                        run: Some(forge::LandingRun { id: 23, attempt: 2, ended: true, clock: None }),
                        preparation: Some(2),
                        base: Some([5; 32]),
                        entry: 7,
                        member: 0,
                        gate: 0,
                        operation: forge::Operation::Dispatch,
                    }]),
                });
                let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).expect("landing key");
                assert_eq!(key.as_ref(), &[23, 0, 0, 0, 0, 0, 0, 0, 61]);
                assert_eq!(records::row(&key, &records::encode(&row), &bounds), Ok(row));
            }
        }
    }
}
fn landing_with_clock(clock: Option<temper_engine_domain_forge_change::Clock>) -> forge::Stored {
    use temper_engine_domain_forge_landing as l;
    forge::Stored::Landing(forge::LandingRow {
        read_hold: None,
        last_event: None,
        turn_since: None,
        waiting: None,
        follows: None,
        flake_reruns: Box::new([]),
        room: None,
        job: None,
        landing: l::Landing {
            task: 61,
            project: 5,
            phase: l::Phase::Proving,
            members: Box::new([]),
            preparation: 1,
            stale_proofs: 0,
            last_prepare: None,
            branch: b"main".as_slice().into(),
            round: l::Round::new(b"temper/1/l61".as_slice().into()),
            last_left: None,
            searched: None,
        },
        previous_asks: Box::new([]),
        entries: Box::new([forge::LandingEntry {
            entry: 7,
            member: 0,
            gate: 0,
            operation: forge::Operation::Dispatch,
            preparation: Some(1),
            base: Some([5; 32]),
            run: Some(forge::LandingRun { id: 23, attempt: 2, ended: false, clock }),
        }]),
    })
}

#[test]
fn landing_own_holds_keep_their_exact_reasons() {
    use temper_engine_domain_forge_landing as l;
    type HoldCase = (l::Hold, Option<&'static [u8]>, Option<&'static [u8]>);
    const CASES: &[HoldCase] = &[
        (l::Hold::Read, Some(b"read: branch main: no answer"), None),
        (l::Hold::Cleanup, None, Some(b"close item: forge refused permission; resource org/repo #9")),
    ];
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for &(why, read, cleanup) in CASES {
        let mut row = landing_with_clock(None);
        let forge::Stored::Landing(landing) = &mut row else { panic!("landing fixture") };
        landing.landing.phase = l::Phase::Held { was: Box::new(l::Phase::Preparing), why };
        if let Some(read) = read {
            landing.read_hold = Some(Box::from(read));
        }
        if let Some(cleanup) = cleanup {
            landing.landing.round.cleanup = Some(Box::from(cleanup));
        }
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
        assert_eq!(records::row(&key, &records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn each_stuck_cause_round_trips_in_an_own_hold() {
    use temper_engine_domain_forge_landing as l;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for cause in [
        l::Stuck::Member { change: 7, held: true },
        l::Stuck::Train { landing: 41 },
        l::Stuck::Wake { on: l::Awaited::Post { member: 3, gate: 5 } },
    ] {
        let mut row = landing_with_clock(None);
        let forge::Stored::Landing(landing) = &mut row else { unreachable!() };
        landing.landing.phase = l::Phase::Held { was: Box::new(l::Phase::Proving), why: l::Hold::Stuck(cause) };
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
        assert_eq!(records::row(&key, &records::encode(&row), &bounds), Ok(row));
    }
}

fn before_landing_stuck(current: &str) -> Box<str> {
    let first = current.find("record LandingStuckMember ").unwrap();
    let past = current.find("enum LandingOwnHold ").unwrap();
    current.replace(&current[first..past], "").replace(", stuck: LandingStuckHold", "").into_boxed_str()
}

#[test]
fn the_previous_own_hold_layout_keeps_cleanup_holds() {
    use temper_engine_domain_forge_landing as l;
    let family = crate::OWNER_SET.families.first().unwrap();
    let before = before_landing_stuck(family.current);
    let mut row = landing_with_clock(None);
    let forge::Stored::Landing(landing) = &mut row else { unreachable!() };
    landing.landing.phase = l::Phase::Held { was: Box::new(l::Phase::Proving), why: l::Hold::Cleanup };
    landing.landing.round.cleanup = Some(b"could not close #9".as_slice().into());
    assert_landing_through_layout(family.current, &before, true, row);
}

#[test]
fn a_landing_claim_hold_round_trips() {
    use temper_engine_domain_forge_landing as l;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let mut row = landing_with_clock(None);
    let forge::Stored::Landing(landing) = &mut row else { unreachable!() };
    landing.landing.phase = l::Phase::Held { was: Box::new(l::Phase::Proving), why: l::Hold::Claimed { change: 5 } };
    let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
    assert_eq!(records::row(&key, &records::encode(&row), &bounds), Ok(row));
}

#[test]
fn a_landing_runs_execution_clock_round_trips() {
    use skein_lib::{Duration, Wall};
    use temper_engine_domain_forge_change::{Clock, Execution};
    let stored = landing_with_clock(Some(Clock {
        since: Wall::from_nanos(7),
        head: [5; 32],
        run: 23,
        attempt: 2,
        ran: Duration::from_nanos(13),
        waited: Duration::from_nanos(17),
        unknown: Duration::ZERO,
        at: Wall::from_nanos(53),
        seen: Execution::Running,
    }));
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
}

#[test]
fn a_landing_follow_round_trips_without_an_outbox_entry() {
    use skein_lib::{Time, Wall};
    let forge::Stored::Landing(mut row) = landing_with_clock(None) else { unreachable!() };
    let mut entry = *row.entries.first().expect("fixture run");
    entry.operation = forge::Operation::Rerun;
    row.entries = Box::new([]);
    row.keep_follows(Box::new([forge::LandingFollow {
        entry,
        dispatch: Some(forge::LandingDispatch {
            workflow: b"gate.yml".as_slice().into(),
            branch: b"temper/1/l61".as_slice().into(),
            inputs: Box::new([skein_forge::api::Input {
                name: b"temper_key".as_slice().into(),
                value: b"00".as_slice().into(),
            }]),
        }),
        start: Some(Time::from_nanos(4)),
        since: Some(Time::from_nanos(9)),
        wall: Some(Wall::from_nanos(13)),
        try_number: 2,
        failed: false,
        reruns: Box::new([skein_forge::api::JobAttempt { head: [5; 32], run: 23, job: 29, attempt: 1 }]),
    }]));
    let stored = forge::Stored::Landing(row);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
}

#[test]
fn a_landing_run_written_without_a_clock_decodes_with_none() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let mut run = None;
    for line in family.current.lines() {
        if line.starts_with("record LandingRun ") {
            run = Some(line);
        }
    }
    let run = run.unwrap();
    let before = family.current.replace(run, "record LandingRun { id: u64 attempt: u32 ended: bool }");
    assert_ne!(before, family.current);
    assert_landing_through_layout(family.current, &before, false, landing_with_clock(None));
}

#[test]
fn a_landing_written_before_read_holds_reads_as_before() {
    let family = crate::OWNER_SET.families.first().unwrap();
    let before = before_landing_stuck(family.current)
        .replace("cleanup: option bytes 512 = none ", "")
        .replace(" read_hold: option bytes 256 = none", "")
        .replace("record LandingClaimed { change: u64 }\n", "")
        .replace(", read, claimed: LandingClaimed, cleanup", "");
    assert_landing_through_layout(family.current, &before, true, landing_with_clock(None));
}

#[test]
fn a_landing_written_before_cleanup_holds_reads_as_before() {
    let family = crate::OWNER_SET.families.first().unwrap();
    let before = before_landing_stuck(family.current)
        .replace("cleanup: option bytes 512 = none ", "")
        .replace(", cleanup }", " }");
    assert_landing_through_layout(family.current, &before, true, landing_with_clock(None));
}

#[test]
fn a_landing_written_before_claim_holds_reads_as_before() {
    use temper_engine_domain_forge_landing as l;
    let family = crate::OWNER_SET.families.first().unwrap();
    let before = before_landing_stuck(family.current)
        .replace("cleanup: option bytes 512 = none ", "")
        .replace("record LandingClaimed { change: u64 }\n", "")
        .replace(", claimed: LandingClaimed, cleanup", "");
    let mut row = landing_with_clock(None);
    let forge::Stored::Landing(landing) = &mut row else { unreachable!() };
    // `read` was the last own-hold variant at the previous layout.
    landing.landing.phase = l::Phase::Held { was: Box::new(l::Phase::Proving), why: l::Hold::Read };
    landing.read_hold = Some(b"read: branch main: no answer".as_slice().into());
    assert_landing_through_layout(family.current, &before, true, row);
}

fn assert_landing_through_layout(current: &str, before: &str, added_variant: bool, stored: forge::Stored) {
    use skein_codec::layout::{Mode, Plan, Workspace};
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(current.as_bytes()).unwrap();
    let old_root = old.root(b"Row", None).unwrap();
    let new_root = new.root(b"Row", None).unwrap();
    let down = if added_variant {
        Plan::between(Mode::Read, &new, new_root, &[], &old, old_root, &[])
    } else {
        Plan::derive(&new, new_root, &[], &old, old_root, &[])
    }
    .unwrap();
    let up = Plan::derive(&old, old_root, &[], &new, new_root, &[]).unwrap();
    let mut work = Workspace::new();
    assert!(work.fit(&down));
    let mut earlier = skein_lib::Buffer::with_capacity(131_072);
    down.convert(&records::encode(&stored), &mut earlier, &mut work).unwrap();
    assert!(work.fit(&up));
    let mut current = skein_lib::Buffer::with_capacity(131_072);
    up.convert(earlier.as_slice(), &mut current, &mut work).unwrap();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(current.as_slice(), &bounds), Ok(stored));
}

#[test]
fn landing_faults_and_own_holds_are_mutually_exclusive() {
    use skein_lib::List;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for faults in [false, true] {
        for own in [false, true] {
            let phase = v25::LandingPhase {
                place: v25::LandingPlace::Proving,
                held: if faults { Some(v25::LandingFaults { member: 0, gate: 0 }) } else { None },
            };
            let landing = v25::LandingState {
                task: 61,
                project: 5,
                phase,
                members: List::with_capacity(0),
                preparation: 0,
                stale_proofs: 0,
                branch: Box::new([]),
                round: None,
                own_hold: own.then_some(v25::LandingOwnHold::Turn),
                last_left: None,
                searched: None,
                last_prepare: None,
            };
            let row = v25::LandingRow {
                read_hold: None,
                last_event: None,
                turn_since: None,
                waiting: None,
                follows: None,
                flake_reruns: List::with_capacity(0),
                room: None,
                landing,
                asks: List::with_capacity(0),
                entries: List::with_capacity(0),
                job: None,
            };
            let wire = v25::Row { value: v25::Value::Landing(Box::new(row)) };
            let bytes = wire.encode(&v25::CEILINGS).expect("measured room");
            let decoded = records::decode(&bytes, &bounds);
            if faults && own {
                assert_eq!(decoded, Err(crate::Undecodable::Malformed), "two holds cannot wrap one landing phase");
            } else {
                let decoded = decoded.expect("a landing with at most one hold decodes");
                assert_eq!(records::encode(&decoded), bytes, "each admitted hold survives exactly");
            }
        }
    }
}

#[test]
fn receipts_preserve_the_check_key_run_and_verdict() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for check in [
        forge::Check::Gate(Box::new([])),
        forge::Check::Tier(Box::new([])),
        forge::Check::Gate(Box::from(b"gate".as_slice())),
        forge::Check::Tier(Box::from(b"tier".as_slice())),
        forge::Check::Gate(Box::from([0; 32])),
        forge::Check::Tier(Box::from([0; 32])),
    ] {
        for passed in [false, true] {
            let mut jobs = skein_lib::List::with_capacity(16);
            if !passed {
                for job in 0..16 {
                    jobs.push(forge::ReceiptJob { job, attempt: u32::MAX, name: Box::new([b'n'; 256]) }).unwrap();
                }
            }
            let row = forge::Stored::Receipt {
                key: forge::ReceiptKey {
                    project: 5,
                    repository: skein_forge::api::Repository { forge: 1, repository: 2 },
                    check: check.clone(),
                    key: [3; 32],
                },
                receipt: forge::Receipt {
                    report: temper_engine_domain_forge::ReceiptReport::Unset,
                    base: if passed { None } else { Some([0xbb; 32]) },
                    jobs: jobs.into_boxed(),
                    logs: Box::new([]),
                    failures: None,
                    passed,
                    run: forge::ReceiptRun { id: 7, number: 8, url: Box::from(b"https://forge/run/7".as_slice()) },
                    candidate: [4; 32],
                    ended: skein_lib::Wall::from_nanos(9),
                },
            };
            let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).expect("receipt key");
            assert_eq!(key[0], 24);
            assert_eq!(crate::keys::decode(&key, &fixtures::LIMITS), Some(forge::stored_key(&row)));
            assert_eq!(records::row(&key, &records::encode(&row), &bounds), Ok(row));
        }
    }
}

#[test]
fn a_changes_failed_effect_account_survives_the_store() {
    use temper_engine_domain_forge_change::{EffectResult, Hold, State};
    let mut row =
        fixtures::change_row(State::Held { was: Box::new(State::Opening { requested: true }), why: Hold::Failed });
    row.effect = EffectResult::Failed;
    row.effect_account = Some(forge::EffectAccount {
        entry: 28,
        words: b"open pull request on org/repo: forge refused permission".as_slice().into(),
    });
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let row = fixtures::change_hot(row);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

#[test]
fn a_changes_producer_failure_survives_the_store_at_the_hold_account_bound() {
    use temper_engine_domain_forge_change::{Hold, State};
    let mut row =
        fixtures::change_row(State::Held { was: Box::new(State::Producing { requested: true }), why: Hold::Failed });
    let mut words = skein_lib::List::with_capacity(jig_domain_tasks::HOLD_ACCOUNT_BYTES);
    for _ in 0..jig_domain_tasks::HOLD_ACCOUNT_BYTES {
        words.push(b'x').expect("hold account bound");
    }
    row.producer_failure = Some(words.into_boxed());
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let row = fixtures::change_hot(row);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

fn writer_rules(row: &forge::Stored) -> bool {
    if let forge::Stored::Binding { address, .. } = row {
        return forge::binding_address(address);
    }
    if let forge::Stored::Writer(writer) = row {
        return writer.author != 0;
    }
    false
}

#[test]
fn admitted_binding_and_writer_saves_obey_writer_rules_before_codec_round_trip() {
    use skein_lib::{Env, Queue, Time, Token, Wall};
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::LIMITS };
    let mut domain = forge::Domain::new(
        &env.limits,
        91,
        temper_engine_domain_forge::Config { push: None, namespace: Box::new([7; 16]), writers: Box::new([]) },
        &[],
        &[],
        false,
    )
    .unwrap();
    let mut out = Queue::uncapped();
    forge::step(
        &mut domain,
        &env,
        forge::Event::Bind {
            request: Token::new(1),
            address: b"Sam@example.com".as_slice().into(),
            subject: b"member".as_slice().into(),
        },
        &mut out,
    );
    assert!(!forge::writer(&mut domain, &env, skein_forge_client::Writer { forge: 1, author: 0 }, &mut out));
    assert!(forge::writer(&mut domain, &env, skein_forge_client::Writer { forge: 1, author: 42 }, &mut out));
    let bounds = crate::bounds(&env.limits, env.limits.client.op_bytes).unwrap();
    let mut saves = 0_u32;
    while let Some(request) = out.pop() {
        if let forge::Request::Save { record, .. } = request {
            assert!(writer_rules(&record));
            let mut altered = record.clone();
            if let forge::Stored::Binding { address, .. } = &mut altered {
                assert_eq!(address.as_ref(), b"sam@example.com");
                address[0] = b'S';
            } else if let forge::Stored::Writer(writer) = &mut altered {
                writer.author = 0;
            } else {
                panic!("only binding and writer saves");
            }
            assert!(!writer_rules(&altered), "bounded mutation violates writer obligation");
            let key = crate::keys::encode(&forge::stored_key(&record), 511).unwrap();
            assert_eq!(records::row(&key, &records::encode(&record), &bounds), Ok(record));
            saves += 1;
        }
    }
    assert_eq!(saves, 2, "both real writer APIs emitted durable saves");
}

fn new_family_round_trip(row: &forge::Stored) {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let bytes = records::encode(row);
    let key = crate::keys::encode(&forge::stored_key(row), u32::MAX).expect("row key");
    assert_eq!(records::row(&key, &bytes, &bounds), Ok(row.clone()));
    assert_eq!(records::row(&[0], &bytes, &bounds), Err(crate::Undecodable::Malformed));
    assert_eq!(
        records::decode(&bytes[..bytes.len().checked_sub(1).expect("versioned row is nonempty")], &bounds),
        Err(crate::Undecodable::Malformed)
    );
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).expect("small row"));
    trailing.put(&bytes).expect("row room");
    trailing.put(&[0]).expect("trailing byte room");
    assert_eq!(records::decode(&trailing.finish(), &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn forge_record_round_trips_every_mode_and_state() {
    for kind in [forge::Kind::Forgejo, forge::Kind::GitHub] {
        for mode in [forge::Mode::Connected, forge::Mode::Managed, forge::Mode::Owned, forge::Mode::Fake] {
            for state in [
                forge::ForgeState::Kept,
                forge::ForgeState::Ended { secrets_erased: false, box_removed: false },
                forge::ForgeState::Ended { secrets_erased: false, box_removed: true },
                forge::ForgeState::Ended { secrets_erased: true, box_removed: false },
                forge::ForgeState::Ended { secrets_erased: true, box_removed: true },
            ] {
                for first in [false, true] {
                    for optional in [false, true] {
                        let mut record = fixtures::forge_record();
                        record.kind = kind;
                        record.mode = mode;
                        record.state = state;
                        record.read_only = optional;
                        record.asked = optional;
                        record.person = if optional { Some(forge::Link { user: 71, person: 29 }) } else { None };
                        record.first = if first {
                            Some(forge::FirstIntent::Empty(forge::EmptyIntent {
                                playground: optional,
                                sample: false,
                                ..match record.first.take().expect("fixture first intent") {
                                    forge::FirstIntent::Empty(first) => first,
                                    forge::FirstIntent::Copies { .. } | forge::FirstIntent::Nothing => {
                                        panic!("empty fixture intent")
                                    }
                                }
                            }))
                        } else {
                            None
                        };
                        if !optional {
                            record.git_program = None;
                            record.place.api.tls = None;
                            record.place.web = None;
                        }
                        new_family_round_trip(&forge::Stored::Forge(record));
                    }
                }
            }
        }
    }
}

#[test]
fn forge_number_round_trips() {
    for last in [0, 1, u16::MAX] {
        new_family_round_trip(&forge::Stored::ForgeNumber { last });
    }
}

#[test]
fn playground_round_trips_every_state() {
    for state in [
        forge::PlaygroundState::Starting,
        forge::PlaygroundState::Running,
        forge::PlaygroundState::Resetting(forge::ResetStep::Project),
        forge::PlaygroundState::Resetting(forge::ResetStep::Forge),
        forge::PlaygroundState::Resetting(forge::ResetStep::Account),
    ] {
        for project in [None, Some(0), Some(u32::MAX)] {
            for account in [None, Some(0), Some(u32::MAX)] {
                new_family_round_trip(&forge::Stored::Playground(forge::Playground {
                    state,
                    forge: u16::MAX,
                    project,
                    account,
                }));
            }
        }
    }
}

#[test]
fn forge_names_decode_at_the_deployment_bound_and_refuse_one_past() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for field in ["org", "repository"] {
        for extra in [0, 1] {
            let mut row = fixtures::forge_record();
            let Some(forge::FirstIntent::Empty(first)) = row.first.as_mut() else { panic!("fixture empty intent") };
            let name =
                skein_lib::bytes::zeroed(usize::try_from(fixtures::LIMITS.name_bytes + extra).expect("name length"));
            match field {
                "org" => first.org = name,
                "repository" => first.repository = name,
                _ => panic!("named field"),
            }
            let row = forge::Stored::Forge(row);
            let decoded = records::decode(&records::encode(&row), &bounds);
            if extra == 0 {
                assert_eq!(decoded, Ok(row), "{field} at bound");
            } else {
                assert_eq!(decoded, Err(crate::Undecodable::Malformed), "{field} past bound");
            }
        }
    }
}

#[test]
fn store_ten_rows_are_refused_by_store_eleven() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let mut old = records::encode(&forge::Stored::ForgeNumber { last: 1 });
    old[1] = 10;
    assert_eq!(records::decode(&old, &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn a_changes_producer_report_survives_the_store() {
    let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
    row.report = b"Done: edited README.\n## Left\n- none".as_slice().into();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let row = fixtures::change_cold(row);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

#[test]
fn a_producer_report_is_bounded_by_the_forge_operation_bytes() {
    let bound = fixtures::LIMITS.client.op_bytes;
    let bounds = crate::bounds(&fixtures::LIMITS, bound).expect("fixture bounds");
    assert_eq!(bounds.limits().change_cold_report, bound);
    for length in [bound, bound + 1] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
        row.report = skein_lib::bytes::zeroed(skein_lib::usize_of(length));
        let row = fixtures::change_cold(row);
        let decoded = records::decode(&records::encode(&row), &bounds);
        if length == bound {
            assert_eq!(decoded, Ok(row));
        } else {
            assert_eq!(decoded, Err(crate::Undecodable::Malformed));
        }
    }
}

#[test]
fn questions_open_follows_the_change_bound() {
    use temper_engine_domain_forge_change::{Open, Stands, State};
    const ASKER: u64 = 7;
    // domain/forge/change/questions.md, 1: the policy's questions and the one that holds the change.
    // The stock takes four, standard takes eighty; 126 leaves one past encodable at the ceiling.
    for (questions, open_bound) in [(0, 1), (4, 5), (80, 81), (126, 127)] {
        let mut limits = fixtures::LIMITS;
        limits.change_policy.questions = questions;
        let bounds = crate::bounds(&limits, limits.client.op_bytes).expect("policy fits the schema");
        assert_eq!(bounds.limits().questions_open, open_bound);
        assert_eq!(records::limits(&limits).expect("record limits").questions_open, open_bound);
        for (count, accepted) in [(open_bound, true), (open_bound + 1, false)] {
            let mut row = fixtures::change_row(State::Producing { requested: true });
            let mut open = skein_lib::List::with_capacity(count);
            for number in 0..count {
                open.push(Open {
                    asker: ASKER,
                    number: u64::from(number),
                    line: Box::new([]),
                    words: Box::new([]),
                    stands: Stands::Taken,
                })
                .unwrap();
            }
            row.questions.open = open.into_boxed();
            let row = fixtures::change_hot(row);
            let expected = if accepted { Ok(row.clone()) } else { Err(crate::Undecodable::Malformed) };
            assert_eq!(
                records::decode(&records::encode(&row), &bounds),
                expected,
                "policy: {questions}, open: {count}"
            );
        }
    }
}

#[test]
fn a_changes_questions_round_trip_unchanged() {
    use temper_engine_domain_forge_change::{Blocked, Hold, Open, Questions, Reply, Route, Stands, State, Waiting};
    const ASKER: u64 = 7;
    let mut row =
        fixtures::change_row(State::Held { was: Box::new(State::Checking { head: [1; 32] }), why: Hold::Questions });
    let mut open = skein_lib::List::uncapped();
    for (number, stands) in [
        (1, Stands::Taken),
        (4, Stands::Asked),
        (
            5,
            Stands::Held {
                answer: b"Keep the documented behavior".as_slice().into(),
                question: 51,
                blocked: Some(Blocked { dependency: Some(28), since: skein_lib::Wall::from_nanos(65) }),
            },
        ),
        (
            6,
            Stands::Held {
                answer: b"Use the alternative".as_slice().into(),
                question: 52,
                blocked: Some(Blocked { dependency: None, since: skein_lib::Wall::EPOCH }),
            },
        ),
    ] {
        open.push(Open {
            asker: ASKER,
            number,
            line: b"Which behavior?".as_slice().into(),
            words: b"Which behavior?\nKeep both options.".as_slice().into(),
            stands,
        })
        .unwrap();
    }
    row.questions = Questions {
        route: Route::Up,
        answered: Box::new([]),
        amended: Box::new([]),
        taken: 3,
        open: open.into_boxed(),
        waiting: Box::new([
            Waiting { from: 40, number: 8, words: Box::new([]), passed: Some(42) },
            Waiting { from: 40, number: 9, words: b"Go on: prerequisite landed".as_slice().into(), passed: None },
        ]),
        replies: Box::new([
            Reply { target: 40, question: Some(1), words: b"This change cannot take the question".as_slice().into() },
            Reply { target: 40, question: None, words: b"No question is open".as_slice().into() },
        ]),
    };
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let row = fixtures::change_hot(row);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

/// questions.md, 1, Its answers, kept for later work: the bounded record survives a restart.
#[test]
fn a_changes_kept_answers_round_trip_unchanged() {
    use temper_engine_domain_forge_change::{AMENDMENTS_KEPT, ANSWERS_KEPT, Amended, Answered, Route, State};
    let mut row = fixtures::change_row(State::Producing { requested: true });
    row.questions.route = Route::Up;
    let mut answered = skein_lib::List::with_capacity(u32::try_from(ANSWERS_KEPT).unwrap());
    for number in 1..=u64::try_from(ANSWERS_KEPT).unwrap() {
        answered
            .push(Answered {
                revision: number,
                asker: 7,
                number,
                line: b"Question: keep both lines?".as_slice().into(),
                answer: b"keep both lines\nKeep blank lines too".as_slice().into(),
            })
            .unwrap();
    }
    row.questions.answered = answered.into_boxed();
    let mut amended = skein_lib::List::with_capacity(u32::try_from(AMENDMENTS_KEPT).unwrap());
    for revision in 1..=u64::try_from(AMENDMENTS_KEPT).unwrap() {
        amended.push(Amended { revision, reason: b"Drop the file".as_slice().into() }).unwrap();
    }
    row.questions.amended = amended.into_boxed();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let stored = fixtures::change_hot(row);
    assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
}

/// A registered field defaults at layout conversion, not in the fixed-layout decoder.
#[test]
fn a_change_row_written_without_kept_answers_decodes_with_none() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    for keep_answers in [false, true] {
        let mut before = family
            .current
            .replace(" replies: list 1024 ChangeReply = []", "")
            .replace(" amended: list 16 Amended = []", "")
            .replace(" revision: u64 = 0", "");
        if !keep_answers {
            before = before.replace(" answered: list 16 Answered = []", "");
        }
        let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
        let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
        if keep_answers {
            row.questions.answered = Box::new([temper_engine_domain_forge_change::Answered {
                revision: 0,
                asker: 7,
                number: 3,
                line: b"Which?".as_slice().into(),
                answer: b"Both".as_slice().into(),
            }]);
        }
        let stored = fixtures::change_hot(row);
        let mut bytes = records::encode(&stored);
        // Write the preceding layout, then read it through the registered layout conversion.
        for (source, target) in [(&new, &old), (&old, &new)] {
            let plan = skein_codec::layout::Plan::derive(
                source,
                source.root(b"Row", None).unwrap(),
                &[],
                target,
                target.root(b"Row", None).unwrap(),
                &[],
            )
            .unwrap();
            let mut work = skein_codec::layout::Workspace::new();
            assert!(work.fit(&plan));
            let mut out = skein_lib::Buffer::with_capacity(131_072);
            plan.convert(&bytes, &mut out, &mut work).unwrap();
            bytes = out.as_slice().into();
        }
        let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
        assert_eq!(records::decode(&bytes, &bounds), Ok(stored));
    }
}

#[test]
fn standard_sized_question_words_and_held_answers_round_trip() {
    use temper_engine_domain_forge_change::{Open, Stands, State};
    let mut limits = fixtures::LIMITS;
    limits.client.op_bytes = 262_144;
    let bounds = crate::bounds(&limits, limits.client.op_bytes).expect("standard operation bounds");
    let mut row = fixtures::change_row(State::Producing { requested: true });
    row.questions.open = Box::new([Open {
        asker: 7,
        number: 1,
        line: Box::new([]),
        words: skein_lib::bytes::zeroed(262_144),
        stands: Stands::Held { answer: skein_lib::bytes::zeroed(262_144), question: 0, blocked: None },
    }]);
    let row = fixtures::change_hot(row);
    let bytes = records::encode(&row);
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    for words in [false, true] {
        let mut short = bounds.clone();
        if words {
            short.limits.open_words -= 1;
        } else {
            short.limits.held_answer -= 1;
        }
        assert_eq!(records::decode(&bytes, &short), Err(crate::Undecodable::Malformed));
    }
}

#[test]
fn a_held_answer_is_bounded_by_the_forge_operation_bytes() {
    use temper_engine_domain_forge_change::{Open, Stands, State};
    let bound = fixtures::LIMITS.client.op_bytes;
    let bounds = crate::bounds(&fixtures::LIMITS, bound).expect("fixture bounds");
    assert_eq!(bounds.limits().held_answer, bound);
    for (length, accepted) in [(bound, true), (bound + 1, false)] {
        let mut row = fixtures::change_row(State::Producing { requested: true });
        row.questions.open = Box::new([Open {
            asker: 7,
            number: 1,
            line: Box::new([]),
            words: Box::new([]),
            stands: Stands::Held {
                answer: skein_lib::bytes::zeroed(usize::try_from(length).unwrap()),
                question: 0,
                blocked: None,
            },
        }]);
        let row = fixtures::change_hot(row);
        let expected = if accepted { Ok(row.clone()) } else { Err(crate::Undecodable::Malformed) };
        assert_eq!(records::decode(&records::encode(&row), &bounds), expected, "answer bytes: {length}");
    }
}

#[test]
fn an_open_questions_words_are_bounded_by_the_forge_operation_bytes() {
    use temper_engine_domain_forge_change::{Open, Stands, State};
    let bound = fixtures::LIMITS.client.op_bytes;
    let bounds = crate::bounds(&fixtures::LIMITS, bound).expect("fixture bounds");
    assert_eq!(bounds.limits().open_words, bound);
    for (length, accepted) in [(bound, true), (bound + 1, false)] {
        let mut row = fixtures::change_row(State::Producing { requested: true });
        row.questions.open = Box::new([Open {
            asker: 7,
            number: 1,
            line: Box::new([]),
            words: skein_lib::bytes::zeroed(usize::try_from(length).unwrap()),
            stands: Stands::Asked,
        }]);
        let row = fixtures::change_hot(row);
        let expected = if accepted { Ok(row.clone()) } else { Err(crate::Undecodable::Malformed) };
        assert_eq!(records::decode(&records::encode(&row), &bounds), expected, "question bytes: {length}");
    }
}

#[test]
fn a_changes_early_start_round_trips_each_arm_and_head_order() {
    use temper_engine_domain_forge_change::{Early, StartedFrom, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let heads = Box::new([StartedFrom { task: 12, head: [2; 32] }, StartedFrom { task: 11, head: [1; 32] }]);
    for early in [
        Early::None,
        Early::Started(Box::new([])),
        Early::Spent(Box::new([])),
        Early::Started(heads.clone()),
        Early::Spent(heads),
    ] {
        let mut row = fixtures::change_row(State::Producing { requested: false });
        row.change.early = early;
        let row = fixtures::change_hot(row);
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}

#[test]
fn repository_makings_and_their_outcomes_round_trip() {
    use forge::{
        MakeFailure, MakeFrom, MakeInto, MakeRefusal, MakeStep, MakingFailed, MakingKey, MakingState, RepositoryMaking,
    };
    // Four live states and five steps with six failure reasons; six refusals and kept.
    const STATES: u32 = 34;
    const OUTCOMES: u32 = 8;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let branch: Box<[u8]> = Box::from(b"main".as_slice());
    let mut states = skein_lib::List::with_capacity(STATES);
    for state in [
        MakingState::Waiting,
        MakingState::Making,
        MakingState::Made { id: 42, branch: branch.clone() },
        MakingState::Adopting { id: 42, branch },
    ] {
        states.push(state).expect("state fixture bound");
    }
    for step in [MakeStep::Admin, MakeStep::Org, MakeStep::Repository, MakeStep::File, MakeStep::Adoption] {
        for reason in [
            MakeFailure::Taken,
            MakeFailure::Empty,
            MakeFailure::TimedOut,
            MakeFailure::Refused { status: 409 },
            MakeFailure::NotAllowed,
            MakeFailure::Other,
        ] {
            states
                .push(MakingState::Failed(MakingFailed { step, reason, words: Box::from(b"forge words".as_slice()) }))
                .expect("state fixture bound");
        }
    }
    for state in states.into_boxed() {
        for from in
            [MakeFrom::Empty, MakeFrom::Copy { address: Box::from(b"https://source.invalid/owner/repo".as_slice()) }]
        {
            for into in [None, Some(MakeInto { project: 7, home: true, host: Box::from(b"forge.invalid".as_slice()) })]
            {
                for copying in [false, true] {
                    let row = forge::Stored::Making(RepositoryMaking {
                        key: MakingKey {
                            forge: 1,
                            org: Box::from(b"owner".as_slice()),
                            name: Box::from(b"repo".as_slice()),
                        },
                        from: from.clone(),
                        into: into.clone(),
                        asked: skein_lib::Wall::from_nanos(123),
                        copying,
                        state: state.clone(),
                    });
                    let encoded = records::encode(&row);
                    assert_eq!(records::decode(&encoded, &bounds), Ok(row));
                }
            }
        }
    }
    let mut outcomes = skein_lib::List::with_capacity(OUTCOMES);
    outcomes.push(forge::ApplicationOutcome::MakeKept).expect("outcome fixture bound");
    for refusal in [
        MakeRefusal::NoSuchForge,
        MakeRefusal::NotOffered,
        MakeRefusal::NotOwner,
        MakeRefusal::Invalid,
        MakeRefusal::HomeHeld,
        MakeRefusal::Full,
        MakeRefusal::NotFailed,
    ] {
        outcomes.push(forge::ApplicationOutcome::MakeRefused(refusal)).expect("outcome fixture bound");
    }
    for outcome in outcomes.into_boxed() {
        let row = forge::Stored::Application { token: 1, until: skein_lib::Wall::from_nanos(456), outcome };
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn dismissed_rows_round_trip_at_the_field_bounds() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for (party, project) in [(0, 0), (7, 3), (u64::MAX, u32::MAX)] {
        for what in [
            forge::PersonFact::ToBegin { project },
            forge::PersonFact::GoalFrom { project, chat: party, goal: u64::MAX, at: u64::MAX },
            forge::PersonFact::KeepAsking {
                project,
                subject: temper_protocol_reaches::Subject::Kind(temper_protocol_reaches::Kind::Rules),
                at: u64::MAX,
            },
            forge::PersonFact::KeepAsking { project, subject: temper_protocol_reaches::Subject::Set(u8::MAX), at: 0 },
        ] {
            let row = forge::Stored::Dismissed { party, what };
            let bytes = records::encode(&row);
            let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
            assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
        }
    }
}

#[test]
fn each_setup_intent_round_trips_in_its_forge_record() {
    use temper_engine_domain_forge::{EmptyIntent, FirstIntent};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let limits = bounds.limits();
    for first in [
        FirstIntent::Nothing,
        FirstIntent::Copies { addresses: Box::new([]) },
        FirstIntent::Copies {
            addresses: full_copy_addresses(limits.copy_intent_addresses, limits.copy_intent_addresses_item),
        },
        FirstIntent::Empty(EmptyIntent {
            org: Box::new([]),
            repository: Box::new([]),
            label: Box::new([]),
            playground: false,
            sample: false,
        }),
        FirstIntent::Empty(EmptyIntent {
            org: skein_lib::bytes::zeroed(usize::try_from(limits.empty_intent_org).unwrap()),
            repository: skein_lib::bytes::zeroed(usize::try_from(limits.empty_intent_repository).unwrap()),
            label: skein_lib::bytes::zeroed(usize::try_from(limits.empty_intent_label).unwrap()),
            playground: true,
            sample: true,
        }),
    ] {
        let mut record = fixtures::forge_record();
        record.first = Some(first);
        new_family_round_trip(&forge::Stored::Forge(record));
    }
}

#[test]
fn a_forge_written_without_sample_decodes_with_false() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family.current.replace(" sample: bool = false", "");
    assert_ne!(before, family.current);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let mut row = fixtures::forge_record();
    let Some(forge::FirstIntent::Empty(first)) = &mut row.first else { panic!("empty fixture intent") };
    first.sample = true;
    let mut bytes = records::encode(&forge::Stored::Forge(row.clone()));
    let mut work = skein_codec::layout::Workspace::new();
    // Removing the sample flag and defaulting it to false cannot grow the row.
    let mut out = skein_lib::Buffer::with_capacity(u32::try_from(bytes.len()).unwrap());
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", Some(b"forge")).unwrap(),
            &[],
            target,
            target.root(b"Row", Some(b"forge")).unwrap(),
            &[],
        )
        .unwrap();
        assert!(work.fit(&plan));
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    let Some(forge::FirstIntent::Empty(first)) = &mut row.first else { panic!("empty fixture intent") };
    first.sample = false;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(&bytes, &bounds), Ok(forge::Stored::Forge(row)));
}

pub(super) fn full_copy_addresses(count: u32, length: u32) -> Box<[Box<[u8]>]> {
    let mut addresses = skein_lib::List::with_capacity(count);
    for _ in 0..count {
        addresses.push(skein_lib::bytes::zeroed(usize::try_from(length).unwrap())).unwrap();
    }
    addresses.into_boxed()
}

#[expect(
    clippy::disallowed_macros,
    reason = "ordinary boundary fixtures allocate rows at configured bounds and one past them"
)]
fn landing_log(landings: u32, members: u32, branch_bytes: u32) -> forge::Stored {
    forge::Stored::LandingLog(forge::LandingLog {
        repository: skein_forge::api::Repository { forge: 1, repository: 2 },
        landings: vec![
            forge::LoggedLanding {
                tiers: Box::new([]),
                commit: [7; 32],
                branch: vec![b'x'; usize::try_from(branch_bytes).unwrap()].into(),
                at: skein_lib::Wall::from_nanos(123),
                changes: vec![
                    forge::LoggedChange {
                        fixes: Box::new([]),
                        goal: None,
                        milestone: None,
                        person: None,
                        goal_title: None,
                        draft: None,
                        signed: Box::new([]),
                        task: 51,
                        pull: Some(2),
                        title: vec![b't'; 256].into()
                    };
                    usize::try_from(members).unwrap()
                ]
                .into(),
            };
            usize::try_from(landings).unwrap()
        ]
        .into(),
    })
}

#[test]
fn landing_logs_round_trip_and_refuse_each_configured_bound() {
    assert_eq!(v25::CEILINGS.logged_red_tests, forge::LOGGED_TESTS);
    let limits = fixtures::LIMITS;
    assert_eq!(records::encode(&landing_log(16, 2, 64)).len(), 11_149);
    let bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
    for extra in [0, 1] {
        for row in [
            landing_log(
                limits.landings_kept + limits.landings_pending + extra,
                limits.landing_members,
                limits.name_bytes,
            ),
            landing_log(limits.landings_kept, limits.landing_members + extra, limits.name_bytes),
            landing_log(limits.landings_kept, limits.landing_members, limits.name_bytes + extra),
        ] {
            let bytes = records::encode(&row);
            let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
            assert_eq!(
                records::row(&key, &bytes, &bounds),
                if extra == 0 { Ok(row) } else { Err(crate::Undecodable::Malformed) }
            );
        }
    }
    let mut row = landing_log(1, 1, 1);
    if let forge::Stored::LandingLog(row) = &mut row {
        row.landings[0].changes[0].pull = None;
    }
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    for verdict in [
        forge::LoggedVerdict::Verifying,
        forge::LoggedVerdict::Cleared { run: u64::MAX },
        forge::LoggedVerdict::Red {
            run: 7,
            failing: Some(u32::MAX),
            tests: Box::new([b"suite::a".as_slice().into(), b"suite::b".as_slice().into()]),
            many: None,
        },
        forge::LoggedVerdict::Red { run: 9, failing: None, tests: Box::new([]), many: None },
        forge::LoggedVerdict::Red {
            run: 10,
            failing: None,
            tests: Box::new([]),
            many: Some(temper_engine_domain_forge_tiers::ManyTests {
                tests: Box::new([b"suite::t00".as_slice().into()]),
                more: 21,
                cut: true,
            }),
        },
    ] {
        let mut row = landing_log(1, 1, 1);
        if let forge::Stored::LandingLog(row) = &mut row {
            row.landings[0].changes[0].goal = Some(u64::MAX);
            row.landings[0].changes[0].milestone = Some(31);
            row.landings[0].tiers = Box::new([forge::LoggedTier { tier: Box::new([b't'; 32]), verdict }]);
        }
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
    let mut row = landing_log(1, 1, 1);
    if let forge::Stored::LandingLog(row) = &mut row {
        let mut tiers = skein_lib::List::with_capacity(limits.tiers.tiers.checked_add(1).unwrap());
        for at in 0..=limits.tiers.tiers {
            tiers
                .push(forge::LoggedTier {
                    tier: skein_lib::Decimal::of(u64::from(at)).as_bytes().into(),
                    verdict: forge::LoggedVerdict::Red {
                        run: 7,
                        failing: Some(1),
                        tests: Box::new([b"suite::a".as_slice().into()]),
                        many: None,
                    },
                })
                .unwrap();
        }
        row.landings[0].tiers = tiers.into_boxed();
    }
    assert_eq!(records::decode(&records::encode(&row), &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn status_rows_round_trip_all_options_and_field_bounds() {
    use skein_lib::Wall;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for (party, project, count) in [(0, 0, 0), (7, 3, 2), (u64::MAX, u32::MAX, u64::MAX)] {
        for present in [false, true] {
            let brief = present.then_some(forge::Brief {
                sentence: Box::from([b's'; forge::BRIEF_SENTENCE_BYTES]),
                lede: Box::from([b'l'; forge::BRIEF_LEDE_BYTES]),
                at: Wall::from_nanos(count),
                landed: count,
                findings: None,
            });
            for running in [
                None,
                Some(forge::StatusRunning {
                    task: None,
                    landed: count,
                    since: Wall::from_nanos(count),
                    findings: None,
                }),
                Some(forge::StatusRunning {
                    task: Some(u64::MAX),
                    landed: count,
                    since: Wall::from_nanos(count),
                    findings: None,
                }),
            ] {
                let facts = forge::Stored::ProjectFacts(forge::ProjectFacts {
                    project,
                    landed: count,
                    last_landed: Wall::from_nanos(count),
                });
                let looked = forge::Stored::Looked(forge::Looked {
                    party,
                    project,
                    previous: present.then_some(Wall::from_nanos(count)),
                    latest: Wall::from_nanos(count),
                    seen: count,
                    next_seen: count,
                });
                let person = forge::Stored::PersonBrief(forge::PersonBrief {
                    party,
                    project,
                    brief: brief.clone(),
                    running,
                    unpaid: present.then_some(Wall::from_nanos(count)),
                });
                for row in [facts, looked, person] {
                    let bytes = records::encode(&row);
                    let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
                    assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
                }
            }
        }
    }
}

fn status_findings_row(brief: Option<[u8; 32]>, running: Option<[u8; 32]>) -> forge::Stored {
    forge::Stored::PersonBrief(forge::PersonBrief {
        party: 7,
        project: 1,
        unpaid: None,
        brief: Some(forge::Brief {
            sentence: b"Findings remain".as_slice().into(),
            lede: Box::new([]),
            at: skein_lib::Wall::EPOCH,
            landed: 0,
            findings: brief,
        }),
        running: Some(forge::StatusRunning {
            task: Some(7),
            landed: 0,
            since: skein_lib::Wall::EPOCH,
            findings: running,
        }),
    })
}

#[test]
fn status_findings_marks_round_trip_some_and_none() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for brief in [None, Some([0; 32]), Some([255; 32])] {
        for running in [None, Some([7; 32])] {
            let row = status_findings_row(brief, running);
            assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        }
    }
}

#[test]
fn status_written_before_findings_marks_defaults_to_none() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family.current.replace(" findings: option fixed 32 = none", "");
    assert_ne!(before, family.current);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let row = status_findings_row(None, None);
    let mut bytes = records::encode(&row);
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", None).unwrap(),
            &[],
            target,
            target.root(b"Row", None).unwrap(),
            &[],
        )
        .expect("registered status fields have defaults");
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(131_072);
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(&bytes, &bounds), Ok(row));
}

fn away_row(present: bool, party: u64, project: u32, at: u64) -> forge::Away {
    forge::Away {
        party,
        project,
        since: skein_lib::Wall::from_nanos(at),
        until: present.then_some(skein_lib::Wall::from_nanos(at)),
        landed: at,
        off: None,
    }
}
fn decided_row(count: usize, at: u64) -> forge::Decided {
    let mut options: skein_lib::List<Box<[u8]>> = skein_lib::List::with_capacity(u32::try_from(count).unwrap());
    for _ in 0..count {
        options.push(Box::from([b'o'; forge::DECIDED_OPTION_BYTES])).unwrap();
    }
    forge::Decided {
        project: u32::MAX,
        number: u32::MAX,
        party: at,
        task: at,
        at: skein_lib::Wall::from_nanos(at),
        question: Box::from([b'q'; forge::DECIDED_QUESTION_BYTES]),
        options: options.into_boxed(),
        took: Some(u8::try_from(count.checked_sub(1).unwrap()).unwrap()),
        why: Box::from([b'w'; forge::DECIDED_WHY_BYTES]),
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
    }
}
#[test]
fn away_and_decided_rows_round_trip_their_options_and_bounds() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for at in [0, 9, u64::MAX] {
        for until in [false, true] {
            for off in [None, Some(skein_lib::Wall::from_nanos(at))] {
                let away = forge::Stored::Away(forge::Away { off, ..away_row(until, at, u32::MAX, at) });
                for count in [0, 2, 6] {
                    for overturned in [false, true] {
                        let mut row = decided_row(count.max(2), at);
                        if count == 0 {
                            row.took = None;
                            row.options = Box::new([]);
                            row.why = Box::new([]);
                        }
                        row.overturned = overturned && count != 0;
                        let decided = forge::Stored::Decided(row);
                        for row in [away.clone(), decided] {
                            let bytes = records::encode(&row);
                            let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
                            assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
                        }
                    }
                }
            }
        }
    }
}
#[test]
fn rows_refuse_invalid_options_and_keys() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for row in [
        forge::Stored::Forge(forge::ForgeRecord {
            person: Some(forge::Link { user: 0, person: 29 }),
            ..fixtures::forge_record()
        }),
        forge::Stored::Forge(forge::ForgeRecord {
            person: Some(forge::Link { user: 71, person: 0 }),
            ..fixtures::forge_record()
        }),
        forge::Stored::Decided(decided_row(1, 0)),
        forge::Stored::Decided(forge::Decided { took: Some(2), ..decided_row(2, 0) }),
    ] {
        assert_eq!(records::decode(&records::encode(&row), &bounds), Err(crate::Undecodable::Malformed));
    }
    for (row, wrong) in [
        (forge::Stored::Away(away_row(false, 7, 1, 0)), forge::Key::Away { party: 8, project: 1 }),
        (forge::Stored::Decided(decided_row(2, 0)), forge::Key::Decided { project: u32::MAX, number: 1 }),
    ] {
        let bytes = records::encode(&row);
        let key = crate::keys::encode(&wrong, u32::MAX).unwrap();
        assert_eq!(records::row(&key, &bytes, &bounds), Err(crate::Undecodable::Malformed));
        for len in 0..bytes.len() {
            assert_eq!(records::decode(&bytes[..len], &bounds), Err(crate::Undecodable::Malformed));
        }
    }
}

/// temper/docs/design/domain/forge/change/README.md, 1: the request and producer survive restart.
#[test]
fn a_change_row_with_a_close_request_round_trips() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
    row.close = Some(br#"{"version":1,"goal":7,"changes":[{"task":9,"landed":true}]}"#.as_slice().into());
    row.delegate = Some((42, temper_engine_domain_forge_change::Delegate::Close));
    for row in [fixtures::change_cold(row.clone()), fixtures::change_hot(row)] {
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn a_change_row_keeps_its_goal_ending_and_delegate_history() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for ending in [
        None,
        Some(forge::Ending::Landed),
        Some(forge::Ending::NoChange),
        Some(forge::Ending::Cancelled),
        Some(forge::Ending::Failed { line: b"failed first line".as_slice().into() }),
    ] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Ended);
        row.goal = Some(7);
        row.ending = ending;
        row.repair_tasks = Box::new([52, 53]);
        row.resolution_tasks = Box::new([90]);
        for row in [fixtures::change_cold(row.clone()), fixtures::change_hot(row)] {
            let meter = skein_world::domain::heap::Meter::new();
            meter.start();
            let cloned = row.clone();
            let held = meter.end().held();
            assert_eq!(forge::stored_bytes(&cloned), Some(held));
            drop(cloned);
            assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        }
    }
}

#[test]
fn a_closer_row_round_trips() {
    use temper_engine_domain_forge_closer::{Closer, Hold, Running, Step};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let rows = [
        forge::CloserRow {
            task: 8,
            goal: 7,
            closer: Closer::new(),
            making: None,
            close_changes: Box::new([]),
            superseded: Box::new([]),
            landing_changes: Box::new([]),
            relay: temper_engine_domain_forge_closer::relay::Relay::new(),
            done: false,
        },
        forge::CloserRow {
            task: 8,
            goal: 7,
            closer: Closer {
                closed: Some(Box::new([])),
                running: Some(Running { step: Step::Close, change: 20 }),
                landing: None,
                hold: Some(Hold::ClosingStep { step: Step::Close, change: 20 }),
                ..Closer::new()
            },
            making: Some(Step::Close),
            close_changes: Box::new([20]),
            superseded: Box::new([]),
            landing_changes: Box::new([]),
            relay: temper_engine_domain_forge_closer::relay::Relay::new(),
            done: false,
        },
        forge::CloserRow {
            task: 8,
            goal: 7,
            closer: Closer {
                closed: Some(Box::new([9, 12])),
                running: Some(Running { step: Step::Land, change: 21 }),
                landing: Some(21),
                hold: Some(Hold::ClosingStep { step: Step::Land, change: 21 }),
                retry: Some(Step::Land),
                ..Closer::new()
            },
            making: Some(Step::Land),
            close_changes: Box::new([20, 22]),
            superseded: Box::new([20]),
            landing_changes: Box::new([21, 23]),
            relay: temper_engine_domain_forge_closer::relay::Relay::new(),
            done: true,
        },
    ];
    for row in rows.into_iter().chain([forge::CloserRow {
        task: 8,
        goal: 7,
        closer: Closer {
            closed: Some(Box::new([9])),
            running: None,
            landing: None,
            hold: None,
            retry: Some(Step::Close),
            ..Closer::new()
        },
        making: None,
        close_changes: Box::new([20]),
        superseded: Box::new([]),
        landing_changes: Box::new([]),
        relay: temper_engine_domain_forge_closer::relay::Relay::new(),
        done: false,
    }]) {
        let row = forge::Stored::Closer(row);
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let cloned = row.clone();
        let held = meter.end().held();
        assert_eq!(forge::stored_bytes(&cloned), Some(held));
        drop(cloned);
        let key = forge::stored_key(&row);
        let key = crate::keys::encode(&key, u32::MAX).expect("closer key fits");
        assert_eq!(crate::keys::decode(&key, &fixtures::LIMITS), Some(forge::stored_key(&row)));
        assert_eq!(records::row(&key, &records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn a_closers_report_check_and_its_last_line_round_trip() {
    use temper_engine_domain_forge_closer::{ReportCheck, relay::Relay};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for check in [ReportCheck::Unread, ReportCheck::Held, ReportCheck::Released] {
        let forge::Stored::Closer(mut row) = closer_relay_row(Relay::new()) else { panic!("closer fixture") };
        row.closer.report_check = check;
        row.closer.goal_left = Some(b"the switch, not yet cut.".as_slice().into());
        let stored = forge::Stored::Closer(row);
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let cloned = stored.clone();
        assert_eq!(forge::stored_bytes(&cloned), Some(meter.end().held()));
        assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
    }
}

fn closer_relay_row(relay: temper_engine_domain_forge_closer::relay::Relay) -> forge::Stored {
    const CLOSER: u64 = 8;
    forge::Stored::Closer(forge::CloserRow {
        task: CLOSER,
        goal: CLOSER_GOAL,
        closer: temper_engine_domain_forge_closer::Closer::new(),
        making: None,
        close_changes: Box::new([]),
        superseded: Box::new([]),
        landing_changes: Box::new([]),
        done: false,
        relay,
    })
}

const CLOSER_GOAL: u64 = 7;
const CLOSER_LANDING: u64 = 20;
const CLOSER_SETTLEMENT: u64 = 21;
const LANDING_QUESTION: u64 = 10;
const SETTLEMENT_QUESTION: u64 = 11;
const NEXT_LANDING_QUESTION: u64 = 12;
const GOAL_QUESTION: u64 = 30;

fn waiting_questions() -> Box<[temper_engine_domain_forge_closer::relay::Question]> {
    use temper_engine_domain_forge_closer::{Step, relay::*};
    Box::new([
        Question {
            asker: CLOSER_SETTLEMENT,
            number: SETTLEMENT_QUESTION,
            step: Step::Close,
            words: b"What should I settle?".as_slice().into(),
            stands: Stands::Taken,
        },
        Question {
            asker: CLOSER_LANDING,
            number: NEXT_LANDING_QUESTION,
            step: Step::Land,
            words: b"What should I check?".as_slice().into(),
            stands: Stands::Taken,
        },
    ])
}

fn standing_question(
    stands: temper_engine_domain_forge_closer::relay::Stands,
) -> temper_engine_domain_forge_closer::relay::Relay {
    use temper_engine_domain_forge_closer::{Step, relay::*};
    Relay {
        open: Some(Question {
            asker: CLOSER_LANDING,
            number: LANDING_QUESTION,
            step: Step::Land,
            words: b"Which rule?".as_slice().into(),
            stands,
        }),
        replies: Box::new([Message {
            target: CLOSER_GOAL,
            form: Form::Words,
            words: b"Change 20: asker ended".as_slice().into(),
        }]),
        waiting: Box::new([]),
    }
}

#[test]
fn a_closers_question_round_trips_with_its_owned_bytes() {
    use skein_lib::Wall;
    use temper_engine_domain_forge_closer::relay::{Held, Stands};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits");
    for stands in [
        Stands::Taken,
        Stands::Asked { question: GOAL_QUESTION, answer: None, held: None },
        Stands::Asked {
            question: GOAL_QUESTION,
            answer: Some(b"Use known reds.".as_slice().into()),
            held: Some(Held { since: Wall::from_nanos(123), notified: true }),
        },
    ] {
        let mut relay = standing_question(stands);
        relay.waiting = waiting_questions();
        let row = closer_relay_row(relay);
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let cloned = row.clone();
        let held = meter.end().held();
        assert_eq!(forge::stored_bytes(&cloned), Some(held));
        drop(cloned);
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    }
}

#[test]
fn an_older_closer_defaults_its_waiting_questions_to_empty() {
    use super::dev_layouts::Dev;
    use temper_engine_domain_forge_closer::relay::Stands;
    let mut dev = Dev::new();
    let expected =
        closer_relay_row(standing_question(Stands::Asked { question: GOAL_QUESTION, answer: None, held: None }));
    let stored = dev.row(&expected, "closer");
    assert_eq!(dev.read(&stored, "closer"), expected);
}

#[test]
fn an_older_closers_held_answer_gets_a_since_but_no_notices() {
    use super::dev_layouts::Dev;
    use skein_lib::Wall;
    use temper_engine_domain_forge_closer::relay::{self, Form, Held, Message, Room, Stands};
    const SINCE: Wall = Wall::from_nanos(3_000_000_000);
    const ANSWER: &[u8] = b"Use known reds.";
    const BOUND: u32 = 512;
    let legacy = Stands::Asked { question: GOAL_QUESTION, answer: Some(ANSWER.into()), held: None };
    let mut dev = Dev::new();
    let mut relay = standing_question(legacy.clone());
    relay.replies = Box::new([]);
    let stored = dev.row(&closer_relay_row(relay), "closer");
    let forge::Stored::Closer(restored) = dev.read(&stored, "closer") else { panic!("closer row") };
    assert_eq!(restored.relay.open.as_ref().expect("held legacy answer").stands, legacy);

    let held = relay::relay(&restored.relay, CLOSER_GOAL, false, Room { left: 0, batch: 1 }, BOUND, SINCE, false);
    assert!(held.notice.is_none());
    assert!(held.messages.is_empty());
    assert!(held.more);
    assert_eq!(
        held.relay.open.as_ref().expect("backfilled hold").stands,
        Stands::Asked {
            question: GOAL_QUESTION,
            answer: Some(ANSWER.into()),
            held: Some(Held { since: SINCE, notified: false }),
        }
    );

    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits");
    let row = closer_relay_row(held.relay);
    let persisted = records::decode(&records::encode(&row), &bounds).expect("current row reads");
    assert_eq!(persisted, row);
    let forge::Stored::Closer(persisted) = persisted else { panic!("closer row") };
    let next =
        relay::relay(&persisted.relay, CLOSER_GOAL, false, Room { left: 1, batch: 1 }, BOUND, Wall::EPOCH, false);
    assert!(next.notice.is_none());
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: CLOSER_LANDING, form: Form::Answer { question: LANDING_QUESTION }, words: ANSWER.into() },]
    );
    assert!(next.relay.open.is_none());
}

#[test]
fn an_older_closer_defaults_superseded_changes_to_empty() {
    use super::dev_layouts::Dev;
    use temper_engine_domain_forge_closer::relay::Relay;
    let forge::Stored::Closer(mut row) = closer_relay_row(Relay::new()) else { panic!("closer fixture") };
    row.close_changes = Box::new([CLOSER_SETTLEMENT]);
    let expected = forge::Stored::Closer(row.clone());
    row.superseded = Box::new([CLOSER_SETTLEMENT]);
    let mut dev = Dev::new();
    let stored = dev.row(&forge::Stored::Closer(row), "closer");
    assert_eq!(dev.read(&stored, "closer"), expected);
}

#[test]
fn a_closers_waiting_questions_alone_keep_the_relay_present() {
    use temper_engine_domain_forge_closer::relay::Relay;
    const WAITING_BOUND: u32 = 2; // One slot per possible open step.
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits");
    let waiting = waiting_questions();
    assert_eq!(bounds.limits().closing_relay_waiting, WAITING_BOUND);
    assert_eq!(u32::try_from(waiting.len()).expect("fixture count"), WAITING_BOUND);
    let row = closer_relay_row(Relay { waiting, ..Relay::new() });
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

#[test]
fn a_closers_third_waiting_question_is_refused_by_the_decoder() {
    use temper_engine_domain_forge_closer::relay::{Question, Relay};
    const EXTRA_QUESTION: u64 = 13;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits");
    let waiting = waiting_questions();
    let [first, second] = waiting.as_ref() else { panic!("two waiting questions") };
    let extra = Question { number: EXTRA_QUESTION, ..first.clone() };
    let row = closer_relay_row(Relay { waiting: Box::new([first.clone(), second.clone(), extra]), ..Relay::new() });
    assert_eq!(records::decode(&records::encode(&row), &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn a_closer_with_a_truncated_relay_is_malformed() {
    const EMPTY_SUPERSEDED: [u8; 4] = 0_u32.to_be_bytes();
    let row = closer_relay_row(temper_engine_domain_forge_closer::relay::Relay::new());
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits");
    let bytes = records::encode(&row);
    // The absent relay's tag precedes the empty superseded list's u32 count.
    let without_list = bytes.strip_suffix(EMPTY_SUPERSEDED.as_slice()).expect("empty superseded list");
    let legacy = without_list.strip_suffix(&[0]).expect("absent relay tag");
    assert_eq!(records::decode(legacy, &bounds), Err(crate::Undecodable::Malformed));
    assert_eq!(records::decode(&legacy[..legacy.len() - 1], &bounds), Err(crate::Undecodable::Malformed));
    let mut truncated = skein_lib::Writer::new(legacy.len() + 1);
    truncated.put(legacy).unwrap();
    truncated.put(&[1]).unwrap(); // A present relay requires its payload.
    assert_eq!(records::decode(&truncated.finish(), &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn a_closers_selection_progress_round_trips_with_its_owned_bytes() {
    use temper_engine_domain_forge_closer::{Closer, PullSelection, SelectionProgress};
    let attempt = skein_forge::api::JobAttempt { head: [3; 32], run: 4, job: 5, attempt: 2 };
    let mut closer = Closer::new();
    closer.selections = Box::new([
        PullSelection::new(1, None),
        PullSelection::new(2, Some(attempt.head)),
        PullSelection {
            pull: 3,
            head: Some(attempt.head),
            progress: SelectionProgress::Jobs { attempts: Box::new([attempt, attempt]), next: 1 },
        },
        PullSelection {
            pull: 4,
            head: Some(attempt.head),
            progress: SelectionProgress::Done { line: Some(b"selection: full".as_slice().into()) },
        },
        PullSelection { pull: 5, head: Some(attempt.head), progress: SelectionProgress::Done { line: None } },
    ]);
    let row = forge::Stored::Closer(forge::CloserRow {
        task: 8,
        goal: 7,
        closer,
        making: None,
        close_changes: Box::new([]),
        superseded: Box::new([]),
        landing_changes: Box::new([]),
        relay: temper_engine_domain_forge_closer::relay::Relay::new(),
        done: false,
    });
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let meter = skein_world::domain::heap::Meter::new();
    meter.start();
    let cloned = row.clone();
    let held = meter.end().held();
    assert_eq!(forge::stored_bytes(&cloned), Some(held));
    drop(cloned);
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

#[test]
fn prefix_judgement_and_candidate_failure_snapshots_survive_the_store() {
    use temper_engine_domain_forge_change::{Judgement, Repair, State};
    let target = Judgement {
        landing: 81,
        round: 3,
        pull: 17,
        branch: b"temper/1/l81".as_slice().into(),
        base: b"goal".as_slice().into(),
        prefix: [4; 32],
    };
    for state in [
        State::Judging { head: [1; 32], ready_since: skein_lib::Wall::from_nanos(7), target, asked: Box::new([9, 12]) },
        State::Repairing { requested: true, why: Repair::LandingAt { gate: 3, candidate: [4; 32], below: None } },
    ] {
        new_family_round_trip(&fixtures::change_hot(fixtures::change_row(state)));
    }
    new_family_round_trip(&forge::Stored::Receipt {
        key: forge::ReceiptKey {
            project: 5,
            repository: skein_forge::api::Repository { forge: 1, repository: 2 },
            check: forge::Check::Gate(b"test".as_slice().into()),
            key: [3; 32],
        },
        receipt: forge::Receipt {
            report: temper_engine_domain_forge::ReceiptReport::Unset,
            base: None,
            jobs: Box::new([]),
            failures: None,
            passed: false,
            candidate: [4; 32],
            ended: skein_lib::Wall::from_nanos(5),
            run: forge::ReceiptRun { id: 6, number: 7, url: b"https://forge/run/6".as_slice().into() },
            logs: b"Failed job: test\nFailed step: compile\nEnd of its log:\nbroken source\n".as_slice().into(),
        },
    });
}

#[test]
fn named_finding_answers_keep_the_deciding_snapshot_and_its_owned_bytes() {
    use forge::{FindingDone as Done, FindingRefusal as Refused, Holder};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let outcomes = [
        Ok(Done::Filed(Holder::Task(40))),
        Ok(Done::Filed(Holder::Party(7))),
        Ok(Done::Handed(Holder::Role(0))),
        Ok(Done::Routed(99)),
        Ok(Done::Backlog),
        Ok(Done::Dropped),
        Err(Refused::NoArea(Box::new([b"tests".as_slice().into(), b"ui".as_slice().into()]))),
        Err(Refused::NotYours(Holder::Task(42))),
        Err(Refused::Full(256)),
        Err(Refused::NoFinding),
        Err(Refused::NoGoal),
        Err(Refused::Ended),
        Err(Refused::Words(b"title".as_slice().into())),
    ];
    for result in outcomes {
        let row = forge::Stored::Call {
            key: forge::NamedCall { task: 40, attempt: 1, completion: 2, position: 3 },
            answer: forge::CallPayload::Finding(Box::new(forge::FindingReply {
                project: 3,
                number: 12,
                area: Some(b"tests".as_slice().into()),
                result,
                more: Box::default(),
                landing: None,
            })),
        };
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let cloned = row.clone();
        let held = meter.end().held();
        assert_eq!(forge::stored_bytes(&cloned), Some(held));
        drop(cloned);
        let encoded = records::encode(&row);
        assert_eq!(records::decode(&encoded, &bounds), Ok(row));
    }
}

#[test]
fn named_repair_and_blame_replies_round_trip_their_variants_context_and_owned_bytes() {
    use forge::{FindingDone as Done, FindingRefusal as Refused};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for result in [
        Ok(Done::Repaired(99)),
        Ok(Done::Blamed),
        Err(Refused::Words(b"repairs are off in project 3".as_slice().into())),
    ] {
        for landing in [None, Some([0xab; 32])] {
            let row = forge::Stored::Call {
                key: forge::NamedCall { task: 40, attempt: 1, completion: 2, position: 3 },
                answer: forge::CallPayload::Finding(Box::new(forge::FindingReply {
                    project: 3,
                    number: 12,
                    area: None,
                    result: result.clone(),
                    more: Box::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
                    landing,
                })),
            };
            let meter = skein_world::domain::heap::Meter::new();
            meter.start();
            let cloned = row.clone();
            let held = meter.end().held();
            assert_eq!(forge::stored_bytes(&cloned), Some(held));
            drop(cloned);
            assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        }
    }
}

#[test]
fn findings_round_trip_full_rows_options_and_topic_and_refuse_wrong_keys() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for kind in [
        forge::FindingKind::Qa,
        forge::FindingKind::Flake,
        forge::FindingKind::Red,
        forge::FindingKind::Review,
        forge::FindingKind::Feedback,
    ] {
        for (state, severity) in [
            (forge::FindingState::New, forge::Severity::Low),
            (forge::FindingState::Backlog, forge::Severity::Medium),
            (forge::FindingState::Routed, forge::Severity::High),
            (forge::FindingState::Dropped, forge::Severity::High),
        ] {
            for options in [false, true] {
                let mut row = fixtures::finding();
                row.kind = kind;
                row.state = state;
                row.severity = severity;
                if !options {
                    row.area = None;
                    row.why = None;
                    row.hander = None;
                    row.goal = None;
                    row.reason = None;
                    row.signature = None;
                    row.seen = None;
                    row.blame = None;
                    row.cleared = None;
                    row.repair = None;
                    row.filer = forge::Filer::Party(1);
                }
                let row = forge::Stored::Finding(row);
                let bytes = records::encode(&row);
                let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
                assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
                let wrong = crate::keys::encode(&forge::Key::Finding { project: 1, number: 1 }, u32::MAX).unwrap();
                assert_eq!(records::row(&wrong, &bytes, &bounds), Err(crate::Undecodable::Malformed));
            }
        }
    }
    let row = forge::Stored::Subscription(forge::Subscriber {
        task: 1,
        number: 2,
        topic: forge::Topic::Findings { project: u32::MAX },
        own_change: None,
        goal_tasks: None,
        paths: Box::default(),
    });
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
    let row = forge::Stored::Finding(forge::Finding { title: Box::default(), ..fixtures::finding() });
    assert_eq!(records::decode(&records::encode(&row), &bounds), Err(crate::Undecodable::Malformed));
}

#[test]
fn findings_round_trip_full_repairs_and_moved_blame() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("store bounds");
    for judging in [
        Some(forge::RepairJudging::Landing([255; 32])),
        Some(forge::RepairJudging::After(u64::MAX)),
        Some(forge::RepairJudging::Release),
        None,
    ] {
        for by in [forge::Filer::Task(u64::MAX), forge::Filer::Party(u64::MAX)] {
            let mut row = fixtures::finding();
            let repair = row.repair.as_mut().expect("full repair");
            repair.judging = judging;
            assert_eq!(repair.failing.len(), 64);
            for id in &repair.failing {
                assert_eq!(id.len(), 512);
            }
            assert!(repair.until.is_some());
            row.blame.as_mut().expect("full blame").moved.as_mut().expect("move").by = by;
            let stored = forge::Stored::Finding(row);
            assert!(
                records::decode(&records::encode(&stored), &bounds) == Ok(stored),
                "repair and moved blame round-trip intact"
            );
        }
    }
}

#[test]
fn sign_offs_round_trip_on_changes_and_landing_logs() {
    use skein_lib::Wall;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for count in [0, 1, forge::SIGN_OFFS] {
        let mut signed = skein_lib::List::with_capacity(u32::try_from(forge::SIGN_OFFS).unwrap());
        for n in 0..count {
            signed
                .push(forge::SignOff {
                    person: u64::try_from(n).unwrap() + 1,
                    head: [u8::try_from(n).unwrap(); 32],
                    at: Wall::from_nanos(u64::MAX - u64::try_from(n).unwrap()),
                })
                .unwrap();
        }
        let signed = signed.into_boxed();
        let mut change = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
        change.signed = signed.clone();
        let mut log = landing_log(1, 1, 1);
        let forge::Stored::LandingLog(row) = &mut log else { panic!("log fixture") };
        row.landings[0].changes[0].signed = signed;
        for row in [fixtures::change_hot(change), log] {
            assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary codec assertions build and mutate bounded byte fixtures"
)]
fn devices_round_trip_at_both_bounds_and_refuse_malformed_rows_and_keys() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for (endpoint, label) in [(1, 1), (1024, 60)] {
        let row = forge::Stored::Device(forge::Device {
            party: u64::MAX,
            number: u32::MAX,
            endpoint: vec![b'e'; endpoint].into_boxed_slice(),
            label: vec![b'l'; label].into_boxed_slice(),
            p256dh: [4; 65],
            auth: [7; 16],
            at: skein_lib::Wall::from_nanos(u64::MAX),
        });
        let bytes = records::encode(&row);
        assert_eq!(bytes.get(2), Some(&36), "Device is Value's tag");
        assert_eq!(records::decode(&bytes, &bounds), Ok(row.clone()));
        let key = forge::stored_key(&row);
        let encoded = crate::keys::encode(&key, u32::MAX).unwrap();
        assert_eq!(encoded.first(), Some(&43));
        assert!(encoded.as_ref() > crate::keys::encode(&forge::Key::Closer(u64::MAX), u32::MAX).unwrap().as_ref());
        assert_eq!(records::row(&encoded, &bytes, &bounds), Ok(row));
        for end in 0..bytes.len() {
            records::decode(&bytes[..end], &bounds).unwrap_err();
        }
        for end in 0..encoded.len() {
            assert!(crate::keys::decode(&encoded[..end], &fixtures::LIMITS).is_none());
        }
        let mut tail = bytes.to_vec();
        tail.push(0);
        records::decode(&tail, &bounds).unwrap_err();
        let mut tail = encoded.to_vec();
        tail.push(0);
        assert!(crate::keys::decode(&tail, &fixtures::LIMITS).is_none());
    }
    for (endpoint, label) in [(vec![], vec![b'l']), (vec![b'e'], vec![]), (vec![b'e'], vec![255])] {
        let row = forge::Stored::Device(forge::Device {
            party: 1,
            number: 1,
            endpoint: endpoint.into_boxed_slice(),
            label: label.into_boxed_slice(),
            p256dh: [4; 65],
            auth: [7; 16],
            at: skein_lib::Wall::EPOCH,
        });
        records::decode(&records::encode(&row), &bounds).unwrap_err();
    }
    let mut oversized = vec![43];
    oversized.extend_from_slice(&1_u64.to_be_bytes());
    oversized.extend_from_slice(&(u64::from(u32::MAX) + 1).to_be_bytes());
    assert!(crate::keys::decode(&oversized, &fixtures::LIMITS).is_none());
    for (endpoint, label) in [(1025, 60), (1024, 61)] {
        v25::Device {
            party: 1,
            number: 1,
            endpoint: vec![b'e'; endpoint].into_boxed_slice(),
            label: vec![b'l'; label].into_boxed_slice(),
            p256dh: [4; 65],
            auth: [7; 16],
            at: 0,
        }
        .encode(&v25::CEILINGS)
        .unwrap_err();
    }
}

#[test]
fn landed_drafts_round_trip_the_smallest_and_full_rows_and_keys() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for full in [false, true] {
        let path: Box<[u8]> = if full {
            let mut path = skein_lib::Writer::new(1024);
            path.put(&[b'p'; 1021]).unwrap();
            path.put(b".md").unwrap();
            path.finish()
        } else {
            b".md".as_slice().into()
        };
        let row = forge::Stored::LandedDraft(forge::LandedDraft {
            repository: skein_forge::api::Repository {
                forge: if full { u16::MAX } else { 0 },
                repository: if full { u32::MAX } else { 0 },
            },
            path,
            project: if full { u32::MAX } else { 0 },
            branch: if full { Box::from([b'b'; 1024]) } else { Box::new([]) },
            commit: if full { [255; 32] } else { [0; 32] },
            title: if full { Box::from([b't'; 200]) } else { Box::new([]) },
            writer: if full { u64::MAX } else { 0 },
            at: skein_lib::Wall::from_nanos(if full { u64::MAX } else { 0 }),
            chat: full.then_some(u64::MAX),
            goal: full.then_some(u64::MAX),
            ready: full,
        });
        let bytes = records::encode(&row);
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
        assert_eq!(key.first(), Some(&41));
        assert_eq!(crate::keys::decode(&key, &fixtures::LIMITS), Some(forge::stored_key(&row)));
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
    }
}

#[test]
fn pull_size_is_omitted_without_changing_the_stored_pull() {
    use skein_forge::api::{Answer, Size};
    let mut pull = fixtures::pull();
    let expected = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(Answer::Pull(pull.clone())))));
    pull.size = Some(Size { files: 19, additions: u64::MAX, deletions: 12 });
    let row = fixtures::named_call(forge::CallPayload::Read(Box::new(Ok(Answer::Pull(pull)))));
    let encoded = records::encode(&row);
    assert_eq!(encoded, records::encode(&expected), "size changes no stored byte");
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(&encoded, &bounds), Ok(expected));
}

#[test]
fn a_change_holding_a_retired_route_or_state_never_decodes() {
    use crate::v25 as w;
    use temper_engine_domain_forge_change::State;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let row = fixtures::change_hot(fixtures::change_row(State::Producing { requested: true }));
    let encoded = records::encode(&row);
    for route in [true, false] {
        let row = w::Row::decode(bounds.limits(), &mut skein_lib::Reader::new(&encoded)).unwrap();
        let w::Value::ChangeHot(change) = row.value else { panic!("stored change") };
        let mut parts = change;
        let mut open = skein_lib::List::with_capacity(1);
        if !route {
            open.push(w::Open {
                asker: 7,
                number: 1,
                line: Box::new([]),
                words: Box::new([]),
                stands: w::Stands::Retired,
            })
            .unwrap();
        }
        parts.questions = w::Questions {
            route: if route { w::Route::Retired } else { w::Route::Up },
            taken: 1,
            open,
            waiting: skein_lib::List::with_capacity(0),
            answered: skein_lib::List::with_capacity(0),
            amended: skein_lib::List::with_capacity(0),
            replies: skein_lib::List::with_capacity(0),
        };
        let row = w::Row { value: w::Value::ChangeHot(parts) };
        let bytes = row.encode(&w::CEILINGS).unwrap();
        assert_eq!(records::decode(&bytes, &bounds), Err(crate::Undecodable::Malformed));
    }
}

#[test]
fn a_pending_merge_job_end_and_its_readback_marker_round_trip() {
    use forge::history as h;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for written in [false, true] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
        row.job = Some(h::JobRecord {
            step: h::JobStep {
                task: 81,
                attempt: 2,
                job: h::Job::Merge,
                outcome: Some(h::JobOutcome::Done),
                head: None,
            },
            round: 0,
            members: Box::new([]),
            head: Some([3; 32]),
            written,
        });
        let record = fixtures::change_hot(row);
        let key = crate::keys::encode(&forge::stored_key(&record), u32::MAX).expect("key");
        assert_eq!(records::row(&key, &records::encode(&record), &bounds), Ok(record));
    }
}

#[test]
fn change_parts_round_trip_agree_with_keys_and_join_the_generated_rows() {
    use skein_lib::Rng;
    let bounds = crate::bounds(&fixtures::LIMITS, 8192).unwrap();
    let mut random = Rng::new(0x12_2f_30);
    for state in fixtures::change_states().into_boxed() {
        let mut row = fixtures::change_row(state);
        if let temper_engine_domain_forge_change::State::Taking { .. } = row.change.state {
            row.source = Some(forge::ChangeSource { branch: b"docs/x".as_slice().into(), at: Some([4; 32]) });
            row.kept = Some([4; 32]);
        }
        row.task = random.next_u64();
        row.change.task = row.task;
        row.priority =
            i32::from_be_bytes(u32::try_from(random.next_u64() & u64::from(u32::MAX)).unwrap().to_be_bytes());
        row.body = Box::new([42; 6144]);
        if row.task & 1 != 0 {
            use jig_domain_tasks::{LlmChoice, ModelEffort};
            use temper_engine_domain_forge_change::{Early, Open, Stands, StartedFrom};
            row.room = Some(random.next_u64());
            let model = ModelEffort { model: 0, effort: None };
            row.producing_llm =
                Some(LlmChoice { account: None, main: model, subagents: Box::new([model]), later: None });
            row.review_llm = row.producing_llm.clone();
            row.close = Some(b"close".as_slice().into());
            row.merge_result = Some(b"merged".as_slice().into());
            row.questions.open = Box::new([Open {
                asker: 1,
                number: 2,
                line: b"question".as_slice().into(),
                words: b"whole question".as_slice().into(),
                stands: Stands::Held { answer: b"answer".as_slice().into(), question: 0, blocked: None },
            }]);
            row.change.early = Early::Started(Box::new([StartedFrom { task: 2, head: [3; 32] }]));
        }
        let (cold, hot) = row.clone().split();
        assert_eq!(forge::ChangeRow::join(cold.clone(), hot.clone()), row);
        let cold_bytes = records::encode(&forge::Stored::ChangeCold(Box::new(cold.clone())));
        let hot_bytes = records::encode(&forge::Stored::ChangeHot(Box::new(hot.clone())));
        assert!(hot_bytes.len() < row.body.len(), "the hot part omits the body");
        assert!(cold_bytes.len() >= row.body.len(), "the cold part keeps the body");
        for (part, other) in [
            (forge::Stored::ChangeCold(Box::new(cold.clone())), forge::Key::ChangeCold(row.task.wrapping_add(1))),
            (forge::Stored::ChangeHot(Box::new(hot)), forge::Key::ChangeHot(row.task.wrapping_add(1))),
        ] {
            let bytes = records::encode(&part);
            let key = crate::keys::encode(&forge::stored_key(&part), u32::MAX).unwrap();
            assert_eq!(
                records::row(&key, &bytes, &bounds),
                Ok(part.clone()),
                "hot {} cold {}",
                hot_bytes.len(),
                cold_bytes.len()
            );
            assert_eq!(
                records::row(&crate::keys::encode(&other, u32::MAX).unwrap(), &bytes, &bounds),
                Err(crate::Undecodable::Malformed)
            );
            let meter = skein_world::domain::heap::Meter::new();
            meter.start();
            let cloned = part.clone();
            let held = meter.end().held();
            assert_eq!(forge::stored_bytes(&cloned), Some(held));
        }
    }
}

#[test]
fn milestone_rows_and_change_membership_round_trip_in_family_order() {
    use forge::{MilestoneEnd, MilestoneLanding, MilestoneOutcome, MilestoneRow};
    use skein_lib::Wall;
    let bounds = crate::bounds(&fixtures::LIMITS, 8192).unwrap();
    for outcome in
        [None, Some(MilestoneOutcome::Complete), Some(MilestoneOutcome::Ended), Some(MilestoneOutcome::Closed)]
    {
        let record = forge::Stored::Milestone(MilestoneRow {
            project: 5,
            task: 42,
            name: "é".repeat(128).into_bytes().into_boxed_slice(),
            last_landing: Some(MilestoneLanding { at: Wall::from_nanos(11), goal: 43, change: 44 }),
            ended: match outcome {
                Some(outcome) => {
                    Some(MilestoneEnd { at: Wall::from_nanos(12), outcome, done: 7, failed: 2, closed: 3, spent: 29 })
                }
                None => None,
            },
        });
        new_family_round_trip(&record);
        let key = crate::keys::encode(&forge::stored_key(&record), u32::MAX).unwrap();
        let mut expected = skein_kv::KeyWriter::new(u32::MAX);
        expected.tag(49).u64(42);
        assert_eq!(key, expected.finish().unwrap());
        assert_eq!(records::row(&key, &records::encode(&record), &bounds), Ok(record));
    }
    let keys = [
        forge::Key::ProjectFacts { project: u32::MAX },
        forge::Key::ChangeCold(u64::MAX),
        forge::Key::ChangeHot(u64::MAX),
        forge::Key::Milestone(0),
        forge::Key::Milestone(u64::MAX),
        forge::Key::HistoryProject { project: 0 },
    ];
    for pair in keys.windows(2) {
        let [first, second] = pair else { unreachable!() };
        assert!(first < second);
        assert!(crate::keys::encode(first, u32::MAX).unwrap() < crate::keys::encode(second, u32::MAX).unwrap());
    }
    let mut change = fixtures::change_row(temper_engine_domain_forge_change::State::Ended);
    change.milestone = Some(42);
    let (cold, hot) = change.clone().split();
    assert_eq!(forge::ChangeRow::join(cold.clone(), hot), change);
    let record = forge::Stored::ChangeCold(Box::new(cold));
    let key = crate::keys::encode(&forge::stored_key(&record), u32::MAX).unwrap();
    assert_eq!(records::row(&key, &records::encode(&record), &bounds), Ok(record));
}

#[test]
fn receipt_reports_round_trip_their_states_and_owned_bytes() {
    for report in [
        forge::ReceiptReport::Unset,
        forge::ReceiptReport::Unread,
        forge::ReceiptReport::Many { bound: 16 },
        forge::ReceiptReport::Tests(Box::new([
            forge::ReceiptTest { key: b"test binary fuzzy::a".as_slice().into(), fuzzy: true },
            forge::ReceiptTest { key: b"test binary b".as_slice().into(), fuzzy: false },
        ])),
    ] {
        let row = forge::Stored::Receipt {
            key: forge::ReceiptKey {
                project: 5,
                repository: fixtures::REPO,
                check: forge::Check::Gate(b"gate".as_slice().into()),
                key: [7; 32],
            },
            receipt: forge::Receipt {
                report,
                base: Some([1; 32]),
                jobs: Box::new([]),
                logs: Box::new([]),
                failures: None,
                passed: false,
                run: forge::ReceiptRun { id: 7, number: 8, url: b"run/7".as_slice().into() },
                candidate: [3; 32],
                ended: skein_lib::Wall::EPOCH,
            },
        };
        new_family_round_trip(&row);
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let cloned = row.clone();
        let held = meter.end().held();
        assert_eq!(forge::stored_bytes(&cloned), Some(held));
        drop(cloned);
    }
}

#[test]
fn receipt_row_report_defaults_without_changing_tier_rows() {
    let mut text = None;
    for family in crate::OWNER_SET.families {
        if family.name == "temper-store" {
            text = Some(family.current);
        }
    }
    let text = text.expect("registered application family");
    let old = text.replace("report: ReceiptReport = unset ", "").replace("report: ReceiptReport ", "");
    assert_ne!(text, old);
    let old = skein_codec::schema::parse(old.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(text.as_bytes()).unwrap();
    let receipt = fixtures::tier_row().result.unwrap();
    assert_eq!(receipt.report, forge::ReceiptReport::Unset);
    let receipt = forge::Stored::Receipt {
        key: forge::ReceiptKey {
            project: 5,
            repository: fixtures::REPO,
            check: forge::Check::Gate(b"gate".as_slice().into()),
            key: [7; 32],
        },
        receipt,
    };
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for (root, row) in
        [(b"ReceiptRow".as_slice(), receipt), (b"TierRow".as_slice(), forge::Stored::Tier(fixtures::tier_row()))]
    {
        let bytes = records::encode(&row);
        let to_old = skein_codec::layout::Plan::derive(
            &new,
            new.root(root, None).unwrap(),
            &[],
            &old,
            old.root(root, None).unwrap(),
            &[],
        )
        .unwrap();
        let to_new = skein_codec::layout::Plan::derive(
            &old,
            old.root(root, None).unwrap(),
            &[],
            &new,
            new.root(root, None).unwrap(),
            &[],
        )
        .expect("a receipt row report defaults and tier layouts stay identical");
        let mut workspace = skein_codec::layout::Workspace::new();
        assert!(workspace.fit(&to_old));
        let mut before = skein_lib::Buffer::with_capacity(131_072);
        to_old.convert(&bytes[3..], &mut before, &mut workspace).unwrap();
        if root == b"TierRow" {
            assert_eq!(before.as_slice(), &bytes[3..], "tier bytes are unchanged");
        }
        assert!(workspace.fit(&to_new));
        let mut after = skein_lib::Buffer::with_capacity(131_072);
        to_new.convert(before.as_slice(), &mut after, &mut workspace).unwrap();
        let restored = skein_lib::bytes::concat(&[&bytes[..3], after.as_slice()]);
        assert_eq!(records::decode(&restored, &bounds).unwrap(), row);
    }
}

#[test]
fn a_stale_updates_first_refusal_survives_the_store() {
    use temper_engine_domain_forge_change::{EffectResult, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for stale in [None, Some(skein_lib::Wall::from_nanos(17))] {
        let mut row = fixtures::change_row(State::Preparing {
            from: [1; 32],
            main: [2; 32],
            ready_since: skein_lib::Wall::from_nanos(0),
        });
        row.effect = EffectResult::Stale;
        row.change.stale = stale;
        let stored = fixtures::change_hot(row);
        assert_eq!(records::decode(&records::encode(&stored), &bounds), Ok(stored));
    }
}

#[test]
fn a_flake_terminal_and_hold_round_trip_in_the_hot_part() {
    use temper_engine_domain_forge_change::{Hold, Repair, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let repairing = State::Repairing { requested: true, why: Repair::Landing { gate: 0 } };
    for state in [repairing.clone(), State::Held { was: Box::new(repairing), why: Hold::Flake }] {
        let mut row = fixtures::change_row(state);
        row.delegate_flake = true;
        row.delegate_head = Some([6; 32]);
        let (cold, hot) = row.clone().split();
        let stored = forge::Stored::ChangeHot(Box::new(hot));
        let decoded = records::decode(&records::encode(&stored), &bounds).unwrap();
        assert_eq!(decoded, stored);
        let forge::Stored::ChangeHot(decoded) = decoded else { panic!("change hot part") };
        assert_eq!(forge::ChangeRow::join(cold, *decoded), row);
    }
}

#[test]
fn a_close_refusal_round_trips_in_the_hot_part() {
    use temper_engine_domain_forge_change::{CloseAt, CloseRefused, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let mut row = fixtures::change_row(State::Producing { requested: true });
    row.close_at = Some(CloseAt {
        branch: [3; 32],
        landing: [4; 32],
        asked: true,
        refused: Some(CloseRefused { counts: Some([5; 32]) }),
    });
    let (cold, hot) = row.clone().split();
    let stored = forge::Stored::ChangeHot(Box::new(hot));
    let decoded = records::decode(&records::encode(&stored), &bounds).expect("current change record");
    let forge::Stored::ChangeHot(decoded) = decoded else { panic!("change hot part") };
    assert_eq!(forge::ChangeRow::join(cold, *decoded), row);
}

#[test]
fn a_changes_regate_round_trips_in_its_hot_part_and_joins_unchanged() {
    use temper_engine_domain_forge_change::State;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for check in [forge::Check::Gate(b"gate".as_slice().into()), forge::Check::Tier(b"tier".as_slice().into())] {
        let mut row = fixtures::change_row(State::Ready { head: [3; 32], ready_since: skein_lib::Wall::EPOCH });
        row.regate = Some(forge::Regate {
            receipt: forge::ReceiptKey { project: 5, repository: row.repository, check, key: [7; 32] },
            run: u64::MAX,
        });
        let (cold, hot) = row.clone().split();
        let stored = forge::Stored::ChangeHot(Box::new(hot.clone()));
        let decoded = records::decode(&records::encode(&stored), &bounds).unwrap();
        assert_eq!(decoded, stored);
        let forge::Stored::ChangeHot(decoded) = decoded else { panic!("change hot part") };
        assert_eq!(forge::ChangeRow::join(cold, *decoded), row);
        assert_eq!(hot.regate, row.regate);
    }
}

/// A project's former brief and run are dropped by the layout's derived plan (store.md, 2).
#[test]
fn project_facts_written_with_a_brief_and_run_decode_with_both_dropped() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family.current.replace(
        "record ProjectFacts { project: u32 landed: u64 last_landed: u64 }",
        "record ProjectFacts { project: u32 landed: u64 last_landed: u64 brief: option StatusBrief running: option StatusRunning }",
    );
    assert_ne!(before, family.current);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let facts = forge::Stored::ProjectFacts(forge::ProjectFacts {
        project: u32::MAX,
        landed: u64::MAX,
        last_landed: skein_lib::Wall::from_nanos(u64::MAX),
    });
    let brief = v25::StatusBrief {
        sentence: b"Project's old brief".as_slice().into(),
        lede: b"Not any person's".as_slice().into(),
        at: 11,
        landed: 7,
        findings: Some([3; 32]),
    };
    let run = v25::StatusRunning { task: Some(19), landed: 9, since: 13, findings: Some([5; 32]) };
    let facts_bytes = records::encode(&facts);
    let mut bytes = skein_lib::Writer::new(
        facts_bytes.len()
            + usize::try_from(brief.measure(&v25::CEILINGS).unwrap() + run.measure(&v25::CEILINGS).unwrap()).unwrap()
            + 2,
    );
    bytes.put(&facts_bytes).unwrap();
    bytes.put(&[1]).unwrap();
    brief.write(&v25::CEILINGS, &mut bytes).unwrap();
    bytes.put(&[1]).unwrap();
    run.write(&v25::CEILINGS, &mut bytes).unwrap();
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"Row", None).unwrap(),
        &[],
        &new,
        new.root(b"Row", None).unwrap(),
        &[],
    )
    .expect("dropped fields derive with no job");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(131_072);
    plan.convert(&bytes.finish(), &mut out, &mut work).unwrap();
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(out.as_slice(), &bounds), Ok(facts));
}

#[test]
fn a_no_test_terminal_and_hold_round_trip_in_the_hot_part() {
    use temper_engine_domain_forge_change::{Hold, Repair, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let repairing = State::Repairing { requested: true, why: Repair::Landing { gate: 0 } };
    for state in [repairing.clone(), State::Held { was: Box::new(repairing), why: Hold::NoTest }] {
        let mut row = fixtures::change_row(state);
        row.delegate_no_test = true;
        row.delegate_head = Some([6; 32]);
        let (cold, hot) = row.clone().split();
        let stored = forge::Stored::ChangeHot(Box::new(hot));
        let decoded = records::decode(&records::encode(&stored), &bounds).unwrap();
        assert_eq!(decoded, stored);
        let forge::Stored::ChangeHot(decoded) = decoded else { panic!("change hot part") };
        assert_eq!(forge::ChangeRow::join(cold, *decoded), row);
    }
}

/// A registered field defaults at layout conversion, not in the fixed-layout decoder.
#[test]
fn a_change_row_written_before_no_test_defaults_its_verdict_to_false() {
    let mut family = None;
    for candidate in crate::OWNER_SET.families {
        if candidate.name == "temper-store" {
            family = Some(candidate);
        }
    }
    let family = family.unwrap();
    let before = family.current.replace(" delegate_no_test: bool = false", "");
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let stored = fixtures::change_hot(fixtures::change_row(temper_engine_domain_forge_change::State::Producing {
        requested: true,
    }));
    let mut bytes = records::encode(&stored);
    // Write the row at the preceding layout, then read it as the registry does.
    for (source, target) in [(&new, &old), (&old, &new)] {
        let plan = skein_codec::layout::Plan::derive(
            source,
            source.root(b"Row", None).unwrap(),
            &[],
            target,
            target.root(b"Row", None).unwrap(),
            &[],
        )
        .unwrap();
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(131_072);
        plan.convert(&bytes, &mut out, &mut work).unwrap();
        bytes = out.as_slice().into();
    }
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    assert_eq!(records::decode(&bytes, &bounds), Ok(stored));
}

#[test]
fn the_writes_family_keeps_a_branch_and_its_bounded_ordered_entries() {
    use forge::writes::{Operation, Sender, WRITES, WriteState, Writes, Written};
    let name = forge::Name {
        forge: fixtures::REPO.forge,
        repository: fixtures::REPO.repository,
        what: forge::What::Branch(Box::new([b"goals".as_slice().into(), b"one".as_slice().into()])),
    };
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let limit = u64::try_from(WRITES).unwrap();
    let mut entries = skein_lib::List::with_capacity(u32::try_from(WRITES).unwrap());
    for order in 1..=limit {
        entries
            .push(Written {
                order,
                operation: Operation::Update { pull: 19, before: [1; 32], base: [2; 32], after: None },
                sender: Sender::Entry { task: 4, entry: order },
                state: WriteState::Uncertain,
            })
            .unwrap();
    }
    let mut row = Writes {
        name: name.clone(),
        entries: entries.into_boxed(),
        head: Some([2; 32]),
        next: limit.checked_add(1).unwrap(),
    };
    let key = forge::Key::Writes(name);
    let key_bytes = crate::keys::encode(&key, u32::MAX).unwrap();
    assert_eq!(key_bytes[0], crate::keys::WRITES);
    assert_eq!(crate::keys::decode(&key_bytes, &fixtures::LIMITS), Some(key));
    let stored = forge::Stored::Writes(row.clone());
    let bytes = records::encode(&stored);
    assert_eq!(records::row(&key_bytes, &bytes, &bounds), Ok(stored));
    row.next = limit;
    let bytes = records::encode(&forge::Stored::Writes(row));
    assert_eq!(records::row(&key_bytes, &bytes, &bounds), Err(crate::Undecodable::Malformed));
    assert_eq!(v25::CEILINGS.writes_entries, u32::try_from(WRITES).unwrap());
}
