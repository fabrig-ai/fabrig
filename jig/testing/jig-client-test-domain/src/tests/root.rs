#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "test fixtures collect observed root outputs and locate their variants"
)]
use crate::{Address, Domain, Event, Request, first, unlinked};
use jig_client::Link;
use jig_client::contract::{self, Answer, Outcome, WatchEnd};
use skein_lib::{Duration, List, Queue, RequestEnvelope, Time, Token};
#[test]
fn keyed_requests_leave_only_through_the_table_after_their_records() {
    let mut root = unlinked(&limits(), 7);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    let rows = ask(&mut root, Token::new(42));
    assert_eq!(rows.len(), 2);
    let Request::Save { record: crate::store::Record::Unlinked(record) } = &rows[0] else {
        unreachable!("save precedes send");
    };
    assert!(record.first_sent.is_some());
    let Request::Client {
        request: jig_client::Request::Down { down: jig_client::Down::Send { request: jig_client::Ask::Jig(ask), .. } },
    } = &rows[1]
    else {
        unreachable!("one jig request was sent");
    };
    assert_eq!(**ask, contract::Ask::Stop { project: 1, task: 9 });
    assert!(taken(&mut root).is_empty());
}
#[test]
fn a_final_answer_reaches_the_child_that_asked_unread_and_once() {
    let mut root = unlinked(&limits(), 8);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    let rows = ask(&mut root, Token::new(42));
    let attempt = rows
        .iter()
        .find_map(|r| match r {
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } => {
                Some(*attempt)
            }
            _ => None,
        })
        .unwrap();
    let saved = rows
        .iter()
        .find_map(|row| match row {
            Request::Save { record: crate::store::Record::Unlinked(record) } => Some(record.key),
            _ => None,
        })
        .unwrap();
    let answer = Answer::Done(Outcome::Document(contract::DocumentOutcome::AccountChanged));
    let rows = drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(jig_client::Answered::Jig(answer.clone())),
                    why: None,
                },
            },
        },
    );
    assert_eq!(rows, vec![Request::Erase { key: crate::store::Key::Unlinked(saved) },]);
    assert!(
        drive(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(answer)),
                        why: None
                    }
                }
            }
        )
        .is_empty()
    );
    assert!(taken(&mut root).is_empty());
}
#[test]
fn a_reload_parks_kept_requests_until_jig_confirms_their_party() {
    let mut old = unlinked(&limits(), 9);
    let watch = start(&mut old, List::with_capacity(0));
    confirm(&mut old, watch, 7);
    let rows = ask(&mut old, Token::new(42));
    let record = rows
        .iter()
        .filter_map(|r| match r {
            Request::Save { record: crate::store::Record::Unlinked(record) } => {
                Some(crate::store::Record::Unlinked(record.clone()))
            }
            _ => None,
        })
        .next_back()
        .unwrap();
    let crate::store::Record::Unlinked(value) = &record else { unreachable!("client record") };
    let saved = value.key;
    let mut kept = List::with_capacity(1);
    kept.push(record).unwrap();
    let mut root = unlinked(&limits(), 10);
    let watch = start(&mut root, kept);
    assert!(taken(&mut root).is_empty());
    confirm(&mut root, watch, 8);
    assert!(taken(&mut root).is_empty());
    let sends = confirm(&mut root, watch, 7);
    let attempt = sends
        .iter()
        .find_map(|r| match r {
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } => {
                Some(*attempt)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(
        drive(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(Answer::Done(Outcome::Document(
                            contract::DocumentOutcome::AccountChanged
                        )))),
                        why: None
                    }
                }
            }
        ),
        vec![Request::Erase { key: crate::store::Key::Unlinked(saved) }]
    );
}
#[test]
fn link_and_confirmation_reach_both_children_and_park_the_table() {
    let mut root = unlinked(&limits(), 11);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    assert_eq!(root.first().link(), Link::Live);
    assert_eq!(root.second().link(), Link::Live);
    assert_eq!(root.first().party(), Some(7));
    assert_eq!(root.second().party(), Some(7));
    ask(&mut root, Token::new(42));
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: WatchEnd::Dropped } },
        },
    );
    assert_eq!(root.first().link(), Link::Offline);
    assert_eq!(root.second().link(), Link::Offline);
    for row in taken(&mut root) {
        if let Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { .. } } } = row {
            unreachable!("offline requests are parked");
        }
    }
    drive(&mut root, Event::SignedOut);
    assert_eq!(root.first().party(), None);
    assert_eq!(root.second().party(), None);
}

#[test]
fn a_current_signed_out_envelope_tells_jig_and_both_siblings() {
    let mut root = unlinked(&limits(), 12);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    let rows = ask(&mut root, Token::new(42));
    let Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } =
        &rows[1]
    else {
        unreachable!("one request is in flight");
    };
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: *attempt,
                    envelope: RequestEnvelope::SignedOut,
                    answer: None,
                    why: None,
                },
            },
        },
    );
    drive(&mut root, Event::Client { event: jig_client::Event::Up { up: jig_client::Up::SignedOut } });
    assert_eq!(root.first().party(), None);
    assert_eq!(root.second().party(), None);
    for row in taken(&mut root) {
        if let Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { .. } } } = row {
            unreachable!("signed out requests remain parked");
        }
    }
}

fn ready(seed: u64) -> Domain {
    let mut root = unlinked(&limits(), seed);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    root
}
fn family_read(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { .. } },
                    },
            } => Some(*read),
            _ => None,
        })
        .unwrap()
}
fn family_watch(rows: &[Request]) -> Token {
    rows.iter()
        .find_map(|row| match row {
            Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { .. } },
                    },
            } => Some(*watch),
            _ => None,
        })
        .unwrap()
}
fn item(revision: u64) -> crate::family::Document {
    crate::family::Document::Connector(crate::connector_values::ConnectorDocument {
        subject: crate::family::Subject { scope: crate::family::Scope::Project(3), kind: crate::family::Kind::Item(5) },
        value: Box::new(crate::connector_values::ClientDocument::Object {
            object: 5,
            now: crate::connector_values::ApplicationObject { revision, value: 13 },
        }),
    })
}

fn family_page(root: &mut Domain) -> Vec<Request> {
    drive(
        root,
        Event::First(first::Event::Compose {
            address: first::Address { project: 3, item: 5 },
            composition: crate::family::PageComposition {
                task: 9,
                second: None,
                proposals: {
                    let mut proposals = List::with_capacity(1);
                    proposals.push(contract::ProposalKey { proposer: contract::Proposer::Task(9), number: 2 }).unwrap();
                    proposals
                },
            },
        }),
    );
    drive(root, Event::Went { place: crate::Place::Address(Address::First(first::Address { project: 3, item: 5 })) })
}
#[test]
fn a_family_page_hands_its_task_and_proposal_to_jigs_working_set() {
    let mut root = ready(22);
    let rows_initial = family_page(&mut root);
    let read = family_read(&rows_initial);
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Value(item(1)) }));
    assert!(root.jig().objects().find(contract::ObjectKey::Task(9)).is_some());
    assert!(
        root.jig()
            .objects()
            .find(contract::ObjectKey::Proposal(contract::ProposalKey {
                proposer: contract::Proposer::Task(9),
                number: 2
            }))
            .is_some()
    );
    assert!(rows_initial.iter().any(|row| match row {
        Request::Client {
            request:
                jig_client::Request::Down {
                    down:
                        jig_client::Down::Open {
                            subject: jig_client::Watchable::Jig(contract::Subject::Chips { .. }), ..
                        },
                },
        } => true,
        _ => false,
    }));
    assert!(rows_initial.iter().any(|row| match row {
        Request::Client {
            request:
                jig_client::Request::Down {
                    down:
                        jig_client::Down::Open {
                            subject: jig_client::Watchable::Jig(contract::Subject::Inbox { .. }), ..
                        },
                },
        } => true,
        _ => false,
    }));
    let watch = family_watch(&rows_initial);
    let rows =
        drive(&mut root, Event::First(first::Event::Watch { watch, event: first::WatchEvent::Delivered(item(1)) }));
    assert!(!rows.iter().any(|row| match row {
        Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Open { .. } } } => true,
        _ => false,
    }));
}
#[test]
fn family_change_refuses_a_moved_revision_and_final_answers_stay_with_the_family() {
    let mut root = ready(23);
    let rows = family_page(&mut root);
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(2)) }),
    );
    let id = root.first().page().unwrap();
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::ChangeObject {
                object: id.token(),
                revision: contract::Revision { value: 1 },
            }),
        },
    );
    assert_eq!(
        root.first().object(id).unwrap().card,
        first::Card::Refused {
            answer: first::Answer::Connector(crate::connector_values::ConnectorAnswer {
                value: crate::connector_values::ApplicationOutcome::Moved {
                    object: 5,
                    now: crate::connector_values::ApplicationObject { revision: 2, value: 13 }
                }
            })
        }
    );
    assert!(taken(&mut root).is_empty());
    let rows = drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::ChangeObject {
                object: id.token(),
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    assert!(taken(&mut root).is_empty());
    let Request::Save { record: crate::store::Record::Unlinked(_) } = &rows[0] else { panic!("record first") };
    let Request::Client {
        request:
            jig_client::Request::Down {
                down: jig_client::Down::Send { attempt, request: jig_client::Ask::Family { record: ask, .. }, .. },
            },
    } = &rows[1]
    else {
        panic!("family request")
    };
    assert_eq!(
        *ask,
        family_request(first::Ask::ChangeObject {
            project: 3,
            object: 5,
            revision: contract::Revision { value: 2 },
            value: 13
        })
    );
    let answer = first::Answer::Connector(crate::connector_values::ConnectorAnswer {
        value: crate::connector_values::ApplicationOutcome::Changed {
            object: 5,
            now: crate::connector_values::ApplicationObject { revision: 3, value: 13 },
        },
    });
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: *attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(family_answer(answer.clone())),
                    why: None,
                },
            },
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Done { answer });
}
#[test]
fn object_parameters_are_the_familys_revision_naming_request() {
    let mut root = ready(24);
    let rows = family_page(&mut root);
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(2)) }),
    );
    let object = root.first().page().unwrap().token();
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Parameters {
                object,
                revision: contract::Revision { value: 2 },
                value: 21,
            }),
        },
    );
    let rows = drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::ChangeObject {
                object,
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    let Request::Client {
        request:
            jig_client::Request::Down {
                down: jig_client::Down::Send { request: jig_client::Ask::Family { record: ask, .. }, .. },
            },
    } = &rows[1]
    else {
        panic!("family parameters request")
    };
    assert_eq!(
        *ask,
        family_request(first::Ask::ChangeObject {
            project: 3,
            object: 5,
            revision: contract::Revision { value: 2 },
            value: 21,
        })
    );
}

#[test]
fn closing_family_watches_keep_their_terminal_obligation_and_stale_tokens_do_nothing() {
    let mut root = ready(25);
    let rows = family_page(&mut root);
    let watch = family_watch(&rows);
    let read = family_read(&rows);
    drive(
        &mut root,
        Event::Went {
            place: crate::Place::Address(Address::Jig(jig_client_view::address::Address::Inbox { project: None })),
        },
    );
    drive(&mut root, Event::First(first::Event::Watch { watch, event: first::WatchEvent::Delivered(item(2)) }));
    assert!(root.first().page().is_none());
    assert!(drive(&mut root, Event::First(first::Event::Ended { watch, end: first::WatchEnd::Closed })).is_empty());
    assert!(drive(&mut root, Event::First(first::Event::Ended { watch, end: first::WatchEnd::Closed })).is_empty());
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Value(item(2)) }));
    drive(&mut root, Event::First(first::Event::Read { read, result: first::ReadResult::Value(item(2)) }));
    let rows = family_page(&mut root);
    let current = family_watch(&rows);
    assert_ne!(watch, current);
    drive(&mut root, Event::First(first::Event::Ended { watch: current, end: first::WatchEnd::Dropped }));
    assert_eq!(crate::next_deadline(&root), Some(Time::ZERO.saturating_add(Duration::from_nanos(5))));
    let mut out = Queue::uncapped();
    let mut later = env();
    later.now = Time::ZERO.saturating_add(Duration::from_nanos(10));
    crate::fire(&mut root, &later, &mut out);
    assert_ne!(family_watch(&out_iter(out)), current);
}

#[test]
fn an_unknown_family_request_inspects_before_another_change() {
    let mut root = ready(26);
    let rows = family_page(&mut root);
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(2)) }),
    );
    let object = root.first().page().unwrap();
    let action = crate::Action::First(first::Act::ChangeObject {
        object: object.token(),
        revision: contract::Revision { value: 2 },
    });
    drive(&mut root, Event::Act { action: action.clone() });
    let first::Card::Deciding { request, .. } = root.first().object(object).unwrap().card else {
        panic!("the family owns its deciding request");
    };
    let rows = drive(
        &mut root,
        Event::First(first::Event::Progress { request, progress: skein_lib::RequestProgress::Unknown }),
    );
    assert_eq!(root.first().object(object).unwrap().card, first::Card::Unknown);
    assert_eq!(root.first().object(object).unwrap().value, first::Value::Loading);
    assert!(drive(&mut root, Event::Act { action }).is_empty());
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(3)) }),
    );
    assert_eq!(root.first().object(object).unwrap().card, first::Card::Open);
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::ChangeObject {
                object: object.token(),
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    assert_eq!(
        root.first().object(object).unwrap().card,
        first::Card::Refused {
            answer: first::Answer::Connector(crate::connector_values::ConnectorAnswer {
                value: crate::connector_values::ApplicationOutcome::Moved {
                    object: 5,
                    now: crate::connector_values::ApplicationObject { revision: 3, value: 13 }
                }
            })
        }
    );
}

#[test]
fn family_capacities_refuse_overflow_and_resource_words_past_the_owner_bound() {
    let mut bounds = limits();
    bounds.first.objects = u32::MAX;
    assert!(!crate::valid(&bounds));
    bounds = limits();
    bounds.first.retry = Duration::ZERO;
    assert!(!crate::valid(&bounds));
    let mut root = ready(27);
    let part = contract::ApplicationPart::Resource { connector: crate::FIRST, path: resource_path(&[b'x'; 33]) };
    let rows = drive(
        &mut root,
        Event::First(first::Event::Show { part: part.clone(), scope: crate::family::Scope::Deployment }),
    );
    assert_eq!(rows, vec![Request::First(first::Out::Fact { fact: first::Fact::Full })]);
    assert!(!root.first().contains(&part));
    let mut side = crate::second::Side::new(1, 1, 32, 1);
    assert!(!side.show(part));
    let part = contract::ApplicationPart::Kind { connector: crate::SECOND, kind: 1 };
    assert!(side.show(part.clone()));
    assert!(side.show(part));
    assert!(!side.show(contract::ApplicationPart::Kind { connector: crate::SECOND, kind: 2 }));
}

fn restored_requests(count: u32) -> List<crate::store::Record> {
    let mut old = unlinked(&limits(), 30);
    let watch = start(&mut old, List::with_capacity(0));
    confirm(&mut old, watch, 7);
    let mut kept = List::with_capacity(count);
    for n in 0..count {
        let rows = drive(
            &mut old,
            Event::Client {
                event: jig_client::Event::Send {
                    name: None,
                    key: jig_client::Key { asker: jig_client::Asker::JIG, number: u64::from(n) },
                    request: jig_client::Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 + u64::from(n) })),
                },
            },
        );
        for row in rows {
            if let Request::Save { record: crate::store::Record::Unlinked(record) } = row {
                kept.push(crate::store::Record::Unlinked(record)).unwrap();
            }
        }
    }
    kept
}

#[test]
fn restored_foreign_scope_counts_are_routed_without_root_page_policy() {
    let mut root = unlinked(&limits(), 31);
    let watch = start(&mut root, restored_requests(2));
    confirm(&mut root, watch, 8);
    assert_eq!(root.jig().page(), &jig_client_view::page::Page::SomeoneElse);
    assert_eq!(root.first().party(), Some(8));
    assert!(taken(&mut root).is_empty());
    let sent = confirm(&mut root, watch, 7);
    assert_eq!(
        sent.iter()
            .filter(|r| match r {
                Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { .. } } } => true,
                _ => false,
            })
            .count(),
        2
    );
}

#[test]
fn restored_own_scope_sends_and_retires_each_final_request_once() {
    let mut root = unlinked(&limits(), 32);
    let watch = start(&mut root, restored_requests(2));
    let sent = confirm(&mut root, watch, 7);
    assert!(work_page(root.jig().page()));
    let sends: Vec<_> = sent
        .into_iter()
        .filter_map(|row| match row {
            Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { attempt, .. } } } => {
                Some(attempt)
            }
            _ => None,
        })
        .collect();
    assert_eq!(sends.len(), 2);
    for attempt in sends {
        let out = drive(
            &mut root,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Answer {
                        attempt,
                        envelope: RequestEnvelope::Final,
                        answer: Some(jig_client::Answered::Jig(Answer::Done(Outcome::Document(
                            contract::DocumentOutcome::AccountChanged,
                        )))),
                        why: None,
                    },
                },
            },
        );
        let [Request::Erase { key: crate::store::Key::Unlinked(_) }] = out.as_slice() else {
            panic!("the final client request is erased once")
        };
    }
    confirm(&mut root, watch, 8);
    assert_eq!(
        root.jig().page(),
        &jig_client_view::page::Page::Work {
            address: jig_client_view::address::Address::Inbox { project: None },
            behind: false,
            excess: 0,
            refused: None
        }
    );
}

fn work_page(page: &jig_client_view::page::Page) -> bool {
    match page {
        jig_client_view::page::Page::Work { .. } => true,
        jig_client_view::page::Page::Starting
        | jig_client_view::page::Page::Elsewhere
        | jig_client_view::page::Page::SigningIn
        | jig_client_view::page::Page::Missing
        | jig_client_view::page::Page::OutOfDate
        | jig_client_view::page::Page::SomeoneElse => false,
    }
}

#[test]
fn a_raw_entrance_refusal_finishes_the_original_family_request_once_with_its_facts() {
    let mut root = ready(29);
    let rows = family_page(&mut root);
    drive(
        &mut root,
        Event::First(first::Event::Read { read: family_read(&rows), result: first::ReadResult::Value(item(2)) }),
    );
    let id = root.first().page().unwrap();
    let rows = drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::ChangeObject {
                object: id.token(),
                revision: contract::Revision { value: 2 },
            }),
        },
    );
    let Request::Save { .. } = &rows[0] else { panic!("retention precedes send") };
    let Request::Client {
        request:
            jig_client::Request::Down {
                down: jig_client::Down::Send { attempt, request: jig_client::Ask::Family { .. }, .. },
            },
    } = &rows[1]
    else {
        panic!("family request")
    };
    let answer = first::Answer::Refused(
        contract::EntranceRefusal::new(contract::Refusal::Authority {
            lacking: contract::Lacking::Budget { needed: 19, left: 3 },
            proposable: Some(false),
        })
        .unwrap(),
    );
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: *attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(family_answer(answer.clone())),
                    why: None,
                },
            },
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Refused { answer: answer.clone() });
    let shown = crate::shown(&root);
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt: *attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(family_answer(first::Answer::Connector(crate::connector_values::ConnectorAnswer {
                        value: crate::connector_values::ApplicationOutcome::Unknown,
                    }))),
                    why: None,
                },
            },
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Refused { answer });
    assert_eq!(crate::shown(&root), shown);
    assert!(taken(&mut root).iter().all(|request| match request {
        Request::Client { request: jig_client::Request::Down { down: jig_client::Down::Send { .. } } } => false,
        _ => true,
    }));
}

#[test]
fn actual_application_object_form_keeps_explicit_zero_observed_revision_and_original_final_child() {
    use crate::connector_values::{
        ApplicationObject, ApplicationOutcome, ClientDocument, ConnectorAnswer, ConnectorDocument,
    };
    use crate::family::{Document, Kind, Scope, Subject};
    let mut root = ready(91);
    let opened = family_page(&mut root);
    let subject = Subject { scope: Scope::Project(3), kind: Kind::Item(5) };
    let value = Document::Connector(ConnectorDocument {
        subject,
        value: Box::new(ClientDocument::Object { object: 5, now: ApplicationObject { revision: 17, value: 64 } }),
    });
    drive(
        &mut root,
        Event::First(first::Event::Read {
            read: family_read(&opened),
            result: first::ReadResult::Value(value.clone()),
        }),
    );
    let id = root.first().find(subject).unwrap();
    assert_eq!(root.first().object(id).unwrap().value, first::Value::Known(value));
    drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::Parameters {
                object: id.token(),
                revision: contract::Revision { value: 17 },
                value: 0,
            }),
        },
    );
    let sent = drive(
        &mut root,
        Event::Act {
            action: crate::Action::First(first::Act::ChangeObject {
                object: id.token(),
                revision: contract::Revision { value: 17 },
            }),
        },
    );
    let (attempt, ask) =
        sent.iter()
            .find_map(|request| match request {
                Request::Client {
                    request:
                        jig_client::Request::Down {
                            down:
                                jig_client::Down::Send {
                                    attempt, request: jig_client::Ask::Family { record: ask, .. }, ..
                                },
                        },
                } => Some((*attempt, ask)),
                _ => None,
            })
            .expect("one keyed complete application request");
    assert_eq!(
        ask,
        &family_request(crate::family::Ask::ChangeObject {
            project: 3,
            object: 5,
            revision: contract::Revision { value: 17 },
            value: 0
        })
    );
    let answer = crate::family::Answer::Connector(ConnectorAnswer {
        value: ApplicationOutcome::Changed { object: 5, now: ApplicationObject { revision: 18, value: 0 } },
    });
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(family_answer(answer.clone())),
                    why: None,
                },
            },
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Done { answer: answer.clone() });
    drive(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: RequestEnvelope::Final,
                    answer: Some(family_answer(crate::family::Answer::Connector(ConnectorAnswer {
                        value: ApplicationOutcome::Role { held: u32::MAX },
                    }))),
                    why: None,
                },
            },
        },
    );
    assert_eq!(root.first().object(id).unwrap().card, first::Card::Done { answer });
}

#[test]
fn whole_owner_parts_require_original_inner_identity_and_keep_unversioned_snapshots_without_forms() {
    use crate::connector_values::{ClientDocument, ConnectorDocument};
    use crate::family::{Document, Kind, Scope, Subject};
    let mut root = ready(92);
    let part = contract::ApplicationPart::Panel { connector: crate::FIRST, task: 9 };
    let opened = drive(&mut root, Event::First(first::Event::Show { part, scope: Scope::Project(3) }));
    let subject = Subject { scope: Scope::Project(3), kind: Kind::Panel(9) };
    let watch = family_watch(&opened);
    let mut too_many = List::with_capacity(1);
    too_many
        .push(crate::connector_values::ClientResource {
            path: crate::connector_values::Path::new(resource_path(b"r"), 32).unwrap(),
            role: crate::connector_values::ResourceRole::Owned,
            hold: crate::connector_values::Hold::None,
            fact: None,
        })
        .unwrap();
    for bad in [
        ClientDocument::Panel {
            task: 10,
            procedure: None,
            result: None,
            resources: List::with_capacity(0),
            effects: List::with_capacity(0),
        },
        ClientDocument::Panel {
            task: 9,
            procedure: None,
            result: None,
            resources: too_many,
            effects: List::with_capacity(0),
        },
    ] {
        let outputs = drive(
            &mut root,
            Event::First(first::Event::Watch {
                watch,
                event: first::WatchEvent::Delivered(Document::Connector(ConnectorDocument {
                    subject,
                    value: Box::new(bad),
                })),
            }),
        );
        let mut invalid = false;
        for request in &outputs {
            if let Request::First(first::Out::Fact { fact: first::Fact::InvalidDocument }) = request {
                invalid = true;
            }
        }
        assert!(invalid);
        assert_eq!(root.first().object(root.first().find(subject).unwrap()).unwrap().value, first::Value::Loading);
    }
    for result in [None, Some(0), Some(u16::MAX)] {
        let document = Document::Connector(ConnectorDocument {
            subject,
            value: Box::new(ClientDocument::Panel {
                task: 9,
                procedure: None,
                result,
                resources: List::with_capacity(0),
                effects: List::with_capacity(0),
            }),
        });
        assert_eq!(document.revision(), None);
        drive(
            &mut root,
            Event::First(first::Event::Watch { watch, event: first::WatchEvent::Delivered(document.clone()) }),
        );
        let id = root.first().find(subject).unwrap();
        assert_eq!(root.first().object(id).unwrap().value, first::Value::Known(document));
        drive(
            &mut root,
            Event::Act {
                action: crate::Action::First(first::Act::ChangeObject {
                    object: id.token(),
                    revision: contract::Revision { value: 0 },
                }),
            },
        );
        for request in taken(&mut root) {
            if let Request::Client {
                request:
                    jig_client::Request::Down {
                        down: jig_client::Down::Send { request: jig_client::Ask::Family { .. }, .. },
                    },
            } = request
            {
                panic!("an unversioned snapshot must not submit an Object form");
            }
        }
    }
}

mod client;
mod fixtures;
mod navigation;
use fixtures::{
    ask, confirm, drive, env, family_answer, family_request, limits, out_iter, resource_path, start, taken,
};

pub(super) fn live_first(first: first::Limits) -> (first::Side, jig_client::Client) {
    let mut root = unlinked(&crate::Limits { first, ..limits() }, 1);
    let watch = start(&mut root, List::with_capacity(0));
    confirm(&mut root, watch, 7);
    (root.first, root.client)
}

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    let base = limits();
    assert!(crate::valid(&base));
    let mut jig = base;
    jig.jig.save_delay = Duration::ZERO;
    let mut first = base;
    first.first.retry = Duration::ZERO;
    let mut client = base;
    client.client.domain.heartbeat = Duration::ZERO;
    for (condition, invalid) in [("jig limits", jig), ("first limits", first), ("client limits", client)] {
        assert!(!crate::valid(&invalid), "{condition}");
    }
}

#[test]
fn view_retention_does_not_price_the_clients_receiving_bound() {
    let mut bounds = limits();
    bounds.client.domain.document_bytes = u64::MAX;
    assert!(jig_client_view::valid(&bounds.jig, &bounds.client.domain));
    assert!(!crate::valid(&bounds));
}
