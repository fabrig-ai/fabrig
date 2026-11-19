use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_core as core;
use jig_domain_people as people;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{Event, SecretOwner};
use skein_lib::{
    Duration, Reader, ReplyTo, Time, Token, Wall, Writer,
    base64::{self, Alphabet},
};

fn ask_body() -> Box<[u8]> {
    let record = w::LinkAsk {};
    let mut bytes = Writer::new(0);
    record.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("empty record");
    bytes.finish()
}
fn ask(peer: &mut Peer, bearer: Option<&[u8]>) -> Token {
    peer.post(b"/jig/v1/link/ask", ask_body(), bearer)
}
fn ask_answer(peer: &Peer, exchange: Token) -> w::LinkAskAnswer {
    let mut reader = Reader::new(peer.answered(exchange).expect("ask answered"));
    let answer = w::LinkAskAnswer::decode(&w::CEILINGS, &mut reader).expect("ask answer");
    assert!(reader.is_empty());
    answer
}
fn whose(peer: &mut Peer, expected: u64) -> ReplyTo {
    let Some(core::Event::People(people::Event::Whose { reply_to, sign_in })) = peer.take_core() else {
        panic!("asks people whose sign-in");
    };
    assert_eq!(sign_in, expected);
    reply_to
}
fn signer() -> people::Reply {
    people::Reply::Signer(Some(people::Signer { identity: fixtures::application_person(), kind: people::Kind::Person }))
}
fn finish_ask(peer: &mut Peer, exchange: Token, sign_in: u64) -> w::LinkAsked {
    let to = whose(peer, sign_in);
    peer.answer_request(to, signer());
    let w::LinkAskAnswer::Asked(link) = ask_answer(peer, exchange) else { panic!("link asked") };
    assert_eq!(link.secret.expose().len(), 32);
    link
}
fn asked(peer: &mut Peer, bearer: &[u8], sign_in: u64) -> w::LinkAsked {
    let exchange = ask(peer, Some(bearer));
    finish_ask(peer, exchange, sign_in)
}
fn link_body(link: &w::LinkAsked) -> Box<[u8]> {
    let record = w::LinkSignIn { secret: link.secret.clone() };
    let mut bytes = Writer::new(skein_lib::length::usize_of(
        record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    record.write(&jig_client_protocol_wire::CEILINGS, &mut bytes).expect("measured link");
    bytes.finish()
}
fn post_link(peer: &mut Peer, link: &w::LinkAsked) -> Token {
    peer.post(b"/jig/v1/sign-in/link", link_body(link), None)
}
fn sign_in_answer(peer: &Peer, exchange: Token) -> w::SignInAnswer {
    let mut reader = Reader::new(peer.answered(exchange).expect("sign-in answered"));
    let answer = w::SignInAnswer::decode(&w::CEILINGS, &mut reader).expect("sign-in answer");
    assert!(reader.is_empty());
    answer
}
fn sign_in_request(peer: &mut Peer, cap: Wall) -> ReplyTo {
    let Some(core::Event::SignIn { reply_to, identity, held, until }) = peer.take_core() else {
        panic!("fresh sign-in")
    };
    assert_eq!(identity, fixtures::application_person());
    assert_eq!(held, None);
    assert_eq!(until, Some(cap));
    reply_to
}
fn reserve(peer: &mut Peer, link: &w::LinkAsked, sign_in: u64) -> (Token, ReplyTo) {
    let exchange = post_link(peer, link);
    let to = whose(peer, sign_in);
    peer.answer_request(to, signer());
    (exchange, sign_in_request(peer, fixtures::expires()))
}
fn token(number: u64) -> [u8; 40] {
    let mut token = [19; 40];
    token[..8].copy_from_slice(&number.to_be_bytes());
    token
}
fn bearer(token: &[u8]) -> Box<[u8]> {
    let mut bytes = Writer::new(61);
    bytes.put(b"Bearer ").expect("prefix");
    base64::encode(Alphabet::Url, token, &mut bytes).expect("credential");
    bytes.finish()
}
fn another_person(peer: &mut Peer, number: u64) -> Box<[u8]> {
    let token = token(number);
    peer.keep_digest(SecretOwner::SignIn(number), &token, fixtures::expires());
    bearer(&token)
}

#[test]
fn an_asked_link_signs_in_the_askers_party_once_with_a_browser_cookie() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    peer.hold_gets();
    let exchange = ask(&mut peer, Some(&bearer));
    assert!(peer.answered(exchange).is_none());
    assert!(peer.take_core().is_none());
    peer.answer_get();
    let link = finish_ask(&mut peer, exchange, fixtures::FIRST_SIGN_IN);
    assert_eq!(link.expires, fixtures::limits().link_span.as_nanos());
    let count = peer.link_signed_in_count();
    let headers = [
        (b"host".as_slice(), fixtures::HOST.as_bytes()),
        (b"content-type".as_slice(), b"application/octet-stream".as_slice()),
        (b"jig-client".as_slice(), b"browser".as_slice()),
        (b"cookie".as_slice(), b"jig-sign-in=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".as_slice()),
    ]
    .map(|(name, value)| skein_http::Header { name: Box::from(name), value: Box::from(value) });
    let exchange = peer.http(skein_http::Method::Post, b"/jig/v1/sign-in/link", link_body(&link), Box::new(headers));
    // Even a carried credential isn't checked: the link makes a fresh sign-in.
    let to = whose(&mut peer, fixtures::FIRST_SIGN_IN);
    peer.answer_request(to, signer());
    let to = sign_in_request(&mut peer, fixtures::expires());
    peer.answer_request(to, people::Reply::SignedIn { person: 93, sign_in: 94, expires: fixtures::expires() });
    assert!(peer.answered(exchange).is_none(), "credential waits for durability");
    peer.answer_kept(94);
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.party, 93);
    assert!(answer.token.is_none(), "browser gets its token by cookie");
    let cookie = peer
        .response_headers(exchange)
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(b"set-cookie"))
        .expect("browser cookie");
    assert!(cookie.value.starts_with(b"jig-sign-in="));
    assert!(cookie.value.ends_with(b"; Path=/jig; HttpOnly; SameSite=Strict; Max-Age=86400"));
    assert_eq!(peer.link_signed_in_count(), count, "no startup notice");
    let prints = peer.link_count();
    let again = post_link(&mut peer, &link);
    assert_eq!(sign_in_answer(&peer, again), w::SignInAnswer::LinkUsed);
    assert_eq!(peer.link_count(), prints, "a remembered asked link draws no printed link");
}

#[test]
fn an_asked_links_expiry_is_capped_and_fires_at_the_saved_expiry() {
    let mut peer = Peer::started(&fixtures::settings());
    let cap = Wall::from_nanos(Duration::from_secs(10).as_nanos());
    let bearer = fixtures::signed_in_until(&mut peer, cap);
    let link = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    assert_eq!(link.expires, cap.as_nanos());
    assert_eq!(peer.next_deadline(), Some(Time::from_nanos(cap.as_nanos())));
    peer.advance_time(Duration::from_secs(10));
    assert_eq!(
        peer.next_deadline(),
        Some(Time::from_nanos(fixtures::limits().link_span.as_nanos())),
        "deadline fired without drawing an asked link"
    );
    let expired = post_link(&mut peer, &link);
    assert_eq!(sign_in_answer(&peer, expired), w::SignInAnswer::LinkExpired);
    assert!(peer.take_core().is_none());
}

#[test]
fn an_asked_link_found_expired_draws_no_printed_replacement() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let link = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    let count = peer.link_count();
    peer.elapse_without_firing(fixtures::limits().link_span);
    let expired = post_link(&mut peer, &link);
    assert_eq!(sign_in_answer(&peer, expired), w::SignInAnswer::LinkExpired);
    assert_eq!(peer.link_count(), count);
    assert!(peer.take_core().is_none());
}

#[test]
fn asking_again_replaces_the_same_sign_ins_live_link() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let first = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    let second = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    assert_ne!((&first.secret), (&second.secret));
    let first = post_link(&mut peer, &first);
    assert_eq!(sign_in_answer(&peer, first), w::SignInAnswer::LinkExpired);
    let (_, to) = reserve(&mut peer, &second, fixtures::FIRST_SIGN_IN);
    peer.answer_request(to, people::Reply::Again(people::Again::Busy));
}

#[test]
fn asks_without_credentials_and_with_services_are_refused_but_failed_checks_are_signed_out() {
    let mut peer = Peer::started(&fixtures::settings());
    let no_credential = ask(&mut peer, None);
    assert_eq!(ask_answer(&peer, no_credential), w::LinkAskAnswer::Refused);
    let service_token = token(11);
    let service = bearer(&service_token);
    peer.keep_digest(SecretOwner::Service(11), &service_token, fixtures::expires());
    let service = ask(&mut peer, Some(&service));
    assert_eq!(ask_answer(&peer, service), w::LinkAskAnswer::Refused);
    let invalid = ask(&mut peer, Some(&bearer(&token(12))));
    assert_eq!(ask_answer(&peer, invalid), w::LinkAskAnswer::SignedOut);
    let person = fixtures::signed_in(&mut peer);
    let service = ask(&mut peer, Some(&person));
    let to = whose(&mut peer, fixtures::FIRST_SIGN_IN);
    peer.answer_request(
        to,
        people::Reply::Signer(Some(people::Signer {
            identity: fixtures::application_person(),
            kind: people::Kind::Service,
        })),
    );
    assert_eq!(ask_answer(&peer, service), w::LinkAskAnswer::Refused);
    let ended = ask(&mut peer, Some(&person));
    let to = whose(&mut peer, fixtures::FIRST_SIGN_IN);
    peer.answer_request(to, people::Reply::Signer(None));
    assert_eq!(ask_answer(&peer, ended), w::LinkAskAnswer::SignedOut);
    assert!(peer.take_core().is_none());
}

#[test]
fn disabled_links_refuse_an_ask_before_looking_up_its_credential() {
    let mut settings = fixtures::settings();
    settings.link = None;
    let mut peer = Peer::started(&settings);
    peer.hold_gets();
    let bearer = another_person(&mut peer, 11);
    let exchange = ask(&mut peer, Some(&bearer));
    assert_eq!(ask_answer(&peer, exchange), w::LinkAskAnswer::Refused);
    assert!(peer.take_core().is_none());
}

#[test]
fn an_ask_while_stopping_is_not_ready() {
    let mut peer = Peer::started(&fixtures::settings());
    let exchange = peer.accepted_post(b"/jig/v1/link/ask", ask_body());
    peer.event(Event::Stop);
    assert_eq!(ask_answer(&peer, exchange), w::LinkAskAnswer::NotReady);
    assert!(peer.take_core().is_none());
}

#[test]
fn a_fifth_asker_is_busy_even_while_another_links_sign_in_is_reserved() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let first = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    for number in 100..103 {
        let bearer = another_person(&mut peer, number);
        asked(&mut peer, &bearer, number);
    }
    let (_, to) = reserve(&mut peer, &first, fixtures::FIRST_SIGN_IN);
    let fifth = another_person(&mut peer, 103);
    let exchange = ask(&mut peer, Some(&fifth));
    let whose = whose(&mut peer, 103);
    peer.answer_request(whose, signer());
    assert_eq!(ask_answer(&peer, exchange), w::LinkAskAnswer::Busy);
    peer.answer_request(to, people::Reply::Again(people::Again::Busy));
    let (_, to) = reserve(&mut peer, &first, fixtures::FIRST_SIGN_IN);
    peer.answer_request(to, people::Reply::SignedIn { person: 93, sign_in: 94, expires: fixtures::expires() });
    peer.answer_kept(94);
}

#[test]
fn an_ask_without_exchange_room_is_busy() {
    let mut limits = fixtures::limits();
    limits.exchanges = 1;
    let mut peer = Peer::with_limits(&fixtures::settings(), limits);
    let bearer = fixtures::signed_in(&mut peer);
    let mint = peer.mint(fixtures::application_person());
    let Some(core::Event::SignIn { reply_to, .. }) = peer.take_core() else { panic!("mint occupies the exchange") };
    let exchange = ask(&mut peer, Some(&bearer));
    assert_eq!(ask_answer(&peer, exchange), w::LinkAskAnswer::Busy);
    assert!(peer.take_core().is_none());
    peer.answer_request(reply_to, people::Reply::Again(people::Again::Busy));
    assert_eq!(peer.mint_refusal(mint), Some(jig_protocol_engine_clients::MintRefusal::Busy));
}

#[test]
fn retry_answers_return_an_asked_link_with_its_drawn_expiry() {
    for reply in [
        people::Reply::Again(people::Again::NotReady),
        people::Reply::Again(people::Again::Busy),
        people::Reply::Full { sign_ins: 2, earliest: Wall::from_nanos(1_000_000_000) },
    ] {
        let mut peer = Peer::started(&fixtures::settings());
        let bearer = fixtures::signed_in(&mut peer);
        let link = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
        let (exchange, to) = reserve(&mut peer, &link, fixtures::FIRST_SIGN_IN);
        peer.advance_time(Duration::from_secs(1));
        let expected = match &reply {
            people::Reply::Again(people::Again::NotReady) => w::SignInAnswer::NotReady,
            people::Reply::Again(people::Again::Busy) => w::SignInAnswer::Busy,
            people::Reply::Full { sign_ins, earliest } => w::SignInAnswer::Full(w::SignInsFull {
                sign_ins: *sign_ins,
                earliest: earliest.as_nanos(),
                link_until: link.expires,
            }),
            _ => unreachable!("retry fixtures"),
        };
        peer.answer_request(to, reply);
        assert_eq!(sign_in_answer(&peer, exchange), expected);
        let (exchange, to) = reserve(&mut peer, &link, fixtures::FIRST_SIGN_IN);
        peer.answer_request(to, people::Reply::Again(people::Again::Busy));
        assert_eq!(sign_in_answer(&peer, exchange), w::SignInAnswer::Busy);
        peer.advance_time(Duration::from_secs(299));
        let exchange = post_link(&mut peer, &link);
        assert_eq!(sign_in_answer(&peer, exchange), w::SignInAnswer::LinkExpired, "original deadline");
    }
}

#[test]
fn a_returned_link_expires_if_its_asker_has_a_new_live_one() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let first = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    let (_, to) = reserve(&mut peer, &first, fixtures::FIRST_SIGN_IN);
    let second = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    peer.answer_request(to, people::Reply::Again(people::Again::NotReady));
    let first = post_link(&mut peer, &first);
    assert_eq!(sign_in_answer(&peer, first), w::SignInAnswer::LinkExpired);
    let (_, to) = reserve(&mut peer, &second, fixtures::FIRST_SIGN_IN);
    peer.answer_request(to, people::Reply::Again(people::Again::Busy));
}

#[test]
fn an_asker_ended_before_the_link_is_used_signs_no_one_in_and_spends_it() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let link = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    let exchange = post_link(&mut peer, &link);
    let to = whose(&mut peer, fixtures::FIRST_SIGN_IN);
    peer.answer_request(to, people::Reply::Signer(None));
    assert_eq!(sign_in_answer(&peer, exchange), w::SignInAnswer::Refused);
    assert!(peer.take_core().is_none());
    let exchange = post_link(&mut peer, &link);
    assert_eq!(sign_in_answer(&peer, exchange), w::SignInAnswer::LinkUsed);
}

#[test]
fn a_reserved_asked_secret_is_busy_and_a_core_refusal_spends_it() {
    let mut peer = Peer::started(&fixtures::settings());
    let bearer = fixtures::signed_in(&mut peer);
    let link = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    let (exchange, to) = reserve(&mut peer, &link, fixtures::FIRST_SIGN_IN);
    let crossed = post_link(&mut peer, &link);
    assert_eq!(sign_in_answer(&peer, crossed), w::SignInAnswer::Busy);
    peer.event(Event::SignInRefused { to });
    assert_eq!(sign_in_answer(&peer, exchange), w::SignInAnswer::Refused);
    let spent = post_link(&mut peer, &link);
    assert_eq!(sign_in_answer(&peer, spent), w::SignInAnswer::LinkUsed);
}

#[test]
fn a_restart_forgets_an_asked_link() {
    let settings = fixtures::settings();
    let mut peer = Peer::started(&settings);
    let bearer = fixtures::signed_in(&mut peer);
    let link = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
    peer.restart(&settings);
    let exchange = post_link(&mut peer, &link);
    assert_eq!(sign_in_answer(&peer, exchange), w::SignInAnswer::LinkUnknown);
    assert!(peer.take_core().is_none());
}

#[test]
fn close_and_abort_drop_live_and_reserved_asked_links() {
    for abort in [false, true] {
        let mut peer = Peer::started(&fixtures::settings());
        let bearer = fixtures::signed_in(&mut peer);
        let first = asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
        let (_, to) = reserve(&mut peer, &first, fixtures::FIRST_SIGN_IN);
        asked(&mut peer, &bearer, fixtures::FIRST_SIGN_IN);
        assert!(peer.next_deadline().is_some());
        peer.event(if abort { Event::Abort } else { Event::Close });
        assert_eq!(peer.next_deadline(), None);
        if !abort {
            peer.answer_request(to, people::Reply::Again(people::Again::Busy));
            assert_eq!(peer.next_deadline(), None, "no reserved link restored while closing");
        }
        assert_eq!(peer.closed(), 1);
    }
}
