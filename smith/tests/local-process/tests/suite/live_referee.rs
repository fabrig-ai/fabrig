//! The live policy consumes accepted outside observations and rejects edited records.
use skein_io::kernel::Exit;
use skein_world::domain::Verdict;
use smith_events::Event;
use smith_local_process_world::{
    Files, Placement, World,
    referee::{
        CheckoutRead,
        live::{self, Outcome, Seen},
    },
};

fn capture(world: &mut World) -> Seen {
    let head = world.head();
    let committed = world.commit_files();
    let mut working = skein_fake_checkout::git::Tree::new();
    for path in committed.keys() {
        let path_in_root = [b"repo/".as_slice(), path].concat();
        if let Some(bytes) = world.file(&path_in_root) {
            working.insert(path.clone(), bytes);
        }
    }
    Seen {
        exit: world.exit(),
        errors: world.errors().to_vec(),
        shown: world.shown().to_vec(),
        records: world.records(),
        head,
        message: world.commit_message().to_vec(),
        committed,
        working,
    }
}

fn rejects(seen: Seen, before: Option<Vec<u8>>, files: Vec<(Vec<u8>, Vec<u8>)>, why: &str) {
    let review = live::review(seen, Outcome { before, files });
    let Verdict::Failed(failure) = review.verdict() else { panic!("live mutation must fail: {why}") };
    assert_eq!(failure.why, why);
}

fn without_finish_start(seen: &Seen) -> Seen {
    let mut missing = seen.clone();
    let before = missing.records.len();
    missing.records.retain(|record| {
        !matches!(&record.event,
        Event::ToolStarted(tool) if tool.tool.as_ref() == b"finish")
    });
    const FINISH_STARTS: usize = 1; // The accepted script calls finish once.
    assert_eq!(before - missing.records.len(), FINISH_STARTS);
    missing
}

fn refused_finish(seen: &Seen) -> Seen {
    let mut refused = seen.clone();
    let mut changed = 0;
    for record in &mut refused.records {
        if let Event::ToolCompleted(tool) = &mut record.event
            && tool.tool.as_ref() == b"finish"
        {
            tool.verdict = smith_events::Verdict::Invalid;
            changed += 1;
        }
    }
    const FINISH_TERMINALS: usize = 1; // The accepted script completes finish once.
    assert_eq!(changed, FINISH_TERMINALS);
    refused
}

fn traced_change(seed: u64, files: &Files) -> World {
    let mut world = World::changed(seed, files, Placement::InProcess);
    world.scenario.launch.traced = true;
    world
}

fn edit_and_deliver() -> Box<[skein_fake_llm_domain::api::Script]> {
    use skein_fake_llm_domain::api::{Line, Script};
    let calls = [
        ("edit", r#"{"path":"original.txt","edits":[{"old":"original","new":"edited"}]}"#),
        ("write", r#"{"path":"result.txt","content":"new\n"}"#),
        ("finish", r#"{"form":"change","fields":{"title":"Updated result","body":"Created the result file"}}"#),
    ];
    Box::new([Script {
        cue: b"@local-shell".as_slice().into(),
        turns: calls
            .into_iter()
            .map(|(name, arguments)| {
                smith_agent_world::fixture::calls(
                    Box::new([Line::Call { name: name.as_bytes().into(), arguments: arguments.as_bytes().into() }]),
                    1,
                )
            })
            .collect(),
    }])
}

#[test]
fn the_live_outcome_referee_accepts_settled_report_and_change() {
    let report = Files::new();
    let change = Files::new();
    let mut changed = traced_change(36, &change);
    changed.scenario.scripts = Some(edit_and_deliver());
    // Lay the command artifacts; this test judges the settled checkout, not the commands.
    let staged = [(b"command.txt".as_slice(), b"ran\n".as_slice()), (b"checks-ran.txt", b"checked\n")];
    for (path, content) in staged {
        changed.fixture_file(&[b"repo/".as_slice(), path].concat(), content);
    }
    let expected = staged
        .into_iter()
        .chain([(b"original.txt".as_slice(), b"edited\n".as_slice()), (b"result.txt", b"new\n")])
        .map(|(path, bytes)| (path.to_vec(), bytes.to_vec()))
        .collect();
    let before = changed.head().expect("initial head");
    for (mut world, outcome) in [
        (World::traced(35, &report, Placement::InProcess, b"First answer"), Outcome { before: None, files: vec![] }),
        (changed, Outcome { before: Some(before), files: expected }),
    ] {
        let seed = world.scenario.launch.seed;
        world.settle();
        live::review(capture(&mut world), outcome).assert_passed(seed);
    }
}

#[test]
fn the_live_report_policy_rejects_mutated_accepted_observations() {
    let files = Files::new();
    let mut world = World::traced(86, &files, Placement::InProcess, b"First answer");
    world.settle();
    let seen = capture(&mut world);
    live::review(seen.clone(), Outcome { before: None, files: Default::default() }).assert_passed(86);
    let mut failed = seen.clone();
    failed.exit = Some(Exit::Code(1));
    let mut diagnostic = seen.clone();
    diagnostic.errors = b"unexpected operator message\n".to_vec();
    for (mutation, why) in [
        (failed, "the live outcome exits successfully"),
        (diagnostic, "only expected live operator diagnostics"),
        (without_finish_start(&seen), "finish started"),
        (refused_finish(&seen), "finish received its accepted or delivered terminal"),
    ] {
        rejects(mutation, None, Default::default(), why);
    }
}

#[test]
fn the_live_change_policy_rejects_mutated_delivery_observations() {
    let files = Files::new();
    let mut world = traced_change(87, &files);
    let before = world.head();
    world.settle();
    let seen = capture(&mut world);
    let expected =
        vec![(b"original.txt".to_vec(), b"original\n".to_vec()), (b"result.txt".to_vec(), b"new\n".to_vec())];
    live::review(seen.clone(), Outcome { before: before.clone(), files: expected.clone() }).assert_passed(87);
    let mut unchanged = seen.clone();
    unchanged.head = before.clone();
    let mut no_identity = seen.clone();
    no_identity.message = b"Updated result\n\nCreated the result file".to_vec();
    let mut no_commit_file = seen.clone();
    no_commit_file.committed.clear();
    let mut no_working_file = seen.clone();
    no_working_file.working.clear();
    let mut no_presentation = seen.clone();
    no_presentation.shown.clear();
    let mut not_delivered = seen.clone();
    let mut changed = 0;
    for record in &mut not_delivered.records {
        if let Event::ToolCompleted(tool) = &mut record.event
            && tool.tool.as_ref() == b"finish"
        {
            tool.delivery = Some(smith_events::Delivered::Refused);
            changed += 1;
        }
    }
    const DELIVERED_FINISHES: usize = 1; // The accepted script delivers one finish.
    assert_eq!(changed, DELIVERED_FINISHES);
    for (mutation, why) in [
        (unchanged, "a new live commit exists"),
        (no_identity, "the live commit carries its delivery identity"),
        (no_commit_file, "the committed file has its prescribed content"),
        (no_working_file, "the working file has its prescribed content"),
        (no_presentation, "the person saw delivery"),
        (not_delivered, "finish received its accepted or delivered terminal"),
    ] {
        rejects(mutation, before.clone(), expected.clone(), why);
    }
}
