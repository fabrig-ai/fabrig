//! Fixed ceilings for the stories and simulation tests and their crash sweeps.
use crate::stories::Story;
use jig_conformance::scenarios::Scenario;

// Measured peak: 237,036 bytes; 25% headroom, rounded up to two significant digits.
const STARTUP_HEAP_CEILING: u64 = 300_000;

// Measured peak: 510,125 bytes; 25% headroom, rounded up to two significant digits.
const STORY_GOAL_ISSUE_HEAP_CEILING: u64 = 640_000;

// Measured peak: 273,898 bytes; 25% headroom, rounded up to two significant digits.
const STORY_ISSUE_CREATION_HEAP_CEILING: u64 = 350_000;

// Measured peak: 482,633 bytes; 25% headroom, rounded up to two significant digits.
const STORY_LOST_MERGE_HEAP_CEILING: u64 = 610_000;

// Measured peak: 519,545 bytes; 25% headroom, rounded up to two significant digits.
const STORY_OTHER_HAND_HEAP_CEILING: u64 = 650_000;

// Measured peak: 238,343 bytes; 25% headroom, rounded up to two significant digits.
const STORY_PERSON_WORDS_HEAP_CEILING: u64 = 300_000;

// Measured peak: 617,188 bytes; 25% headroom, rounded up to two significant digits.
const STORY_REPAIR_CI_HEAP_CEILING: u64 = 780_000;

// Measured peak: 508,708 bytes; 25% headroom, rounded up to two significant digits.
const STORY_REVIEWED_CHANGE_HEAP_CEILING: u64 = 640_000;

// Measured peak: 715,129 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_CHANGED_JUDGE_HEAP_CEILING: u64 = 900_000;

// Measured peak: 398,211 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_COMPOUND_HEAP_CEILING: u64 = 500_000;

// Measured peak: 382,753 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_CONVERSATION_ENDS_WHILE_READ_HEAP_CEILING: u64 = 480_000;

// Measured peak: 284,632 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_GRACE_HEAP_CEILING: u64 = 360_000;

// Measured peak: 315,443 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_LATE_COPY_HEAP_CEILING: u64 = 400_000;

// Measured peak: 427,731 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_NARROWING_POLICY_HEAP_CEILING: u64 = 540_000;

// Measured peak: 512,627 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_OTHER_HAND_HEAP_CEILING: u64 = 650_000;

// Measured peak: 385,696 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_QUERY_WATCHED_WHILE_TASKS_END_HEAP_CEILING: u64 = 490_000;

// Measured peak: 285,756 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_RACING_DECISIONS_HEAP_CEILING: u64 = 360_000;

// Measured peak: 340,670 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_RUN_BUDGET_HEAP_CEILING: u64 = 430_000;

// Measured peak: 458,540 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_STANDING_TASK_HEAP_CEILING: u64 = 580_000;

// Measured peak: 303,430 bytes; 25% headroom, rounded up to two significant digits.
const SHARED_UNCERTAIN_RESTARTS_HEAP_CEILING: u64 = 380_000;

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
