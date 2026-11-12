use jig_client_protocol_wire::v20 as w;
use jig_engine_clients_world::fixtures::{self, Peer};
use jig_protocol_engine_clients::{Component, Event, LINK_SECRET_BYTES, MintRefusal, Settings, Unusable};
use skein_lib::{Duration, Env, Queue, Secret, Time, Wall};

fn given_settings() -> Settings {
    Settings { link_secret: Some(Secret::new(Box::new([9; 32]))), ..fixtures::settings() }
}
fn before_link_expiry() -> Duration {
    Duration::from_nanos(fixtures::limits().link_span.as_nanos() - 1)
}
fn given_link() -> Box<[u8]> {
    fixtures::printed(&Secret::new(Box::new([9; 32])))
}
fn finish_given_sign_in(peer: &mut Peer, link: &[u8]) -> w::SignedIn {
    let exchange = peer.post_link(link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    peer.sign_in_answer(exchange)
}
#[test]
fn a_given_secret_signs_in_as_the_first_link_and_no_link_is_said() {
    assert_eq!(LINK_SECRET_BYTES, 32);
    let mut peer = Peer::started(&given_settings());
    peer.advance_time(before_link_expiry());
    let answer = finish_given_sign_in(&mut peer, &given_link());
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(answer.expires(), fixtures::expires().as_nanos());
    assert_eq!(peer.link_count(), 0);
    assert_eq!(peer.local_link_count(), 1);
    let local = peer.local_link();
    assert!(local != given_link(), "the given secret belongs only to the printed link");
    finish_given_sign_in(&mut peer, &local);
}
#[test]
fn a_given_local_secret_signs_in_as_the_first_local_link_and_no_local_link_is_said() {
    let settings = Settings { local_secret: Some(Secret::new(Box::new([8; 32]))), ..given_settings() };
    let local = fixtures::printed(settings.local_secret.as_ref().expect("given local link"));
    let printed = given_link();
    let mut peer = Peer::started(&settings);
    assert_eq!(peer.local_link_count(), 0);
    assert_eq!(peer.link_count(), 0);
    let answer = finish_given_sign_in(&mut peer, &local);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(peer.local_link_count(), 1);
    let next = peer.local_link();
    assert!(next != local && next != printed, "the next local link differs from both given links");
    finish_given_sign_in(&mut peer, &printed);
    let exchange = peer.post_link(&local);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkUsed);
    assert_eq!(peer.link_count(), 0, "a used local link never prints");
    peer.advance_time(fixtures::limits().link_span);
    peer.restart(&settings);
    peer.advance_time(before_link_expiry());
    peer.post_link(&local);
    let to = pending_reply(&mut peer);
    peer.answer_request(to, jig_domain_people::Reply::Again(jig_domain_people::Again::NotReady));
    peer.advance_time(Duration::from_nanos(1));
    let exchange = peer.post_link(&local);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkExpired);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.local_link_count(), 2, "the expired local link is replaced");
}
#[test]
fn a_given_secret_spent_draws_a_fresh_link() {
    let mut peer = Peer::started(&given_settings());
    let link = given_link();
    finish_given_sign_in(&mut peer, &link);
    let exchange = peer.post_link(&link);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkUsed);
    assert_eq!(peer.link_count(), 1);
    assert!(peer.printed_link() != link, "replacement differs from the given link");
    let replacement = peer.printed_link();
    finish_given_sign_in(&mut peer, &replacement);
    assert_eq!(peer.link_count(), 1);
}
#[test]
fn a_given_secret_expires_a_span_from_its_taking() {
    let settings = given_settings();
    let mut peer = Peer::started(&settings);
    let span = fixtures::limits().link_span;
    peer.advance_time(span);
    peer.restart(&settings);
    peer.advance_time(before_link_expiry());
    let link = given_link();
    peer.post_link(&link);
    let to = pending_reply(&mut peer);
    peer.answer_request(to, jig_domain_people::Reply::Again(jig_domain_people::Again::NotReady));
    peer.advance_time(Duration::from_nanos(1));
    let exchange = peer.post_link(&link);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkExpired);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.link_count(), 1);
    assert!(peer.printed_link() != link, "expired link replaced");
    let replacement = peer.printed_link();
    finish_given_sign_in(&mut peer, &replacement);
    assert_eq!(peer.link_count(), 1);
}
#[test]
fn a_given_secret_answered_not_ready_is_not_spent() {
    for reply in retry_answers() {
        let mut peer = Peer::started(&given_settings());
        let link = given_link();
        peer.post_link(&link);
        let to = pending_reply(&mut peer);
        peer.answer_request(to, reply);
        let answer = finish_given_sign_in(&mut peer, &link);
        assert_eq!(answer.party(), fixtures::FIRST_PERSON);
        assert_eq!(peer.link_count(), 0);
    }
}
#[test]
fn a_given_secret_of_another_length_is_refused_at_start() {
    for (length, local, equal, expected) in [
        (0, false, false, Unusable::LinkSecret),
        (LINK_SECRET_BYTES - 1, false, false, Unusable::LinkSecret),
        (LINK_SECRET_BYTES + 1, false, false, Unusable::LinkSecret),
        (0, true, false, Unusable::LocalSecret),
        (LINK_SECRET_BYTES - 1, true, false, Unusable::LocalSecret),
        (LINK_SECRET_BYTES + 1, true, false, Unusable::LocalSecret),
        (LINK_SECRET_BYTES, true, true, Unusable::LocalSecret),
    ] {
        let secret = Some(Secret::new(vec![9; length].into_boxed_slice()));
        let settings = Settings {
            link_secret: if local { given_settings().link_secret } else { secret.clone() },
            local_secret: if local { secret } else { None },
            ..fixtures::settings()
        };
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::limits() };
        let mut out = Queue::uncapped();
        assert_eq!(
            Component::new(settings, &env, &mut out).err(),
            Some(expected),
            "length {length}, local {local}, equal {equal}"
        );
        assert!(out.is_empty());
    }
}
#[test]
fn a_given_secret_without_the_printed_link_is_no_link() {
    let settings = Settings { link: None, local_secret: Some(Secret::new(Box::new([8; 32]))), ..given_settings() };
    let mut peer = Peer::started(&settings);
    let exchange = peer.post_link(&given_link());
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Refused);
    let local = fixtures::printed(settings.local_secret.as_ref().expect("given local link"));
    let exchange = peer.post_link(&local);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Refused);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.link_count(), 0);
    assert_eq!(peer.local_link_count(), 0);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::limits() };
    let component = Component::new(settings, &env, &mut Queue::uncapped()).expect("disabled links");
    assert_eq!(component.next_deadline(), None, "disabled links have no live secret");
}
fn peer() -> Peer {
    Peer::started(&fixtures::settings())
}
fn pending_sign_in(peer: &mut Peer) -> skein_lib::Token {
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    exchange
}
fn complete_sign_in(peer: &mut Peer) -> w::SignedIn {
    let exchange = pending_sign_in(peer);
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    peer.sign_in_answer(exchange)
}
#[test]
fn the_printed_link_signs_the_first_owner_in() {
    let mut peer = peer();
    let answer = complete_sign_in(&mut peer);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(answer.expires(), fixtures::expires().as_nanos());
}
#[test]
fn a_spent_link_is_used_and_a_fresh_one_printed() {
    let mut peer = peer();
    let link = peer.printed_link();
    complete_sign_in(&mut peer);
    let exchange = peer.post_link(&link);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkUsed);
    assert!(peer.printed_link() != link, "the replacement printed link differs");
    assert_eq!(peer.link_signed_in_count(), 1, "a used link says nothing more");
}
#[test]
fn a_link_answered_not_ready_is_not_spent() {
    let mut peer = peer();
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    let Some(jig_domain_core::Event::SignIn { reply_to, .. }) = peer.take_core() else { panic!("sign-in") };
    peer.answer_request(reply_to, jig_domain_people::Reply::Again(jig_domain_people::Again::NotReady));
    assert_eq!(peer.answered(exchange), Some(b"\x02".as_slice()), "not_ready wire record");
    let answer = complete_sign_in(&mut peer);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(peer.printed_link(), link, "no replacement link");
}
#[test]
fn the_token_leaves_only_after_its_digest_is_kept() {
    let mut peer = peer();
    let exchange = pending_sign_in(&mut peer);
    assert_eq!(peer.answered(exchange), None);
    assert_eq!(peer.digest_kept(), Some(fixtures::FIRST_SIGN_IN));
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    assert_eq!(
        u64::from_be_bytes(
            answer.token().as_ref().expect("native token").expose()[..8].try_into().expect("sign-in number")
        ),
        fixtures::FIRST_SIGN_IN
    );
}
#[test]
fn a_failed_keep_hands_no_token_out() {
    let mut peer = peer();
    let exchange = pending_sign_in(&mut peer);
    peer.fail_keep();
    assert_eq!(peer.answered(exchange), None);
}
#[test]
fn an_unknown_bearer_is_answered_signed_out() {
    let mut peer = peer();
    let exchange = peer.post(
        b"/jig/v1/request",
        Box::new([0]),
        Some(b"Bearer AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
    );
    assert_eq!(peer.answered(exchange), Some(b"\x04".as_slice()));
}
#[test]
fn two_tokens_drawn_in_turn_differ() {
    let mut peer = peer();
    let first = complete_sign_in(&mut peer);
    let spent = peer.printed_link();
    peer.post_link(&spent);
    let second = complete_sign_in(&mut peer);
    assert_ne!(first.token(), second.token());
}
#[test]
fn a_sign_in_refused_above_the_fill_mark_is_answered_refused() {
    let mut peer = peer();
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    peer.refuse_sign_in();
    assert_eq!(peer.answered(exchange), Some(b"\x01".as_slice()));
}
#[test]
fn no_core_event_holds_a_token_or_digest() {
    for local in [false, true] {
        let mut peer = peer();
        let link = if local { peer.local_link() } else { peer.printed_link() };
        peer.post_link(&link);
        assert_eq!(
            peer.core_record(),
            "SignIn { reply_to: ReplyTo(Token(1)), identity: Identity { key: IdentityKey { provider: 0, subject: [111, 119, 110, 101, 114] }, login: [111, 119, 110, 101, 114], name: [79, 119, 110, 101, 114] }, held: None }"
        );
    }
}

#[test]
fn a_mint_for_an_application_provider_signs_in_as_the_link_does() {
    let mut peer = peer();
    let exchange = peer.mint(fixtures::application_person());
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    assert!(peer.minted(exchange).is_none(), "before digest durability");
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let (credential, expires) = peer.minted(exchange).expect("minted");
    assert_eq!(u64::from_be_bytes(credential.expose()[..8].try_into().expect("number")), fixtures::FIRST_SIGN_IN);
    assert_eq!(expires, fixtures::expires());
    assert_eq!(credential.expose().len(), 40);
    assert_eq!(peer.link_signed_in_count(), 0, "a mint never says the printed link signed in");
}
fn assert_mint_refused(identity: jig_domain_people::Identity, why: MintRefusal) {
    let mut peer = peer();
    let exchange = peer.mint(identity);
    assert_eq!(peer.mint_refusal(exchange), Some(why));
    assert!(peer.take_core().is_none());
    assert_eq!(peer.digest_kept(), None);
}
#[test]
fn a_mint_with_a_login_that_is_not_utf8_is_refused_as_bound() {
    let identity = jig_domain_people::Identity { login: Box::new([255]), ..fixtures::application_person() };
    assert_mint_refused(identity, MintRefusal::Bound);
}
#[test]
fn a_mint_for_the_link_s_provider_is_refused_as_not_application() {
    assert_mint_refused(fixtures::first_owner(), MintRefusal::NotApplication);
}
#[test]
fn a_mint_with_a_name_past_its_bound_is_refused_as_bound() {
    let identity =
        jig_domain_people::Identity { name: vec![b'x'; 1025].into_boxed_slice(), ..fixtures::application_person() };
    assert_mint_refused(identity, MintRefusal::Bound);
}
#[test]
fn a_mint_the_people_child_answers_busy_is_refused_busy() {
    let mut peer = peer();
    let exchange = peer.mint(fixtures::application_person());
    let Some(jig_domain_core::Event::SignIn { reply_to, .. }) = peer.take_core() else { panic!("sign-in") };
    peer.event(Event::Reply { to: reply_to, reply: jig_domain_people::Reply::Again(jig_domain_people::Again::Busy) });
    assert_eq!(peer.mint_refusal(exchange), Some(MintRefusal::Busy));
    assert_eq!(peer.digest_kept(), None);
}

fn bearer(token: &[u8]) -> Box<[u8]> {
    let mut bytes = skein_lib::Writer::new(7 + (token.len() * 8).div_ceil(6));
    bytes.put(b"Bearer ").expect("prefix");
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, token, &mut bytes).expect("40-byte credential");
    bytes.finish()
}
fn fresh_link(peer: &mut Peer) -> Box<[u8]> {
    let spent = peer.printed_link();
    let exchange = peer.post_link(&spent);
    assert_eq!(decoded(peer, exchange), w::SignInAnswer::LinkUsed);
    let fresh = peer.printed_link();
    assert_ne!(fresh, spent);
    fresh
}

#[test]
fn a_live_credential_keeps_the_sign_in_without_minting_and_spends_the_link() {
    let mut peer = peer();
    let first = complete_sign_in(&mut peer);
    let bearer = bearer(first.token().as_ref().expect("credential").expose());
    let link = fresh_link(&mut peer);
    let count = peer.link_signed_in_count();
    assert_eq!(count, 1);
    let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
    let Some(jig_domain_core::Event::SignIn { reply_to, identity, held }) = peer.take_core() else { panic!("sign-in") };
    assert_eq!(held, Some(fixtures::FIRST_SIGN_IN));
    assert_eq!(identity, fixtures::first_owner());
    peer.answer_request(
        reply_to,
        jig_domain_people::Reply::SignedIn {
            person: fixtures::FIRST_PERSON,
            sign_in: fixtures::FIRST_SIGN_IN,
            expires: fixtures::expires(),
        },
    );
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.token(), &None);
    assert_eq!((answer.party(), answer.expires()), (first.party(), first.expires()));
    assert_eq!(peer.digest_kept(), None);
    assert!(peer.has_digest(fixtures::FIRST_SIGN_IN));
    assert_eq!(peer.link_signed_in_count(), count + 1, "a kept printed sign-in is said once");
    assert_ne!(fresh_link(&mut peer), link);
}

#[test]
fn a_held_credential_answered_with_another_sign_in_draws_a_token() {
    const NEW_SIGN_IN: u64 = fixtures::FIRST_SIGN_IN + 1;
    // A credential carries an eight-byte sign-in number and 32 drawn bytes (protocol/client/sign-in.md).
    const TOKEN_BYTES: usize = 40;
    let mut peer = peer();
    let bearer = fixtures::signed_in(&mut peer);
    let link = fresh_link(&mut peer);
    let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
    let Some(jig_domain_core::Event::SignIn { reply_to, held, .. }) = peer.take_core() else { panic!("sign-in") };
    assert_eq!(held, Some(fixtures::FIRST_SIGN_IN));
    peer.answer_request(
        reply_to,
        jig_domain_people::Reply::SignedIn {
            person: fixtures::FIRST_PERSON,
            sign_in: NEW_SIGN_IN,
            expires: fixtures::expires(),
        },
    );
    assert_eq!(peer.digest_kept(), Some(NEW_SIGN_IN));
    assert_eq!(peer.answered(exchange), None, "waits for digest durability");
    peer.answer_kept(NEW_SIGN_IN);
    let answer = peer.sign_in_answer(exchange);
    assert_eq!(answer.party(), fixtures::FIRST_PERSON);
    let token = answer.token().as_ref().expect("fresh token").expose();
    assert_eq!(token.len(), TOKEN_BYTES);
    assert_eq!(u64::from_be_bytes(token[..8].try_into().expect("sign-in number")), NEW_SIGN_IN);
}

#[test]
fn a_links_unaccepted_credential_always_signs_in_afresh() {
    use jig_protocol_engine_clients::SecretOwner;
    for case in ["wrong digest", "expired", "ended while checking", "service", "unknown", "malformed", "none"] {
        let mut peer = peer();
        let first = complete_sign_in(&mut peer);
        let original = first.token().as_ref().expect("40-byte credential").expose();
        let mut token = original.to_vec();
        match case {
            "wrong digest" => token[39] ^= 1,
            "expired" => {
                peer.keep_digest(SecretOwner::SignIn(fixtures::FIRST_SIGN_IN), original, skein_lib::Wall::EPOCH);
            }
            "ended while checking" => peer.hold_gets(),
            "service" => {
                token[..8].copy_from_slice(&99_u64.to_be_bytes());
                peer.keep_digest(SecretOwner::Service(99), &token, fixtures::expires());
            }
            "unknown" => token[..8].copy_from_slice(&98_u64.to_be_bytes()),
            "malformed" => token.truncate(39),
            "none" => {}
            _ => unreachable!(),
        }
        let link = fresh_link(&mut peer);
        let bearer = bearer(&token);
        let exchange = peer.post(
            b"/jig/v1/sign-in/link",
            peer.link_body(&link),
            if case == "none" { None } else { Some(&bearer) },
        );
        if case == "ended while checking" {
            assert!(peer.take_core().is_none(), "Get is pending");
            peer.event(Event::SignInEnded { sign_in: fixtures::FIRST_SIGN_IN });
            peer.answer_get(); // Even the stale, accepted Get cannot resurrect the ended sign-in.
        }
        assert_eq!(peer.answered(exchange), None, "{case}: waits for people");
        peer.answer_signed_in(42, fixtures::FIRST_PERSON); // Also asserts core held: None.
        assert_eq!(peer.digest_kept(), Some(42), "{case}: new digest");
        assert_eq!(peer.answered(exchange), None, "{case}: waits for durability");
        peer.answer_kept(42);
        let answer = peer.sign_in_answer(exchange);
        assert_eq!(answer.party(), fixtures::FIRST_PERSON, "{case}");
        assert_eq!(answer.token().as_ref().expect("fresh token").len(), 40, "{case}");
        assert_ne!(answer.token(), first.token(), "{case}");
    }
}

#[test]
fn a_links_pending_credential_check_survives_stop_and_drain_as_an_admitted_sign_in() {
    for drain in [false, true] {
        let mut peer = peer();
        let first = complete_sign_in(&mut peer);
        let bearer = bearer(first.token().as_ref().expect("credential").expose());
        let link = fresh_link(&mut peer);
        peer.hold_gets();
        let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
        assert!(peer.take_core().is_none());
        peer.event(if drain { Event::Close } else { Event::Stop });
        assert_eq!(peer.answered(exchange), None);
        assert_eq!(peer.closed(), 0);
        peer.answer_get();
        let Some(jig_domain_core::Event::SignIn { reply_to, held: Some(fixtures::FIRST_SIGN_IN), .. }) =
            peer.take_core()
        else {
            panic!("admitted sign-in completes after stop");
        };
        peer.answer_request(
            reply_to,
            jig_domain_people::Reply::SignedIn {
                person: fixtures::FIRST_PERSON,
                sign_in: fixtures::FIRST_SIGN_IN,
                expires: fixtures::expires(),
            },
        );
        assert_eq!(peer.sign_in_answer(exchange).token(), &None);
        assert_eq!(peer.digest_kept(), None);
        assert_eq!(peer.closed(), usize::from(drain));
    }
}

#[test]
fn aborting_a_links_pending_credential_check_releases_no_sign_in_or_token() {
    let mut peer = peer();
    let first = complete_sign_in(&mut peer);
    let bearer = bearer(first.token().as_ref().expect("credential").expose());
    let link = fresh_link(&mut peer);
    peer.hold_gets();
    let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
    assert!(peer.take_core().is_none());
    peer.event(Event::Abort);
    assert!(peer.take_core().is_none());
    assert_eq!(peer.answered(exchange), None);
    assert_eq!(peer.digest_kept(), None);
    assert_eq!(peer.closed(), 1);
}

#[test]
fn a_lost_links_credential_check_spends_the_link_like_a_lost_core_answer() {
    for local in [false, true] {
        let mut peer = peer();
        let first = complete_sign_in(&mut peer);
        let bearer = bearer(first.token().as_ref().expect("credential").expose());
        let link = if local { peer.local_link() } else { fresh_link(&mut peer) };
        peer.hold_gets();
        let exchange = peer.post(b"/jig/v1/sign-in/link", peer.link_body(&link), Some(&bearer));
        peer.cut_exchange(exchange);
        peer.answer_get();
        let Some(jig_domain_core::Event::SignIn { reply_to, held: Some(fixtures::FIRST_SIGN_IN), .. }) =
            peer.take_core()
        else {
            panic!("the admitted lost exchange still spends its link");
        };
        peer.answer_request(
            reply_to,
            jig_domain_people::Reply::SignedIn {
                person: fixtures::FIRST_PERSON,
                sign_in: fixtures::FIRST_SIGN_IN,
                expires: fixtures::expires(),
            },
        );
        assert_eq!(peer.answered(exchange), None);
        assert_eq!(peer.digest_kept(), None);
        let next = if local { peer.local_link() } else { fresh_link(&mut peer) };
        assert!(next != link, "the lost sign-in spends its link");
    }
}

fn retry_answers() -> [jig_domain_people::Reply; 3] {
    use jig_domain_people::{Again, Reply};
    [
        Reply::Again(Again::NotReady),
        Reply::Again(Again::Busy),
        Reply::Full { sign_ins: 2, earliest: skein_lib::Wall::from_nanos(900_000_000_000) },
    ]
}
fn pending_reply(peer: &mut Peer) -> skein_lib::ReplyTo {
    let Some(jig_domain_core::Event::SignIn { reply_to, .. }) = peer.take_core() else { panic!("sign-in") };
    reply_to
}
fn decoded(peer: &Peer, exchange: skein_lib::Token) -> w::SignInAnswer {
    let bytes = peer.answered(exchange).expect("answer");
    let mut reader = skein_lib::Reader::new(bytes);
    let answer = w::SignInAnswer::decode(&w::CEILINGS, &mut reader).expect("sign-in answer");
    assert!(reader.is_empty());
    answer
}

#[test]
fn busy_and_full_return_the_same_printed_link_and_it_can_sign_in() {
    let [not_ready, busy, full] = retry_answers();
    for (local, reply) in [
        (false, not_ready.clone()),
        (false, busy.clone()),
        (false, full.clone()),
        (true, not_ready),
        (true, busy),
        (true, full),
    ] {
        let mut peer = peer();
        let link = if local { peer.local_link() } else { peer.printed_link() };
        let exchange = peer.post_link(&link);
        let to = pending_reply(&mut peer);
        peer.advance_time(skein_lib::Duration::from_secs(1));
        let expected = match reply {
            jig_domain_people::Reply::Again(jig_domain_people::Again::NotReady) => w::SignInAnswer::NotReady,
            jig_domain_people::Reply::Again(jig_domain_people::Again::Busy) => w::SignInAnswer::Busy,
            jig_domain_people::Reply::Full { sign_ins, earliest } => w::SignInAnswer::Full(
                w::SignInsFull::new(
                    &w::CEILINGS,
                    w::SignInsFullParts { sign_ins, earliest: earliest.as_nanos(), link_until: 299_000_000_000 },
                )
                .expect("full answer within wire bounds"),
            ),
            jig_domain_people::Reply::Again(jig_domain_people::Again::Full { .. })
            | jig_domain_people::Reply::SignedIn { .. }
            | jig_domain_people::Reply::Signer(_)
            | jig_domain_people::Reply::SignedOut
            | jig_domain_people::Reply::Outcome(_)
            | jig_domain_people::Reply::Refused(_) => {
                unreachable!("retry fixtures contain only not ready, busy and full")
            }
        };
        peer.answer_request(to, reply);
        assert_eq!(decoded(&peer, exchange), expected);
        assert!(if local { peer.local_link() == link } else { peer.printed_link() == link }, "retry keeps its link");
        // One nanosecond before the original expiry remains usable.
        peer.advance_time(skein_lib::Duration::from_nanos(299_000_000_000 - 1));
        let exchange = peer.post_link(&link);
        peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
        peer.answer_kept(fixtures::FIRST_SIGN_IN);
        assert_eq!(peer.sign_in_answer(exchange).party(), fixtures::FIRST_PERSON);
        assert!(local || peer.printed_link() == link, "printed link is unchanged");
    }
}

#[test]
fn a_returned_link_past_its_original_expiry_is_expired_and_replaced() {
    for reply in retry_answers() {
        let mut peer = peer();
        let link = peer.printed_link();
        peer.post_link(&link);
        let to = pending_reply(&mut peer);
        peer.advance_time(skein_lib::Duration::from_secs(1));
        peer.answer_request(to, reply);
        peer.advance_time(skein_lib::Duration::from_secs(299));
        let exchange = peer.post_link(&link);
        assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkExpired);
        assert_ne!(peer.printed_link(), link);
        assert!(peer.take_core().is_none());
    }
}

#[test]
fn a_reserved_link_answers_its_secret_busy_and_another_unknown_without_drawing() {
    for local in [false, true] {
        let mut peer = peer();
        let link = if local { peer.local_link() } else { peer.printed_link() };
        let original = peer.post_link(&link);
        let to = pending_reply(&mut peer);
        let crossed = peer.post_link(&link);
        assert_eq!(decoded(&peer, crossed), w::SignInAnswer::Busy);
        let other =
            w::LinkSignIn::new(&w::CEILINGS, w::LinkSignInParts { secret: skein_lib::Secret::new(Box::new([0; 32])) })
                .expect("real-length link secret");
        let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(other.measure()));
        other.encode(&mut bytes).expect("measured link record");
        let wrong = peer.post(b"/jig/v1/sign-in/link", bytes.finish(), None);
        assert_eq!(decoded(&peer, wrong), w::SignInAnswer::LinkUnknown);
        assert!(if local { peer.local_link() == link } else { peer.printed_link() == link }, "reserved link unchanged");
        assert!(peer.take_core().is_none());
        peer.answer_request(to, jig_domain_people::Reply::Again(jig_domain_people::Again::Busy));
        assert_eq!(decoded(&peer, original), w::SignInAnswer::Busy);
        let exchange = peer.post_link(&link);
        peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
        peer.answer_kept(fixtures::FIRST_SIGN_IN);
        assert_eq!(peer.sign_in_answer(exchange).party(), fixtures::FIRST_PERSON);
    }
}

#[test]
fn a_core_refusal_drops_the_reservation_and_the_next_attempt_prints_a_link() {
    for case in ["printed", "local", "local stopped", "local signed out", "local refused"] {
        let mut peer = peer();
        let local = case != "printed";
        let link = if local { peer.local_link() } else { peer.printed_link() };
        let exchange = peer.post_link(&link);
        if case == "local signed out" || case == "local refused" {
            let to = pending_reply(&mut peer);
            let reply = if case == "local signed out" {
                jig_domain_people::Reply::SignedOut
            } else {
                jig_domain_people::Reply::Refused(jig_domain_people::Refusal::Unknown)
            };
            peer.answer_request(to, reply);
        } else {
            if case == "local stopped" {
                peer.event(Event::Stop);
            }
            peer.refuse_sign_in();
        }
        assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Refused);
        assert_eq!(peer.local_link_count(), 1 + usize::from(local), "local refusal renews at once");
        assert_eq!(peer.link_count(), 1);
        if case != "local stopped" {
            let retried = peer.post_link(&link);
            assert_eq!(decoded(&peer, retried), w::SignInAnswer::LinkUsed);
            assert_eq!(peer.link_count(), if local { 1 } else { 2 });
            let next = if local { peer.local_link() } else { peer.printed_link() };
            assert!(next != link, "refused link replaced");
            finish_given_sign_in(&mut peer, &next);
        }
    }
}

#[test]
fn admission_full_answers_busy_and_leaves_the_link_open() {
    let mut limits = fixtures::limits();
    limits.exchanges = 1;
    let mut peer = Peer::with_limits(&fixtures::settings(), limits);
    let mint = peer.mint(fixtures::application_person());
    let link = peer.printed_link();
    let exchange = peer.post_link(&link);
    assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Busy);
    assert_eq!(peer.printed_link(), link);
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN + 1, fixtures::FIRST_PERSON);
    peer.answer_kept(fixtures::FIRST_SIGN_IN + 1);
    assert!(peer.minted(mint).is_some());
    complete_sign_in(&mut peer);
    assert_eq!(peer.printed_link(), link);
}

#[test]
fn a_mint_the_people_child_answers_full_is_refused_busy() {
    let mut peer = peer();
    let exchange = peer.mint(fixtures::application_person());
    let to = pending_reply(&mut peer);
    peer.answer_request(to, retry_answers()[2].clone());
    assert_eq!(peer.mint_refusal(exchange), Some(MintRefusal::Busy));
    assert_eq!(peer.digest_kept(), None);
}

#[test]
fn a_services_sign_in_is_busy_for_busy_or_full() {
    use jig_protocol_engine_clients::SecretOwner;
    for reply in [retry_answers()[1].clone(), retry_answers()[2].clone()] {
        let mut peer = peer();
        let token = [7; 40];
        let number = u64::from_be_bytes(token[..8].try_into().expect("eight-byte service number"));
        peer.keep_digest(SecretOwner::Service(number), &token, fixtures::expires());
        let exchange = peer.post(b"/jig/v1/sign-in/deployment", Box::new([0]), Some(&bearer(&token)));
        let to = pending_reply(&mut peer);
        peer.answer_request(to, reply);
        assert_eq!(decoded(&peer, exchange), w::SignInAnswer::Busy);
        assert_eq!(peer.digest_kept(), None);
    }
}

// The in-memory history's bound (jig/docs/design/protocol/client/sign-in.md).
const REMEMBERED_LINKS: u32 = 8;
#[test]
fn a_link_that_cannot_sign_in_answers_why() {
    // Expire this many links, then try the oldest. The ninth is forgotten.
    for (label, past, used, empty, reserved, expected) in [
        ("unknown 32 bytes", 0, false, false, false, w::SignInAnswer::LinkUnknown),
        ("unknown empty", 0, false, true, false, w::SignInAnswer::LinkUnknown),
        ("reserved unknown 32 bytes", 0, false, false, true, w::SignInAnswer::LinkUnknown),
        ("reserved unknown empty", 0, false, true, true, w::SignInAnswer::LinkUnknown),
        ("eighth expired", REMEMBERED_LINKS, false, false, false, w::SignInAnswer::LinkExpired),
        ("ninth forgotten", REMEMBERED_LINKS + 1, false, false, false, w::SignInAnswer::LinkUnknown),
        ("reserved used", 0, true, false, true, w::SignInAnswer::LinkUsed),
        ("reserved expired", 1, false, false, true, w::SignInAnswer::LinkExpired),
    ] {
        let mut peer = peer();
        let oldest = peer.printed_link();
        let local = peer.local_link();
        peer.post_link(&local);
        let local_reservation = pending_reply(&mut peer);
        if used {
            finish_given_sign_in(&mut peer, &oldest);
            peer.post_link(&oldest);
        }
        for _ in 0..past {
            let link = peer.printed_link();
            peer.advance_time(fixtures::limits().link_span);
            peer.post_link(&link);
        }
        let live = peer.printed_link();
        let reservation = if reserved {
            peer.post_link(&live);
            Some(pending_reply(&mut peer))
        } else {
            None
        };
        let secret = if empty { Box::<[u8]>::from([]) } else { Box::<[u8]>::from([0; LINK_SECRET_BYTES]) };
        let link = if past == 0 && !used { fixtures::printed(&Secret::new(secret)) } else { oldest };
        let count = peer.link_count();
        let exchange = peer.post_link(&link);
        assert_eq!(decoded(&peer, exchange), expected, "{label}");
        assert_eq!(peer.link_count(), count, "{label}: no replacement of a live or reserved link");
        assert_eq!(peer.printed_link(), live, "{label}");
        assert!(peer.take_core().is_none(), "{label}");
        if let Some(to) = reservation {
            peer.answer_request(to, jig_domain_people::Reply::Again(jig_domain_people::Again::Busy));
        }
        finish_given_sign_in(&mut peer, &live);
        peer.answer_request(local_reservation, jig_domain_people::Reply::Again(jig_domain_people::Again::Busy));
    }
}

fn links_are_never_printed(printed: &[Box<[u8]>], local: &[Box<[u8]>]) {
    assert!(printed.iter().all(|link| !local.contains(link)), "local links are never printed");
}
#[test]
fn the_printed_and_the_local_link_each_sign_in_once_and_leave_the_other() {
    for case in ["printed first", "local first", "crossing"] {
        let mut peer = peer();
        assert_eq!(peer.link_count(), 1);
        assert_eq!(peer.local_link_count(), 1, "one local link at start");
        let printed = peer.printed_link();
        let local = peer.local_link();
        assert!(printed != local, "the links differ");
        let mut locals = vec![local.clone()];
        assert_eq!(peer.link_signed_in_count(), 0);
        if case == "crossing" {
            let exchange = peer.post_link(&printed);
            let to = pending_reply(&mut peer);
            let retry = peer.post_link(&printed);
            assert_eq!(decoded(&peer, retry), w::SignInAnswer::Busy);
            finish_given_sign_in(&mut peer, &local);
            assert_eq!(peer.link_signed_in_count(), 0);
            peer.answer_request(
                to,
                jig_domain_people::Reply::SignedIn {
                    person: fixtures::FIRST_PERSON,
                    sign_in: fixtures::FIRST_SIGN_IN + 1,
                    expires: fixtures::expires(),
                },
            );
            assert_eq!(peer.link_signed_in_count(), 1);
            peer.answer_kept(fixtures::FIRST_SIGN_IN + 1);
            assert_eq!(peer.sign_in_answer(exchange).party(), fixtures::FIRST_PERSON);
        } else {
            let (first, second) = if case == "printed first" { (&printed, &local) } else { (&local, &printed) };
            let local_count = peer.local_link_count();
            finish_given_sign_in(&mut peer, first);
            assert_eq!(peer.link_signed_in_count(), usize::from(case == "printed first"));
            assert_eq!(peer.local_link_count(), local_count + usize::from(case == "local first"));
            let local_count = peer.local_link_count();
            finish_given_sign_in(&mut peer, second);
            assert_eq!(peer.link_signed_in_count(), 1);
            assert_eq!(peer.local_link_count(), local_count + usize::from(case == "printed first"));
        }
        assert_eq!(peer.link_count(), 1, "spending either link never prints");
        assert_eq!(peer.local_link_count(), 2, "spending the local link replaces it at once");
        let next = peer.local_link();
        assert!(next != local, "the next local link differs");
        locals.push(next.clone());
        finish_given_sign_in(&mut peer, &next);
        locals.push(peer.local_link());
        assert_eq!(peer.link_signed_in_count(), 1, "local sign-ins say nothing");
        assert_eq!(peer.link_count(), 1);
        for link in [&printed, &local] {
            let exchange = peer.post_link(link);
            assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkUsed, "{case}");
        }
        links_are_never_printed(&[printed, peer.printed_link()], &locals);
    }
}
#[test]
fn a_local_links_dead_secret_never_prints_a_link() {
    for case in ["used", "expired"] {
        let mut peer = peer();
        let printed = peer.printed_link();
        let local = peer.local_link();
        let expected = if case == "used" {
            finish_given_sign_in(&mut peer, &local);
            w::SignInAnswer::LinkUsed
        } else {
            peer.advance_time(fixtures::limits().link_span);
            w::SignInAnswer::LinkExpired
        };
        let count = peer.link_count();
        let exchange = peer.post_link(&local);
        assert_eq!(decoded(&peer, exchange), expected);
        assert_eq!(peer.link_count(), count, "a dead local link never prints");
        let next = peer.local_link();
        assert!(next != local, "a fresh local link is said");
        finish_given_sign_in(&mut peer, &next);
        assert_eq!(peer.link_count(), count);
        links_are_never_printed(&[printed, peer.printed_link()], &[local, next, peer.local_link()]);
    }
}
#[test]
fn a_restart_forgets_both_links() {
    let mut settings = fixtures::settings();
    let mut peer = Peer::started(&settings);
    let old_printed = peer.printed_link();
    let old_local = peer.local_link();
    settings.seed = Secret::new(Box::new([8; 32]));
    peer.restart(&settings);
    assert_eq!(peer.link_count(), 2);
    assert_eq!(peer.local_link_count(), 2);
    let printed = peer.printed_link();
    let local = peer.local_link();
    assert!(printed != old_printed, "restart draws a new printed link");
    assert!(local != old_local, "restart draws a new local link");
    for old in [&old_printed, &old_local] {
        let exchange = peer.post_link(old);
        assert_eq!(decoded(&peer, exchange), w::SignInAnswer::LinkUnknown);
    }
    finish_given_sign_in(&mut peer, &printed);
    finish_given_sign_in(&mut peer, &local);
    links_are_never_printed(&[old_printed, printed], &[old_local, local, peer.local_link()]);
}
