//! Step tests of blame.md, 2–5, with ledgers and observations built by hand.

use skein_lib::Time;
use skein_world::faults::{Counter, FaultLedger, Kind, Struck};
use temper_blame::{
    BASE_RED, CANCELLED, Class, LOST, MISSING_REPORT, Names, Observed, RUNNER_WAIT, Reaction, SET_UP, SILENT, STARVED,
    WRONG_TREE, class, judge,
};

use temper_faults::forge::LATE;

const AT: Time = Time::from_nanos(20);
const START: Time = Time::from_nanos(10);
const SEED: u64 = 42;

fn run(id: &str) -> Struck {
    Struck { gate_run: Some(id.into()), ..Struck::default() }
}

fn candidate(id: &str) -> Struck {
    Struck { candidate: Some(id.into()), ..Struck::default() }
}

fn base(id: &str) -> Struck {
    Struck { base: Some(id.into()), ..Struck::default() }
}

fn seen(reaction: Reaction) -> Observed {
    Observed { at: AT, reaction }
}

fn retry(gate: &str, run: &str, bound: u64) -> Observed {
    seen(Reaction::GateRetry { gate: gate.into(), run: run.into(), bound })
}

fn repair() -> Observed {
    seen(Reaction::Repair { change: "task 12".into(), candidate: "ab12".into(), base: "cd34".into() })
}

fn base_repair() -> Observed {
    seen(Reaction::BaseRepair { base: "cd34".into() })
}

fn hold(names: Names) -> Observed {
    seen(Reaction::Hold {
        object: "task 12".into(),
        names,
        run: Some("7".into()),
        candidate: Some("ab12".into()),
        base: Some("cd34".into()),
    })
}

#[test]
fn kinds_map_to_gate_classes_and_other_kinds_do_not() {
    for (kind, name, expected) in [
        (SET_UP, "set_up", Class::Runner),
        (LOST, "lost", Class::Runner),
        (RUNNER_WAIT, "runner_wait", Class::Runner),
        (SILENT, "silent", Class::Runner),
        (CANCELLED, "cancelled", Class::Runner),
        (MISSING_REPORT, "missing_report", Class::Runner),
        (STARVED, "starved", Class::Starved),
        (WRONG_TREE, "wrong_tree", Class::Change),
        (BASE_RED, "base_red", Class::Base),
    ] {
        assert_eq!(kind, Kind { family: "ci", name });
        assert_eq!(class(kind), Some(expected), "{kind:?}");
    }
    for (kind, expected) in
        [(Kind { family: "ci", name: "rerun" }, None), (LATE, None), (Kind { family: "other", name: "lost" }, None)]
    {
        assert_eq!(class(kind), expected, "{kind:?}");
    }
}

#[test]
fn retry_counts_each_runner_or_starved_fault_even_after_it_stopped() {
    let mut ledger = FaultLedger::new();
    let lost = ledger.inject(LOST, START, run("7"));
    ledger.stop(lost, START).expect("an instant's lost job");
    let starved = ledger.inject(STARVED, START, run("7"));
    let unrelated = ledger.inject(LOST, START, run("8"));
    let future = ledger.inject(LOST, Time::from_nanos(21), run("7"));
    assert_eq!(judge(&mut ledger, &[retry("temper/widget gate.yml@k1", "7", 2)], SEED), Vec::<String>::new());
    assert_eq!(ledger.get(lost).count(Counter::GateRetries), 1);
    assert_eq!(ledger.get(starved).count(Counter::GateRetries), 1);
    assert_eq!(ledger.get(lost).count(Counter::Repairs), 0);
    assert_eq!(ledger.get(unrelated).count(Counter::GateRetries), 0);
    assert_eq!(ledger.get(future).count(Counter::GateRetries), 0);
}

#[test]
fn retry_cannot_blame_a_wrong_tree() {
    let mut ledger = FaultLedger::new();
    let wrong = ledger.inject(WRONG_TREE, START, run("7"));
    let findings = judge(&mut ledger, &[retry("gate", "7", 2)], SEED);
    assert_eq!(findings.len(), 1);
    assert!(findings.first().expect("finding").ends_with("struck by ci/wrong_tree"));
    assert_eq!(ledger.get(wrong).count(Counter::GateRetries), 0);
}

#[test]
fn retry_bounds_count_per_gate_not_run_or_other_gate() {
    let mut ledger = FaultLedger::new();
    for id in ["7", "8", "9"] {
        ledger.inject(LOST, START, run(id));
    }
    let findings =
        judge(&mut ledger, &[retry("gate a", "7", 1), retry("gate b", "8", 1), retry("gate a", "9", 1)], SEED);
    assert_eq!(findings.len(), 1);
    assert_eq!(
        findings.first().expect("past the bound"),
        "blame: gate retry of gate a at Time(20), seed 42: gate retries exceed bound; struck by ci/lost"
    );
}

#[test]
fn one_runner_fault_does_not_explain_two_retries() {
    let mut ledger = FaultLedger::new();
    let fault = ledger.inject(LOST, START, run("7"));
    let findings = judge(&mut ledger, &[retry("gate", "7", 2), retry("gate", "7", 2)], SEED);
    assert_eq!(findings.len(), 1);
    assert!(findings.first().expect("repeated fault").contains("more than one reaction"));
    assert_eq!(ledger.get(fault).count(Counter::GateRetries), 2);
}

#[test]
fn repair_counts_only_wrong_trees_on_its_candidate() {
    let mut ledger = FaultLedger::new();
    let wrong = ledger.inject(WRONG_TREE, START, candidate("ab12"));
    let another = ledger.inject(WRONG_TREE, START, candidate("ab12"));
    let base_red = ledger.inject(BASE_RED, START, base("cd34"));
    assert_eq!(judge(&mut ledger, &[repair()], SEED), Vec::<String>::new());
    assert_eq!(ledger.get(wrong).count(Counter::Repairs), 1);
    assert_eq!(ledger.get(another).count(Counter::Repairs), 1);
    assert_eq!(ledger.get(base_red).count(Counter::Repairs), 0);
    assert_eq!(ledger.get(wrong).count(Counter::GateRetries), 0);
}

#[test]
fn repair_without_a_wrong_candidate_names_what_struck() {
    for (kind, struck, expected) in [
        (BASE_RED, base("cd34"), "ci/base_red"),
        (WRONG_TREE, base("cd34"), "ci/wrong_tree"),
        (LOST, candidate("ab12"), "ci/lost"),
        (WRONG_TREE, candidate("other"), "nothing"),
    ] {
        let mut ledger = FaultLedger::new();
        let fault = ledger.inject(kind, START, struck);
        let findings = judge(&mut ledger, &[repair()], SEED);
        assert_eq!(findings.len(), 1, "{kind:?}: {findings:?}");
        assert!(findings.first().expect("wrong cause").ends_with(&format!("struck by {expected}")));
        assert_eq!(ledger.get(fault).count(Counter::Repairs), 0);
    }
}

#[test]
fn one_wrong_tree_does_not_explain_a_second_repair_even_of_another_change() {
    let mut ledger = FaultLedger::new();
    let wrong = ledger.inject(WRONG_TREE, START, candidate("ab12"));
    let second = seen(Reaction::Repair { change: "task 13".into(), candidate: "ab12".into(), base: "ef56".into() });
    let findings = judge(&mut ledger, &[repair(), second], SEED);
    assert_eq!(findings.len(), 1);
    assert!(findings.first().expect("second repair").starts_with("blame: repair of task 13"));
    assert_eq!(ledger.get(wrong).count(Counter::Repairs), 2);
}

#[test]
fn base_repair_needs_the_bases_own_red_and_counts_once() {
    let mut ledger = FaultLedger::new();
    let fault = ledger.inject(BASE_RED, START, base("cd34"));
    assert_eq!(judge(&mut ledger, &[base_repair()], SEED), Vec::<String>::new());
    assert_eq!(ledger.get(fault).count(Counter::Repairs), 1);
    assert_eq!(judge(&mut ledger, &[base_repair()], SEED).len(), 1);
    for (kind, struck) in [(BASE_RED, base("other")), (WRONG_TREE, base("cd34")), (BASE_RED, candidate("cd34"))] {
        let mut ledger = FaultLedger::new();
        let fault = ledger.inject(kind, START, struck);
        assert_eq!(judge(&mut ledger, &[base_repair()], SEED).len(), 1, "{kind:?}");
        assert_eq!(ledger.get(fault).count(Counter::Repairs), 0);
    }
    assert_eq!(judge(&mut FaultLedger::new(), &[base_repair()], SEED).len(), 1);
}

#[test]
fn holds_name_the_class_that_struck_without_spending_a_counter() {
    for (kind, struck, names, wrong_names) in [
        (LOST, run("7"), Names::Runner, Names::Change),
        (STARVED, run("7"), Names::Runner, Names::Base),
        (WRONG_TREE, candidate("ab12"), Names::Change, Names::Runner),
        (BASE_RED, base("cd34"), Names::Base, Names::Change),
    ] {
        let mut ledger = FaultLedger::new();
        let fault = ledger.inject(kind, START, struck);
        assert_eq!(judge(&mut ledger, &[hold(names)], SEED), Vec::<String>::new());
        let findings = judge(&mut ledger, &[hold(wrong_names)], SEED);
        assert_eq!(findings.len(), 1, "{kind:?}");
        assert!(findings.first().expect("wrong blame").contains(&format!("{}/{}", kind.family, kind.name)));
        assert_eq!(ledger.get(fault).count(Counter::GateRetries), 0);
        assert_eq!(ledger.get(fault).count(Counter::Repairs), 0);
    }
    assert_eq!(judge(&mut FaultLedger::new(), &[hold(Names::Runner)], SEED).len(), 1);
}

#[test]
fn two_fault_control_retries_the_lost_job_and_repairs_the_wrong_tree_not_the_job() {
    let mut ledger = FaultLedger::new();
    let wrong = ledger.inject(WRONG_TREE, START, candidate("ab12"));
    let lost = ledger.inject(LOST, START, run("7"));
    assert_eq!(judge(&mut ledger, &[retry("gate", "7", 1), repair()], SEED), Vec::<String>::new());
    assert_eq!(ledger.get(lost).count(Counter::GateRetries), 1);
    assert_eq!(ledger.get(lost).count(Counter::Repairs), 0);
    assert_eq!(ledger.get(wrong).count(Counter::Repairs), 1);
    assert_eq!(ledger.get(wrong).count(Counter::GateRetries), 0);

    let mut twin = FaultLedger::new();
    twin.inject(WRONG_TREE, START, candidate("ab12"));
    twin.inject(LOST, START, run("7"));
    // The policy substitutes a repair for the lost-job retry, then repairs
    // the candidate's own red: two repairs of the same wrong tree.
    let findings = judge(&mut twin, &[repair(), repair()], SEED);
    assert_eq!(findings.len(), 1);
    assert!(findings.first().expect("wrong-cause twin").contains("more than one reaction"));
}

#[test]
fn red_base_control_repairs_the_base_and_pauses_the_green_change_not_repairs_it() {
    let mut ledger = FaultLedger::new();
    let red = ledger.inject(BASE_RED, START, base("cd34"));
    assert_eq!(judge(&mut ledger, &[base_repair(), hold(Names::Base)], SEED), Vec::<String>::new());
    assert_eq!(ledger.get(red).count(Counter::Repairs), 1);
    assert_eq!(ledger.get(red).count(Counter::GateRetries), 0);

    let mut twin = FaultLedger::new();
    twin.inject(BASE_RED, START, base("cd34"));
    let findings = judge(&mut twin, &[repair()], SEED);
    assert_eq!(findings.len(), 1);
    assert!(findings.first().expect("green change repaired").ends_with("struck by ci/base_red"));
}

#[test]
fn findings_name_reaction_object_time_seed_and_all_striking_kinds_in_observed_order() {
    let mut ledger = FaultLedger::new();
    ledger.inject(BASE_RED, START, base("cd34"));
    ledger.inject(LOST, START, candidate("ab12"));
    ledger.inject(LATE, START, candidate("ab12"));
    let findings = judge(
        &mut ledger,
        &[
            repair(),
            Observed { at: START, reaction: Reaction::GateRetry { gate: "gate".into(), run: "9".into(), bound: 1 } },
        ],
        SEED,
    );
    assert_eq!(
        findings,
        vec![
            "blame: repair of task 12 at Time(20), seed 42: no fault explains the reaction; struck by ci/base_red, ci/lost, busy/late",
            "blame: gate retry of gate at Time(10), seed 42: no fault explains the reaction; struck by nothing",
        ]
    );
    assert!(findings.iter().all(|line| line.lines().count() == 1));
}
