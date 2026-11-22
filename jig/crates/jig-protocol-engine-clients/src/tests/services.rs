use super::super::*;
use super::fixtures as f;
use f::Exchange;
use skein_lib::Secret;
#[test]
fn a_made_service_answers_its_credential_once_with_its_digest_kept() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.make_service();
    exchange.service_made(to);
    exchange.service_got(None);
    exchange.keep_service();
    assert!(exchange.component.connections.get(&owner).expect("owner").response.is_none());
    exchange.service_kept_reply(f::SERVICE);
    assert_eq!(exchange.service_credential(owner), f::credential());
    exchange.service_kept_reply(f::SERVICE);
    assert_eq!(exchange.service_credential(owner), f::credential());
    assert!(exchange.component.pending.exchanges.is_empty());
    assert!(exchange.out.is_empty());
}
#[test]
fn inspection_keeps_a_reissue_under_its_asker_until_sign_out() {
    let mut exchange = Exchange::new();
    let (_, to) = exchange.reissue_service(f::SERVICE);
    assert_eq!(exchange.component.inspection_exchange_count(), 1);
    let (pending, sign_in) = exchange.component.inspection_exchange(0).expect("reissue exchange");
    assert_eq!(sign_in, Some(f::NUMBER));

    exchange.service_reissued(to, f::SERVICE);
    exchange.take_service_keep(f::SERVICE);
    assert_eq!(exchange.component.inspection_exchange(0), Some((pending, Some(f::NUMBER))));
    exchange.component.down(&exchange.env, Event::SignInEnded { sign_in: f::NUMBER }, &mut exchange.out);
    assert_eq!(exchange.component.inspection_exchange(0), Some((pending, None)));
    exchange.service_kept_reply(f::SERVICE);
    assert_eq!(exchange.component.inspection_exchange_count(), 0);
    assert_eq!(exchange.component.inspection_exchange(0), None);
}

#[test]
fn a_reissue_answers_only_after_keeping_its_fresh_credential() {
    let mut exchange = Exchange::new();
    let party = f::REISSUED_SERVICE;
    let (owner, to) = exchange.reissue_service(party);
    exchange.service_reissued(to, party);
    let kept = exchange.take_service_keep(party);
    assert!(!exchange.component.up(&exchange.env, &mut exchange.out), "not yet kept");
    exchange.service_kept_reply(party);
    let credential = exchange.reissued_credential(owner, party);
    let bytes = credential.expose();
    assert_eq!(bytes.len(), f::SERVICE_CREDENTIAL_BYTES);
    assert_eq!(bytes[..size_of::<u64>()], party.to_be_bytes());
    assert_eq!(kept.digest.expose(), &skein_lib::sha256::digest(bytes));

    assert!(exchange.component.up(&exchange.env, &mut exchange.out));
    let Some(Request::Transport(crate::ToTransport::Reply { exchange: answered, .. })) = exchange.out.pop() else {
        panic!("credential answer")
    };
    assert_eq!(answered, owner);
    exchange.service_kept_reply(party);
    assert!(!exchange.component.up(&exchange.env, &mut exchange.out));
    assert!(exchange.out.is_empty());
}

fn assert_service_held(exchange: &mut Exchange, owner: Token, party: u64, kept: Kept) {
    exchange.service_got_for(party, Some(kept));
    assert_eq!(exchange.sign_in_answer(owner), w::SignInAnswer::Busy);
    assert!(exchange.out.is_empty(), "no core sign-in");
    assert!(exchange.component.up(&exchange.env, &mut exchange.out));
    exchange.out.pop().expect("busy answer");
}

#[test]
fn a_reissue_holds_only_its_services_checked_sign_ins() {
    let mut exchange = Exchange::new();
    let waiting = exchange.get_service(f::SERVICE, f::credential().expose());
    let (_, to) = exchange.reissue_service(f::SERVICE);
    assert_service_held(&mut exchange, waiting, f::SERVICE, f::service_kept());

    exchange.service_reissued(to, f::SERVICE);
    exchange.take_service_keep(f::SERVICE);
    let waiting = exchange.get_service(f::SERVICE, f::credential().expose());
    assert_service_held(&mut exchange, waiting, f::SERVICE, f::service_kept());

    let other = f::REISSUED_SERVICE;
    let token = f::sign_in_token(other);
    exchange.get_service(other, &token);
    let digest = Secret::new(Box::new(skein_lib::sha256::digest(&token)));
    exchange.service_got_for(other, Some(Kept { digest, expires: f::service_kept().expires }));
    let _to = exchange.take_service_sign_in(other);
}

fn refused(refusal: w::Refusal) -> w::KeyedAnswer {
    let final_answer = w::Final::Refused(w::Refused { refusal, at: None });
    let record = final_answer.encode(&f::wire()).expect("bounded refusal");
    w::KeyedAnswer::FinalAnswer(w::Owned { owner: w::Owner::Jig, version: w::VERSION, record })
}

#[test]
fn unsuccessful_reissues_pass_through_and_release_service_sign_ins() {
    use people::{Again, Refusal, Reply, Role};
    for (reply, expected) in [
        (
            Reply::Refused(Refusal::Role { held: Some(Role::Member) }),
            refused(w::Refusal::Role(w::RefusalRole { held: Some(w::HeldRole::Member) })),
        ),
        (Reply::Refused(Refusal::NotOffered), refused(w::Refusal::NotOffered)),
        (Reply::Refused(Refusal::Unknown), refused(w::Refusal::Unknown)),
        (Reply::SignedOut, w::KeyedAnswer::SignedOut),
        (Reply::Again(Again::NotReady), w::KeyedAnswer::NotReady),
    ] {
        let mut exchange = Exchange::new();
        let (owner, to) = exchange.reissue_service(f::SERVICE);
        exchange.reply(to, reply);
        assert_eq!(exchange.answer(owner), expected);
        assert!(exchange.out.is_empty(), "no keep");
        let (_, _to) = exchange.sign_in_service();
    }
}

#[test]
fn a_reissue_waits_for_a_make_before_keeping() {
    let mut exchange = Exchange::new();
    let (made, to) = exchange.make_service();
    exchange.service_made(to);
    exchange.service_got(None);
    exchange.keep_service();

    let (reissued, to) = exchange.reissue_service(f::SERVICE);
    exchange.service_reissued(to, f::SERVICE);
    assert!(exchange.out.is_empty(), "the make's keep is still in flight");
    exchange.service_kept_reply(f::SERVICE);
    let kept = exchange.take_service_keep(f::SERVICE);
    exchange.service_kept_reply(f::SERVICE);
    let credential = exchange.reissued_credential(reissued, f::SERVICE);
    assert_ne!(credential, exchange.service_credential(made));
    assert_eq!(kept.digest.expose(), &skein_lib::sha256::digest(credential.expose()));
}

#[test]
fn overlapping_reissues_under_one_key_keep_distinct_credentials_in_turn() {
    let mut exchange = Exchange::new();
    let (first, to) = exchange.reissue_service(f::SERVICE);
    exchange.service_reissued(to, f::SERVICE);
    let first_keep = exchange.take_service_keep(f::SERVICE);
    let (second, to) = exchange.reissue_service(f::SERVICE);
    exchange.service_reissued(to, f::SERVICE);
    assert!(exchange.out.is_empty(), "the first keep is still in flight");

    exchange.service_kept_reply(f::SERVICE);
    let second_keep = exchange.take_service_keep(f::SERVICE);
    let first = exchange.reissued_credential(first, f::SERVICE);
    exchange.service_kept_reply(f::SERVICE);
    let second = exchange.reissued_credential(second, f::SERVICE);
    assert_ne!(first, second);
    assert_eq!(first_keep.digest.expose(), &skein_lib::sha256::digest(first.expose()));
    assert_eq!(second_keep.digest.expose(), &skein_lib::sha256::digest(second.expose()));
}

#[test]
fn a_reissue_replaces_the_digest_even_if_the_asker_signs_out() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.reissue_service(f::SERVICE);
    exchange.component.down(&exchange.env, Event::SignInEnded { sign_in: f::NUMBER }, &mut exchange.out);
    let Some(Request::Secrets(SecretsRequest::Erase { owner: SecretOwner::SignIn(f::NUMBER) })) = exchange.out.pop()
    else {
        panic!("erase the ended sign-in");
    };
    assert_eq!(exchange.answer(owner), w::KeyedAnswer::SignedOut);
    assert!(exchange.component.up(&exchange.env, &mut exchange.out));
    exchange.out.pop().expect("signed-out answer");
    exchange.service_reissued(to, f::SERVICE);
    let kept = exchange.take_service_keep(f::SERVICE);
    assert_eq!(kept.digest, f::credential_digest());
    exchange.service_kept_reply(f::SERVICE);
    assert!(exchange.component.pending.exchanges.is_empty());
    assert!(exchange.out.is_empty());
}

#[test]
fn a_replayed_make_service_answers_its_party_without_a_credential() {
    for concurrent in [false, true] {
        let mut exchange = Exchange::new();
        let (owner, to) = exchange.make_service();
        exchange.service_made(to);
        if concurrent {
            exchange.service_got(None);
            exchange.keep_service();
            let (other, to) = exchange.make_service();
            exchange.service_made(to);
            exchange.service_got(None);
            assert!(exchange.out.is_empty());
            exchange.service_kept_reply(f::SERVICE);
            assert_eq!(exchange.service_credential(owner), f::credential());
            assert_eq!((exchange.service_credential(other).expose()).len(), 0);
        } else {
            exchange.service_got(Some(f::service_kept()));
            assert_eq!((exchange.service_credential(owner).expose()).len(), 0);
        }
        assert!(exchange.out.is_empty());
        assert!(exchange.component.pending.exchanges.is_empty());
    }
}
#[test]
fn a_service_signs_in_with_its_credential_and_exchanges_with_its_sign_in() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.sign_in_service();
    exchange.reply(
        to,
        people::Reply::SignedIn { person: f::SERVICE, sign_in: f::NUMBER, expires: skein_lib::Wall::from_nanos(99) },
    );
    let Some(Request::Secrets(SecretsRequest::Keep { owner: who, .. })) = exchange.out.pop() else {
        panic!("sign-in keep")
    };
    assert_eq!(who, SecretOwner::SignIn(f::NUMBER));
    exchange.component.down(&exchange.env, Event::Secrets(SecretsAnswer::Kept { owner: who }), &mut exchange.out);
    let answer = exchange.signed_in_answer(owner);
    assert_eq!(answer.party, f::SERVICE);
    assert_eq!(answer.token.as_ref().expect("native token").expose()[..8], f::NUMBER.to_be_bytes());
    exchange.token = answer.token.as_ref().expect("native token").expose().into();
    let (request_owner, to) = exchange.ask();
    exchange.reply(to, people::Reply::Outcome(people::Outcome::Said { task: 8, message: 9 }));
    assert_eq!(exchange.final_answer(request_owner), f::started());
    assert!(exchange.out.is_empty());
}
#[test]
fn a_deployment_sign_in_naming_no_service_is_refused() {
    let mut exchange = Exchange::new();
    let (owner, to) = exchange.sign_in_service();
    exchange.reply(to, people::Reply::SignedOut);
    let (_, bytes) = exchange.response(owner);
    assert_eq!(
        w::SignInAnswer::decode(&f::wire(), &mut skein_lib::Reader::new(bytes)).expect("refusal"),
        w::SignInAnswer::Refused
    );
    assert!(exchange.out.is_empty());
    assert!(exchange.component.pending.exchanges.is_empty());
}
