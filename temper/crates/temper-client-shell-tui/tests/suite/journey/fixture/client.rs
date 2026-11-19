//! Client reports used by the suite's recorded walks.
use jig_client_protocol_wire::Record as _;
use jig_client::{Message, Request};
use jig_client_protocol_wire as w;
use skein_lib::{Duration, List, Reader, Writer};
use temper_client_shell_tui::host::{Duties, Report};

pub(crate) fn stream_bytes(event: &w::StreamEvent) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(event.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("stream size"));
    event.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("stream fits");
    writer.finish()
}

pub(crate) fn hello_bytes() -> Box<[u8]> {
    let owners = temper_client_shell_tui::limits::tui().native.client.client.domain.owners;
    let mut served = List::with_capacity(u32::try_from(owners.len() + 1).expect("owner count"));
    served
        .push(
            w::Served {
                    owner: w::Owner::Jig,
                    first: jig_client_protocol_wire::VERSION,
                    last: jig_client_protocol_wire::VERSION,
                },
        )
        .expect("one owner");
    for owner in owners {
        let family = w::Family { number: owner.family };
        served
            .push(
                w::Served { owner: w::Owner::Family(family), first: owner.version, last: owner.version },
            )
            .expect("family owner");
    }
    let hello = jig_client_test_records::opened(
        w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) },
        jig_client_test_records::frame_record(),
    );
    stream_bytes(&hello)
}

pub(crate) fn me_bytes(_duties: &Duties) -> Box<[u8]> {
    let watch = 0;
    let text = |bytes: &[u8]| {
        w::DocumentText { bytes: bytes.into(), cut: false }
    };
    let me = w::Me {
            party: w::MeParty { number: 7, kind: w::PartyKind::Person, login: text(b"owner"), name: text(b"Owner") },
            projects: Box::new([]),
            needs: w::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
            needs_total: 0,
            updates: w::Count { value: 0, limited: false },
            results_seen: 0,
            cut: false,
            retention: Duration::from_secs(1000),
            deployment_role: None,
            providers: Box::new([]),
            providers_cut: false,
            key_bytes: 256,
            redirect_bytes: 256,
            words_bytes: 8192,
        };
    let value = w::DeliveryValue::Me(me);
    let mut writer = Writer::new(usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("Me size"));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("Me fits");
    let document = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() };
    let delivery =
        w::StreamEvent::Delivery(w::Delivery { watch, document });
    stream_bytes(&delivery)
}

pub(crate) fn delivery_report(watch: u64, value: &w::DeliveryValue) -> Report {
    let mut writer = Writer::new(usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("delivery size"));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("delivery fits");
    let document = w::Owned { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() };
    let event = w::StreamEvent::Delivery(
        w::Delivery { watch, document },
    );
    Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_bytes(&event)) })
}

pub(crate) fn watch(duties: &Duties, subject: &w::Subject) -> u64 {
    maybe_watch(duties, subject).unwrap_or_else(|| panic!("requested watch {subject:?}: {:?}", duties.asked))
}

fn openings(message: &Message) -> Vec<w::Open> {
    match message {
        Message::Watch(bytes) => vec![w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("watch")],
        Message::Batch(bytes) => w::Batch::decode(&w::CEILINGS, &mut Reader::new(bytes))
            .expect("batch")
            
            .members
            .into_boxed()
            .into_vec()
            .into_iter()
            .filter_map(|member| match member {
                w::Member::Open(open) => Some(open),
                w::Member::Read(_) | w::Member::Close(_) => None,
            })
            .collect(),
        Message::LinkAsk(_)
        | Message::Authorize(_)
        | Message::ApprovalAsk(_)
        | Message::ApprovalRead(_)
        | Message::ApprovalDecide(_)
        | Message::SignInApproval(_)
        | Message::Request(_)
        | Message::Read(_)
        | Message::Unwatch(_)
        | Message::StreamGone(_)
        | Message::SignInLink(_)
        | Message::SignInService
        | Message::SignOut => vec![],
    }
}

fn maybe_watch(duties: &Duties, subject: &w::Subject) -> Option<u64> {
    if *subject == w::Subject::Me {
        return Some(0);
    }
    duties.asked.iter().rev().find_map(|request| {
        let Request::Exchange { message, .. } = request else { return None };
        openings(message).into_iter().rev().find_map(|open| {
            if open.subject.owner == w::Owner::Jig
                && w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject.record.as_ref())).expect("subject")
                    == *subject
            {
                Some(open.watch)
            } else {
                None
            }
        })
    })
}

fn batch_members(bytes: &[u8]) -> Vec<(u16, Message)> {
    let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("batch");
    batch
        
        .members
        .into_boxed()
        .into_vec()
        .into_iter()
        .enumerate()
        .map(|(place, record)| {
            let mut writer = Writer::new(usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture") - 1).expect("member size"));
            let message = match record {
                w::Member::Read(record) => {
                    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("read");
                    Message::Read(writer.finish())
                }
                w::Member::Open(record) => {
                    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("open");
                    Message::Watch(writer.finish())
                }
                w::Member::Close(record) => {
                    record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("close");
                    Message::Unwatch(writer.finish())
                }
            };
            (u16::try_from(place).expect("member"), message)
        })
        .collect()
}

pub(crate) fn background_reply(request: &Request) -> Vec<Report> {
    if let Request::Exchange { exchange, message: Message::Batch(bytes), .. } = request {
        let mut reports = Vec::new();
        for (member, message) in batch_members(bytes) {
            let reply = background_single(&Request::Exchange { exchange: *exchange, credential: None, message })
                .expect("batch member reply");
            let Report::Heard(jig_client::Event::Reply { message, .. }) = reply else { panic!("member reply") };
            reports.push(ReplySlot { exchange: *exchange, member: Some(member) }.report(message));
        }
        reports.push(Report::Heard(jig_client::Event::Done { exchange: *exchange }));
        reports
    } else {
        background_single(request).into_iter().collect()
    }
}

fn background_single(request: &Request) -> Option<Report> {
    let Request::Exchange { exchange, message, .. } = request else {
        return matches!(request, Request::Save { .. } | Request::Erase { .. })
            .then_some(Report::Heard(jig_client::Event::Stored));
    };
    let read = if let Message::Read(bytes) = message
        && let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("read envelope")
        && owned.owner != w::Owner::Jig
        && let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(owned.record.as_ref())).expect("family read")
        && matches!(temper_client_protocol_wire::decode_reading(envelope.record.as_ref(), &temper_client_protocol_wire::v1::CEILINGS).as_ref().map(temper_client_protocol_wire::v1::Reading::read), Some(temper_client_protocol_wire::v1::Read::Setting(setting)) if setting.kind() == 7)
    {
        use temper_client_protocol_wire::{self as wire, v1 as f};
        let record = wire::setting_answer(
            &f::SettingAnswer::new(&f::CEILINGS, f::SettingAnswerParts { revision: None, value: None })
                .expect("stock ways"),
        );
        w::ReadAnswer::Value(
            w::Owned { owner: owned.owner.clone(), version: wire::CLIENT_VERSION, record },
        )
    } else {
        w::ReadAnswer::None(
            w::ReadAnswerNone { value: w::ReadNone::Unknown },
        )
    };
    let size = match message {
        Message::Watch(_) => w::OpenAnswer::Admitted.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
        Message::Unwatch(_) => w::CloseAnswer::Closed.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
        Message::Read(_) => read.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
        Message::Batch(_)
        | Message::Authorize(_)
        | Message::LinkAsk(_)
        | Message::ApprovalAsk(_)
        | Message::ApprovalRead(_)
        | Message::ApprovalDecide(_)
        | Message::Request(_)
        | Message::StreamGone(_)
        | Message::SignInLink(_)
        | Message::SignInApproval(_)
        | Message::SignInService
        | Message::SignOut => return None,
    };
    let mut writer = Writer::new(usize::try_from(size).expect("answer size"));
    match message {
        Message::Watch(_) => w::OpenAnswer::Admitted.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("watch admitted"),
        Message::Unwatch(_) => w::CloseAnswer::Closed.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("watch closed"),
        Message::Read(_) => read.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("read answer"),
        Message::Batch(_)
        | Message::Authorize(_)
        | Message::LinkAsk(_)
        | Message::ApprovalAsk(_)
        | Message::ApprovalRead(_)
        | Message::ApprovalDecide(_)
        | Message::Request(_)
        | Message::StreamGone(_)
        | Message::SignInLink(_)
        | Message::SignInApproval(_)
        | Message::SignInService
        | Message::SignOut => unreachable!("background answer"),
    }
    Some(Report::Heard(jig_client::Event::Reply { exchange: *exchange, message: writer.finish() }))
}

#[derive(Clone, Copy)]
struct ReplySlot {
    exchange: skein_lib::Token,
    member: Option<u16>,
}
impl ReplySlot {
    fn report(self, message: Box<[u8]>) -> Report {
        let event = if let Some(member) = self.member {
            let part =
                w::BatchReply { member, reply: Some(message) };
            let mut writer = Writer::new(usize::try_from(part.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("part size"));
            part.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("part bytes");
            jig_client::Event::Part { exchange: self.exchange, message: writer.finish() }
        } else {
            jig_client::Event::Reply { exchange: self.exchange, message }
        };
        Report::Heard(event)
    }
}

pub(crate) fn terminal_document(login: Option<(u32, bool)>) -> w::DeliveryValue {
    w::DeliveryValue::Terminal(
        w::Terminal {
                login: login.map(|(number, cancelled)| {
                    w::TerminalLogin { number, provider: 17, name: b"work".as_slice().into(), cancelled }
                }),
            },
    )
}
