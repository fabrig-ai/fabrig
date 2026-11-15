//! A release retires failed evidence and takes its fresh branch (change/repairs.md, A release).
use super::*;
use temper_engine_domain_forge_change::{Delegate, Hold as Cause, State, Status};

const CHILD: u64 = 52;
const FRESH: [u8; 32] = [8; 32];

fn failed(kind: Delegate) -> Domain {
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut row = fixtures::opening_row();
    row.pending = None;
    row.effect = temper_engine_domain_forge_change::EffectResult::None;
    row.pull = Some(1);
    row.delegate = Some((CHILD, kind));
    row.delegate_status = Status::Failed;
    row.producer_failure = Some(b"refused at the saved head".as_slice().into());
    row.merge_result = Some(b"old result".as_slice().into());
    row.change.state =
        State::Held { was: Box::new(State::Resolving { requested: true, base: fixtures::BASE }), why: Cause::Failed };
    outputs(&mut d, Event::Change { row });
    d
}

fn release(d: &mut Domain, revision: u64) -> Box<[Request]> {
    outputs(d, Event::ReleaseChange { task: fixtures::CHANGE, release: Some(TaskRelease { revision, at: env().wall }) })
}

#[test]
fn a_release_at_or_below_the_consumed_revision_changes_nothing() {
    let mut d = failed(Delegate::Resolve { base: fixtures::BASE });
    release(&mut d, 3);
    let mut row = d.change(fixtures::CHANGE).unwrap().clone();
    row.change.state = State::Held { was: Box::new(row.change.state.clone()), why: Cause::Resolutions };
    row.change.released = None;
    row.change.resolutions = 2;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row.clone())) });
    for revision in [3, 2] {
        assert!(release(&mut d, revision).is_empty());
        assert_eq!(d.change(fixtures::CHANGE), Some(&row));
    }
}

#[test]
fn a_repeated_released_step_cannot_lift_a_later_hold() {
    let mut d = failed(Delegate::Resolve { base: fixtures::BASE });
    release(&mut d, 3);
    outputs(&mut d, waiting_step(false, true));
    let mut row = d.change(fixtures::CHANGE).unwrap().clone();
    row.change.state = State::Held { was: Box::new(row.change.state.clone()), why: Cause::Resolutions };
    row.change.released = None;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row.clone())) });
    outputs(&mut d, waiting_step(false, true));
    assert_eq!(d.change(fixtures::CHANGE), Some(&row));
}

#[test]
fn a_failed_delegates_terminal_and_words_are_retired_across_restore() {
    for kind in [
        Delegate::Resolve { base: fixtures::BASE },
        Delegate::Merge { base: fixtures::BASE, from: Some(fixtures::HEAD) },
    ] {
        let mut d = failed(kind);
        let ended = d.change(fixtures::CHANGE).unwrap().clone();
        outputs(
            &mut d,
            Event::DelegateResult {
                task: fixtures::CHANGE,
                child: CHILD,
                ended_at: None,
                status: Status::Passed,
                no_change: false,
                failed: false,
                cancelled: false,
                words: b"duplicate result".as_slice().into(),
            },
        );
        assert_eq!(d.change(fixtures::CHANGE), Some(&ended), "a terminal is taken once, merge jobs included");
        release(&mut d, 1);
        let released = d.change(fixtures::CHANGE).unwrap().clone();
        assert_eq!(released.delegate, None, "no archived terminal can be loaded");
        assert_eq!(released.delegate_status, Status::Unknown);
        assert_eq!(released.producer_failure, None);
        assert_eq!(released.merge_result, None);
        let mut restored = fixtures::connected_domain();
        outputs(&mut restored, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut restored, Event::Restore { record: Stored::Change(Box::new(released.clone())) });
        outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut restored, Event::ReadAfresh);
        outputs(&mut restored, Event::SettleOutbox);
        for domain in [&mut d, &mut restored] {
            outputs(
                domain,
                Event::DelegateResult {
                    task: fixtures::CHANGE,
                    child: CHILD,
                    ended_at: None,
                    status: Status::Failed,
                    no_change: false,
                    failed: true,
                    cancelled: false,
                    words: b"refused at the saved head".as_slice().into(),
                },
            );
            assert_eq!(domain.change(fixtures::CHANGE), Some(&released));
            gather(domain, FRESH);
            assert_eq!(domain.change(fixtures::CHANGE).unwrap().change.state, State::Checking { head: FRESH });
            outputs(domain, Event::Delegated { task: fixtures::CHANGE, child: CHILD + 1, kind });
            let replacement = domain.change(fixtures::CHANGE).unwrap();
            assert_eq!(replacement.delegate, Some((CHILD + 1, kind)));
            assert_eq!(replacement.delegate_status, Status::Unknown);
            assert_eq!(replacement.producer_failure, None);
        }
    }
}

#[test]
fn a_refused_replacement_resolution_can_be_released_again() {
    let kind = Delegate::Resolve { base: fixtures::BASE };
    let mut d = failed(kind);
    release(&mut d, 1);
    outputs(&mut d, Event::DelegateAdmissionRefused { task: fixtures::CHANGE, kind });
    assert_eq!(
        d.change(fixtures::CHANGE).unwrap().change.state,
        State::Held { was: Box::new(State::Resolving { requested: false, base: fixtures::BASE }), why: Cause::Failed }
    );
    release(&mut d, 2);
    gather(&mut d, FRESH);
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.state, State::Checking { head: FRESH });
}

fn gather(d: &mut Domain, head: [u8; 32]) -> bool {
    use skein_forge::api::{Answer, Check, Ci, Op, Read};
    outputs(d, waiting_step(false, false));
    let mut baseline = false;
    for _ in 0_u32..16 {
        if d.steps.is_empty() {
            return baseline;
        }
        let calls = ready(d);
        let mut answered = false;
        for request in calls {
            let Request::Call { call, op: Op::Read(read), .. } = request else { continue };
            answered = true;
            let answer = match read {
                Read::Branch { branch } => {
                    Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { head })
                }
                Read::Pull { .. } | Read::PullFor { .. } => {
                    let mut pull = fixtures::open_pull();
                    pull.commit = head;
                    pull.ci = Ci::Pending;
                    Answer::Pull(pull)
                }
                Read::Checks { commit } => {
                    fixtures::checks(if commit == head { Check::Pending } else { Check::Passed })
                }
                Read::Commits { .. } => Answer::Commits { commits: Box::new([]), more: false },
                Read::Reviews { .. } => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
                Read::Compare { before, after } => {
                    Answer::Compare { before, after, files: Box::new([]), ahead: 1, behind: 0, changed: 1 }
                }
                Read::File { .. } => {
                    fixtures::answer(d, call, Err(skein_forge::api::Error::Missing));
                    continue;
                }
                other @ (Read::Locate { .. }
                | Read::Items { .. }
                | Read::Item { .. }
                | Read::Statuses { .. }
                | Read::Remarks { .. }
                | Read::Branches { .. }
                | Read::PullFiles { .. }
                | Read::Job { .. }
                | Read::JobState { .. }
                | Read::Protection { .. }
                | Read::Settings
                | Read::Collaborators { .. }
                | Read::Permission { .. }
                | Read::Runs { .. }
                | Read::Run { .. }
                | Read::Tree { .. }
                | Read::Artifact { .. }) => panic!("release gather: {other:?}"),
            };
            for request in fixtures::answer(d, call, Ok(answer)) {
                if let Request::Save { record: Stored::Client(skein_forge_client::Stored::Live(row)), .. } = request {
                    baseline |= row.pushed == Some(head);
                }
            }
        }
        assert!(answered, "the gather has a read: {:?}", d.steps);
    }
    panic!("the gather must decide");
}

#[test]
fn a_failed_resolutions_release_takes_the_clients_drift_baseline() {
    let mut d = failed(Delegate::Resolve { base: fixtures::BASE });
    let name = Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(Box::new([b"temper".as_slice().into(), b"51".as_slice().into()])),
    };
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: name.clone(),
                task: fixtures::CHANGE,
                writer: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        },
    );
    outputs(&mut d, Event::Names { task: fixtures::CHANGE, resources: Box::new([name.clone()]) });
    fixtures::answer_next(&mut d, Ok(skein_forge::api::Answer::Commit(fixtures::HEAD)));
    outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Pushed {
            resource: skein_forge_client::Resource {
                repository: REPO,
                what: skein_forge_client::What::Branch(b"temper/51".as_slice().into()),
            },
            commit: fixtures::HEAD,
        }),
    );
    release(&mut d, 1);
    assert!(gather(&mut d, FRESH), "the fresh head becomes the client's pushed baseline");
    assert_eq!(d.change(fixtures::CHANGE).unwrap().change.last_head, Some(FRESH), "{:?}", d.change(fixtures::CHANGE));
    assert_eq!(d.branch_head(&name), Some(FRESH));
    gather(&mut d, FRESH);
    let row = d.change(fixtures::CHANGE).unwrap();
    assert_eq!(row.drift, None);
    assert_eq!(row.change.state, State::Checking { head: FRESH });
}
