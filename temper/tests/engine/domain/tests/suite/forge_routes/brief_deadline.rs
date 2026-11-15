//! Slow forge sections still start runs (temper/docs/design/domain/forge/change/repairs.md,
//! “A forge section is sized by its cutoff”; jig/docs/design/domain/engine/briefs.md, 1).
use super::*;

const STEPS: u32 = 600; // Usual 100 ms ticks cover discovery and the shipped 30 s brief.
const JOB_CUT: &str = "[job log cut at the brief's deadline; read_checks names its job and read_job reads its log]";

fn world(passes: u32, ci_contexts: Option<&'static [&'static [u8]]>) -> (World, engine::Assignment, Box<[u8]>) {
    let brief = temper_engine_domain_world::forge_routes::box_limits().brief;
    assert_eq!(brief.gather, Duration::from_secs(30));
    let world = World::configured_limits(
        true,
        true,
        false,
        passes,
        Gate::default(),
        None,
        StoryLimits { brief: Some(brief), ci_contexts, ..StoryLimits::default() },
    );
    let (world, _chat, producer, branch) = change_world_in(world, Change::default());
    (world, producer, branch)
}

fn assigned(world: &mut World) -> engine::Assignment {
    for _ in 0..STEPS {
        world.tick();
        if world.assigned.len() >= 3 {
            break;
        }
    }
    let assigned = world.assigned.get(2).expect("the slow section still assigned its run").clone();
    assert_eq!(live_task(world, assigned.task).tries, tasks::Tries::NONE);
    assigned
}

fn pull_words(assigned: &engine::Assignment) -> &str {
    let section = assigned
        .sections
        .iter()
        .find(|section| section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull))
        .expect("pull section");
    let engine::BriefBody::Text(words) = &section.body else { panic!("pull text: {section:?}") };
    std::str::from_utf8(words).expect("pull text is UTF-8")
}

fn repair_started(assigned: &engine::Assignment, branch: &[u8]) {
    assert!(matches!(assigned.workspace.repositories[0].start,
        engine::ForgeStart::Branch(ref source) if source.as_ref() == branch));
}

#[test]
fn a_resolution_with_slow_files_starts_with_deadline_notices() {
    let (mut world, producer, branch) = world(1000, None);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    world.fake_env.limits.limits.files = 12;
    let files = (0..12)
        .map(|number| raw::File {
            path: format!("file-{number:02}").into_bytes().into_boxed_slice(),
            content: b"branch change".as_slice().into(),
        })
        .collect();
    let head = fake::commit_tree(&mut world.fake, &world.fake_env.limits, 1, None, files, b"Many files")
        .expect("twelve changed files");
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: head, expected: Some(1) })),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    ));
    let base = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"file-00", b"base change", 1)
        .expect("another change conflicts with the first file");
    // The world's files translator does not byte-cut pages. Leave the first read
    // unanswered instead: a resolution never continues onto page 2.
    assert!(world.env.limits.forge.client.rows < 12);
    world.review_files_delay = Some(Duration::from_secs(30));
    world.forge_reads = Some(Vec::new());
    goal_landing::finish_run(
        &mut world,
        &producer,
        tasks::TaskResult::Change {
            title: jig_domain_tasks::ChangeTitle::new(b"Many files".as_slice().into()),
            connector: 0,
            kind: 2,
            resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
            words: b"pushed".as_slice().into(),
        },
    );
    let resolver = assigned(&mut world);
    assert!(matches!(resolver.workspace.repositories[0].start,
        engine::ForgeStart::Merge { branch: ref source, base: expected }
            if source.as_ref() == branch.as_ref() && expected == translate::commit(base)));
    let words = pull_words(&resolver);
    assert!(words.contains("[files cut at the brief's deadline: 0 of 12 files read; read_pull_files from page 1 lists the rest, and read_file reads a path at the head or the base]"), "{words}");
    assert!(words.contains("[comparison cut at the brief's deadline: not read; read it in your checkout]"), "{words}");
    let files_reads: Vec<_> = world
        .forge_reads
        .as_ref()
        .expect("recorded reads")
        .iter()
        .filter_map(|(_, read)| {
            if let skein_forge::api::Read::PullFiles { page, skip, .. } = read { Some((*page, *skip)) } else { None }
        })
        .collect();
    assert_eq!(files_reads, [(1, 0)], "only the first files page was started");
    world.close();
}

#[test]
fn a_ci_repair_with_slow_job_logs_starts_with_the_logs_it_read() {
    let (mut world, producer, branch) = world(0, Some(&[b"build", b"lint", b"test"]));
    world.job_read_delay = Some(Duration::from_secs(12));
    world.forge_reads = Some(Vec::new());
    start_failed_ci(&mut world, &producer, &branch, &[b"build", b"lint", b"test"]);
    let repair = assigned(&mut world);
    repair_started(&repair, &branch);
    let words = std::str::from_utf8(ci_words(&repair)).expect("CI text is UTF-8");
    let checks: Vec<_> = words.split("Failed check: ").skip(1).collect();
    assert_eq!(checks.len(), 3, "{words}");
    for (check, name) in checks.iter().zip(["build", "lint", "test"]) {
        assert!(check.starts_with(&format!("{name}\n")), "{check}");
        assert!(check.contains("Description:"), "{check}");
        assert!(check.contains("Link:"), "{check}");
    }
    assert!(checks[0].contains("Failed step: 1 Run build\nEnd of its log:\n"), "{words}");
    assert!(contains(checks[0].as_bytes(), BUILD_LOG_END), "{words}");
    assert!(!checks[0].contains(JOB_CUT), "{words}");
    for check in &checks[1..] {
        assert!(check.contains(JOB_CUT), "{check}");
        assert!(!check.contains("End of its log:"), "{check}");
    }
    assert_eq!(words.matches(JOB_CUT).count(), 2, "{words}");
    assert_eq!(
        world
            .forge_reads
            .as_ref()
            .expect("recorded reads")
            .iter()
            .filter(|(_, read)| matches!(read, skein_forge::api::Read::Job { .. }))
            .count(),
        2,
        "the second job was left reading, and the third was never started"
    );
    world.close();
}

#[test]
fn a_ci_repair_with_checks_past_the_cutoff_starts_on_its_whole_pull() {
    let (mut world, producer, branch) = world(0, None);
    world.repair_checks_delay = Some(Duration::from_secs(30));
    start_failed_ci(&mut world, &producer, &branch, &[b"build"]);
    let repair = assigned(&mut world);
    repair_started(&repair, &branch);
    assert_eq!(
        std::str::from_utf8(ci_words(&repair)).expect("CI text is UTF-8"),
        "[the failed checks could not be read: not read by the brief's deadline]\n"
    );
    let words = pull_words(&repair);
    assert!(words.contains("Pull request: #"), "{words}");
    assert!(words.contains("Title: Small fix"), "{words}");
    assert!(words.ends_with("Files and diff:\nFile: file\nBefore:\n\nAfter:\nfirst\nAgainst base:\nfile\n"), "{words}");
    assert!(!words.contains("cut at the brief's deadline"), "{words}");
    assert!(!words.contains("could not be read"), "{words}");
    world.close();
}
