//! The watcher and tick loop over the fake world, never real git or agents.
use super::world::{Agent, Agents, Clock, Git, World, baseline, id, land};
use std::fs;
use temper_qa_tool::{
    Answer, BuildState, Exit, Ledger,
    faces::{Commit, Signal, parse_commits, trailers},
    runner::{Run, parse_tokens},
    watch::{parse_landing, watch},
};

fn read(git: &mut Git, world: &World) {
    world.home.transact(|ledger| watch(git, &world.home.configuration.git, ledger, 1)).expect("watch");
}

#[test]
fn git_text_parses_parents_final_trailers_and_subjects_from_the_end() {
    let text = format!("{}\n{} {}\n{} {} {}\n", id(0), id(1), id(0), id(2), id(0), id(1));
    assert_eq!(
        parse_commits(&text).expect("rev-list"),
        vec![
            Commit { name: id(0), parents: vec![] },
            Commit { name: id(1), parents: vec![id(0)] },
            Commit { name: id(2), parents: vec![id(0), id(1)] },
        ]
    );
    assert!(parse_commits("not-a-commit\n").is_err());
    assert!(parse_commits("").expect("empty").is_empty());
    let subject = "Merge pull request 'Title (x)' (#12) from temper5/g7 into main";
    let landing = parse_landing(subject, "main").expect("goal subject");
    assert_eq!((landing.goal, landing.pull, landing.title.as_str()), (7, Some(12), "Title (x)"));
    let hostile_title =
        "Merge pull request 'Title ' (#9) from someone/g2 into main (x)' (#12) from temper5/g7 into main";
    assert_eq!(
        parse_landing(hostile_title, "main").expect("last suffix").title,
        "Title ' (#9) from someone/g2 into main (x)"
    );
    for message in [
        "Merge pull request 'Change' (#12) from temper5/c9 into temper5/g7",
        "Merge branch 'main' into temper5/g7",
        "Merge pull request 'Change' (#12) from temper5/g7 into temper5/g8",
        "Ordinary subject\n\nGoal: 7\n\nNot a trailer",
        "Ordinary subject\n\nGoal: 7\nnot a trailer",
    ] {
        assert!(parse_landing(message, "main").is_none(), "{message}");
    }
    let message = format!("{subject}\n\nGoal: 8\nPull-request: 19\nQA: work\n");
    let landing = parse_landing(&message, "main").expect("trailers take precedence");
    assert_eq!((landing.goal, landing.pull), (8, Some(19)));
    assert_eq!(trailers(&message).last(), Some(&("QA".into(), "work".into())));
    assert_eq!(parse_landing("A merge\n \nGoal: 7\nPull-request: 12\n", "main").expect("blank separator").goal, 7);
    assert!(parse_landing("A merge\n\nGoal: +7\n", "main").is_none());
    assert_eq!(parse_tokens("tokens 2\nbody\ntokens 1234\n\n"), Some(1234));
    for text in ["tokens +1", "tokens -1", "tokens 1\nlast words", "", "tokens 1.0"] {
        assert_eq!(parse_tokens(text), None);
    }
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
    assert!(world.ledger().landings.is_empty());
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
    let reopened = temper_qa_tool::Home::open(world.scratch.path()).expect("restart");
    git.head = id(4);
    reopened.transact(|ledger| watch(&mut git, &reopened.configuration.git, ledger, 2)).expect("restart watch");
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
    world.home.transact(|ledger| watch(&mut git, &config, ledger, 3)).expect("fetch watch");
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

#[test]
fn triage_is_serial_and_its_brief_and_environment_are_the_merges_inputs() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::None]);
    run.tick();
    land(&mut run, 2, 7);
    land(&mut run, 4, 8);
    run.git.nodes.get_mut(&id(2)).expect("merge").message =
        "Goal 7's title\n\nDraft: `temper/docs/design/draft/x.execution.md`\n\nGoal: 7\nPull-request: 12\n".into();
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
    run.git.nodes.get_mut(&id(2)).expect("merge").paths.push("temper/docs/design/draft/other.status.md".into());
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.sessions.len(), 1);
    assert_eq!(ledger.sessions[0].landing, 1);
    assert_eq!(ledger.sessions[0].effort, "cheap");
    assert_eq!(ledger.sessions[0].pages["web"], ["chat", "work"]);
    assert_eq!(ledger.sessions[0].pages["tui"], ["chat"]);
    assert_eq!(ledger.sessions[0].setups, ["fresh", "project"]);
    let command = &run.processes.commands[0];
    assert_eq!(command.arguments, ["agent", "--effort=cheap"]);
    assert_eq!(command.directory, world.home.session_directory(1).join("tree"));
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
        "1 insertion(+)",
        "sessions/1/diff.patch",
        "QA: Work lists drafts",
        "web: chat",
        "tui: chat",
        "fresh",
        "A project ready for work. Two lines.",
        "temper-qa triage none WHY",
        "temper-qa triage case FILE",
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
    assert_eq!(ledger.sessions[1].landing, 2);
    assert!(!world.home.session_directory(1).join("tree").exists());
    let brief = fs::read_to_string(world.home.session_directory(2).join("brief")).expect("second brief");
    for words in ["Case 1: web / work; landings 1", "setup fresh", "open Work", "Drafts lists the draft"] {
        assert!(brief.contains(words), "{words}");
    }
    run.tick();
    assert!(world.ledger().running_sessions().is_empty());
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
    assert!(run.processes.signals.is_empty());
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
        assert!(!world.home.session_directory(1).join("tree").exists());
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
    assert!(world.ledger().lines.is_empty());
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
    assert!(world.ledger().lines.is_empty());
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
fn cycles_wait_for_poll_but_wake_on_agent_exit_or_bound() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::None]);
    run.cycle();
    assert_eq!(run.clock.time, 60, "an idle cycle waits for poll");
    land(&mut run, 2, 7);
    run.cycle();
    assert_eq!(run.clock.time, 61, "the first one-second probe sees the exit");
    assert_eq!(world.ledger().sessions[0].exit, None, "the next tick records the end");
    assert!(world.ledger().landings[0].answer.is_some(), "commands can record between ticks");
    run.cycle();
    assert_eq!(world.ledger().sessions[0].ended, Some(61));
    assert_eq!(run.clock.time, 121);
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.cycle();
    assert_eq!(run.clock.time, 30, "the bound also wakes the next tick");
    assert_eq!(run.processes.signals, [(1, Signal::Term)]);
    run.home.configuration.bounds.poll = 0;
    run.cycle();
    assert_eq!(run.clock.time, 30, "zero poll has no wait");
}

#[test]
fn one_run_holds_the_home_and_restart_ends_every_running_session_lost_and_prunes() {
    let world = World::new();
    world
        .home
        .transact(|ledger| {
            ledger.record_landing(&id(2), 7, None, "landed", 0);
            ledger.start_session(1, "cheap", super::pages(), vec!["fresh".into()], 0)?;
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
    assert!(ledger.running_sessions().is_empty());
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
