use super::fixtures::{self, Client, SUBJECT_TASK, TASK};
use crate::{Event, Handle, Message, Readable, Request, With, contract as c};
use jig_client_protocol_wire::v28 as w;
use skein_lib::{List, Reader, Secret, Time, Writer};

fn client() -> Client {
    let mut limits = fixtures::limits();
    limits.exchanges = 1;
    limits.domain.reads = 2;
    let mut client = Client::with_limits(limits, List::with_capacity(0));
    client.step(Event::SignIn { with: With::Held(Secret::new(Box::new([7; 40]))) });
    client.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    client.step(Event::Stored);
    client
}

#[test]
fn a_lost_read_retries_once_and_completes_its_current_handle() {
    for answer in [false, true] {
        let mut c = Client::signed_in();
        let first = c.read(TASK);
        let [Request::Exchange { message: Message::Read(original), .. }] = c.requests() else { panic!("first read") };
        let original = original.clone();
        c.step(Event::Lost { exchange: first });
        let [Request::Exchange { exchange, message: Message::Read(retried), .. }] = c.requests() else {
            panic!("one read retry: {:?}", c.requests())
        };
        assert_ne!(*exchange, first);
        assert_eq!(*retried, original);
        let second = *exchange;
        let expected = if answer {
            c.step(Event::Reply { exchange: second, message: fixtures::unknown_read() });
            c::ReadNone::Unknown
        } else {
            c.step(Event::Lost { exchange: second });
            c::ReadNone::Unreachable
        };
        assert_eq!(
            c.requests(),
            &[Request::Told {
                told: crate::Told::Page { handle: TASK, page: crate::Readout::Jig(c::ReadResult::None(expected)) }
            }]
        );
    }
}

#[test]
fn a_lost_read_does_not_retry_a_cancelled_or_replaced_handle() {
    for replaced in [false, true] {
        let mut c = Client::signed_in();
        let old = c.read(TASK);
        let current = if replaced {
            Some(c.read(TASK))
        } else {
            c.step(Event::Unwatch { handle: TASK });
            None
        };
        c.step(Event::Lost { exchange: old });
        assert!(c.requests().is_empty(), "no retry for an obsolete read");
        if let Some(current) = current {
            c.step(Event::Reply { exchange: current, message: fixtures::unknown_read() });
            assert_eq!(
                c.requests(),
                &[Request::Told {
                    told: crate::Told::Page {
                        handle: TASK,
                        page: crate::Readout::Jig(c::ReadResult::None(c::ReadNone::Unknown)),
                    }
                }]
            );
        }
    }
}

#[test]
fn a_conversations_original_read_leaves_once_when_an_exchange_slot_is_released() {
    let mut client = client();
    let occupied = client.read(Handle { number: 100, ..TASK });
    let mut thread = fixtures::thread::model();
    let read = thread.moved(4).unwrap();
    client.step(Event::Read { handle: TASK, read: Readable::Jig(read) });
    assert!(client.requests().is_empty(), "local pressure keeps the original demand, without a terminal");
    assert!(thread.reading.is_some(), "the conversation still owns its read");
    let admission = client.machine.admission();
    assert_eq!(admission.attempts, 1);
    assert_eq!(admission.deferred_reads, 1);
    client.fire(Time::ZERO);
    assert!(client.requests().is_empty(), "no timer admits the read while the slot is occupied");

    client.step(Event::Reply { exchange: occupied, message: fixtures::unknown_read() });
    let mut admitted = None;
    for request in client.requests() {
        if let Request::Exchange { exchange, message: Message::Read(bytes), .. } = request {
            assert!(admitted.replace(*exchange).is_none(), "exactly one deferred read leaves");
            let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
            let w::Read::Activity(activity) = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).unwrap()
            else {
                panic!("the original conversation demand")
            };
            assert_eq!(activity.task(), SUBJECT_TASK);
            assert!(activity.before().is_none());
        }
    }
    assert!(admitted.is_some(), "the release is the wake, without another document or fire");
    client.step(Event::Reply { exchange: occupied, message: fixtures::unknown_read() });
    assert!(client.requests().is_empty(), "a stale release does not send the read twice");
    client.fire(Time::ZERO);
    assert_eq!((client.requests()).len(), 0);
    assert!(thread.loaded < thread.committed);
}

#[test]
fn replacing_a_queued_read_keeps_only_the_current_demand_within_the_read_bound() {
    let mut client = client();
    let occupied = client.read(Handle { number: 100, ..TASK });
    for project in 2..12 {
        client.step(Event::Read { handle: TASK, read: Readable::Jig(c::Read::Project { project }) });
        assert_eq!((client.requests()).len(), 0);
    }
    client.step(Event::Read {
        handle: Handle { number: 101, ..TASK },
        read: Readable::Jig(c::Read::Project { project: 12 }),
    });
    let [Request::Told { told: crate::Told::Page { page: crate::Readout::Limit, .. } }] = client.requests() else {
        panic!("the unchanged logical read bound still refuses a third current read")
    };
    client.step(Event::Reply { exchange: occupied, message: fixtures::unknown_read() });
    let mut reads = List::with_capacity(2);
    for request in client.requests() {
        if let Request::Exchange { message: Message::Read(bytes), .. } = request {
            reads.push(bytes).unwrap();
        }
    }
    assert_eq!(reads.len(), 1);
    let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(reads.get(0).unwrap())).unwrap();
    let w::Read::Project(project) = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).unwrap() else {
        panic!("only the replacement project read leaves")
    };
    assert_eq!(project.project(), 11);
}

#[test]
fn unwatch_and_a_credential_boundary_discard_queued_reads_before_slots_are_reused() {
    for signing_in in [false, true] {
        let mut client = client();
        let occupied = client.read(Handle { number: 100, ..TASK });
        client.step(Event::Read {
            handle: TASK,
            read: Readable::Jig(c::Read::Activity { task: SUBJECT_TASK, before: None }),
        });
        assert_eq!((client.requests()).len(), 0);
        if signing_in {
            client.step(Event::SignIn { with: With::Held(Secret::new(Box::new([9; 40]))) });
            let mut terminals = List::with_capacity(2);
            for request in client.requests() {
                if let Request::Told { told: crate::Told::Page { handle, page } } = request
                    && *handle == TASK
                {
                    terminals.push(page).unwrap();
                }
            }
            assert_eq!(terminals.as_slice(), &[&crate::Readout::Jig(c::ReadResult::None(c::ReadNone::SignedOut))]);
        } else {
            client.step(Event::Unwatch { handle: TASK });
        }
        for request in client.requests() {
            if let Request::Exchange { message: Message::Read(_), .. } = request {
                panic!("no cancelled read is admitted")
            }
        }
        client.step(Event::Reply { exchange: occupied, message: fixtures::unknown_read() });
        for request in client.requests() {
            if let Request::Exchange { message: Message::Read(_), .. } = request {
                panic!("no cancelled read is admitted")
            }
        }
        client.fire(Time::ZERO);
        for request in client.requests() {
            if let Request::Exchange { message: Message::Read(_), .. } = request {
                panic!("no cancelled read is admitted")
            }
        }
    }
}

#[test]
fn peer_refusals_and_an_exhausted_loss_end_an_admitted_read() {
    for (answer, expected) in [
        (Some(w::ReadNone::Busy), c::ReadNone::Busy),
        (Some(w::ReadNone::Unknown), c::ReadNone::Unknown),
        (None, c::ReadNone::Unreachable),
    ] {
        let mut client = client();
        let occupied = client.read(Handle { number: 100, ..TASK });
        client.step(Event::Read {
            handle: TASK,
            read: Readable::Jig(c::Read::Activity { task: SUBJECT_TASK, before: None }),
        });
        assert_eq!((client.requests()).len(), 0);
        client.step(Event::Reply { exchange: occupied, message: fixtures::unknown_read() });
        let admitted = client.exchange();
        match answer {
            Some(answer) => client.step(Event::Reply { exchange: admitted, message: fixtures::read_none(answer) }),
            None => {
                client.step(Event::Lost { exchange: admitted });
                client.step(Event::Lost { exchange: client.exchange() });
            }
        }
        let [Request::Told { told: crate::Told::Page { handle, page } }] = client.requests() else {
            panic!("one actual terminal: {:?}", client.requests())
        };
        assert_eq!(*handle, TASK);
        assert_eq!(*page, crate::Readout::Jig(c::ReadResult::None(expected)));
        client.fire(Time::ZERO);
        assert!(client.requests().is_empty(), "a peer result or exhausted loss has no further retry");
    }
}

#[test]
fn a_peer_version_refusal_ends_queued_reads_instead_of_admitting_them() {
    let mut client = client();
    let occupied = client.read(Handle { number: 100, ..TASK });
    client.step(Event::Read {
        handle: TASK,
        read: Readable::Jig(c::Read::Activity { task: SUBJECT_TASK, before: None }),
    });
    assert_eq!((client.requests()).len(), 0);
    let answer = w::ReadAnswer::NotServed(
        w::NotServed::new(&w::CEILINGS, w::NotServedParts { owner: w::Owner::Jig, served: List::with_capacity(0) })
            .unwrap(),
    );
    let mut bytes = Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut bytes).unwrap();
    client.step(Event::Reply { exchange: occupied, message: bytes.finish() });
    let mut terminal = None;
    for request in client.requests() {
        if let Request::Told { told: crate::Told::Page { handle, page } } = request
            && *handle == TASK
        {
            assert!(terminal.replace(page).is_none());
        }
    }
    let Some(crate::Readout::NotServed { family: None, .. }) = terminal else {
        panic!("the peer's unserved-version result ends the queued read")
    };
    client.step(Event::Reply { exchange: occupied, message: fixtures::unknown_read() });
    for request in client.requests() {
        if let Request::Exchange { message: Message::Read(_), .. } = request {
            panic!("no refused read is admitted")
        }
    }
}

/// O2: the old link's consequences leave before the new domain's requests.
#[test]
fn link_consequences_precede_the_next_domain_request() {
    let mut limits = fixtures::limits();
    limits.exchanges = 3;
    let mut client = Client::with_limits(limits, List::with_capacity(0));
    client.step(Event::SignIn { with: With::Held(fixtures::credential(&fixtures::CREDENTIAL)) });
    client.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    let mut old_exchanges = List::with_capacity(3);
    client.step(Event::Stored);
    for task in [20, 21, 22] {
        client.step(Event::Watch {
            handle: Handle { number: task, ..TASK },
            subject: crate::Watchable::Jig(c::Subject::Task { task }),
        });
        if task == 20 {
            old_exchanges.push(client.exchange()).unwrap();
        }
    }
    let blockers = [Handle { number: 200, ..TASK }, Handle { number: 201, ..TASK }];
    for handle in blockers {
        old_exchanges.push(client.read(handle)).unwrap();
    }
    for handle in [TASK, fixtures::ME] {
        client.step(Event::Read { handle, read: Readable::Jig(c::Read::Project { project: 1 }) });
    }
    client.step(Event::SignIn { with: With::Held(fixtures::credential(&fixtures::NEW_CREDENTIAL)) });
    let Some(Request::CloseStream) = client.requests().first() else { panic!("the old stream closes first") };
    let mut opening = None;
    for (index, request) in client.requests().iter().enumerate() {
        if let Request::OpenStream { .. } = request {
            assert!(opening.replace(index).is_none());
        }
    }
    let opening = opening.expect("new stream");
    for handle in [TASK, fixtures::ME, Handle { number: 200, ..TASK }, Handle { number: 201, ..TASK }] {
        let mut ending = None;
        for (index, request) in client.requests().iter().enumerate() {
            if let Request::Told { told: crate::Told::Page { handle: ended, .. } } = request
                && *ended == handle
            {
                assert!(ending.replace(index).is_none(), "one terminal for the old read");
            }
        }
        assert!(ending.expect("old read ends") < opening, "old reads end before the new stream opens");
    }
    for number in [20, 21, 22] {
        let mut ending = None;
        for (index, request) in client.requests().iter().enumerate() {
            if let Request::Told { told: crate::Told::Ended { handle, .. } } = request
                && handle.number == number
            {
                assert!(ending.replace(index).is_none(), "one terminal for the old watch");
            }
        }
        assert!(ending.expect("old watch ends") < opening);
    }

    // Ending the old sign-in cancels logical attempts, not their transport credit.
    for exchange in &old_exchanges {
        client.step(Event::Reply { exchange: *exchange, message: Box::new([]) });
        assert!(client.requests().is_empty(), "late old terminals only free credit");
    }
    // Three reads occupy the exchange edge; the next
    // two reads must wait, then leave in order as one slot is released.
    client.stream(fixtures::hello(jig_client_protocol_wire::VERSION));
    client.step(Event::Stored);
    let opening = client.read(Handle { number: 300, ..TASK });
    let first = client.read(TASK);
    let second = client.read(fixtures::ME);
    for number in [30, 31] {
        client.step(Event::Read {
            handle: Handle { number, ..TASK },
            read: Readable::Jig(c::Read::Project { project: u32::try_from(number).unwrap() }),
        });
        assert!(client.requests().is_empty(), "the edge is occupied");
    }
    for (exchange, project) in [(first, 30), (second, 31)] {
        client.step(Event::Reply { exchange, message: fixtures::unknown_read() });
        let mut sent = List::with_capacity(1);
        for request in client.requests() {
            if let Request::Exchange { message: Message::Read(bytes), .. } = request {
                let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
                let w::Read::Project(read) = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).unwrap()
                else {
                    panic!("deferred project read")
                };
                sent.push(read.project()).unwrap();
            }
        }
        assert_eq!(sent.as_slice(), &[project]);
    }
    client.step(Event::Reply { exchange: opening, message: fixtures::unknown_read() });
    for request in client.requests() {
        if let Request::Exchange { message: Message::Read(_), .. } = request {
            panic!("releasing another exchange sends no read twice");
        }
    }
}
