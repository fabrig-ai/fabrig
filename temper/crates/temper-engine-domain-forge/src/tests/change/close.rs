//! Close job read-back (temper/docs/design/domain/forge/change/repairs-releases.md, 1).
use super::*;
use fixtures::{CHANGE, answer_next, opening_row, step_change};
use skein_forge::api::Error;
use temper_engine_domain_forge_change as c;

#[test]
fn an_outdated_close_ends_on_its_first_job() {
    const JOB: u64 = 71;
    const HEAD: [u8; 32] = [0x11; 32];
    const RESULT: &[u8] = b"{\"outdated\":{\"head\":\"1111111111111111111111111111111111111111111111111111111111111111\",\"why\":\"ancestry\"}}\n";
    const CLOSE_REQUEST_BYTES: u32 = 512; // The fixture sends only the result.
    let mut d = fixtures::adopted_domain();
    d.close_kind(CLOSE_REQUEST_BYTES, u32::try_from(RESULT.len()).unwrap());
    let mut row = opening_row();
    row.close = Some(b"close".as_slice().into());
    row.pending = None;
    row.effect = c::EffectResult::None;
    row.delegate_status = c::Status::Unknown;
    row.change.state = c::State::Producing { requested: true };
    row.change.last_head = None;
    outputs(&mut d, Event::Change { row });

    let at = c::CloseAt { branch: HEAD, landing: fixtures::BASE, asked: true, refused: None, outdated: None };
    outputs(&mut d, Event::CloseDelegated { task: CHANGE, child: JOB, at });
    outputs(
        &mut d,
        Event::DelegateResult {
            task: CHANGE,
            child: JOB,
            ended_at: None,
            status: c::Status::Passed,
            no_change: false,
            flake: false,
            no_test: false,
            failed: false,
            lost: false,
            cancelled: false,
            words: RESULT.into(),
        },
    );

    step_change(&mut d);
    let ended = answer_next(&mut d, Err(Error::Missing));
    let decision =
        Request::ChangeDecision { task: CHANGE, decision: c::Decision::NoChange, entry: None, evidence: None };
    let stored_at = c::CloseAt { outdated: Some(c::CloseOutdated::Ancestry), ..at };
    let mut decided = false;
    let mut saved = false;
    for request in &ended {
        if let Request::CloseDecision { .. } | Request::ChangeDecision { .. } = request {
            assert!(!decided, "one decision for the finished job");
            assert_eq!(request, &decision);
            decided = true;
        }
        if let Request::Save { record: Stored::ChangeHot(hot), .. } = request
            && hot.task == CHANGE
            && let c::State::NoChange { .. } = hot.change.state
        {
            assert_eq!(hot.close_at, Some(stored_at));
            saved = true;
        }
    }
    assert!(decided, "the finished job produced a decision");
    assert!(saved, "the terminal row kept the finding");
}
