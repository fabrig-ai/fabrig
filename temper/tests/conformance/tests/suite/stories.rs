use jig_conformance::{Application, Cut};
use temper_conformance_world::stories::{self, Story};
use temper_conformance_world::{Config, Temper};
#[test]
fn chat_change_lands_only_after_current_head_ci_and_review() {
    stories::run(Story::ReviewedChange, Cut::None, 71).expect("reviewed change");
}
#[test]
fn failed_ci_is_repaired_and_the_new_head_is_reviewed() {
    stories::run(Story::RepairCi, Cut::None, 71).expect("CI repair");
}
#[test]
fn another_hand_holds_the_change_and_preserves_its_branch() {
    stories::run(Story::OtherHand, Cut::None, 71).expect("other hand");
}
#[test]
fn lost_merge_answer_is_found_at_its_head_after_restart() {
    stories::run(Story::LostMerge, Cut::None, 71).expect("merge recovery");
}
#[test]
fn person_words_reach_the_live_coder_after_durability() {
    stories::run(Story::PersonWords, Cut::None, 71).expect("person words");
}
#[test]
fn a_goal_issue_follows_its_milestone_and_closes() {
    stories::run(Story::GoalIssue, Cut::None, 71).expect("goal milestones");
}
#[test]
fn lost_issue_creation_answer_recovers_without_a_second_issue() {
    stories::run(Story::IssueCreation, Cut::None, 71).expect("issue creation uncertainty");
}

#[test]
fn a_forked_cut_ends_as_a_fresh_cut_does() {
    // PersonWords has the fewest native baseline commits: 17, versus IssueCreation's 21.
    let story = Story::PersonWords;
    let seed = 101;
    let first_live = Temper::store(&Config::for_story(seed, story)).applied + 1;
    let baseline = stories::run(story, Cut::None, seed).expect("baseline");
    // Compare complete outcomes at the first, middle and last live cut. The
    // forked sweep still runs every cut and checks every story and referee rule.
    let compared = [first_live, first_live + (baseline.commits - first_live) / 2, baseline.commits];
    let fresh = compared
        .into_iter()
        .map(|cut| {
            let outcome = stories::run(story, Cut::AfterCommit(cut), seed).expect("fresh cut");
            assert!(outcome.crashes.contains(&cut), "fresh cut {cut} fired");
            (cut, outcome)
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let outcome = stories::run(story, Cut::EachCommitFrom(first_live), seed).expect("forked cuts");
    if outcome.forked.is_empty() || outcome.crashes != baseline.crashes {
        assert!(outcome.forked.is_empty(), "a child starts without the parent's completed cuts");
        assert_eq!(outcome.crashes.len(), baseline.crashes.len() + 1, "one additional cold cut");
        // The story also restarts itself; the complete crash list identifies its added cut.
        let mut cuts = fresh.iter().filter(|(_, expected)| expected.crashes == outcome.crashes);
        if let Some((cut, expected)) = cuts.next() {
            assert!(cuts.next().is_none(), "crashes identify one cut");
            assert_eq!(outcome, *expected, "cut {cut}");
        }
    }
    jig_conformance::cut_passed();
    assert_eq!(outcome.forked, (first_live..=baseline.commits).collect::<Vec<_>>(), "every live cut completed");
}

/// Crash one native story at its configured receipt cuts, including writer
/// restoration, an already deleted branch, a lost milestone comment and a
/// retained live-run word; its baseline runs above and in the fuzzy sweep.
/// Fixture startup keeps the real first start, then preseeds restored keys.
/// These offsets count live receipts after setup; the engine world separately
/// crashes an empty deployment at its first-start receipt one.
fn crashed_at(story: Story, cuts: [u64; 2]) {
    let setup = Temper::store(&Config::for_story(101, story)).applied;
    for cut in cuts {
        let cut = cut + setup - 1;
        let outcome = stories::run(story, Cut::AfterCommit(cut), 101)
            .unwrap_or_else(|error| panic!("{story:?}, cut {cut}: {error:?}"));
        assert!(outcome.crashes.contains(&cut), "{story:?} cut {cut} did not fire");
    }
}

#[test]
fn a_reviewed_change_recovers_its_focused_commit_cuts() {
    crashed_at(Story::ReviewedChange, [2, 17]);
}

#[test]
fn a_ci_repair_recovers_its_focused_commit_cuts() {
    crashed_at(Story::RepairCi, [2, 23]);
}

#[test]
fn a_goal_issue_recovers_its_focused_commit_cuts() {
    crashed_at(Story::GoalIssue, [21, 23]);
}

#[test]
fn another_hand_recovers_its_focused_commit_cuts() {
    crashed_at(Story::OtherHand, [2, 17]);
}

#[test]
fn a_lost_merge_recovers_its_focused_commit_cuts() {
    crashed_at(Story::LostMerge, [2, 18]);
}

#[test]
fn person_words_recover_a_retained_live_run_word() {
    crashed_at(Story::PersonWords, [2, 9]);
}

#[test]
fn an_issue_creation_recovers_its_focused_commit_cuts() {
    crashed_at(Story::IssueCreation, [2, 7]);
}

#[test]
fn a_ci_repair_keeps_its_settle_wait_across_restart() {
    let cut = 32 + Temper::store(&Config::for_story(101, Story::RepairCi)).applied - 1;
    let outcome = stories::run(Story::RepairCi, Cut::AfterCommit(cut), 101).expect("settling survives restart");
    assert!(outcome.crashes.contains(&cut));
}
