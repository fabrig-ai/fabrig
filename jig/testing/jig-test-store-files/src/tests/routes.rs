mod fixtures;
use core::engine::store::{Answer, In, Out, Range, Rows};
pub(crate) use fixtures::ended_chat;
use jig_domain_core as core;
use jig_domain_core::engine::store as port;
use jig_domain_people as people;
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up, route};
use skein_lib::Token;

const START_LOAD: Token = Token::new(1);

fn start(ranges: Box<[core::RestoreRange]>, after: Option<port::Cursor>) -> StoreOut {
    StoreOut::Engine(Out::Load { load: START_LOAD, range: Range::Start { ranges, after } })
}

fn live() -> StoreOut {
    start(Box::new(core::RestoreRange::LIVE), None)
}

fn connector(number: u16, after: Option<Box<[u8]>>) -> StoreOut {
    StoreOut::ConnectorLoad { number, after }
}

fn core_pages(component: &mut fixtures::Component, routes: &mut route::Routes) -> Vec<usize> {
    let mut after = None;
    let mut lengths = Vec::new();
    for _ in 0..2 {
        let outputs = fixtures::routed(component, routes, start(Box::new(core::RestoreRange::LIVE), after));
        let [
            Up::Root(root::Event::Store(In::Loaded {
                load: START_LOAD,
                result: Answer::Rows(Rows::Start { items, next }),
            })),
        ]: [Up; 1] = outputs.into_vec().try_into().expect("one answer per engine load")
        else {
            panic!("core page")
        };
        assert!(items.iter().all(|item| matches!(item, port::Item::Record(_))));
        lengths.push(items.len());
        after = next;
    }
    assert!(after.is_none(), "terminal page");
    assert!(routes.idle());
    lengths
}

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
        let mut owners = std::collections::BTreeMap::new();
        for number in 1..=FLIGHTS {
            let request = fixtures::down(&mut routes, connector(number, None));
            let jig_protocol_store::Request::Restore { owner, .. } = &request else {
                panic!("restart page request");
            };
            owners.insert(*owner, number);
            flights.insert(number, (skein_lib::List::with_capacity(ROWS), 0_u32));
            requests.push(request);
        }
        for _ in 0..PAGES {
            // Retain all replies before releasing any; newer flights answer first.
            let mut delayed = skein_lib::Stack::with_capacity(u32::from(FLIGHTS));
            while let Some(request) = requests.pop() {
                let jig_protocol_store::Request::Restore { owner, .. } = &request else {
                    panic!("restart page request");
                };
                let number = owners.remove(owner).expect("retained route owner");
                let mut events = skein_lib::Queue::uncapped();
                fixtures::send(&mut component, request, &mut events);
                while let Some(event) = events.pop() {
                    delayed.push((number, event)).expect("one delayed page per flight");
                }
            }
            while let Some((expected, event)) = delayed.pop() {
                let (rows, terminals) = flights.get_mut(&expected).expect("retained flight");
                let mut outputs = skein_lib::Queue::uncapped();
                route::up(&mut routes, event, &mut outputs);
                let Some(Up::Root(root::Event::ConnectorPage { number, records, after })) = outputs.pop() else {
                    panic!("typed connector page")
                };
                assert!(outputs.is_empty(), "the connector, not the adapter, asks for another page");
                assert_eq!(number, expected);
                assert_eq!(*terminals, 0, "rows precede their terminal");
                for record in records {
                    rows.push(root::Record::Connector { number, record }).expect("one copy of each fixture row");
                }
                if let Some(after) = after {
                    let request = fixtures::down(&mut routes, connector(number, Some(after)));
                    let jig_protocol_store::Request::Restore { owner, .. } = &request else {
                        panic!("restart continuation")
                    };
                    owners.insert(*owner, number);
                    requests.push(request);
                } else {
                    *terminals = terminals.checked_add(1).expect("bounded terminal count");
                }
            }
        }
        assert!(requests.is_empty());
        assert!(routes.idle());
        for (number, (rows, terminals)) in flights {
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
    let request = fixtures::down(
        &mut routes,
        StoreOut::Commit { number: 1, writes: Box::new([root::Write::Save(fixtures::root_record(record))]) },
    );
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
fn load_core_pages_reach_the_root_in_two_answers() {
    let mut component = fixtures::component();
    let mut routes = fixtures::routes();
    fixtures::routed(&mut component, &mut routes, fixtures::core_commit());
    assert_eq!(core_pages(&mut component, &mut routes), [2, 1]);
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
    let outputs = fixtures::routed(&mut component, &mut routes, connector(fixtures::FIRST, None));
    let [Up::Root(root::Event::ConnectorPage { number: fixtures::FIRST, records, after: None })] = outputs.as_ref()
    else {
        panic!("one terminal connector page")
    };
    assert_eq!(records.as_ref(), &[record]);
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
        StoreOut::Engine(Out::Load {
            load: Token::new(17),
            range: Range::Record(core::Key::Tasks(jig_domain_tasks::Key::Ended(task))),
        }),
    );
    match outputs.as_ref() {
        [
            Up::Root(root::Event::Store(In::Loaded {
                load,
                result: Answer::Rows(Rows::Records { records, more: false }),
            })),
        ] => {
            assert_eq!(*load, Token::new(17));
            assert_eq!(records.as_ref(), &[core::Record::Tasks(jig_domain_tasks::Stored::Ended(row))]);
        }
        _ => unreachable!("the retained chat answers its load"),
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
        StoreOut::Engine(port::Out::Load {
            load: Token::new(21),
            range: port::Range::Window { task: 21, from: (1, 1) },
        }),
    );
    let mut expected_turn = 1_u32;
    let mut pages = 0_u32;
    for output in outputs {
        match output {
            Up::Root(root::Event::Store(port::In::Loaded {
                load,
                result: port::Answer::Rows(port::Rows::Turns { turns: rows, more }),
            })) => {
                assert_eq!(load, Token::new(21));
                pages = pages.checked_add(1).expect("bounded pages");
                for (row, text) in rows {
                    assert!(text.is_none());
                    assert_eq!(row.turn, expected_turn);
                    assert_eq!(row.transcript.as_ref(), b"opaque");
                    expected_turn = expected_turn.checked_add(1).expect("bounded turns");
                }
                assert_eq!(more, expected_turn != 4);
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
    let outputs = fixtures::routed(&mut component, &mut routes, connector(fixtures::FIRST, None));
    let [Up::Root(root::Event::ConnectorPage { number: fixtures::FIRST, records, after: None })] = outputs.as_ref()
    else {
        panic!("restored writer row on terminal page")
    };
    assert_eq!(records.as_ref(), &[jig_test_connector::Record::Result { task: 7, code: fixtures::FIRST }]);
    assert!(routes.idle());
}

#[test]
fn restart_configuration_preserves_the_ordinary_read_allowance() {
    let mut limits = fixtures::root_limits();
    limits.core.load_bytes = 7;
    let mut routes = fixtures::routes();
    routes.configure(&limits);
    let request = fixtures::down(
        &mut routes,
        StoreOut::Engine(core::engine::store::Out::Load {
            load: Token::new(1),
            range: core::engine::store::Range::Notes(jig_domain_notes::Range::Entry { name: 1 }),
        }),
    );
    let jig_protocol_store::Request::Load { bytes, .. } = request else { panic!("ordinary load") };
    assert_eq!(bytes, 7);
}

#[test]
fn start_loads_keep_their_exact_ranges_and_bounds() {
    use jig_protocol_store::{Request, Restore};
    let census =
        core::Census { questions: Box::new([(7, 3)]), results: Box::new([core::ReadResults { party: 4, after: 2 }]) };
    let bytes = jig_protocol_store::sizes::restore_bytes(&fixtures::component_limits()).expect("restore allowance");
    for ranges in [
        Box::<[core::RestoreRange]>::from(core::RestoreRange::LIVE),
        Box::from([core::RestoreRange::Census(census)]),
        Box::from([core::RestoreRange::People]),
    ] {
        let mut routes = fixtures::routes();
        let Request::Restore { range: Restore::Jig(found), after: None, rows: 2, bytes: allowance, .. } =
            fixtures::down(&mut routes, start(ranges.clone(), None))
        else {
            panic!("first engine page")
        };
        assert_eq!(found, ranges);
        assert_eq!(allowance, bytes);
    }
    let mut routes = fixtures::routes();
    let Request::Restore { range: Restore::Application(found), after: None, rows: 2, bytes: allowance, .. } =
        fixtures::down(&mut routes, connector(2, None))
    else {
        panic!("first connector page")
    };
    assert_eq!(found.as_ref(), &[jig_test_store::live(2)]);
    assert_eq!(allowance, bytes);
}

#[test]
fn start_connector_pages_decode_rows_in_order_then_finish_the_phase() {
    let mut component = fixtures::starting_component();
    let mut routes = fixtures::routes();
    let rows = [routed_result(1, 1), routed_result(1, 2), routed_result(1, 3)];
    let mut writes = skein_lib::List::with_capacity(4);
    writes.push(fixtures::header(1)).expect("header");
    for row in rows.clone() {
        writes.push(root::Write::Save(row)).expect("fixture row");
    }
    fixtures::routed(&mut component, &mut routes, StoreOut::Commit { number: 1, writes: writes.into_boxed() });
    let outputs = fixtures::routed(&mut component, &mut routes, connector(1, None));
    let [Up::Root(root::Event::ConnectorPage { number: 1, records, after: Some(after) })]: [Up; 1] =
        outputs.into_vec().try_into().expect("one first page")
    else {
        panic!("connector first page")
    };
    assert_eq!(
        records.as_ref(),
        &[
            jig_test_connector::Record::Result { task: 1, code: 1 },
            jig_test_connector::Record::Result { task: 2, code: 1 },
        ]
    );
    let outputs = fixtures::routed(&mut component, &mut routes, connector(1, Some(after)));
    let [Up::Root(root::Event::ConnectorPage { number: 1, records, after: None })] = outputs.as_ref() else {
        panic!("connector terminal page")
    };
    assert_eq!(records.as_ref(), &[jig_test_connector::Record::Result { task: 3, code: 1 }]);
    assert!(routes.idle());
}

#[test]
fn start_core_page_records_reach_the_root_without_people_restored() {
    let mut component = fixtures::starting_component();
    let mut routes = fixtures::routes();
    fixtures::routed(&mut component, &mut routes, fixtures::core_commit());
    assert_eq!(core_pages(&mut component, &mut routes), [2, 1]);
}

#[test]
fn start_missing_owner_keeps_both_keys() {
    use jig_protocol_store::{Event, Request, restore::Item};
    let mut routes = fixtures::routes();
    let Request::Restore { owner, .. } = fixtures::down(
        &mut routes,
        start(
            Box::new([core::RestoreRange::Census(core::Census { questions: Box::new([]), results: Box::new([]) })]),
            None,
        ),
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
    let Some(Up::Root(root::Event::Store(In::Loaded {
        load: START_LOAD,
        result: Answer::Rows(Rows::Start { items, next: None }),
    }))) = out.pop()
    else {
        panic!("typed engine page")
    };
    let [port::Item::Missing { record: found_record, target: found_target }] = items.as_ref() else {
        panic!("missing link")
    };
    assert_eq!((found_record, found_target), (&record, &target));
    assert!(out.is_empty());
    assert!(routes.idle());
}

#[test]
fn refused_start_page_preserves_the_store_failure_and_terminates_the_flight() {
    use jig_protocol_store::{
        Event, Request,
        restore::{Cause, Failure, RangeFailure},
    };
    let mut routes = fixtures::routes();
    let Request::Restore { owner, .. } = fixtures::down(&mut routes, live()) else { panic!("core page") };
    let failure = Failure { cause: Cause::Range(RangeFailure::Cursor), row: None };
    let mut out = skein_lib::Queue::uncapped();
    route::up(&mut routes, Event::PageRefused { owner, failure: failure.clone() }, &mut out);
    let Some(Up::Root(root::Event::StartPageRefused { load: Some(START_LOAD), failure: found })) = out.pop() else {
        panic!("typed page refusal")
    };
    assert_eq!(found, failure);
    assert!(out.is_empty());
    assert!(routes.idle());
}

#[test]
fn switched_start_completes_the_switch_phase() {
    use jig_protocol_store::{Event, Request};
    let mut routes = fixtures::routes();
    let mut out = skein_lib::Queue::uncapped();
    let Some(Request::Switch { owner }) = route::down(&mut routes, StoreOut::Engine(Out::Switch), true, &mut out)
    else {
        panic!("switch request")
    };
    assert!(out.is_empty());
    route::up(&mut routes, Event::Switched { owner }, &mut out);
    let Some(Up::Root(root::Event::Store(In::Switched))) = out.pop() else { panic!("switch done") };
    assert!(out.is_empty());
    assert!(routes.idle());
}

#[test]
fn start_application_decode_failure_names_its_cause_and_has_the_record_key() {
    use jig_protocol_store::{
        Event, Request, Row,
        restore::{Cause, DecodeFailure, Item},
    };
    let mut routes = fixtures::routes();
    let Request::Restore { owner, .. } = fixtures::down(&mut routes, connector(1, None)) else {
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
    let Some(Up::Root(root::Event::StartPageRefused { load: None, failure })) = out.pop() else {
        panic!("typed decode refusal")
    };
    assert_eq!(failure.cause, Cause::Decode(DecodeFailure::Value));
    assert_eq!(
        failure.row.expect("application row").key,
        jig_protocol_store::keys::application(&key, u32::MAX).expect("key")
    );
    assert!(out.is_empty());
    assert!(routes.idle());
}

#[test]
fn a_notes_page_reports_more_and_fails_when_no_record_fits() {
    use jig_domain_notes as notes;
    let mut component = fixtures::component();
    let mut limits = fixtures::root_limits();
    limits.core.notes.load_rows = 1;
    let mut routes = fixtures::routes();
    routes.configure(&limits);
    let line = |name| {
        core::Record::Notes(notes::Record::Line(notes::Line {
            scope: notes::Scope::Deployment,
            name,
            revision: 1,
            description: Box::from(b"note".as_slice()),
        }))
    };
    let writes = Box::new([
        fixtures::header(1),
        root::Write::Save(root::Record::Core(line(1))),
        root::Write::Save(root::Record::Core(line(2))),
    ]);
    fixtures::routed(&mut component, &mut routes, StoreOut::Commit { number: 1, writes });
    let load = Token::new(23);
    let range = notes::Range::Lines { scope: notes::Scope::Deployment, after: None };
    let out = fixtures::routed(
        &mut component,
        &mut routes,
        StoreOut::Engine(Out::Load { load, range: Range::Notes(range.clone()) }),
    );
    let [
        Up::Root(root::Event::Store(In::Loaded {
            load: answered,
            result: Answer::Rows(Rows::Records { records, more: true }),
        })),
    ] = out.as_ref()
    else {
        panic!("one note plus a continuation: {out:?}")
    };
    assert_eq!(*answered, load);
    assert_eq!(records.as_ref(), [line(1)]);
    assert!(routes.idle());
    limits.core.load_bytes = 1;
    routes.configure(&limits);
    let out =
        fixtures::routed(&mut component, &mut routes, StoreOut::Engine(Out::Load { load, range: Range::Notes(range) }));
    let [Up::Root(root::Event::Store(In::Loaded { load: answered, result: Answer::Failed(port::Failure::Failed) }))] =
        out.as_ref()
    else {
        panic!("one byte cannot fit a note: {out:?}")
    };
    assert_eq!(*answered, load);
    assert!(routes.idle());
}
