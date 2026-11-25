use super::*;
use jig_client::{Down, Message, Request, Watchable};
use jig_client_protocol_wire as w;
use skein_lib::{Reader, Token};
use temper_client_protocol_wire as wire;

pub(super) const TASK: u64 = 9;
pub(super) const RUN: u64 = 44;
// The existing CLI disk fixture's native token, never a real credential.
const TOKEN_BYTES: &[u8] = b"0123456789012345678901234567890123456789";
pub(super) fn credential() -> skein_lib::Secret {
    skein_lib::Secret::new(TOKEN_BYTES.into())
}

pub(super) const LINK: &[u8] = b"http://127.0.0.1:7100/sign-in?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

pub(super) fn link_chat(stored: Option<skein_lib::Secret>) -> (NativeClient, Token) {
    let mut host = common::native();
    host.address = domain::Address::Chat { task: TASK };
    host.sign_in(LINK).expect("fixture link to the configured engine");
    host.loaded(stored, Vec::new());
    assert!(host.with.is_none(), "the queued explicit link was consumed once");
    assert!(host.root.client().link().credential.is_none(), "stored Held must not precede the explicit Link");
    let signing = link_exchange(&mut host);
    (host, signing)
}

pub(super) fn link_exchange(host: &mut NativeClient) -> Token {
    settle(host);
    let mut signing = None;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client { request: Request::Exchange { exchange, credential, message } } = request {
            match message {
                Message::SignInLink(record) => {
                    host.authentication.insert(exchange, false).expect("bounded sign-in exchanges");
                    assert_eq!(credential, host.credential, "the link offers the directory bearer");
                    let link = w::LinkSignIn::decode(&w::CEILINGS, &mut Reader::new(&record)).expect("link exchange");
                    assert_eq!(link.secret.expose(), &[0; 32], "the printed fixture link owns the exchange");
                    assert!(signing.replace(exchange).is_none(), "one sign-in exchange");
                }
                Message::Read(_) | Message::Batch(_) => panic!("only the link exchange may leave before its answer"),
                // M4's Authorize place answers it
                Message::Authorize(_) => {}
                Message::LinkAsk(_)
                | Message::ApprovalAsk(_)
                | Message::ApprovalRead(_)
                | Message::ApprovalDecide(_)
                | Message::SignInApproval(_)
                | Message::SignInService
                | Message::SignOut
                | Message::Request(_)
                | Message::Watch(_)
                | Message::Unwatch(_)
                | Message::StreamGone(_) => panic!("unexpected sign-in exchange"),
            }
        }
    }
    signing.expect("explicit link exchanged")
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

pub(super) fn live_chat() -> NativeClient {
    let mut host = loaded_chat();
    open_chat(&mut host, 7);
    host
}

pub(super) fn open_chat(host: &mut NativeClient, stream: u8) {
    chat_stream_opened(host);
    chat_hello(host, stream);
}

pub(super) fn chat_stream_opened(host: &mut NativeClient) {
    settle(host);
    let mut opening = false;
    while let Some(request) = host.out.pop() {
        if let domain::Request::Client { request } = request {
            assert!(!matches!(request, Request::Exchange { .. }), "no page exchange before the hello");
            opening |= matches!(request, Request::OpenStream { .. });
        }
    }
    assert!(opening, "the client opened its stream");
}

pub(super) fn chat_hello(host: &mut NativeClient, stream: u8) {
    let served = List::from_boxed(Box::new([
        w::Served { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION },
        w::Served {
            owner: w::Owner::Family(w::Family { number: wire::FORGE }),
            first: wire::CLIENT_VERSION,
            last: wire::CLIENT_VERSION,
        },
    ]));
    let mut me = jig_client_test_records::frame_record();
    me.party.number = CURRENT_PARTY;
    let hello = jig_client_test_records::opened_with_layouts(
        w::Hello { stream: [stream; 16], served, heartbeat: host.env.limits.client.domain.heartbeat },
        me,
        &[jig_client_test_records::family_layout(wire::FORGE, wire::CLIENT_LAYOUT)],
    );
    host.transport(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_event(&hello)) });
    assert_eq!(host.root.client().link().state, jig_client::Link::Live);
}

#[derive(Clone, Copy)]
pub(super) struct ActivityExchange {
    pub exchange: Token,
    member: Option<u16>,
}

impl ActivityExchange {
    pub(super) fn reply(self, host: &mut NativeClient, message: Box<[u8]>) {
        if let Some(member) = self.member {
            batch_reply(host, self.exchange, member, message);
            host.transport(jig_client::Event::Done { exchange: self.exchange });
        } else {
            host.transport(jig_client::Event::Reply { exchange: self.exchange, message });
        }
    }
}

fn batch_reply(host: &mut NativeClient, exchange: Token, member: u16, message: Box<[u8]>) {
    let part = w::BatchReply { member, reply: Some(message) };
    host.transport(jig_client::Event::Part { exchange, message: part.encode(&w::CEILINGS).expect("batch reply") });
}

type ActivityReads = Vec<(ActivityExchange, Option<skein_lib::Secret>)>;

pub(super) fn activity_read(host: &mut NativeClient) -> (ActivityExchange, Option<skein_lib::Secret>) {
    one_activity(activity_reads(host))
}

fn one_activity(mut reads: ActivityReads) -> (ActivityExchange, Option<skein_lib::Secret>) {
    assert_eq!(reads.len(), 1, "the linked root emitted one activity exchange");
    reads.pop().expect("one activity exchange")
}

pub(super) fn activity_reads(host: &mut NativeClient) -> ActivityReads {
    chat_work(host).0
}

fn chat_activity(read: &w::Owned) -> bool {
    if read.owner != w::Owner::Jig {
        return false;
    }
    let read = w::Read::decode(&w::CEILINGS, &mut Reader::new(&read.record)).expect("jig read");
    if matches!(read, w::Read::Notes(_)) {
        return false;
    }
    assert!(matches!(read, w::Read::Activity(_)), "chat asks for activity");
    true
}

fn admit_chat_watch(open: &w::Open, task_watch: &mut Option<u64>) -> Box<[u8]> {
    if open.subject.owner == w::Owner::Jig {
        let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(&open.subject.record)).expect("watch subject");
        if let w::Subject::Task(task) = subject {
            assert_eq!(task.task, TASK);
            *task_watch = Some(open.watch);
        }
    }
    w::OpenAnswer::Admitted.encode(&w::CEILINGS).expect("opening answer")
}

fn unknown_read() -> Box<[u8]> {
    w::ReadAnswer::None(w::ReadAnswerNone { value: w::ReadNone::Unknown }).encode(&w::CEILINGS).expect("unrelated read")
}

fn chat_work(host: &mut NativeClient) -> (ActivityReads, Option<u64>) {
    settle(host);
    let mut reads = Vec::new();
    let mut task_watch = None;
    while let Some(request) = host.out.pop() {
        let domain::Request::Client { request: Request::Exchange { exchange, credential, message } } = request else {
            continue;
        };
        let reply = match message {
            Message::Read(bytes) => {
                let read = w::Owned::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("read envelope");
                if chat_activity(&read) {
                    reads.push((ActivityExchange { exchange, member: None }, credential));
                    continue;
                }
                unknown_read()
            }
            Message::Batch(bytes) => {
                let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("chat batch");
                let mut pending = false;
                for (place, member) in batch.members.into_boxed().into_iter().enumerate() {
                    let place = u16::try_from(place).expect("batch member index");
                    let reply = match member {
                        w::Member::Read(read) if chat_activity(&read) => {
                            assert!(!pending, "one activity read in the batch");
                            reads.push((ActivityExchange { exchange, member: Some(place) }, credential.clone()));
                            pending = true;
                            continue;
                        }
                        w::Member::Read(_) => unknown_read(),
                        w::Member::Open(open) => admit_chat_watch(&open, &mut task_watch),
                        w::Member::Close(_) => w::CloseAnswer::Closed.encode(&w::CEILINGS).expect("closing answer"),
                    };
                    batch_reply(host, exchange, place, reply);
                }
                if !pending {
                    host.transport(jig_client::Event::Done { exchange });
                }
                continue;
            }
            Message::Watch(bytes) => admit_chat_watch(
                &w::Open::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("opening"),
                &mut task_watch,
            ),
            Message::Unwatch(_) => w::CloseAnswer::Closed.encode(&w::CEILINGS).expect("closing answer"),
            Message::Request(_)
            | Message::StreamGone(_)
            | Message::Authorize(_)
            | Message::SignInLink(_)
            | Message::SignInService
            | Message::SignOut
            | Message::LinkAsk(_)
            | Message::ApprovalAsk(_)
            | Message::ApprovalRead(_)
            | Message::ApprovalDecide(_)
            | Message::SignInApproval(_) => panic!("unexpected chat exchange: {message:?}"),
        };
        host.transport(jig_client::Event::Reply { exchange, message: reply });
    }
    (reads, task_watch)
}

pub(super) fn activity_answer(nonempty: bool) -> Box<[u8]> {
    let event =
        w::Activity::RunBegan(w::ActivityRunBegan { attempt: RUN, why: w::RunWhy::Made, resumed: true, fresh: None });
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

pub(super) fn signed_link_chat() -> NativeClient {
    let (mut host, signing) = link_chat(None);
    assert!(host.root.pages().chat().expect("chat").thread.reading.is_some(), "the unsent read is pending");
    host.transport(jig_client::Event::Reply { exchange: signing, message: signed_in() });
    assert_eq!(host.root.client().link().credential, Some(&signed_credential()), "new identity installed");
    host
}

pub(super) fn recovered_link_chat() -> (NativeClient, ActivityExchange) {
    let mut host = signed_link_chat();
    open_chat(&mut host, 7);
    let (current, credential) = activity_read(&mut host);
    assert_eq!(credential, Some(signed_credential()));
    (host, current)
}

pub(super) fn lost_sign_out(host: &mut NativeClient, state: jig_client::Link) {
    assert_eq!(host.root.client().link().state, state);
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
    assert_eq!(host.root.client().link().state, state, "lost sign-out re-tells the current link state");
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

pub(super) fn committed_chat() -> (NativeClient, ActivityExchange) {
    let mut host = live_chat();
    let (reads, task_watch) = chat_work(&mut host);
    let (activity, _) = one_activity(reads);
    let value = w::DeliveryValue::Task(Box::new(committed_task()));
    let mut bytes = skein_lib::Writer::new(
        usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("task length"),
    );
    value.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("task fits");
    let document = w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: bytes.finish() };
    let delivery = w::StreamEvent::Delivery(w::Delivery { watch: task_watch.expect("chat task watch"), document });
    host.transport(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_event(&delivery)) });
    (host, activity)
}

pub(super) fn settle(host: &mut NativeClient) {
    domain::settle(&mut host.root, &host.env, &mut host.out);
}
