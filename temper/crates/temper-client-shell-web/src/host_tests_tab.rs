//! A press opens the sign-in's tab; its answer points it while the page holds.
use super::*;
use jig_client_protocol_wire::{VERSION, v19 as w};
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
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
    host.start(b"/accounts/connect", Vec::new(), None);
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
    host.requests();
    host.client(jig_client::Event::Stored);
    host.requests();
    build(&mut host);
    host
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
    assert!(settled.tab.is_empty());
    assert!(!settled.asked.iter().any(|ask| matches!(ask, Asked::Exchange { .. })), "no saved Return");
}
#[test]
fn a_refused_tab_never_points_from_the_answer() {
    let (mut host, _) = press_sign_in();
    host.report(Report::TabRefused);
    let answer = answer_sign_in(&mut host);
    assert!(answer.tab.is_empty());
    assert!(answer.navigate.is_none());
}
