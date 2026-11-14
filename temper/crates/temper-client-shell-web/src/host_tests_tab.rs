//! A press opens the sign-in's tab; its answer points it while the page holds.
use super::*;
use jig_client_protocol_wire::{VERSION, v22 as w};
use skein_lib::{Duration, Reader};
use temper_client_protocol_wire as wire;

fn exchange(host: &mut Host) -> (u32, Box<[u8]>) {
    let requests = host.requests();
    requests
        .into_iter()
        .find_map(|request| {
            let Asked::Exchange { id, call, .. } = request else { return None };
            (call.path.as_ref() == b"/jig/v1/request").then_some((id, call.body))
        })
        .expect("keyed request left the linked host")
}

pub(super) fn linked() -> Host {
    linked_at(b"/accounts/connect", Vec::new())
}
fn linked_at(path: &[u8], records: Vec<skein_ui_web::store::Record>) -> Host {
    let mut host =
        Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
    host.start(path, records, None);
    host.requests();
    host.heard(Heard::StreamOpened);
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
    streamed(
        &mut host,
        &w::StreamEvent::Hello(
            w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
                .expect("hello"),
        ),
    );
    let watch = host
        .requests()
        .into_iter()
        .find_map(|request| {
            let Asked::Exchange { id, call, .. } = request else { return None };
            if call.path.as_ref() != b"/jig/v1/watch" {
                return None;
            }
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("watch envelope");
            let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
            if matches!(subject, w::Subject::Me) {
                let answer = w::OpenAnswer::Admitted;
                host.heard(Heard::Answered {
                    exchange: id,
                    status: 200,
                    body: encoded(answer.measure(), |writer| answer.encode(writer)),
                    headers: Vec::new(),
                });
                Some(open.watch())
            } else {
                None
            }
        })
        .expect("me watch");
    host.client(jig_client::Event::Stored);
    host.requests();
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
    let value = w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, me).expect("me"));
    streamed(
        &mut host,
        &w::StreamEvent::Delivery(
            w::Delivery::new(
                &w::CEILINGS,
                w::DeliveryParts { watch, document: owned(encoded(value.measure(), |writer| value.encode(writer))) },
            )
            .expect("me delivery"),
        ),
    );
    admit_watches(&mut host);
    host.client(jig_client::Event::Stored);
    admit_watches(&mut host);
    build(&mut host);
    host
}

fn admit_watches(host: &mut Host) {
    loop {
        let mut admitted = false;
        for request in host.requests() {
            if let Asked::Exchange { id, call, .. } = request
                && call.path.as_ref() == b"/jig/v1/watch"
            {
                let answer = w::OpenAnswer::Admitted;
                host.heard(Heard::Answered {
                    exchange: id,
                    status: 200,
                    body: encoded(answer.measure(), |writer| answer.encode(writer)),
                    headers: Vec::new(),
                });
                admitted = true;
            }
        }
        if !admitted {
            return;
        }
    }
}

fn press_sign_in() -> (Host, Duties) {
    let mut host = linked();
    let button = node(&host, Role::Button, b"Continue with ChatGPT");
    let duties = host.report(Report::Act { node: button, act: Act::Press });
    (host, duties)
}
fn answer_sign_in(host: &mut Host) -> Duties {
    host.client(jig_client::Event::Stored);
    let (signing, _) = exchange(host);
    let started = w::DocumentOutcome::AccountSignInStarted(
        w::OutcomeAccountSignInStarted::new(
            &w::CEILINGS,
            w::OutcomeAccountSignInStartedParts { source: 43, address: ADDRESS.into() },
        )
        .expect("sign-in started"),
    );
    let outcome = w::Outcome::Document(
        w::OutcomeDocument::new(&w::CEILINGS, w::OutcomeDocumentParts { value: started }).expect("outcome"),
    );
    let answer = w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).expect("done"));
    let answer = w::KeyedAnswer::FinalAnswer(owned(encoded(answer.measure(), |writer| answer.encode(writer))));
    host.report(Report::Heard(Heard::Answered {
        exchange: signing,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
        headers: Vec::new(),
    }))
}
#[test]
fn the_press_opens_before_transport_and_the_answer_points_without_return() {
    let (mut host, press) = press_sign_in();
    assert_eq!(press.tab, [TabAct::Open { words: "Opening ChatGPT’s sign-in…".as_bytes().into() }]);
    assert!(press.navigate.is_none());
    let answer = answer_sign_in(&mut host);
    assert_eq!(answer.tab, [TabAct::Point { to: ADDRESS.into() }]);
    assert_eq!(answer.show, Some((b"/accounts/connect?provider=17&way=sign-in&source=43".as_slice().into(), true)));
    assert!(answer.navigate.is_none());
    assert!(!answer.asked.iter().any(|ask| matches!(ask, Asked::Exchange { .. })), "no Return exchange");
    let settled = host.report(Report::Carried { reports: Vec::new(), writes: answer.writes });
    assert_eq!(settled.tab.len(), 0);
    assert!(!settled.asked.iter().any(|ask| matches!(ask, Asked::Exchange { .. })), "no saved Return");
}
#[test]
fn a_refused_tab_never_points_from_the_answer() {
    let (mut host, _) = press_sign_in();
    host.report(Report::TabRefused);
    let answer = answer_sign_in(&mut host);
    assert_eq!(answer.tab.len(), 0);
    assert!(answer.navigate.is_none());
}

#[test]
fn an_answer_saves_its_source_address_and_refusal_without_a_client_write() {
    for refused in [false, true] {
        let (mut host, _) = press_sign_in();
        if refused {
            host.report(Report::TabRefused);
        }
        let answer = answer_sign_in(&mut host);
        let saves: Vec<_> = answer
            .asked
            .iter()
            .filter_map(|ask| {
                let Asked::Save { key, value } = ask else { return None };
                (key.as_ref() == b"sign-in").then_some(value.as_ref())
            })
            .collect();
        let expected = [if refused { b"143 ".as_slice() } else { b"043 ".as_slice() }, ADDRESS].concat();
        assert_eq!(saves, [expected.as_slice()]);
        assert_eq!(answer.writes, 1, "only jig's answer erase is counted");
    }
}

#[test]
fn start_gives_the_kept_sign_in_after_start_and_drops_malformed_values() {
    for value in [b"143 https://issuer.example/visit".as_slice(), b"143", b"243 address", b"14294967296 address"] {
        let record = skein_ui_web::store::Record { key: records::SIGN_IN_KEY.into(), value: value.into() };
        let mut host = linked_at(b"/accounts/connect?provider=17&way=sign-in&source=43", vec![record]);
        let form = &host.root.pages().connect_model().expect("Start opened the form").request;
        let expected = if value == b"143 https://issuer.example/visit" {
            Some(b"https://issuer.example/visit".as_slice())
        } else {
            None
        };
        assert_eq!(form.sign_in_address(host.root.client()), expected);
        assert_eq!(host.root.pages().connect_model().expect("model page").tab_refused(), expected.is_some());
        assert!(host.report(Report::Start).tab.is_empty(), "restoring never opens or points a tab");
    }
}
#[test]
fn keeping_a_sign_in_is_a_save_without_a_client_write() {
    let mut host =
        Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
    host.out.push(Request::KeepSignIn {
        source: 43,
        address: b"https://issuer.example/visit".as_slice().into(),
        refused: false,
    });
    let duties = host.report(Report::Start);
    assert_eq!(duties.writes, 0);
    assert_eq!(duties.asked.len(), 1);
    let Asked::Save { key, value } = &duties.asked[0] else { panic!("sign-in save") };
    assert_eq!(key.as_ref(), records::SIGN_IN_KEY);
    assert_eq!(value.as_ref(), b"043 https://issuer.example/visit");
}
