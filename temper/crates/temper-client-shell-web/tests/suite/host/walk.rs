//! The test's shell: linked sign-in played as platform reports, through bytes.
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::VERSION;
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{
    Duration, List, Reader, Time, Wall, Writer,
    base64::{self, Alphabet},
};
use skein_ui::{Act, Role};
use skein_ui_web::{mapping::Primary, page::Page, transport::Carried};
use temper_client_protocol_wire as wire;
use temper_client_shell_web::{
    host::{Duties, Host, Report, TabAct},
    journey::{Facts, Recorder},
};

pub(super) const ADDRESS: &[u8] = b"http://127.0.0.1:9/authorize?state=test";
struct Walk {
    host: Host,
    page: Page<'static>,
    recorder: Recorder,
    accounts: Option<u64>,
    refuse_tab: bool,
    now: Time,
    wall: Wall,
}
impl Walk {
    fn new() -> Self {
        let facts = Facts {
            seed: 11,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            primary: Primary::Control,
            zone_minutes: 0,
            ways: None,
            forges: Vec::new(),
            path: b"/accounts/connect".as_slice().into(),
            secret: None,
            stored: Vec::new(),
        };
        let (host, page) = facts.start();
        Self {
            host,
            page,
            recorder: Recorder::new(&facts),
            accounts: None,
            refuse_tab: false,
            now: facts.now,
            wall: facts.wall,
        }
    }
    fn turn(&mut self, reports: Vec<Report>) -> Vec<Asked> {
        self.now = self.now.saturating_add(Duration::from_millis(10));
        self.wall = Wall::from_nanos(self.wall.as_nanos() + Duration::from_millis(10).as_nanos());
        self.host.clocks(self.now, self.wall);
        self.recorder.turn(self.now, self.wall);
        let mut pending: std::collections::VecDeque<_> = reports.into();
        let mut asks = Vec::new();
        while let Some(report) = pending.pop_front() {
            let duties = self.recorder.report(&mut self.host, &self.page, report);
            assert!(duties.navigate.is_none() && duties.replace.is_none(), "sign-in keeps the page");
            if self.refuse_tab && duties.tab.iter().any(|tab| matches!(tab, TabAct::Open { .. })) {
                pending.push_front(Report::TabRefused);
            }
            let Duties { asked, writes, .. } = duties;
            for ask in &asked {
                if let Asked::Exchange { id, call, .. } = ask {
                    for open in opens(&call.path, &call.body) {
                        if open.subject.owner == w::Owner::Jig
                            && w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject.record.as_ref()))
                                .expect("subject")
                                == w::Subject::Accounts
                        {
                            self.accounts = Some(open.watch);
                        }
                    }
                    if call.path.as_ref() == b"/jig/v1/watch" {
                        let answer = w::OpenAnswer::Admitted;
                        pending.push_back(Report::Heard(Heard::Answered {
                            exchange: *id,
                            status: 200,
                            body: answer.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding"),
                            headers: Vec::new(),
                        }));
                    } else if call.path.as_ref() == b"/jig/v1/batch" {
                        pending.push_back(Report::Heard(Heard::Begun { exchange: *id, status: 200 }));
                        let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("batch");
                        let mut complete = true;
                        for (place, member) in batch.members.into_boxed().into_iter().enumerate() {
                            let reply = match member {
                                w::Member::Open(_) => w::OpenAnswer::Admitted
                                    .encode(&jig_client_protocol_wire::CEILINGS)
                                    .expect("bounded fixture encoding"),
                                w::Member::Close(_) => w::CloseAnswer::Closed
                                    .encode(&jig_client_protocol_wire::CEILINGS)
                                    .expect("bounded fixture encoding"),
                                w::Member::Read(_) => {
                                    complete = false;
                                    continue; // The fixture leaves lone reads unanswered too.
                                }
                            };
                            let part =
                                w::BatchReply { member: u16::try_from(place).expect("member"), reply: Some(reply) };
                            let bytes =
                                part.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding");
                            let frame = [
                                part.measure(&jig_client_protocol_wire::CEILINGS)
                                    .expect("bounded fixture")
                                    .to_be_bytes()
                                    .as_slice(),
                                bytes.as_ref(),
                            ]
                            .concat()
                            .into_boxed_slice();
                            pending.push_back(Report::Heard(Heard::Part { exchange: *id, bytes: frame }));
                        }
                        if complete {
                            pending.push_back(Report::Heard(Heard::Done { exchange: *id }));
                        }
                    }
                }
            }
            let carried: Vec<_> = asked
                .iter()
                .filter_map(|ask| {
                    if let Asked::Save { key, .. } = ask { Some(Carried::Written { key: key.clone() }) } else { None }
                })
                .collect();
            asks.extend(asked);
            if writes != 0 || !carried.is_empty() {
                pending.push_back(Report::Carried { reports: carried, writes });
            }
        }
        self.host.draw(&mut self.page, &mut Vec::new());
        self.recorder.deadline(self.host.deadline());
        asks
    }
    fn press_heading(&mut self) {
        let heading = self
            .host
            .tree()
            .nodes()
            .iter()
            .find(|node| matches!(node.role, Role::Heading { .. }))
            .expect("a heading")
            .id;
        assert!(self.host.event_of(heading, Act::Press).is_none());
        assert!(self.turn(vec![Report::Act { node: heading, act: Act::Press }]).is_empty());
    }
    fn continue_with_chatgpt(&mut self) -> u32 {
        let button = self
            .host
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT"))
            .expect("sign-in button")
            .id;
        exchange(self.turn(vec![Report::Act { node: button, act: Act::Press }]))
    }
    fn act_on(&mut self, role: Role, name: &[u8], act: Act) -> Vec<Asked> {
        let node = self
            .host
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == role && node.name.as_deref() == Some(name))
            .expect("walk's control")
            .id;
        self.turn(vec![Report::Act { node, act }])
    }
    fn key_event(&self, value: &[u8]) -> Box<[u8]> {
        let node = self
            .host
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == Role::TextField && node.name.as_deref() == Some(b"Key (pasted once)"))
            .expect("key field")
            .id;
        temper_client_shell_web::journey::event(&self.host, node, &Act::Input { value: value.into() })
            .expect("key event")
    }
    fn streamed(&mut self, event: &w::StreamEvent) -> Vec<Asked> {
        let bytes = event.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding");
        let size = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("fixture fits"))
            .expect("base64 fits");
        let mut writer = Writer::new(usize::try_from(size).expect("fixture fits"));
        base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("measured base64");
        let mut piece = b"data: ".to_vec();
        piece.extend_from_slice(&writer.finish());
        piece.extend_from_slice(b"\n\n");
        self.turn(vec![Report::Heard(Heard::Piece { bytes: piece.into() })])
    }
    fn answered(&mut self, exchange: u32, answer: &w::KeyedAnswer) -> Vec<Asked> {
        self.turn(vec![Report::Heard(Heard::Answered {
            exchange,
            status: 200,
            body: answer.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding"),
            headers: Vec::new(),
        })])
    }
}

fn owned(record: Box<[u8]>) -> w::Owned {
    w::Owned { owner: w::Owner::Jig, version: VERSION, record }
}
fn exchange(asks: Vec<Asked>) -> u32 {
    asks.into_iter()
        .find_map(|ask| {
            let Asked::Exchange { id, call, .. } = ask else {
                return None;
            };
            (call.path.as_ref() == b"/jig/v1/request").then_some(id)
        })
        .expect("keyed request left the linked host")
}
pub(super) fn walked() -> Box<[u8]> {
    walked_with_no_event(false)
}
pub(super) fn walked_with_no_event(no_event: bool) -> Box<[u8]> {
    sign_in(no_event, false, &mut |_| {})
}
pub(super) struct Saves {
    pub recording: Box<[u8]>,
    pub first: Box<[u8]>,
    pub later: Box<[u8]>,
}
fn opens(path: &[u8], body: &[u8]) -> Vec<w::Open> {
    if path == b"/jig/v1/watch" {
        return vec![w::Open::decode(&w::CEILINGS, &mut Reader::new(body)).expect("opening")];
    }
    if path == b"/jig/v1/batch" {
        return w::Batch::decode(&w::CEILINGS, &mut Reader::new(body))
            .expect("batch")
            .members
            .into_boxed()
            .into_vec()
            .into_iter()
            .filter_map(|member| match member {
                w::Member::Open(open) => Some(open),
                w::Member::Read(_) | w::Member::Close(_) => None,
            })
            .collect();
    }
    vec![]
}
#[expect(dead_code, reason = "the implicit me watch uses name zero rather than an opening exchange")]
fn opening(path: &[u8], body: &[u8]) -> Option<w::Open> {
    opens(path, body).into_iter().find(|open| {
        open.subject.owner == w::Owner::Jig
            && w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject.record.as_ref())).expect("subject")
                == w::Subject::Me
    })
}
fn linked() -> Walk {
    let mut walk = Walk::new();
    walk.turn(vec![Report::Start]);
    walk.turn(vec![Report::Heard(Heard::StreamOpened)]);
    let mut served = List::with_capacity(2);
    for (owner, version) in
        [(w::Owner::Jig, VERSION), (w::Owner::Family(w::Family { number: wire::FORGE }), wire::CLIENT_VERSION)]
    {
        served.push(w::Served { owner, first: version, last: version }).expect("two families");
    }
    let asks = walk.streamed(&jig_client_test_records::opened_with_layouts(
        w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) },
        jig_client_test_records::frame_record(),
        &[jig_client_test_records::family_layout(wire::FORGE, wire::CLIENT_LAYOUT)],
    ));
    let _ = asks;
    let watch = 0;
    let value = me_with_openai();
    walk.streamed(&w::StreamEvent::Delivery(w::Delivery {
        watch,
        document: owned(value.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding")),
    }));
    walk
}
pub(super) fn walked_with_saves(no_event: bool) -> Saves {
    let mut snapshots = Vec::new();
    let recording = sign_in(no_event, false, &mut |walk| snapshots.push(walk.recorder.save(&walk.page)));
    let [_, first, later] = snapshots.try_into().expect("the three sign-in snapshots");
    Saves { recording, first, later }
}
pub(super) fn walked_with_refused_tab() -> Box<[u8]> {
    sign_in(false, true, &mut |_| {})
}
fn sign_in(no_event: bool, refuse_tab: bool, snapshot: &mut dyn FnMut(&Walk)) -> Box<[u8]> {
    let mut walk = linked();
    walk.refuse_tab = refuse_tab;
    if no_event {
        walk.press_heading();
    }
    snapshot(&walk);
    let signing = walk.continue_with_chatgpt();
    snapshot(&walk);
    let started = w::DocumentOutcome::AccountSignInStarted(w::OutcomeAccountSignInStarted {
        source: 43,
        address: ADDRESS.into(),
    });
    let outcome = w::Outcome::Document(w::OutcomeDocument { value: started });
    let answer = w::Final::Done(w::Done { outcome });
    let answer = w::KeyedAnswer::FinalAnswer(owned(
        answer.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding"),
    ));
    walk.answered(signing, &answer);
    snapshot(&walk);
    let watch = walk.accounts.expect("accounts watch");
    for ended in [false, true] {
        let value = accounts(ended);
        walk.streamed(&w::StreamEvent::Delivery(w::Delivery {
            watch,
            document: owned(value.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding")),
        }));
    }
    walk.recorder.end(&walk.page)
}
fn accounts(ended: bool) -> w::DeliveryValue {
    let signing = w::AccountSignIn {
        source: 43,
        provider: 17,
        holder: w::AccountHolder::Party(w::AccountHolderParty { party: 23 }),
    };
    let signing =
        if ended { Box::default() } else { piece(&signing.encode(&w::CEILINGS).expect("bounded fixture encoding")) };
    let last_end = ended.then_some({
        w::AccountLastEnd { source: 43, end: w::AccountEnd::Made(w::AccountEndMade { account: 7, declined: false }) }
    });
    w::DeliveryValue::Accounts(w::Accounts {
        lendable: false,
        usable: false,
        settled: false,
        cut: false,
        readiness_cut: false,
        lending: None,
        lent_end: None,
        program_hosts: Box::default(),
        accounts: Box::default(),
        signing,
        last_end,
        readiness: Box::default(),
    })
}
pub(super) fn departed() -> Box<[u8]> {
    let mut walk = linked();
    let asks =
        walk.turn(vec![Report::Place { path: b"/oauth/authorize?client_id=forgejo&state=test".as_slice().into() }]);
    let exchange = asks
        .into_iter()
        .find_map(|ask| {
            if let Asked::Exchange { id, call, .. } = ask {
                (call.path.as_ref() == b"/jig/v1/authorize").then_some(id)
            } else {
                None
            }
        })
        .expect("authorization exchange");
    let answer = w::AuthorizeAnswer::Go(w::AuthorizeGo {
        location: b"http://127.0.0.1:9/callback?code=test&state=test".as_slice().into(),
    });
    walk.recorder.turn(walk.now, walk.wall);
    let duties = walk.recorder.report(
        &mut walk.host,
        &walk.page,
        Report::Heard(Heard::Answered {
            exchange,
            status: 200,
            body: answer.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding"),
            headers: Vec::new(),
        }),
    );
    assert!(duties.departs(), "authorization ends the recording before drawing or more reports");
    walk.recorder.end(&walk.page)
}

fn key_ready() -> Walk {
    let mut walk = Walk::new();
    walk.turn(vec![Report::Start]);
    walk.turn(vec![Report::Heard(Heard::StreamOpened)]);
    let mut served = List::with_capacity(2);
    for (owner, version) in
        [(w::Owner::Jig, VERSION), (w::Owner::Family(w::Family { number: wire::FORGE }), wire::CLIENT_VERSION)]
    {
        served.push(w::Served { owner, first: version, last: version }).expect("two families");
    }
    let asks = walk.streamed(&jig_client_test_records::opened_with_layouts(
        w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) },
        jig_client_test_records::frame_record(),
        &[jig_client_test_records::family_layout(wire::FORGE, wire::CLIENT_LAYOUT)],
    ));
    let _ = asks;
    let watch = 0;
    let value = me_with_openai_key();
    walk.streamed(&w::StreamEvent::Delivery(w::Delivery {
        watch,
        document: owned(value.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding")),
    }));
    walk.act_on(Role::Link, b"Add key", Act::Press);
    walk
}
pub(super) fn key_event(value: &[u8]) -> Box<[u8]> {
    key_ready().key_event(value)
}
pub(super) fn keyed(value: &[u8], byte_at_a_time: bool) -> Box<[u8]> {
    let mut walk = key_ready();
    if byte_at_a_time {
        for end in 1..=value.len() {
            walk.act_on(Role::TextField, b"Key (pasted once)", Act::Input { value: value[..end].into() });
        }
    } else {
        walk.act_on(Role::TextField, b"Key (pasted once)", Act::Input { value: value.into() });
    }
    let sent = exchange(walk.act_on(Role::Button, b"Add key", Act::Press));
    let outcome = w::Outcome::Document(w::OutcomeDocument { value: w::DocumentOutcome::AccountChanged });
    let answer = w::Final::Done(w::Done { outcome });
    walk.answered(
        sent,
        &w::KeyedAnswer::FinalAnswer(owned(
            answer.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding"),
        )),
    );
    walk.recorder.end(&walk.page)
}
pub(super) fn stored_stream(record: skein_ui_web::store::Record) -> Box<[u8]> {
    let facts = Facts {
        seed: 11,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        primary: Primary::Control,
        zone_minutes: 0,
        ways: None,
        forges: Vec::new(),
        path: b"/accounts/connect".as_slice().into(),
        secret: None,
        stored: vec![record],
    };
    let (host, page) = facts.start();
    let mut walk = Walk {
        host,
        page,
        recorder: Recorder::new(&facts),
        accounts: None,
        refuse_tab: false,
        now: facts.now,
        wall: facts.wall,
    };
    walk.turn(vec![Report::Start]);
    walk.recorder.end(&walk.page)
}
fn me_with_openai_key() -> w::DeliveryValue {
    let w::DeliveryValue::Me(me) = me_with_openai() else { panic!("me fixture") };
    let mut me = me;
    let mut reader = Reader::new(&me.providers);
    reader.u32().expect("provider's length");
    let mut provider = w::ProviderOffer::decode(&w::CEILINGS, &mut reader).expect("provider");
    provider.programs_cut = true;
    let provider = provider;
    me.providers = piece(&provider.encode(&w::CEILINGS).expect("bounded fixture encoding"));
    w::DeliveryValue::Me(me)
}

fn me_with_openai() -> w::DeliveryValue {
    let mut me = jig_client_test_records::frame_record();
    let provider = w::ProviderOffer {
        provider: 17,
        name: w::DocumentText { bytes: b"openai".as_slice().into(), cut: false },
        sign_in: true,
        key: true,
        programs_cut: false,
        none: false,
        signed_in_plan: true,
        programs: List::with_capacity(0),
        manage_usage: None,
        key_bytes: 4096,
    };
    me.providers = piece(&provider.encode(&w::CEILINGS).expect("bounded fixture encoding"));
    w::DeliveryValue::Me(me)
}

fn piece(bytes: &[u8]) -> Box<[u8]> {
    let mut writer = Writer::new(bytes.len().checked_add(4).expect("fixture piece fits"));
    writer.put(&u32::try_from(bytes.len()).expect("fixture length fits").to_be_bytes()).expect("piece prefix");
    writer.put(bytes).expect("piece body");
    writer.finish()
}
