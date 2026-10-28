//! The linked host's own sign-in answer must reach the return handshake.
use super::*;
use jig_client_protocol_wire::{VERSION, v19 as w};
use skein_lib::{Duration, Reader};
use temper_client_protocol_wire::{self as wire, v1 as f};

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
    let mut host = Host::new(11, Time::ZERO, Wall::EPOCH, temper_client_view::sign_in::Ways::NONE);
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
            let Asked::Exchange { call, .. } = request else { return None };
            if call.path.as_ref() != b"/jig/v1/watch" {
                return None;
            }
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("watch envelope");
            let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
            matches!(subject, w::Subject::Me).then_some(open.watch())
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

fn waiting_for_return() -> (Host, Vec<Asked>, u32) {
    let mut host = linked();
    let button = node(&host, Role::Button, b"Continue with ChatGPT");
    host.act(button, Act::Press);
    host.requests();
    host.client(jig_client::Event::Stored);
    let (signing, _) = exchange(&mut host);
    let pending = host.root.pages().connect_model().expect("flow").request.pending.as_ref().expect("own sign-in");
    assert!(pending.name.is_some() && pending.outcome.is_none(), "own unanswered request");
    let started = w::OwnerOutcome::AccountSignInStarted(
        w::OwnerOutcomeAccountSignInStarted::new(
            &w::CEILINGS,
            w::OwnerOutcomeAccountSignInStartedParts { source: 43, address: ADDRESS.into() },
        )
        .expect("sign-in started"),
    );
    let outcome = w::Outcome::Owner(
        w::OutcomeOwner::new(&w::CEILINGS, w::OutcomeOwnerParts { value: started }).expect("owner outcome"),
    );
    let answer = w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).expect("done"));
    let answer = w::KeyedAnswer::FinalAnswer(owned(encoded(answer.measure(), |writer| answer.encode(writer))));
    host.heard(Heard::Answered {
        exchange: signing,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
    });
    let (asked, writes) = host.duties();
    assert_eq!(writes, 2, "the sign-in erase and return save are separate batches");
    assert!(asked.iter().any(|ask| matches!(ask, Asked::Save { .. })), "return record");
    assert!(asked.iter().any(|ask| matches!(ask, Asked::Erase { .. })), "finished sign-in record");
    assert!(host.navigate.is_none(), "return is acknowledged before navigation");
    (host, asked, writes)
}

#[test]
fn the_linked_sign_in_answer_sends_a_return_before_leaving() {
    let (mut host, _, writes) = waiting_for_return();
    host.carried(Vec::new(), writes);
    let (returning, body) = exchange(&mut host);
    let keyed = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(&body)).expect("keyed return");
    assert_eq!(keyed.version(), wire::CLIENT_VERSION);
    let family = w::FamilyRequest::decode(&w::CEILINGS, &mut Reader::new(keyed.request())).expect("family request");
    assert_eq!(*family.scope(), w::Scope::Own);
    let request = wire::decode_requesting(family.record(), &f::CEILINGS).expect("return record");
    let f::Request::Return(request) = request.into_parts().request else { panic!("return request") };
    assert_eq!(request.source(), 43);
    assert_eq!(request.path(), b"/accounts/connect?provider=17&way=sign-in&source=43");
    let answer = f::Answer::new(&f::CEILINGS, f::AnswerParts { outcome: f::Outcome::Returning }).expect("returning");
    let record = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts {
            owner: w::Owner::Family(
                w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).expect("forge family"),
            ),
            version: wire::CLIENT_VERSION,
            record: encoded(answer.measure(), |writer| answer.encode(writer)),
        },
    )
    .expect("family answer");
    let answer = w::KeyedAnswer::FinalAnswer(record);
    host.heard(Heard::Answered {
        exchange: returning,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
    });
    host.requests();
    assert_eq!(host.navigate.take().as_deref(), Some(ADDRESS));
}

#[test]
fn a_return_whose_storage_fails_never_leaves_the_linked_host() {
    let (mut host, asked, writes) = waiting_for_return();
    let key = asked
        .into_iter()
        .find_map(|ask| if let Asked::Save { key, .. } = ask { Some(key) } else { None })
        .expect("return save");
    host.carried(vec![skein_ui_web::transport::Carried::Refused { key }], writes);
    assert!(!host.requests().iter().any(|ask| matches!(ask, Asked::Exchange { .. })), "failed return has no exchange");
    assert!(host.navigate.is_none(), "a failed return has no departure");
}
