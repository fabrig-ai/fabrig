use skein_oauth_accounts_world::sign_in::{self, Story};
use skein_world::Memory;
#[test]
fn a_confidential_client_signs_in_through_the_owners_redirect() {
    drop(sign_in::run(47, Story::Confidential, true, false, Memory::Checked));
}
#[test]
fn a_wrong_path_is_refused_and_the_sign_in_still_waits() {
    for story in [Story::WrongPath, Story::WrongHost, Story::LongHead] {
        drop(sign_in::run(49, story, true, false, Memory::Checked));
    }
}
#[test]
fn a_close_while_a_sign_in_waits_for_a_person_still_waits_for_it() {
    for story in [Story::CloseWaiting, Story::CloseConfidential] {
        drop(sign_in::run(51, story, false, false, Memory::Checked));
    }
}
#[test]
fn a_sign_in_the_owner_cancels_fails_as_cancelled_and_closes_its_listener() {
    for story in [Story::Cancel, Story::Abort] {
        drop(sign_in::run(53, story, false, false, Memory::Checked));
    }
}
#[test]
fn the_shipped_person_deadline_ends_the_sign_in() {
    drop(sign_in::run(55, Story::Timeout, false, false, Memory::Checked));
}
#[test]
fn a_sign_in_whose_record_is_not_kept_fails_once_and_lends_nothing() {
    drop(sign_in::run(57, Story::NotKept, true, false, Memory::Checked));
}
#[test]
fn sign_in_redirect_and_refresh_run_in_separate_processes_and_replay() {
    let first = sign_in::run(59, Story::Public, true, false, Memory::Checked);
    let second = sign_in::run(59, Story::Public, true, false, Memory::Checked);
    sign_in::assert_replay(&first, &second);
    assert_clients_settle_before_issuer(&first);
    assert_eq!(first.peaks(), second.peaks());
}
#[test]
fn full_issuer_tokens_plans_codes_and_rotations_fit_the_process_heap() {
    drop(sign_in::run(61, Story::Public, false, true, Memory::Checked));
}
#[test]
fn protocol_streams_cover_the_owner_redirect_cancel_close_and_keep_terminals() {
    for story in [
        Story::Paste,
        Story::PasteThenListener,
        Story::ListenerThenPaste,
        Story::Public,
        Story::VaryingTaken,
        Story::TwoPorts,
        Story::Confidential,
        Story::WrongPath,
        Story::WrongHost,
        Story::LongHead,
        Story::CloseWaiting,
        Story::CloseConfidential,
        Story::Cancel,
        Story::Abort,
        Story::NotKept,
        Story::Timeout,
    ] {
        let first = sign_in::protocol(67, story);
        let second = sign_in::protocol(67, story);
        assert_eq!(first, second);
    }
}

#[test]
fn a_varying_port_signs_in_with_its_preferred_port_taken() {
    for protocol in [false, true] {
        if protocol {
            drop(sign_in::protocol(71, Story::VaryingTaken));
        } else {
            drop(sign_in::run(71, Story::VaryingTaken, false, false, Memory::Checked));
        }
    }
}

fn assert_clients_settle_before_issuer(outcome: &skein_world::Outcome<sign_in::Process>) {
    use skein_sim::{Event, Summary};
    let listener = outcome
        .trace
        .iter()
        .find_map(|entry| match entry.event {
            Event::Submit { kind: Summary::Listen { fd, .. }, .. } if entry.pid.raw() == 0 => Some(fd),
            Event::Submit { .. } | Event::Complete { .. } | Event::Fault(_) | Event::Digest(_) => None,
        })
        .expect("the issuer listened");
    let clients_closed = outcome
        .trace
        .iter()
        .enumerate()
        .filter_map(|(at, entry)| {
            (entry.pid.raw() != 0
                && matches!(entry.event, Event::Complete { kind: Summary::Close { .. }, result: Ok(_), .. }))
            .then_some(at)
        })
        .max()
        .expect("the client and browser closed their descriptors");
    let issuer_closed = outcome
        .trace
        .iter()
        .position(|entry| {
            entry.pid.raw() == 0
                && matches!(entry.event,
            Event::Submit { kind: Summary::Close { fd }, .. } if fd == listener)
        })
        .expect("the issuer closed its listener");
    assert!(clients_closed < issuer_closed, "the issuer stays live until all client closes have completed");
}

#[test]
fn required_scopes_declined_then_a_grant_is_refused() {
    for protocol in [false, true] {
        let facts = if protocol {
            sign_in::protocol(79, Story::RequiredScopes).0
        } else {
            let outcome = sign_in::run(79, Story::RequiredScopes, false, false, Memory::Checked);
            sign_in::facts(&outcome.procs).to_vec()
        };
        assert_eq!(
            facts,
            vec![
                sign_in::Fact::Visit,
                sign_in::Fact::Keep(1),
                sign_in::Fact::SignedIn(1),
                sign_in::Fact::Missing(b"write".as_slice().into()),
                sign_in::Fact::Failed(skein_oauth_accounts::Failure::NotGranted),
                sign_in::Fact::Closed,
            ]
        );
    }
}

fn openid_story(story: Story) {
    for protocol in [false, true] {
        let facts = if protocol {
            sign_in::protocol(83, story).0
        } else {
            let outcome = sign_in::run(83, story, false, false, Memory::Checked);
            sign_in::facts(&outcome.procs).to_vec()
        };
        assert_eq!(facts.last(), Some(&sign_in::Fact::Closed), "{story:?}, protocol {protocol}");
    }
}
#[test]
fn signing_in_again_fetches_the_issuers_keys_before_each_visit() {
    openid_story(Story::OpenIdAgain);
}
#[test]
fn a_new_registration_signs_in_with_its_issued_client_id() {
    openid_story(Story::IssuedClientId);
}
#[test]
fn a_sign_in_of_another_subject_fails_and_keeps_nothing() {
    openid_story(Story::AnotherSubject);
}

// skein/docs/foundation/testing-strategy.md, 10: rows carry literal outside facts.
#[test]
fn sign_in_owner_facts_survive_refresh_restart_and_returns() {
    use sign_in::Fact::{Closed, Failed, Granted, Keep, Profile, Refreshed, SignedIn, Visit};
    use sign_in::Return;
    const SIGNED_IN: &[sign_in::Fact] = &[Visit, Keep(1), SignedIn(1), Granted(1), Closed];
    const REFRESHED: &[sign_in::Fact] = &[Visit, Keep(1), SignedIn(1), Granted(1), Keep(2), Granted(2), Closed];
    const PROFILE_REFRESH: &[sign_in::Fact] =
        &[Visit, Keep(1), SignedIn(1), Profile, Granted(1), Keep(2), Refreshed(2), Granted(2), Closed];
    const PROFILE_SILENT: &[sign_in::Fact] =
        &[Visit, Keep(1), SignedIn(1), Profile, Granted(1), Keep(2), Granted(2), Closed];
    const RESTARTED: &[sign_in::Fact] = &[
        Visit,
        Failed(skein_oauth_accounts::Failure::Exchange(skein_oauth::Failure::Cancelled)),
        Visit,
        Keep(1),
        SignedIn(1),
        Granted(1),
        Closed,
    ];
    const TWO_PORTS_SIMULATED: &[sign_in::Fact] = &[
        Visit,
        Visit,
        Failed(skein_oauth_accounts::Failure::Exchange(skein_oauth::Failure::Malformed)),
        Keep(1),
        SignedIn(1),
        Closed,
    ];
    const TWO_PORTS: &[sign_in::Fact] =
        &[Visit, Visit, Keep(1), SignedIn(1), Granted(1), Keep(1), SignedIn(1), Granted(1), Closed];
    const SIGNED_IN_AGAIN: &[sign_in::Fact] =
        &[Visit, Keep(1), SignedIn(1), Granted(1), Visit, Keep(2), SignedIn(2), Granted(2), Closed];
    for (seed, story, faulted, [simulated, protocol]) in [
        (43, Story::Public, false, [REFRESHED; 2]),
        (43, Story::Profile(true), false, [PROFILE_REFRESH; 2]),
        (43, Story::Profile(false), false, [PROFILE_SILENT; 2]),
        (97, Story::Restart, false, [RESTARTED; 2]),
        (73, Story::TwoPorts, false, [TWO_PORTS_SIMULATED, TWO_PORTS]),
        (89, Story::VaryingTaken, true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Back), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Before), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::After), true, [SIGNED_IN_AGAIN; 2]),
        (89, Story::Return(Return::Replace), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Long), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Space), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Scheme), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Empty), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Control), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::NonAscii), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::Bound), true, [SIGNED_IN; 2]),
        (89, Story::Return(Return::WrongPath), true, [SIGNED_IN; 2]),
    ] {
        let outcome = sign_in::run(seed, story, faulted, false, Memory::Checked);
        assert_eq!(sign_in::facts(&outcome.procs), simulated, "{story:?}, simulated");
        assert_eq!(sign_in::protocol(seed, story).0, protocol, "{story:?}, protocol");
    }
}

#[test]
fn a_private_return_sends_the_browser_back_with_303() {
    drop(sign_in::run_private_return(91));
}

// oauth.md, 6.5 bounds owner pastes refused for state.
const STATE_REFUSAL_BOUND: u32 = 4;

const PASTED_AND_GRANTED: &[sign_in::Fact] = &[
    sign_in::Fact::Visit,
    sign_in::Fact::Taken,
    sign_in::Fact::Keep(1),
    sign_in::Fact::SignedIn(1),
    sign_in::Fact::Granted(1),
    sign_in::Fact::Closed,
];

fn assert_paste_story(story: Story, both_tiers: bool, expected: &[sign_in::Fact]) {
    const PASTE_SEED: u64 = 97;
    let outcome = sign_in::run(PASTE_SEED, story, false, false, Memory::Checked);
    assert_eq!(sign_in::facts(&outcome.procs), expected);
    if both_tiers {
        assert_eq!(sign_in::protocol(PASTE_SEED, story).0, expected);
    }
}

#[test]
fn a_public_client_signs_in_with_a_pasted_redirect() {
    assert_paste_story(Story::Paste, true, PASTED_AND_GRANTED);
}

#[test]
fn a_paste_wins_and_the_listener_answers_404() {
    assert_paste_story(Story::PasteThenListener, true, PASTED_AND_GRANTED);
}

#[test]
fn the_listener_wins_and_a_paste_is_not_waiting() {
    use sign_in::Fact::{Closed, Granted, Keep, Refused, SignedIn, Visit};
    assert_paste_story(
        Story::ListenerThenPaste,
        true,
        &[Visit, Keep(1), SignedIn(1), Granted(1), Refused(skein_oauth_accounts::Refusal::NotWaiting), Closed],
    );
}

#[test]
fn a_second_identical_paste_is_refused_as_not_waiting() {
    use sign_in::Fact::{Closed, Granted, Keep, Refused, SignedIn, Taken, Visit};
    assert_paste_story(
        Story::PasteTwice,
        false,
        &[Visit, Taken, Refused(skein_oauth_accounts::Refusal::NotWaiting), Keep(1), SignedIn(1), Granted(1), Closed],
    );
}

#[test]
fn wrong_pasted_addresses_leave_the_sign_in_waiting() {
    use sign_in::Fact::{Closed, Granted, Keep, Refused, SignedIn, Taken, Visit};
    use skein_oauth_accounts::Refusal::Redirect;
    assert_paste_story(
        Story::PasteWrong,
        false,
        &[
            Visit,
            Refused(Redirect), // Host.
            Refused(Redirect), // Port.
            Refused(Redirect), // Path.
            Refused(Redirect), // Fragment.
            Refused(Redirect), // Duplicate state.
            Refused(Redirect), // Past redirect_bytes.
            Taken,
            Keep(1),
            SignedIn(1),
            Granted(1),
            Closed,
        ],
    );
}

#[test]
fn exhausted_paste_state_refusals_still_let_the_listener_sign_in() {
    use sign_in::Fact::{Closed, Granted, Keep, Refused, SignedIn, Visit};
    use skein_oauth_accounts::Refusal::{Full, State};
    assert_paste_story(
        Story::StateRefusals,
        false,
        &[
            Visit,
            Refused(State),
            Refused(State),
            Refused(State),
            Refused(State),
            Refused(Full { bound: STATE_REFUSAL_BOUND }),
            Keep(1),
            SignedIn(1),
            Granted(1),
            Closed,
        ],
    );
}

#[test]
fn another_listener_state_gets_404_then_the_right_one_signs_in() {
    use sign_in::Fact::{Closed, Granted, Keep, SignedIn, Visit};
    assert_paste_story(Story::ListenerWrongState, false, &[Visit, Keep(1), SignedIn(1), Granted(1), Closed]);
}

#[test]
fn an_expired_sign_in_refuses_the_paste_as_not_waiting() {
    use sign_in::Fact::{Closed, Failed, Refused, Visit};
    assert_paste_story(
        Story::PasteExpired,
        false,
        &[
            Visit,
            Failed(skein_oauth_accounts::Failure::Exchange(skein_oauth::Failure::TimedOut)),
            Refused(skein_oauth_accounts::Refusal::NotWaiting),
            Closed,
        ],
    );
}

#[test]
fn a_cancelled_sign_in_refuses_the_paste_as_not_waiting() {
    use sign_in::Fact::{Closed, Failed, Refused, Visit};
    assert_paste_story(
        Story::PasteCancelled,
        false,
        &[
            Visit,
            Failed(skein_oauth_accounts::Failure::Exchange(skein_oauth::Failure::Cancelled)),
            Refused(skein_oauth_accounts::Refusal::NotWaiting),
            Closed,
        ],
    );
}

#[test]
fn a_paste_completes_a_public_sign_in_through_close() {
    use sign_in::Fact::{Closed, Keep, SignedIn, Taken, Visit};
    assert_paste_story(Story::PasteClose, false, &[Visit, Taken, Keep(1), SignedIn(1), Closed]);
}
