//! Redirect admission from the owner's paste and the listener (oauth/accounts.md, 5).
use super::private::Owner;
use super::*;
use skein_lib::Token;
use skein_lib::bytes::concat;
use skein_lib::stream::{Down, Up};

const REGISTERED: &[u8] = b"http://127.0.0.1:0/callback";
const BOUND: &[u8] = b"http://127.0.0.1:41000/callback";
const CONFIDENTIAL: &[u8] = b"https://service.example/callback";
const PORT: u16 = 41000;

fn account(public: bool, issued: bool) -> Account {
    let mut account = refresh_account(0, ([127, 0, 0, 1], 31000).into());
    if let Account::SignIn { registration, keeper, .. } = &mut account {
        registration.redirect_uri = bytes::copy_of(if public { REGISTERED } else { CONFIDENTIAL });
        registration.client_secret =
            if public { None } else { Some(skein_lib::Secret::new(bytes::copy_of(b"fake secret"))) };
        registration.terms = Some(skein_oauth::Terms { varying_port: public, issued_client_id: issued, ..terms() });
        *keeper = Keeper::Owner { kept: None };
    }
    account
}
fn configured(public: bool, issued: bool) -> Owner {
    let mut profile = limits();
    profile.client.document.issued_client_bytes = 32;
    Owner::configured([account(public, issued)], &profile)
}
fn waiting(issued: bool) -> (Owner, Token, Box<[u8]>) {
    let mut owner = configured(true, issued);
    owner.ask(Request::SignIn { account: 0, subject: None });
    let listener = owner.listen();
    owner.listening(listener, PORT);
    let visit = owner.visit();
    (owner, listener, visit)
}

fn paste(owner: &mut Owner, address: &[u8], state: &[u8], fields: &[u8]) {
    owner.ask(Request::Redirected {
        account: 0,
        uri: skein_lib::Secret::new(concat(&[address, b"?state=", state, fields])),
    });
}
fn taken(owner: &mut Owner) {
    assert_event(owner.above.pop(), Event::Taken { account: 0 });
    assert!(owner.above.is_empty(), "the sign-in terminal follows later");
}
fn refused(owner: &mut Owner, why: Refusal) {
    assert_refused(owner.above.pop(), 0, Asked::Redirected, why);
    assert!(owner.above.is_empty());
}
fn head(state: &[u8], fields: &[u8]) -> Box<[u8]> {
    concat(&[b"GET /callback?state=", state, fields, b" HTTP/1.1\r\nHost: 127.0.0.1:41000\r\n\r\n"])
}
fn settle_listener(owner: &mut Owner, listener: Token) {
    let mut held = Queue::uncapped();
    for _ in 0_u32..16 {
        owner.fire();
        let mut closed = false;
        while let Some(request) = owner.io.pop() {
            if let Lower::Close { entity } = request
                && entity == Token::new(10)
            {
                owner.socket(LowerEvent::Closed { owner: listener });
                closed = true;
            } else {
                held.push(request);
            }
        }
        while let Some(request) = held.pop() {
            owner.io.push(request);
        }
        if closed {
            return;
        }
    }
    panic!("taken sign-in closes its listener");
}
fn signs_in(owner: &mut Owner) {
    let identity = owner.sent();
    let mut events = owner.finish_tokens(identity, private::BODY);
    assert_event(
        events.pop(),
        Event::SignedIn {
            account: 0,
            generation: 1,
            missing: Box::new([]),
            subject: None,
            profile: skein_oauth::ProfileClaims::default(),
        },
    );
    assert!(events.is_empty(), "one sign-in terminal");
    for _ in 0_u32..8 {
        owner.fire();
    }
    assert!(owner.io.is_empty() && owner.above.is_empty(), "one code exchange");
}

// Synthetic issuer answers and browser navigation (oauth/accounts.md, sections 5–8).
pub(super) fn synthetic(prefix: &[u8], length: usize) -> Box<[u8]> {
    const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_";
    let mut output = skein_lib::Writer::new(length);
    for byte in prefix.iter().chain(ALPHABET.iter().cycle()).take(length) {
        output.put(&[*byte]).expect("synthetic length");
    }
    output.finish()
}

pub(super) fn answer_profile() -> Limits {
    let mut profile = limits();
    profile.client.uri_bytes = 4096;
    profile.client.code_bytes = 4096;
    profile.client.scope_bytes = 4096;
    profile.client.state_bytes = 64;
    profile.client.url_bytes = 2048;
    profile.client.document.issued_client_bytes = 1024;
    profile.client.request_bytes = 8192;
    profile.http.request = 8192;
    profile.http.send = 8192;
    profile.tls.send = 8192;
    profile.redirect_bytes = 16_384;
    profile.server.head = listener_head(profile.redirect_bytes).expect("browser head");
    profile.server.headers = 32;
    profile
}

pub(super) fn issuer_answer(state: &[u8], code: &[u8]) -> Box<[u8]> {
    concat(&[
        BOUND,
        b"?code=",
        code,
        b"&scope=openid+profile+email+offline_access&state=",
        state,
        b"&client_id=",
        &synthetic(b"app_", 28),
    ])
}

pub(super) fn browser_request(answer: &[u8]) -> Box<[u8]> {
    let target = answer.strip_prefix(b"http://127.0.0.1:41000").expect("loopback answer");
    concat(&[
        b"GET ", target, b" HTTP/1.1\r\n",
        b"Host: 127.0.0.1:41000\r\nConnection: keep-alive\r\n",
        b"sec-ch-ua: \"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\"\r\n",
        b"sec-ch-ua-mobile: ?0\r\nsec-ch-ua-platform: \"Linux\"\r\nUpgrade-Insecure-Requests: 1\r\n",
        b"User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36\r\n",
        b"Accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7\r\n",
        b"Sec-Fetch-Site: cross-site\r\nSec-Fetch-Mode: navigate\r\nSec-Fetch-User: ?1\r\nSec-Fetch-Dest: document\r\n",
        b"Referer: https://issuer.example/\r\nAccept-Encoding: gzip, deflate, br, zstd\r\n",
        b"Accept-Language: en-GB,en-US;q=0.9,en;q=0.8\r\nCookie: session=", &synthetic(b"c_", 4000), b"\r\n\r\n",
    ])
}

fn answer_waiting() -> (Owner, Token, Box<[u8]>) {
    let mut owner = Owner::configured([account(true, true)], &answer_profile());
    owner.ask(Request::SignIn { account: 0, subject: None });
    let listener = owner.listen();
    owner.listening(listener, PORT);
    let visit = owner.visit();
    (owner, listener, visit)
}

fn browser_answer(owner: &mut Owner, listener: Token, answer: &[u8]) -> Box<[u8]> {
    callback_response(owner, listener, &browser_request(answer))
}

fn callback_response(owner: &mut Owner, listener: Token, request: &[u8]) -> Box<[u8]> {
    let callback = owner.callback(listener);
    let mut remaining = request;
    let mut response = List::with_capacity(4096);
    for _ in 0_u32..256 {
        owner.fire();
        if owner.callback_io(listener, callback, &mut remaining, &mut response) {
            assert!(remaining.is_empty(), "the browser request was read whole");
            return response.into_boxed();
        }
    }
    panic!("the browser response closes")
}

fn refusal_of_event(event: Option<&Event>) -> Option<Refusal> {
    if let Some(Event::Refused { why, .. }) = event { Some(*why) } else { None }
}

fn signs_in_with_code(owner: &mut Owner, code: &[u8]) {
    let (identity, request, _) = owner.sent_request();
    let body_at = bytes::find(&request, b"\r\n\r\n").expect("request body").checked_add(4).expect("body offset");
    let length_at =
        bytes::find(&request, b"Content-Length: ").expect("body length").checked_add(16).expect("length offset");
    let length_line = request.get(length_at..).expect("length field");
    let length_end = bytes::find(length_line, b"\r\n").expect("length end");
    let length =
        usize::try_from(bytes::decimal(length_line.get(..length_end).expect("length digits")).expect("length"))
            .expect("body length");
    let total = body_at.checked_add(length).expect("request length");
    let mut sent = skein_lib::Writer::new(total);
    sent.put(&request).expect("request head room");
    for _ in 0_u32..128 {
        owner.fire();
        let mut deferred = Queue::uncapped();
        while let Some(request) = owner.io.pop() {
            match request {
                Lower::Stream { down: Down::Send(bytes), .. } => sent.put(&bytes).expect("request body room"),
                Lower::Stream { down: Down::Demand { room, .. }, .. } if room != 0 => {
                    owner.socket(LowerEvent::Stream { owner: identity, up: Up::Room });
                }
                request @ (Lower::Listen { .. }
                | Lower::Connect { .. }
                | Lower::Bind { .. }
                | Lower::Reject { .. }
                | Lower::Stream { .. }
                | Lower::Output { .. }
                | Lower::Spawn { .. }
                | Lower::SpawnAlone { .. }
                | Lower::Signal { .. }
                | Lower::Usage { .. }
                | Lower::Window { .. }
                | Lower::Close { .. }
                | Lower::Abort { .. }) => deferred.push(request),
            }
        }
        while let Some(request) = deferred.pop() {
            owner.io.push(request);
        }
        if sent.written() == total {
            break;
        }
    }
    assert_eq!(sent.written(), total, "the token request body was sent whole");
    let sent = sent.finish();
    let body = sent.get(body_at..).expect("body");
    assert!(Owner::query(&concat(&[b"?", body]), b"code") == Some(code), "the whole code reaches the token request");
    let mut events = owner.finish_tokens(identity, private::BODY);
    assert_event(
        events.pop(),
        Event::SignedIn {
            account: 0,
            generation: 1,
            missing: Box::new([]),
            subject: None,
            profile: skein_oauth::ProfileClaims::default(),
        },
    );
    assert!(events.is_empty(), "one sign-in terminal");
}

#[test]
fn a_long_issuer_answer_is_taken_from_the_listener_and_pasted() {
    skein_rows::check(
        [
            ("listener 1900", (1900, true)),
            ("listener 3900", (3900, true)),
            ("paste 1900", (1900, false)),
            ("paste 3900", (3900, false)),
        ],
        |(length, listening)| {
            let (mut owner, listener, visit) = answer_waiting();
            let code = synthetic(b"ac_", length);
            let answer = issuer_answer(Owner::query(&visit, b"state").expect("state"), &code);
            if listening {
                let response = browser_answer(&mut owner, listener, &answer);
                assert!(response.starts_with(b"HTTP/1.1 200"), "listener response: {response:?}");
                assert!(bytes::find(&response, b"Sign-in received.").is_some());
            } else {
                owner.ask(Request::Redirected { account: 0, uri: skein_lib::Secret::new(answer) });
                let event = owner.above.pop();
                assert_eq!(refusal_of_event(event.as_ref()), None, "paste must be Taken");
                assert_event(event, Event::Taken { account: 0 });
                settle_listener(&mut owner, listener);
            }
            signs_in_with_code(&mut owner, &code);
        },
    );
}

#[test]
fn the_listener_bounds_the_whole_redirect_address() {
    let (mut owner, listener, visit) = answer_waiting();
    let code = synthetic(b"ac_", 1900);
    let answer = issuer_answer(Owner::query(&visit, b"state").expect("state"), &code);
    let length = usize::try_from(owner.env.limits.redirect_bytes).expect("redirect bound");
    let pad = synthetic(b"", length - answer.len() - 5);
    let padded = concat(&[&answer, b"&pad=", &pad]);
    assert_eq!(padded.len(), length);
    let response = browser_answer(&mut owner, listener, &padded);
    assert!(response.starts_with(b"HTTP/1.1 200"), "the address at redirect_bytes is taken");
    assert!(bytes::find(&response, b"Sign-in received.").is_some());
    signs_in_with_code(&mut owner, &code);
}

#[test]
fn an_over_long_listener_redirect_ends_only_its_own_sign_in() {
    skein_rows::check([("own state", 0_u8), ("another state", 1), ("another path", 2), ("state twice", 3)], |kind| {
        let (mut owner, listener, visit) = answer_waiting();
        let now = owner.env.now;
        let bound = owner.env.limits.redirect_bytes;
        let state = Owner::query(&visit, b"state").expect("state");
        let code = synthetic(b"ac_", 1900);
        let answer = issuer_answer(state, &code);
        let wrong = match kind {
            0 => answer.clone(),
            1 => issuer_answer(b"another-state", &code),
            2 => concat(&[b"http://127.0.0.1:41000/other", answer.strip_prefix(BOUND).expect("query")]),
            3 => concat(&[&answer, b"&state=", state]),
            _ => unreachable!("redirect row"),
        };
        let length = usize::try_from(bound).expect("redirect bound") + 1;
        let pad = synthetic(b"", length - wrong.len() - 5);
        let padded = concat(&[&wrong, b"&pad=", &pad]);
        assert_eq!(padded.len(), length);
        owner.ask(Request::Return { account: 0, location: bytes::copy_of(b"https://app.example/back") });
        let response = if kind == 0 {
            let callback = owner.callback_read(listener, &browser_request(&padded));
            assert!(owner.above.is_empty(), "the listener has not judged the state");
            owner.fire();
            let terminal = owner.above.pop();
            assert_event(
                terminal,
                Event::Failed {
                    account: 0,
                    ends: Ends::SignIn,
                    failure: Failure::LongRedirect { length: bound + 1, bound },
                },
            );
            owner.callback_answer(listener, callback)
        } else {
            browser_answer(&mut owner, listener, &padded)
        };
        assert_eq!(owner.env.now, now, "no clock advance to the sign-in deadline");
        if kind == 0 {
            assert!(response.starts_with(b"HTTP/1.1 414"), "own redirect: {response:?}");
            let body = bytes::find(&response, b"\r\n\r\n").expect("response head") + 4;
            assert_eq!(response.get(body..).expect("response body"), b"This sign-in's answer is longer than accounts-test accepts. Go back to accounts-test and start the sign-in again.\n");
            assert!(bytes::find(&response, b"content-type: text/plain; charset=utf-8\r\n").is_some());
            assert!(bytes::find(&response, b"location:").is_none());
            assert!(owner.above.is_empty());
            for _ in 0_u32..8 {
                owner.fire();
            }
            assert!(owner.io.is_empty() && owner.above.is_empty());
            assert!(!owner.component.has_work());
        } else {
            assert!(response.starts_with(b"HTTP/1.1 404"));
            assert!(bytes::find(&response, b"This is not the registered redirect.\n").is_some());
            assert!(bytes::find(&response, b"location:").is_none());
            assert!(owner.above.is_empty(), "another redirect leaves the sign-in waiting");
            let response = browser_answer(&mut owner, listener, &answer);
            assert!(response.starts_with(b"HTTP/1.1 303"));
            signs_in_with_code(&mut owner, &code);
        }
    });
}

#[test]
fn a_pasted_redirect_is_taken_and_signs_in() {
    let (mut owner, listener, visit) = waiting(false);
    // A connection already accepted when the paste wins still receives a refusal.
    let callback = owner.callback(listener);
    paste(&mut owner, BOUND, Owner::query(&visit, b"state").expect("state"), b"&code=code");
    taken(&mut owner);
    owner.socket(LowerEvent::Accepted {
        owner: listener,
        socket: Token::new(12),
        peer: skein_io::Addr::Ip(([127, 0, 0, 1], 41002).into()),
    });
    assert!(matches_reject(owner.io.pop()));
    let request = head(Owner::query(&visit, b"state").expect("state"), b"&code=code");
    let mut remaining = request.as_ref();
    let mut answer = List::with_capacity(4096);
    for _ in 0_u32..128 {
        owner.fire();
        if owner.callback_io(listener, callback, &mut remaining, &mut answer) {
            break;
        }
    }
    assert!(answer.as_slice().starts_with(b"HTTP/1.1 404"));
    signs_in(&mut owner);
}
fn matches_reject(request: Option<Lower>) -> bool {
    if let Some(Lower::Reject { socket }) = request { socket == Token::new(12) } else { false }
}

#[test]
fn a_paste_that_is_not_the_sign_ins_is_refused_and_the_sign_in_still_waits() {
    for (name, address, fields, wrong_state, expected) in [
        (
            "another host",
            b"http://localhost:41000/callback".as_slice(),
            b"&code=code".as_slice(),
            false,
            Refusal::Redirect,
        ),
        ("another port", b"http://127.0.0.1:41001/callback", b"&code=code", false, Refusal::Redirect),
        ("registered zero port", REGISTERED, b"&code=code", false, Refusal::Redirect),
        ("another path", b"http://127.0.0.1:41000/other", b"&code=code", false, Refusal::Redirect),
        ("fragment", BOUND, b"&code=code#fragment", false, Refusal::Redirect),
        ("state twice", BOUND, b"&state=other&code=code", false, Refusal::Redirect),
        ("code twice", BOUND, b"&code=one&code=two", false, Refusal::Redirect),
        ("space", BOUND, b"&code=one two", false, Refusal::Redirect),
        ("control byte", BOUND, b"&code=one\x01two", false, Refusal::Redirect),
        ("both", BOUND, b"&code=code&error=denied", false, Refusal::Redirect),
        ("neither", BOUND, b"", false, Refusal::Redirect),
        ("empty code", BOUND, b"&code=", false, Refusal::Redirect),
        ("no issued id", BOUND, b"&code=code", false, Refusal::Redirect),
        ("empty issued id", BOUND, b"&code=code&client_id=", false, Refusal::Redirect),
        ("another state", BOUND, b"&code=code&client_id=issued", true, Refusal::State),
        ("past url bound", BOUND, b"&code=code&extra=", false, Refusal::Redirect),
    ] {
        let (mut owner, listener, visit) = waiting(true);
        let state = Owner::query(&visit, b"state").expect("state");
        if name == "past url bound" {
            let fields = concat(&[fields, &[b'x'; 1024]]);
            paste(&mut owner, address, state, &fields);
        } else {
            paste(&mut owner, address, if wrong_state { b"another-state" } else { state }, fields);
        }
        assert_eq!(refusal_of(owner.above.pop()), Some((0, Asked::Redirected, expected)), "{name}");
        assert!(owner.io.is_empty() && owner.above.is_empty(), "{name}");
        paste(&mut owner, BOUND, state, b"&code=code&client_id=issued");
        taken(&mut owner);
        settle_listener(&mut owner, listener);
        signs_in(&mut owner);
    }
}

#[test]
fn after_state_refusals_every_paste_is_full_and_the_listener_still_takes_its_own() {
    let (mut owner, listener, visit) = waiting(false);
    for _ in 0..STATE_REFUSALS {
        paste(&mut owner, BOUND, b"another-state", b"&code=code");
        refused(&mut owner, Refusal::State);
    }
    paste(&mut owner, BOUND, Owner::query(&visit, b"state").expect("state"), b"&code=code");
    refused(&mut owner, Refusal::Full { bound: 4 });
    let callback = owner.callback_read(listener, &head(Owner::query(&visit, b"state").expect("state"), b"&code=code"));
    assert!(owner.callback_answer(listener, callback).starts_with(b"HTTP/1.1 200"));
    signs_in(&mut owner);
}

#[test]
fn a_paste_is_refused_as_not_waiting() {
    for name in [
        "before sign-in",
        "before listener",
        "before visit",
        "at deadline",
        "past deadline",
        "after cancel",
        "after taken",
        "after terminal",
    ] {
        let (mut owner, listener, visit) =
            if name == "before sign-in" || name == "before listener" || name == "before visit" {
                let mut owner = configured(true, false);
                if name != "before sign-in" {
                    owner.ask(Request::SignIn { account: 0, subject: None });
                }
                if name == "before visit" {
                    let listener = owner.listen();
                    owner.listening(listener, PORT);
                }
                (owner, Token::new(0), Box::from([]))
            } else {
                waiting(false)
            };
        match name {
            "at deadline" => owner.env.now = Time::ZERO.saturating_add(owner.env.limits.client.sign_in_time),
            "past deadline" => {
                owner.env.now = Time::ZERO
                    .saturating_add(owner.env.limits.client.sign_in_time)
                    .saturating_add(Duration::from_secs(1));
            }
            "after cancel" => {
                owner.ask(Request::Cancel { account: 0 });
                drop(owner.above.pop());
            }
            "after taken" | "after terminal" => {
                paste(&mut owner, BOUND, Owner::query(&visit, b"state").expect("state"), b"&code=code");
                taken(&mut owner);
                if name == "after terminal" {
                    settle_listener(&mut owner, listener);
                    signs_in(&mut owner);
                }
            }
            "before sign-in" | "before listener" | "before visit" => {}
            _ => unreachable!("fixture phase"),
        }
        paste(&mut owner, BOUND, b"arbitrary", b"&code=code");
        assert_eq!(refusal_of(owner.above.pop()), Some((0, Asked::Redirected, Refusal::NotWaiting)), "{name}");
        if name == "at deadline" || name == "past deadline" {
            owner.fire();
            assert_eq!(
                failure_of(owner.above.pop()),
                Some(Failed {
                    account: 0,
                    ends: Ends::SignIn,
                    failure: Failure::Exchange(skein_oauth::Failure::TimedOut),
                }),
                "{name}"
            );
        }
    }
}

#[test]
fn a_paste_judged_between_the_listeners_read_and_its_answer_wins_and_the_listener_refuses() {
    let (mut owner, listener, visit) = waiting(false);
    let state = Owner::query(&visit, b"state").expect("state");
    let callback = owner.callback_read(listener, &head(state, b"&code=listener-code"));
    paste(&mut owner, BOUND, state, b"&code=pasted-code");
    taken(&mut owner);
    assert!(owner.callback_answer(listener, callback).starts_with(b"HTTP/1.1 404"));
    signs_in(&mut owner);
}

#[test]
fn the_listeners_redirect_taken_first_leaves_a_later_paste_not_waiting() {
    let (mut owner, listener, visit) = waiting(false);
    let state = Owner::query(&visit, b"state").expect("state");
    let callback = owner.callback_read(listener, &head(state, b"&code=listener-code"));
    owner.fire();
    paste(&mut owner, BOUND, state, b"&code=pasted-code");
    refused(&mut owner, Refusal::NotWaiting);
    assert!(owner.callback_answer(listener, callback).starts_with(b"HTTP/1.1 200"));
    signs_in(&mut owner);
}

#[test]
fn the_listeners_redirect_with_another_state_is_refused_and_the_sign_in_still_waits() {
    let (mut owner, listener, visit) = waiting(false);
    for _ in 0..STATE_REFUSALS.saturating_add(1) {
        let callback = owner.callback_read(listener, &head(b"another-state", b"&code=code"));
        assert!(owner.callback_answer(listener, callback).starts_with(b"HTTP/1.1 404"));
        assert!(owner.above.is_empty());
    }
    // Listener state refusals do not spend the owner's paste allowance.
    paste(&mut owner, BOUND, b"another-state", b"&code=code");
    refused(&mut owner, Refusal::State);
    let callback = owner.callback_read(listener, &head(Owner::query(&visit, b"state").expect("state"), b"&code=code"));
    assert!(owner.callback_answer(listener, callback).starts_with(b"HTTP/1.1 200"));
    signs_in(&mut owner);
}

#[test]
fn a_listener_redirect_at_the_deadline_is_refused_and_the_sign_in_times_out() {
    let (mut owner, listener, visit) = waiting(false);
    let callback = owner.callback_read(listener, &head(Owner::query(&visit, b"state").expect("state"), b"&code=code"));
    owner.env.now = Time::ZERO.saturating_add(owner.env.limits.client.sign_in_time);
    assert!(owner.callback_answer(listener, callback).starts_with(b"HTTP/1.1 404"));
    assert_eq!(
        failure_of(owner.above.pop()),
        Some(Failed { account: 0, ends: Ends::SignIn, failure: Failure::Exchange(skein_oauth::Failure::TimedOut) })
    );
    assert!(owner.above.is_empty() && owner.io.is_empty());
}

#[test]
fn a_confidential_redirect_is_answered_taken() {
    let mut owner = configured(false, false);
    owner.ask(Request::SignIn { account: 0, subject: None });
    let visit = owner.visit();
    paste(&mut owner, CONFIDENTIAL, Owner::query(&visit, b"state").expect("state"), b"&code=code");
    taken(&mut owner);
    signs_in(&mut owner);
}

#[test]
fn a_state_bound_under_32_is_unusable() {
    for (bound, expected) in [(31, Some(Unusable::Limits)), (32, None)] {
        let mut profile = limits();
        profile.client.state_bytes = bound;
        let mut configured = List::with_capacity(1);
        configured.push(account(true, false)).expect("account room");
        assert_eq!(
            Component::new(configured, &profile, crypto(17), b"accounts-test").err(),
            expected,
            "state bound {bound}"
        );
        if bound == 32 {
            let mut owner = Owner::configured([account(true, false)], &profile);
            owner.ask(Request::SignIn { account: 0, subject: None });
            let listener = owner.listen();
            owner.listening(listener, PORT);
            let visit = owner.visit();
            assert_eq!(Owner::query(&visit, b"state").expect("state").len(), 32);
        }
    }
}

#[test]
fn an_admitted_provider_error_gets_the_return_before_the_sign_ins_failure() {
    let (mut owner, listener, visit) = waiting(true);
    owner.ask(Request::Return { account: 0, location: bytes::copy_of(b"https://app.example/back") });
    let callback =
        owner.callback_read(listener, &head(Owner::query(&visit, b"state").expect("state"), b"&error=access_denied"));
    let answer = owner.callback_answer(listener, callback);
    assert!(answer.starts_with(b"HTTP/1.1 303"));
    assert!(bytes::find(&answer, b"location: https://app.example/back\r\n").is_some());
    assert!(bytes::find(&answer, b"Sign-in received.").is_some());
    assert_eq!(
        failure_of(owner.above.pop()),
        Some(Failed { account: 0, ends: Ends::SignIn, failure: Failure::Exchange(skein_oauth::Failure::Refused) })
    );
    assert!(owner.io.is_empty() && owner.above.is_empty());
}

#[test]
fn an_owner_cancel_or_abort_stops_a_callback_waiting_for_judgement() {
    for (name, request) in [("cancel", Request::Cancel { account: 0 }), ("abort", Request::Abort)] {
        let (mut owner, listener, visit) = waiting(false);
        let callback =
            owner.callback_read(listener, &head(Owner::query(&visit, b"state").expect("state"), b"&code=code"));
        owner.ask(request);
        assert!(owner.callback_answer(listener, callback).is_empty(), "{name}");
        assert_eq!(
            failure_of(owner.above.pop()),
            Some(Failed {
                account: 0,
                ends: Ends::SignIn,
                failure: Failure::Exchange(skein_oauth::Failure::Cancelled),
            }),
            "{name}"
        );
        for _ in 0_u32..8 {
            owner.fire();
        }
        if name == "abort" {
            assert_event(owner.above.pop(), Event::Closed);
        }
        assert!(owner.above.is_empty() && owner.io.is_empty(), "{name}");
        assert!(!owner.component.has_work(), "{name}");
    }
}

#[test]
fn a_refused_listener_redirect_keeps_the_return_for_the_redirect_taken() {
    let (mut owner, listener, visit) = waiting(false);
    owner.ask(Request::Return { account: 0, location: bytes::copy_of(b"https://app.example/back") });
    let request = concat(&[
        b"GET /other?state=",
        Owner::query(&visit, b"state").expect("state"),
        b"&code=code HTTP/1.1\r\nHost: 127.0.0.1:41000\r\n\r\n",
    ]);
    let answer = callback_response(&mut owner, listener, &request);
    assert!(answer.starts_with(b"HTTP/1.1 404"));
    assert!(bytes::find(&answer, b"This is not the registered redirect.\n").is_some());
    assert!(owner.above.is_empty());
    let callback = owner.callback_read(listener, &head(b"another-state", b"&code=code"));
    let answer = owner.callback_answer(listener, callback);
    assert!(answer.starts_with(b"HTTP/1.1 404"));
    assert!(bytes::find(&answer, b"location:").is_none());
    let callback = owner.callback_read(listener, &head(Owner::query(&visit, b"state").expect("state"), b"&code=code"));
    let answer = owner.callback_answer(listener, callback);
    assert!(answer.starts_with(b"HTTP/1.1 303"));
    assert!(bytes::find(&answer, b"location: https://app.example/back\r\n").is_some());
    signs_in(&mut owner);
}
