//! Refusals pinned to their design text (protocol/store/README.md, 2, and protocol/store/opening.md, 1).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "text assertions use ordinary Rust (programming-model.md 10.2)"
)]
use crate::{Check, Generations, Older, OpenRefusal};

const DESIGN: &str = concat!(
    include_str!("../../../../docs/design/protocol/store/README.md"),
    "\n",
    include_str!("../../../../docs/design/protocol/store/records.md"),
    "\n",
    include_str!("../../../../docs/design/protocol/store/opening.md"),
    "\n",
    include_str!("../../../../docs/design/protocol/store/checkpoints.md"),
);
#[test]
fn checkpoint_only_open_refusals_say_what_to_do_without_changing_the_deployment() {
    says(OpenRefusal::Unready { why: crate::Unready::Empty }, "- `Empty`: \"", "\"");
    says(OpenRefusal::Unready { why: crate::Unready::Upgrade }, "- `Upgrade`: \"", "\"");
}

pub(super) const FORMAT: &str = "putting the family's lower-case name in place of <family>:\n\n  > ";
pub(super) const DEPLOYMENT: &str = "deployment than its header (`records.md`, 3):\n\n  > ";
pub(super) const NUMBERS: &str = "the one the header records (`commits.md`, section 1):\n\n  > ";
pub(super) const FAILED: &str = "cannot recover (`opening.md`):\n\n  > ";

fn unwrapped(start: &str, end: &str) -> String {
    let text = DESIGN.split_once(start).expect("design's refusal start").1;
    let text = text.split_once(end).expect("design's refusal end").0;
    text.lines()
        .map(|line| line.trim().trim_start_matches("> "))
        .collect::<Vec<_>>()
        .join(" ")
        .replace("<bridge>", crate::FLOOR_BRIDGE)
}
fn says(refusal: OpenRefusal, start: &str, end: &str) {
    assert_eq!(refusal.words().join("`<deployment>`"), unwrapped(start, end));
}

pub(super) fn has_deployment_slots(events: &[crate::Event], start: &str) {
    let [crate::Event::OpenRefused { why, .. }] = events else { panic!("one refusal: {events:?}") };
    let text = unwrapped(start, "\n\n-");
    assert_ne!((why.words()).len(), 0);
    let pieces = text.matches("`<deployment>`").count().checked_add(1).expect("design's pieces fit");
    assert_eq!(why.words().len(), pieces);
}

#[test]
fn a_format_refusal_names_its_family_and_says_to_move_the_generations() {
    use crate::Family;
    let settings = unwrapped(FORMAT, "\n\n-").replace("<family>", "settings");
    for (family, name) in [(Family::Settings, "settings"), (Family::Application, "application")] {
        let words = OpenRefusal::Format { family, found: 2, wanted: 1 }.words().join("`<deployment>`");
        assert_eq!(words, settings.replacen("whose settings", &["whose ", name].concat(), 1), "{family:?}");
    }
}

#[test]
fn a_deployment_refusal_says_to_restore_whole_or_move_the_generations() {
    says(OpenRefusal::Deployment, DEPLOYMENT, "\n\n-");
}

#[test]
fn a_numbers_refusal_says_to_restore_whole_or_move_the_generations() {
    says(OpenRefusal::Numbers { store: 2, header: 1 }, NUMBERS, "\n\n-");
}

#[test]
fn a_failed_refusal_says_to_check_space_and_permissions_then_restore() {
    says(OpenRefusal::Failed, FAILED, "\n\n-");
}
#[test]
fn an_unplanned_layout_says_to_start_a_build_that_converts_it() {
    says(
        OpenRefusal::Layout {
            family: "fixture".into(),
            value_type: 0,
            id: 2,
            values: Some(1),
            why: crate::Unplanned::Text,
        },
        "- `Layout`: \"",
        "\"",
    );
}

#[test]
fn todays_layouts_refusal_says_to_move_its_three_directories() {
    says(OpenRefusal::OlderBuild { why: Older::Directory }, "  > ", "\n\n- **`OpenRefusal::Format`");
}
#[test]
fn a_busy_refusal_says_to_stop_the_other_process() {
    says(OpenRefusal::Generations { why: Generations::Busy }, "- `Busy`: \"", "\"");
}
#[test]
fn an_unreadable_refusal_says_to_restore_or_name_a_generation() {
    says(OpenRefusal::Generations { why: Generations::Unreadable }, "- `Unreadable`: \"", "\"");
}
#[test]
fn a_missing_refusal_says_to_restore_or_name_a_generation() {
    says(OpenRefusal::Generations { why: Generations::Missing { generation: 7 } }, "- `Missing`: \"", "\"");
}
#[test]
fn an_unnamed_refusal_says_to_name_its_generation() {
    says(OpenRefusal::Generations { why: Generations::Unnamed }, "- `Unnamed`: \"", "\"");
}
#[test]
fn a_checkpoints_refusal_says_to_restore_or_move_the_checkpoints() {
    says(OpenRefusal::Generations { why: Generations::Checkpoints }, "- `Checkpoints`: \"", "\"");
}
#[test]
fn a_layout_refusal_names_its_family_and_says_to_move_the_generations() {
    use crate::Family;
    says(
        OpenRefusal::OlderBuild { why: Older::Layout(Family::Settings) },
        "(the others name theirs):\n\n  > ",
        "\n\n## 4",
    );
    let settings = unwrapped("(the others name theirs):\n\n  > ", "\n\n## 4");
    for (family, clause) in [
        (Family::Component, "whose component family's"),
        (Family::Secrets, "whose secrets family's"),
        (Family::SkeinOauth, "whose kept skein-oauth records'"),
        (Family::SmithTranscript, "whose kept smith turns'"),
        (Family::Application, "whose application family's"),
    ] {
        let words = OpenRefusal::OlderBuild { why: Older::Layout(family) }.words().join("`<deployment>`");
        assert_eq!(words, settings.replacen("whose settings family's", clause, 1), "{family:?}");
    }
}

#[test]
fn a_job_refusal_says_the_designs_words() {
    says(
        OpenRefusal::Job { owner: crate::Owner::Jig, job: "fixture".into(), key: None, why: "refused".into() },
        "- `Job`: \"",
        "\"",
    );
}

#[test]
fn check_refusals_say_the_designs_words() {
    for (why, start) in [
        (Check::Load { key: Box::new([]) }, "- `Check { why: Load }`: \""),
        (Check::Private { name: Box::new([]) }, "- `Check { why: Private }`: \""),
    ] {
        says(OpenRefusal::Check { why }, start, "\"");
    }
}

#[test]
fn floor_and_unregistered_refusals_name_the_bridge() {
    says(OpenRefusal::OlderBuild { why: Older::Floor }, "- `OlderBuild { why: Floor }` (`records.md`, 3): \"", "\"");
    says(
        OpenRefusal::OlderBuild { why: Older::Unregistered },
        "- `OlderBuild { why: Unregistered }` (`layouts.md`, 5.1): \"",
        "\"",
    );
}

#[test]
fn ahead_and_admission_refusals_say_the_designs_words() {
    for (refusal, start) in [
        (OpenRefusal::Ahead { owner: crate::Owner::Jig, found: Some(2), wanted: Some(1) }, "- `Ahead`: \""),
        (OpenRefusal::Set { owner: crate::Owner::Application }, "- `Set`: \""),
    ] {
        says(refusal, start, "\"");
    }
}

#[test]
fn behind_and_unregistered_refusals_name_the_bridge_build() {
    for (why, start) in [
        (Older::Behind, "- `OlderBuild { why: Behind }`: \""),
        (Older::Unregistered, "- `OlderBuild { why: Unregistered }` (`layouts.md`, 5.1): \""),
    ] {
        says(OpenRefusal::OlderBuild { why }, start, "\"");
    }
}

#[test]
fn an_unfinished_restore_says_how_to_finish_or_replace_it() {
    says(OpenRefusal::Generations { why: Generations::Unmatched }, "- `Unmatched`: \"", "\"");
    says(OpenRefusal::Generations { why: Generations::Unfinished }, "- `Unfinished`: \"", "\"");
}
