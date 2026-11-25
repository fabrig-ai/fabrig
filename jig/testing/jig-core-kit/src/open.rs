//! Open-problem exclusions (`jig/docs/design/domain/testing-fakes.md`, 1.3).
//! The sweeps drawing these faults take these exclusions.

use skein_world::{faults::Kind, sweep::Exclusion};

pub const PARKS_WITH_A_QUESTION_OPEN: Kind =
    Kind { family: "runs that park, resume and move", name: "a run parks with a question to its requester open" };
pub const AMENDS_A_PASSED_DEADLINE: Kind =
    Kind { family: "people and parties", name: "amending a deadline after it has passed" };
pub const WIDENS_GOAL_AUTHORITY: Kind =
    Kind { family: "people and parties", name: "a party widens a goal's authority" };
pub const NEW_SERVICE_RUNS_ON_AN_ACCOUNT: Kind =
    Kind { family: "people and parties", name: "a new service party runs on an account" };
pub const START_WITH_NO_BUDGET: Kind = Kind { family: "the engine's own bounds", name: "a start with no budget" };
pub const NEW_PARTY_ASKS_FOR_FUNDS: Kind =
    Kind { family: "the engine's own bounds", name: "a new party asks for funds" };
pub const DELEGATING_RUN_AT_ITS_MINIMUM: Kind =
    Kind { family: "the engine's own bounds", name: "a delegating task's run at its minimum" };

pub const EXCLUSIONS: [Exclusion; 6] = [
    Exclusion::new(
        "a_party_extends_its_goals_deadline",
        "provider failures",
        AMENDS_A_PASSED_DEADLINE,
        &["the amending party started the goal"],
    ),
    Exclusion::new(
        "an_owners_widening_is_never_proposed_to_itself",
        "provider failures",
        WIDENS_GOAL_AUTHORITY,
        &["the widening party is the project's owner"],
    ),
    Exclusion::new(
        "a_new_party_gets_account_uses",
        "provider failures",
        NEW_SERVICE_RUNS_ON_AN_ACCOUNT,
        &["the account's uses were set before the service was made"],
    ),
    Exclusion::new(
        "a_start_without_a_budget_leaves_room",
        "provider failures",
        START_WITH_NO_BUDGET,
        &["its pool funds another request"],
    ),
    Exclusion::new(
        "a_full_period_is_named_as_funding",
        "provider failures",
        NEW_PARTY_ASKS_FOR_FUNDS,
        &["its period is fully allocated"],
    ),
    Exclusion::new(
        "a_small_delegating_task_funds_its_delegates",
        "provider failures",
        DELEGATING_RUN_AT_ITS_MINIMUM,
        &["half its budget is at most the minimum run spend"],
    ),
];
