//! The loop's pass step over commits, recorded agent actions and movable time (qa/passes.md, 9).
use super::world::{Agent, Loop, World, baseline, id, land};
use std::{fs, fs::File, os::unix::fs::PermissionsExt, path::Path};
use temper_qa_tool::{CaseState, Exit, PassState, faces::Signal};

const SETUPS: &str = "temper/docs/development/qa/setups";

fn recipe(run: &mut Loop, commit: u64, name: &str, readme: &str) {
    for (file, bytes) in [
        ("recipe", b"recipe 1\nweb see Ready\n".as_slice()),
        ("README.md", readme.as_bytes()),
        ("scripts.json", b"{}".as_slice()),
        ("assets/note", b"asset".as_slice()),
    ] {
        run.git.file_at(commit, &format!("{SETUPS}/{name}/{file}"), bytes);
    }
}

fn ready(world: &World, agent: Agent) -> Loop {
    let mut run = world.start(baseline(), &[Agent::Case, agent]);
    run.tick();
    land(&mut run, 2, 7);
    recipe(&mut run, 2, "project", "# Project\n\nA ready project. More detail.\n");
    world.installable_build(2);
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 1, "the pass started");
    run
}

fn hold(place: &Path) -> File {
    fs::create_dir_all(place).expect("place");
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(place.join("drive.lock"))
        .expect("lock file");
    file.try_lock().expect("holder takes lock");
    file
}

#[test]
fn start_installs_the_newest_reached_build_and_writes_an_ordered_brief() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.git.nodes.get_mut(&id(2)).expect("merge").message =
        "Goal 7's title\n\nDraft: temper/docs/design/draft/promises.execution.md\n\nGoal: 7\nPull-request: 12\n".into();
    run.git.file_at(2, "temper/docs/design/draft/promises.md", b"# Promises");
    run.git.commit(3, &[2], "later build");
    recipe(&mut run, 3, "project", "# Project\n\nA ready project. More detail.\n");
    world.installable_build(2);
    let build = world.installable_build(3);
    let place = world.home.directory.join("places/1");
    fs::create_dir_all(place.join("bin")).expect("old bin");
    fs::create_dir_all(place.join("bundle")).expect("old bundle");
    fs::write(place.join("bin/stale"), b"stale").expect("old executable");
    fs::write(place.join("bundle/stale"), b"stale").expect("old asset");
    fs::write(place.join("setup"), b"project oldhash").expect("old setup");
    run.tick();
    world
        .home
        .transact(|ledger| {
            let mut tui = super::case();
            tui.surface = "tui".into();
            tui.page = "chat".into();
            ledger.add_case(1, &tui, 0)?;
            let mut web = super::case();
            web.adds = "a ready draft".into();
            web.steps[0].see = Some("Drafts".into());
            ledger.add_case(1, &web, 0)?;
            Ok(())
        })
        .expect("cases recorded by triage");
    run.tick();
    let ledger = world.ledger();
    let pass = &ledger.passes[0];
    assert_eq!((pass.build.as_str(), pass.place.as_str()), (id(3).as_str(), "places/1"));
    assert_eq!(pass.cases, [2]);
    assert_eq!(ledger.cases[0].state, CaseState::Queued, "tui stays queued");
    assert_eq!(ledger.cases[1].state, CaseState::Assigned);
    assert_eq!(ledger.sessions[1].effort, "high");
    for path in
        ["bin/temper-box", "bin/temper", "bin/temper-client-journeys", "bundle/index.html", "bundle/assets/app.js"]
    {
        assert_eq!(fs::read(place.join(path)).expect("installed"), fs::read(build.join(path)).expect("build"));
    }
    assert_eq!(fs::metadata(place.join("bin/temper")).expect("mode").permissions().mode() & 0o777, 0o751);
    assert!(!place.join("bin/stale").exists());
    assert!(!place.join("bundle/stale").exists());
    assert!(!place.join("setup").exists());
    assert_eq!(
        fs::read(world.home.directory.join("passes/1/setups/project/assets/note")).expect("recursive setups"),
        b"asset"
    );
    let command = &run.processes.commands[1];
    assert_eq!(command.arguments, ["agent", "--effort=high"]);
    assert_eq!(command.directory, world.home.session_directory(2).join("tree"));
    assert_eq!(command.output, world.home.session_directory(2).join("output"));
    for (name, value) in [
        (temper_qa_tool::HOME.name, world.home.directory.display().to_string()),
        (temper_qa_tool::SESSION.name, "2".into()),
        ("CAPTURED", "kept".into()),
    ] {
        assert!(
            command
                .environment
                .iter()
                .any(|(key, bytes)| key.as_ref() == name.as_bytes() && bytes.as_ref() == value.as_bytes())
        );
    }
    let brief = fs::read_to_string(&command.stdin).expect("stdin brief");
    let mut previous = 0;
    for heading in ["# What a pass is", "## Pass", "## Landings", "## Cases", "## Setups", "## Commands"] {
        let at = brief.find(heading).expect("heading");
        assert!(at >= previous, "{heading} is out of order");
        previous = at;
    }
    assert!(brief.contains("Landing 1; goal 7; merge"));
    assert!(brief.contains("temper/docs/design/draft/promises.md"));
    assert!(brief.contains("Case 2: web / work; landings 1"));
    assert!(
        brief.contains("adds: a ready draft\n  - do: open Work\n    expect: Drafts lists the draft\n    see: Drafts")
    );
    assert!(brief.contains("project: A ready project.\n"));
    assert!(brief.contains("`pass setup`, `pass recipe` and `pass open` may take minutes"));
    assert!(brief.contains("shell's own timeout does not cut them"));
}

#[test]
fn a_busy_drive_lock_keeps_the_place_free_of_new_installs_or_records() {
    let world = World::new();
    let place = world.home.directory.join("places/1");
    let lock = hold(&place);
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    world.installable_build(2);
    run.tick();
    run.tick();
    assert!(world.ledger().passes.is_empty());
    assert_eq!(run.processes.commands.len(), 1);
    assert!(!place.join("bin").exists());
    assert!(!world.home.directory.join("passes").exists());
    drop(lock);
    run.tick();
    assert_eq!(world.ledger().passes[0].place, "places/1");
    assert_eq!(run.processes.commands.len(), 2);
    run.tick();
    assert_eq!(run.processes.commands.len(), 2, "a recorded running pass keeps the place busy too");
}

#[test]
fn a_bad_manifest_names_the_path_records_nothing_and_retries_next_tick() {
    for missing in [false, true] {
        let world = World::new();
        let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
        run.tick();
        land(&mut run, 2, 7);
        let build = world.installable_build(2);
        let manifest = fs::read_to_string(build.join("manifest")).expect("manifest");
        let why = if missing {
            let shortened = manifest
                .lines()
                .filter(|line| !line.ends_with("  bundle/assets/app.js"))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(build.join("manifest"), shortened).expect("missing manifest entry");
            "file is absent from manifest"
        } else {
            fs::write(build.join("bundle/assets/app.js"), b"changed").expect("corrupted build");
            "checksum differs from manifest"
        };
        run.tick();
        run.tick();
        assert!(world.ledger().passes.is_empty());
        assert_eq!(world.ledger().sessions.len(), 1);
        assert_eq!(world.ledger().cases[0].state, CaseState::Queued);
        run.clock.time = 3600;
        run.tick();
        let ledger = world.ledger();
        assert!(ledger.passes.is_empty());
        assert!(
            ledger.lines.iter().any(|line| line
                .words
                .contains(&format!("passes has failed for an hour: bundle/assets/app.js: {why}")))
        );
        fs::write(build.join("manifest"), manifest).expect("repaired manifest");
        fs::write(build.join("bundle/assets/app.js"), b"javascript").expect("repaired build");
        run.tick();
        assert_eq!(world.ledger().passes.len(), 1);
        assert_eq!(
            run.processes
                .commands
                .iter()
                .filter(|command| command.arguments.first().is_some_and(|program| program == "agent"))
                .count(),
            2
        );
    }
}

#[test]
fn installation_reads_sha256sum_escaped_byte_paths() {
    use sha2::{Digest, Sha256};
    use std::{ffi::OsString, io::Write, os::unix::ffi::OsStringExt, path::PathBuf};
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    let build = world.installable_build(2);
    let name = PathBuf::from(OsString::from_vec(b"bundle/weird\\\n\r\xff".to_vec()));
    fs::write(build.join(&name), b"escaped asset").expect("byte path");
    let mut line = format!("\\{:x}  bundle/weird\\\\\\n\\r", Sha256::digest(b"escaped asset")).into_bytes();
    line.extend_from_slice(b"\xff\n");
    File::options()
        .append(true)
        .open(build.join("manifest"))
        .expect("manifest")
        .write_all(&line)
        .expect("escaped manifest row");
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 1);
    assert_eq!(
        fs::read(world.home.directory.join("places/1").join(name)).expect("installed byte path"),
        b"escaped asset"
    );
}

#[test]
fn triage_and_pass_sessions_run_together_and_keep_their_own_bounds() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::IgnoreTerm, Agent::IgnoreTerm]);
    run.home.configuration.bounds.pass = 20;
    run.tick();
    land(&mut run, 2, 7);
    world.installable_build(2);
    run.tick();
    run.tick();
    let _holder = hold(&world.home.directory.join("places/1"));
    land(&mut run, 4, 8);
    run.tick();
    assert_eq!(world.ledger().running_sessions().len(), 2);
    run.clock.time = 19;
    run.tick();
    assert!(run.processes.signals.is_empty());
    run.clock.time = 20;
    run.tick();
    assert_eq!(run.processes.signals, [(2, Signal::Term)]);
    assert_eq!(world.ledger().sessions[1].exit, None);
    run.clock.time = 29;
    run.tick();
    assert_eq!(run.processes.signals.len(), 1);
    run.clock.time = 30;
    run.tick();
    assert_eq!(run.processes.signals, [(2, Signal::Term), (2, Signal::Kill), (3, Signal::Term)]);
    assert_eq!(world.ledger().sessions[1].exit, Some(Exit::Bound));
    assert_eq!(world.ledger().sessions[2].exit, None);
    run.clock.time = 39;
    run.tick();
    assert_eq!(run.processes.signals.len(), 3);
    run.clock.time = 40;
    run.tick();
    assert_eq!(run.processes.signals.last(), Some(&(3, Signal::Kill)));
    assert_eq!(world.ledger().sessions[2].exit, Some(Exit::Bound));
    assert!(!world.home.session_directory(2).join("tree").exists());
    assert!(!world.home.session_directory(3).join("tree").exists());
}

#[test]
fn agent_exits_apply_recorded_outcomes_or_nothing_and_write_the_report() {
    for (agent, state, blocks, pass_state) in [
        (Agent::Pass, CaseState::Passed, 0, PassState::Ended),
        (Agent::Note, CaseState::Queued, 0, PassState::Ended),
        (Agent::Empty, CaseState::Queued, 1, PassState::Ended),
        (Agent::Blocked, CaseState::Queued, 1, PassState::Ended),
        (Agent::Fail, CaseState::Failed, 0, PassState::Waiting),
    ] {
        let world = World::new();
        let mut run = ready(&world, agent);
        let holder = hold(&world.home.directory.join("places/1"));
        run.tick();
        let ledger = world.ledger();
        assert_eq!(ledger.cases[0].state, state);
        assert_eq!(ledger.cases[0].blocks, blocks);
        assert_eq!(ledger.passes[0].state, pass_state);
        assert_eq!(ledger.sessions[1].exit, Some(Exit::Code(0)));
        assert_eq!(ledger.sessions[1].tokens, Some(1234));
        assert!(!world.home.session_directory(2).join("tree").exists());
        assert!(world.home.session_directory(2).join("brief").is_file());
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(world.home.directory.join("passes/1/report.json")).expect("report file"))
                .expect("report JSON");
        assert_eq!(report, ledger.report(1).expect("report value"));
        if pass_state == PassState::Waiting {
            assert_eq!(ledger.passes[0].requests[0].findings, [1]);
            assert_eq!(ledger.builds[0].held, Some(1));
        } else {
            assert_eq!(ledger.builds[0].held, None);
        }
        if matches!(agent, Agent::Blocked) {
            drop(holder);
            run.processes.next.push_back(Agent::Blocked);
            run.tick();
            run.tick();
            let ledger = world.ledger();
            assert_eq!(ledger.cases[0].state, CaseState::Blocked);
            assert_eq!(ledger.cases[0].blocks, 2);
            assert!(ledger.lines.iter().any(|line| line.words.contains("blocked twice")));
        }
    }
}

#[test]
fn a_failed_report_write_keeps_the_end_recorded_and_retries_the_report() {
    let world = World::new();
    let mut run = ready(&world, Agent::Pass);
    let directory = world.home.directory.join("passes/1");
    fs::create_dir(directory.join("report.tmp")).expect("report write fault");
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[1].exit, Some(Exit::Code(0)));
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.cases[0].state, CaseState::Passed);
    assert!(!directory.join("report.json").exists());
    fs::remove_dir(directory.join("report.tmp")).expect("report repair");
    run.tick();
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("report.json")).expect("retried report")).expect("JSON");
    assert_eq!(report, world.ledger().report(1).expect("report value"));
    assert!(!world.home.session_directory(2).join("tree").exists());
}

#[test]
fn changed_recipes_on_the_request_compare_the_three_files_to_the_build_commit() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Fail]);
    run.tick();
    land(&mut run, 2, 7);
    for name in ["recipe", "readme", "scripts", "missing", "unchanged"] {
        recipe(&mut run, 2, name, "A state.");
    }
    world.installable_build(2);
    run.tick();
    run.tick();
    let setups = world.home.directory.join("passes/1/setups");
    for path in ["recipe/recipe", "readme/README.md", "scripts/scripts.json", "unchanged/assets/note"] {
        fs::write(setups.join(path), b"changed").expect("agent's edit");
    }
    fs::remove_file(setups.join("missing/scripts.json")).expect("agent removes scripts");
    fs::create_dir_all(setups.join("new")).expect("new recipe");
    fs::write(setups.join("new/recipe"), b"recipe 1\n").expect("new recipe");
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes[0].requests[0].recipes, ["missing", "new", "readme", "recipe", "scripts"]);
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
}

#[test]
fn an_unstarted_agent_undoes_the_pass_start_and_removes_its_tree() {
    let world = World::new();
    let mut run = ready(&world, Agent::Unstarted);
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[1].exit, Some(Exit::Unstarted));
    assert_eq!(ledger.cases[0].state, CaseState::Queued);
    assert_eq!(ledger.cases[0].pass, None);
    assert_eq!(ledger.cases[0].blocks, 0);
    assert!(ledger.cases[0].outcomes.is_empty());
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.builds[0].held, None);
    assert!(!world.home.session_directory(2).join("tree").exists());
    run.tick();
    assert_eq!(world.ledger().passes[1].state, PassState::Running);
    assert_eq!(world.ledger().sessions[2].exit, None);
}

#[test]
fn a_waiting_pass_resumes_before_a_start_on_a_later_build_with_findings_first() {
    let world = World::new();
    let mut run = ready(&world, Agent::Fail);
    run.tick();
    world.home.transact(|ledger| ledger.file_request(1, 1, Some(99), 1)).expect("filed request");
    land(&mut run, 4, 99);
    run.git.commit(5, &[4], "later than the goal's landing");
    recipe(&mut run, 5, "project", "An updated project.");
    world.installable_build(5);
    run.processes.next.extend([Agent::Case, Agent::Never]);
    run.tick();
    world
        .home
        .transact(|ledger| {
            let mut case = super::case();
            case.page = "chat".into();
            ledger.add_case(3, &case, 1)?;
            Ok(())
        })
        .expect("queued case competing for a new start");
    let setups = world.home.directory.join("passes/1/setups");
    fs::write(setups.join("project/recipe"), b"pass's old edit").expect("old edit");
    fs::create_dir_all(setups.join("old-only")).expect("old recipe");
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 1, "resume before any new start");
    assert_eq!(ledger.passes[0].build, id(5));
    assert_eq!(ledger.passes[0].sessions, [2, 4]);
    assert_eq!(ledger.passes[0].cases, [1, 3, 2]);
    assert!(ledger.cases.iter().all(|case| case.state == CaseState::Assigned && case.pass == Some(1)));
    assert_eq!(ledger.builds[0].held, None);
    assert_eq!(ledger.builds[1].held, Some(1));
    assert_eq!(fs::read_to_string(setups.join("project/README.md")).expect("new setups"), "An updated project.");
    assert_eq!(fs::read(setups.join("project/recipe")).expect("reset recipe"), b"recipe 1\nweb see Ready\n");
    assert!(!setups.join("old-only").exists());
    let brief = fs::read_to_string(world.home.session_directory(4).join("brief")).expect("resume brief");
    let mut previous = 0;
    for heading in [
        "# What a pass is",
        "## Pass",
        "## Landings",
        "## Open findings",
        "## Failed cases",
        "## Cases",
        "## Setups",
        "## Commands",
    ] {
        let at = brief.find(heading).expect("resume heading");
        assert!(at >= previous, "{heading}");
        previous = at;
    }
    assert!(brief.contains("Resumed after goal 99: landed"));
    assert!(brief.contains("Finding 1: Start does nothing\nSetup: fresh\nLines:\n- web press button Start\nExpected: a goal starts\nSeen: no goal"));
    assert!(brief.contains("## Failed cases\n\nRun these after the open findings.\nCase 1: web / work"));
    assert!(run.git.calls.contains(&format!("add {}", id(5))));
    run.processes.finish(4, Agent::Gone);
    let _holder = hold(&world.home.directory.join("places/1"));
    run.tick();
    assert_eq!(world.ledger().findings[0].state, temper_qa_tool::FindingState::Closed);
    assert_eq!(world.ledger().findings[0].closed, Some(id(5)));
}

#[test]
fn an_unstarted_resume_restores_the_waiting_build_and_retries_without_a_landing() {
    let world = World::new();
    let mut run = ready(&world, Agent::Fail);
    run.tick();
    world
        .home
        .transact(|ledger| {
            ledger.file_request(1, 1, Some(99), 1)?;
            ledger.end_goal(99, 2)
        })
        .expect("goal ended without landing");
    run.git.commit(3, &[2], "later build");
    recipe(&mut run, 3, "project", "A later state.");
    world.installable_build(3);
    run.processes.next.extend([Agent::Unstarted, Agent::Never]);
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[2].exit, Some(Exit::Unstarted));
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
    assert_eq!(ledger.passes[0].build, id(2));
    assert_eq!(ledger.cases[0].state, CaseState::Failed);
    assert_eq!(ledger.cases[0].blocks, 0);
    assert_eq!(ledger.builds[0].held, Some(1));
    assert_eq!(ledger.builds[1].held, None);
    assert!(!world.home.session_directory(3).join("tree").exists());
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 1);
    assert_eq!(ledger.passes[0].state, PassState::Running);
    // The released build has no queued cases and the builds step deletes it;
    // a goal ended without landing may retry on the still-kept prior build.
    assert_eq!(ledger.builds[1].state, temper_qa_tool::BuildState::Deleted);
    assert_eq!(ledger.passes[0].build, id(2));
    assert_eq!(ledger.sessions[3].exit, None);
    let brief = fs::read_to_string(world.home.session_directory(4).join("brief")).expect("resume brief");
    assert!(brief.contains("Resumed after goal 99: ended without landing"));
}

#[test]
fn places_start_in_configuration_order_and_each_keeps_its_running_session() {
    let world = World::new();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never, Agent::Case, Agent::Never]);
    run.home.configuration.places = vec!["places/z".into(), "places/a".into()];
    run.tick();
    land(&mut run, 2, 7);
    world.installable_build(2);
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes[0].place, "places/z");
    land(&mut run, 4, 8);
    world.installable_build(4);
    run.tick();
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 2);
    assert_eq!(ledger.passes[1].place, "places/a");
    assert_eq!(ledger.running_sessions().len(), 2);
    assert_eq!(ledger.passes[0].build, id(2));
    assert_eq!(ledger.passes[1].build, id(4));
    run.processes.finish(2, Agent::Pass);
    run.processes.finish(4, Agent::Pass);
    run.tick();
    let ledger = world.ledger();
    assert!(ledger.running_sessions().is_empty());
    assert!(ledger.passes.iter().all(|pass| pass.state == PassState::Ended));
    for number in [1, 2] {
        assert!(world.home.directory.join(format!("passes/{number}/report.json")).is_file());
    }
}

#[test]
fn restart_ends_running_pass_and_triage_lost_with_changed_setups_and_a_report() {
    let world = World::new();
    let mut run = ready(&world, Agent::Never);
    land(&mut run, 4, 8);
    run.tick();
    fs::write(world.home.directory.join("passes/1/setups/project/README.md"), b"new description")
        .expect("edited setup");
    let git = std::mem::take(&mut run.git);
    drop(run);
    let restarted = world.start(git, &[]);
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[1].exit, Some(Exit::Lost));
    assert_eq!(ledger.sessions[2].exit, Some(Exit::Lost));
    assert!(ledger.running_sessions().is_empty());
    assert_eq!(ledger.cases[0].blocks, 1);
    assert_eq!(ledger.passes[0].requests[0].recipes, ["project"]);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(world.home.directory.join("passes/1/report.json")).expect("lost report"))
            .expect("JSON");
    assert_eq!(report, ledger.report(1).expect("report value"));
    for number in [2, 3] {
        assert!(!world.home.session_directory(number).join("tree").exists());
    }
    assert!(restarted.git.prunes > 0);
}
