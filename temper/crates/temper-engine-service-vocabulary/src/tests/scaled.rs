//! Work-in-flight formulas, including their format caps (configuration/derived.md, 5.4).
use crate::limits;
use jig_protocol_config as config;
use temper_service_core::derived;

fn value(name: &str, jig: &config::Limits, own: &limits::Limits) -> u64 {
    if let Some(row) = config::names::find(name.as_bytes()) {
        return (row.value)(jig);
    }
    (limits::find(name.as_bytes()).unwrap().value)(own)
}
fn counts(goals: u32) {
    let stock = limits::stock();
    let stock_jig = config::stock();
    let (jig, own) = limits::base(goals, derived::STOCK.payload, derived::STOCK).unwrap();
    for name in [
        "tasks.tasks",
        "tasks.project_tasks",
        "tasks.project_goals",
        "tasks.placed_holds",
        "tasks.project_placed_holds",
        "tasks.party_placed_holds",
        "tasks.exemptions",
        "tasks.funders",
        "tasks.pools",
        "tasks.hold_waiters",
        "people.inbox_tasks",
        "forge.tasks",
        "forge.holds",
        "forge.subscriptions",
        "forge.entries",
        "forge.client.entries",
        "forge.client.resources",
        "forge.issues",
        "forge.landings",
    ] {
        let formula = value(name, &stock_jig, &stock).checked_mul(u64::from(goals)).unwrap().div_ceil(40);
        let expected = match name {
            "people.inbox_tasks" => u64::from(jig.core.tasks.tasks).checked_mul(2).unwrap(),
            "forge.holds" => formula.min(256),
            "forge.tasks" => formula.min(1024),
            "tasks.exemptions" => formula.min(4096).min(u64::from(jig.core.tasks.tasks)),
            "forge.landings" => formula.min(u64::from(own.forge.holds.checked_div(own.forge.landing_members).unwrap())),
            _ => formula,
        };
        assert_eq!(value(name, &jig, &own), expected, "{name} at {goals}");
    }
    assert_eq!(own.forge.changes, stock.forge.changes);
}
#[test]
fn forty_live_goals_keep_every_stock_count() {
    counts(40);
}
#[test]
fn eighty_live_goals_double_counts_until_their_caps() {
    counts(80);
    let (jig, own) = limits::base(80, derived::STOCK.payload, derived::STOCK).unwrap();
    assert_eq!(jig.core.tasks.tasks, 1536);
    assert_eq!(own.forge.entries, 16);
}
#[test]
fn one_live_goal_rounds_each_count_up() {
    counts(1);
}
