use super::*;
use jig_client::{Down, Request, Watchable};
use jig_client_protocol_wire as w;

pub(super) const TASK: u64 = 9;
pub(super) const RUN: u64 = 44;
// The existing CLI disk fixture's native token, never a real credential.
const TOKEN_BYTES: &[u8] = b"0123456789012345678901234567890123456789";
pub(super) fn credential() -> skein_lib::Secret {
    skein_lib::Secret::new(TOKEN_BYTES.into())
}

pub(super) const LINK: &[u8] =
    b"http://127.0.0.1:7100/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

pub(super) fn link_chat(stored: Option<skein_lib::Secret>) -> (NativeClient, skein_lib::Token, skein_lib::Token) {
    let mut host = common::native();
    host.address = domain::Address::Chat { task: TASK };
    host.sign_in(LINK).expect("fixture link to the configured engine");
    host.loaded(stored, Vec::new());
    settle(&mut host);
    assert!(host.with.is_none(), "the queued explicit link was consumed once");
    assert!(host.root.client().link().credential.is_none(), "stored Held must not precede the explicit Link");
    let mut signing = None;
    let mut activity = None;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client { request: Request::Exchange { exchange, credential, message } } = request {
            match message {
                jig_client::Message::SignInLink(record) => {
                    host.authentication.insert(exchange, false).expect("bounded sign-in exchanges");
                    assert_eq!(credential, host.credential, "the link offers the directory bearer");
                    let link = w::LinkSignIn::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record))
                        .expect("actual link exchange");
                    assert_eq!(link.secret.expose(), &[0; 32], "the printed fixture link owns the exchange");
                    assert!(signing.replace(exchange).is_none(), "one sign-in exchange");
                }
                jig_client::Message::Read(_) => {
                    assert!(credential.is_none(), "no identity installed before the link succeeds");
                    assert!(activity.replace(exchange).is_none(), "one initial activity read");
                }
                // M4's Authorize place answers it
                jig_client::Message::Authorize(_) => {}
                jig_client::Message::LinkAsk(_)
                | jig_client::Message::ApprovalAsk(_)
                | jig_client::Message::ApprovalRead(_)
                | jig_client::Message::ApprovalDecide(_)
                | jig_client::Message::SignInApproval(_)
                | jig_client::Message::SignInService
                | jig_client::Message::SignOut
                | jig_client::Message::Request(_)
                | jig_client::Message::Watch(_)
                | jig_client::Message::Batch(_)
                | jig_client::Message::Unwatch(_)
                | jig_client::Message::StreamGone(_) => panic!("unexpected startup exchange"),
            }
        }
    }
    (host, signing.expect("explicit link exchanged"), activity.expect("initial page read"))
}

pub(super) fn signed_credential() -> skein_lib::Secret {
    // The pinned jig sign-in fixtures' NEW_CREDENTIAL.
    skein_lib::Secret::new(Box::new([9; 40]))
}

pub(super) fn signed_in() -> Box<[u8]> {
    sign_in_answer(Some(signed_credential()))
}

pub(super) fn sign_in_answer(token: Option<skein_lib::Secret>) -> Box<[u8]> {
    let answer = w::SignInAnswer::SignedIn(w::SignedIn { token, party: CURRENT_PARTY, expires: 0 });
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("answer fits"),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("answer room");
    bytes.finish()
}

pub(super) fn loaded_chat() -> NativeClient {
    let mut host = common::native();
    host.address = domain::Address::Chat { task: TASK };
    host.loaded(Some(credential()), Vec::new());
    settle(&mut host);
    host
}

pub(super) fn activity_read(host: &mut NativeClient) -> (skein_lib::Token, Option<skein_lib::Secret>) {
    let mut found = activity_reads(host);
    assert_eq!(found.len(), 1, "the actual linked root emitted one activity exchange");
    found.pop().expect("one activity exchange")
}

pub(super) fn activity_reads(host: &mut NativeClient) -> Vec<(skein_lib::Token, Option<skein_lib::Secret>)> {
    settle(host);
    let mut found = Vec::new();
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client {
            request: Request::Exchange { exchange, credential, message: jig_client::Message::Read(record) },
        } = request
        {
            let envelope = w::Owned::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record)).expect("read envelope");
            let read =
                w::Read::decode(&w::CEILINGS, &mut skein_lib::Reader::new(envelope.record.as_ref())).expect("jig read");
            if matches!(read, w::Read::Notes(_)) {
                continue;
            }
            assert!(matches!(read, w::Read::Activity(_)), "chat asks for activity");
            found.push((exchange, credential));
        }
    }
    found
}

pub(super) fn activity_answer(nonempty: bool) -> Box<[u8]> {
    let event = w::Activity::RunBegan(w::ActivityRunBegan { attempt: RUN, why: w::RunWhy::Made, resumed: true });
    let source = w::ActivityLine { task: 9, position: 1, at: 1, line: event };
    let row = w::DecodedActivityLine { source, decoded: None };
    let row_bytes = if nonempty {
        usize::try_from(row.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("row fits")
    } else {
        0
    };
    let mut rows = skein_lib::Writer::new(if nonempty { row_bytes + 4 } else { 0 });
    if nonempty {
        rows.put(&u32::try_from(row_bytes).expect("row length fits").to_be_bytes()).expect("length room");
        row.write(&jig_client_protocol_wire::CEILINGS, &mut rows).expect("row room");
    }
    let page = w::ActivityPage { cut: false, rows: rows.finish(), before: None, after: None };
    activity_page_answer(page)
}

fn activity_page_answer(page: w::ActivityPage) -> Box<[u8]> {
    let value = w::ReadValue::Activity(page);
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("value fits"),
    );
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("value room");
    let owned = w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: bytes.finish() };
    let answer = w::ReadAnswer::Value(owned);
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("answer fits"),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("answer room");
    bytes.finish()
}

pub(super) fn started_kept() -> (NativeClient, skein_lib::Token) {
    let mut host = common::native();
    host.root = domain::unlinked(&host.env.limits, 7);
    let record = common::table_record();
    host.kept_keys.insert(record.request.key).expect("one key");
    let mut kept = List::with_capacity(1);
    kept.push(record).expect("one record");
    host.event(domain::Event::Start { address: host.address.clone(), kept: List::with_capacity(0), with: None });
    host.event(domain::Event::Client { event: jig_client::Event::StartUnlinked { kept } });
    host.event(domain::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
    host.event(domain::Event::Client {
        event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    });
    while host.out.pop().is_some() {}
    (host, skein_lib::Token::new(0))
}

pub(super) fn recovered_link_chat() -> (NativeClient, skein_lib::Token) {
    let (mut host, signing, old) = link_chat(None);
    host.transport(jig_client::Event::Reply { exchange: signing, message: signed_in() });
    let (current, credential) = activity_read(&mut host);
    assert_ne!(current, old, "sign-in replaces the cancelled first read");
    assert_eq!(credential, Some(signed_credential()));
    (host, current)
}

pub(super) fn lost_sign_out(host: &mut NativeClient) {
    assert_eq!(host.root.client().link().state, jig_client::Link::Starting);
    assert!(!host.root.client().link().sign_out_lost);
    host.event(domain::Event::Act { action: domain::Action::Pages(domain::pages::Act::SignOut) });
    settle(host);
    let mut signing = None;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client {
            request: Request::Exchange { exchange, credential, message: jig_client::Message::SignOut },
        } = &request
        {
            assert_eq!(credential.as_ref(), host.root.client().link().credential);
            assert!(signing.replace(*exchange).is_none(), "one sign-out exchange");
        } else {
            panic!("sign-out must only ask to sign out: {request:?}");
        }
    }
    host.transport(jig_client::Event::Lost { exchange: signing.expect("linked sign-out exchange") });
    assert_eq!(host.root.client().link().state, jig_client::Link::Starting, "lost sign-out re-tells Starting");
    assert!(host.root.client().link().sign_out_lost, "the real failed sign-out reached the linked root");
}

pub(super) fn unlinked_page(address: domain::Address) -> (NativeClient, skein_lib::Token) {
    let mut host = common::native();
    host.root = domain::unlinked(&host.env.limits, 7);
    host.event(domain::Event::Start { address, kept: List::with_capacity(0), with: None });
    host.event(domain::Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } });
    host.event(domain::Event::Client {
        event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    });
    let (mut reads, _) = page_work(&mut host);
    assert_eq!(reads.len(), 1, "one old first activity read");
    (host, reads.pop().expect("old read"))
}

pub(super) fn recovered_unlinked_page(
    address: domain::Address,
) -> (NativeClient, skein_lib::Token, skein_lib::Token, skein_lib::Token) {
    let (mut host, old) = unlinked_page(address);
    host.event(domain::Event::Client { event: jig_client::Event::SignIn { with: jig_client::With::Transport } });
    assert_eq!(host.root.client().link().state, jig_client::Link::Starting);
    let (mut reads, mut watches) = page_work(&mut host);
    assert_eq!(reads.len(), 1, "successful Transport sign-in must replace the cancelled first activity read");
    let current = reads.pop().expect("fresh read");
    assert_ne!(old, current, "fresh read has a new token");
    assert_eq!(watches.len(), 1, "the task watch reopens at sign-in");
    (host, old, current, watches.pop().expect("task watch"))
}

pub(super) fn page_work(host: &mut NativeClient) -> (Vec<skein_lib::Token>, Vec<skein_lib::Token>) {
    let mut reads = Vec::new();
    let mut watches = Vec::new();
    while let Some(request) = host.out.pop() {
        if let domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        Down::Read {
                            read,
                            what: jig_client::Readable::Jig(jig_client::contract::Read::Activity { task, before }),
                        },
                },
        } = request
        {
            assert_eq!(task, TASK);
            assert!(before.is_none(), "first page reads the latest activity");
            reads.push(read);
        } else if let domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: Watchable::Jig(jig_client::contract::Subject::Task { task }) },
                },
        } = request
        {
            assert_eq!(task, TASK);
            watches.push(watch);
        }
    }
    (reads, watches)
}

pub(super) fn quiet_task(host: &mut NativeClient, watch: skein_lib::Token) {
    let task = jig_client_kit::stories::task();
    assert_eq!(task.chip.task, TASK);
    assert_eq!(task.activity, 0);
    assert!(task.priority.is_some(), "the Task address becomes Tracked");
    host.event(domain::Event::Client {
        event: jig_client::Event::Up {
            up: jig_client::Up::Watched {
                watch,
                event: jig_client::contract::WatchEvent::Delivered(jig_client::contract::Delivery::Task(Box::new(
                    task,
                ))),
            },
        },
    });
}

pub(super) fn activity_done(host: &mut NativeClient, read: skein_lib::Token, nonempty: bool) {
    let bytes = activity_answer(nonempty);
    let w::ReadAnswer::Value(owned) =
        w::ReadAnswer::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("fixture read answer")
    else {
        panic!("fixture contains an activity page")
    };
    let value =
        link::translate::reads::value(owned.record.as_ref(), &common::link_limits()).expect("fixture activity value");
    host.event(domain::Event::Client {
        event: jig_client::Event::Up {
            up: jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(jig_client::contract::ReadResult::Value(value)),
            },
        },
    });
}

// The delivery announces the fourth activity while the initial read is pending.
pub(super) const COMMITTED: u64 = 4;
pub(super) fn committed_activity_answer() -> Box<[u8]> {
    let row = jig_client_test_records::turn_line(COMMITTED, 1, 1, &jig_client_test_records::full_turn());
    let mut page = jig_client_test_records::activity_rows(&[row]);
    page.cut = false;
    page.after = None;
    activity_page_answer(page)
}

pub(super) fn busy_activity_answer() -> Box<[u8]> {
    let answer = w::ReadAnswer::None(w::ReadAnswerNone { value: w::ReadNone::Busy });
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("refusal length"),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("refusal fits");
    bytes.finish()
}

fn stream_event(event: &w::StreamEvent) -> Box<[u8]> {
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(event.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("stream event length"),
    );
    event.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("stream event fits");
    bytes.finish()
}

fn committed_task() -> w::Task {
    let mut task = jig_client_test_records::task_record();
    let mut reader = skein_lib::Reader::new(task.chip.rows.as_ref());
    let length = reader.u32().expect("root length");
    let mut root_reader = skein_lib::Reader::new(reader.bytes(length).expect("root bytes"));
    let w::Fact::Chip(root) = w::Fact::decode(&w::CEILINGS, &mut root_reader).expect("chip root") else {
        panic!("chip root first")
    };
    let mut root = root;
    let mut value = root.value;
    value.task = TASK;
    root.value = value;
    let root = w::Fact::Chip(root);
    let root = jig_client_test_records::fact_bytes(&root);
    let mut rows = skein_lib::Writer::new(root.len() + usize::try_from(reader.remaining()).expect("chip rows length"));
    rows.put(&root).expect("root fits");
    rows.put(reader.bytes(reader.remaining()).expect("remaining chip rows")).expect("remaining chip rows fit");
    task.chip = w::Chip { rows: rows.finish() };
    task.activity = COMMITTED;
    task
}

pub(super) fn committed_chat() -> NativeClient {
    let mut host = loaded_chat();
    let served = w::Served { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION };
    let mut versions = List::with_capacity(1);
    versions.push(served).expect("one owner");
    let hello = jig_client_test_records::opened(
        w::Hello { stream: [7; 16], served: versions, heartbeat: host.env.limits.client.domain.heartbeat },
        jig_client_test_records::frame_record(),
    );
    host.transport(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_event(&hello)) });
    settle(&mut host);
    let mut work = Queue::with_capacity(host.out.capacity());
    let mut task_watch = None;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client { request: Request::Exchange { exchange, message, .. } } = &request {
            let mut opens = skein_lib::List::with_capacity(64);
            if let jig_client::Message::Watch(bytes) = message {
                opens
                    .push(w::Open::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("opening"))
                    .expect("one opening");
            } else if let jig_client::Message::Batch(bytes) = message {
                for member in w::Batch::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes))
                    .expect("batch")
                    .members
                    .into_boxed()
                {
                    if let w::Member::Open(open) = member {
                        opens.push(open).expect("bounded openings");
                    }
                }
            }
            if opens.is_empty() {
                work.push(request);
                continue;
            }
            for open in &opens {
                if open.subject.owner == w::Owner::Jig {
                    let subject =
                        w::Subject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(open.subject.record.as_ref()))
                            .expect("watch subject");
                    if let w::Subject::Task(task) = subject {
                        assert_eq!(task.task, TASK);
                        task_watch = Some(open.watch);
                    }
                }
            }
            let answer = w::OpenAnswer::Admitted;
            let mut body = skein_lib::Writer::new(
                usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                    .expect("answer length"),
            );
            answer.write(&jig_client_protocol_wire::CEILINGS, &mut body).expect("opening answer");
            let body = body.finish();
            if let jig_client::Message::Batch(bytes) = message {
                let batch = w::Batch::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("batch");
                for (index, member) in (&batch.members).into_iter().enumerate() {
                    if let w::Member::Open(_) = member {
                        let part = w::BatchReply {
                            member: u16::try_from(index).expect("batch index"),
                            reply: Some(body.clone()),
                        };
                        let mut bytes = skein_lib::Writer::new(
                            usize::try_from(
                                part.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
                            )
                            .expect("part size"),
                        );
                        part.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("opening part fits");
                        host.transport(jig_client::Event::Part { exchange: *exchange, message: bytes.finish() });
                    }
                }
                if batch.members.len() == opens.len() {
                    host.transport(jig_client::Event::Done { exchange: *exchange });
                }
            } else {
                host.transport(jig_client::Event::Reply { exchange: *exchange, message: body });
            }
        } else {
            work.push(request);
        }
    }
    host.out = work;
    let value = w::DeliveryValue::Task(Box::new(committed_task()));
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("task length"),
    );
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("task fits");
    let document = w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: bytes.finish() };
    let delivery = w::StreamEvent::Delivery(w::Delivery { watch: task_watch.expect("chat task watch"), document });
    host.transport(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_event(&delivery)) });
    host
}

pub(super) fn settle(host: &mut NativeClient) {
    domain::settle(&mut host.root, &host.env, &mut host.out);
}
