//! Removing, renumbering or editing a live follow check (temper/docs/design/domain/forge/follows.md, 6).
use super::*;
use api::{Answer, Error, Op, Read, Write};
use fixtures::{BASE, CHANGE, HEAD, OPEN};
use skein_forge_client::{Entry, Made, Outcome, RecoveryClock};

const OLD: u32 = 1;
const NEW: u32 = 2;
const DISPATCH: u64 = OPEN + 1;
const NEXT: u64 = DISPATCH + 1;
const REVISION: u64 = 2;
const WRITER: u64 = 7; // The identity from fixtures::connect.
const WORKFLOW: &[u8] = b".forgejo/workflows/edited.yml";
const MADE: Outcome = Outcome::Made { made: Made::Set, found: true };
const FAILED_DISPATCH: Outcome = Outcome::Failed(Error::Missing);
// Leave room for background workflow discovery alongside the ordinary change reads.
const GATHER_READS: u32 = 12;

fn remove(d: &mut Domain) {
    follow_setting(d, REVISION, OLD, None);
}

fn restart(d: &mut Domain) -> Box<[Request]> {
    let mut out = outputs(d, Event::Restored { clock: RecoveryClock::Monotonic }).into_vec();
    out.extend(outputs(d, Event::CatchUp));
    out.into_boxed_slice()
}

fn freshen(d: &mut Domain) {
    for _ in 0..LIMITS.client.pending {
        if domain::fresh_reads_in(d) {
            return;
        }
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) if branch.as_ref() == b"main" => Answer::Commit(BASE),
            Op::Read(Read::Items { .. }) => empty_items(),
            Op::Read(_) | Op::Write(_) => panic!("unexpected startup call: {op:?}"),
        };
        fixtures::answer(d, call, Ok(answer));
    }
    panic!("initial branch reads should finish");
}

fn row(d: &Domain) -> &ChangeRow {
    d.change(CHANGE).expect("follow change")
}

fn dispatching() -> Domain {
    let (mut d, mut row) = configured();
    fixtures::connect(&mut d);
    restart(&mut d);
    freshen(&mut d);
    keyed(&mut d, &mut row);
    row.pending = None;
    row.effect = c::EffectResult::None;
    outputs(&mut d, Event::Change { row });
    let Gather::Decision(out) = gather(&mut d, DISPATCH) else { panic!("the keyed check should dispatch") };
    assert!(out.iter().any(|r| matches!(r, Request::Save { record: Stored::Entry { entry, .. }, .. }
        if entry.number == DISPATCH && matches!(entry.effect.write, Write::Dispatch { .. }))));
    let flight = self::row(&d).follow_check.as_ref().unwrap().flight.as_ref().unwrap();
    assert_eq!((flight.entry, flight.waiting), (DISPATCH, true));
    d
}

fn cold(live: &Domain, renumbered: Option<u32>) -> Domain {
    let mut d = fixtures::connected_domain();
    for provider in [REPO, SOURCE] {
        outputs(&mut d, Event::Restore { record: Stored::Repository(live.repository(provider).unwrap().clone()) });
    }
    if let Some(number) = renumbered {
        follow_setting(&mut d, REVISION, number, live.follow(PROJECT, OLD).cloned());
    }
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(row(live).clone())) });
    let entry = live.entries.get(&DISPATCH).unwrap().entry.clone();
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: false } });
    d
}

// Drive the change's ordinary gather, leaving a check's own read in flight.
#[derive(Debug)]
enum Gather {
    Decision(Box<[Request]>),
    Read(Token, Read),
}

#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture refuses any read outside this gather")]
fn gather(d: &mut Domain, number: u64) -> Gather {
    let mut step = fixtures::change_step();
    let Event::StepChange { entry, .. } = &mut step else { unreachable!("change fixture") };
    *entry = number;
    let mut out = outputs(d, step);
    for _ in 0..GATHER_READS {
        if out.iter().any(|r| matches!(r, Request::ChangeDecision { task: CHANGE, .. })) {
            return Gather::Decision(out);
        }
        let (call, op) = fixtures::next_at(d, &env());
        let Op::Read(read) = op else { panic!("unexpected write: {op:?}") };
        let answer = match read {
            Read::Branch { branch } => Answer::Commit(if branch.as_ref() == b"temper/51" { HEAD } else { BASE }),
            Read::Pull { .. } | Read::PullFor { .. } => Answer::Pull(fixtures::open_pull()),
            Read::Items { .. } => empty_items(),
            Read::Checks { .. } => fixtures::checks(api::Check::Passed),
            Read::Commits { .. } => Answer::Commits { commits: Box::new([]), more: false },
            Read::Compare { before, after } => {
                Answer::Compare { before, after, files: Box::new([]), ahead: 1, behind: 0, changed: 0 }
            }
            Read::Reviews { .. } => Answer::Reviews { reviews: Box::new([]), more: false, cut: false },
            Read::File { path, .. }
                if [b".forgejo/workflows".as_slice(), b".github/workflows"].contains(&path.as_ref()) =>
            {
                fixtures::answer(d, call, Err(Error::Missing));
                continue;
            }
            Read::Runs { .. } | Read::Run { .. } | Read::Tree { .. } | Read::Artifact { .. } => {
                return Gather::Read(call, read);
            }
            _ => panic!("unexpected gather read: {read:?}"),
        };
        out = fixtures::answer(d, call, Ok(answer));
    }
    panic!("change should decide or read its check: {out:?}");
}

fn empty_items() -> Answer {
    Answer::Items { cut: false, items: Box::new([]), more: false, now: env().now }
}

fn settle(d: &mut Domain, outcome: Outcome) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    domain::scripted_outcome(d, &env(), DISPATCH, CHANGE, outcome, &mut out);
    collected(out)
}

fn reading(passed: bool) -> (Domain, Token, api::Run) {
    let mut d = dispatching();
    fixtures::send_write(&mut d, DISPATCH);
    settle(&mut d, MADE);
    let mut run = workflow(row(&d), passed);
    run.actor = WRITER;
    let Gather::Read(call, Read::Runs { .. }) = gather(&mut d, NEXT) else { panic!("find the dispatched run") };
    fixtures::answer(&mut d, call, Ok(Answer::Runs { runs: Box::new([run.clone()]), more: false }));
    let gathered = gather(&mut d, NEXT);
    let Gather::Read(call, Read::Run { run: RUN }) = gathered else { panic!("read the found run: {gathered:?}") };
    (d, call, run)
}

fn assert_quiet(out: &[Request]) {
    for request in out {
        assert!(
            !matches!(
                request,
                Request::Call { .. }
                    | Request::Save { record: Stored::Entry { .. } | Stored::Receipt { .. } | Stored::Follow(_), .. }
                    | Request::ChangeDecision { decision: c::Decision::Effect(c::Effect::FollowCheck { .. }), .. }
            ),
            "unexpected check activity: {request:?}"
        );
    }
}

fn assert_unjudged(d: &Domain, out: &[Request]) {
    assert_quiet(out);
    assert!(out.contains(&Request::WakeProcedure { task: CHANGE }), "{out:?}");
    assert!(d.steps.is_empty(), "no follow-up read");
    assert!(d.entries.is_empty(), "no follow-up write");
    assert!(d.landing.receipts.is_empty());
    assert!(row(d).verdicts.is_empty());
    assert!(row(d).gate_remarks.is_empty());
    let check = row(d).follow_check.as_ref().expect("next step drops the old check");
    assert_eq!(check.faults, 0);
    assert!(check.held.is_none());
    assert!(check.flight.as_ref().is_none_or(|flight| flight.fault.is_none()));
}

fn assert_retired(d: &Domain, out: &[Request], outcome: Outcome) {
    assert_unjudged(d, out);
    assert!(out.contains(&Request::Erase { key: Key::Entry(DISPATCH) }));
    assert!(out.iter().any(|r| matches!(r, Request::Outcome { entry: DISPATCH, task: CHANGE, outcome: got, .. }
        if *got == outcome)));
    assert!(row(d).follow_check.as_ref().unwrap().flight.as_ref().is_none_or(|flight| !flight.waiting));
    assert_eq!(row(d).pending, None);
}

fn after_restart(renumbered: Option<u32>) {
    let live = dispatching();
    for outcome in [MADE, FAILED_DISPATCH] {
        let mut d = cold(&live, renumbered);
        assert_quiet(&restart(&mut d));
        assert!(check::in_flight(&d, row(&d)).is_none());
        let out = settle(&mut d, outcome);
        assert_retired(&d, &out, outcome);
        freshen(&mut d);
        let Gather::Decision(_) = gather(&mut d, NEXT) else { panic!("the old check cannot own another read") };
        assert_dropped(row(&d), row(&live).follows.as_ref());
    }
}

fn assert_dropped(row: &ChangeRow, follows: Option<&ChangeFollows>) {
    assert!(row.follow_check.is_none());
    assert!(!row.change.gates.iter().any(|gate| gate.kind == c::GateKind::Follow));
    assert!(!row.verdicts.iter().any(|verdict| verdict.number == c::FOLLOW_CHECK_GATE));
    assert_eq!(row.follows.as_ref(), follows);
}

#[test]
fn a_removed_follows_dispatch_settles_on_restart_without_a_check() {
    after_restart(None);
}

#[test]
fn a_renumbered_follows_dispatch_is_not_the_new_follows() {
    after_restart(Some(NEW));
}

#[test]
fn a_run_answer_after_removal_does_not_judge_the_check() {
    let (mut d, call, run) = reading(true);
    remove(&mut d);
    let out = fixtures::answer(&mut d, call, Ok(Answer::Run(run)));
    assert_unjudged(&d, &out);
}

#[test]
fn a_report_answer_after_removal_does_not_judge_the_check() {
    let (mut d, call, run) = reading(false);
    fixtures::answer(&mut d, call, Ok(Answer::Run(run)));
    let (call, op) = fixtures::next_call(&mut d);
    assert!(matches!(op, Op::Read(Read::Artifact { run: RUN, .. })));
    remove(&mut d);
    let out = fixtures::answer(&mut d, call, Ok(artifact_answer(FAILED)));
    assert_unjudged(&d, &out);
}

fn cancel(d: &mut Domain) {
    remove(d);
    let out = outputs(
        d,
        Event::Ask(contract::Ask::Effects(contract::effects::Ask::Close {
            task: CHANGE,
            root: CHANGE,
            ending: contract::ops::Ending::Cancelled {
                by: contract::ops::Party::Person(WRITER),
                at: env().wall,
                reason: b"cancelled".as_slice().into(),
                result: None,
            },
        })),
    );
    assert_retired(d, &out, Outcome::Withdrawn);
    assert!(out.contains(&Request::Up(contract::Up::Effects(contract::effects::Up::Closed { task: CHANGE }))));
}

#[test]
fn cancelling_a_removed_follows_dispatch_withdraws_without_a_retry() {
    cancel(&mut dispatching());
}

#[test]
fn cancellation_retires_an_obsolete_dispatch_with_an_outbox_read_pending() {
    let mut d = dispatching();
    outputs(&mut d, Event::Committed { entry: DISPATCH });
    let (call, op) = fixtures::next_call(&mut d);
    assert!(matches!(op, Op::Read(Read::Items { .. })));
    cancel(&mut d);
    assert_quiet(&fixtures::answer(&mut d, call, Ok(empty_items())));
}

#[test]
fn an_edited_check_discards_the_old_answer_and_rekeys() {
    let (mut d, call, run) = reading(true);
    let old_key = check::receipt_key(&d, row(&d));
    let mut follow = d.follow(PROJECT, OLD).unwrap().clone();
    follow.check.workflow = WORKFLOW.into();
    follow_setting(&mut d, REVISION, OLD, Some(follow));
    assert!(check::in_flight(&d, row(&d)).is_none());
    // A wake coalesced with the read does not let the old step start the replacement.
    fixtures::step_change(&mut d);
    let out = fixtures::answer(&mut d, call, Ok(Answer::Run(run)));
    assert_unjudged(&d, &out);
    assert!(d.receipt(&old_key).is_none());
    assert!(d.follow_row(PROJECT, OLD).unwrap().parity.is_none());
    let Gather::Read(call, Read::Tree { object: HEAD, page: 1 }) = gather(&mut d, NEXT) else {
        panic!("the edited check must compute its own key");
    };
    let edited = row(&d).follow_check.as_ref().unwrap();
    assert_eq!(edited.definition.as_ref(), b"\x00\x1d.forgejo/workflows/edited.yml\x00\x07results\x01");
    assert_eq!(edited.key, None);
    fixtures::answer(&mut d, call, Ok(tree()));
    let Entry { effect, .. } = &d.entries.get(&NEXT).expect("the edited check dispatches").entry;
    assert!(matches!(&effect.write, Write::Dispatch { workflow, .. } if workflow.as_ref() == WORKFLOW));
    assert_ne!(check::receipt_key(&d, row(&d)), old_key);
}

#[test]
fn removing_a_judged_check_drops_its_verdict_not_its_receipt() {
    let (mut d, call, run) = reading(true);
    fixtures::answer(&mut d, call, Ok(Answer::Run(run)));
    let provenance = row(&d).follows.clone();
    let key = check::receipt_key(&d, row(&d));
    let receipt = d.receipt(&key).unwrap().clone();
    remove(&mut d);
    let Gather::Decision(out) = gather(&mut d, NEXT) else { panic!("the removed check cannot read again") };
    assert_quiet(&out);
    assert_dropped(row(&d), provenance.as_ref());
    assert_eq!(d.receipt(&key), Some(&receipt));
}
