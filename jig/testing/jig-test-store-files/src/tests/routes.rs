mod fixtures;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up, route};
use skein_lib::Token;

/// Two batches of eight flights delay all pages before answering in reverse
/// order; each flight restores three rows across two pages.
#[test]
fn outstanding_routes_complete_with_delayed_replies() {
    delayed_routes(fixtures::routes());
}

#[test]
fn outstanding_routes_outgrow_the_supplied_table_size() {
    const TABLE_HINT: u32 = 1; // All eight admitted flights must retain their continuations.
    let mut routes = route::Routes::new(TABLE_HINT);
    routes.configure(&fixtures::root_limits());
    routes.configure_restore(&fixtures::component_limits()).expect("restart page bounds fit");
    delayed_routes(routes);
}

fn delayed_routes(mut routes: route::Routes) {
    const FLIGHTS: u16 = 8; // The routing fixture's supplied table size.
    const BATCHES: u32 = 2; // Sixteen operations across eight-flight batches.
    const ROWS: u32 = 3; // Two restore rows per page, then one terminal page.
    const PAGES: u32 = 2; // The fixture's three rows need two pages.
    const WRITES: u32 = 25; // Eight flights' three rows, plus the deployment header.
    let mut component = fixtures::component();
    let mut writes = skein_lib::List::with_capacity(WRITES);
    writes.push(fixtures::header(1)).expect("deployment header");
    for number in 1..=FLIGHTS {
        for task in 1..=ROWS {
            writes.push(root::Write::Save(routed_result(number, u64::from(task)))).expect("fixture row");
        }
    }
    fixtures::routed(&mut component, &mut routes, StoreOut::Commit { number: 1, writes: writes.into_boxed() });
    for _ in 0..BATCHES {
        let mut requests = skein_lib::Queue::uncapped();
        let mut flights = std::collections::BTreeMap::new();
        for connector in 1..=FLIGHTS {
            let step = core::start::Phase::RestoreConnector { connector };
            let request =
                route::down(&mut routes, StoreOut::Start { phase: step, census: None }).expect("admitted route");
            let jig_protocol_store::Request::Restore { owner, .. } = &request else {
                panic!("restart page request");
            };
            flights.insert(*owner, (connector, skein_lib::List::with_capacity(ROWS), 0_u32));
            requests.push(request);
        }
        for _ in 0..PAGES {
            // Retain all replies before releasing any; newer flights answer first.
            let mut delayed = skein_lib::Stack::with_capacity(u32::from(FLIGHTS));
            while let Some(request) = requests.pop() {
                let jig_protocol_store::Request::Restore { owner, .. } = &request else {
                    panic!("restart page request");
                };
                let owner = *owner;
                let mut events = skein_lib::Queue::uncapped();
                fixtures::send(&mut component, request, &mut events);
                while let Some(event) = events.pop() {
                    delayed.push((owner, event)).expect("one delayed page per flight");
                }
            }
            while let Some((owner, event)) = delayed.pop() {
                let (connector, rows, terminals) = flights.get_mut(&owner).expect("retained flight");
                let mut outputs = skein_lib::Queue::uncapped();
                route::up(&mut routes, event, &mut outputs);
                while let Some(output) = outputs.pop() {
                    match output {
                        Up::Root(root::Event::StartRecord(row)) => {
                            assert_eq!(*terminals, 0, "rows precede their terminal");
                            rows.push(row).expect("one copy of each fixture row");
                        }
                        Up::Root(root::Event::StartDone(step)) => {
                            assert_eq!(step, core::start::Phase::RestoreConnector { connector: *connector });
                            *terminals = terminals.checked_add(1).expect("bounded terminal count");
                        }
                        Up::Ask(request) => {
                            let jig_protocol_store::Request::Restore { owner: next, .. } = &request else {
                                panic!("restart continuation");
                            };
                            assert_eq!(*next, owner);
                            requests.push(request);
                        }
                        Up::Root(_) => panic!("only the matching start terminal"),
                    }
                }
            }
        }
        assert!(requests.is_empty());
        assert!(routes.idle());
        for (_, (number, rows, terminals)) in flights {
            let mut expected = skein_lib::List::with_capacity(ROWS);
            for task in 1..=ROWS {
                expected.push(routed_result(number, u64::from(task))).expect("fixture row");
            }
            assert_eq!(rows.into_boxed(), expected.into_boxed());
            assert_eq!(terminals, 1);
        }
    }
}
fn routed_result(number: u16, task: u64) -> root::Record {
    root::Record::Connector { number, record: jig_test_connector::Record::Result { task, code: number } }
}

#[test]
fn a_commit_routes_connector_records_as_bytes() {
    let mut routes = fixtures::routes();
    let record = jig_test_connector::Record::Pool { path: fixtures::path(), slots: 1 };
    let request = route::down(
        &mut routes,
        StoreOut::Commit { number: 1, writes: Box::new([root::Write::Save(fixtures::root_record(record))]) },
    )
    .expect("commit request");
    let writes = fixtures::commit_writes(request);
    match writes.as_ref() {
        [jig_protocol_store::Write::Put { key, value }] => {
            assert_eq!(key.as_ref(), &[0, 0, 0, 0, 0, 0, 0, 1, 8, 1, 1, 0, 0, 1, 1, 0, 0, 0]);
            assert_eq!(value.as_ref(), &[0, 2, 9, 0, 0, 0, 2, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 1]);
        }
        _ => unreachable!("one application's put"),
    }
}
#[test]
fn load_core_pages_reach_the_root_then_people_restored() {
    let mut component = fixtures::component();
    let mut routes = fixtures::routes();
    fixtures::routed(&mut component, &mut routes, fixtures::core_commit());
    let outputs = fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Start { phase: core::start::Phase::LoadCore, census: None },
    );
    let mut rows = 0_u32;
    let mut restored = false;
    for output in outputs {
        match output {
            Up::Root(root::Event::StartRecord(_)) => {
                assert!(!restored);
                rows = rows.checked_add(1).expect("bounded rows");
            }
            Up::Root(root::Event::StartDone(step)) => {
                assert_eq!(rows, 3);
                restored = true;
                assert_eq!(step, core::start::Phase::LoadCore);
            }
            Up::Root(_) | Up::Ask(_) => unreachable!("start rows and terminals only"),
        }
    }
    assert!(restored);
}
#[test]
fn a_connectors_step_restores_its_records() {
    let mut component = fixtures::component();
    let mut routes = fixtures::routes();
    let record = jig_test_connector::Record::Pool { path: fixtures::path(), slots: 1 };
    fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Commit {
            number: 1,
            writes: Box::new([fixtures::header(1), root::Write::Save(fixtures::root_record(record.clone()))]),
        },
    );
    let outputs = fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Start { phase: core::start::Phase::RestoreConnector { connector: fixtures::FIRST }, census: None },
    );
    match outputs.as_ref() {
        [
            Up::Root(root::Event::StartRecord(row)),
            Up::Root(root::Event::StartDone(core::start::Phase::RestoreConnector { connector: 1 })),
        ] => {
            assert_eq!(*row, fixtures::root_record(record));
        }
        _ => unreachable!("the typed connector row then its terminal"),
    }
}
#[test]
fn a_seen_conversation_is_answered_from_the_store() {
    let mut component = fixtures::component();
    let mut routes = fixtures::routes();
    let row = fixtures::ended_chat();
    let task = row.number;
    fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Commit {
            number: 1,
            writes: Box::new([
                fixtures::header(1),
                root::Write::Save(root::Record::Core(core::Record::Tasks(jig_domain_tasks::Stored::Ended(
                    row.clone(),
                )))),
            ]),
        },
    );
    let outputs = fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Now(core::Now::SeenConversation { request: Token::new(17), task }),
    );
    match outputs.as_ref() {
        [
            Up::Root(root::Event::Core(core::Event::SeenConversationLoaded { request, row: Some(found), busy: false })),
        ] => {
            assert_eq!(*request, Token::new(17));
            assert_eq!(*found, row);
        }
        _ => unreachable!("the retained chat answers its request"),
    }
}
#[test]
fn a_window_read_reaches_the_root_as_transcript_pages() {
    let mut component = fixtures::component();
    let mut routes = fixtures::routes();
    fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Commit {
            number: 1,
            writes: Box::new([fixtures::header(1), fixtures::turn(1), fixtures::turn(2), fixtures::turn(3)]),
        },
    );
    let outputs = fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Now(core::Now::StartPreparation { task: 21, transcript_waiter: None, from: Some((1, 1)) }),
    );
    let mut expected_turn = 1_u32;
    let mut pages = 0_u32;
    for output in outputs {
        match output {
            Up::Root(root::Event::TranscriptLoaded { task, rows, done }) => {
                assert_eq!(task, 21);
                pages = pages.checked_add(1).expect("bounded pages");
                for (row, text) in rows {
                    assert!(text.is_none());
                    assert_eq!(row.turn, expected_turn);
                    assert_eq!(row.transcript.as_ref(), b"opaque");
                    expected_turn = expected_turn.checked_add(1).expect("bounded turns");
                }
                assert_eq!(done, expected_turn == 4);
            }
            Up::Root(_) | Up::Ask(_) => unreachable!("only transcript pages"),
        }
    }
    assert_eq!(pages, 2);
}

#[test]
fn writer_admitted_rows_restore_above_the_ordinary_read_allowance() {
    let mut component = fixtures::component();
    let mut limits = fixtures::root_limits();
    limits.core.load_bytes = 1;
    let mut routes = route::Routes::new(1);
    routes.configure(&limits);
    routes.configure_restore(&component.limits).expect("restart page bounds fit");
    let record = routed_result(fixtures::FIRST, 7);
    fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Commit { number: 1, writes: Box::new([fixtures::header(1), root::Write::Save(record.clone())]) },
    );
    let step = core::start::Phase::RestoreConnector { connector: fixtures::FIRST };
    let outputs = fixtures::routed(&mut component, &mut routes, StoreOut::Start { phase: step, census: None });
    let mut outputs = outputs.into_iter();
    let Some(Up::Root(root::Event::StartRecord(row))) = outputs.next() else { panic!("restored writer row") };
    assert_eq!(row, record);
    let Some(Up::Root(root::Event::StartDone(done))) = outputs.next() else { panic!("restart terminal") };
    assert_eq!(done, step);
    assert!(outputs.next().is_none());
    assert!(routes.idle());
}

#[test]
#[should_panic(expected = "configure restore before routing requests")]
fn missing_restore_configuration_panics_before_routing() {
    let mut routes = route::Routes::new(1);
    routes.configure(&fixtures::root_limits());
    let _request = route::down(&mut routes, StoreOut::Start { phase: core::start::Phase::LoadCore, census: None });
}

#[test]
fn restart_configuration_preserves_the_ordinary_read_allowance() {
    let mut limits = fixtures::root_limits();
    limits.core.load_bytes = 7;
    let mut routes = fixtures::routes();
    routes.configure(&limits);
    let request = route::down(
        &mut routes,
        StoreOut::NotesLoad { owner: Token::new(1), range: jig_domain_notes::Range::Entry { name: 1 } },
    )
    .expect("ordinary read request");
    let jig_protocol_store::Request::Load { bytes, .. } = request else { panic!("ordinary load") };
    assert_eq!(bytes, 7);
}

#[test]
fn start_phases_select_their_exact_ranges() {
    use core::start::Phase;
    use jig_protocol_store::{Request, Restore};
    let census =
        core::Census { questions: Box::new([(7, 3)]), results: Box::new([core::ReadResults { party: 4, after: 2 }]) };
    for phase in
        [Phase::LoadCore, Phase::LoadInbox, Phase::RestoreConnector { connector: 2 }, Phase::Sweep, Phase::Switch]
    {
        let mut routes = fixtures::routes();
        let selected = if phase == Phase::LoadInbox { Some(census.clone()) } else { None };
        let request = route::down(&mut routes, StoreOut::Start { phase, census: selected }).expect("start request");
        if phase == Phase::Switch {
            let Request::Switch { .. } = request else { panic!("switch request") };
            continue;
        }
        let Request::Restore { range, after: None, rows: 2, bytes, .. } = request else { panic!("first restore page") };
        assert!(bytes > 0);
        match (phase, range) {
            (Phase::LoadCore, Restore::Jig(ranges)) => assert_eq!(ranges.as_ref(), &core::RestoreRange::LIVE),
            (Phase::LoadInbox, Restore::Jig(ranges)) => {
                assert_eq!(ranges.as_ref(), &[core::RestoreRange::Census(census.clone())]);
            }
            (Phase::RestoreConnector { connector }, Restore::Application(ranges)) => {
                assert_eq!(ranges.as_ref(), &[jig_test_store::live(connector)]);
            }
            (Phase::Sweep, Restore::Jig(ranges)) => assert_eq!(ranges.as_ref(), &[core::RestoreRange::People]),
            _ => panic!("phase's exact restore range"),
        }
    }
}

#[test]
fn start_connector_pages_decode_rows_in_order_then_finish_the_phase() {
    use core::start::Phase;
    let mut component = fixtures::starting_component();
    let mut routes = fixtures::routes();
    let rows = [routed_result(1, 1), routed_result(1, 2), routed_result(1, 3)];
    let mut writes = skein_lib::List::with_capacity(4);
    writes.push(fixtures::header(1)).expect("header");
    for row in rows.clone() {
        writes.push(root::Write::Save(row)).expect("fixture row");
    }
    fixtures::routed(&mut component, &mut routes, StoreOut::Commit { number: 1, writes: writes.into_boxed() });
    let phase = Phase::RestoreConnector { connector: 1 };
    let outputs = fixtures::routed(&mut component, &mut routes, StoreOut::Start { phase, census: None });
    let mut found = skein_lib::List::with_capacity(3);
    let mut done = false;
    for output in outputs {
        match output {
            Up::Root(root::Event::StartRecord(row)) => {
                assert!(!done);
                found.push(row).expect("fixture row");
            }
            Up::Root(root::Event::StartDone(found)) => {
                assert!(!done);
                assert_eq!(found, phase);
                done = true;
            }
            Up::Root(_) | Up::Ask(_) => panic!("start rows then its terminal"),
        }
    }
    assert!(done);
    assert_eq!(found.into_boxed().as_ref(), &rows);
    assert!(routes.idle());
}

#[test]
fn start_core_page_records_reach_the_root_without_people_restored() {
    use core::start::Phase;
    let mut component = fixtures::starting_component();
    let mut routes = fixtures::routes();
    fixtures::routed(&mut component, &mut routes, fixtures::core_commit());
    let outputs =
        fixtures::routed(&mut component, &mut routes, StoreOut::Start { phase: Phase::LoadCore, census: None });
    let mut records = skein_lib::List::with_capacity(3);
    for output in outputs {
        match output {
            Up::Root(root::Event::StartRecord(row)) => {
                records.push(row).expect("fixture row");
            }
            Up::Root(root::Event::StartDone(Phase::LoadCore)) => assert_eq!(records.len(), 3),
            Up::Root(_) | Up::Ask(_) => panic!("only start records and done"),
        }
    }
    assert_eq!(records.len(), 3);
    assert!(routes.idle());
}

#[test]
fn start_missing_owner_keeps_both_keys() {
    use core::start::Phase;
    use jig_protocol_store::{Event, Request, restore::Item};
    let mut routes = fixtures::routes();
    let Some(Request::Restore { owner, .. }) = route::down(
        &mut routes,
        StoreOut::Start {
            phase: Phase::LoadInbox,
            census: Some(core::Census { questions: Box::new([]), results: Box::new([]) }),
        },
    ) else {
        panic!("inbox page")
    };
    let record = core::Key::People(people::Key::EndedRead { task: 7, party: 4 });
    let target = core::Key::Tasks(jig_domain_tasks::Key::Ended(7));
    let mut out = skein_lib::Queue::uncapped();
    route::up(
        &mut routes,
        Event::Page {
            owner,
            items: Box::new([Item::MissingLink { record: record.clone(), target: target.clone() }]),
            next: None,
        },
        &mut out,
    );
    let Some(Up::Root(root::Event::StartMissing { record: found_record, target: found_target })) = out.pop() else {
        panic!("missing link")
    };
    assert_eq!((found_record, found_target), (record, target));
    let Some(Up::Root(root::Event::StartDone(Phase::LoadInbox))) = out.pop() else { panic!("inbox done") };
    assert!(out.is_empty());
    assert!(routes.idle());
}

#[test]
fn refused_start_page_preserves_the_store_failure_and_terminates_the_flight() {
    use core::start::Phase;
    use jig_protocol_store::{
        Event, Request,
        restore::{Cause, Failure, RangeFailure},
    };
    let mut routes = fixtures::routes();
    let Some(Request::Restore { owner, .. }) =
        route::down(&mut routes, StoreOut::Start { phase: Phase::LoadCore, census: None })
    else {
        panic!("core page")
    };
    let failure = Failure { cause: Cause::Range(RangeFailure::Cursor), row: None };
    let mut out = skein_lib::Queue::uncapped();
    route::up(&mut routes, Event::PageRefused { owner, failure: failure.clone() }, &mut out);
    let Some(Up::Root(root::Event::StartPageRefused { failure: found })) = out.pop() else {
        panic!("typed page refusal")
    };
    assert_eq!(found, failure);
    assert!(out.is_empty());
    assert!(routes.idle());
}

#[test]
fn switched_start_completes_the_switch_phase() {
    use core::start::Phase;
    use jig_protocol_store::{Event, Request};
    let mut routes = fixtures::routes();
    let Some(Request::Switch { owner }) =
        route::down(&mut routes, StoreOut::Start { phase: Phase::Switch, census: None })
    else {
        panic!("switch request")
    };
    let mut out = skein_lib::Queue::uncapped();
    route::up(&mut routes, Event::Switched { owner }, &mut out);
    let Some(Up::Root(root::Event::StartDone(Phase::Switch))) = out.pop() else { panic!("switch done") };
    assert!(out.is_empty());
    assert!(routes.idle());
}

#[test]
fn start_application_decode_failure_names_its_cause_and_has_the_record_key() {
    use core::start::Phase;
    use jig_protocol_store::{
        Event, Request, Row,
        restore::{Cause, DecodeFailure, Item},
    };
    let mut routes = fixtures::routes();
    let Some(Request::Restore { owner, .. }) =
        route::down(&mut routes, StoreOut::Start { phase: Phase::RestoreConnector { connector: 1 }, census: None })
    else {
        panic!("connector page")
    };
    let key = jig_test_store::keys::encode(1, &jig_test_connector::RecordKey::Result(7));
    let mut out = skein_lib::Queue::uncapped();
    route::up(
        &mut routes,
        Event::Page {
            owner,
            items: Box::new([Item::Row(Row::Application { key: key.clone(), value: Box::new([]) })]),
            next: None,
        },
        &mut out,
    );
    let Some(Up::Root(root::Event::StartPageRefused { failure })) = out.pop() else { panic!("typed decode refusal") };
    assert_eq!(failure.cause, Cause::Decode(DecodeFailure::Value));
    assert_eq!(
        failure.row.expect("application row").key,
        jig_protocol_store::keys::application(&key, u32::MAX).expect("key")
    );
    assert!(out.is_empty());
    assert!(routes.idle());
}
