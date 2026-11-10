//! The requester reads all the evidence of a CSV change's repair and landing.
use super::*;

const CSV: &[u8] = b"fn main() { println!(\"fruit,count\\napples,3\"); }\n";
fn finish(world: &mut World, run: &engine::Assignment, result: tasks::TaskResult) {
    let event = world.protocol.answer(
        Token::new(7),
        run.task,
        run.attempt,
        5,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
}
fn change_result() -> tasks::TaskResult {
    tasks::TaskResult::Change {
        title: jig_domain_tasks::ChangeTitle::new(b"Change".as_slice().into()),
        connector: 0,
        kind: 2,
        resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
        words: b"pushed".as_slice().into(),
    }
}
fn next(world: &mut World, index: usize) -> engine::Assignment {
    for _ in 0..200 {
        world.tick();
        if let Some(run) = world.assigned.get(index) {
            return run.clone();
        }
    }
    panic!("delegate {index} was not assigned");
}
#[test]
fn the_csv_changes_requester_reads_the_failed_context_repair_and_approved_review_whole() {
    let (mut world, chat, producer, branch) = repair_world();
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("change") };
    // Park the requester so its next assignment must carry the whole result.
    let event =
        world.protocol.answer(Token::new(7), chat.task, chat.attempt, 5, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    assert!(matches!(
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
        raw::Answer::Branch(raw::Created::Created)
    ));
    let h1 =
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"src/main.rs", b"fn main() {}\n", 1)
            .expect("producer pushed its first source");
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            description: Box::new([]),
            link: Box::new([]),
            commit: h1,
            context: b"/ changes (pull_request)".as_slice().into(),
            state: raw::Check::Failed,
        }),
    );
    finish(&mut world, &producer, change_result());
    let repair = next(&mut world, 2);
    assert!(contains(ci_words(&repair), b"Failed check: / changes (pull_request)\n"));
    let words = &live_task(&world, repair.task).spec.words;
    let cut = &words[..words.len().min(128)];
    let head = forge_top::commit_text(&translate::commit(h1));
    let expected = format!(
        "Repair CI at {}: / changes (pull_request)\n",
        core::str::from_utf8(&head[..12]).expect("hexadecimal head")
    );
    assert!(cut.starts_with(expected.as_bytes()), "client's cut repair words: {cut:?}");
    let h2 = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"src/main.rs", CSV, 1)
        .expect("repair pushed the CSV source");
    finish(&mut world, &repair, change_result());
    let review = next(&mut world, 3);
    finish(
        &mut world,
        &review,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    let requester = next(&mut world, 4);
    assert_eq!(requester.task, chat.task);
    assert_csv_landing(&world, &requester, change, repair.task, h2);
    world.close();
}

fn assert_csv_landing(world: &World, requester: &engine::Assignment, change: u64, repair_task: u64, h2: u64) {
    let Some(Record::Forge { row, .. }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("change row")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    let temper_engine_domain_forge_change::State::Landed { merge, .. } = row.change.state else { panic!("landed") };
    let merge = String::from_utf8(forge_top::commit_text(&merge).into_vec()).expect("hexadecimal merge id");
    let mut head = [0; 32];
    head[..20].copy_from_slice(&fake::object_bytes(h2));
    let head = String::from_utf8(forge_top::commit_text(&head).into_vec()).expect("hexadecimal head id");
    let result = requester
        .messages
        .iter()
        .find(|word| {
            word.from == tasks::Party::Task(change)
                && matches!(word.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))
        })
        .expect("requester's result");
    let words = core::str::from_utf8(&result.words).expect("UTF-8 result");
    let lines: Vec<_> = words.lines().collect();
    assert_eq!(lines.len(), 9, "{words}");
    assert_eq!(lines[0], merge);
    assert!(lines[1].starts_with("pull request #"), "{words}");
    assert_eq!(
        lines[2],
        format!("Landed: pull request #{} merged into main as {}.", row.pull.expect("landed pull"), &merge[..12])
    );
    assert_eq!(lines[3], "repairs: 1: CI failed: / changes (pull_request)");
    assert_eq!(lines[4], format!("successful repair task {repair_task}: pushed"));
    assert_eq!(lines[5], "Producer report (historical; before repairs):");
    assert_eq!(lines[6], "pushed");
    assert_eq!(lines[7], format!("checks at {head}: build passed"));
    assert_eq!(lines[8], format!("reviews: approved at {head}"));
    assert!(!words.contains("bytes cut"), "the requester reads it whole");
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one landing scene checks retained reports, evidence and requester messages at each bound"
)]
fn a_landed_change_carries_its_producers_report() {
    const REPORT: &[u8] = b"Done: edited README.\n## Left\n- none";
    // The three landing rows measured 156 ms CPU together, including durable restart checks.
    const STOCK_RESULT: u32 = 10_660; // temper/docs/design/protocol/configuration/stock.md.
    for (report_bytes, result_bytes, cut) in
        [(REPORT.len(), 512, false), (3_585, STOCK_RESULT, false), (10_660, STOCK_RESULT, true)]
    {
        let mut report = REPORT.to_vec();
        report.resize(report_bytes, b'r');
        let mut world = World::configured_limits(
            true,
            true,
            false,
            1000,
            Gate::default(),
            None,
            StoryLimits { result_bytes: Some(result_bytes), ..StoryLimits::default() },
        );
        world.adopt();
        world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
        let ask = adoption_chat(&world);
        let chat = assigned_chat(&mut world, 92, ask);
        let engine::CallAnswer::Delegated(made) = change_tool::change(&mut world, &chat, 93, None, b"Edit README")
        else {
            panic!("change delegated")
        };
        let change = made[0].task;
        assert_eq!(
            live_task(&world, change).contract,
            tasks::Contract::Change { connector: 0, kind: 1, words: result_bytes }
        );
        let producer = next(&mut world, 1);
        let branch = producer.workspace.repositories[0].push.clone().expect("change branch");
        let tasks::Party::Task(parent) = live_task(&world, producer.task).requester else {
            panic!("producer belongs to its change")
        };
        assert_eq!(parent, change);
        assert_eq!(
            world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 })),
            raw::Answer::Branch(raw::Created::Created)
        );
        let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"README", b"edited", 1)
            .expect("producer pushed README");
        let event =
            world.protocol.answer(Token::new(7), chat.task, chat.attempt, 5, tasks::End::Parked, engine::Parts::none());
        world.send(event);
        let tasks::TaskResult::Change { title, connector, kind, resource, .. } = change_result() else {
            panic!("producer result")
        };
        finish(
            &mut world,
            &producer,
            tasks::TaskResult::Change { title, connector, kind, resource, words: report.clone().into() },
        );
        let requester = next(&mut world, 2);
        assert_eq!(requester.task, chat.task);
        let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
            panic!("durable change")
        };
        let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
        assert_eq!(row.report.as_ref(), report.as_slice());
        let temper_engine_domain_forge_change::State::Landed { merge, .. } = row.change.state else { panic!("landed") };
        let merge = String::from_utf8(forge_top::commit_text(&merge).into_vec()).expect("hexadecimal merge");
        let head =
            String::from_utf8(forge_top::commit_text(&translate::commit(head)).into_vec()).expect("hexadecimal head");
        let message = requester
            .messages
            .iter()
            .find(|message| {
                message.from == tasks::Party::Task(change)
                    && matches!(message.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))
            })
            .expect("requester's change result");
        let words = core::str::from_utf8(&message.words).expect("UTF-8 result");
        let (identity, _) = words.split_once("\nReport:\n").expect("report label");
        let lines: Vec<_> = identity.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], merge);
        assert!(lines[1].starts_with("pull request #"));
        assert_eq!(
            lines[2],
            format!("Landed: pull request #{} merged into main as {}.", row.pull.expect("landed pull"), &merge[..12])
        );
        let whole = format!(
            "{identity}\nReport:\n{}\nchecks at {head}: build passed\nrepairs: 0\nreviews: none",
            core::str::from_utf8(&report).expect("report text"),
        );
        assert_eq!(words.contains("\n["), cut, "report {report_bytes}");
        if let Some((prefix, marker)) = words.rsplit_once("\n[") {
            let cut: usize = marker.strip_suffix(" bytes cut]\n").expect("cut marker").parse().expect("cut count");
            assert!(whole.starts_with(prefix), "report {report_bytes}, result {result_bytes}");
            assert_eq!(cut, whole.len() - prefix.len());
            assert!(cut > 0);
        } else {
            assert_eq!(words, whole, "report {report_bytes}, result {result_bytes}");
        }
        let rendered = engine::inbox_words(chat.task, message);
        let sender = engine::inbox_sender(message);
        assert!(
            rendered.len() + sender.len() <= usize::try_from(result_bytes).expect("result bound"),
            "report {report_bytes}: {} rendered bytes and {} sender bytes exceed {result_bytes}",
            rendered.len(),
            sender.len()
        );
        world.close();
    }
}

const CSV_REPORT: &[u8] =
    b"Pushed: exports the widget table as CSV. Not included: changes/2.md; it needs a repair then.";
const CSV_REPAIR: &[u8] = b"Pushed: Adds changes/2.md with the correct opened line.";
const JSON_REPORT: &[u8] = b"Pushed: exports JSON. Its PR number (3) and opened timestamp are guesses.";
const JSON_REPAIR: &[u8] = b"Pushed: Corrects changes/3.md with PR 3 and its correct opened line.";

fn finish_report(world: &mut World, run: &engine::Assignment, words: &[u8]) {
    let tasks::TaskResult::Change { title, connector, kind, resource, .. } = change_result() else {
        panic!("change result")
    };
    finish(world, run, tasks::TaskResult::Change { title, connector, kind, resource, words: words.into() });
}

fn repair_world() -> (World, engine::Assignment, engine::Assignment, Box<[u8]>) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate { agent: true, ..Gate::default() },
        None,
        StoryLimits { result_bytes: Some(512), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    let ask = adoption_chat(&world);
    let chat = assigned_chat(&mut world, 92, ask);
    let answer = change_tool::change(&mut world, &chat, 93, None, b"Export the widget table");
    assert!(matches!(answer, engine::CallAnswer::Delegated(_)));
    let producer = next(&mut world, 1);
    let branch = producer.workspace.repositories[0].push.clone().expect("change branch");
    (world, chat, producer, branch)
}

fn push_export(world: &mut World, branch: &[u8], path: &[u8], producer_words: &[u8]) -> u64 {
    let source = if producer_words == JSON_REPORT { b"exports JSON".as_slice() } else { CSV };
    let mut head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"src/main.rs", source, 1)
        .expect("producer exports");
    if producer_words == JSON_REPORT {
        head = fake::advance(
            &mut world.fake,
            &world.fake_env,
            b"org/repo",
            branch,
            path,
            b"PR: guessed\nOpened: guessed\n",
            1,
        )
        .expect("producer guesses note values");
    }
    head
}

fn opened_note(world: &mut World, number: u64) -> Box<[u8]> {
    let raw::Answer::Item { item, .. } =
        world.external(1, raw::Op::Read(raw::Read::Item { number, after: 0, since: None }))
    else {
        panic!("pull item")
    };
    format!("PR: {}\nOpened: {}ns\n", item.number, item.created.as_nanos()).into_bytes().into()
}

fn repaired_story(path: &[u8], producer_words: &[u8], repair_words: &[u8]) {
    let (mut world, chat, producer, branch) = repair_world();
    let tasks::Party::Task(change) = live_task(&world, producer.task).requester else { panic!("change") };
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let number = if producer_words == JSON_REPORT { 3 } else { 2 };
    for n in 1..number {
        assert_eq!(
            world.external(
                1,
                raw::Op::Write(raw::Write::CreateIssue {
                    title: b"Existing work".as_slice().into(),
                    body: Box::new([]),
                    labels: Box::new([]),
                })
            ),
            raw::Answer::Created(n)
        );
    }
    let head = push_export(&mut world, &branch, path, producer_words);
    world.external(
        1,
        raw::Op::Write(raw::Write::Status {
            description: Box::new([]),
            link: Box::new([]),
            commit: head,
            context: b"/ changes (pull_request)".as_slice().into(),
            state: raw::Check::Failed,
        }),
    );
    let parked =
        world.protocol.answer(Token::new(7), chat.task, chat.attempt, 5, tasks::End::Parked, engine::Parts::none());
    world.send(parked);
    finish_report(&mut world, &producer, producer_words);
    let repair = next(&mut world, 2);
    assert!(contains(ci_words(&repair), b"Failed check: / changes (pull_request)"));
    let note = opened_note(&mut world, number);
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, path, &note, 1).expect("repair note");
    finish_report(&mut world, &repair, repair_words);
    finish_repaired_story(world, &chat, change, path, producer_words, repair_words, note);
}

fn finish_repaired_story(
    mut world: World,
    chat: &engine::Assignment,
    change: u64,
    path: &[u8],
    producer_words: &[u8],
    repair_words: &[u8],
    note: Box<[u8]>,
) {
    let review = next(&mut world, 3);
    world.drain_cut();
    world.restart_same();
    world.until(Until::Ready);
    finish(
        &mut world,
        &review,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    let requester = next(&mut world, 4);
    assert_eq!(requester.task, chat.task);
    let message = requester
        .messages
        .iter()
        .find(|message| {
            message.from == tasks::Party::Task(change)
                && matches!(message.kind, tasks::MessageKind::Result(tasks::ResultKind::Change { .. }))
        })
        .expect("landed result");
    assert!(
        contains(&message.words, repair_words),
        "successful repair missing: {}",
        core::str::from_utf8(&message.words).expect("UTF-8 result")
    );
    assert!(contains(&message.words, producer_words), "export description retained");
    assert!(contains(&message.words, b"historical"), "producer caveats are historical");
    let raw::Answer::Commit(main) =
        world.external(1, raw::Op::Read(raw::Read::Branch { branch: b"main".as_slice().into() }))
    else {
        panic!("main")
    };
    assert_eq!(
        world.external(1, raw::Op::Read(raw::Read::File { commit: main, path: path.into() })),
        raw::Answer::File(note)
    );
    world.close();
}

#[test]
fn a_landed_change_carries_its_successful_repairs_report() {
    repaired_story(b"changes/2.md", CSV_REPORT, CSV_REPAIR);
    repaired_story(b"changes/3.md", JSON_REPORT, JSON_REPAIR);
}
