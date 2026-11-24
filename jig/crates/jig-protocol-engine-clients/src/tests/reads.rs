use super::super::*;
use super::fixtures as f;
use jig_domain_core_vocabulary::reads;
#[test]
fn pending_amendments_answer_their_value_over_the_exchange() {
    use jig_domain_core_vocabulary::documents as d;
    let limits = f::wire_limits();
    for proposer in [d::Proposer::Party(11), d::Proposer::Task(17)] {
        let mut proposal = f::pending_amendment();
        proposal.proposer = proposer;
        let key = w::ProposalKey {
            proposer: match proposer {
                d::Proposer::Party(value) => w::Proposer::Party(w::ProposerParty { value }),
                d::Proposer::Task(value) => w::Proposer::Task(w::ProposerTask { value }),
            },
            number: 13,
        };
        let read = w::Read::Proposal(w::ReadProposal { proposal: key });
        let mut exchange = f::read_exchange();
        let (owner, reply, requested) = exchange.read(&read);
        assert_eq!(requested, reads::Read::Proposal { proposer, proposal: 13 });
        exchange
            .release_read(reply, reads::Answer::Document(Box::new(d::Document::Proposal(Box::new(proposal.clone())))));
        let answer = exchange.read_answer(owner);
        let w::ReadAnswer::Value(_) = &answer else { panic!("amendment read must carry its value, got {answer:?}") };
        let expected = w::ReadValue::Proposal(crate::translate::objects::proposal(&[], &proposal, &limits).unwrap());
        assert_eq!(answer, f::value_answer(&expected));
    }
}

#[test]
fn me_reports_the_reading_sign_ins_saved_expiry() {
    let mut exchange = f::read_exchange();
    let expires = skein_lib::Wall::from_nanos(1_800_000_000_000_000_000);
    let (owner, reply, requested) = exchange.read_until(&w::Read::Me, expires);
    assert_eq!(requested, reads::Read::Frame);
    exchange.release_read(
        reply,
        reads::Answer::Document(Box::new(core::documents::Document::Frame(f::frame_two_projects()))),
    );
    let w::ReadAnswer::Value(value) = exchange.read_answer(owner) else { panic!("me read") };
    let w::ReadValue::Me(me) =
        w::ReadValue::decode(&exchange.env.limits.wire, &mut skein_lib::Reader::new(&value.record))
            .expect("read value")
    else {
        panic!("me")
    };
    assert_eq!(me.sign_in_expires, expires.as_nanos());
}

#[test]
fn a_read_is_answered_only_once_held_is_released() {
    let cases = f::held_reads();
    for (label, value, expected_read, answer, expected_answer) in cases {
        let mut exchange = f::read_exchange();
        let (owner, reply, read) = exchange.read(&value);
        assert_eq!(read, expected_read, "{label}");
        assert!(exchange.read_unanswered(owner), "{label}");
        let name = reply.into_token();
        exchange.release_read(ReplyTo::new(name), answer.clone());
        assert_eq!(exchange.read_answer(owner), expected_answer, "{label}");
        exchange.release_read(ReplyTo::new(name), reads::Answer::None(reads::Why::Unknown));
        assert_eq!(exchange.read_answer(owner), expected_answer, "{label}");
        assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
        assert!(exchange.out.is_empty(), "{label}");
    }
}
#[test]
fn a_cursor_round_trips_to_the_same_position() {
    let party = (
        "Party",
        reads::Cursor::Store {
            scope: reads::Scope::ScopedNotes {
                scope: jig_domain_notes::Scope::Party { party: u64::MAX },
                description: Some(Box::from(b"filter".as_slice())),
            },
            after: Some(reads::Position::Owner(jig_domain_core_vocabulary::Key::Notes(jig_domain_notes::Key::Line {
                scope: jig_domain_notes::Scope::Party { party: u64::MAX },
                name: u64::MAX,
            }))),
        },
    );
    for (label, position) in f::cursors().into_iter().chain([party]) {
        let encoded = crate::translate::cursor::encode(&position, w::CEILINGS.cursor_value).expect(label);
        assert_eq!(crate::translate::cursor::decode(&encoded), Some(position.clone()), "{label}");
        let size = u32::try_from(encoded.len()).expect("small cursor");
        assert_eq!(crate::translate::cursor::encode(&position, size), Some(encoded.clone()), "{label}");
        assert_eq!(
            crate::translate::cursor::encode(&position, size.checked_sub(1).expect("cursor header")),
            None,
            "{label}"
        );
        for cut in 0..encoded.len() {
            assert_eq!(crate::translate::cursor::decode(&encoded[..cut]), None, "{label}, cut {cut}");
        }
        let mut trailing = skein_lib::Writer::new(encoded.len().checked_add(1).expect("small cursor"));
        trailing.put(&encoded).expect("cursor");
        trailing.put(&[0]).expect("extra byte");
        assert_eq!(crate::translate::cursor::decode(&trailing.finish()), None, "{label}, trailing");
    }
}
#[test]
fn a_cursor_that_does_not_decode_is_answered_unknown() {
    use crate::{cursor_v2::LAYOUT, translate::cursor::CURSOR_VERSION};
    for (label, bytes) in [
        ("unknown format", Box::from([0, 3].as_slice())),
        ("unknown variant", cursor_with_head(CURSOR_VERSION, &LAYOUT, &[2])),
        ("truncated scope", cursor_with_head(CURSOR_VERSION, &LAYOUT, &[0, 1, 10])),
    ] {
        assert_spend_cursor_unknown(bytes, label);
    }
}

#[test]
fn a_cursor_of_another_version_or_layout_is_answered_unknown() {
    use crate::{cursor_v2::LAYOUT, translate::cursor};
    let value = reads::Cursor::Store { scope: reads::Scope::Spend { project: SPEND_PROJECT }, after: None };
    let bytes = cursor::encode(&value, w::CEILINGS.cursor_value).expect("spend cursor");
    let form = bytes.get(CURSOR_HEAD_BYTES..).expect("cursor form");
    for (label, version, layout) in [("version", 1, &LAYOUT), ("layout", cursor::CURSOR_VERSION, &[0; LAYOUT.len()])] {
        assert_spend_cursor_unknown(cursor_with_head(version, layout, form), label);
    }
}

const SPEND_PROJECT: u32 = 3;
const CURSOR_HEAD_BYTES: usize = size_of::<u16>() + crate::cursor_v2::LAYOUT.len();

fn cursor_with_head(version: u16, layout: &[u8; 32], form: &[u8]) -> Box<[u8]> {
    let mut out = skein_lib::Writer::new(CURSOR_HEAD_BYTES.checked_add(form.len()).expect("short cursor"));
    out.put(&version.to_be_bytes()).expect("version fits");
    out.put(layout).expect("layout fits");
    out.put(form).expect("form fits");
    out.finish()
}

fn assert_spend_cursor_unknown(bytes: Box<[u8]>, label: &str) {
    let after = w::Cursor { value: bytes };
    let read = w::Read::Spend(w::ReadSpend { project: SPEND_PROJECT, after: Some(after) });
    let mut exchange = f::read_exchange();
    let owner = exchange.send(crate::Message::Read(f::owned_read(w::Owner::Jig, w::VERSION, f::encoded_read(&read))));
    exchange.authenticate();
    assert!(exchange.out.is_empty(), "{label}: no core read");
    assert_eq!(exchange.read_answer(owner), f::read_none(w::ReadNone::Unknown), "{label}");
    assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
}

#[test]
fn a_read_signed_out_is_answered_signed_out() {
    for label in ["core answer", "ended pending sign-in", "absent digest"] {
        let mut exchange = f::read_exchange();
        let owner = if label == "absent digest" {
            let owner = exchange.send(crate::Message::Read(f::owned_read(
                w::Owner::Jig,
                jig_client_protocol_wire::VERSION,
                f::encoded_read(&w::Read::Me),
            )));
            let Some(Request::Secrets(SecretsRequest::Get { owner: who })) = exchange.out.pop() else {
                panic!("digest get")
            };
            exchange.component.down(
                &exchange.env,
                Event::Secrets(SecretsAnswer::Got { owner: who, value: None }),
                &mut exchange.out,
            );
            owner
        } else {
            let (owner, reply, _) = exchange.read(&w::Read::Me);
            if label == "core answer" {
                exchange.release_read(reply, reads::Answer::None(reads::Why::SignedOut));
            } else {
                exchange.component.down(&exchange.env, Event::SignInEnded { sign_in: f::NUMBER }, &mut exchange.out);
                let Some(Request::Secrets(SecretsRequest::Erase { owner: secret_owner })) = exchange.out.pop() else {
                    panic!("digest erase")
                };
                assert_eq!(secret_owner, SecretOwner::SignIn(f::NUMBER), "{label}");
            }
            owner
        };
        assert_eq!(exchange.read_answer(owner), f::read_none(w::ReadNone::SignedOut), "{label}");
        assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
        assert!(exchange.out.is_empty(), "{label}");
    }
}
#[test]
fn an_application_read_is_routed_and_answered_under_its_family() {
    for label in ["document", "unknown"] {
        let mut exchange = f::read_exchange();
        let (owner, reply) = exchange.application_read();
        assert!(exchange.read_unanswered(owner), "{label}");
        let name = reply.into_token();
        exchange.release_application_read(ReplyTo::new(name), 8, None);
        assert!(exchange.read_unanswered(owner), "{label}");
        let record = if label == "document" { Some(Box::from([255, 0, 2])) } else { None };
        exchange.release_application_read(ReplyTo::new(name), 9, record);
        let expected = if label == "document" {
            let family = w::Family { number: 9 };
            w::ReadAnswer::Value(w::Owned {
                owner: w::Owner::Family(family),
                version: 2,
                record: Box::from([255, 0, 2]),
            })
        } else {
            f::read_none(w::ReadNone::Unknown)
        };
        assert_eq!(exchange.read_answer(owner), expected, "{label}");
        exchange.release_application_read(ReplyTo::new(name), 9, None);
        assert_eq!(exchange.read_answer(owner), expected, "{label}, late");
        assert!(exchange.component.pending.exchanges.is_empty(), "{label}");
    }
}

fn task_query() -> jig_domain_core_vocabulary::documents::TaskQuery {
    jig_domain_core_vocabulary::documents::TaskQuery {
        parts: skein_lib::List::with_capacity(0),
        project: Some(3),
        requester: Some(jig_domain_tasks::Party::Person(5)),
        root: Some(7),
        parent: Some(11),
        label: Some(13),
        phase: None,
        executor: None,
        tracked: None,
        order: jig_domain_core_vocabulary::documents::QueryOrder::Number,
    }
}
const QUERY_ORDERS: [jig_domain_core_vocabulary::documents::QueryOrder; 5] = {
    use jig_domain_core_vocabulary::documents::QueryOrder as O;
    [O::Number, O::Changed, O::Activity, O::Priority, O::Ended]
};
const QUERY_PHASES: [Option<jig_domain_core_vocabulary::documents::PhaseClass>; 6] = {
    use jig_domain_core_vocabulary::documents::PhaseClass as P;
    [None, Some(P::Working), Some(P::Waiting), Some(P::Held), Some(P::Closing), Some(P::Ended)]
};
const QUERY_EXECUTORS: [Option<jig_domain_core_vocabulary::documents::ExecutorKind>; 4] = {
    use jig_domain_core_vocabulary::documents::ExecutorKind as E;
    [None, Some(E::Agent), Some(E::Procedure), Some(E::Person)]
};
#[test]
fn task_query_cursors_preserve_their_keys_orders_and_phases() {
    for order in QUERY_ORDERS {
        for phase in QUERY_PHASES {
            for executor in QUERY_EXECUTORS {
                let query = jig_domain_core_vocabulary::documents::TaskQuery { order, phase, executor, ..task_query() };
                for query in [
                    query.clone(),
                    jig_domain_core_vocabulary::documents::TaskQuery {
                        requester: None,
                        tracked: Some(true),
                        ..query.clone()
                    },
                    jig_domain_core_vocabulary::documents::TaskQuery {
                        requester: Some(jig_domain_tasks::Party::Task(19)),
                        tracked: Some(false),
                        ..query
                    },
                ] {
                    let scope = reads::Scope::Tasks { query };
                    let cursors = [
                        reads::Cursor::Store { scope: scope.clone(), after: None },
                        reads::Cursor::Live {
                            scope,
                            after: jig_domain_core_vocabulary::documents::Cursor::Number { number: 17 },
                            store_after: None,
                        },
                    ];
                    for cursor in cursors {
                        let bytes = crate::translate::cursor::encode(&cursor, 1024).unwrap();
                        assert_eq!(crate::translate::cursor::decode(&bytes), Some(cursor));
                    }
                }
            }
        }
    }
}

#[test]
fn turn_failure_terminals_keep_their_reason_after_release() {
    for (why, expected) in [
        (reads::Why::Malformed, w::ReadNone::Malformed),
        (reads::Why::TooLarge { bytes: 8192 }, w::ReadNone::TooLarge(w::TurnTooLarge { bytes: 8192 })),
    ] {
        let mut exchange = f::read_exchange();
        let (owner, reply, _) = exchange.read(&w::Read::Me);
        assert!(exchange.read_unanswered(owner));
        exchange.release_read(reply, reads::Answer::None(why));
        assert_eq!(exchange.read_answer(owner), f::read_none(expected));
    }
}

#[test]
fn a_single_turn_read_carries_the_same_archive_as_activity() {
    for withheld in [false, true] {
        let page = f::private_activity(false);
        let reads::PageRow::Activity { turn: Some(reads::PageTurn::Carried(record)), .. } = page.rows.get(0).unwrap()
        else {
            panic!("carried fixture turn")
        };
        let limits = f::wire();
        let value = w::Read::Turn(w::ReadTurn { task: record.task, attempt: record.attempt, turn: record.turn });
        let mut settings = f::settings();
        settings.private = f::private();
        let mut exchange = f::read_exchange_with(settings);
        let (owner, reply, read) = exchange.read(&value);
        assert_eq!(read, reads::Read::Turn { task: record.task, attempt: record.attempt, turn: record.turn });
        exchange.release_read(
            reply,
            reads::Answer::Ended(Box::new(reads::Ended {
                value: if withheld {
                    reads::EndedValue::WithheldTurn(record.clone())
                } else {
                    reads::EndedValue::Turn(record.clone())
                },
                at: None,
                by: None,
            })),
        );
        let w::ReadAnswer::Value(envelope) = exchange.read_answer(owner) else { panic!("turn answer") };
        let w::ReadValue::Turn(archive) =
            w::ReadValue::decode(&limits, &mut skein_lib::Reader::new(envelope.record.as_ref())).unwrap()
        else {
            panic!("archive")
        };
        assert_eq!(
            (archive.task, archive.name.attempt, archive.name.number),
            (record.task, record.attempt, record.turn)
        );
        assert_eq!((archive.cumulative, archive.read, archive.at), (record.spent, record.read, record.at.as_nanos()));
        let w::TurnBytes::Carried(turn) = &archive.turn else { panic!("carried") };
        assert_eq!(turn.bytes.as_ref(), f::private_transcript(withheld).as_ref(), "withheld {withheld}");
    }
}

#[test]
fn a_client_reading_a_too_linked_task_is_told_its_join_bound() {
    let mut exchange = f::read_exchange();
    let task = w::Read::Task(w::ReadTask { task: 1 });
    let (owner, reply, _) = exchange.read(&task);
    exchange.release_read(reply, reads::Answer::None(reads::Why::TooLinked { joins: 4 }));
    // read_answer decodes the real engine response bytes with the client codec.
    let w::ReadAnswer::None(reason) = exchange.read_answer(owner) else { panic!("read refusal") };
    assert_eq!(
        jig_client_protocol_link::translate::reads::none(&reason.value),
        jig_client_domain::contract::ReadNone::TooLinked { joins: 4 },
    );
}

#[test]
fn cursor_boundaries_refuse_unrepresentable_scopes_oversized_values_and_other_forms() {
    use crate::translate::cursor;
    use jig_domain_core_vocabulary::documents;
    let after = documents::Cursor::Number { number: 7 };
    for scope in [
        reads::Scope::TaskText { task: 7, field: reads::TaskTextField::Result, from: 0 },
        reads::Scope::EndedTasks,
        reads::Scope::TaskRows { task: 7 },
        reads::Scope::SettingsHistory { first: 1, last: 2, newest: 2, whole: true },
    ] {
        for value in [
            reads::Cursor::Store { scope: scope.clone(), after: None },
            reads::Cursor::Live { scope, after, store_after: None },
        ] {
            assert_eq!(cursor::encode(&value, w::CEILINGS.cursor_value), None);
        }
    }
    let oversized = deployment_notes_cursor(Box::new([0; LARGEST_DESCRIPTION + 1]));
    assert_eq!(cursor::encode(&oversized, w::CEILINGS.cursor_value), None);
    let bytes = cursor::encode(&oversized, CURSOR_BYTES + 1).expect("field within schema ceiling");
    assert_eq!(bytes.len(), skein_lib::length::usize_of(CURSOR_BYTES + 1));
    assert_eq!(cursor::decode(&bytes), None);
    let tree = cursor::encode_tree(&after, 7, 3, w::CEILINGS.cursor_value).expect("tree cursor");
    assert_eq!(cursor::decode(&tree), None);
    assert_eq!(cursor::decode_window(&tree), None);
    let bare = crate::cursor_v2::DocumentCursor::Number(crate::cursor_v2::Number { value: 7 });
    let bytes = bare.encode(&crate::cursor_v2::CEILINGS).expect("bounded document cursor");
    assert_eq!(cursor::decode_window(&bytes), Some(after));
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).expect("small cursor"));
    trailing.put(&bytes).expect("document cursor");
    trailing.put(&[0]).expect("extra byte");
    assert_eq!(cursor::decode_window(&trailing.finish()), None);
    assert_eq!(cursor::decode(&[1, 1, 10, 7, 0, 0, 0, 0, 0, 0, 0, 0]), None, "previous owner format");
}

// The old 1,024-byte bound minus 12 bytes: version (2), four tags (4),
// description presence and length (5), and absent store position (1).
const LARGEST_DESCRIPTION: usize = 1012;
const CURSOR_BYTES: u32 = 1056; // 1,024 old bytes plus the 32-byte layout.

fn deployment_notes_cursor(description: Box<[u8]>) -> reads::Cursor {
    reads::Cursor::Store {
        scope: reads::Scope::ScopedNotes { scope: jig_domain_notes::Scope::Deployment, description: Some(description) },
        after: None,
    }
}

#[test]
fn a_form_that_filled_the_old_cursor_still_fits() {
    use crate::translate::cursor;
    let value = deployment_notes_cursor(Box::new([0; LARGEST_DESCRIPTION]));
    let bytes = cursor::encode(&value, w::CEILINGS.cursor_value).expect("cursor fits the new ceiling");
    assert_eq!(bytes.len(), skein_lib::length::usize_of(CURSOR_BYTES));
    assert_eq!(cursor::decode(&bytes), Some(value));
    w::Cursor { value: bytes }.encode(&w::CEILINGS).expect("wire cursor fits too");
}

fn tasks_read(description: Option<Box<[u8]>>, after: Option<w::Cursor>) -> w::Read {
    w::Read::Notes(w::ReadNotes { scope: w::NoteScope::Tasks(w::NoteScopeTasks { project: 13 }), description, after })
}

#[test]
fn tasks_translates_onto_the_project_read() {
    let cursor = reads::Cursor::Store { scope: reads::Scope::Notes { project: 13 }, after: None };
    let encoded = crate::translate::cursor::encode(&cursor, 1024).unwrap();
    for (after, from) in [(None, None), (Some(w::Cursor { value: encoded }), Some(cursor))] {
        let mut exchange = f::read_exchange();
        let (owner, reply, read) = exchange.read(&tasks_read(None, after));
        assert_eq!(read, reads::Read::Notes { project: 13, from });
        let mut page = f::notes_page();
        let reads::PageRow::Note(note) = page.rows.get_mut(0).unwrap() else { panic!("note row") };
        note.scope = core::documents::NoteLocation::Goal { project: 13, task: 19 };
        note.body = core::documents::Text { bytes: Box::new([]), cut: true };
        let expected =
            crate::translate::pages::page(&f::models(), &read, &page, &crate::Private::none(), &f::wire()).unwrap();
        let w::ReadValue::Notes(_) = &expected else { panic!("notes wire page") };
        exchange.release_read(reply, reads::Answer::Page(Box::new(page)));
        assert_eq!(exchange.read_answer(owner), f::value_answer(&expected));
    }
}

#[test]
fn a_description_on_tasks_is_refused() {
    for description in [Box::<[u8]>::from([]), Box::from(b"title".as_slice())] {
        let value = tasks_read(Some(description), None);
        assert_eq!(crate::translate::reads::read(value.clone()), None);
        let mut exchange = f::read_exchange();
        let owner = exchange.send(crate::Message::Read(f::owned_read(
            w::Owner::Jig,
            jig_client_protocol_wire::VERSION,
            f::encoded_read(&value),
        )));
        exchange.authenticate();
        assert_eq!(exchange.read_answer(owner), f::read_none(w::ReadNone::Unknown));
        assert!(exchange.out.is_empty());
    }
}

#[test]
fn an_activity_beside_read_crosses_the_wire() {
    use jig_client_domain::contract as c;
    for (side, expected) in [(c::Side::Earlier, reads::Side::Earlier), (c::Side::Later, reads::Side::Later)] {
        let query = c::Read::ActivityBeside { task: 7, position: 11, side };
        let wire = jig_client_protocol_link::translate::reads::read(&query);
        let mut exchange = f::read_exchange();
        let (_, _, read) = exchange.read(&wire);
        assert_eq!(read, reads::Read::ActivityBeside { task: 7, position: 11, side: expected });
    }
}

#[test]
fn a_later_page_says_more_by_its_after() {
    let scope = reads::Scope::Activity { task: 7 };
    let next = reads::Cursor::Store {
        scope,
        after: Some(reads::Position::Owner(jig_domain_core_vocabulary::Key::Tasks(jig_domain_tasks::Key::Activity {
            task: 7,
            position: 13,
        }))),
    };
    let read = reads::Read::ActivityBeside { task: 7, position: 11, side: reads::Side::Later };
    let page = reads::Page {
        rows: skein_lib::List::with_capacity(0),
        before: None,
        next: Some(next.clone()),
        cut: false,
        inbox_selection: None,
    };
    let w::ReadValue::Activity(value) =
        crate::translate::pages::page(&[], &read, &page, &crate::Private::none(), &f::wire()).unwrap()
    else {
        panic!("activity page")
    };
    assert_eq!((&value.before), &None);
    let after = value.after.as_ref().expect("more later lines");
    assert_eq!(crate::translate::cursor::decode(after.value.as_ref()), Some(next));
}

#[test]
fn an_answer_it_cannot_encode_is_malformed_not_closed() {
    use jig_domain_tasks as t;
    // The two-byte words exceed the one-byte wire bound.
    const WORDS: &[u8] = b"hi";
    const PAST_BOUND: u32 = 1;
    let mut exchange = f::read_exchange();
    exchange.env.limits.wire.activity_words_words = PAST_BOUND;
    let query = w::Read::Activity(w::ReadActivity { task: 7, before: None });
    let (owner, reply, _) = exchange.read(&query);
    let page = f::page(skein_lib::List::from_boxed(Box::new([reads::PageRow::Activity {
        line: t::ActivityLine {
            task: 7,
            position: 11,
            at: skein_lib::Wall::from_nanos(17),
            line: t::Line::Words {
                message: 1,
                by: t::Party::Person(3),
                kind: t::MessageKind::Words,
                words: Box::from(WORDS),
            },
        },
        turn: None,
    }])));
    let name = reply.into_token();
    exchange.release_read(ReplyTo::new(name), reads::Answer::Page(Box::new(page)));
    let answer = exchange.read_answer(owner);
    assert_eq!(answer, f::read_none(w::ReadNone::Malformed));
    exchange.release_read(ReplyTo::new(name), reads::Answer::None(reads::Why::Unknown));
    assert_eq!(exchange.read_answer(owner), answer);
    let connection = exchange.component.connections.get(&owner).expect("answered exchange");
    assert!(!connection.closing);
    assert!(exchange.component.pending.exchanges.is_empty());
    assert!(exchange.out.is_empty());
    let (next, reply, _) = exchange.read(&query);
    exchange.release_read(reply, reads::Answer::None(reads::Why::Unknown));
    assert_eq!(exchange.read_answer(next), f::read_none(w::ReadNone::Unknown));
}
