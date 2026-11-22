use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_core as core;
use jig_engine_clients_world::fixtures::{self, Peer};
use skein_lib::{Duration, Time, Writer};
use skein_world::inspect::{Entry, Release, Wait, Wake};

#[test]
fn a_withheld_heartbeat_is_found_and_firing_it_passes() {
    let settings = fixtures::settings();
    let mut peer = Peer::signed_in(&settings, fixtures::FIRST_SIGN_IN);
    let stream = peer.open_stream();
    let inspection = peer.inspection();
    let wait = inspection.waits().iter().find(|wait| wait.kind == "the heartbeat's span").expect("heartbeat armed");
    let deadline = settings.heartbeat;
    assert_eq!(wait.bound, Some(Time::ZERO.saturating_add(deadline)));
    assert_eq!(wait.wakes, [Wake::Timer(wait.bound.expect("bounded heartbeat"))]);
    let waiter = wait.waiter.clone();
    let before = peer.inspection_moment();
    let bytes = peer.stream_bytes(stream).len();
    peer.elapse_without_firing(deadline.saturating_add(Duration::from_nanos(1)));
    let moment = peer.inspection_moment();
    assert_eq!(moment.seed, 1);
    assert_eq!(moment.iteration, before.iteration, "elapsed time alone does not pump");
    let finding = peer.check_inspection().expect_err("past its bound before firing");
    assert_eq!(
        finding,
        format!(
            "seed 1: at {} ns, iteration {}: jig engine clients {waiter}: waits on the heartbeat's span past bound {} ns",
            moment.at.as_nanos(),
            moment.iteration,
            deadline.as_nanos(),
        )
    );
    peer.advance_time(Duration::ZERO);
    assert!(peer.inspection_moment().iteration > moment.iteration);
    assert_eq!(&peer.stream_bytes(stream)[bytes..], b":\n\n");
    assert_eq!(peer.check_inspection(), Ok(()));
    let after = peer.inspection();
    let wait = after.waits().iter().find(|wait| wait.waiter == waiter).expect("next heartbeat armed");
    assert_eq!(wait.bound, Some(moment.at.saturating_add(deadline)));
}

#[test]
fn pending_exchanges_keep_their_bound_and_leave_on_their_own_answers() {
    let mut limits = fixtures::limits();
    limits.exchanges_per_client = 2;
    let mut peer =
        Peer::signed_in_with_bounds(&fixtures::settings(), fixtures::FIRST_SIGN_IN, limits, fixtures::http_limits());
    assert_eq!(peer.inspection().tables(), []);
    let first = peer.authenticated_post(b"/jig/v1/read", super::reads::read());
    let Some(core::Event::Read { reply: a, .. }) = peer.take_core() else { panic!("first read") };
    let a = a.into_token();
    let name_a = format!("exchange {}", a.raw());
    let inspection = peer.inspection();
    assert_eq!(inspection.component(), "jig engine clients");
    assert_eq!(inspection.tables().len(), 1);
    let table = &inspection.tables()[0];
    assert_eq!(table.name, "a sign-in's pending exchanges");
    assert_eq!(table.capacity, 2);
    assert_eq!(
        table.entries,
        [Entry { key: name_a.clone(), rule: "each request's answer", release: Release::Pending }]
    );
    assert_eq!(table.refused.as_slice(), []);
    assert!(inspection.waits().contains(&Wait {
        waiter: name_a.clone(),
        kind: "awaiting the engine's answer".into(),
        wakes: vec![Wake::Party("engine".into())],
        bound: None,
    }));
    assert_eq!(peer.check_inspection(), Ok(()));
    let second = peer.authenticated_post(b"/jig/v1/read", super::reads::read());
    let Some(core::Event::Read { reply: b, .. }) = peer.take_core() else { panic!("second read") };
    let b = b.into_token();
    let name_b = format!("exchange {}", b.raw());
    assert_eq!(
        peer.inspection().tables()[0].entries.iter().map(|entry| &entry.key).collect::<Vec<_>>(),
        [&name_a, &name_b]
    );
    let refused = peer.authenticated_post(b"/jig/v1/read", super::reads::read());
    assert!(peer.answered(refused).is_some(), "past the bound is answered, not parked");
    assert!(peer.take_core().is_none());
    assert_eq!(peer.inspection().tables()[0].entries.len(), 2);
    assert_eq!(peer.inspection().tables()[0].refused.as_slice(), []);
    peer.answer_read(skein_lib::ReplyTo::new(a), core::reads::Answer::None(core::reads::Why::Gone));
    assert!(peer.answered(first).is_some());
    assert!(peer.answered(second).is_none());
    assert_eq!(
        peer.inspection().tables()[0].entries,
        [Entry { key: name_b, rule: "each request's answer", release: Release::Pending }]
    );
    assert_eq!(peer.check_inspection(), Ok(()));
    peer.answer_read(skein_lib::ReplyTo::new(b), core::reads::Answer::None(core::reads::Why::Gone));
    assert!(peer.answered(second).is_some());
    assert_eq!(peer.inspection().tables(), []);
    assert!(!peer.inspection().waits().iter().any(|wait| wait.kind == "awaiting the engine's answer"));
    assert_eq!(peer.check_inspection(), Ok(()));
}

#[test]
fn link_and_approval_expiries_are_individual_timer_wakes() {
    let mut peer = Peer::started(&fixtures::settings());
    let deadline = Time::ZERO.saturating_add(fixtures::limits().link_span);
    let inspection = peer.inspection();
    for name in ["printed link", "local link"] {
        assert!(inspection.waits().contains(&Wait {
            waiter: name.into(),
            kind: "a link's expiry".into(),
            wakes: vec![Wake::Timer(deadline)],
            bound: Some(deadline),
        }));
    }
    peer.advance_time(fixtures::limits().link_span);
    assert_eq!(peer.inspection().waits(), []);

    let settings = jig_protocol_engine_clients::Settings { approval: true, link: None, ..fixtures::settings() };
    let mut peer = Peer::started(&settings);
    let ask = w::ApprovalAsk { port: None };
    let mut bytes = Writer::new(skein_lib::length::usize_of(ask.measure(&w::CEILINGS).expect("bounded ask")));
    ask.write(&w::CEILINGS, &mut bytes).expect("approval ask");
    let socket = peer.post(b"/jig/v1/approval/ask", bytes.finish(), None);
    assert!(peer.answered(socket).is_some());
    let deadline = Time::ZERO.saturating_add(Duration::from_secs(600));
    assert_eq!(
        peer.inspection().waits(),
        [Wait {
            waiter: "approval 0".into(),
            kind: "an approval's expiry".into(),
            wakes: vec![Wake::Timer(deadline)],
            bound: Some(deadline),
        }]
    );
    peer.advance_time(Duration::from_secs(600));
    assert_eq!(peer.inspection().waits(), []);
}

#[test]
fn an_asked_link_keeps_only_its_live_expiry() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let socket = peer.post(b"/jig/v1/link/ask", Box::new([]), Some(&bearer));
    let Some(core::Event::People(jig_domain_people::Event::Whose { reply_to, .. })) = peer.take_core() else {
        panic!("whose sign-in")
    };
    peer.answer_request(
        reply_to,
        jig_domain_people::Reply::Signer(Some(jig_domain_people::Signer {
            identity: fixtures::application_person(),
            kind: jig_domain_people::Kind::Person,
        })),
    );
    assert!(peer.answered(socket).is_some());
    let deadline = Time::ZERO.saturating_add(fixtures::limits().link_span);
    assert!(peer.inspection().waits().contains(&Wait {
        waiter: format!("asked link of sign-in {}", fixtures::FIRST_SIGN_IN),
        kind: "a link's expiry".into(),
        wakes: vec![Wake::Timer(deadline)],
        bound: Some(deadline),
    }));
    peer.advance_time(fixtures::limits().link_span);
    assert_eq!(peer.inspection().waits(), []);
}
