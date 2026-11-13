//! The stock's source directory and startup room (domain/agent/charter.md, 1).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "the test lists stock source files and compares whole assembled texts"
)]
use super::{engine, generated};
use jig_domain_tasks::Reasons;
use temper_charter_set::File;
#[test]
fn the_stock_set_holds_exactly_five_files_and_assembles_both_charters() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/charters/stock");
    let mut names = alloc::collections::BTreeSet::new();
    for entry in std::fs::read_dir(&directory).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        let name = entry.file_name().into_string().unwrap();
        assert!(temper_charter_set::file(&name).unwrap().is_some(), "{name}");
        let text = std::fs::read_to_string(entry.path()).unwrap();
        assert!(text.ends_with('\n') && !text.ends_with("\n\n"));
        assert!(!text.contains("claude-") && !text.contains("gpt-"), "{name}");
        assert!(names.insert(name));
    }
    assert_eq!(
        names,
        [File::Common, File::Goal, File::Designer, File::Coordinator, File::Implementer]
            .into_iter()
            .map(|file| file.name().to_owned())
            .collect()
    );
    let common = std::fs::read_to_string(directory.join("common.md")).unwrap();
    let goal_text = std::fs::read_to_string(directory.join("goal.md")).unwrap();
    let designer = std::fs::read_to_string(directory.join("designer.md")).unwrap();
    let coordinator = std::fs::read_to_string(directory.join("coordinator.md")).unwrap();
    let implementer = std::fs::read_to_string(directory.join("implementer.md")).unwrap();
    let set = temper_charter_set::Set {
        common: Some(&common),
        goal: Some(&goal_text),
        designer: Some(&designer),
        coordinator: Some(&coordinator),
        implementer: Some(&implementer),
    };
    let assembled = temper_charter_set::goal_agent(&set, |name| Reasons::named(name.as_bytes())).unwrap();
    let engine = engine(&generated());
    let goal = engine.charters.iter().find(|charter| charter.number == 5).unwrap();
    assert_eq!(goal.policy.instructions, assembled.instructions);
    assert_eq!(
        goal.policy.instructions.as_ref(),
        format!("{}\n\n{}", common.trim_matches('\n'), goal_text.trim_matches('\n')).as_bytes()
    );
    assert_eq!(goal.parts.len(), 4);
    assert_eq!(goal.parts[0].on, Reasons::FIRST);
    assert_eq!(goal.parts[0].instructions.as_ref(), designer.trim_matches('\n').as_bytes());
    let sections: Box<[_]> =
        coordinator.split("## part, on:").skip(1).map(|section| section.split_once('\n').unwrap()).collect();
    assert_eq!(sections.len(), 3);
    for (part, (names, text)) in goal.parts[1..].iter().zip(sections) {
        let on = names
            .split(',')
            .fold(Reasons::EMPTY, |all, name| all.union(Reasons::named(name.trim().as_bytes()).unwrap()));
        assert_eq!(part.on, on);
        assert_eq!(part.instructions.as_ref(), text.trim_matches('\n').as_bytes());
    }
    let coder = engine.charters.iter().find(|charter| charter.number == 2).unwrap();
    assert_eq!(coder.policy.instructions, temper_charter_set::implementer(&set));
    assert_eq!(
        coder.policy.instructions.as_ref(),
        format!("{}\n\n{}", common.trim_matches('\n'), implementer.trim_matches('\n')).as_bytes()
    );
    assert!(coder.parts.is_empty());
}

#[test]
fn the_goal_parts_cover_every_choosing_kind_its_policy_wakes() {
    use jig_domain_tasks::{ResultsWake, WakeRule};
    let engine = engine(&generated());
    let charter = engine.charters.iter().find(|charter| charter.number == 5).unwrap();
    let mut needed =
        Reasons::ESCALATION.union(Reasons::PROPOSAL).union(Reasons::PERSON_STALLED).union(Reasons::NOTHING_AWAITED);
    for (rule, reason) in [
        (charter.wake.words, Reasons::WORDS),
        (charter.wake.notices, Reasons::NOTICE),
        (charter.wake.news, Reasons::NEWS),
    ] {
        match rule {
            WakeRule::Never => {}
            WakeRule::Immediate | WakeRule::Batch { .. } | WakeRule::Paced { .. } => needed = needed.union(reason),
        }
    }
    match charter.wake.results {
        ResultsWake::Never => {}
        ResultsWake::Each | ResultsWake::LastOrFailure => needed = needed.union(Reasons::RESULT),
    }
    if charter.wake.questions {
        needed = needed
            .union(Reasons::QUESTION)
            .union(Reasons::QUESTION_ANSWERED)
            .union(Reasons::QUESTION_ENDED)
            .union(Reasons::QUESTION_OWED);
    }
    if charter.wake.timers {
        needed = needed.union(Reasons::TIMER);
    }
    let covered = charter.parts.iter().fold(Reasons::EMPTY, |all, part| all.union(part.on));
    assert!(covered.contains(needed), "missing wake kinds: {:?}", Reasons(needed.0 & !covered.0));
}
