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
        assert_no_repository_path(&name, &text);
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

fn assert_no_repository_path(name: &str, text: &str) {
    for path in ["docs/", "skein/", "smith/", "jig/", "temper/", "hearth/", "tools/", "examples/", ".config/"] {
        assert!(!text.contains(path), "{name}: repository path {path}");
    }
    for word in text.split(|ch: char| !ch.is_alphanumeric() && !"._-".contains(ch)) {
        let file = word.trim_end_matches('.');
        let markdown = file.rsplit_once('.').is_some_and(|(_, extension)| extension == "md");
        assert!(!markdown || file == "AGENTS.md", "{name}: Markdown file {file}");
    }
}

fn assembled_goal() -> temper_charter_set::GoalAgent<Reasons> {
    temper_charter_set::goal_agent(&crate::stock::charter_set(), |name| Reasons::named(name.as_bytes())).unwrap()
}

#[test]
fn the_goal_specs_keep_only_steps_at_the_boundary() {
    let assembled = assembled_goal();
    let instructions = core::str::from_utf8(&assembled.instructions).unwrap();
    assert!(instructions.contains("- `## Steps`: exactly what crosses the change's boundary (the public names, types and signatures other changes or the design rely on, cited by the design's section) and the order where order matters. No sketch of the code's shape and no example until a change's question or report shows it needs one. The rest is the implementer's. No estimate and no test commands: the implementer runs only its new tests and repairs, and delivery runs the checks."));
    assert!(!instructions.contains("sketch the code's place"));
    assert!(instructions.contains("- **Joined where changes meet:** where two changes meet at a line, wire or file format, a test runs one's output through the other, in the spec of the one whose `after` names the other."));
    let report = instructions.split("\n\n").find(|paragraph| paragraph.starts_with("**Your report note:**")).unwrap();
    assert!(report.contains("What the request asks for in its result goes in the body too."));
    let change = instructions.split("\n\n").find(|paragraph| paragraph.starts_with("**A change** for code")).unwrap();
    assert!(change.contains("While the change deleting the goal's contracts document is live, `amend` its `after` to name each change you cut."));
}

#[test]
fn the_designer_keeps_a_lean_design_and_temporary_contracts() {
    let assembled = assembled_goal();
    let part = assembled.parts.first().unwrap();
    assert_eq!(part.reasons.as_ref(), &[Reasons::FIRST]);
    let designer = core::str::from_utf8(&part.text).unwrap();
    assert!(designer.contains("1. **A lean design, eventually.** Every part of the request designed, in documents that outlast the goal. They keep concepts and their rules, invariants and transitions, stored formats and wires, failures and their order, order and time, trust, the edges between areas, and why; they leave to the code the types, fields and steps that no other area reads."));
    assert!(!designer.contains("A complete design, eventually"));
    assert!(designer.contains("- **A wave's design** comes before its changes: the sections the request asks for, keeping what a lasting design keeps, with wire and stored forms; the shapes its changes share and no other area reads (types, fields, signatures) as binding contracts in the goal's contracts document, a document of the goal's own named `contracts`, beside the area's design and with its own status where the project keeps status, which no lasting document cites; both by path and section; and the status the project keeps, as it will stand once the goal lands. Never edit the draft's text. `deliver` it before you cut the changes that cite it, by path and section."));
    assert!(!designer.contains("and the shapes that cross changes (types, fields, signatures, wire forms)"));
    assert!(designer.contains("- **Fewer concepts:** your note lists the concepts the goal adds and removes, with every public type it adds or removes, test kits' included."));
    assert!(designer.contains("- **The contracts document goes:** where the goal has one, cut a last change before you park, after every other, that deletes it and its status."));
}

#[test]
fn the_coordinator_prefers_fewer_concepts_and_corrects_the_design_on_results() {
    let assembled = assembled_goal();
    let questions = assembled.parts.iter().find(|part| part.reasons.contains(&Reasons::QUESTION)).unwrap();
    let questions = core::str::from_utf8(&questions.text).unwrap();
    assert!(questions.contains("- **Decide** by the design and spec, preferring the design's shape and, among options that keep it, the one with fewer concepts; accept a simplification that keeps the behaviour and the lasting design true. If wrong or missing a shared shape, revise and `deliver` it before answering, a shape the changes share going in the contracts document, with a **Design departs** entry for what is built differently."));
    assert!(!questions.contains("preferring the design's shape. If wrong or missing a shared shape"));
    let results = assembled.parts.iter().find(|part| part.reasons.contains(&Reasons::RESULT)).unwrap();
    let mut lines = core::str::from_utf8(&results.text).unwrap().lines();
    let ended = lines.find(|line| line.starts_with("- **A change ended**")).unwrap();
    assert!(ended.ends_with("With nothing left to cut, park: the closer lands the goal."));
    let detail = lines.next().unwrap();
    assert!(detail.starts_with("- **Detail on demand:**"));
    assert!(detail.contains("- **Detail on demand:** where a finished change's report, its `## Left` and `## Simpler` included, shows the design or a spec short or untrue, correct it before you park: what a lasting design keeps in the design, a shape changes share in the contracts document, the rest in the specs still to run, by `amend`. Never accept such a report as outside acceptance: correct the design, or have its status say Not yet or Design departs. Each correction goes in your note: what was missing or wrong, what showed it, and where it went."));
}

#[test]
fn the_implementer_reports_simplifications() {
    let engine = engine(&generated());
    let coder = engine.charters.iter().find(|charter| charter.number == 2).unwrap();
    let instructions = core::str::from_utf8(&coder.policy.instructions).unwrap();
    assert!(instructions.contains("- **Say where it repeats:** where the design or your spec makes you write the same thing twice, or a type with no behaviour of its own, say so: as a question when the simpler form crosses your change's edge, otherwise under `## Simpler` in your report, naming the place and the simpler form."));
}

#[test]
fn the_goals_report_headline_says_whats_done_and_left_for_a_person() {
    let assembled = assembled_goal();
    let instructions = core::str::from_utf8(&assembled.instructions).unwrap();
    let report = instructions
        .split("\n\n")
        .find(|paragraph| paragraph.starts_with("**Your report note:**"))
        .expect("goal report note");
    assert!(report.contains("a one-line headline for a person of what's done and what's left, in the work's own terms, with no task, change or pull numbers, codes (P1), branch (g) or words of how goals run (change, cut, wave, delivery, landing, repair, spec): \"Three of the four parts are done; the page part is still being built.\""));
    assert!(!report.contains("5 of its 7 changes"));
    assert!(instructions.contains("plain words saying what it does, at most 30 characters, no Markdown, codes or internal names (\"Cancel a landed change's runs\")"));
    let designer = core::str::from_utf8(&assembled.parts.first().unwrap().text).unwrap();
    assert!(!designer.contains("Your note's headline says the tree"));
}

#[test]
fn the_goals_report_left_line_names_uncut_and_undone_work() {
    let assembled = assembled_goal();
    let report = core::str::from_utf8(&assembled.instructions)
        .unwrap()
        .split("\n\n")
        .find(|paragraph| paragraph.starts_with("**Your report note:**"))
        .expect("goal report note");
    assert!(report.contains(
        "A `Left: ` line names what you planned and haven't cut or leave undone, and why; none if nothing: the closer holds on it."
    ));
}

#[test]
fn the_goals_report_note_answers_what_its_request_asks_for_in_its_result() {
    let assembled = assembled_goal();
    let mut paragraphs = core::str::from_utf8(&assembled.instructions).unwrap().split("\n\n");
    paragraphs.find(|paragraph| paragraph.starts_with("**Your report note:**")).expect("goal report note");
    let asked = paragraphs.next().expect("paragraph after the goal report note");
    assert!(
        asked.starts_with("**What your request asks for in its result**"),
        "expected the Asked block instructions after the goal report note, got: {asked}"
    );
    assert!(asked.contains(" an `Asked: ` line"));
    assert!(asked.contains("couldn't be had"));
    assert!(asked.contains("no blank line inside"));
    assert!(asked.contains("The goal's end report carries the block to your requester"));
}

#[test]
fn the_designer_parks_when_only_undone_work_is_left() {
    let assembled = assembled_goal();
    let designer = core::str::from_utf8(&assembled.parts.first().unwrap().text).unwrap();
    assert!(!designer.contains("designed and cut into changes. Then park."));
    assert!(designer.contains(
        "You are done when every part of the request is designed and cut into changes, or on your `Left:` line with why. Then park."
    ));
}

#[test]
fn the_coordinator_settles_left_work_before_releasing_the_closer() {
    let assembled = assembled_goal();
    let part = assembled
        .parts
        .iter()
        .find(|part| part.reasons.contains(&Reasons::RESULT) && part.reasons.contains(&Reasons::ESCALATION))
        .expect("goal results part");
    let words = core::str::from_utf8(&part.text).unwrap();
    let closer = words
        .lines()
        .find(|line| line.starts_with("- **Your closer's escalation**"))
        .expect("closer escalation bullet");
    assert!(closer.contains(
        "Held on your `Left:` line: cut what it names, or have the status mark a dropped part not built, why on the line; then release once."
    ));
    let ended = words.lines().find(|line| line.starts_with("- **A change ended**")).expect("change ended bullet");
    assert!(ended.ends_with("With nothing left to cut, park: the closer lands the goal."));
}

#[test]
fn the_coding_agents_charter_keeps_its_commits_and_writes_for_the_history() {
    let engine = engine(&generated());
    let charter = engine.charters.iter().find(|charter| charter.number == 2).unwrap();
    let text = core::str::from_utf8(&charter.policy.instructions).unwrap();
    assert!(text.contains(
        "Your branch's commits are yours, as many as you like and worded as you like, work in progress included"
    ));
    assert!(text.contains("none of them reaches your base but through your change's tree"));
    assert!(text.contains("Never push to your base or main, nor rewrite them."));
    assert!(text.contains(
        "Your `title` and your body's first paragraph, headings aside, become the commit your change lands as"
    ));
    assert!(text.contains("that paragraph, before the rest of the report, says what changed and why"));
}

#[test]
fn the_designers_charter_writes_deliveries_for_history_and_catches_up_only_for_a_change() {
    let engine = engine(&generated());
    let charter = engine.charters.iter().find(|charter| charter.number == 5).unwrap();
    let part = charter.parts.first().unwrap();
    assert_eq!(part.on, Reasons::FIRST);
    let bytes = [charter.policy.instructions.as_ref(), b"\n\n", part.instructions.as_ref()].concat();
    let text = core::str::from_utf8(&bytes).unwrap();
    assert!(text.contains("Each delivery stays on g as its own commit, never squashed or rewritten"));
    assert!(text.contains("write its title and body for the history"));
    assert!(text.contains("Deliver at any point, even while g lands: the closer lands the new head."));
    assert!(text.contains("**Main into g** only for a change that needs what main gained since g was made"));
    assert!(text.contains("`\"merge\"` naming main's tip (read it with `read_forge what=branch`), its words saying what it brings and for which change, named in that change's `after`"));
    assert!(text.contains("The merge job makes it, with no implementer"));
    assert!(text.contains("a conflict going to a resolution; with no merge job the call is refused: ask your person"));
    assert!(text.contains("Never take main to keep g current"));
    assert!(text.contains("cut no implementer's change whose only work is a catch-up"));
    assert!(text.contains("A change needing what main gained since g was made waits on its catch-up"));
    assert!(text.contains("waits on its catch-up (the rules' **Main into g**)"));
}

#[test]
fn the_coordinators_charter_writes_deliveries_for_history_and_catches_up_only_for_a_change() {
    let engine = engine(&generated());
    let charter = engine.charters.iter().find(|charter| charter.number == 5).unwrap();
    let part = charter.parts.iter().find(|part| part.on.contains(Reasons::QUESTION)).unwrap();
    let bytes = [charter.policy.instructions.as_ref(), b"\n\n", part.instructions.as_ref()].concat();
    let text = core::str::from_utf8(&bytes).unwrap();
    assert!(text.contains("Each delivery stays on g as its own commit, never squashed or rewritten"));
    assert!(text.contains("write its title and body for the history"));
    assert!(text.contains("Deliver at any point, even while g lands: the closer lands the new head."));
    assert!(text.contains("**Main into g** only for a change that needs what main gained since g was made"));
    assert!(text.contains("`\"merge\"` naming main's tip (read it with `read_forge what=branch`), its words saying what it brings and for which change, named in that change's `after`"));
    assert!(text.contains("The merge job makes it, with no implementer"));
    assert!(text.contains("a conflict going to a resolution; with no merge job the call is refused: ask your person"));
    assert!(text.contains("Never take main to keep g current"));
    assert!(text.contains("cut no implementer's change whose only work is a catch-up"));
    assert!(text.contains("Main's gains since g come by its catch-up (the rules' **Main into g**)"));
    assert!(text.contains("never an implementer's change merging main"));
    assert!(!core::str::from_utf8(&charter.policy.instructions).unwrap().contains("Make no catch-up of g"));
    for part in &charter.parts {
        assert!(!core::str::from_utf8(&part.instructions).unwrap().contains("Make no catch-up of g"));
    }
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
