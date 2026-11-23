//! The watcher and tick loop over the fake world, never real git or agents.
use super::world::{Agent, Agents, Clock, Git, World, baseline, id, land};
use std::fs;
use temper_qa_tool::faces::RealFiles;
use temper_qa_tool::{
    Answer, BuildState, Exit, Ledger,
    faces::{Changed, Signal},
    runner::Run,
    watch::watch,
};

fn read(git: &mut Git, world: &World) {
    world
        .home
        .transact(&mut RealFiles, |ledger, _| watch(git, &world.home.configuration.git, ledger, 1))
        .expect("watch");
}

#[test]
fn watcher_sees_all_parents_once_across_rewinds_and_orders_two_landings() {
    let initial = World::new();
    let mut existing = baseline();
    existing.commit(1, &[0], "old tip");
    existing.merge(2, &[0, 1], 99);
    read(&mut existing, &initial);
    assert_eq!(initial.ledger().head, Some(id(2)));
    assert!(initial.ledger().landings.is_empty(), "first read never replays old landings");
    let world = World::new();
    let mut git = baseline();
    read(&mut git, &world);
    assert_eq!(world.ledger().head, Some(id(0)));
    assert_eq!(world.ledger().landings.len(), 0);
    git.commit(1, &[0], "goal's one-parent tip\n\nGoal: 7\n");
    git.merge(2, &[0, 1], 7);
    git.commit(3, &[0], "H1");
    git.commit(4, &[3, 2], "H2 carries MA off the first-parent line");
    read(&mut git, &world);
    assert_eq!(world.ledger().landings.iter().map(|l| (l.goal, l.commit.clone())).collect::<Vec<_>>(), [(7, id(2))]);
    assert!(!git.calls.contains(&format!("message {}", id(1))), "one-parent messages are never read");
    read(&mut git, &world);
    git.head = id(0);
    read(&mut git, &world);
    // A restart from the stored, rewound head must not replay MA.
    let reopened = temper_qa_tool::Home::open(world.scratch.path(), &mut RealFiles).expect("restart");
    git.head = id(4);
    reopened
        .transact(&mut RealFiles, |ledger, _| watch(&mut git, &reopened.configuration.git, ledger, 2))
        .expect("restart watch");
    assert_eq!(world.ledger().landings.len(), 1);
    git.commit(5, &[4], "one parent\n\nGoal: 88\n");
    git.merge(6, &[4, 5], 8);
    git.commit(7, &[6], "next goal tip");
    git.merge(8, &[6, 7], 9);
    read(&mut git, &world);
    assert_eq!(world.ledger().landings.iter().map(|l| l.goal).collect::<Vec<_>>(), [7, 8, 9]);
    assert!(!git.calls.contains(&format!("message {}", id(5))));
    let mut config = world.home.configuration.git.clone();
    config.fetch = Some("upstream".into());
    config.branch = "trunk".into();
    world.home.transact(&mut RealFiles, |ledger, _| watch(&mut git, &config, ledger, 3)).expect("fetch watch");
    assert_eq!(&git.calls[git.calls.len() - 2..], ["fetch upstream trunk", "head refs/temper-qa/trunk"]);
}

#[test]
fn builds_learn_holds_only_when_the_observed_head_contains_them() {
    let world = World::new();
    let mut git = baseline();
    read(&mut git, &world);
    git.commit(1, &[0], "tip");
    git.merge(2, &[0, 1], 7);
    world.build(2);
    git.head = id(0);
    read(&mut git, &world);
    assert_eq!(world.ledger().builds[0].holds, None);
    assert_eq!(world.ledger().builds[0].order, None);
    git.head = id(2);
    read(&mut git, &world);
    let ledger = world.ledger();
    assert_eq!(ledger.builds[0].holds, Some(vec![1]));
    assert_eq!(ledger.builds[0].order, Some(3));
    git.calls.clear();
    read(&mut git, &world);
    assert_eq!(git.calls, ["head refs/heads/main"], "known holds are final");
}

fn check_landing_base(parents: &[u64], branch_parent: u64, main_parent: u64, previous: bool, base: u64) {
    let world = World::new();
    let mut git = baseline();
    git.commit(1, &[0], "earlier change");
    git.merge(2, &[0, 1], 7);
    git.commit(3, &[branch_parent], "branch change\n\nQA: branch behaviour\n");
    git.commit(4, &[main_parent], "main change\n\nQA: main behaviour\n");
    git.merge(5, parents, 8);
    for parent in parents {
        let path = format!("temper/docs/design/draft/from-{parent}.md");
        git.file_at(5, &path, b"# Promise\n");
        git.nodes.get_mut(&id(5)).expect("landing").ranges.insert(
            id(*parent),
            super::world::Range {
                patch: format!("diff from {parent}\n").into_bytes(),
                stat: vec![Changed { path: format!("from-{parent}.rs"), added: Some(*parent), deleted: Some(0) }],
                paths: vec![path],
            },
        );
    }
    world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            ledger.record_head(&id(if parents.contains(&4) { 4 } else { 2 }));
            if previous {
                ledger.record_landing(&id(2), 7, None, "earlier landing", 0);
                let session = super::session(ledger, 1);
                ledger.answer_none(session, "already triaged")?;
                ledger.end_session(session, Exit::Code(0), None, 0, "qa")?;
            }
            Ok(())
        })
        .expect("previous landing");
    let mut run = world.start(git, &[]);
    run.tick();
    let ledger = world.ledger();
    let session = ledger.running_sessions().into_iter().next().expect("triage");
    let directory = world.home.session_directory(session.number);
    let brief = fs::read_to_string(directory.join("brief")).expect("brief");
    assert_eq!(fs::read(directory.join("diff.patch")).expect("patch"), format!("diff from {base}\n").as_bytes());
    assert!(brief.contains(&format!("from-{base}.rs | +{base} -0")), "{brief}");
    assert!(brief.contains(&format!("temper/docs/design/draft/from-{base}.md")), "{brief}");
    for other in parents.iter().filter(|parent| **parent != base) {
        assert!(!brief.contains(&format!("from-{other}.rs")), "{brief}");
        assert!(!brief.contains(&format!("temper/docs/design/draft/from-{other}.md")), "{brief}");
    }
    let (included, excluded) = if base == 3 { ("main", "branch") } else { ("branch", "main") };
    assert!(brief.contains(&format!("QA: {included} behaviour")), "{brief}");
    assert!(!brief.contains(&format!("QA: {excluded} behaviour")), "{brief}");
    for operation in ["diff", "stat", "paths", "qa"] {
        assert!(run.git.calls.contains(&format!("{operation} {} {}", id(base), id(5))));
    }
}

#[test]
fn by_hand_landing_reads_inputs_against_main_as_second_parent() {
    check_landing_base(&[3, 2], 0, 2, true, 2);
    check_landing_base(&[3, 4], 0, 2, true, 4);
}

#[test]
fn forge_landing_reads_inputs_against_main_as_first_parent() {
    check_landing_base(&[2, 3], 0, 2, true, 2);
    check_landing_base(&[4, 3], 0, 2, true, 4);
}

#[test]
fn first_landing_reads_inputs_against_first_parent() {
    check_landing_base(&[3, 4], 0, 2, false, 3);
}

#[test]
fn both_parents_holding_previous_landing_reads_inputs_against_first_parent() {
    check_landing_base(&[3, 4], 2, 2, true, 3);
}

#[test]
fn neither_parent_holding_previous_landing_reads_inputs_against_first_parent() {
    check_landing_base(&[3, 4], 0, 0, true, 3);
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one serial triage session pins its inputs, fakes-only brief and environment"
)]
fn triage_is_serial_and_its_brief_and_environment_are_the_merges_inputs() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::None]);
    run.tick();
    land(&mut run, 2, 7);
    land(&mut run, 4, 8);
    run.git.nodes.get_mut(&id(2)).expect("merge").message =
        "Goal 7's title\n\nDraft: `temper/docs/design/draft/x.execution.md`\n\nGoal: 7\nPull-request: 12\nQA: Work lists drafts\nQA: Work lists drafts\n".into();
    for (path, bytes) in [
        ("temper/docs/design/draft/x.md", b"# Promise\n".as_slice()),
        ("temper/docs/design/draft/other.md", b"# Other\n".as_slice()),
        (
            "temper/docs/development/qa/setups/project/README.md",
            b"# Project\n\nA project ready for work.\nTwo lines.\n\nLater paragraph.".as_slice(),
        ),
    ] {
        run.git.file_at(2, path, bytes);
    }
    let node = run.git.nodes.get_mut(&id(2)).expect("merge");
    node.paths.push("temper/docs/design/draft/other.status.md".into());
    node.stat = [("x/golden/3", 10, 0), ("a/src/lib.rs", 3, 1), ("x/golden/1", 10, 0), ("x/golden/2", 10, 0)]
        .into_iter()
        .map(|(path, added, deleted)| Changed { path: path.into(), added: Some(added), deleted: Some(deleted) })
        .collect();
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.sessions.len(), 1);
    assert_eq!(ledger.sessions[0].landing, Some(1));
    assert_eq!(ledger.sessions[0].effort, "cheap");
    assert_eq!(ledger.sessions[0].pages["web"], ["chat", "work"]);
    assert_eq!(ledger.sessions[0].pages["tui"], ["chat"]);
    assert_eq!(ledger.sessions[0].setups, ["fresh", "project"]);
    let command = &run.processes.commands[0];
    assert_eq!(command.arguments, ["agent", "--effort=cheap"]);
    assert_eq!(command.directory, world.home.triage_tree());
    assert_eq!(command.output, world.home.session_directory(1).join("output"));
    for (key, value) in [
        ("CAPTURED", "kept".to_owned()),
        ("TEMPER_INTERNAL_QA_HOME", world.home.directory.display().to_string()),
        ("TEMPER_INTERNAL_QA_SESSION", "1".to_owned()),
    ] {
        assert!(
            command
                .environment
                .iter()
                .any(|(name, bytes)| name.as_ref() == key.as_bytes() && bytes.as_ref() == value.as_bytes())
        );
    }
    let brief = fs::read_to_string(&command.stdin).expect("brief");
    let mut previous = 0;
    for heading in [
        "# Triage",
        "## Landing",
        "## Drafts",
        "## Diff",
        "## QA trailers",
        "## Surfaces",
        "## Setups",
        "## Queued cases",
        "## Commands",
    ] {
        let at = brief.find(heading).expect("ordered section");
        assert!(at >= previous);
        previous = at;
    }
    for words in [
        "goal 7",
        "Goal 7's title",
        "temper/docs/design/draft/x.md",
        "## Diff\n\na/src/lib.rs | +3 -1\nx/golden/ | 3 files, +30 -0\n4 files changed, +33 -1\n\nWhole diff:",
        "sessions/1/diff.patch",
        "QA: Work lists drafts",
        "web: chat",
        "tui: chat",
        "fresh",
        "A project ready for work. Two lines.",
        "temper-qa triage none WHY",
        "temper-qa triage case FILE",
        "The fakes alone",
        "the person's forge is the box's fake Forgejo",
        "what a person meets that these cannot show is a gap",
        "is a gap: `temper-qa triage gap KIND WHY`, never none",
        "\"expect\"",
    ] {
        assert!(brief.contains(words), "{words}\n{brief}");
    }
    assert!(!brief.contains("other.md"), "named drafts take precedence over the diff");
    assert_eq!(brief.matches("QA: Work lists drafts").count(), 1);
    assert_eq!(
        fs::read(world.home.session_directory(1).join("diff.patch")).expect("patch"),
        b"diff --git a/page b/page\n+new page\n"
    );
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.landings[0].answer, Some(Answer::Cases));
    assert_eq!(ledger.landings[0].triaged, Some(0));
    assert_eq!(ledger.sessions[0].exit, Some(Exit::Code(0)));
    assert_eq!(ledger.sessions[0].tokens, Some(1234));
    assert_eq!(ledger.sessions.len(), 2);
    assert_eq!(ledger.sessions[1].landing, Some(2));
    assert!(world.home.triage_tree().is_dir());
    let brief = fs::read_to_string(world.home.session_directory(2).join("brief")).expect("second brief");
    for words in ["Case 1: web / work; landings 1", "setup fresh", "open Work", "Drafts lists the draft"] {
        assert!(brief.contains(words), "{words}");
    }
    run.tick();
    assert_eq!(world.ledger().running_sessions().len(), 0);
}

#[test]
fn drafts_fall_back_to_changed_status_and_execution_files_or_say_the_title_is_the_request() {
    for touched in [true, false] {
        let world = World::new();
        let mut run = world.start(baseline(), &[]);
        run.tick();
        land(&mut run, 2, 7);
        run.git.file_at(2, "temper/docs/design/draft/x.md", b"# Draft\n");
        let node = run.git.nodes.get_mut(&id(2)).expect("merge");
        node.paths = if touched {
            vec![
                "temper/docs/design/draft/x.status.md".into(),
                "temper/docs/design/draft/x.execution.md".into(),
                "temper/docs/design/draft/notes.md".into(),
                "temper/docs/design/draft/archive/old.md".into(),
            ]
        } else {
            vec![]
        };
        run.tick();
        let brief = fs::read_to_string(world.home.session_directory(1).join("brief")).expect("brief");
        if touched {
            assert_eq!(brief.matches("temper/docs/design/draft/x.md").count(), 1);
        } else {
            assert!(brief.contains("No draft is named; the request was the title's words."));
        }
        assert!(!brief.contains("notes.md"));
        assert!(!brief.contains("archive/old.md"));
    }
}

#[test]
fn empty_agents_twice_are_unanswered_but_bound_agents_are_stopped_at_the_deadline() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Empty, Agent::Empty]);
    run.tick();
    land(&mut run, 2, 7);
    run.tick();
    run.tick();
    assert_eq!(world.ledger().landings[0].sessions, 1);
    assert_eq!(world.ledger().sessions.len(), 2);
    run.tick();
    let ledger = world.ledger();
    assert!(ledger.landings[0].unanswered);
    assert_eq!(ledger.landings[0].sessions, 2);
    assert!(ledger.landings[0].triaged.is_some());
    assert_eq!(ledger.lines.len(), 1);
    assert!(ledger.lines[0].words.contains("triage recorded nothing in two sessions; triaged as unanswered"));
    run.tick();
    assert_eq!(world.ledger().lines.len(), 1);
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.tick();
    run.clock.time = 29;
    run.tick();
    assert_eq!(run.processes.signals.len(), 0);
    run.clock.time = 30;
    run.tick();
    assert_eq!(run.processes.signals, [(1, Signal::Term)]);
    assert_eq!(world.ledger().sessions[0].exit, Some(Exit::Bound));
    run.clock.time = 39;
    run.tick();
    assert_eq!(run.processes.signals.len(), 1);
    run.clock.time = 40;
    run.tick();
    assert_eq!(run.processes.signals, [(1, Signal::Term), (1, Signal::Kill)]);
}

#[test]
fn a_group_ignoring_term_is_killed_ten_seconds_later() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::IgnoreTerm]);
    run.tick();
    land(&mut run, 2, 7);
    run.tick();
    run.clock.time = 30;
    run.tick();
    assert_eq!(run.processes.signals, [(1, Signal::Term)]);
    assert_eq!(world.ledger().sessions[0].exit, None);
    run.clock.time = 39;
    run.tick();
    assert_eq!(world.ledger().sessions[0].exit, None);
    run.clock.time = 40;
    run.tick();
    assert_eq!(run.processes.signals.last(), Some(&(1, Signal::Kill)));
    assert_eq!(world.ledger().sessions[0].exit, Some(Exit::Bound));
}

#[test]
fn refusing_a_worktree_or_command_ends_unstarted_and_leaves_the_landing_count_unchanged() {
    for git_refuses in [true, false] {
        let world = World::new();
        let mut run = world.start(baseline(), &[Agent::Unstarted]);
        run.tick();
        land(&mut run, 2, 7);
        run.git.fail_add = git_refuses;
        run.tick();
        let ledger = world.ledger();
        assert_eq!(ledger.sessions.len(), 1);
        assert_eq!(ledger.sessions[0].exit, Some(Exit::Unstarted));
        assert_eq!(ledger.landings[0].sessions, 0);
        assert_eq!(ledger.landings[0].triaged, None);
        assert_eq!(world.home.triage_tree().is_dir(), !git_refuses);
        run.git.fail_add = false;
        run.processes.next.clear();
        run.processes.next.push_back(Agent::None);
        run.tick();
        assert_eq!(world.ledger().sessions[1].exit, None);
        run.tick();
        assert!(world.ledger().landings[0].triaged.is_some());
    }
}

#[test]
fn a_goal_answered_no_qa_finishes_and_its_build_is_deleted_even_if_already_gone() {
    for already_gone in [false, true] {
        let world = World::new();
        let mut run = world.start(baseline(), &[Agent::None]);
        run.tick();
        land(&mut run, 2, 7);
        let directory = world.build(2);
        run.tick();
        assert!(directory.exists());
        if already_gone {
            fs::remove_dir_all(&directory).expect("deployment already removed it");
        }
        run.tick();
        let ledger = world.ledger();
        assert_eq!(ledger.landings[0].goal, 7);
        assert_eq!(ledger.landings[0].answer, Some(Answer::None { why: "internal refactor".into() }));
        assert!(ledger.landings[0].triaged.is_some());
        assert_eq!(ledger.sessions[0].exit, Some(Exit::Code(0)));
        assert_eq!(ledger.builds[0].state, BuildState::Deleted);
        assert_eq!(ledger.builds[0].deleted, Some(0));
        assert!(!directory.exists());
        assert!(world.home.session_directory(1).join("brief").is_file());
        assert!(world.home.session_directory(1).join("output").is_file());
    }
}

#[test]
fn newer_reached_build_releases_the_old_one_but_a_decoded_hold_keeps_it() {
    for held in [false, true] {
        let world = World::new();
        let mut run = world.start(baseline(), &[Agent::Case, Agent::None]);
        run.tick();
        land(&mut run, 2, 7);
        let old = world.build(2);
        run.tick();
        run.tick();
        assert!(old.exists(), "newest reached build for the queued case");
        if held {
            let mut document = world.ledger().document();
            document["builds"][0]["held"] = serde_json::json!(91);
            let decoded = Ledger::decode(&serde_json::to_vec(&document).expect("document")).expect("held document");
            fs::write(&world.home.ledger, decoded.encode().expect("ledger")).expect("held on disk");
        }
        land(&mut run, 4, 8);
        let new = world.build(4);
        run.tick();
        assert!(old.exists(), "new build is not reached yet");
        run.tick();
        assert_eq!(old.exists(), held);
        assert!(new.exists());
        assert_eq!(world.ledger().builds[0].state, if held { BuildState::Kept } else { BuildState::Deleted });
        assert_eq!(world.ledger().builds[1].state, BuildState::Kept);
    }
}

#[test]
fn waiting_cases_make_one_line_and_notify_gets_it_once_despite_its_exit() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case]);
    run.tick();
    land(&mut run, 2, 7);
    run.tick();
    run.tick();
    run.clock.time = 89;
    run.tick();
    assert_eq!(world.ledger().lines.len(), 0);
    run.clock.time = 90;
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.lines.len(), 1);
    assert!(ledger.lines[0].given);
    assert!(ledger.lines[0].words.contains("has waited past 90 seconds for a kept build holding its landings"));
    assert_eq!(run.processes.notifications, [format!("qa: {}\n", ledger.lines[0].words).into_bytes()]);
    run.clock.time = 180;
    run.tick();
    run.tick();
    assert_eq!(world.ledger().lines.len(), 1);
    assert_eq!(run.processes.notifications.len(), 1);
    let notify = run.processes.commands.last().expect("notify command");
    assert_eq!(notify.arguments, ["notify"]);
    assert_eq!(notify.directory, world.home.directory);
    assert_eq!(notify.environment.len(), 1);
}

#[test]
fn a_step_failing_for_an_hour_is_one_line_until_it_succeeds() {
    let world = World::new();
    let mut run = world.start(baseline(), &[]);
    run.git.fail_head = true;
    run.tick();
    assert_eq!(world.ledger().head, None);
    run.clock.time = 3599;
    run.tick();
    assert_eq!(world.ledger().lines.len(), 0);
    run.clock.time = 3600;
    run.tick();
    assert_eq!(world.ledger().lines.len(), 1);
    assert!(world.ledger().lines[0].words.contains("watch has failed for an hour: head refused"));
    run.clock.time = 7200;
    run.tick();
    assert_eq!(world.ledger().lines.len(), 1);
    run.git.fail_head = false;
    run.tick();
    assert_eq!(world.ledger().head, Some(id(0)));
    run.git.fail_head = true;
    run.tick();
    run.clock.time = 10800;
    run.tick();
    assert_eq!(world.ledger().lines.len(), 2);
}

#[test]
fn cycles_wait_for_events_and_the_nearest_bound_without_polling() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::None]);
    run.tick();
    assert_eq!(run.wait_timeout().expect("idle timeout"), None);
    land(&mut run, 2, 7);
    run.waker().poke();
    run.cycle();
    assert_eq!(run.clock.time, 0, "a poke runs a tick at once");
    assert_eq!(world.ledger().sessions.len(), 1);
    run.cycle();
    assert_eq!(world.ledger().sessions[0].ended, Some(0), "exit wakes without advancing the clock");
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.tick();
    assert_eq!(run.wait_timeout().expect("bound"), Some(30));
    run.cycle();
    assert_eq!(run.clock.time, 30);
    assert_eq!(run.processes.signals, [(1, Signal::Term)]);
    assert_eq!(world.ledger().sessions[0].exit, Some(Exit::Bound));
    assert_eq!(run.wait_timeout().expect("pending kill"), Some(10));
}

#[test]
fn one_run_holds_the_home_and_restart_ends_every_running_session_lost_and_prunes() {
    let world = World::new();
    world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            ledger.record_landing(&id(2), 7, None, "landed", 0);
            ledger.start_session(1, "cheap", super::pages(), vec!["fresh".into()], Vec::new(), 0)?;
            Ok(())
        })
        .expect("running session");
    // Recovery also handles more than one live record read from a document.
    let mut document = world.ledger().document();
    let mut second = document["sessions"][0].clone();
    second["number"] = serde_json::json!(2);
    document["sessions"].as_array_mut().expect("sessions").push(second);
    fs::write(&world.home.ledger, serde_json::to_vec(&document).expect("document")).expect("two running sessions");
    for number in [1, 2] {
        fs::create_dir_all(world.home.session_directory(number).join("tree")).expect("stale tree");
    }
    let run = world.start(baseline(), &[]);
    let ledger = world.ledger();
    assert!(ledger.sessions.iter().all(|session| session.exit == Some(Exit::Lost)));
    assert_eq!(ledger.landings[0].sessions, 2);
    assert!(ledger.landings[0].unanswered);
    assert_eq!(ledger.running_sessions().len(), 0);
    for number in [1, 2] {
        assert!(!world.home.session_directory(number).join("tree").exists());
    }
    assert_eq!(run.git.prunes, 1);
    let why =
        Run::start(world.home.clone(), baseline(), Agents::default(), Clock::default(), vec![]).err().expect("locked");
    assert_eq!(why.to_string(), format!("another temper-qa run holds {}", world.home.directory.display()));
    drop(run);
    let restarted = world.start(baseline(), &[]);
    assert_eq!(world.ledger().landings[0].sessions, 2);
    drop(restarted);
}

fn request_directory(world: &World, pass: u64, request: u64) -> std::path::PathBuf {
    world.home.directory.join("passes").join(pass.to_string()).join("requests").join(request.to_string())
}
fn deployment(world: &World, name: &str, body: &str) -> Vec<String> {
    let script = world.home.directory.join(format!("{name}.sh"));
    fs::write(&script, format!(r#"set -eu
printf '%s\n' "$PWD" "$CAPTURED" "$TEMPER_INTERNAL_QA_HOME" "$TEMPER_INTERNAL_QA_AREA" "$@" >> "$TEMPER_INTERNAL_QA_HOME/{name}-calls"
find . -type f | sort > "$TEMPER_INTERNAL_QA_HOME/{name}-files"
if read -r ignored; then exit 94; fi
{body}
"#)).expect("deployment script");
    vec!["/bin/sh".into(), script.display().to_string()]
}
fn seed_request(world: &World, count: u64, time: u64, page: &str, commit: &str) -> u64 {
    let pass = world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            let case = super::passes::queue(ledger, page, "web");
            let holds = ledger.landings.iter().map(|landing| landing.number).collect::<Vec<_>>();
            super::passes::build(ledger, commit, &holds, time);
            let start = super::passes::start(ledger);
            let mut findings = Vec::new();
            for index in 0..count {
                let mut finding = super::passes::finding(case);
                finding.title = format!("Start does nothing {index}");
                finding.setup = "recipe".into();
                finding.lines = vec!["web open work".into(), format!("web press button Start{index}")];
                finding.expected = format!("goal {index} starts");
                findings
                    .push(ledger.record_finding(start.session, &finding, &["recipe".into()], "product", time)?.number);
            }
            ledger.record_outcome(
                start.session,
                case,
                if count == 0 { "passed" } else { "failed" },
                "tested",
                &findings,
                time,
            )?;
            ledger.end_pass_session(start.session, Exit::Code(0), None, time, "qa", &["recipe".into()])?;
            Ok(start.pass)
        })
        .expect("request through ledger operations");
    let recipe = world.home.directory.join("passes").join(pass.to_string()).join("setups/recipe");
    fs::create_dir_all(recipe.join("nested")).expect("recipe directory");
    fs::write(recipe.join("README.md"), "# Recipe\n\nThe whole recipe.\n").expect("recipe");
    fs::write(recipe.join("nested/bytes"), [0, 1, 255, 10]).expect("nested binary recipe file");
    pass
}

#[test]
fn requests_write_the_document_recipes_message_and_goal_then_record_stdout_goal() {
    let mut world = World::new();
    let commit = "abcdef1234567890abcdef1234567890abcdef12";
    seed_request(&world, 2, 5, "work", commit);
    world.home.configuration.request.area = "fixes".into();
    world.home.configuration.request.command = deployment(
        &world,
        "file",
        r#"
cp -R tree "$TEMPER_INTERNAL_QA_HOME/received"
cat message > "$TEMPER_INTERNAL_QA_HOME/received-message"
cat goal > "$TEMPER_INTERNAL_QA_HOME/received-goal"
printf 'progress\ngoal 41\n\n \n'
printf 'goal 99\n' >&2
"#,
    );
    let mut run = world.start_with_environment(
        baseline(),
        &[],
        vec![
            (b"CAPTURED".as_slice().into(), b"kept".as_slice().into()),
            (b"TEMPER_INTERNAL_QA_HOME".as_slice().into(), b"stale-home".as_slice().into()),
            (b"TEMPER_INTERNAL_QA_AREA".as_slice().into(), b"stale-area".as_slice().into()),
        ],
    );
    run.clock.time = 10;
    run.tick();
    let ledger = world.ledger();
    assert_eq!((ledger.passes[0].requests[0].filed, ledger.passes[0].requests[0].goal), (Some(10), Some(41)));
    assert!(ledger.findings.iter().all(|finding| finding.goals == [41]));
    let directory = request_directory(&world, 1, 1);
    let path = "temper/docs/development/qa/requests/pass-1-1.md";
    let document = fs::read_to_string(directory.join("tree").join(path)).expect("request document");
    assert!(document.starts_with("# QA pass 1, request 1: 2 findings at abcdef123456\n"));
    assert!(document.contains("lowest world that reproduces it, starting from the finding's lines"));
    assert!(document.contains("what the cases expected and nothing past it"));
    for (number, index) in [(1, 0), (2, 1)] {
        let mut previous = 0;
        for words in [
            format!("## {number}. Start does nothing {index}"),
            "Severity: medium".into(),
            "Surface: web".into(),
            "Page: work".into(),
            "Area: product".into(),
            "goal 11: a landing".into(),
            "Setup: temper/docs/development/qa/setups/recipe/".into(),
            format!("```text\nweb open work\nweb press button Start{index}\n```"),
            format!("Expected: goal {index} starts"),
            "Seen: no goal".into(),
            format!("evidence/{number}/look.png"),
            "Earlier goals: none".into(),
        ] {
            let section = document.split(&format!("## {number}. ")).nth(1).expect("finding section");
            let at = if words.starts_with("## ") {
                0
            } else {
                section.find(&words).unwrap_or_else(|| panic!("{words}\n{section}"))
            };
            assert!(at >= previous, "{words} is in document order");
            previous = at;
        }
    }
    assert!(document.ends_with("## Recipes\n\n- temper/docs/development/qa/setups/recipe/\n"));
    assert_eq!(
        fs::read_to_string(world.home.directory.join("received").join(path)).expect("command saw files"),
        document
    );
    let setup = directory.join("tree/temper/docs/development/qa/setups/recipe");
    assert_eq!(fs::read_to_string(setup.join("README.md")).expect("whole recipe"), "# Recipe\n\nThe whole recipe.\n");
    assert_eq!(fs::read(setup.join("nested/bytes")).expect("nested file"), [0, 1, 255, 10]);
    assert_eq!(
        fs::read_to_string(world.home.directory.join("received-message")).expect("message"),
        "QA pass 1, request 1: 2 findings at abcdef123456\n\nQA-pass: 1\n"
    );
    assert_eq!(
        fs::read_to_string(world.home.directory.join("received-goal")).expect("goal"),
        format!(
            "Read the QA request at {path} and fix each finding it lists, each with a test in the lowest world that reproduces it, starting from its lines.\n"
        )
    );
    assert_eq!(
        fs::read_to_string(world.home.directory.join("file-calls")).expect("environment"),
        format!("{}\nkept\n{}\nfixes\n", directory.display(), world.home.directory.display())
    );
    let output = fs::read_to_string(directory.join("output")).expect("output");
    assert!(output.contains("progress\ngoal 41"));
    assert!(output.contains("goal 99"));
    assert_eq!(run.processes.bounded[0].1, 600);
    for name in ["TEMPER_INTERNAL_QA_HOME", "TEMPER_INTERNAL_QA_AREA"] {
        assert_eq!(
            run.processes.bounded[0].0.environment.iter().filter(|(key, _)| key.as_ref() == name.as_bytes()).count(),
            1
        );
    }
    run.tick();
    assert_eq!(run.processes.bounded.len(), 1, "filed request is not run again; no state command configured");
}

#[test]
fn recipes_alone_are_filed_without_a_document_or_goal_and_replace_stale_files() {
    let mut world = World::new();
    seed_request(&world, 0, 5, "work", &id(0));
    let directory = request_directory(&world, 1, 1);
    fs::create_dir_all(directory.join("tree")).expect("old attempt");
    fs::write(directory.join("goal"), "stale goal").expect("old goal");
    fs::write(directory.join("tree/stale"), "stale tree").expect("old tree");
    world.home.configuration.request.command = deployment(
        &world,
        "file",
        r#"
test ! -e goal
test ! -e tree/stale
test ! -e tree/temper/docs/development/qa/requests
cat message > "$TEMPER_INTERNAL_QA_HOME/received-message"
printf 'goal 999\n\377\n'
"#,
    );
    let mut run = world.start(baseline(), &[]);
    run.tick();
    assert_eq!(world.ledger().passes[0].requests[0].filed, Some(0));
    assert_eq!(world.ledger().passes[0].requests[0].goal, None);
    assert!(!directory.join("goal").exists());
    assert!(!directory.join("tree/temper/docs/development/qa/requests").exists());
    assert!(directory.join("tree/temper/docs/development/qa/setups/recipe/nested/bytes").is_file());
    assert_eq!(
        fs::read_to_string(world.home.directory.join("received-message")).expect("message"),
        "QA pass 1: recipes\n\nQA-pass: 1\n"
    );
    assert!(fs::read_to_string(world.home.directory.join("file-calls")).expect("owner fallback").contains("\nqa\n"));
}

#[test]
fn failed_filing_retries_each_tick_and_reports_one_line_after_an_hour() {
    for body in [
        "printf 'goal 41\\n'; exit 1",
        "printf 'goal 41\\nnot a goal\\n'",
        "printf 'goal 41\\n' >&2",
        "printf 'goal +41\\n'",
        "printf 'goal 0\\n'",
        "printf 'goal 18446744073709551616\\n'",
    ] {
        let mut world = World::new();
        seed_request(&world, 1, 5, "work", &id(0));
        world.home.configuration.request.command = deployment(&world, "file", body);
        let mut run = world.start(baseline(), &[]);
        for (time, lines) in [(0, 0), (3599, 0), (3600, 1), (7200, 1)] {
            run.clock.time = time;
            run.tick();
            let ledger = world.ledger();
            assert_eq!(ledger.passes[0].requests[0].filed, None, "{body}");
            assert_eq!(ledger.passes[0].requests[0].goal, None);
            assert_eq!(ledger.findings[0].goals.len(), 0);
            assert_eq!(ledger.lines.len(), lines);
        }
        assert_eq!(run.processes.bounded.len(), 4, "retry each tick");
        assert!(world.ledger().lines[0].words.contains("requests has failed for an hour"));
        deployment(&world, "file", "printf 'goal 41\\n'");
        run.tick();
        assert_eq!(world.ledger().passes[0].requests[0].goal, Some(41));
    }
}

#[test]
fn goal_state_live_waits_and_ended_allows_resume_on_the_newest_reached_build() {
    let mut world = World::new();
    seed_request(&world, 1, 5, "work", &id(0));
    world.home.configuration.request.command = deployment(&world, "file", "printf 'goal 41\\n'");
    world.home.configuration.request.state = Some(
        deployment(
            &world,
            "state",
            r#"
test "$1" = status
test "$2" = 41
cat "$TEMPER_INTERNAL_QA_HOME/state-answer"
printf 'ended\n' >&2
"#,
        )
        .into_iter()
        .chain(["status".into()])
        .collect(),
    );
    fs::write(world.home.directory.join("state-answer"), "progress\nlive\n\n").expect("live");
    let mut run = world.start(baseline(), &[]);
    run.tick();
    let before = world.ledger();
    assert_eq!(before.passes[0].requests[0].goal, Some(41), "file before polling state");
    assert_eq!(before.passes[0].requests[0].ended, None, "stderr cannot end the goal");
    assert_eq!(run.processes.bounded.iter().map(|(_, seconds)| *seconds).collect::<Vec<_>>(), [600, 60]);
    assert_eq!(run.wait_timeout().expect("waiting goal check"), Some(60));
    run.tick();
    assert_eq!(world.ledger(), before, "live changes nothing");
    fs::write(world.home.directory.join("state-answer"), "ended\n\n").expect("ended");
    run.clock.time = 20;
    run.tick();
    assert_eq!(world.ledger().passes[0].requests[0].ended, Some(20));
    assert_eq!(run.wait_timeout().expect("ended goal disarms state check"), None);
    let calls = run.processes.bounded.len();
    run.tick();
    assert_eq!(run.processes.bounded.len(), calls, "ended goals are not polled again");
    assert!(
        fs::read_to_string(request_directory(&world, 1, 1).join("output"))
            .expect("filing output retained")
            .contains("goal 41")
    );
    world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            super::passes::build(ledger, &id(1), &[1], 100);
            let resumed = ledger
                .resume_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 21)
                .expect("resume allowed by ended");
            assert_eq!(resumed.build, id(1));
            Ok(())
        })
        .expect("resume through ledger rule");
}

#[test]
fn a_second_request_names_the_earlier_goal_that_carried_the_finding() {
    let mut world = World::new();
    seed_request(&world, 1, 5, "work", &id(0));
    world.home.configuration.request.command = deployment(&world, "file", "printf 'goal 41\\n'");
    let mut run = world.start(baseline(), &[]);
    run.tick();
    world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            ledger.end_goal(41, 10)?;
            let resumed =
                ledger.resume_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 11).expect("resume");
            ledger.record_check(resumed.session, 1, "holds", "still fails", 12)?;
            ledger.end_pass_session(resumed.session, Exit::Code(0), None, 13, "qa", &[])
        })
        .expect("second request");
    deployment(&world, "file", "printf 'goal 42\\n'");
    run.tick();
    let document = fs::read_to_string(
        request_directory(&world, 1, 2).join("tree/temper/docs/development/qa/requests/pass-1-2.md"),
    )
    .expect("second document");
    assert!(document.contains("Earlier goals:\n- goal 41\n"));
    assert_eq!(world.ledger().findings[0].goals, [41, 42]);
}

#[test]
fn goal_state_failures_retry_but_unconfigured_or_landed_goals_are_not_polled() {
    for body in ["printf 'live\\n'; exit 1", "printf 'other\\n'", "printf 'ended\\n' >&2"] {
        let mut world = World::new();
        seed_request(&world, 1, 5, "work", &id(0));
        world.home.configuration.request.command = deployment(&world, "file", "printf 'goal 41\\n'");
        let mut run = world.start(baseline(), &[]);
        run.tick();
        assert_eq!(run.processes.bounded.len(), 1, "no state command means no probe");
        run.home.configuration.request.state = Some(deployment(&world, "state", body));
        run.tick();
        run.clock.time = 3600;
        run.tick();
        assert_eq!(world.ledger().passes[0].requests[0].ended, None);
        assert!(world.ledger().lines.iter().any(|line| line.words.contains("requests has failed for an hour")));
        assert_eq!(run.processes.bounded.len(), 3);
        world
            .home
            .transact(&mut RealFiles, |ledger, _| {
                let landing = ledger.record_landing(&id(2), 41, None, "goal's landing", 3601).expect("new landing");
                let session = super::session(ledger, landing);
                ledger.answer_none(session, "fix landed")?;
                ledger.end_session(session, Exit::Code(0), None, 3601, "qa")
            })
            .expect("landed");
        run.tick();
        assert_eq!(run.processes.bounded.len(), 3, "a landing outranks state polling");
    }
}

#[test]
fn oldest_unfiled_requests_run_first_and_success_survives_a_later_failure() {
    let mut world = World::new();
    seed_request(&world, 0, 20, "work", &id(0));
    seed_request(&world, 0, 10, "home", &id(1));
    world.home.configuration.request.command = deployment(
        &world,
        "file",
        r#"
printf '%s\n' "$PWD" >> "$TEMPER_INTERNAL_QA_HOME/order"
case "$PWD" in */passes/1/requests/1) exit 1;; esac
"#,
    );
    let mut run = world.start(baseline(), &[]);
    run.tick();
    let order = fs::read_to_string(world.home.directory.join("order")).expect("order");
    assert_eq!(
        order,
        format!("{}\n{}\n", request_directory(&world, 2, 1).display(), request_directory(&world, 1, 1).display())
    );
    assert_eq!(
        world.ledger().passes[1].requests[0].filed,
        Some(0),
        "earlier success is saved even when the step fails"
    );
    assert_eq!(world.ledger().passes[0].requests[0].filed, None);
    deployment(&world, "file", "exit 0");
    run.tick();
    assert_eq!(run.processes.bounded.len(), 3, "only the failed request is retried");
    assert_eq!(world.ledger().passes[0].requests[0].filed, Some(0));
}

fn request_answers(run: &mut super::world::Loop, answers: &[(Exit, Option<&str>)]) {
    run.processes.answers.extend(answers.iter().map(|(exit, stdout)| temper_qa_tool::faces::CommandAnswer {
        exit: *exit,
        last_stdout: stdout.map(str::to_owned),
    }));
}

fn seed_open_gap(world: &World) {
    world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            let landing = u64::try_from(ledger.landings.len()).expect("landing count") + 1;
            super::landing(ledger, landing);
            let session = super::session(ledger, landing);
            ledger.triage_gap(session, "ctl", "no runner tool", "tools", 0)?;
            ledger.end_session(session, Exit::Code(0), None, 0, "qa")
        })
        .expect("open gap");
}

fn request_git(world: &World) -> Git {
    let mut git = baseline();
    for landing in world.ledger().landings {
        git.commit(landing.number, &[landing.number - 1], "landing");
    }
    git
}

fn unfiled_requests() -> World {
    let world = World::new();
    seed_request(&world, 1, 20, "home", &id(0));
    seed_request(&world, 2, 10, "work", &id(1));
    world
}

#[test]
fn an_older_request_failure_leaves_a_later_success_filed() {
    let world = unfiled_requests();
    let mut run = world.start(request_git(&world), &[]);
    request_answers(&mut run, &[(Exit::Code(1), None), (Exit::Code(0), Some("goal 42"))]);
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes[0].requests[0].filed, Some(0));
    assert_eq!(ledger.passes[0].requests[0].goal, Some(42));
    assert_eq!(ledger.passes[1].requests[0].filed, None);
    assert_eq!(
        run.processes.bounded.iter().map(|(command, _)| command.directory.clone()).collect::<Vec<_>>(),
        [request_directory(&world, 2, 1), request_directory(&world, 1, 1)]
    );
    assert!(request_directory(&world, 1, 1).join("tree/temper/docs/development/qa/requests/pass-1-1.md").is_file());
    run.clock.time = 3600;
    request_answers(&mut run, &[(Exit::Code(1), None)]);
    run.tick();
    assert_eq!(
        world.ledger().lines[0].words,
        "requests has failed for an hour: pass 2, request 1: command exited Code(1)"
    );
    request_answers(&mut run, &[(Exit::Code(0), Some("goal 41"))]);
    run.tick();
    assert_eq!(world.ledger().passes[1].requests[0].goal, Some(41));
    assert_eq!(run.processes.bounded.len(), 4, "only the failed request is retried");
}

#[test]
fn a_gap_is_filed_while_a_findings_request_fails() {
    let world = World::new();
    seed_request(&world, 1, 5, "work", &id(0));
    seed_open_gap(&world);
    let mut run = world.start(request_git(&world), &[]);
    request_answers(&mut run, &[(Exit::Code(1), None), (Exit::Code(0), None)]);
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes[0].requests[0].filed, None);
    assert_eq!(ledger.gaps[0].state, temper_qa_tool::GapState::Filed);
    assert_eq!(ledger.gaps[0].filed, Some(0));
    let directory = world.home.directory.join("gaps/requests/1");
    assert!(directory.join("tree/temper/docs/development/qa/gaps/gap-1.md").is_file());
    assert!(!directory.join("goal").exists());
    assert_eq!(run.processes.bounded[1].0.directory, directory);
}

#[test]
fn two_failed_requests_are_named_in_oldest_first_order() {
    let world = unfiled_requests();
    let mut run = world.start(request_git(&world), &[]);
    for time in [0, 3600] {
        run.clock.time = time;
        request_answers(&mut run, &[(Exit::Code(1), None), (Exit::Code(2), None)]);
        run.tick();
    }
    let ledger = world.ledger();
    assert_eq!(ledger.passes[0].requests[0].filed, None);
    assert_eq!(ledger.passes[1].requests[0].filed, None);
    assert_eq!(
        ledger.lines[0].words,
        "requests has failed for an hour: pass 2, request 1: command exited Code(1); pass 1, request 1: command exited Code(2)"
    );
}

#[test]
fn every_request_failure_is_reported_while_waiting_goals_are_checked() {
    let mut world = World::new();
    seed_request(&world, 0, 5, "home", &id(0));
    seed_request(&world, 1, 10, "work", &id(1));
    seed_request(&world, 2, 20, "home", &id(2));
    world
        .home
        .transact(&mut RealFiles, |ledger, _| {
            ledger.file_request(2, 1, Some(41), 0)?;
            ledger.file_request(3, 1, Some(42), 0)
        })
        .expect("waiting goals");
    seed_open_gap(&world);
    world.home.configuration.request.state = Some(vec!["state".into()]);
    let mut run = world.start(request_git(&world), &[]);
    request_answers(
        &mut run,
        &[(Exit::Code(1), None), (Exit::Code(2), None), (Exit::Code(3), None), (Exit::Code(0), Some("ended"))],
    );
    run.tick();
    assert_eq!(world.ledger().passes[2].requests[0].ended, Some(0));
    assert_eq!(run.processes.bounded[2].0.arguments, ["state", "41"]);
    assert_eq!(run.processes.bounded[3].0.arguments, ["state", "42"]);
    run.clock.time = 3600;
    request_answers(&mut run, &[(Exit::Code(1), None), (Exit::Code(2), None), (Exit::Code(3), None)]);
    run.tick();
    assert_eq!(
        world.ledger().lines[0].words,
        "requests has failed for an hour: pass 1, request 1: command exited Code(1); gaps 1: command exited Code(2); goal 41: state command exited Code(3)"
    );
    assert_eq!(world.ledger().passes[1].requests[0].ended, None);
    assert_eq!(world.ledger().gaps[0].state, temper_qa_tool::GapState::Open);
    assert_eq!(run.processes.bounded.len(), 7, "each failure retries; the ended goal is not checked again");
}

#[test]
fn bounded_requests_are_not_filed_and_real_commands_are_killed_at_the_bound() {
    use temper_qa_tool::faces::{ProcessCommand, Processes, RealProcesses};
    let mut world = World::new();
    seed_request(&world, 1, 5, "work", &id(0));
    world.home.configuration.request.command = deployment(&world, "file", "printf 'goal 41\\n'");
    let mut run = world.start(baseline(), &[]);
    run.processes.command_bound = true;
    run.tick();
    assert_eq!(world.ledger().passes[0].requests[0].filed, None);
    run.processes.command_bound = false;
    run.tick();
    assert_eq!(world.ledger().passes[0].requests[0].goal, Some(41));
    let command = ProcessCommand {
        arguments: vec!["/bin/sh".into(), "-c".into(), "while :; do :; done".into()],
        directory: world.home.directory.clone(),
        stdin: "/dev/null".into(),
        output: world.home.directory.join("bounded-output"),
        environment: vec![],
    };
    assert_eq!(RealProcesses::default().run(&command, 0).expect("bounded command").exit, Exit::Bound);
    assert!(!command.output.with_extension("stdout").exists(), "temporary stdout is removed");
}

#[test]
fn triage_lists_ctl_groups_from_the_merge_tree_before_web_and_tui() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    for name in ["ctl-task.md", "ctl-project.md", "ctl.md", "help.md", "ctl-no.txt", "ctl-directory/README.md"] {
        run.git.file_at(2, &format!("temper/docs/help/reference/{name}"), b"# Reference");
    }
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[0].pages["ctl"], ["project", "task"]);
    let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
    let surfaces = brief.split("## Surfaces").nth(1).expect("surfaces").split("## Setups").next().expect("section");
    assert!(surfaces.find("- ctl: project:").expect("ctl first") < surfaces.find("- web:").expect("web"));
    assert!(surfaces.find("- web:").expect("web") < surfaces.find("- tui:").expect("tui"));
    for words in [
        "- ctl: task: temper/docs/help/reference/ctl-task.md",
        "temper/docs/design/client/ctl/README.md",
        "temper/docs/design/client/ctl/tools.md",
        "through ctl, the web client or the TUI, ctl first",
    ] {
        assert!(brief.contains(words), "{words}\n{brief}");
    }
    assert!(!brief.contains("ctl and help are outside"));
}

fn triage_catalogue(ctl_only: bool) -> (World, super::world::Loop) {
    let mut world = World::new();
    if ctl_only {
        world.home.configuration.surfaces = vec!["ctl".into()];
    }
    let mut run = world.start(baseline(), &[Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.git.file_at(2, "temper/docs/help/reference/ctl-task.md", b"# Task\n");
    for (name, recipe) in [
        ("ctl-root", "recipe 1\ncheck\nctl task list\n"),
        ("ctl-chain", "recipe 1\nafter \"ctl-root\"\ncheck\nctl task list\n"),
        ("web-root", "recipe 1\nweb see \"Welcome\"\n"),
        ("web-chain", "recipe 1\nafter \"web-root\"\ncheck\nctl task list\n"),
        ("tui-root", "recipe 1\ntui see \"Welcome\"\n"),
        ("tui-chain", "recipe 1\nafter tui-root\ncheck\nctl task list\n"),
        ("missing-parent", "recipe 1\nafter absent\ncheck\nctl task list\n"),
        ("cycle", "recipe 1\nafter cycle\ncheck\nctl task list\n"),
    ] {
        let directory = format!("temper/docs/development/qa/setups/{name}");
        run.git.file_at(2, &format!("{directory}/recipe"), recipe.as_bytes());
        run.git.file_at(2, &format!("{directory}/README.md"), format!("# Setup\n\nThe {name} state.\n").as_bytes());
    }
    run.tick();
    (world, run)
}

#[test]
fn triage_brief_says_the_happy_path_and_reads_help() {
    let (_world, run) = triage_catalogue(true);
    let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
    assert!(brief.contains(super::HAPPY_PATH));
    assert!(brief.contains("Write no case, step or gap for one, and answer no QA for a landing whose only change a person meets is one, saying so."));
    assert!(brief.contains("A security hole or lost data still counts wherever it is."));
    assert!(brief.contains("For ctl steps, write do as ctl or forge drive lines, or `temper help PAGE`. In its pass, a `temper help PAGE` step is read with the place's own program, with no box; compare its words as a `see` is."));
}

#[test]
fn triage_brief_offers_web_requests_only_with_web_configured() {
    let words = "For a web page whose draft promises a request budget, a step's do may be `web requests`, after a step that opens the page cold, such as a reload of its path; its see is that line's last line as the budget counts it, `N requests in W waves`: `2 requests in 2 waves` for two waves of two requests.";
    for ctl_only in [false, true] {
        let (_world, run) = triage_catalogue(ctl_only);
        let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
        if ctl_only {
            assert!(!brief.contains(words), "ctl-only triage has no web request guidance");
        } else {
            let commands = brief.split("## Commands\n\n").nth(1).expect("commands");
            assert!(commands.contains(&format!("{words} Use only the pages and setups above;")), "{commands}");
        }
    }
}

#[test]
fn ctl_only_triage_lists_its_surfaces_and_walkable_chains() {
    let (world, run) = triage_catalogue(true);
    let ledger = world.ledger();
    let session = &ledger.sessions[0];
    assert_eq!(session.pages, [("ctl".into(), vec!["task".into()])].into());
    assert_eq!(session.setups, ["fresh", "ctl-chain", "ctl-root"]);
    let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
    assert!(brief.starts_with("# Triage\n\nJudge what a person now sees or does through ctl, the person's acts and temper's account as `temper ctl` gives them. QA judges the happy path alone:"));
    let surfaces =
        brief.split("## Surfaces\n\n").nth(1).expect("surfaces").split("\n## Setups").next().expect("section");
    assert_eq!(
        surfaces,
        "- ctl: task: temper/docs/help/reference/ctl-task.md\n- ctl design: temper/docs/design/client/ctl/README.md; temper/docs/design/client/ctl/tools.md\n"
    );
    let setups =
        brief.split("## Setups\n\n").nth(1).expect("setups").split("\n## Queued cases").next().expect("section");
    assert_eq!(
        setups,
        "- fresh (box): a fresh box at its welcome with nothing set up\n- ctl-chain (box): The ctl-chain state. (temper/docs/development/qa/setups/ctl-chain/README.md)\n- ctl-root (box): The ctl-root state. (temper/docs/development/qa/setups/ctl-root/README.md)\n"
    );
    assert!(brief.contains("\"surface\":\"ctl\""));
    assert!(brief.contains(r#"Quoted bytes take printable ASCII alone; write other bytes as `\xHH`, with `\"` and `\\` for quote and backslash."#));
    assert!(brief.contains("A case's surface is ctl; for ctl, page is one of its groups."));
}

#[test]
fn default_triage_keeps_all_surfaces_and_setup_cases() {
    let (world, run) = triage_catalogue(false);
    let mut ledger = world.ledger();
    let number = ledger.sessions[0].number;
    assert_eq!(
        ledger.sessions[0].setups,
        ["fresh", "ctl-chain", "ctl-root", "cycle", "missing-parent", "tui-chain", "tui-root", "web-chain", "web-root"]
    );
    let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
    assert!(brief.starts_with("# Triage\n\nJudge what a person now sees or does through ctl, the web client or the TUI, ctl first: write ctl cases before web and TUI cases, the person's acts and temper's account as `temper ctl` gives them. QA judges the happy path alone:"));
    assert!(brief.contains("\"surface\":\"web\""));
    assert!(brief.contains("A case's surface is ctl, web or tui; for ctl, page is one of its groups."));
    for (surface, page, setup) in
        [("ctl", "task", "ctl-chain"), ("web", "work", "web-chain"), ("tui", "chat", "tui-chain")]
    {
        let mut input = super::case();
        input.surface = surface.into();
        input.page = page.into();
        input.setup = setup.into();
        ledger.add_case(number, &input, 1).expect("default accepts each surface and setup");
    }
    assert_eq!(ledger.cases.iter().map(|case| case.surface.as_str()).collect::<Vec<_>>(), ["ctl", "web", "tui"]);
}

#[test]
fn triage_names_only_each_configured_surface_set() {
    for (configured, words) in [
        (vec!["web"], "the web client"),
        (vec!["tui"], "the TUI"),
        (
            vec!["web", "ctl"],
            "ctl or the web client, ctl first: write ctl cases before web cases, the person's acts and temper's account as `temper ctl` gives them",
        ),
        (
            vec!["tui", "ctl"],
            "ctl or the TUI, ctl first: write ctl cases before TUI cases, the person's acts and temper's account as `temper ctl` gives them",
        ),
        (vec!["tui", "web"], "the web client or the TUI"),
        (vec![], "the configured surfaces (none)"),
    ] {
        let mut world = World::new();
        world.home.configuration.surfaces = configured.iter().map(|name| (*name).to_owned()).collect();
        let mut run = world.start(baseline(), &[Agent::Never]);
        run.tick();
        land(&mut run, 2, 7);
        run.tick();
        let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
        assert!(brief.starts_with(&format!(
            "# Triage\n\nJudge what a person now sees or does through {words}. QA judges the happy path alone:"
        )));
        let ledger = world.ledger();
        let surfaces =
            brief.split("## Surfaces\n\n").nth(1).expect("surfaces").split("\n## Setups").next().expect("section");
        for name in ["ctl", "web", "tui"] {
            assert_eq!(
                ledger.sessions[0].pages.contains_key(name),
                configured.contains(&name),
                "{configured:?}: {name}"
            );
            assert_eq!(
                surfaces.contains(&format!("- {name}")),
                configured.contains(&name),
                "{configured:?}: {surfaces}"
            );
        }
    }
}

#[test]
fn ctl_triage_takes_eight_parents_and_leaves_out_nine() {
    // The recipe and its eight parents qualify; a ninth parent exceeds recipe.md, 3.2.
    const QUALIFYING_CHAIN: usize = 9;
    let mut world = World::new();
    world.home.configuration.surfaces = vec!["ctl".into()];
    let mut run = world.start(baseline(), &[Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    for index in 0..=QUALIFYING_CHAIN {
        let parent = index.checked_sub(1).map_or(String::new(), |parent| format!("after chain-{parent}\n"));
        let recipe = format!("recipe 1\n{parent}check\nctl task list\n");
        run.git.file_at(2, &format!("temper/docs/development/qa/setups/chain-{index}/recipe"), recipe.as_bytes());
    }
    run.tick();
    assert_eq!(
        world.ledger().sessions[0].setups,
        ["fresh", "chain-0", "chain-1", "chain-2", "chain-3", "chain-4", "chain-5", "chain-6", "chain-7", "chain-8"]
    );
}

fn a_running_triage() -> (World, super::world::Loop) {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::None, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.tick();
    (world, run)
}

#[test]
fn a_second_triage_moves_and_cleans_the_kept_tree_instead_of_making_one() {
    let (world, mut run) = a_running_triage();
    let tree = world.home.triage_tree();
    fs::write(tree.join("leftover"), "agent's untracked file").expect("leftover");
    land(&mut run, 4, 8);
    run.tick();
    assert_eq!(run.git.calls.iter().filter(|call| call.starts_with("add ")).count(), 1);
    assert!(run.git.calls.contains(&format!("checkout {}", id(4))));
    assert!(run.git.calls.contains(&"clean -ffdxq".into()));
    assert!(!run.git.calls.iter().any(|call| call.starts_with("remove ")));
    assert!(!tree.join("leftover").exists());
    assert_eq!(run.processes.commands[0].directory, tree);
    assert_eq!(run.processes.commands[1].directory, tree);
    let brief = fs::read_to_string(&run.processes.commands[1].stdin).expect("brief");
    assert!(brief.contains(&format!("Tree: {}", tree.display())));
}

#[test]
fn a_failed_triage_tree_move_removes_and_makes_it_again() {
    let (world, mut run) = a_running_triage();
    fs::write(world.home.triage_tree().join("leftover"), "old").expect("leftover");
    run.git.move_error = Some("checkout refused".into());
    land(&mut run, 4, 8);
    run.tick();
    let calls = &run.git.calls;
    let moved = calls.iter().position(|call| call == &format!("checkout {}", id(4))).expect("move tried");
    let removed = calls.iter().position(|call| call.starts_with("remove ")).expect("removed");
    let added = calls.iter().position(|call| call == &format!("add {}", id(4))).expect("made again");
    assert!(moved < removed && removed < added);
    assert_eq!(run.git.prunes, 1);
    assert!(!world.home.triage_tree().join("leftover").exists());
    assert_eq!(world.ledger().sessions[1].exit, None);
}

#[test]
fn triage_offers_only_the_configured_places_kinds() {
    use temper_qa_tool::Kind;
    for kinds in [vec![Kind::Box], vec![Kind::Deployment], vec![Kind::Box, Kind::Deployment]] {
        let mut world = World::new();
        world.home.configuration.places.clear();
        world.home.configuration.kinds.clear();
        for (index, kind) in kinds.iter().enumerate() {
            let place = format!("places/{index}");
            world.home.configuration.places.push(place.clone());
            world.home.configuration.kinds.insert(place, *kind);
        }
        let mut run = world.start(baseline(), &[Agent::Never]);
        run.tick();
        land(&mut run, 2, 7);
        let setups = "temper/docs/development/qa/setups";
        let file = temper_config_shell::FILE;
        for name in ["ctl-hello", "turn-landing"] {
            run.git.file_at(2, &format!("{setups}/{name}/recipe"), b"recipe 1\ncheck\nctl task list\n");
        }
        run.git.file_at(2, &format!("{setups}/turn-landing/config/engine/{file}"), &super::engine_file());
        run.tick();
        let mut ledger = world.ledger();
        let session = &ledger.sessions[0];
        let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
        for (name, kind) in [("fresh", Kind::Box), ("ctl-hello", Kind::Box), ("turn-landing", Kind::Deployment)] {
            let offered = kinds.contains(&kind);
            assert_eq!(session.setups.contains(&name.into()), offered, "{name}: {kinds:?}");
            assert_eq!(brief.contains(&format!("- {name} ({}):", kind.name())), offered, "{brief}");
        }
        assert_eq!(session.deployments, if kinds.contains(&Kind::Deployment) { vec!["turn-landing"] } else { vec![] });
        let number = session.number;
        let mut input = super::case();
        input.setup = if kinds == [Kind::Deployment] { "ctl-hello" } else { "turn-landing" }.into();
        if kinds.len() == 1 {
            let why = ledger.add_case(number, &input, 1).expect_err("setup not offered").to_string();
            assert_eq!(
                why,
                format!(
                    "case refused: setup {} is not in the catalogue; setups: {}",
                    input.setup,
                    ledger.sessions[0].setups.join(", ")
                )
            );
        } else {
            ledger.add_case(number, &input, 1).expect("both kinds offered");
            assert_eq!(ledger.cases[0].kind, Kind::Deployment);
        }
    }
}

#[test]
fn triage_catalogue_follows_roots_at_merge_and_records_deployments_in_every_configuration() {
    for surfaces in [vec!["ctl"], vec!["ctl", "web", "tui"], vec!["web"]] {
        let mut world = World::new();
        world.home.configuration.surfaces = surfaces.iter().map(|surface| (*surface).into()).collect();
        world.home.configuration.places.push("places/2".into());
        world.home.configuration.kinds.insert("places/2".into(), temper_qa_tool::Kind::Deployment);
        let mut run = world.start(baseline(), &[Agent::Never]);
        run.tick();
        land(&mut run, 2, 7);
        let setups = "temper/docs/development/qa/setups";
        let file = temper_config_shell::FILE;
        for (name, recipe) in [
            ("root", "recipe 1\ncheck\nctl task list\n"),
            ("turn-landing", "recipe 1\nafter root\ncheck\nctl task list\n"),
            ("welcome", "recipe 1\ncheck\nctl task list\n"),
            ("box-child", "recipe 1\nafter welcome\ncheck\nctl task list\n"),
            ("cycle", "recipe 1\nafter cycle\ncheck\n"),
            ("missing", "recipe 1\nafter absent\ncheck\n"),
        ] {
            run.git.file_at(2, &format!("{setups}/{name}/recipe"), recipe.as_bytes());
        }
        run.git.file_at(2, &format!("{setups}/root/config/engine/{file}"), &super::engine_file());
        run.git.file_at(2, &format!("{setups}/box-child/config/engine/{file}"), &super::engine_file());
        for depth in 1..=9 {
            let parent = if depth == 1 { "root".into() } else { format!("depth-{}", depth - 1) };
            run.git.file_at(
                2,
                &format!("{setups}/depth-{depth}/recipe"),
                format!("recipe 1\nafter {parent}\ncheck\n").as_bytes(),
            );
        }
        run.tick();
        let ledger = world.ledger();
        let session = &ledger.sessions[0];
        assert!(session.deployments.contains(&"turn-landing".into()));
        assert!(session.deployments.contains(&"depth-8".into()), "eight parents are allowed");
        assert!(!session.deployments.contains(&"depth-9".into()), "a ninth parent is not followed");
        assert!(!session.deployments.contains(&"box-child".into()), "only the root's config decides kind");
        let brief = fs::read_to_string(&run.processes.commands[0].stdin).expect("brief");
        for line in ["- fresh (box):", "- turn-landing (deployment):", "- box-child (box):", "- depth-8 (deployment):"]
        {
            assert!(brief.contains(line), "{line}\n{brief}");
        }
        if surfaces == ["ctl"] {
            assert!(!session.setups.contains(&"depth-9".into()));
            assert!(!session.setups.contains(&"cycle".into()));
            assert!(!session.setups.contains(&"missing".into()));
        } else {
            for name in ["depth-9", "cycle", "missing"] {
                assert!(brief.contains(&format!("- {name} (box):")), "unfollowable chains default to box");
            }
        }
        assert_eq!(Ledger::decode(&ledger.encode().expect("encode")).expect("decode"), ledger);
    }
}
