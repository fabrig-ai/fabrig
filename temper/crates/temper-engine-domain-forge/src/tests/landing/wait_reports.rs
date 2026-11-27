use super::*;
use jig_domain_core_vocabulary::waits::{Reported, ReportedBound, Thing, Unbounded, WaitKind, WaitObject};
use skein_lib::List;
use temper_client_protocol_wire::waits::{LANDING_JUDGEMENT, LANDING_READY, THING_TASK};

const LANDING: u64 = 61;
const HOLDER: u64 = 62;

#[test]
fn quiet_landing_reports_name_the_change_or_turn_holder_and_clear_for_prepare_and_hold() {
    use policy::{Awaited, Phase};
    let mut d = connected_member_landing();
    let clock = env();
    let mut row = d.landing(LANDING).unwrap().clone();
    let since = clock.wall;
    let change = row.landing.members[0].change;
    let mut holder = row.clone();
    holder.landing.task = HOLDER;
    holder.landing.phase = Phase::Turn { at: since };
    let mut other = d.change(change).unwrap().clone();
    other.task = 52;
    other.change.task = 52;
    other.branch = b"temper/52".as_slice().into();
    other.pull = Some(3);
    d.changes.insert(other.task, other).unwrap();
    holder.landing.members[0].change = 52;
    holder.landing.members[0].pull = 3;
    landing::save(&mut d, &clock, holder, &mut Queue::uncapped());
    for (on, step, name) in [
        (Awaited::Ready { member: 0 }, LANDING_READY, LandingWaitFor::Ready),
        (Awaited::Judgement { member: 0 }, LANDING_JUDGEMENT, LandingWaitFor::Judgement),
    ] {
        row.waiting = Some(on);
        let report = report_row(&mut d, &clock, &row);
        assert_eq!(
            wait_report(&report, LANDING),
            Some(
                [Reported {
                    since,
                    kind: WaitKind::Step,
                    object: WaitObject::Procedure {
                        step,
                        things: List::from_boxed(Box::new([Thing { what: THING_TASK, number: change, owner: None }]))
                    },
                    bound: ReportedBound::None(Unbounded::Behind(change)),
                }]
                .as_slice()
            )
        );
        assert_panel_wait(&d, &clock, LANDING, name, Some(&wait_report(&report, LANDING).unwrap()[0]));
    }
    let turn = Reported {
        since,
        kind: WaitKind::Turn,
        object: WaitObject::Task(HOLDER),
        bound: ReportedBound::None(Unbounded::Behind(HOLDER)),
    };
    row.waiting = Some(Awaited::Turn);
    let report = report_row(&mut d, &clock, &row);
    assert_eq!(wait_report(&report, LANDING), Some([turn.clone()].as_slice()));
    assert_panel_wait(&d, &clock, LANDING, LandingWaitFor::Turn, Some(&wait_report(&report, LANDING).unwrap()[0]));
    row.waiting = Some(Awaited::Prepare);
    assert_eq!(wait_report(&report_row(&mut d, &clock, &row), LANDING), Some([].as_slice()));
    row.waiting = Some(Awaited::Turn);
    assert_eq!(wait_report(&report_row(&mut d, &clock, &row), LANDING), Some([turn].as_slice()));
    row.landing.phase = Phase::Held { was: Box::new(row.landing.phase), why: policy::Hold::Read };
    assert_eq!(wait_report(&report_row(&mut d, &clock, &row), LANDING), Some([].as_slice()));
    row.waiting = Some(Awaited::Release);
    assert_eq!(wait_report(&report_row(&mut d, &clock, &row), LANDING), None);
    assert_panel_wait(&d, &clock, LANDING, LandingWaitFor::Release, None);
}

fn report_row(d: &mut Domain, clock: &Env<Limits>, row: &LandingRow) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    landing::save(d, clock, row.clone(), &mut out);
    crate::room::ask(d, clock, &mut out);
    collected(out)
}

#[test]
fn a_gate_report_tracks_its_runner_wait_then_its_running_deadline() {
    use jig_domain_core_vocabulary::waits::AtBound;
    use skein_forge::api::RunState;
    use temper_client_protocol_wire::waits::{LANDING_RUN, THING_GATE, THING_RUN};
    let (mut d, _, mut run) = clocked_gate();
    let mut clock = env();
    let since = clock.wall;
    let object = WaitObject::Procedure {
        step: LANDING_RUN,
        things: List::from_boxed(Box::new([
            Thing { what: THING_GATE, number: 0, owner: None },
            Thing { what: THING_RUN, number: run.number, owner: None },
        ])),
    };
    let wait = |since, at| Reported {
        kind: WaitKind::Step,
        object: object.clone(),
        since,
        bound: ReportedBound::At { at, then: AtBound::Hold },
    };
    let runner_at = advanced_wall(since, clock.limits.change_policy.runner_wait);
    let (_, report) = poll_clocked_gate_with_report(&mut d, &clock, &run);
    assert_eq!(report.as_deref(), Some([wait(since, runner_at)].as_slice()));
    assert_panel_wait(&d, &clock, LANDING, LandingWaitFor::Run, Some(&report.unwrap()[0]));
    clock = outbox_room::later(&clock, Duration::from_secs(2));
    let (_, report) = poll_clocked_gate_with_report(&mut d, &clock, &run);
    assert_eq!(report, None, "another waiting reading moves neither since nor the runner deadline");
    run.state = RunState::Running;
    let running_at = advanced_wall(clock.wall, Duration::from_nanos(5_000_000_001));
    let (_, report) = poll_clocked_gate_with_report(&mut d, &clock, &run);
    assert_eq!(report.as_deref(), Some([wait(clock.wall, running_at)].as_slice()));
    assert_panel_wait(&d, &clock, LANDING, LandingWaitFor::Run, Some(&report.unwrap()[0]));
}
