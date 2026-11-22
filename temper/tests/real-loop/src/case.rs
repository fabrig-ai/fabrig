//! The reference case's cast, accounts and first kit tree (testing/reference-case.md).
use crate::{fixtures, forge, landing};
use serde_json::{Value, json};
use skein_fake_forge_domain::api;

pub mod acts;
pub mod defaults;
pub mod oracle;
pub mod scripts;

// Fresh deployment numbers; the oracle checks them against the person's documents
// and A's question's tool output and the answer delivered to A.
pub const PROJECT: u64 = 1;
pub const DIRECTOR: u64 = 1;
pub const A: u64 = 2;
pub const DESIGN: u64 = 3;
// The goals' landing subscriptions and the design's result precede this question.
pub const QUESTION: u64 = 6;
// After step_4_1's chat and A's delegation create tasks 1..=5,
// step_4_3's chat creates CHECK; its delegate creates B, which delegates BENCHMARK.
pub const CHECK: u64 = 6;
pub const B: u64 = 7;
pub const BENCHMARK: u64 = 8;
// The change procedure in step_4_1 creates its coder, then its reviewer.
pub const CODER: u64 = 4;
pub const REVIEW: u64 = 5;
pub const CHECK_PERSON: &str = "@case_check Schedule B's benchmark and run the two-hour check.";
pub const B_SPEC: &str = "@case_b Own the benchmark; delegate it and report its ending.";
pub const BENCHMARK_SPEC: &str = "@case_benchmark Run the benchmark and wait in place.";
pub const BENCHMARK_RUNNING: &str = "The benchmark is running.";
pub const CHECK_REASON: &str = "The two-hour check found the benchmark stalled; cancel it.";
pub const B_REPORT: &str = "The benchmark was cancelled; B has nothing left running.";
pub const CHECK_REPORT: &str = "The two-hour check cancelled the stalled benchmark and B reported.";
pub const FIRST_ACCOUNT: u64 = 1;
pub const SECOND_KEY_FILE: &str = "second.key";
pub const DEFAULT_MODEL: &str = "fake";
pub const DEFAULT_MODEL_NUMBER: u8 = 0;
pub const HELD_PERSON: &str = "@case_director Delegate A's report and wait for it.";
pub const HELD_A_SPEC: &str = "@case_a Report when your account allows the work to continue.";
pub const RESUMED_REPORT: &str = "The work continued on its account.";
// The fake forge's one repository stands as kit, since forge.rs names it,
// until the fixture holds two.
pub const KIT: &str = "acme/app";
pub const OWNER_MODEL: &str = "owner";
pub const OWNER_EFFORT: &str = "xhigh";
// Model and effort numbers are their configuration order, after the default model.
pub const OWNER_MODEL_NUMBER: u8 = 1;
pub const OWNER_EFFORT_NUMBER: u8 = 0;
pub const DRAFT: &str = "docs/drafts/export.md";
pub const DESIGN_DOC: &str = "docs/design/export.md";
pub const OWNERS_DOC: &str = "docs/process/owners.md";
pub const DRAFT_WORDS: &str = "# Export draft\n\nExport the application's data.\n";
pub const OWNERS_WORDS: &str = "# Owners\n\nAn owner designs its draft, reports, then implements it.\n";
pub const DESIGN_WORDS: &str = "# Export\n\nDesigned by A from the draft at its commit.\n";
pub const DESIGN_LINE: &str = "Designed by A from the draft at its commit.";
pub const PERSON: &str = "@case_director Export's draft has landed; schedule it, priority 1.";
pub const A_SPEC_PREFIX: &str = "@case_a Own the draft Export: docs/drafts/export.md at kit's commit ";
pub const A_SPEC_SUFFIX: &str = ". Design it as a change of Markdown alone, send me your design report as a question, and finish on my green light.";
pub const SCHEDULED: &str = "Scheduled: A owns Export at priority 1.";
pub const GREEN_LIGHT: &str = "Green light: go on.";
pub const ANSWERED: &str = "Answered A.";
pub const DIRECTOR_REPORT: &str = "A designed Export and has its green light.";
pub const DESIGN_SPEC: &str = "@case_design Write Export's design at docs/design/export.md, Markdown alone.";
pub const DESIGN_DELEGATED: &str = "Design delegated.";
pub const DESIGN_REPORT: &str =
    "Export's design has landed on kit's main at docs/design/export.md: one Markdown file. Green light to go on?";
pub const ASKED: &str = "Asked the director.";
pub const A_REPORT: &str = "Export's design landed, and the director gave its green light.";
pub const DESIGN_TITLE: &str = "Export's design";
pub const REVIEWED: &str = "Checked";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Agent {
    Director,
    A,
    Design,
    Review,
    Check,
    B,
    Benchmark,
    /// The director whose run crosses a worker restart.
    Hang,
    /// A's next phase, asking that director for its direction.
    Phase,
    Rules,
    Scope,
    Pause,
    Owner,
    Bowner,
    Helper,
    Implementer,
    H1Coder,
    Shape,
    Mover,
    Direction,
    StoryOwner,
    OverlapOwner,
    X3Writer,
    PlannedWriter,
    StoryWriter,
    // Act 3, step 6.5 (testing/reference-case.md)
    BudgetDirector,
    BudgetOwner,
    BudgetSubowner,
    BudgetInvestigator,
    BudgetAdvisor,
    // 6.6
    WaitingStarter,
    WaitingKitCoder,
    WaitingAppCoder,
    // 6.4
    RedRepairCoder,
    QaDirector,
    QaOwner,
    QaSubowner,
    QaTester,
    QaCoder,
}
impl Agent {
    #[must_use]
    pub fn cue(self) -> &'static [u8] {
        match self {
            // Act 3, step 6.5 (testing/reference-case.md)
            Self::BudgetDirector => b"@case_budget_director",
            Self::BudgetOwner => b"@case_budget_owner",
            Self::BudgetSubowner => b"@case_budget_subowner",
            Self::BudgetInvestigator => b"@case_budget_investigator",
            Self::BudgetAdvisor => b"@case_budget_advisor",
            // 6.6
            Self::WaitingStarter => b"@case_waiting_start",
            Self::WaitingKitCoder => b"@case_waiting_kit",
            Self::WaitingAppCoder => b"@case_waiting_app",
            // 6.4
            Self::RedRepairCoder => b"Repair landing branch CI",
            Self::Direction => b"@case_direction",
            Self::StoryOwner => b"@case_story_owner",
            Self::OverlapOwner => b"@case_overlap_owner",
            Self::X3Writer => b"@case_x3_writer",
            Self::PlannedWriter => b"@case_planned_writer",
            Self::StoryWriter => b"@case_story_writer",
            Self::Shape => b"@case_shape",
            Self::Mover => b"@case_mover",
            Self::Rules => b"@case_rules",
            Self::Scope => b"@case_scope",
            Self::H1Coder => b"@case_h1",
            Self::Pause => b"@case_pause",
            Self::Owner => b"@case_owner",
            Self::Bowner => b"@case_bowner",
            Self::Helper => b"@case_helper",
            Self::Implementer => b"@case_implementer",
            Self::Hang => b"@case_hang",
            Self::Phase => b"@case_phase",
            Self::Check => b"@case_check",
            Self::B => b"@case_b ",
            Self::Benchmark => b"@case_benchmark",
            Self::Director => b"@case_director",
            Self::A => b"@case_a ",
            Self::Design => b"@case_design",
            Self::Review => b"Review this change at the named head",
            Self::QaDirector => b"@case_qa_director",
            Self::QaOwner => b"@case_qa_owner",
            Self::QaSubowner => b"@case_qa_subowner",
            Self::QaTester => b"@case_qa_tester",
            Self::QaCoder => b"@case_qa_coder",
        }
    }
}

#[must_use]
pub fn kit_tree() -> Box<[api::File]> {
    let mut tree = (forge::fixture().setup)().tree.into_vec();
    for (path, words) in [(DRAFT, DRAFT_WORDS), (OWNERS_DOC, OWNERS_WORDS)] {
        tree.push(api::File { path: path.as_bytes().into(), content: words.as_bytes().into() });
    }
    tree.into_boxed_slice()
}

// The seeded first repository's SHA-1 spelling; derived again in the test below.
const DRAFT_COMMIT: &str = "910a2dec89025cc1000000000000000100000000";

#[must_use]
pub fn draft_commit() -> String {
    DRAFT_COMMIT.into()
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use skein_fake_forge_domain as domain;

    #[test]
    fn draft_identity_matches_the_seeded_repository() {
        let config = temper_worker_checkout_world::hosted::config();
        let mut domain = domain::Domain::new(&config, fixtures::SEED);
        let first = domain::repository(&mut domain, &config, (fixture().setup)());
        let object = skein_fake_forge_forgejo::routes::object(first);
        let mut words = String::new();
        assert!(object.bytes[20..].iter().all(|byte| *byte == 0), "seeded repository uses SHA-1");
        for byte in &object.bytes[..20] {
            use std::fmt::Write;
            write!(words, "{byte:02x}").expect("commit hex");
        }
        assert_eq!(draft_commit(), words);
    }
}

#[must_use]
pub fn fixture() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (landing::fixture().setup)();
            setup.tree = kit_tree();
            setup.checks.cue =
                Some(api::Cue::File { path: DESIGN_DOC.as_bytes().into(), green: DESIGN_LINE.as_bytes().into() });
            setup
        },
        configure: |value, agent, forge| {
            (landing::fixture().configure)(value, agent, forge);
            // The seed retains its authority; the service receives these defaults directly.
            for member in ["charters", "templates", "policies", "rules"] {
                value["engine"].as_object_mut().expect("engine").remove(member);
            }
            fixtures::own_policy(value);
            value["world_settings"]["case"] = true.into();
            // These stories budget ten-unit runs and their nested delegates;
            // their own cap keeps those purses independent of the binary's K.
            value["world_settings"]["maximum_run_spend"] = 10.into();
            value.jig_over("people.projects", 2);
            value.jig_over("authority.projects", 2);
            value.jig_over("tasks.funders", 4);
            // A runs on the director's explicit choice, distinct from the default.
            let mut model = value["world_settings"]["llm"]["models"][0].clone();
            model["name"] = OWNER_MODEL.into();
            model["efforts"] = json!([OWNER_EFFORT]);
            value["world_settings"]["llm"]["models"].as_array_mut().expect("models").push(model);
            value["world_settings"]["llm"]["providers"][0]["models"] = json!(["fake", OWNER_MODEL]);
            // The director delegates, messages, amends and keeps project notes.
            // The extra owner adds one delegation level before the change procedure.
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["tools"] = 391.into();
                project["policy"]["ceiling"]["notes"] = 2.into();
                project["policy"]["roles"][0]["authority"]["notes"] = 2.into();
                project["policy"]["ceiling"]["delegation"]["depth"] = 5.into();
                project["policy"]["roles"][0]["authority"]["delegation"]["depth"] = 5.into();
                project["policy"]["roles"][0]["authority"]["tools"] = 391.into();
            }
            // A tracked owner reserves core news, landing news, an answer and its
            // delegate's result together; the landing fixture's inbox holds 4 KiB.
            value.jig_over("tasks.inbox_bytes", 16384);
            // The director's prescribed final report exceeds the chat's 32 bytes.
            value["world_settings"]["report_words"] = 128.into();
            // The brief includes the full draft commit and its instructions.
            value.jig_over("documents.text_bytes", 512);
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

/// Keep only the director on its worker; delegates retain the usual parking waits.
#[must_use]
pub fn fixture_placed() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            // The chat and A both use charter 1 in the base fixture. A waiting
            // run keeps its one slot until a message resumes it, so give only
            // the director a charter whose wait exceeds PATIENCE.
            let wait = crate::PATIENCE.saturating_mul(3);
            value["world_settings"]["second_wait_ms"] = wait.as_millis().into();
            // Creating offers the landing charters 1 and 4 plus this director's charter 2.
            value.jig_over("documents.lines", 3);
            value.jig_over("tasks.charters", 2);
            value["world_settings"]["executor"] = 2.into();
            // The person must also be offered the new executor for that chat.
            let mut kinds = value["world_settings"]["policies"][0]["policy"]["ceiling"]["delegation"]["kinds"].clone();
            kinds.as_array_mut().expect("executor kinds").push(json!({"kind":"charter","number":2}));
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
                project["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds.clone();
            }
            landing::admit(value);
        },
        ..fixture()
    }
}

// Keep two calls, payload 4,096 and output 8,192. Window 262,144 gives
// final A = 1,320,403 and F(A) = 1,320,467, within Small's 1,590,110-byte
// page ceiling; the old 1,048,576-token window required a 1,792,326-byte frame.
fn small_two_call_window(value: &mut Value, agent: &mut Value) {
    for model in value["world_settings"]["llm"]["models"].as_array_mut().expect("engine models") {
        model["window"] = 262_144.into();
    }
    agent["endpoints"][0]["limits"]["window"] = 262_144.into();
}

/// The check keeps all three runs in place, with timer and cancellation authority.
#[must_use]
pub fn fixture_check() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            // One adopted repository contains one pull. Its initial scan and
            // the scripted forge hooks supply the listing changes; repeating
            // the listing every 10 ms adds traffic throughout the timer story.
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value.temper_over(format!("forge.client.{name}"), story_ms + 1);
            }
            // Branch watches keep their 100 ms slow reconciliation: at most
            // 2 * (PATIENCE / 100 ms + 1) steady reads for the two branches.
            // Subscribe is family 32; cancel shares control (4), already in 391.
            let tools = 0x0187 | 0x0020;
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["tools"] = tools.into();
                project["policy"]["roles"][0]["authority"]["tools"] = tools.into();
            }
            // This placement story keeps its own ten-unit run cap so waiting
            // agents stay placed until the late benchmark worker is added.
            value["world_settings"]["maximum_run_spend"] = 10.into();
            // Announce placement and wait in one completion, with serial tools.
            agent["profile"]["declared"]["calls_per_response"] = 2.into();
            small_two_call_window(value, agent);
            agent["profile"]["policy"]["parallel_tools"] = 1.into();
            value.jig_over("hub.run_calls", 2);
            // The opener's checkout needs an LLM grant and an application git
            // grant. One refuses its coder; two fit the three-worker ring.
            value.jig_over("hub.accounts", 2);
            landing::admit(value);
        },
        ..fixture_placed()
    }
}

/// The third-chat scenario starts a fresh deployment; its director is first.
pub const HANG: u64 = 1;
/// A's next phase is this deployment's first delegate.
pub const PHASE: u64 = 2;
/// The person requests the restart scenario.
pub const HANG_PERSON: &str = "@case_hang Delegate A's next phase and answer it once after the workstation goes away.";
/// The brief for A's next phase.
pub const PHASE_SPEC: &str = "@case_phase Own Export's next phase and ask the director for its green light.";
/// A's question retained across the director's retry.
pub const PHASE_QUESTION: &str = "May Export's next phase begin?";
/// The director's single answer.
pub const PHASE_ANSWER: &str = "Begin Export's next phase.";
/// A's completed next-phase report.
pub const PHASE_REPORT: &str = "Export's next phase has its direction.";
/// The committed read barrier before the workstation stops.
pub const HANG_READ: &str = "A's next-phase question is read; its answer is still waiting.";
/// The restarted director's final report.
pub const HANG_REPORT: &str = "The restarted director answered A and its next phase reported.";
/// The first question in this fresh deployment, checked in the read and answer.
pub const PHASE_QUESTION_NUMBER: u64 = 1;

/// The base case's one-slot workers, owner model, notes and placed director.
/// No repository adoption, landing policy, protected tree or check scripts are needed.
#[must_use]
pub fn fixture_4_6() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            value["world_settings"]["landing"]["projects"] = json!([]);
            // This question-retention story keeps ws away until the retry ends.
            // A zero restart window permits that retry on the other workers (executors.md, 6.4).
            value.jig_over("fleet.restart_wait_ms", 0);
            // The retry path is the same, only its wait is short; the backoff is not what this story tests.
            value.jig_over("tasks.retries.agent.base_ms", 1);
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

// Act 2, step 5.1 (testing/reference-case.md)
pub const RULES: u64 = 1;
pub const SCOPE: u64 = 2;
pub const SCOPE_QUESTION_NUMBER: u64 = 2;
pub const RULES_HOLD_REVISION: u64 = 1;
pub const RULES_PERSON: &str = "@case_rules Delegate A to ask about x3's scope and wait for its report.";
pub const SCOPE_SPEC: &str = "@case_scope Own x3's scope; ask the director and report on its answer.";
pub const SCOPE_QUESTION: &str = "What is x3's scope in the next milestone?";
pub const SCOPE_ANSWER: &str = "x3 owns the next milestone's design and its defined end.";
pub const RULES_DELEGATED: &str = "A owns the scope question.";
pub const RULES_ANSWERED: &str = "A has its scope answer.";
pub const SCOPE_ASKED: &str = "The scope question awaits the director.";
pub const SCOPE_REPORT: &str = "x3's scope has its direction.";
pub const RULES_REPORT: &str = "A reported x3's scope under the new process.";
pub const SCOPE_QUESTION_HOLD: &[u8] = b"case-scope-question";
pub const RULES_FILE: &str = "AGENTS.md";
pub const DROPPED_RULE: &str = "Owners check shared version numbers before they land.";
pub const OLD_RULES: &str = "# Process\n\nOwners check shared version numbers before they land.\n";
pub const NEW_RULES: &str = "# Process\n\nAn owner takes one mission, at most four milestones, then stops.\nMilestones take 2-3 hours under a 4-hour cap, with a sub-owner each.\nThe plan comes first. Start a fresh session at each milestone's end.\nRun a QA pass beside the next milestone.\n";

#[must_use]
pub fn fixture_5_1() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (forge::fixture().setup)();
            let mut tree = setup.tree.into_vec();
            tree.push(api::File { path: RULES_FILE.as_bytes().into(), content: OLD_RULES.as_bytes().into() });
            setup.tree = tree.into_boxed_slice();
            setup
        },
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            value["world_settings"]["second_resume"] = false.into();
            value["world_settings"]["landing"]["projects"] = json!([]);
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

// Act 2, step 5.3 (testing/reference-case.md)
pub const SHAPE: u64 = 1;
pub const MOVER: u64 = 2;
pub const MOVER_QUESTION_NUMBER: u64 = 1;
pub const MOVER_AMENDMENT_REVISION: u64 = 1;
pub const SHAPE_PERSON: &str = "@case_shape Help A move Export to the new shape; amend its instructions when it asks.";
pub const MOVER_SPEC: &str = "@case_mover Own Export; ask the director how to go on under the new shape and report.";
pub const OLD_BRIEF: &str = "Implement Export in one long session.";
pub const MOVER_QUESTION: &str = "How do I go on under the new shape?";
pub const NEW_BRIEF: &str = "x3 finishes the old way, outside any milestone, under the person's two-hour cap; if it doesn't finish, we evaluate together. Then make the milestone plan first.";
pub const SHAPE_REASON: &str = "Continue x3 under the cap, then plan the milestones.";
pub const SHAPE_DELEGATED: &str = "A owns Export under its old brief.";
pub const SHAPE_AMENDED: &str = "A's instructions now answer its question.";
pub const MOVER_ASKED: &str = "The new-shape question awaits the director.";
pub const MOVER_REPORT: &str = "x3 will finish under the cap; the milestone plan comes next.";
pub const SHAPE_REPORT: &str = "A reported under its amended instructions.";

#[must_use]
pub fn fixture_5_3() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            value["world_settings"]["owner_brief"] = true.into();
            // Creating offers the landing charters 1 and 4 plus the owner's charter 2.
            value.jig_over("documents.lines", 3);
            value.jig_over("tasks.charters", 2);
            let mut kinds = value["world_settings"]["policies"][0]["policy"]["ceiling"]["delegation"]["kinds"].clone();
            kinds.as_array_mut().expect("executor kinds").push(json!({"kind":"charter","number":2}));
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
                project["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds.clone();
            }
            // Neither task mounts a repository or delegates a change.
            value["world_settings"]["landing"]["projects"] = json!([]);
            landing::admit(value);
        },
        ..fixture()
    }
}

// Act 2, step 5.2 (testing/reference-case.md)
// The chat delegates A, the helper, then B; only B creates descendants.
// B delegates its implementer, then h1, which creates its coder and reviewer.
pub const PAUSE_WORKERS: &[&str] = &["ws", "s1", "s2"];
pub const CASE_5_2_WORKERS: &[&str] = &["ws", "s1", "s2", "s3"];
pub const PAUSE: u64 = 1;
pub const OWNER: u64 = 2;
pub const HELPER: u64 = 3;
pub const HELPER_CHARTER: u64 = 9;
pub const BOWNER: u64 = 4;
pub const IMPLEMENTER: u64 = 5;
pub const H1: u64 = 6;
pub const H1_CODER: u64 = 7;
pub const H1_REVIEWER: u64 = 8;
pub const B_HOLD: u64 = 1;
pub const PAUSE_PERSON: &str = "@case_pause Pause every owner but A; I'll go through each one's work and say what resumes. Stop the old shape's leftovers.";
pub const OWNER_SPEC: &str = "@case_owner A remains free to report.";
pub const BOWNER_SPEC: &str = "@case_bowner Delegate your implementer and h1, then await their endings.";
pub const HELPER_SPEC: &str = "@case_helper Wait on the old shape's charter.";
pub const IMPLEMENTER_SPEC: &str = "@case_implementer Finish your running work when released.";
pub const H1_SPEC: &str = "@case_h1 Write docs/design/h1.md, Markdown alone.";
pub const PAUSE_PROPOSAL: &str = "I propose stopping B; the person must stop it.";
pub const HELPER_REASON: &str = "The person has stopped the old shape's leftovers; cancel this helper.";
pub const IMPLEMENTER_RESULT: &str = "B's implementer finished its running work.";
pub const OWNER_REPORT: &str = "A remained free and finished.";
pub const BOWNER_REPORT: &str = "B received its implementer's result and h1's landing after release.";
pub const PAUSE_REPORT: &str = "B reported after release; A finished and the old helper was cancelled.";
pub const H1_DOC: &str = "docs/design/h1.md";
pub const H1_WORDS: &str = "# h1\n\nB's change went on while B was stopped.\n";
pub const H1_LINE: &str = "B's change went on while B was stopped.";
pub const IMPLEMENTER_HOLD: &[u8] = b"case-5-2-implementer";
pub const REVIEW_HOLD: &[u8] = b"case-5-2-review";

#[must_use]
pub fn fixture_5_2() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (fixture().setup)();
            setup.checks.cue =
                Some(api::Cue::File { path: H1_DOC.as_bytes().into(), green: H1_LINE.as_bytes().into() });
            setup
        },
        configure: |value, agent, forge| {
            (fixture_placed().configure)(value, agent, forge);
            // Four one-slot workers keep the chat, implementer, reviewer and
            // B's release activation at once (testing/reference-case.md, act 2).
            value.jig_over("fleet.calls", CASE_5_2_WORKERS.len());
            // Initial adoption and forge hooks supply the listing changes,
            // as in fixture_check; repeating the listing adds no evidence.
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value.temper_over(format!("forge.client.{name}"), story_ms + 1);
            }
            value["world_settings"]["helper"] = true.into();
            // Creating offers charters 1, 2 and 4 plus the helper's charter 9.
            value.jig_over("documents.lines", 4);
            // Nine catalogue slots hold the stock with the helper replacing charter 9.
            // The director, chat, helper, reviewer and change procedure need five delegation kinds.
            for (name, size) in [("tasks.charters", 9), ("tasks.executor_kinds", 5), ("authority.executors", 5)] {
                value.jig_over(name, size);
            }
            let mut kinds = value["world_settings"]["policies"][0]["policy"]["ceiling"]["delegation"]["kinds"].clone();
            kinds.as_array_mut().expect("kinds").push(json!({"kind":"charter","number":HELPER_CHARTER}));
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["kinds"] = kinds.clone();
                project["policy"]["roles"][0]["authority"]["delegation"]["kinds"] = kinds.clone();
                // The chat's run, A, the helper and B reserve 10 + 5 + 5 + 40.
                project["policy"]["roles"][0]["authority"]["budget"]["spend"] = 65.into();
            }
            // The coder's checkout needs both its LLM and application git grants,
            // as in fixture_check. The cluster supplies one slot per worker.
            value.jig_over("hub.accounts", 2);
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

// Act 2, step 5.4 (testing/reference-case.md)
pub const DIRECTION: u64 = 1;
pub const STORY_OWNER: u64 = 2;
pub const OVERLAP_OWNER: u64 = 3;
pub const X3: u64 = 4;
pub const X3_PULL_NUMBER: u64 = 1;
pub const STORY_PULL_NUMBER: u64 = 2;
pub const X3_CODER: u64 = 5;
pub const X3_REVIEWER: u64 = 6;
pub const PLANNED: u64 = 7;
pub const PLANNED_CODER: u64 = 8;
pub const STORY_CHANGE: u64 = 9;
pub const STORY_CODER: u64 = 10;
pub const STORY_REVIEWER: u64 = 11;
pub const DIRECTION_REVISION: u64 = 1;
pub const OVERLAP_HOLD_REVISION: u64 = 1;
// Landing subscriptions precede the direction, amendment and deferred note.
pub const DEFERRED_NOTE_NUMBER: u64 = 7;
pub const DIRECTION_PERSON_MESSAGE: u64 = 5;
pub const DIRECTION_AMENDMENT_MESSAGE: u64 = 6;
pub const OVERLAP_SUBSCRIPTION: u64 = 3;
pub const OVERLAP_RECHECK_ATTEMPT: u64 = 3;
pub const DIRECTION_PERSON: &str = "@case_direction Delegate A and B as owners on kit and wait for my direction.";
pub const DIRECTION_MESSAGE: &str = "The end-to-end story first; cut it to one path; placeholders are fine. Stop x3. B's files are A's where the story needs them.";
pub const DIRECTION_BRIEF: &str = "The end-to-end story first, one path with placeholders. Stop x3 and defer its plan in a project note. B's files are A's where the story needs them.";
pub const DIRECTION_REASON: &str =
    "The person wants the end-to-end story first; stop x3 and use B's files where needed.";
pub const STORY_OWNER_SPEC: &str =
    "@case_story_owner Own A's work: delegate x3, then await the director's new instructions.";
pub const OVERLAP_OWNER_SPEC: &str =
    "@case_overlap_owner Own B's work: delegate planned h1 on B's file, then wait for news.";
pub const X3_SPEC: &str = "@case_x3_writer Write docs/design/x3.md, Markdown alone.";
pub const PLANNED_SPEC: &str = "@case_planned_writer Plan a change to B's file; wait before coding it.";
pub const STORY_SPEC: &str = "@case_story_writer Write the end-to-end story in docs/design/b.md, Markdown alone.";
pub const X3_FILE: &str = "docs/design/x3.md";
pub const X3_WORDS: &str = "# x3\n\nThe old plan, deferred.\n";
pub const B_FILE: &str = "docs/design/b.md";
pub const B_WORDS: &str = "# B\n\nA placeholder before the story.\n";
pub const STORY_WORDS: &str = "# B\n\nThe end-to-end story takes one path through B's file.\n";
pub const STORY_GREEN: &str = "The end-to-end story takes one path through B's file.";
pub const X3_CANCEL_REASON: &str = "Defer x3: the person's end-to-end story comes first.";
pub const PLANNED_CANCEL_REASON: &str = "A's landing overlaps h1; cancel the plan to go again from the new main.";
pub const DEFERRED_NOTE: &str = "x3's deferred plan";
pub const DEFERRED_WORDS: &str =
    "Task 4 (x3) is deferred under its closed pull; return to its old plan after the end-to-end story.";
pub const STORY_OWNER_REPORT: &str = "The story landed; x3's deferred plan was recalled.";
pub const OVERLAP_OWNER_REPORT: &str = "B heard A's overlapping landing after release and cancelled h1.";
pub const DIRECTION_REPORT: &str = "A's story landed and B reported after release.";
pub const DIRECTION_HELD_WORDS: &str = "Waiting at the story's event.";
pub const B_DELEGATE_HOLD: &[u8] = b"case-5-4-b-delegate";
pub const X3_REVIEW_HOLD: &[u8] = b"case-5-4-x3-review";
pub const PLANNED_CODING_HOLD: &[u8] = b"case-5-4-h1-code";
pub const STORY_RECALL_HOLD: &[u8] = b"case-5-4-recall";

#[must_use]
pub fn fixture_5_4() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (fixture().setup)();
            let mut tree = setup.tree.into_vec();
            tree.push(api::File { path: B_FILE.as_bytes().into(), content: B_WORDS.as_bytes().into() });
            setup.tree = tree.into();
            setup.checks.cue = Some(api::Cue::File { path: B_FILE.as_bytes().into(), green: b"# B".as_slice().into() });
            setup
        },
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            // Delegate/amend/cancel and their wait share one LLM completion.
            agent["profile"]["declared"]["calls_per_response"] = 2.into();
            small_two_call_window(value, agent);
            value.jig_over("hub.run_calls", 2);
            // Read all three spec parameters, including the planned path (name 6).
            value.jig_over("documents.lines", 3);
            // The director's two-owner batch was refused: "batch limit".
            value.jig_over("tasks.batch", 2);
            // Then "task 2 goal limit": both owners must be tracked.
            value.jig_over("tasks.project_goals", 2);
            value.jig_over("fleet.calls", PAUSE_WORKERS.len());
            // "Authority refused: task allowance": the tree has eleven tasks,
            // two owners plus A's six descendants and B's two, under the chat.
            value.jig_over("tasks.tree_tasks", 11);
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["tasks"] = 11.into();
                project["policy"]["roles"][0]["authority"]["delegation"]["tasks"] = 11.into();
            }
            // The director was told "spend allowance": its run reserves ten,
            // and A and B reserve thirty and twenty for their subtrees.
            value["world_settings"]["cap"] = 60.into();
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["roles"][0]["authority"]["budget"]["spend"] = 60.into();
            }
            // Adoption and the fake forge's hooks supply listing changes (5.2).
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value.temper_over(format!("forge.client.{name}"), story_ms + 1);
            }
            value.jig_over("hub.accounts", 2);
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

pub const BATCH_WRITING: &str = "The batch delegate is writing.";

// The mission and sub-owner deadlines shared by the Act 3 stories.
pub const A_DEADLINE: u64 = 4_102_444_800_000_000_000;
pub const A1_DEADLINE: u64 = A_DEADLINE - 14_400_000_000_000;

// Act 3, step 6.5 (testing/reference-case.md)
pub const BUDGET_DIRECTOR: u64 = 1;
pub const BUDGET_OWNER: u64 = 2;
pub const BUDGET_SUBOWNER: u64 = 3;
pub const BUDGET_A5: u64 = 4;
pub const BUDGET_ADVISOR: u64 = 5;
pub const BUDGET_QUESTION_NUMBER: u64 = 1;
pub const BUDGET_UP_QUESTION_NUMBER: u64 = 2;
pub const BUDGET_PERSON_ANSWER_NUMBER: u64 = 3;
pub const BUDGET_NOTICE_NUMBER: u64 = 4;
pub const BUDGET_ANSWER_NUMBER: u64 = 5;
pub const BUDGET_TIMER_NUMBER: u64 = 6;
pub const BUDGET_FINDING_NUMBER: u64 = 7;
pub const BUDGET_AMENDMENT_NUMBER: u64 = 8;
pub const BUDGET_A5_REPORT_NUMBER: u64 = 9;
pub const BUDGET_ADVISOR_REPORT_NUMBER: u64 = 10;
pub const BUDGET_REVISION: u64 = 1;
pub const BUDGET_MEDIUM_NUMBER: u8 = 0;
pub const BUDGET_HIGH_NUMBER: u8 = 1;
pub const BUDGET_MEDIUM: &str = "medium";
pub const BUDGET_HIGH: &str = "high";
pub const BUDGET_PERSON: &str = "@case_budget_director Delegate A to own M1's budget question and wait for its report.";
pub const BUDGET_QUESTION: &str =
    "A1 asks A: a4's story takes 70 s against its suite's 60 s. May the budget be raised?";
pub const BUDGET_UP_QUESTION: &str =
    "A asks the director: may a4's story take 70 s against its suite's 60 s? Recommend no raise, make it cheaper.";
pub const BUDGET_PERSON_ANSWER: &str = "The person agrees: no raise, make it cheaper.";
pub const BUDGET_ANSWER: &str = "A answers A1: the person allows no raise; make the story cheaper.";
pub const BUDGET_FINDING: &str = "a5 found no cause at medium; evidence: the story's repeated document reads.";
pub const BUDGET_REASON: &str = "Try the investigation again at high effort, using the repeated-read evidence.";
pub const BUDGET_A5_REPORT: &str = "a5 found none again at high; evidence: repeated document reads dominate the story.";
pub const BUDGET_ADVISOR_SPEC: &str = "@case_budget_advisor Read a5's report and evidence in this brief, name the cause, and report once: a5 found none again at high; evidence: repeated document reads dominate the story.";
pub const BUDGET_ADVISOR_REPORT: &str = "The cause is redundant document polling; await the event and read once.";
pub const BUDGET_SUBOWNER_REPORT: &str = "A1 reports the advisor named the cause of a4's cost.";
pub const BUDGET_OWNER_REPORT: &str = "A reports M1's budget question answered and its investigation advised.";
pub const BUDGET_DIRECTOR_REPORT: &str = "The director reports A resolved the budget question.";
pub const BUDGET_AMEND_HOLD: &[u8] = b"case-6-5-amend";
pub const BUDGET_HIGH_HOLD: &[u8] = b"case-6-5-high";

#[must_use]
pub fn fixture_6_5() -> forge::Fixture {
    forge::Fixture {
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            value["world_settings"]["llm"]["models"][0]["efforts"] = json!([BUDGET_MEDIUM, BUDGET_HIGH]);
            // Small profile; default engine/agent model efforts [medium, high].
            // Singleton delegations, five tasks, parked owners and investigator.
            // Two calls carry a tool and wait in one completion; three fleet
            // calls admit PAUSE_WORKERS. Spend: a5 10, advisor 9; A1 25
            // (3 spent itself + a5's 3 + 9 advisor + 10 run reserve),
            // A 35 (25 + 10 run reserve), director 45 (35 + 10 reserve).
            // The base eight-task allowance holds this five-task tree.
            // landing.projects is empty: no repository or change is adopted.
            agent["profile"]["declared"]["calls_per_response"] = 2.into();
            small_two_call_window(value, agent);
            value.jig_over("hub.run_calls", 2);
            value.jig_over("fleet.calls", PAUSE_WORKERS.len());
            value["world_settings"]["cap"] = 45.into();
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["roles"][0]["authority"]["budget"]["spend"] = 45.into();
                project["policy"]["ceiling"]["tools"] = (0x0187 | 0x20).into();
                project["policy"]["roles"][0]["authority"]["tools"] = (0x0187 | 0x20).into();
            }
            value["world_settings"]["landing"]["projects"] = json!([]);
            landing::admit(value);
        },
        ..fixture()
    }
}

// Act 3, step 6.4 (testing/reference-case.md)
pub const RED_CHANGE: u64 = 1;
pub const RED_REVIEW: u64 = 2;
pub const RED_REPAIR: u64 = 3;
pub const RED_REPAIR_CODER: u64 = 4;
pub const RED_REPAIR_REVIEW: u64 = 5;
pub const RED_UPDATED_REVIEW: u64 = 6;
pub const RED_FIRST_HEAD: u64 = 2;
pub const RED_UPDATED_HEAD: u64 = 6;
pub const RED_A4_PULL: u64 = 1;
pub const RED_REPAIR_PULL: u64 = 2;
pub const RED_REVISION: u64 = 0;
pub const RED_FILE: &str = "docs/story.md";
pub const RED_GREEN: &str = "# Story green";
pub const RED_BASE: &str = "# Story green\n\nBase story.\n";
pub const RED_BROKEN: &str = "# Story red\n\nBase story.\n";
pub const RED_A4_FILE: &str = "docs/a4.md";
pub const RED_A4: &str = "# A4\n\nA4's story.\n";
pub const RED_REVIEW_HOLD: &[u8] = b"case-6-4-review";
pub const RED_REPAIR_HOLD: &[u8] = b"case-6-4-repair";

#[must_use]
pub fn fixture_6_4() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (fixture().setup)();
            let mut tree = setup.tree.into_vec();
            tree.push(api::File { path: RED_FILE.as_bytes().into(), content: RED_BASE.as_bytes().into() });
            setup.tree = tree.into();
            setup.checks.cue =
                Some(api::Cue::File { path: RED_FILE.as_bytes().into(), green: RED_GREEN.as_bytes().into() });
            setup
        },
        prepare: |forge, env| {
            use skein_fake_forge_domain as domain;
            // a4 starts as the person's existing change; only its update onto
            // repaired main needs a worker's real git in this story.
            let base = forge.branch(b"acme/app", b"main").expect("seeded main");
            let mut out = skein_lib::Queue::uncapped();
            let mut env = skein_lib::Env { now: env.now, wall: env.wall, limits: env.limits.clone() };
            domain::step(
                forge,
                &env,
                domain::Event::Call {
                    reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
                    repository: b"acme/app".as_slice().into(),
                    user: 1,
                    op: api::Op::Git(api::Git::Create { branch: b"a4".as_slice().into(), commit: base }),
                },
                &mut out,
            );
            loop {
                if let Some(domain::Request::Reply { result, .. }) = out.pop() {
                    assert_eq!(result, Ok(api::Answer::Branch(api::Created::Created)));
                    break;
                }
                env.now = forge.next_deadline().expect("branch creation reply");
                domain::fire(forge, &env, &mut out);
            }
            domain::advance(forge, &env, b"acme/app", b"a4", RED_A4_FILE.as_bytes(), RED_A4.as_bytes(), 1)
                .expect("a4's existing green change");
        },
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            value["world_settings"]["minimum_run_spend"] = 1.into();
            agent["profile"]["declared"]["calls_per_response"] = 1.into();
            value.jig_over("hub.run_calls", 1);
            value.jig_over("hub.accounts", 2);
            value.jig_over("fleet.calls", 1);
            value.jig_over("documents.lines", 2);
            // Independent allotments fund a4's two reviews and the repair;
            // the project leaves five units beyond the role for that repair.
            value["world_settings"]["cap"] = 26.into();
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                // Opening the person's source branch grants its owned copy's creation.
                let mut create = project["policy"]["ceiling"]["grants"][0].clone();
                create["kind"] = 9.into();
                project["policy"]["ceiling"]["grants"].as_array_mut().expect("ceiling grants").push(create.clone());
                project["policy"]["roles"][0]["authority"]["grants"].as_array_mut().expect("role grants").push(create);
                project["policy"]["roles"][0]["authority"]["budget"]["spend"] = 26.into();
                project["policy"]["roles"][0]["period_spend"] = 26.into();
                project["policy"]["period_spend"] = 31.into();
            }
            for (name, size) in
                [("tasks.tasks", 5), ("tasks.exemptions", 5), ("tasks.project_tasks", 5), ("tasks.tree_tasks", 6)]
            {
                value.jig_over(name, size);
            }
            value.temper_over("forge.change_policy.repairs", 1);
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value.temper_over(format!("forge.client.{name}"), story_ms + 1);
            }
            landing::admit(value);
        },
        ..forge::fixture()
    }
}

// Act 3, step 6.6 (testing/reference-case.md). These identities belong only
// to this two-repository case; earlier steps retain KIT = acme/app.
pub mod waiting {
    pub const STARTER: u64 = 1;
    pub const E1: u64 = 2;
    pub const REPIN: u64 = 3;
    pub const KIT_ID: u32 = 3;
    pub const APP_ID: u32 = 2;
    pub const KIT: &str = "acme/kit";
    pub const APP: &str = "acme/app";
    pub const FILE: &str = "src/fake.rs";
    pub const FIXED: &str = "// B's fake answers correctly.\n";
    pub const PIN: &str = "kit.pin";
    pub const CI_FILE: &str = "ci.txt";
    pub const GREEN: &str = "green\n";
    pub const PERSON: &str = "@case_waiting_start Land e1 in kit, then repin app to its merge.";
    pub const E1_SPEC: &str = "@case_waiting_kit Fix src/fake.rs and land e1 in kit.";
    pub const REPIN_SPEC: &str = "@case_waiting_app Repin kit.pin to e1's landed kit commit and land in app.";
    pub const REPORT: &str = "Kit and app landed.";
    pub const REPIN_HOLD: &[u8] = b"case-6-6-repin-producer";
    // Script inputs only: app root 1, kit root 2, e1 coder 3, kit merge 4,
    // repin coder 5, app merge 6. The repin turn waits for kit to land.
    // The oracle instead renders the unique kit Merged.commit.
    pub const PIN_INPUT: u64 = 4;

    #[must_use]
    pub fn pin_input() -> String {
        use std::fmt::Write;
        let mut words = String::new();
        for byte in skein_fake_forge_domain::object_bytes(PIN_INPUT) {
            write!(words, "{byte:02x}").expect("commit words");
        }
        words
    }
}

fn waiting_setup(id: u32, name: &str) -> api::Setup {
    let mut setup = (landing::fixture().setup)();
    setup.id = id;
    setup.name = name.as_bytes().into();
    let mut tree = setup.tree.into_vec();
    tree.push(api::File { path: waiting::CI_FILE.as_bytes().into(), content: waiting::GREEN.as_bytes().into() });
    setup.tree = tree.into();
    // The same stable CI file is present at the base and changed tree.
    setup.checks.cue =
        Some(api::Cue::File { path: waiting::CI_FILE.as_bytes().into(), green: waiting::GREEN.as_bytes().into() });
    setup
}

#[must_use]
pub fn fixture_6_6() -> forge::Fixture {
    forge::Fixture {
        prepare: |_, _| {},
        manual_merges: api::ManualMerges::OFF,
        setup: || waiting_setup(waiting::APP_ID, waiting::APP),
        extra_setups: || Box::new([waiting_setup(waiting::KIT_ID, waiting::KIT)]),
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            let mut kit_rule = value["world_settings"]["landing"]["projects"][0]["rules"][0].clone();
            kit_rule["pattern"]["segments"][3] = "kit".into();
            value["world_settings"]["landing"]["projects"][0]["rules"].as_array_mut().expect("rules").push(kit_rule);
            // Two enforced rules each contribute CI, up-to-date and exact-head
            // blocking review: 2 * 3 = 6 requirements. Both repos are hooked.
            value.jig_over("authority.requirements", 6);
            // Configuring-3's new admission check measures this two-repository
            // project policy at 4398 bytes, above the landing fixture's 4096.
            value.jig_over("core.policy_bytes", 8192);
            // One starter, two changes and their producers and reviewers.
            for name in ["tasks.batch", "authority.batch"] {
                value.jig_over(name, 2);
            }
            // Three lines carry a change's repository, branch and path; they
            // also cover the maximum requester ancestry. Two adopted repositories
            // and two checkout repositories fit the existing small capacities.
            value.jig_over("documents.lines", 3);
            // The coder's write + finish and the starter's delegate + wait share
            // one completion, serialized by the existing single call slot.
            agent["profile"]["declared"]["calls_per_response"] = 1.into();
            value.jig_over("hub.run_calls", 1);
            value.jig_over("hub.accounts", 2);
            value.jig_over("fleet.calls", 1);
            // F10: the atomic batch exceeds the inherited 2048-byte link
            // input. Until F10 admission/drop handling is fixed, this fixture
            // gives both link sides the agent's existing tool_payload bound.
            value.jig_over("core.call_answer_bytes", 4096);
            // Two ten-unit procedures and the starter's ten-unit run reserve.
            value["world_settings"]["cap"] = 30.into();
            // Hooks/adoption supply changes; retain the case's PATIENCE+1 polls.
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value.temper_over(format!("forge.client.{name}"), story_ms + 1);
            }
            landing::admit(value);
        },
    }
}

// Act 3, step 6.8 (testing/reference-case.md).
pub mod qa {
    pub const DIRECTOR: u64 = 1;
    pub const A: u64 = 2;
    pub const A1: u64 = 3;
    pub const T: u64 = 4;
    pub const C: u64 = 5;
    pub const CODER: u64 = 6;
    pub const REVIEWER: u64 = 7;
    pub const PERSON: &str = "@case_qa_director Delegate A for QA; report when its tester's test lands.";
    pub const OWNER_SPEC: &str = "@case_qa_owner Delegate A1 to land QA's test.";
    pub const SUBOWNER_SPEC: &str = "@case_qa_subowner Delegate a tester then land its test through your change.";
    pub const TESTER_SPEC: &str = "@case_qa_tester Report one adversarial test as path and content.";
    pub const CHANGE_SPEC: &str = "@case_qa_coder Land the test at tests/qa.rs from T's result, your named spec input.";
    pub const QA_TEST_PATH: &str = "tests/qa.rs";
    pub const QA_TEST: &str = "#[test]\nfn regression() { assert_eq!(2 + 2, 4); }\n";
    pub const TEST_REPORT: &str = "tests/qa.rs\n#[test]\nfn regression() { assert_eq!(2 + 2, 4); }\n";
    pub const SUBOWNER_REPORT: &str = "A1 closes M1 after its change lands T's test.";
    pub const OWNER_REPORT: &str = "A reports QA's test landed through A1.";
    pub const DIRECTOR_REPORT: &str = "The director reports QA's test landed through A1.";
    pub const STATUS_CONTEXT: &str = "tester-without-authority";
}

#[must_use]
pub fn fixture_6_8() -> forge::Fixture {
    forge::Fixture {
        setup: || {
            let mut setup = (fixture().setup)();
            setup.checks.cue =
                Some(api::Cue::File { path: qa::QA_TEST_PATH.as_bytes().into(), green: qa::QA_TEST.as_bytes().into() });
            setup
        },
        configure: |value, agent, forge| {
            (fixture().configure)(value, agent, forge);
            // c adds its input result to three ancestors and its own result;
            // coder/reviewer have four ancestors and a result. Small's four
            // store joins cannot read these; use standard's sixteen, as 6.2.

            value.temper_over("forge.named_calls", 128);
            value.temper_over("root.journal.deliveries", 1097);
            value.temper_over("root.journal.held", 3291);
            // Role -> director -> A -> A1 -> c -> coder: the role's
            // depth six yields the director's five, as 6.2's fixture does.
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["ceiling"]["delegation"]["depth"] = 6.into();
                project["policy"]["roles"][0]["authority"]["delegation"]["depth"] = 6.into();
            }
            // Seven tasks, with A adding one ancestor over 6.6's smoke.
            // Keep each batch singleton and serialize each response's calls.
            value.jig_over("documents.lines", 4);
            agent["profile"]["declared"]["calls_per_response"] = 1.into();
            value.jig_over("hub.run_calls", 1);
            value.jig_over("fleet.calls", PAUSE_WORKERS.len());
            value.jig_over("hub.accounts", 2);
            // A reserves A1 (50) and its run's ten; the director
            // reserves A (80) and its run. All remain below the base ceiling 100.
            value["world_settings"]["cap"] = 100.into();
            for project in value["world_settings"]["policies"].as_array_mut().expect("projects") {
                project["policy"]["roles"][0]["authority"]["budget"]["spend"] = 100.into();
            }
            let story_ms = crate::PATIENCE.as_nanos().div_euclid(1_000_000);
            for name in ["poll_ms", "poll_max_ms"] {
                value.temper_over(format!("forge.client.{name}"), story_ms + 1);
            }
            landing::admit(value);
        },
        ..fixture()
    }
}
