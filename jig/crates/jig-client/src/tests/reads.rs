use super::fixtures::{self, Client, SUBJECT_TASK, TASK};
use crate::{Event, Handle, Message, Readable, Request, With, contract as c};
use jig_client_protocol_wire as w;
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
fn a_batched_busy_read_ends_while_its_busy_opening_retries_after_backoff() {
    let mut client = client();
    let mut out = skein_lib::Queue::uncapped();
    for event in [
        Event::Read { handle: TASK, read: Readable::Jig(c::Read::Project { project: 9 }) },
        Event::Watch { handle: Handle { number: 200, ..TASK }, subject: fixtures::task() },
    ] {
        client.machine.step(Time::ZERO, skein_lib::Wall::EPOCH, event, &mut out);
    }
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut out);
    let Some(Request::Exchange { exchange, message: Message::Batch(bytes), .. }) = out.pop() else {
        panic!("read and opening batch")
    };
    let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    let [w::Member::Read(_), w::Member::Open(open)] = batch.members().as_slice() else {
        panic!("ordered read and opening")
    };
    let original = open.watch();
    let busy = w::OpenAnswer::Refused(
        w::OpenAnswerRefused::new(&w::CEILINGS, w::OpenAnswerRefusedParts { why: w::WatchRefusal::Busy }).unwrap(),
    );
    let mut writer = Writer::new(usize::try_from(busy.measure()).unwrap());
    busy.encode(&mut writer).unwrap();
    for (member, reply) in [(0, fixtures::read_none(w::ReadNone::Busy)), (1, writer.finish())] {
        let part = w::BatchReply::new(&w::CEILINGS, w::BatchReplyParts { member, reply: Some(reply) }).unwrap();
        let mut writer = Writer::new(usize::try_from(part.measure()).unwrap());
        part.encode(&mut writer).unwrap();
        client.machine.step(
            Time::ZERO,
            skein_lib::Wall::EPOCH,
            Event::Part { exchange, message: writer.finish() },
            &mut out,
        );
    }
    client.machine.step(Time::ZERO, skein_lib::Wall::EPOCH, Event::Done { exchange }, &mut out);
    let mut pages = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Told { told: crate::Told::Page { handle, page } } = request {
            assert_eq!(handle, TASK);
            assert_eq!(page, crate::Readout::Jig(c::ReadResult::None(c::ReadNone::Busy)));
            pages += 1;
        }
    }
    assert_eq!(pages, 1);
    assert_eq!(client.machine.document(Handle { number: 200, ..TASK }).unwrap().state, crate::State::Behind);
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut out);
    assert!(out.is_empty(), "the busy opening waits for its backoff");
    let due = client.machine.next_deadline().unwrap();
    client.machine.fire(due, skein_lib::Wall::EPOCH, &mut out);
    client.machine.settle(due, skein_lib::Wall::EPOCH, &mut out);
    let Some(Request::Exchange { message: Message::Watch(bytes), .. }) = out.pop() else { panic!("opening retried") };
    let retried = w::Open::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    assert_ne!(retried.watch(), original);
}

#[test]
fn parts_and_done_without_a_batch_leave_an_ordinary_exchange_untouched() {
    let mut client = client();
    let occupied = client.read(Handle { number: 100, ..TASK });
    client.step(Event::Read { handle: TASK, read: Readable::Jig(c::Read::Project { project: 9 }) });
    assert_eq!(client.requests(), []);
    for exchange in [occupied, skein_lib::Token::new(u64::MAX)] {
        client.step(Event::Part { exchange, message: fixtures::unknown_read() });
        assert!(client.requests().is_empty(), "a part answers no ordinary read");
        client.step(Event::Done { exchange });
        assert!(client.requests().is_empty(), "done releases no ordinary exchange");
        assert_eq!(client.machine.admission().deferred_reads, 0, "held by the link, not deferred before it");
    }
    client.step(Event::Reply { exchange: occupied, message: fixtures::unknown_read() });
    assert_eq!(client.machine.admission().deferred_reads, 0);
    assert_ne!(client.exchange(), occupied, "the ordinary reply still releases the deferred read");
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
    assert_eq!(admission.deferred_reads, 0, "the link holds the queued read");
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
#[expect(
    clippy::too_many_lines,
    reason = "one credential boundary retains read terminals, watch terminals and held HTTP credit"
)]
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
        } else {
            let answer = w::OpenAnswer::Admitted;
            let mut bytes = Writer::new(usize::try_from(answer.measure()).unwrap());
            answer.encode(&mut bytes).unwrap();
            client.step(Event::Reply { exchange: client.exchange(), message: bytes.finish() });
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
    let mut closing = None;
    let mut opening = None;
    for (index, request) in client.requests().iter().enumerate() {
        if let Request::CloseStream = request {
            assert!(closing.replace(index).is_none());
        }
        if let Request::OpenStream { .. } = request {
            assert!(opening.replace(index).is_none());
        }
    }
    let opening = opening.expect("new stream");
    assert!(closing.expect("the old stream closes") < opening, "the old stream closes before its replacement opens");
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
    client.step(Event::Reply { exchange: first, message: fixtures::unknown_read() });
    let mut bytes = None;
    for request in client.requests() {
        if let Request::Exchange { message: Message::Batch(body), .. } = request {
            bytes = Some(body);
        }
    }
    let bytes = bytes.expect("two waiting reads share the released slot");
    let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(bytes)).unwrap();
    let mut sent = List::with_capacity(2);
    for member in batch.members() {
        let w::Member::Read(owned) = member else { panic!("queued reads") };
        let w::Read::Project(read) = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).unwrap() else {
            panic!("project read")
        };
        sent.push(read.project()).unwrap();
    }
    assert_eq!(sent.as_slice(), &[30, 31]);
    client.step(Event::Reply { exchange: second, message: fixtures::unknown_read() });
    for request in client.requests() {
        assert_no_exchange(request);
    }
    client.step(Event::Reply { exchange: opening, message: fixtures::unknown_read() });
    for request in client.requests() {
        if let Request::Exchange { message: Message::Read(_), .. } = request {
            panic!("releasing another exchange sends no read twice");
        }
    }
}

#[test]
fn reads_leave_in_asked_order_only_at_the_iterations_settle() {
    let mut client = Client::signed_in();
    for (number, project) in [(40, 2), (41, 1)] {
        client.machine.step(
            Time::ZERO,
            skein_lib::Wall::EPOCH,
            Event::Read { handle: Handle { number, ..TASK }, read: Readable::Jig(c::Read::Project { project }) },
            &mut client.out,
        );
    }
    assert!(client.out.is_empty(), "steps gather reads without handing out exchanges");
    client.machine.fire(Time::ZERO, skein_lib::Wall::EPOCH, &mut client.out);
    assert!(client.out.is_empty(), "fire does not hand out gathered work");
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut client.out);
    let Some(Request::Exchange { credential: Some(credential), message: Message::Batch(bytes), .. }) = client.out.pop()
    else {
        panic!("one batch for gathered reads")
    };
    assert_eq!(credential.expose(), &[7; 40]);
    assert!(client.out.is_empty());
    let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
    let mut projects = List::with_capacity(2);
    for member in batch.members() {
        let w::Member::Read(owned) = member else { panic!("read member") };
        let w::Read::Project(project) = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).unwrap() else {
            panic!("project read")
        };
        projects.push(project.project()).unwrap();
    }
    assert_eq!(projects.as_slice(), [2, 1]);
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut client.out);
    assert!(client.out.is_empty(), "a second settle sends nothing twice");
}

#[test]
fn a_keyed_sends_save_leaves_at_step_and_exchange_at_settle_after_storage() {
    let mut client = Client::signed_in();
    client.machine.step(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Send { key: fixtures::STOP, name: None, request: fixtures::stop() },
        &mut client.out,
    );
    let mut saved = false;
    for request in &client.out {
        if let Request::Save { .. } = request {
            saved = true;
        }
        assert_no_exchange(request);
    }
    assert!(saved, "the keyed request saves at step");
    client.out.clear();
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut client.out);
    assert!(client.out.is_empty(), "the record must be stored first");
    client.machine.step(Time::ZERO, skein_lib::Wall::EPOCH, Event::Stored, &mut client.out);
    for request in &client.out {
        assert_no_exchange(request);
    }
    client.out.clear();
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut client.out);
    let Some(Request::Exchange { message: Message::Request(_), .. }) = client.out.pop() else {
        panic!("a stored keyed request leaves at settle");
    };
    assert!(client.out.is_empty());
}

#[test]
fn an_unlinked_clients_settle_is_empty() {
    let mut client = crate::Client::unlinked(&fixtures::limits(), 7).expect("fixture record");
    let mut unlinked = skein_lib::Queue::uncapped();
    client.step_unlinked(Time::ZERO, skein_lib::Wall::EPOCH, Event::SignIn { with: With::Transport }, &mut unlinked);
    let mut out = skein_lib::Queue::uncapped();
    client.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut out);
    assert!(out.is_empty());
}

#[test]
fn a_full_links_read_waits_for_the_first_settle_after_a_terminal() {
    let mut client = client();
    let occupied = client.read(Handle { number: 100, ..TASK });
    client.machine.step(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Read { handle: TASK, read: Readable::Jig(c::Read::Project { project: 2 }) },
        &mut client.out,
    );
    assert!(client.out.is_empty());
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut client.out);
    assert!(client.out.is_empty(), "full link retains the demand");
    client.machine.step(
        Time::ZERO,
        skein_lib::Wall::EPOCH,
        Event::Reply { exchange: occupied, message: fixtures::unknown_read() },
        &mut client.out,
    );
    for request in &client.out {
        assert_no_exchange(request);
    }
    client.out.clear();
    client.machine.settle(Time::ZERO, skein_lib::Wall::EPOCH, &mut client.out);
    let Some(Request::Exchange { message: Message::Read(bytes), .. }) = client.out.pop() else {
        panic!("the waiting read leaves at this settle");
    };
    let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("fixture record");
    let w::Read::Project(project) =
        w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).expect("fixture record")
    else {
        panic!("project read");
    };
    assert_eq!(project.project(), 2);
    assert!(client.out.is_empty());
}

fn assert_no_exchange(request: &Request) {
    if let Request::Exchange { .. } | Request::OpenStream { .. } | Request::CloseStream = request {
        panic!("transport work leaves only at settle: {request:?}");
    }
}
