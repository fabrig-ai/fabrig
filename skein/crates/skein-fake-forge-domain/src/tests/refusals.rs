use super::*;
use crate::{Called, Fault};

#[test]
fn a_policy_refuses_author_and_writer_closes_only_until_cleared() {
    let mut h = Harness::new(calm());
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.open(b"work").unwrap();
    h.push(ENGINE, b"other", head).unwrap();
    h.open(b"other").unwrap();
    crate::refuse_close(&mut h.domain, REPOSITORY, 1, true);
    for user in [ENGINE, MAINTAINER] {
        let at = h.env.now;
        assert_eq!(h.call(user, write(Write::Close { number: 1 })), Err(Error::Protected));
        let fell = h.domain.pop_fell().unwrap();
        assert_eq!(
            fell,
            crate::Fell {
                at,
                until: None,
                fault: Fault::CloseRefused,
                repository: Some(copy_of(REPOSITORY)),
                user: Some(user),
                call: Some(Called::Write),
                reply_to: Some(Token::new(h.calls)),
                pull: Some(1),
                commit: None,
                run: None,
                runner: None,
            }
        );
        assert!(h.domain.pop_fell().is_none());
        assert_eq!(h.pull(1).state, State::Open);
    }
    h.ok(MAINTAINER, write(Write::Close { number: 2 }));
    assert_eq!(h.pull(2).state, State::Closed);
    crate::refuse_close(&mut h.domain, REPOSITORY, 1, false);
    h.ok(ENGINE, write(Write::Close { number: 1 }));
    assert_eq!(h.pull(1).state, State::Closed);
    assert!(h.domain.pop_fell().is_none());
}

#[test]
fn a_late_refused_close_keeps_its_call_token() {
    let mut h = Harness::new(calm());
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.open(b"work").unwrap();
    h.settle();
    crate::refuse_close(&mut h.domain, REPOSITORY, 1, true);
    h.env.limits.landing = 1000;
    h.env.limits.land_min = Duration::from_secs(2);
    h.env.limits.land_max = Duration::from_secs(2);
    assert_eq!(h.call(ENGINE, write(Write::Close { number: 1 })), Err(Error::Timeout));
    let token = Token::new(h.calls);
    assert_eq!(h.domain.pop_fell().unwrap().fault, Fault::Landing);
    let at = h.env.now.saturating_add(Duration::from_secs(2));
    h.until(at);
    assert_eq!(
        h.domain.pop_fell(),
        Some(crate::Fell {
            at,
            until: None,
            fault: Fault::CloseRefused,
            repository: Some(copy_of(REPOSITORY)),
            user: Some(ENGINE),
            call: Some(Called::Write),
            reply_to: Some(token),
            pull: Some(1),
            commit: None,
            run: None,
            runner: None,
        })
    );
    assert_eq!(h.pull(1).state, State::Open);
    assert!(h.domain.pop_fell().is_none());
}

#[test]
fn refusal_neither_survives_an_image_nor_prevents_head_deletion() {
    let mut h = Harness::new(calm());
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.open(b"work").unwrap();
    h.settle();
    crate::refuse_close(&mut h.domain, REPOSITORY, 1, true);
    let mut restored = Harness::new(calm());
    restored.domain = Domain::from_image(&h.env.limits, &h.domain.image().unwrap()).unwrap();
    restored.ok(ENGINE, write(Write::Close { number: 1 }));
    assert_eq!(restored.pull(1).state, State::Closed);
    h.ok(ENGINE, write(Write::DeleteBranch { branch: copy_of(b"work") }));
    assert_eq!(h.pull(1).state, State::Closed);
    assert!(h.domain.pop_fell().is_none());
}
