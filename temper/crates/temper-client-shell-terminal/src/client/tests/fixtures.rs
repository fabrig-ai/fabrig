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
                    assert_eq!(link.secret(), &[0; 32], "the printed fixture link owns the exchange");
                    assert!(signing.replace(exchange).is_none(), "one sign-in exchange");
                }
                jig_client::Message::Read(_) => {
                    assert!(credential.is_none(), "no identity installed before the link succeeds");
                    assert!(activity.replace(exchange).is_none(), "one initial activity read");
                }
                // M4's Authorize place answers it
                jig_client::Message::Authorize(_) => {}
                jig_client::Message::SignInService
                | jig_client::Message::SignOut
                | jig_client::Message::Request(_)
                | jig_client::Message::Watch(_)
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
    let answer = w::SignInAnswer::SignedIn(
        w::SignedIn::new(&w::CEILINGS, w::SignedInParts { token, party: CURRENT_PARTY, expires: 0 })
            .expect("signed-in answer"),
    );
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("answer fits"));
    answer.encode(&mut bytes).expect("answer room");
    bytes.finish()
}

pub(super) fn loaded_chat() -> NativeClient {
    let mut host = common::native();
    host.address = domain::Address::Chat { task: TASK };
    host.loaded(Some(credential()), Vec::new());
    host
}

pub(super) fn activity_read(host: &mut NativeClient) -> (skein_lib::Token, Option<skein_lib::Secret>) {
    let mut found = activity_reads(host);
    assert_eq!(found.len(), 1, "the actual linked root emitted one activity exchange");
    found.pop().expect("one activity exchange")
}

pub(super) fn activity_reads(host: &mut NativeClient) -> Vec<(skein_lib::Token, Option<skein_lib::Secret>)> {
    let mut found = Vec::new();
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client {
            request: Request::Exchange { exchange, credential, message: jig_client::Message::Read(record) },
        } = request
        {
            let envelope = w::Owned::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&record)).expect("read envelope");
            let read = w::Read::decode(&w::CEILINGS, &mut skein_lib::Reader::new(envelope.record())).expect("jig read");
            assert!(matches!(read, w::Read::Activity(_)), "chat asks for activity");
            found.push((exchange, credential));
        }
    }
    found
}

pub(super) fn activity_answer(nonempty: bool) -> Box<[u8]> {
    let event = w::Activity::RunBegan(
        w::ActivityRunBegan::new(
            &w::CEILINGS,
            w::ActivityRunBeganParts { attempt: RUN, how: w::RunBeginning::Resumed, why: Box::default() },
        )
        .expect("run marker"),
    );
    let row =
        w::ActivityLine::new(&w::CEILINGS, w::ActivityLineParts { position: 1, at: 1, event }).expect("activity row");
    let row_bytes = if nonempty { usize::try_from(row.measure()).expect("row fits") } else { 0 };
    let mut rows = skein_lib::Writer::new(if nonempty { row_bytes + 4 } else { 0 });
    if nonempty {
        rows.put(&u32::try_from(row_bytes).expect("row length fits").to_be_bytes()).expect("length room");
        row.encode(&mut rows).expect("row room");
    }
    let page = w::ActivityPage::new(
        &w::CEILINGS,
        w::ActivityPageParts { cut: false, rows: rows.finish(), before: None, after: None },
    )
    .expect("quiet activity page");
    activity_page_answer(page)
}

fn activity_page_answer(page: w::ActivityPage) -> Box<[u8]> {
    let value = w::ReadValue::Activity(page);
    let mut bytes = skein_lib::Writer::new(usize::try_from(value.measure()).expect("value fits"));
    value.encode(&mut bytes).expect("value room");
    let owned = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: w::VERSION, record: bytes.finish() },
    )
    .expect("owned activity");
    let answer = w::ReadAnswer::Value(owned);
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("answer fits"));
    answer.encode(&mut bytes).expect("answer room");
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
    let mut frame = None;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client {
            request:
                Request::Down { down: Down::Open { watch, subject: Watchable::Jig(jig_client::contract::Subject::Me) } },
        } = request
        {
            frame = Some(watch);
        }
    }
    (host, frame.expect("frame watch"))
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
        if let domain::Request::Client {
            request:
                Request::Down {
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
        } else if let domain::Request::Client {
            request:
                Request::Down {
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
    let task = jig_client_world::stories::task();
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
    let value = link::translate::reads::value(owned.record(), &common::link_limits()).expect("fixture activity value");
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
    let row = jig_client_test_records::turn_line(COMMITTED, 1, 1, jig_client_test_records::FULL_TURN);
    let mut page = jig_client_test_records::activity_rows(&[row]).into_parts();
    page.cut = false;
    page.after = None;
    activity_page_answer(w::ActivityPage::new(&w::CEILINGS, page).expect("committed page"))
}

pub(super) fn busy_activity_answer() -> Box<[u8]> {
    let answer = w::ReadAnswer::None(
        w::ReadAnswerNone::new(&w::CEILINGS, w::ReadAnswerNoneParts { value: w::ReadNone::Busy })
            .expect("read refusal"),
    );
    let mut bytes = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("refusal length"));
    answer.encode(&mut bytes).expect("refusal fits");
    bytes.finish()
}

fn stream_event(event: &w::StreamEvent) -> Box<[u8]> {
    let mut bytes = skein_lib::Writer::new(usize::try_from(event.measure()).expect("stream event length"));
    event.encode(&mut bytes).expect("stream event fits");
    bytes.finish()
}

fn committed_task() -> w::Task {
    let mut task = jig_client_test_records::task_record().into_parts();
    let mut reader = skein_lib::Reader::new(task.chip.rows());
    let length = reader.u32().expect("root length");
    let mut root_reader = skein_lib::Reader::new(reader.bytes(length).expect("root bytes"));
    let w::Fact::Chip(root) = w::Fact::decode(&w::CEILINGS, &mut root_reader).expect("chip root") else {
        panic!("chip root first")
    };
    let mut root = root.into_parts();
    let mut value = root.value.into_parts();
    value.task = TASK;
    root.value = w::ChipFields::new(&w::CEILINGS, value).expect("chat chip");
    let root = w::Fact::Chip(w::ChipRow::new(&w::CEILINGS, root).expect("chat root"));
    let root = jig_client_test_records::fact_bytes(&root);
    let mut rows = skein_lib::Writer::new(root.len() + usize::try_from(reader.remaining()).expect("chip rows length"));
    rows.put(&root).expect("root fits");
    rows.put(reader.bytes(reader.remaining()).expect("remaining chip rows")).expect("remaining chip rows fit");
    task.chip = w::Chip::new(&w::CEILINGS, w::ChipParts { rows: rows.finish() }).expect("chat chip");
    task.activity = COMMITTED;
    w::Task::new(&w::CEILINGS, task).expect("chat task")
}

pub(super) fn committed_chat() -> NativeClient {
    let mut host = loaded_chat();
    let served =
        w::Served::new(&w::CEILINGS, w::ServedParts { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION })
            .expect("served jig");
    let mut versions = List::with_capacity(1);
    versions.push(served).expect("one owner");
    let hello = w::StreamEvent::Hello(
        w::Hello::new(
            &w::CEILINGS,
            w::HelloParts { stream: [7; 16], served: versions, heartbeat: host.env.limits.client.domain.heartbeat },
        )
        .expect("stream hello"),
    );
    host.transport(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_event(&hello)) });
    let mut work = Queue::with_capacity(host.out.capacity());
    let mut task_watch = None;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client {
            request: Request::Exchange { message: jig_client::Message::Watch(bytes), .. },
        } = &request
        {
            let open = w::Open::decode(&w::CEILINGS, &mut skein_lib::Reader::new(bytes)).expect("watch open");
            if open.subject().owner() == &w::Owner::Jig {
                let subject = w::Subject::decode(&w::CEILINGS, &mut skein_lib::Reader::new(open.subject().record()))
                    .expect("watch subject");
                if let w::Subject::Task(task) = subject {
                    assert_eq!(task.task(), TASK);
                    task_watch = Some(open.watch());
                }
            }
        }
        work.push(request);
    }
    host.out = work;
    let value = w::DeliveryValue::Task(Box::new(committed_task()));
    let mut bytes = skein_lib::Writer::new(usize::try_from(value.measure()).expect("task length"));
    value.encode(&mut bytes).expect("task fits");
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: w::VERSION, record: bytes.finish() },
    )
    .expect("owned task");
    let delivery = w::StreamEvent::Delivery(
        w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: task_watch.expect("chat task watch"), document })
            .expect("task delivery"),
    );
    host.transport(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_event(&delivery)) });
    host
}
