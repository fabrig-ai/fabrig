//! Fixed ceilings for the stories and simulation tests and their crash sweeps.
//! Each is three times its measured peak plus a quarter, rounded up
//! (temper/docs/design/domain/testing.md, 5.2).
use crate::stories::Story;
use jig_conformance::scenarios::Scenario;

// Measured peak: 237,036 bytes; three times the 25%-headroom ceiling.
const STARTUP_HEAP_CEILING: u64 = 900_000;

// Measured peak: 642,209 bytes; three times the 25%-headroom ceiling.
const STORY_GOAL_ISSUE_HEAP_CEILING: u64 = 2_430_000;

// Measured peak: 273,898 bytes; three times the 25%-headroom ceiling.
const STORY_ISSUE_CREATION_HEAP_CEILING: u64 = 1_050_000;

// Measured peak: 637,786 bytes; three times the 25%-headroom ceiling.
const STORY_LOST_MERGE_HEAP_CEILING: u64 = 2_400_000;

// Measured peak: 677,099 bytes; three times the 25%-headroom ceiling.
const STORY_OTHER_HAND_HEAP_CEILING: u64 = 2_550_000;

// Measured peak: 238,343 bytes; three times the 25%-headroom ceiling.
const STORY_PERSON_WORDS_HEAP_CEILING: u64 = 900_000;

// Measured peak: 783,255 bytes; three times the 25%-headroom ceiling.
const STORY_REPAIR_CI_HEAP_CEILING: u64 = 2_940_000;

// Measured peak: 644,720 bytes; three times the 25%-headroom ceiling.
const STORY_REVIEWED_CHANGE_HEAP_CEILING: u64 = 2_430_000;

// Measured peak: 715,129 bytes; three times the 25%-headroom ceiling.
const SHARED_CHANGED_JUDGE_HEAP_CEILING: u64 = 2_700_000;

// Measured peak: 398,211 bytes; three times the 25%-headroom ceiling.
const SHARED_COMPOUND_HEAP_CEILING: u64 = 1_500_000;

// Measured peak: 382,753 bytes; three times the 25%-headroom ceiling.
const SHARED_CONVERSATION_ENDS_WHILE_READ_HEAP_CEILING: u64 = 1_440_000;

// Measured peak: 284,632 bytes; three times the 25%-headroom ceiling.
const SHARED_GRACE_HEAP_CEILING: u64 = 1_080_000;

// Measured peak: 315,443 bytes; three times the 25%-headroom ceiling.
const SHARED_LATE_COPY_HEAP_CEILING: u64 = 1_200_000;

// Measured peak: 427,731 bytes; three times the 25%-headroom ceiling.
const SHARED_NARROWING_POLICY_HEAP_CEILING: u64 = 1_620_000;

// Measured peak: 512,627 bytes; three times the 25%-headroom ceiling.
const SHARED_OTHER_HAND_HEAP_CEILING: u64 = 1_950_000;

// Measured peak: 385,696 bytes; three times the 25%-headroom ceiling.
const SHARED_QUERY_WATCHED_WHILE_TASKS_END_HEAP_CEILING: u64 = 1_470_000;

// Measured peak: 285,756 bytes; three times the 25%-headroom ceiling.
const SHARED_RACING_DECISIONS_HEAP_CEILING: u64 = 1_080_000;

// Measured peak: 340,670 bytes; three times the 25%-headroom ceiling.
const SHARED_RUN_BUDGET_HEAP_CEILING: u64 = 1_290_000;

// Measured peak: 458,540 bytes; three times the 25%-headroom ceiling.
const SHARED_STANDING_TASK_HEAP_CEILING: u64 = 1_740_000;

// Measured peak: 303,430 bytes; three times the 25%-headroom ceiling.
const SHARED_UNCERTAIN_RESTARTS_HEAP_CEILING: u64 = 1_140_000;

pub(super) fn maximum(scenario: Option<Scenario>, story: Option<Story>) -> u64 {
    if let Some(story) = story {
        return match story {
            Story::GoalIssue => STORY_GOAL_ISSUE_HEAP_CEILING,
            Story::IssueCreation => STORY_ISSUE_CREATION_HEAP_CEILING,
            Story::LostMerge => STORY_LOST_MERGE_HEAP_CEILING,
            Story::OtherHand => STORY_OTHER_HAND_HEAP_CEILING,
            Story::PersonWords => STORY_PERSON_WORDS_HEAP_CEILING,
            Story::RepairCi => STORY_REPAIR_CI_HEAP_CEILING,
            Story::ReviewedChange => STORY_REVIEWED_CHANGE_HEAP_CEILING,
        };
    }
    match scenario {
        None => STARTUP_HEAP_CEILING,
        Some(Scenario::ChangedJudge) => SHARED_CHANGED_JUDGE_HEAP_CEILING,
        Some(Scenario::Compound) => SHARED_COMPOUND_HEAP_CEILING,
        Some(Scenario::ConversationEndsWhileRead) => SHARED_CONVERSATION_ENDS_WHILE_READ_HEAP_CEILING,
        Some(Scenario::Grace) => SHARED_GRACE_HEAP_CEILING,
        Some(Scenario::LateCopy) => SHARED_LATE_COPY_HEAP_CEILING,
        Some(Scenario::NarrowingPolicy) => SHARED_NARROWING_POLICY_HEAP_CEILING,
        Some(Scenario::OtherHand) => SHARED_OTHER_HAND_HEAP_CEILING,
        Some(Scenario::QueryWatchedWhileTasksEnd) => SHARED_QUERY_WATCHED_WHILE_TASKS_END_HEAP_CEILING,
        Some(Scenario::RacingDecisions) => SHARED_RACING_DECISIONS_HEAP_CEILING,
        Some(Scenario::RunBudget) => SHARED_RUN_BUDGET_HEAP_CEILING,
        Some(Scenario::StandingTask) => SHARED_STANDING_TASK_HEAP_CEILING,
        Some(Scenario::UncertainRestarts) => SHARED_UNCERTAIN_RESTARTS_HEAP_CEILING,
        Some(Scenario::ShrinkingPool) => unreachable!("temper has no pooled holds"),
    }
}
