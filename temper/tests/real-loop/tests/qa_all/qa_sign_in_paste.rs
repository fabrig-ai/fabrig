//! Design-led QA on the fake issuer and the assembled standalone service.
use temper_real_loop_world::{Exit, calls, run_sign_in, terminal};

const FIELD: &[u8] = b"Address of the page that couldn't be reached";
// A varying-port callback may fail its address check before its state check.
const REFUSED: &[u8] = b"That ";
const READY: &[u8] = b"Signed in to ChatGPT";
/// The browser returns natively once per story.
const ONE_RETURN: usize = 1;

fn begin() -> Vec<calls::Step> {
    vec![
        calls::sign_in(calls::D1),
        terminal::open(),
        terminal::type_on(b"Accounts", b""),
        terminal::type_on(b"+ Connect a model", b""),
        terminal::type_on(b"Continue with ChatGPT", b""),
        terminal::see(FIELD),
    ]
}

// protocol/credentials.md 3; skein's oauth.md 6.5: a paste wins once,
// and a later native return cannot exchange the code again.
#[test]
fn paste_then_native_return_exchanges_once() {
    let ran = run_sign_in(|| {
        let mut steps = begin();
        steps.extend([
            calls::Step::Authorize,
            calls::Step::Terminal(terminal::Act::TypeKept { label: FIELD }),
            terminal::type_on(b"Finish connecting", b""),
            terminal::see(READY),
            calls::Step::Return,
            terminal::close(),
        ]);
        steps
    });
    assert_eq!(ran.code_exchanges(), 1, "one code exchange");
    assert_eq!(ran.returned().len(), ONE_RETURN);
    let late = &ran.returned()[0];
    assert!(late.error.is_some() || late.status.is_some_and(|status| status >= 400), "late return refused");
    assert_eq!(ran.terminal().exit, Some(Exit::Code(0)));
}

// client/README.md 5.5; jig's protocol/llm.md 5.4: an address refusal
// must leave the real listener available for the native return.
#[test]
fn bad_paste_leaves_native_return_working() {
    let ran = run_sign_in(|| {
        let mut steps = begin();
        steps.extend([
            terminal::type_on(FIELD, b"not-an-address"),
            terminal::type_on(b"Finish connecting", b""),
            terminal::see(b"That isn't this sign-in's address."),
            calls::Step::Authorize,
            calls::Step::Return,
            terminal::see(READY),
            terminal::close(),
        ]);
        steps
    });
    assert_eq!(ran.code_exchanges(), 1);
    assert_eq!(ran.returned().len(), ONE_RETURN);
    assert_eq!(ran.returned()[0].status, Some(200));
    assert!(ran.returned()[0].error.is_none());
}

// jig's domain/accounts.md 5; skein's oauth.md 6.5: replacing a waiting
// sign-in invalidates its old callback before any code exchange.
#[test]
fn old_address_after_starting_again_is_refused_without_an_exchange() {
    let ran = run_sign_in(|| {
        let mut steps = begin();
        steps.extend([
            calls::Step::Authorize,
            terminal::type_on("‹ All ways".as_bytes(), b""),
            terminal::type_on(b"Continue with ChatGPT", b""),
            terminal::see(FIELD),
            calls::Step::Terminal(terminal::Act::TypeKept { label: FIELD }),
            terminal::type_on(b"Finish connecting", b""),
            terminal::see(REFUSED),
            terminal::close(),
        ]);
        steps
    });
    assert_eq!(ran.code_exchanges(), 0, "the superseded address cannot exchange a code");
}

// skein's oauth.md 6.5: only state refusals spend the four-paste quota.
#[test]
fn repeated_malformed_pastes_do_not_spend_the_state_quota() {
    let ran = run_sign_in(|| {
        let mut steps = begin();
        for _ in 0..5 {
            steps.extend([
                terminal::type_on(FIELD, b"not-an-address"),
                terminal::type_on(b"Finish connecting", b""),
                terminal::see(b"That isn't this sign-in's address."),
            ]);
        }
        steps.extend([
            calls::Step::Authorize,
            calls::Step::Terminal(terminal::Act::TypeKept { label: FIELD }),
            terminal::type_on(b"Finish connecting", b""),
            terminal::see(READY),
            terminal::close(),
        ]);
        steps
    });
    assert_eq!(ran.code_exchanges(), 1);
}

#[test]
#[ignore = "needs: real-loop fake issuer capacity for two outstanding authorization codes"]
fn restarting_after_approval_then_pasting_the_new_callback_succeeds() {}

#[test]
#[ignore = "needs: real-loop sign-in fixture with a kernel-reserved fixed callback port"]
fn restarting_a_fixed_port_sign_in_waits_for_its_old_listener_to_settle() {}

#[test]
#[ignore = "needs: real-loop act to mutate a kept callback without exposing it"]
fn four_wrong_states_limit_pastes_but_not_the_native_listener() {}

#[test]
#[ignore = "needs: real-loop sign-in deadline control and a held code exchange"]
fn expiry_and_native_paste_race_take_at_most_one_redirect() {}

#[test]
#[ignore = "needs: real-loop callback submission from a second authenticated party"]
fn another_party_cannot_finish_the_starters_sign_in() {}
