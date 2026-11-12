//! The test's shell: linked sign-in played as platform reports, through bytes.
use jig_client_protocol_wire::{VERSION, v19 as w};
use jig_client_shell_web::adapter::{Asked, Heard};
use skein_lib::{
    Duration, List, Overflow, Reader, Time, Wall, Writer,
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
                if let Asked::Exchange { id, call, .. } = ask
                    && call.path.as_ref() == b"/jig/v1/watch"
                {
                    let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("watch envelope");
                    if *open.subject().owner() == w::Owner::Jig {
                        let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record()))
                            .expect("subject");
                        if subject == w::Subject::Accounts {
                            self.accounts = Some(open.watch());
                        }
                    }
                    let answer = w::OpenAnswer::Admitted;
                    pending.push_back(Report::Heard(Heard::Answered {
                        exchange: *id,
                        status: 200,
                        body: encoded(answer.measure(), |writer| answer.encode(writer)),
                        headers: Vec::new(),
                    }));
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
        let bytes = encoded(event.measure(), |writer| event.encode(writer));
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
            body: encoded(answer.measure(), |writer| answer.encode(writer)),
            headers: Vec::new(),
        })])
    }
}
fn encoded(size: u32, write: impl FnOnce(&mut Writer) -> Result<(), Overflow>) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(size).expect("fixture fits"));
    write(&mut writer).expect("measured fixture");
    writer.finish()
}
fn owned(record: Box<[u8]>) -> w::Owned {
    w::Owned::new(&w::CEILINGS, w::OwnedParts { owner: w::Owner::Jig, version: VERSION, record })
        .expect("fixture envelope")
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
    walked_with_saves(no_event).recording
}
pub(super) struct Saves {
    pub recording: Box<[u8]>,
    pub before_press: Box<[u8]>,
    pub first: Box<[u8]>,
    pub later: Box<[u8]>,
}
fn linked() -> Walk {
    let mut walk = Walk::new();
    walk.turn(vec![Report::Start]);
    walk.turn(vec![Report::Heard(Heard::StreamOpened)]);
    let mut served = List::with_capacity(2);
    for (owner, version) in [
        (w::Owner::Jig, VERSION),
        (
            w::Owner::Family(
                w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).expect("forge family"),
            ),
            wire::CLIENT_VERSION,
        ),
    ] {
        served
            .push(
                w::Served::new(&w::CEILINGS, w::ServedParts { owner, first: version, last: version })
                    .expect("served family"),
            )
            .expect("two families");
    }
    let asks = walk.streamed(&w::StreamEvent::Hello(
        w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello"),
    ));
    let watch = asks
        .into_iter()
        .find_map(|ask| {
            let Asked::Exchange { call, .. } = ask else {
                return None;
            };
            if call.path.as_ref() != b"/jig/v1/watch" {
                return None;
            }
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("watch envelope");
            let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
            matches!(subject, w::Subject::Me).then_some(open.watch())
        })
        .expect("me watch");
    let value = me_with_openai();
    walk.streamed(&w::StreamEvent::Delivery(
        w::Delivery::new(
            &w::CEILINGS,
            w::DeliveryParts { watch, document: owned(encoded(value.measure(), |writer| value.encode(writer))) },
        )
        .expect("me delivery"),
    ));
    walk
}
pub(super) fn walked_with_saves(no_event: bool) -> Saves {
    sign_in(no_event, false)
}
pub(super) fn walked_with_refused_tab() -> Box<[u8]> {
    sign_in(false, true).recording
}
fn sign_in(no_event: bool, refuse_tab: bool) -> Saves {
    let mut walk = linked();
    walk.refuse_tab = refuse_tab;
    if no_event {
        walk.press_heading();
    }
    let before_press = walk.recorder.save(&walk.page);
    let signing = walk.continue_with_chatgpt();
    let first = walk.recorder.save(&walk.page);
    let started = w::DocumentOutcome::AccountSignInStarted(
        w::OutcomeAccountSignInStarted::new(
            &w::CEILINGS,
            w::OutcomeAccountSignInStartedParts { source: 43, address: ADDRESS.into() },
        )
        .expect("sign-in started"),
    );
    let outcome = w::Outcome::Document(
        w::OutcomeDocument::new(&w::CEILINGS, w::OutcomeDocumentParts { value: started }).expect("owner outcome"),
    );
    let answer = w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).expect("done"));
    let answer = w::KeyedAnswer::FinalAnswer(owned(encoded(answer.measure(), |writer| answer.encode(writer))));
    walk.answered(signing, &answer);
    let later = walk.recorder.save(&walk.page);
    let watch = walk.accounts.expect("accounts watch");
    for ended in [false, true] {
        let value = accounts(ended);
        walk.streamed(&w::StreamEvent::Delivery(
            w::Delivery::new(
                &w::CEILINGS,
                w::DeliveryParts { watch, document: owned(encoded(value.measure(), |writer| value.encode(writer))) },
            )
            .expect("accounts delivery"),
        ));
    }
    Saves { recording: walk.recorder.save(&walk.page), before_press, first, later }
}
fn accounts(ended: bool) -> w::DeliveryValue {
    let signing = w::AccountSignIn::new(
        &w::CEILINGS,
        w::AccountSignInParts {
            source: 43,
            provider: 17,
            holder: w::AccountHolder::Party(
                w::AccountHolderParty::new(&w::CEILINGS, w::AccountHolderPartyParts { party: 23 }).expect("holder"),
            ),
        },
    )
    .expect("signing");
    let signing = if ended {
        Box::default()
    } else {
        encoded(signing.measure() + 4, |writer| {
            writer.put(&signing.measure().to_be_bytes())?;
            signing.encode(writer)
        })
    };
    let last_end = ended.then(|| {
        w::AccountLastEnd::new(
            &w::CEILINGS,
            w::AccountLastEndParts {
                source: 43,
                end: w::AccountEnd::Made(
                    w::AccountEndMade::new(&w::CEILINGS, w::AccountEndMadeParts { account: 7, declined: false })
                        .expect("made"),
                ),
            },
        )
        .expect("sign-in end")
    });
    w::DeliveryValue::Accounts(
        w::Accounts::new(
            &w::CEILINGS,
            w::AccountsParts {
                skein_bools: [false; 5],
                lending: None,
                lent_end: None,
                program_hosts: Box::default(),
                accounts: Box::default(),
                signing,
                last_end,
                readiness: Box::default(),
            },
        )
        .expect("accounts"),
    )
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
    let answer = w::AuthorizeAnswer::Go(
        w::AuthorizeGo::new(
            &w::CEILINGS,
            w::AuthorizeGoParts { location: b"http://127.0.0.1:9/callback?code=test&state=test".as_slice().into() },
        )
        .expect("authorization location"),
    );
    walk.recorder.turn(walk.now, walk.wall);
    let duties = walk.recorder.report(
        &mut walk.host,
        &walk.page,
        Report::Heard(Heard::Answered {
            exchange,
            status: 200,
            body: encoded(answer.measure(), |writer| answer.encode(writer)),
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
    for (owner, version) in [
        (w::Owner::Jig, VERSION),
        (
            w::Owner::Family(
                w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).expect("forge family"),
            ),
            wire::CLIENT_VERSION,
        ),
    ] {
        served
            .push(
                w::Served::new(&w::CEILINGS, w::ServedParts { owner, first: version, last: version })
                    .expect("served family"),
            )
            .expect("two families");
    }
    let asks = walk.streamed(&w::StreamEvent::Hello(
        w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello"),
    ));
    let watch = asks
        .into_iter()
        .find_map(|ask| {
            let Asked::Exchange { call, .. } = ask else {
                return None;
            };
            if call.path.as_ref() != b"/jig/v1/watch" {
                return None;
            }
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("watch envelope");
            let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
            matches!(subject, w::Subject::Me).then_some(open.watch())
        })
        .expect("me watch");
    let value = me_with_openai_key();
    walk.streamed(&w::StreamEvent::Delivery(
        w::Delivery::new(
            &w::CEILINGS,
            w::DeliveryParts { watch, document: owned(encoded(value.measure(), |writer| value.encode(writer))) },
        )
        .expect("me delivery"),
    ));
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
    let outcome = w::Outcome::Document(
        w::OutcomeDocument::new(&w::CEILINGS, w::OutcomeDocumentParts { value: w::DocumentOutcome::AccountChanged })
            .expect("account outcome"),
    );
    let answer = w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).expect("done"));
    walk.answered(sent, &w::KeyedAnswer::FinalAnswer(owned(encoded(answer.measure(), |writer| answer.encode(writer)))));
    walk.recorder.save(&walk.page)
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
    walk.recorder.save(&walk.page)
}
fn me_with_openai_key() -> w::DeliveryValue {
    let w::DeliveryValue::Me(me) = me_with_openai() else { panic!("me fixture") };
    let mut me = me.into_parts();
    let mut reader = Reader::new(&me.providers);
    reader.u32().expect("provider's length");
    let mut provider = w::ProviderOffer::decode(&w::CEILINGS, &mut reader).expect("provider").into_parts();
    provider.skein_bools[2] = true;
    let provider = w::ProviderOffer::new(&w::CEILINGS, provider).expect("key provider");
    me.providers = encoded(provider.measure() + 4, |writer| {
        writer.put(&provider.measure().to_be_bytes())?;
        provider.encode(writer)
    });
    w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, me).expect("me"))
}

fn me_with_openai() -> w::DeliveryValue {
    let mut me = jig_client_test_records::frame_record().into_parts();
    let provider = w::ProviderOffer::new(
        &w::CEILINGS,
        w::ProviderOfferParts {
            provider: 17,
            name: w::DocumentText::new(
                &w::CEILINGS,
                w::DocumentTextParts { bytes: b"openai".as_slice().into(), cut: false },
            )
            .expect("name"),
            skein_bools: [true, true, false, false, true],
            programs: List::with_capacity(0),
            manage_usage: None,
            key_bytes: 4096,
        },
    )
    .expect("provider");
    me.providers = encoded(provider.measure() + 4, |writer| {
        writer.put(&provider.measure().to_be_bytes())?;
        provider.encode(writer)
    });
    w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, me).expect("me"))
}
