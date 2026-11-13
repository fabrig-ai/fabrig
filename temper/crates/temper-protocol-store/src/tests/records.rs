use super::fixtures;
use crate::{records, v17};
use temper_engine_domain_forge as forge;

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
    let stored = forge::Stored::Change(Box::new(row));
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
    larger.name_bytes = v17::CEILINGS.adopted_host.checked_add(1).expect("one past ceiling");
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
            let row = forge::Stored::Change(Box::new(row));
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
#[test]
fn an_ended_changes_hold_round_trips_in_its_state() {
    use temper_engine_domain_forge_change::{Hold, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for ended_held in [None, Some(Hold::Repairs), Some(Hold::Resolutions), Some(Hold::Rejected), Some(Hold::Stalled)] {
        let mut row = fixtures::change_row(State::Ended);
        row.change.ended_held = ended_held;
        let row = forge::Stored::Change(Box::new(row));
        let bytes = records::encode(&row);
        let wire = v17::Row::decode(&v17::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("encoded row");
        let v17::Value::Change(wire) = wire.into_parts().value else { panic!("change row") };
        assert_eq!(wire.change().state().live(), &v17::ChangeLive::Ended);
        assert_eq!(wire.change().state().holds().len(), u32::from(ended_held.is_some()));
        assert_eq!(records::decode(&bytes, &bounds), Ok(row));
    }
}

#[test]
fn old_ended_rows_decode_with_or_without_a_hold() {
    use temper_engine_domain_forge_change::{Hold, State};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    let row = fixtures::change_row(State::Ended);
    let bytes = records::encode(&forge::Stored::Change(Box::new(row.clone())));
    for ended_held in [None, Some(Hold::Repairs)] {
        let wire = v17::Row::decode(&v17::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("fixture row");
        let v17::Value::Change(wire) = wire.into_parts().value else { panic!("change row") };
        let mut wire = wire.into_parts();
        let mut change = wire.change.into_parts();
        let mut holds = skein_lib::List::with_capacity(1);
        if ended_held.is_some() {
            holds.push(v17::ChangeHold::Repairs).expect("one hold");
        }
        // The unchanged schema's representation, independent of the domain encoder.
        change.state =
            v17::ChangeState::new(&v17::CEILINGS, v17::ChangeStateParts { live: v17::ChangeLive::Ended, holds })
                .expect("old state");
        wire.change = v17::Change::new(&v17::CEILINGS, change).expect("old change");
        let value = v17::Value::Change(Box::new(v17::ChangeRow::new(&v17::CEILINGS, wire).expect("old row")));
        let wire = v17::Row::new(&v17::CEILINGS, v17::RowParts { value }).expect("old row");
        let mut writer = skein_lib::Writer::new(usize::try_from(wire.measure()).expect("row size"));
        wire.encode(&mut writer).expect("measured room");
        let mut expected = row.clone();
        expected.change.ended_held = ended_held;
        let forge::Stored::Change(decoded) = records::decode(&writer.finish(), &bounds).expect("old row decodes")
        else {
            panic!("change row")
        };
        assert_eq!(decoded.change.state, State::Ended);
        assert_eq!(decoded.change.ended_held, ended_held);
        assert_eq!(*decoded, expected);
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
        let row = forge::Stored::Change(Box::new(row));
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
                let row = forge::Stored::Change(Box::new(fixtures::change_row(saved)));
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
            let row = forge::Stored::Change(Box::new(change));
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
        let row = forge::Stored::Change(Box::new(row));
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
    let row = forge::Stored::Change(Box::new(fixtures::change_row(state)));
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
        let row = forge::Stored::Change(Box::new(row));
        let bytes = records::encode(&row);
        assert_eq!(records::decode(&bytes, &bounds), Ok(row), "{delegate:?}");
    }
}

/// The merge delegate and its retained fields survive storage (temper/docs/design/domain/forge/change/repairs.md).
#[test]
fn a_change_row_with_a_merge_delegate_and_a_catch_up_round_trips() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits fit");
    for from in [Some([30; 32]), None] {
        let row = forge::Stored::Change(Box::new(fixtures::merge_change_row(from)));
        assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row), "from {from:?}");
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
            forge::Stored::Change(Box::new(fixtures::change_row(temper_engine_domain_forge_change::State::Landed {
                merge: [7; 32],
                since: skein_lib::Wall::from_nanos(17),
            }))),
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
    const REFUSAL_CASES: u32 = 201;
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
        (jig_protocol_store::people_v16::CEILINGS.refusal_circle_tasks, true),
        (jig_protocol_store::people_v16::CEILINGS.refusal_kept_pending, false),
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
fn every_application_outcome_preserves_its_complete_record() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for outcome in application_outcomes() {
        let row = application(outcome);
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).expect("key");
        let bytes = records::encode(&row);
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row.clone()), "{row:?}");
        assert_eq!(bytes[2], 15, "the Application family tag");
    }
    // Settings judgement words cannot occur in a forge application refusal.
    let limits =
        jig_protocol_store::people_v16::Limits { refusal_judged_why: 0, ..jig_protocol_store::people_v16::CEILINGS };
    assert!(
        jig_protocol_store::people_v16::Refusal::encoded_bound(&limits).expect("bounded facts")
            <= u64::from(v17::CEILINGS.core_refused_refusal)
    );
}
#[test]
fn application_decoding_refuses_malformed_refusal_facts() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for refusal in [Box::from([255]), Box::from([2]), Box::from([9, 0])] {
        let value = v17::CoreRefused::new(&v17::CEILINGS, v17::CoreRefusedParts { refusal }).expect("small piece");
        let application = v17::Application::new(
            &v17::CEILINGS,
            v17::ApplicationParts { token: 9, until: 10, outcome: v17::ApplicationOutcome::CoreRefused(value) },
        )
        .expect("application");
        let row =
            v17::Row::new(&v17::CEILINGS, v17::RowParts { value: v17::Value::Application(application) }).expect("row");
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
fn forge_writers_round_trip_with_the_v6_golden_bytes() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    for (forge, author) in [(0, 1), (1, 0), (1, 42), (u16::MAX, u64::MAX)] {
        let row = forge::Stored::Writer(skein_forge_client::Writer { forge, author });
        let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).expect("writer key");
        let bytes = records::encode(&row);
        assert_eq!(bytes.len(), 13); // Row version, family, forge, author.
        assert_eq!(&bytes[..3], &[0, 17, 16]);
        assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
        if forge == u16::MAX {
            let mut golden = [255; 11];
            golden[0] = 16;
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
        assert_eq!(&bytes[..3], &[0, 17, 17]);
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
    use crate::v17 as w;
    for (address, subject) in [
        (Box::from([b'a'; 255]), b"subject".as_slice().into()),
        (b"sam@example.com".as_slice().into(), Box::from([b's'; 256])),
    ] {
        w::Binding::new(&w::CEILINGS, w::BindingParts { address, subject }).unwrap_err();
    }
}

#[test]
fn embedded_people_refusals_changed_at_store_nine() {
    // The people v12 generated Unknown golden was [9]; v13 inserts NameTaken
    // before every existing refusal. All 38 existing Refusal goldens change.
    let mut after = skein_lib::Writer::new(1);
    jig_protocol_store::people_v16::Refusal::Unknown.encode(&mut after).unwrap();
    assert_eq!(after.finish().as_ref(), &[10]);
    assert_eq!(crate::STORE_VERSION, 17);
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
        let row = forge::Stored::Change(Box::new(row));
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
    let row = forge::Stored::Change(Box::new(change));
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
            let row = forge::Stored::Change(Box::new(row));
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
    let row = forge::Stored::Change(Box::new(row));
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
            row.change.landing = landing;
            let row = forge::Stored::Change(Box::new(row));
            let bytes = records::encode(&row);
            assert_eq!(records::decode(&bytes, &bounds), Ok(row));
        }
    }
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
                Some(c::Ask::Repair { gate: 0, candidate: [1; 32] }),
            ] {
                let row = forge::Stored::Landing(forge::LandingRow {
                    landing: l::Landing {
                        round: temper_engine_domain_forge_landing::Round::new(Box::from(b"temper/1/l61".as_slice())),

                        task: 61,
                        project: 5,
                        phase: phase.clone(),
                        members: Box::new([l::Member {
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
                        branch: Box::from(b"temper/g1".as_slice()),
                    },
                    asks: Box::new([ask]),
                    entries: Box::new([forge::LandingEntry {
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
            let row = forge::Stored::Receipt {
                key: forge::ReceiptKey {
                    project: 5,
                    repository: skein_forge::api::Repository { forge: 1, repository: 2 },
                    check: check.clone(),
                    key: [3; 32],
                },
                receipt: forge::Receipt {
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
    let row = forge::Stored::Change(Box::new(row));
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
    let row = forge::Stored::Change(Box::new(row));
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
    let row = forge::Stored::Change(Box::new(row));
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
}

#[test]
fn a_producer_report_is_bounded_by_the_forge_operation_bytes() {
    let bound = fixtures::LIMITS.client.op_bytes;
    let bounds = crate::bounds(&fixtures::LIMITS, bound).expect("fixture bounds");
    assert_eq!(bounds.limits().change_row_report, bound);
    for length in [bound, bound + 1] {
        let mut row = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: true });
        row.report = skein_lib::bytes::zeroed(skein_lib::usize_of(length));
        let row = forge::Stored::Change(Box::new(row));
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
            let row = forge::Stored::Change(Box::new(row));
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
    use temper_engine_domain_forge_change::{Hold, Open, Questions, Route, Stands, State};
    const ASKER: u64 = 7;
    let mut row =
        fixtures::change_row(State::Held { was: Box::new(State::Checking { head: [1; 32] }), why: Hold::Questions });
    let mut open = skein_lib::List::uncapped();
    for (number, stands) in [
        (1, Stands::Taken),
        (4, Stands::Asked),
        (5, Stands::Held { answer: b"Keep the documented behavior".as_slice().into() }),
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
    row.questions = Questions { route: Route::Up, taken: 3, open: open.into_boxed() };
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture bounds");
    let row = forge::Stored::Change(Box::new(row));
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
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
        stands: Stands::Held { answer: skein_lib::bytes::zeroed(262_144) },
    }]);
    let row = forge::Stored::Change(Box::new(row));
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
            stands: Stands::Held { answer: skein_lib::bytes::zeroed(usize::try_from(length).unwrap()) },
        }]);
        let row = forge::Stored::Change(Box::new(row));
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
        let row = forge::Stored::Change(Box::new(row));
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
        let row = forge::Stored::Change(Box::new(row));
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
        }),
        FirstIntent::Empty(EmptyIntent {
            org: skein_lib::bytes::zeroed(usize::try_from(limits.empty_intent_org).unwrap()),
            repository: skein_lib::bytes::zeroed(usize::try_from(limits.empty_intent_repository).unwrap()),
            label: skein_lib::bytes::zeroed(usize::try_from(limits.empty_intent_label).unwrap()),
            playground: true,
        }),
    ] {
        let mut record = fixtures::forge_record();
        record.first = Some(first);
        new_family_round_trip(&forge::Stored::Forge(record));
    }
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
                commit: [7; 32],
                branch: vec![b'x'; usize::try_from(branch_bytes).unwrap()].into(),
                at: skein_lib::Wall::from_nanos(123),
                changes: vec![
                    forge::LoggedChange {
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
    let limits = fixtures::LIMITS;
    assert_eq!(records::encode(&landing_log(16, 2, 64)).len(), 10_797);
    let bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
    for extra in [0, 1] {
        for row in [
            landing_log(limits.landings_kept + extra, limits.landing_members, limits.name_bytes),
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
            });
            for running in [
                None,
                Some(forge::StatusRunning { task: None, landed: count, since: Wall::from_nanos(count) }),
                Some(forge::StatusRunning { task: Some(u64::MAX), landed: count, since: Wall::from_nanos(count) }),
            ] {
                let facts = forge::Stored::ProjectFacts(forge::ProjectFacts {
                    project,
                    landed: count,
                    last_landed: Wall::from_nanos(count),
                    brief: brief.clone(),
                    running,
                });
                let looked = forge::Stored::Looked(forge::Looked {
                    party,
                    project,
                    previous: present.then_some(Wall::from_nanos(count)),
                    latest: Wall::from_nanos(count),
                    seen: count,
                    next_seen: count,
                });
                for row in [facts, looked] {
                    let bytes = records::encode(&row);
                    let key = crate::keys::encode(&forge::stored_key(&row), u32::MAX).unwrap();
                    assert_eq!(records::row(&key, &bytes, &bounds), Ok(row));
                }
            }
        }
    }
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
    let row = forge::Stored::Change(Box::new(row));
    assert_eq!(records::decode(&records::encode(&row), &bounds), Ok(row));
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
        let row = forge::Stored::Change(Box::new(row));
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
                retry: None,
                selections: Box::new([]),
            },
            making: Some(Step::Close),
            close_changes: Box::new([20]),
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
                selections: Box::new([]),
            },
            making: Some(Step::Land),
            close_changes: Box::new([20, 22]),
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
            selections: Box::new([]),
        },
        making: None,
        close_changes: Box::new([20]),
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
fn a_closers_question_round_trips_with_its_owned_bytes() {
    use temper_engine_domain_forge_closer::{Closer, Step, relay::*};
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits");
    for stands in [
        Stands::Taken,
        Stands::Asked { question: 30, answer: None },
        Stands::Asked { question: 30, answer: Some(b"Use known reds.".as_slice().into()) },
    ] {
        let row = forge::Stored::Closer(forge::CloserRow {
            task: 8,
            goal: 7,
            closer: Closer::new(),
            making: None,
            close_changes: Box::new([]),
            landing_changes: Box::new([]),
            done: false,
            relay: Relay {
                open: Some(Question {
                    asker: 20,
                    number: 10,
                    step: Step::Land,
                    words: b"Which rule?".as_slice().into(),
                    stands,
                }),
                replies: Box::new([Message {
                    target: 7,
                    form: Form::Words,
                    words: b"Change 20: asker ended".as_slice().into(),
                }]),
            },
        });
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
fn a_closer_with_a_truncated_relay_is_malformed() {
    use temper_engine_domain_forge_closer::{Closer, relay::Relay};
    let row = forge::Stored::Closer(forge::CloserRow {
        task: 8,
        goal: 7,
        closer: Closer::new(),
        making: None,
        close_changes: Box::new([]),
        landing_changes: Box::new([]),
        done: false,
        relay: Relay::new(),
    });
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).expect("fixture limits");
    let bytes = records::encode(&row);
    assert_eq!(bytes.last(), Some(&0));
    let legacy = &bytes[..bytes.len() - 1]; // The current version requires the optional member's presence tag.
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
        State::Repairing { requested: true, why: Repair::LandingAt { gate: 3, candidate: [4; 32] } },
    ] {
        new_family_round_trip(&forge::Stored::Change(Box::new(fixtures::change_row(state))));
    }
    new_family_round_trip(&forge::Stored::Receipt {
        key: forge::ReceiptKey {
            project: 5,
            repository: skein_forge::api::Repository { forge: 1, repository: 2 },
            check: forge::Check::Gate(b"test".as_slice().into()),
            key: [3; 32],
        },
        receipt: forge::Receipt {
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
        for row in [forge::Stored::Change(Box::new(change)), log] {
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
        assert_eq!(bytes.get(2), Some(&37), "Device is Value's appended tag");
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
        v17::Device::new(
            &v17::CEILINGS,
            v17::DeviceParts {
                party: 1,
                number: 1,
                endpoint: vec![b'e'; endpoint].into_boxed_slice(),
                label: vec![b'l'; label].into_boxed_slice(),
                p256dh: [4; 65],
                auth: [7; 16],
                at: 0,
            },
        )
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
    use crate::v17 as w;
    use temper_engine_domain_forge_change::State;
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let row = forge::Stored::Change(Box::new(fixtures::change_row(State::Producing { requested: true })));
    let encoded = records::encode(&row);
    for route in [true, false] {
        let row = w::Row::decode(bounds.limits(), &mut skein_lib::Reader::new(&encoded)).unwrap();
        let w::Value::Change(change) = row.into_parts().value else { panic!("stored change") };
        let mut parts = change.into_parts();
        let mut open = skein_lib::List::with_capacity(1);
        if !route {
            open.push(
                w::Open::new(
                    &w::CEILINGS,
                    w::OpenParts {
                        asker: 7,
                        number: 1,
                        line: Box::new([]),
                        words: Box::new([]),
                        stands: w::Stands::Retired,
                    },
                )
                .unwrap(),
            )
            .unwrap();
        }
        parts.questions = w::Questions::new(
            &w::CEILINGS,
            w::QuestionsParts { route: if route { w::Route::Retired } else { w::Route::Up }, taken: 1, open },
        )
        .unwrap();
        let row = w::Row::new(
            &w::CEILINGS,
            w::RowParts { value: w::Value::Change(Box::new(w::ChangeRow::new(&w::CEILINGS, parts).unwrap())) },
        )
        .unwrap();
        let mut bytes = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
        row.encode(&mut bytes).unwrap();
        assert_eq!(records::decode(&bytes.finish(), &bounds), Err(crate::Undecodable::Malformed));
    }
}
