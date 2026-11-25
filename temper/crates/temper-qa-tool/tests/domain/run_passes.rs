//! The loop's pass step over commits, recorded agent actions and movable time (qa/passes.md, 9).
use std::{io::Write, path::Path};
use temper_qa_tool::{
    BuildState, CaseState, Exit, PassState,
    faces::{DirectoryMode, FilesFace, Mode, PathKind, RemoveMode, Signal, WriteMode, WriteOptions},
    testing::{Agent, FileOp, Lock, Loop, Tree, World, baseline, id, land},
};

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

fn hold(files: &mut Tree, place: &Path) -> Lock {
    files.directory(place, DirectoryMode::Parents).expect("place");
    files.hold_lock(place.join("drive.lock")).expect("holder takes lock")
}

#[test]
#[expect(clippy::too_many_lines, reason = "one pass start pins its installed build, environment and fakes-only brief")]
fn start_installs_the_newest_reached_build_and_writes_an_ordered_brief() {
    let mut world = World::new();
    let mut files = world.files();
    world.home.configuration.surfaces = vec!["web".into(), "tui".into()];
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
    files.directory(&place.join("bin"), DirectoryMode::Parents).expect("old bin");
    files.directory(&place.join("bundle"), DirectoryMode::Parents).expect("old bundle");
    files.write(&place.join("bin/stale"), b"stale", WriteOptions::REPLACE, false).expect("old executable");
    files.write(&place.join("bundle/stale"), b"stale", WriteOptions::REPLACE, false).expect("old asset");
    files.write(&place.join("setup"), b"project oldhash", WriteOptions::REPLACE, false).expect("old setup");
    run.tick();
    world
        .home
        .transact(&mut files, |ledger, _| {
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
    run.home.configuration.surfaces = vec!["web".into()];
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
        assert_eq!(files.read(&place.join(path)).expect("installed"), files.read(&build.join(path)).expect("build"));
    }
    assert_eq!(files.mode(place.join("bin/temper")).expect("mode") & 0o777, 0o751);
    assert!(!files.exists(place.join("bin/stale")));
    assert!(!files.exists(place.join("bundle/stale")));
    assert!(!files.exists(place.join("setup")));
    assert_eq!(
        files.read(&world.home.directory.join("passes/1/setups/project/assets/note")).expect("recursive setups"),
        b"asset"
    );
    let command = &run.processes.commands[1];
    assert_eq!(command.arguments, ["agent", "--effort=high"]);
    assert_eq!(command.directory, world.home.pass_tree("places/1"));
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
    let brief = files.read_to_string(&command.stdin).expect("stdin brief");
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
    assert!(brief.contains("A ctl-only recipe's lines end with `check` and its ctl reads and waits."));
    assert!(brief.contains("shell's own timeout does not cut them"));
    let drive_lines = brief.lines().find(|line| line.starts_with("Drive lines:")).expect("drive lines");
    assert!(drive_lines.contains("`forge commit forgejo REPOSITORY MESSAGE PATH...`"));
    assert!(drive_lines.contains("`forge checks forgejo REPOSITORY`"));
    assert!(drive_lines.contains("`forge push forgejo REPOSITORY BRANCH MESSAGE PATH...`"));
    assert!(drive_lines.contains(r#"Quoted bytes take printable ASCII alone; write other bytes as `\xHH`, with `\"` and `\\` for quote and backslash."#));
    assert!(drive_lines.contains("at its repository path before the line"));
    assert!(brief.contains(&world.home.directory.join("passes/1/files").display().to_string()));
    for words in [
        "The fakes alone",
        "the person's forge is the box's fake Forgejo",
        "A case no line or setup can bring to its state is a gap, not blocked: `temper-qa pass gap CASE KIND WHY`",
        "{forgejo.api}",
        "{forgejo.git}",
        "{forgejo.admin}",
        "{git}",
        "--first work/hello",
        "--git-program {git}",
    ] {
        assert!(brief.contains(words), "{words}\n{brief}");
    }
    assert!(!brief.contains("owned Forgejo"));
    assert!(!drive_lines.contains("CONTENT"));
}

#[test]
fn pass_brief_says_the_happy_path_and_reads_help() {
    let world = World::new();
    let mut files = world.files();
    let run = ready(&world, Agent::Never);
    let brief = files.read_to_string(&run.processes.commands[1].stdin).expect("brief");
    assert!(brief.contains(super::HAPPY_PATH));
    assert!(brief.contains(
        "Run a case's intended-use steps alone; an edge case met is a note (temper-qa pass note), never a finding."
    ));
    assert!(brief.contains("A security hole or lost data still counts wherever it is."));
    assert!(brief.contains("Read a case's `temper help PAGE` with the place's own program, as `Help pages:` below names it. A help page needs no box; compare its words as a `see` is."));
    assert!(brief.lines().any(|line| line.starts_with("Help pages: /") && line.ends_with("/bin/temper help PAGE")));
}

#[test]
fn pass_brief_explains_web_requests_on_start_and_resume() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Fail);
    for session in [2, 3] {
        let brief = files.read_to_string(&world.home.session_directory(session).join("brief")).expect("brief");
        let purpose = brief.split("\n\n## Pass\n").next().expect("what a pass is");
        assert!(purpose.contains("judge the rest against the draft's promises. A `web` step whose do is `web requests` is that drive line: compare its see with the line's last shown line, `N requests in W waves`, which must equal it; one that differs is a finding, its seen the line's whole answer."), "{purpose}");
        let drive_lines = brief.lines().find(|line| line.starts_with("Drive lines:")).expect("drive lines");
        assert!(drive_lines.contains("`web look [WIDTH]`, `web requests`;"), "{drive_lines}");
        assert!(drive_lines.contains("A look prints accessible text or the TUI's frame; a web look with WIDTH also writes a screenshot. `web requests` answers the requests the page has sent since it last loaded, each with its wave, then `N requests in W waves`."), "{drive_lines}");
        if session == 2 {
            run.tick();
            world
                .home
                .transact(&mut files, |ledger, _| {
                    ledger.file_request(1, 1, Some(99), 1)?;
                    ledger.end_goal(99, 2)
                })
                .expect("ended goal");
            run.git.commit(3, &[2], "build for the resume");
            world.installable_build(3);
            run.processes.next.push_back(Agent::Never);
            run.tick();
            assert_eq!(world.ledger().passes[0].sessions.len(), 2, "resumed the pass");
        }
    }
}

#[test]
fn a_pass_starts_with_empty_files_and_keeps_them_on_resume() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Fail);
    let directory = world.home.directory.join("passes/1/files");
    assert_eq!(files.kind(&directory), Some(PathKind::Directory), "a pass starts with files/");
    assert_eq!(files.list(&directory).expect("files directory").len(), 0);
    files.directory(&directory.join("dir"), DirectoryMode::One).expect("repository directory");
    files
        .write(&directory.join("dir/b.txt"), b"kept across resumes", WriteOptions::REPLACE, false)
        .expect("commit file");
    run.tick();
    world
        .home
        .transact(&mut files, |ledger, _| {
            ledger.file_request(1, 1, Some(99), 1)?;
            ledger.end_goal(99, 2)
        })
        .expect("ended goal");
    run.git.commit(3, &[2], "build for the resume");
    world.installable_build(3);
    run.processes.next.push_back(Agent::Never);
    run.tick();
    assert_eq!(world.ledger().passes[0].sessions.len(), 2, "resumed the pass");
    assert_eq!(files.read(&directory.join("dir/b.txt")).expect("kept file"), b"kept across resumes");
}

#[test]
fn a_tui_pass_brief_explains_see_and_row_evidence() {
    let world = World::new();
    let mut files = world.files();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    world.installable_build(2);
    run.tick();
    world
        .home
        .transact(&mut files, |ledger, _| {
            let mut tui = super::case();
            tui.surface = "tui".into();
            tui.page = "chat".into();
            ledger.add_case(1, &tui, 0)?;
            Ok(())
        })
        .expect("tui case");
    run.tick();
    let brief = files.read_to_string(&run.processes.commands[1].stdin).expect("brief");
    assert!(brief.contains("Case 1: tui / chat"), "{brief}");
    assert!(brief.contains("`web see` line for a `web` case and a `tui see` line for a `tui` case"));
    assert!(brief.contains("`pass line \"tui look\"`'s answer saved to a file"));
    assert!(brief.contains("TUI starts at its first line and the session signs it in"));
}

#[test]
fn third_step_opens_a_wait_behind_a_holder_and_closes_after_cases_are_taken() {
    let world = World::new();
    let mut files = world.files();
    let lock = hold(&mut files, &world.home.directory.join("places/1"));
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    assert!(world.ledger().place_waits.is_empty(), "no build or cases");
    land(&mut run, 2, 7);
    world.installable_build(2);
    run.clock.time = 10;
    run.tick();
    assert!(world.ledger().place_waits.is_empty(), "triage has not reached the build");
    run.clock.time = 20;
    run.tick();
    assert_eq!(world.ledger().place_waits, [temper_qa_tool::PlaceWait { from: 20, to: None }]);
    assert_eq!(run.wait_timeout().expect("place retry"), Some(60));
    run.clock.time = 30;
    run.tick();
    assert_eq!(world.ledger().place_waits.len(), 1, "an open wait is not duplicated");
    drop(lock);
    run.clock.time = 40;
    run.tick();
    assert_eq!(world.ledger().place_waits, [temper_qa_tool::PlaceWait { from: 20, to: Some(40) }]);
    assert_eq!(world.ledger().cases[0].state, CaseState::Assigned);
}

#[test]
fn a_busy_drive_lock_keeps_the_place_free_of_new_installs_or_records() {
    let world = World::new();
    let mut files = world.files();
    let place = world.home.directory.join("places/1");
    let lock = hold(&mut files, &place);
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    world.installable_build(2);
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 0);
    assert_eq!(run.processes.commands.len(), 1);
    assert!(!files.exists(place.join("bin")));
    assert!(!files.exists(world.home.directory.join("passes")));
    drop(lock);
    run.tick();
    assert_eq!(world.ledger().passes[0].place, "places/1");
    assert_eq!(run.processes.commands.len(), 2);
    run.tick();
    assert_eq!(run.processes.commands.len(), 2, "a recorded running pass keeps the place busy too");
}

#[test]
fn a_bad_manifest_names_the_path_records_nothing_and_retries_next_tick() {
    let world = World::new();
    let mut files = world.files();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    let build = world.installable_build(2);
    let manifest = files.read_to_string(&build.join("manifest")).expect("manifest");
    let shortened =
        manifest.lines().filter(|line| !line.ends_with("  bundle/assets/app.js")).collect::<Vec<_>>().join("\n");
    files
        .write(&build.join("manifest"), shortened.as_bytes(), WriteOptions::REPLACE, false)
        .expect("missing manifest entry");
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 0);
    assert_eq!(world.ledger().sessions.len(), 1);
    assert_eq!(world.ledger().cases[0].state, CaseState::Queued);
    run.clock.time = 3600;
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 0);
    assert!(ledger.lines.iter().any(|line| {
        line.words.contains("passes has failed for an hour: bundle/assets/app.js: file is absent from manifest")
    }));
    files.write(&build.join("manifest"), manifest.as_bytes(), WriteOptions::REPLACE, false).expect("repaired manifest");
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

#[test]
fn installation_reads_sha256sum_escaped_byte_paths() {
    installation_reads_path(std::path::Path::new("bundle/weird\\\n\r"), b"bundle/weird\\\\\\n\\r");
}

#[test]
fn installation_reads_a_non_utf8_manifest_path_as_its_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
    installation_reads_path(
        &PathBuf::from(OsString::from_vec(b"bundle/weird\\\n\r\xff".to_vec())),
        b"bundle/weird\\\\\\n\\r\xff",
    );
}

fn installation_reads_path(name: &std::path::Path, escaped: &[u8]) {
    use sha2::{Digest, Sha256};
    let world = World::new();
    let mut files = world.files();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    let build = world.installable_build(2);
    files.write(&build.join(name), b"escaped asset", WriteOptions::REPLACE, false).expect("byte path");
    let mut line = format!("\\{:x}  ", Sha256::digest(b"escaped asset")).into_bytes();
    line.extend_from_slice(escaped);
    line.push(b'\n');
    files
        .open_write(&build.join("manifest"), WriteOptions { way: WriteMode::Append, mode: Mode::Default, read: false })
        .expect("manifest")
        .write_all(&line)
        .expect("escaped manifest row");
    run.tick();
    run.tick();
    assert_eq!(world.ledger().passes.len(), 1);
    assert_eq!(
        files.read(&world.home.directory.join("places/1").join(name)).expect("installed byte path"),
        b"escaped asset"
    );
}

#[test]
fn triage_and_pass_sessions_run_together_and_keep_their_own_bounds() {
    let world = World::new();
    let mut files = world.files();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::IgnoreTerm, Agent::IgnoreTerm]);
    run.home.configuration.bounds.pass = 20;
    run.tick();
    land(&mut run, 2, 7);
    world.installable_build(2);
    run.tick();
    run.tick();
    let _holder = hold(&mut files, &world.home.directory.join("places/1"));
    land(&mut run, 4, 8);
    run.tick();
    assert_eq!(world.ledger().running_sessions().len(), 2);
    run.clock.time = 19;
    run.tick();
    assert_eq!(run.processes.signals.len(), 0);
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
    assert_eq!(files.kind(world.home.pass_tree("places/1")), Some(PathKind::Directory));
    assert_eq!(files.kind(world.home.triage_tree()), Some(PathKind::Directory));
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
        let mut files = world.files();
        let mut run = ready(&world, agent);
        let place = world.home.directory.join("places/1");
        let holder = hold(&mut files, &place);
        run.tick();
        assert_eq!(run.processes.ended_holders, [place]);
        let ledger = world.ledger();
        assert_eq!(ledger.cases[0].state, state);
        assert_eq!(ledger.cases[0].blocks, blocks);
        assert_eq!(ledger.passes[0].state, pass_state);
        assert_eq!(ledger.sessions[1].exit, Some(Exit::Code(0)));
        assert_eq!(ledger.sessions[1].tokens, Some(1234));
        assert_eq!(files.kind(world.home.pass_tree("places/1")), Some(PathKind::Directory));
        assert_eq!(files.kind(world.home.session_directory(2).join("brief")), Some(PathKind::File));
        let report: serde_json::Value = serde_json::from_slice(
            &files.read(&world.home.directory.join("passes/1/report.json")).expect("report file"),
        )
        .expect("report JSON");
        assert_eq!(report, ledger.report(1).expect("report value"));
        if pass_state == PassState::Waiting {
            assert_eq!(ledger.passes[0].requests[0].findings, [1]);
            assert_eq!(ledger.findings[0].state, temper_qa_tool::FindingState::Open);
            assert_eq!(ledger.builds[0].state, BuildState::Deleted);
            assert!(!files.exists(world.home.builds.join(id(2))));
        }
        assert_eq!(ledger.builds[0].held, None);
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
fn a_waiting_pass_releases_a_build_kept_for_a_queued_case_it_partly_holds() {
    let world = World::new();
    let mut files = world.files();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never, Agent::Empty]);
    run.tick();
    land(&mut run, 2, 7);
    let build = world.installable_build(2);
    let mut tui = super::case();
    tui.surface = "tui".into();
    tui.page = "chat".into();
    run.tick();
    world.home.transact(&mut files, |ledger, _| ledger.add_case(1, &tui, 0)).expect("first tui ask");
    let surfaces = std::mem::replace(&mut run.home.configuration.surfaces, vec!["web".into()]);
    run.tick();

    run.home.configuration.surfaces = surfaces;
    land(&mut run, 4, 8);
    run.tick();
    world.home.transact(&mut files, |ledger, _| ledger.add_case(3, &tui, 0)).expect("second tui ask");
    run.processes.finish(2, Agent::Fail);
    run.tick();

    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 1, "no build can run the queued case");
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
    let queued = ledger.cases.iter().find(|case| case.state == CaseState::Queued).expect("queued case");
    assert_eq!(queued.landings(), [1, 2]);
    assert_eq!(ledger.builds[0].holds.as_deref(), Some([1].as_slice()));
    assert_eq!(ledger.builds[0].held, None);
    assert_eq!(ledger.builds[0].state, BuildState::Kept);
    assert!(files.exists(build));
}

#[test]
fn a_tick_deletes_a_legacy_waiting_pass_build_without_queued_cases() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Fail);
    let build = world.home.builds.join(id(2));
    files.inject(&build, FileOp::Remove, std::io::ErrorKind::PermissionDenied);
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
    assert!(files.exists(&build), "the failed deletion left the build on disk");

    let mut legacy = ledger.document();
    legacy["builds"][0]["held"] = serde_json::json!(1);
    let bytes = serde_json::to_vec(&legacy).expect("legacy ledger JSON");
    files.write(&world.home.ledger, &bytes, WriteOptions::REPLACE, false).expect("legacy ledger on disk");
    run.tick();

    assert_eq!(world.ledger().builds[0].state, BuildState::Deleted);
    assert!(!files.exists(&build));
    let saved: serde_json::Value =
        serde_json::from_slice(&files.read(&world.home.ledger).expect("saved ledger")).expect("saved ledger JSON");
    assert!(saved["builds"][0].get("held").is_none());
}

#[test]
fn a_failed_report_write_keeps_the_end_recorded_and_retries_the_report() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Pass);
    let directory = world.home.directory.join("passes/1");
    files.directory(&directory.join("report.tmp"), DirectoryMode::One).expect("report write fault");
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[1].exit, Some(Exit::Code(0)));
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.cases[0].state, CaseState::Passed);
    assert!(!files.exists(directory.join("report.json")));
    files.remove(&directory.join("report.tmp"), RemoveMode::Tree).expect("report repair");
    run.tick();
    let report: serde_json::Value =
        serde_json::from_slice(&files.read(&directory.join("report.json")).expect("retried report")).expect("JSON");
    assert_eq!(report, world.ledger().report(1).expect("report value"));
    assert_eq!(files.kind(world.home.pass_tree("places/1")), Some(PathKind::Directory));
}

#[test]
fn changed_recipes_on_the_request_compare_the_three_files_to_the_build_commit() {
    let world = World::new();
    let mut files = world.files();
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
        files.write(&setups.join(path), b"changed", WriteOptions::REPLACE, false).expect("agent's edit");
    }
    files.remove(&setups.join("missing/scripts.json"), RemoveMode::File).expect("agent removes scripts");
    files.directory(&setups.join("new"), DirectoryMode::Parents).expect("new recipe");
    files.write(&setups.join("new/recipe"), b"recipe 1\n", WriteOptions::REPLACE, false).expect("new recipe");
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes[0].requests[0].recipes, ["missing", "new", "readme", "recipe", "scripts"]);
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
}

#[test]
fn an_unstarted_agent_undoes_the_pass_start_and_keeps_its_tree() {
    let world = World::new();
    let files = world.files();
    let mut run = ready(&world, Agent::Unstarted);
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[1].exit, Some(Exit::Unstarted));
    assert_eq!(ledger.cases[0].state, CaseState::Queued);
    assert_eq!(ledger.cases[0].pass, None);
    assert_eq!(ledger.cases[0].blocks, 0);
    assert_eq!(ledger.cases[0].outcomes.len(), 0);
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.builds[0].held, None);
    assert_eq!(files.kind(world.home.pass_tree("places/1")), Some(PathKind::Directory));
    run.tick();
    assert_eq!(world.ledger().passes[1].state, PassState::Running);
    assert_eq!(world.ledger().sessions[2].exit, None);
}

#[test]
fn a_waiting_pass_resumes_before_a_start_on_a_later_build_with_findings_first() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Fail);
    run.tick();
    assert_eq!(
        world.ledger().builds[0].state,
        BuildState::Deleted,
        "waiting released the old build before the landing"
    );
    world.home.transact(&mut files, |ledger, _| ledger.file_request(1, 1, Some(99), 1)).expect("filed request");
    land(&mut run, 4, 99);
    run.git.commit(5, &[4], "later than the goal's landing");
    recipe(&mut run, 5, "project", "An updated project.");
    world.installable_build(5);
    run.processes.next.extend([Agent::Case, Agent::Never]);
    run.tick();
    world
        .home
        .transact(&mut files, |ledger, _| {
            let mut case = super::case();
            case.page = "chat".into();
            ledger.add_case(3, &case, 1)?;
            Ok(())
        })
        .expect("queued case competing for a new start");
    let setups = world.home.directory.join("passes/1/setups");
    files.write(&setups.join("project/recipe"), b"pass's old edit", WriteOptions::REPLACE, false).expect("old edit");
    files.directory(&setups.join("old-only"), DirectoryMode::Parents).expect("old recipe");
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 1, "resume before any new start");
    assert_eq!(ledger.passes[0].build, id(5));
    assert_eq!(ledger.passes[0].sessions, [2, 4]);
    assert_eq!(ledger.passes[0].cases, [1, 3, 2]);
    assert!(ledger.cases.iter().all(|case| case.state == CaseState::Assigned && case.pass == Some(1)));
    assert_eq!(ledger.builds[0].held, None);
    assert_eq!(ledger.builds[0].state, BuildState::Deleted);
    assert_eq!(ledger.builds[1].held, Some(1));
    assert_eq!(files.read_to_string(&setups.join("project/README.md")).expect("new setups"), "An updated project.");
    assert_eq!(files.read(&setups.join("project/recipe")).expect("reset recipe"), b"recipe 1\nweb see Ready\n");
    assert!(!files.exists(setups.join("old-only")));
    let brief = files.read_to_string(&world.home.session_directory(4).join("brief")).expect("resume brief");
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
    assert!(run.git.calls.contains(&format!("checkout {}", id(5))));
    run.processes.finish(4, Agent::Gone);
    let _holder = hold(&mut files, &world.home.directory.join("places/1"));
    run.tick();
    assert_eq!(world.ledger().findings[0].state, temper_qa_tool::FindingState::Closed);
    assert_eq!(world.ledger().findings[0].closed, Some(id(5)));
}

#[test]
fn a_goal_ended_without_landing_waits_for_a_build_then_resumes_on_the_newest() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Fail);
    run.tick();
    world
        .home
        .transact(&mut files, |ledger, _| {
            ledger.file_request(1, 1, Some(99), 1)?;
            ledger.end_goal(99, 2)
        })
        .expect("goal ended without landing");
    for _ in 0..2 {
        run.tick();
        let ledger = world.ledger();
        assert_eq!(ledger.passes[0].state, PassState::Waiting);
        assert_eq!(ledger.sessions.len(), 2, "no build to resume on");
        assert_eq!(ledger.builds[0].state, BuildState::Deleted);
        assert_eq!(ledger.lines.len(), 0);
    }

    run.git.commit(3, &[2], "older available build");
    run.git.commit(4, &[3], "newest available build");
    world.installable_build(3);
    world.installable_build(4);
    run.processes.next.push_back(Agent::Never);
    run.tick();

    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 1);
    assert_eq!(ledger.passes[0].state, PassState::Running);
    assert_eq!(ledger.passes[0].build, id(4));
    assert_eq!(ledger.passes[0].sessions, [2, 3]);
    assert_eq!(ledger.builds[1].held, None);
    assert_eq!(ledger.builds[2].held, Some(1));
}

#[test]
fn an_unstarted_resume_restores_the_waiting_pass_without_a_hold_then_retries() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Fail);
    run.tick();
    world
        .home
        .transact(&mut files, |ledger, _| {
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
    assert_eq!(ledger.builds[0].held, None);
    assert_eq!(ledger.builds[1].held, None);
    assert_eq!(ledger.builds[0].state, BuildState::Deleted);
    assert_eq!(ledger.builds[1].state, BuildState::Deleted);
    assert_eq!(files.kind(world.home.pass_tree("places/1")), Some(PathKind::Directory));
    run.git.commit(4, &[3], "build for the retry");
    world.installable_build(4);
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.passes.len(), 1);
    assert_eq!(ledger.passes[0].state, PassState::Running);
    assert_eq!(ledger.passes[0].build, id(4));
    assert_eq!(ledger.builds[2].held, Some(1));
    assert_eq!(ledger.sessions[3].exit, None);
    let brief = files.read_to_string(&world.home.session_directory(4).join("brief")).expect("resume brief");
    assert!(brief.contains("Resumed after goal 99: ended without landing"));
}

#[test]
fn places_start_in_configuration_order_and_each_keeps_its_running_session() {
    let world = World::new();
    let files = world.files();
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
    let first = run.home.pass_tree("places/z");
    let second = run.home.pass_tree("places/a");
    assert_ne!(first, second);
    assert_eq!(run.processes.commands[1].directory, first);
    assert_eq!(run.processes.commands[3].directory, second);
    assert!(files.kind(first) == Some(PathKind::Directory) && files.kind(second) == Some(PathKind::Directory));
    run.processes.finish(2, Agent::Pass);
    run.processes.finish(4, Agent::Pass);
    run.tick();
    let ledger = world.ledger();
    assert_eq!(ledger.running_sessions().len(), 0);
    assert!(ledger.passes.iter().all(|pass| pass.state == PassState::Ended));
    for number in [1, 2] {
        assert_eq!(files.kind(world.home.directory.join(format!("passes/{number}/report.json"))), Some(PathKind::File));
    }
}

#[test]
fn restart_ends_running_pass_and_triage_lost_with_changed_setups_and_a_report() {
    let world = World::new();
    let mut files = world.files();
    let mut run = ready(&world, Agent::Never);
    land(&mut run, 4, 8);
    run.tick();
    files
        .write(
            &world.home.directory.join("passes/1/setups/project/README.md"),
            b"new description",
            WriteOptions::REPLACE,
            false,
        )
        .expect("edited setup");
    let git = std::mem::take(&mut run.git);
    drop(run);
    let restarted = world.start(git, &[]);
    let ledger = world.ledger();
    assert_eq!(ledger.sessions[1].exit, Some(Exit::Lost));
    assert_eq!(ledger.sessions[2].exit, Some(Exit::Lost));
    assert_eq!(ledger.running_sessions().len(), 0);
    assert_eq!(ledger.cases[0].blocks, 1);
    assert_eq!(ledger.passes[0].requests[0].recipes, ["project"]);
    let report: serde_json::Value =
        serde_json::from_slice(&files.read(&world.home.directory.join("passes/1/report.json")).expect("lost report"))
            .expect("JSON");
    assert_eq!(report, ledger.report(1).expect("report value"));
    assert_eq!(files.kind(world.home.pass_tree("places/1")), Some(PathKind::Directory));
    assert_eq!(files.kind(world.home.triage_tree()), Some(PathKind::Directory));
    assert_eq!(restarted.git.prunes, 0);
}

#[test]
fn pass_brief_lists_ctl_cases_first_and_opens_a_ctl_session() {
    let world = World::new();
    let mut files = world.files();
    let mut run = world.start(baseline(), &[Agent::Case, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.git.file_at(2, "temper/docs/help/reference/ctl-task.md", b"# Tasks");
    world.installable_build(2);
    run.tick();
    world
        .home
        .transact(&mut files, |ledger, _| {
            ledger.add_case(1, &super::case(), 0)?;
            let mut tui = super::case();
            tui.surface = "tui".into();
            tui.page = "chat".into();
            ledger.add_case(1, &tui, 0)?;
            let mut ctl = super::case();
            ctl.surface = "ctl".into();
            ctl.page = "task".into();
            ctl.steps[0].act = "ctl task list".into();
            ledger.add_case(1, &ctl, 0)?;
            Ok(())
        })
        .expect("three surfaces");
    run.tick();
    let brief = files.read_to_string(&run.processes.commands[1].stdin).expect("brief");
    let cases = brief.split("## Cases").nth(1).expect("cases");
    assert!(cases.find("ctl / task").expect("ctl") < cases.find("web / work").expect("web"));
    assert!(cases.find("ctl / task").expect("ctl") < cases.find("tui / chat").expect("tui"));
    assert!(brief.contains("Run ctl cases first in a ctl session, `temper-qa pass open --ctl`"));
}

#[test]
fn a_second_pass_session_moves_its_places_tree_or_makes_it_again_after_a_failed_move() {
    for fails in [false, true] {
        let world = World::new();
        let mut files = world.files();
        let mut run = ready(&world, Agent::Empty);
        let tree = world.home.pass_tree("places/1");
        let triage = world.home.triage_tree();
        files.write(&tree.join("leftover"), b"old pass", WriteOptions::REPLACE, false).expect("leftover");
        files.write(&triage.join("triage-file"), b"separate tree", WriteOptions::REPLACE, false).expect("triage");
        run.git.move_error = fails.then(|| "checkout refused".into());
        let before = run.git.calls.len();
        run.tick();
        let calls = &run.git.calls[before..];
        assert_eq!(run.processes.commands[2].directory, tree);
        assert!(calls.contains(&format!("checkout {}", id(2))));
        assert_eq!(calls.iter().filter(|call| call.starts_with("add ")).count(), usize::from(fails));
        assert_eq!(calls.iter().filter(|call| call.starts_with("remove ")).count(), usize::from(fails));
        assert!(!files.exists(tree.join("leftover")));
        assert_eq!(files.kind(triage.join("triage-file")), Some(PathKind::File));
        let brief = files.read_to_string(&run.processes.commands[2].stdin).expect("brief");
        assert!(brief.contains(&format!("Tree: {}", tree.display())));
    }
}

#[test]
fn ctl_only_pass_brief_lists_the_same_walkable_chains_as_triage() {
    let mut world = World::new();
    let mut files = world.files();
    world.home.configuration.surfaces = vec!["ctl".into()];
    let mut run = world.start(baseline(), &[Agent::Empty, Agent::Never]);
    run.tick();
    land(&mut run, 2, 7);
    run.git.file_at(2, "temper/docs/help/reference/ctl-task.md", b"# Task");
    for (name, lines) in [
        ("ctl-root", "check\nctl task list\n"),
        ("ctl-child", "after ctl-root\ncheck\nctl task list\n"),
        ("web-root", "web see Ready\n"),
        ("web-child", "after web-root\ncheck\nctl task list\n"),
        ("tui-root", "tui see Ready\n"),
        ("tui-child", "after tui-root\ncheck\nctl task list\n"),
    ] {
        recipe(&mut run, 2, name, "A state.");
        run.git.file_at(2, &format!("{SETUPS}/{name}/recipe"), format!("recipe 1\n{lines}").as_bytes());
    }
    world.installable_build(2);
    run.tick();
    let triage = files.read_to_string(&run.processes.commands[0].stdin).expect("triage brief");
    world
        .home
        .transact(&mut files, |ledger, _| {
            let mut case = super::case();
            case.surface = "ctl".into();
            case.page = "task".into();
            ledger.add_case(1, &case, 0)?;
            Ok(())
        })
        .expect("ctl case");
    run.tick();
    let brief = files.read_to_string(&run.processes.commands[1].stdin).expect("pass brief");
    for name in ["fresh", "ctl-root", "ctl-child", "web-root", "web-child", "tui-root", "tui-child"] {
        let listed = format!("- {name}:");
        assert_eq!(brief.contains(&listed), triage.contains(&format!("- {name} (box):")), "{name}");
        assert_eq!(brief.contains(&listed), name == "fresh" || name.starts_with("ctl-"), "{name}");
    }
}

#[test]
fn a_deployment_pass_installs_its_scratch_and_briefs_only_its_kind() {
    use temper_qa_tool::Kind;
    for kind in [Kind::Deployment, Kind::Box] {
        let mut world = World::new();
        let mut files = world.files();
        world.home.configuration.kinds.insert("places/1".into(), kind);
        world
            .home
            .configuration
            .ports
            .insert("places/1".into(), "31001,31002,31003,31004,31005,31006,31007,31008,31009".into());
        world
            .home
            .transact(&mut files, |ledger, _| {
                super::passes::queue_kinds(ledger);
                Ok(())
            })
            .expect("queue kinds");
        let mut git = baseline();
        git.merge(1, &[0], 11);
        git.commit(2, &[1], "build");
        for (name, recipe) in [
            ("root", "recipe 1\ncheck\nctl task list\n"),
            ("turn-landing", "recipe 1\nafter root\ncheck\nctl task list\n"),
            ("welcome", "recipe 1\ncheck\nctl task list\n"),
        ] {
            git.file_at(2, &format!("{SETUPS}/{name}/recipe"), recipe.as_bytes());
            git.file_at(2, &format!("{SETUPS}/{name}/README.md"), format!("The {name} state. More detail.").as_bytes());
        }
        git.file_at(2, &format!("{SETUPS}/root/config/engine/{}", temper_config_shell::FILE), &super::engine_file());
        world.installable_build(2);
        let mut run = world.start(git, &[Agent::Never]);
        run.tick();
        let ledger = world.ledger();
        assert_eq!(ledger.passes[0].kind, kind);
        assert_eq!(ledger.passes[0].cases, if kind == Kind::Deployment { vec![1] } else { vec![2] });
        let place = world.home.directory.join("places/1");
        assert_eq!(files.kind(place.join(kind.name())), Some(PathKind::Directory));
        assert!(!files.exists(place.join(if kind == Kind::Deployment { "box" } else { "deployment" })));
        let brief = files.read_to_string(&run.processes.commands[0].stdin).expect("brief");
        assert!(brief.contains(&format!("place places/1 ({})", kind.name())));
        let setups =
            brief.split("## Setups").nth(1).expect("setups").split("Commit files directory:").next().expect("section");
        if kind == Kind::Deployment {
            assert!(brief.contains("At this deployment's place run ctl cases first in a ctl session, `temper-qa pass open --ctl`, then web and tui cases in a full session, `temper-qa pass open`, the browser and the TUI on the deployment's engine."));
            assert!(brief.contains("The owner's changes are `deployment patch DOCUMENT PATH` lines; write their PATHs in the pass's files directory first. These lines end the TUI and load the page again."));
            assert!(setups.contains("- turn-landing: The turn-landing state."));
            assert!(!setups.contains("- fresh:"));
            assert!(!setups.contains("- welcome:"));
        } else {
            assert!(setups.contains("- fresh: an empty box"));
            assert!(setups.contains("- welcome: The welcome state."));
            assert!(!setups.contains("- turn-landing:"));
        }
    }
}
