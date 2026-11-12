//! Wake-only session ends (jig/docs/design/domain/tasks/standing.md, 5).
use crate::{Reasons, Session, SessionEnd, SessionWake, context_bound, quiet_span, session_end};
use skein_lib::{Duration, Wall};

const SESSION: Session = Session { first: 7, reasons: Reasons::FIRST, last: Wall::from_nanos(60_000_000_000) };

fn wake() -> SessionWake {
    SessionWake {
        talk: crate::TalkState::None,
        at: Wall::from_nanos(
            SESSION.last.as_nanos().checked_add(Duration::from_secs(1).as_nanos()).expect("small wake time"),
        ),
        span: quiet_span(300_000),
        bytes: 100,
        bound: context_bound(1_000),
        kinds: Reasons::EMPTY,
        offered: Reasons::EMPTY,
        resumable: true,
    }
}

#[test]
fn quiet_ends_only_past_nine_tenths_and_zero_always_ends() {
    assert_eq!(quiet_span(300_000), Duration::from_secs(270));
    assert_eq!(session_end(SESSION, SessionWake { at: Wall::EPOCH, ..wake() }), None);
    for (seconds, end) in [(269, None), (270, None), (271, Some(SessionEnd::Quiet))] {
        assert_eq!(
            session_end(
                SESSION,
                SessionWake {
                    at: Wall::from_nanos(
                        SESSION
                            .last
                            .as_nanos()
                            .checked_add(Duration::from_secs(seconds).as_nanos())
                            .expect("small wake time")
                    ),
                    ..wake()
                }
            ),
            end
        );
    }
    assert_eq!(
        session_end(SESSION, SessionWake { span: quiet_span(0), at: Wall::EPOCH, ..wake() }),
        Some(SessionEnd::Quiet)
    );
    assert_eq!(
        session_end(
            SESSION,
            SessionWake {
                at: Wall::from_nanos(
                    SESSION.last.as_nanos().checked_add(Duration::from_secs(271).as_nanos()).expect("small wake time")
                ),
                bytes: 2_001,
                kinds: Reasons::RESULT,
                resumable: false,
                ..wake()
            }
        ),
        Some(SessionEnd::Quiet),
        "quiet wins before every other end"
    );
}

#[test]
fn bound_ends_only_past_half_the_four_byte_token_window() {
    assert_eq!(context_bound(1_000), 2_000);
    assert_eq!(session_end(SESSION, SessionWake { bytes: 2_000, ..wake() }), None);
    assert_eq!(session_end(SESSION, SessionWake { bytes: 2_001, ..wake() }), Some(SessionEnd::Bound));
    assert_eq!(
        session_end(SESSION, SessionWake { bytes: 2_001, kinds: Reasons::RESULT, resumable: false, ..wake() }),
        Some(SessionEnd::Bound),
        "bound wins before unoffered and fresh"
    );
}

#[test]
fn unoffered_ends_only_for_a_waking_kind_the_session_does_not_offer() {
    assert_eq!(session_end(SESSION, wake()), None, "always-offered kinds need no part");
    assert_eq!(session_end(SESSION, SessionWake { kinds: Reasons::RESULT, ..wake() }), Some(SessionEnd::Unoffered));
    assert_eq!(session_end(SESSION, SessionWake { kinds: Reasons::RESULT, offered: Reasons::RESULT, ..wake() }), None);
    assert_eq!(
        session_end(SESSION, SessionWake { kinds: Reasons::RESULT, resumable: false, ..wake() }),
        Some(SessionEnd::Unoffered)
    );
}

#[test]
fn fresh_ends_only_when_the_run_cannot_resume() {
    assert_eq!(session_end(SESSION, wake()), None);
    assert_eq!(session_end(SESSION, SessionWake { resumable: false, ..wake() }), Some(SessionEnd::Fresh));
}

#[test]
fn a_talk_suppresses_quiet_but_left_precedes_every_wake_end() {
    use crate::TalkState;
    let at = Wall::from_nanos(SESSION.last.as_nanos() + Duration::from_secs(600).as_nanos());
    assert_eq!(session_end(SESSION, SessionWake { at, ..wake() }), Some(SessionEnd::Quiet));
    assert_eq!(session_end(SESSION, SessionWake { at, talk: TalkState::Open, ..wake() }), None);
    assert_eq!(
        session_end(SESSION, SessionWake { at, talk: TalkState::Open, bytes: 2_001, ..wake() }),
        Some(SessionEnd::Bound)
    );
    assert_eq!(
        session_end(SESSION, SessionWake { at, talk: TalkState::Lapsed, bytes: 2_001, resumable: false, ..wake() }),
        Some(SessionEnd::Left)
    );
}

#[test]
fn a_talk_lapses_only_past_thirty_minutes_from_the_later_last() {
    use crate::{TALK_LAPSE, Talk, TalkState, talk_state};
    assert_eq!(TALK_LAPSE, Duration::from_secs(1_800));
    let talk = Talk { party: 1, opened: Wall::EPOCH, last: Wall::EPOCH };
    let boundary = Wall::from_nanos(SESSION.last.as_nanos() + TALK_LAPSE.as_nanos());
    assert_eq!(talk_state(Some(talk), Some(SESSION), boundary), TalkState::Open);
    let at = Wall::from_nanos(SESSION.last.as_nanos() + Duration::from_secs(31 * 60).as_nanos());
    let state = talk_state(Some(talk), Some(SESSION), at);
    assert_eq!(state, TalkState::Lapsed);
    assert_eq!(session_end(SESSION, SessionWake { at, talk: state, ..wake() }), Some(SessionEnd::Left));
    let talk = Talk { last: boundary, ..talk };
    assert_eq!(talk_state(Some(talk), Some(SESSION), at), TalkState::Open);
    assert_eq!(talk_state(None, Some(SESSION), at), TalkState::None);
}
