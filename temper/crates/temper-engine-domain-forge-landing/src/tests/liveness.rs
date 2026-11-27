//! The verdict after a landing's ordinary decision (temper/docs/design/domain/landing-liveness.md, 2–3).
use super::step::{decide, prefix, stack, stack_facts, stacked};
use crate::{Awaited, Combine, Decision, Hold, Phase, RoundPull, Status, Stepped, Stuck, liveness::check};
use skein_lib::Wall;

const OLDER: u64 = 41;
const TRAIN: Stuck = Stuck::Train { landing: OLDER };

#[test]
fn a_stacked_landing_ends_if_another_landing_holds_its_train() {
    let landing = stack(1);
    let mut facts = stack_facts(1);
    facts.train_holder = Some(OLDER);
    let next = decide(&landing, &facts);
    assert_eq!(next.decision, Decision::End(TRAIN));
    assert_eq!(next.landing, landing, "the unissued prepare changes nothing");
}

#[test]
fn a_stacked_landing_with_a_landed_prefix_holds_if_another_holds_its_train() {
    let mut landing = stack(2);
    landing.members[0].landed = Some(prefix(0));
    let mut facts = stack_facts(2);
    facts.train_holder = Some(OLDER);
    let next = decide(&landing, &facts);
    let why = Hold::Stuck(TRAIN);
    assert_eq!(next.decision, Decision::Hold(why));
    assert_eq!(next.landing.phase, Phase::Held { was: Box::new(landing.phase), why });
    assert_eq!(next.landing.members[0].landed, Some(prefix(0)));
    facts.released = true;
    assert_eq!(decide(&next.landing, &facts), next, "release checks the standing cause again");
}

#[test]
fn a_single_landing_ends_for_a_held_member_it_did_not_remove() {
    let mut landing = stack(1);
    landing.round.branch = Box::new([]);
    let mut facts = stack_facts(1);
    facts.members[0].held = true;
    assert_eq!(decide(&landing, &facts).decision, Decision::End(Stuck::Member { change: 1, held: true }));
}

#[test]
fn an_ended_member_still_in_the_landing_is_not_a_wake() {
    let mut landing = stack(1);
    landing.round.branch = Box::new([]);
    let mut facts = stack_facts(1);
    facts.members[0].cancelled = true;
    let step = Stepped { landing, decision: Decision::Wait { until: None, on: Awaited::Ready { member: 0 } } };
    assert_eq!(check(&step.landing, &step, &facts), Some(Stuck::Member { change: 1, held: false }));
}

#[test]
fn a_wait_without_a_holder_or_deadline_has_no_wake() {
    let mut landing = stack(1);
    landing.round.branch = Box::new([]);
    landing.phase = Phase::Proved { at: Wall::from_nanos(10) };
    let mut facts = stack_facts(1);
    facts.turn_free = true;
    let on = Awaited::Turn;
    let mut step = Stepped { landing, decision: Decision::Wait { until: None, on } };
    assert_eq!(check(&step.landing, &step, &facts), Some(Stuck::Wake { on }));
    step.decision = Decision::Wait { until: Some(Wall::from_nanos(20)), on };
    assert_eq!(check(&step.landing, &step, &facts), None);
}

#[test]
fn a_bad_train_waits_for_an_uncertain_merge_before_the_check_can_end_or_hold_it() {
    let (mut landing, mut facts) = stacked(2);
    landing.phase = Phase::Turn { at: Wall::from_nanos(10) };
    // The first member was proved separately; its suffix still needs preparing.
    landing.members[0].combined = Some(Combine::Squash);
    landing.members[1].candidate = None;
    facts.members[1].candidate = None;
    facts.train_holder = Some(OLDER);
    facts.members[0].merge = Status::Failed;
    let waiting = decide(&landing, &facts);
    assert_eq!(waiting.decision, Decision::Wait { until: None, on: Awaited::Merge });
    assert_eq!(waiting.landing, landing);

    facts.round_pull = RoundPull::Closed;
    assert_eq!(decide(&waiting.landing, &facts).decision, Decision::End(TRAIN));

    facts.round_pull = RoundPull::Merged(prefix(0));
    let record = decide(&waiting.landing, &facts);
    assert_eq!(record.landing.members[0].landed, Some(prefix(0)));
    assert_eq!(record.decision, Decision::Record { member: 0, prefix: prefix(0) });
    facts.members[0].record = Status::Passed;
    let told = decide(&record.landing, &facts);
    assert_eq!(told.decision, Decision::Landed { member: 0, prefix: prefix(0) });
    let held = decide(&told.landing, &facts);
    assert_eq!(held.decision, Decision::Hold(Hold::Stuck(TRAIN)));
    assert!(held.landing.members[0].told, "the landed member was told before the hold");
}

#[test]
fn held_and_finishing_landings_do_not_need_wakes_and_acts_supply_their_own() {
    let mut landing = stack(1);
    landing.phase = Phase::Held { was: Box::new(Phase::Preparing), why: Hold::Claimed { change: 1 } };
    let mut facts = stack_facts(1);
    facts.train_holder = Some(OLDER);
    let step = decide(&landing, &facts);
    assert_eq!(step, Stepped { landing, decision: Decision::Wait { until: None, on: Awaited::Release } });
    let mut step = Stepped { landing: stack(1), decision: Decision::Finish };
    assert_eq!(check(&step.landing, &step, &facts), None);
    // An act skips the wake clause but still checks its members and train.
    facts.train_holder = None;
    step.decision = Decision::None;
    assert_eq!(check(&step.landing, &step, &facts), None);
}
