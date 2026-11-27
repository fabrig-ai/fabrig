//! The report keeps the revision read when asked, not when it ends (gates.md, 1, Valid at a spec revision).
use super::*;
use skein_forge::api::Answer;
use temper_engine_domain_forge_change as c;

fn gate_step(d: &mut Domain) -> Box<[Request]> {
    fixtures::step_change(d);
    for answer in [Answer::Commit(fixtures::HEAD), Answer::Commit(fixtures::BASE), Answer::Pull(fixtures::open_pull())]
    {
        fixtures::answer_next(d, Ok(answer));
    }
    fixtures::finish_open(d);
    fixtures::answer_next(d, Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }))
}

fn assert_asked(out: &[Request], kind: c::Delegate) {
    let mut asked = false;
    for rq in out {
        if let Request::ChangeDecision { decision: c::Decision::Delegate(delegate), .. } = rq {
            asked |= *delegate == kind;
        }
    }
    assert!(asked, "{out:?}");
}

#[test]
fn a_review_ending_after_amendment_reports_at_its_asked_revision() {
    let (mut d, _) = fixtures::opening();
    let row = d.changes.get_mut(&fixtures::CHANGE).expect("change");
    row.pending = None;
    row.effect = c::EffectResult::None;
    row.delegate = None;
    row.delegate_status = c::Status::Unknown;
    row.change.state = c::State::Gating { head: fixtures::HEAD, asked: Box::new([]) };
    row.change.gates = Box::new([c::Gate {
        number: 9,
        kind: c::GateKind::Agent,
        blocking: true,
        freshness: c::Freshness::Exact,
        eager: false,
    }]);
    let kind = c::Delegate::Gate { number: 9, head: fixtures::HEAD };
    let asked = gate_step(&mut d);
    assert_asked(&asked, kind);
    outputs(&mut d, Event::Delegated { task: fixtures::CHANGE, child: 52, kind, close_at: None });
    let mut revision_output = Queue::uncapped();
    d.refresh_spec_revision(fixtures::CHANGE, 1, &mut revision_output);
    assert_eq!(d.change(fixtures::CHANGE).expect("amended").gate_revision, 0);
    outputs(
        &mut d,
        Event::DelegateResult {
            ended_at: None,
            task: fixtures::CHANGE,
            child: 52,
            status: c::Status::Passed,
            no_change: false,
            failed: false,
            lost: false,
            flake: false,
            no_test: false,
            cancelled: false,
            words: b"approved".as_slice().into(),
        },
    );
    let report = d.change(fixtures::CHANGE).expect("reported").verdicts[0];
    assert_eq!(report.revision, 0);
    assert_eq!(report.status, c::Status::Passed);
    let again = gate_step(&mut d);
    assert_asked(&again, kind);
    let row = d.change(fixtures::CHANGE).expect("asked again");
    assert_eq!(row.gate_revision, 1);
    assert!(row.change.gate_failures.is_empty());
    assert_eq!(row.change.repairs, 0);
}
