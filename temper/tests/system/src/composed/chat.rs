//! Stock chat inspection and change routing across Smith and the engine.
//! Contract: temper/docs/design/domain/agent.md, 4.3.1 and 8;
//! temper/docs/design/domain/forge.md, 8.1.
use super::{World, agreement, call, engine, fake, parties, tasks, text_script};
use skein_fake_llm_domain::api::{Part, Script, Turn};

const REQUEST: &[u8] = b"Add the line \"Tier walk: first.\" to the end of README.md in widget.";
const PROHIBITION: &str =
    "Repository changes must go through change; never implement them by writing your chat workspace.";
const COMPLETION: &str = "After calling change, say only that the work started; never say added, fixed or equivalent until the delegate's Landed result names its pull request and merge commit.";

fn chat(turns: Vec<Turn>) -> World {
    let script = Script { cue: b"Help the person with their work.".as_slice().into(), turns: turns.into_boxed_slice() };
    let mut world = World::configured_with(923, script.clone(), true, agreement::config);
    world.scripts.push_back(script);
    world.party = parties::Party::new(
        1,
        parties::Role::Owner,
        400,
        Box::new([
            parties::Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() },
            parties::Act::Request { key: [41; 16], ask: parties::Ask::Chat { words: REQUEST.into() } },
        ]),
    );
    world
}

fn finish_chat(world: &mut World) {
    for _ in 0..10000 {
        world.tick();
        if world.observed.gone == 1 {
            assert!(matches!(world.agent().answer(), smith_domain_run::Answer::Parked { .. }));
            return;
        }
    }
    panic!("chat did not park: {:?}", world.observed);
}

fn assert_prompt(world: &World) {
    let prompt = &world.agent().prompts()[0];
    let system = String::from_utf8_lossy(&prompt.system);
    assert!(system.contains(PROHIBITION), "stock prompt must prohibit workspace-only repository edits");
    assert!(system.contains(COMPLETION), "stock prompt must reserve completion for a reported landing");
    assert!(prompt.messages.iter().flat_map(|message| &message.parts).any(|part| {
        matches!(part, Part::Text { text } if text.windows(REQUEST.len()).any(|bytes| bytes == REQUEST))
    }));
    assert!(!world.assignment().run.policy.modify, "the stock chat inspects its workspace");
}

fn assert_no_landing(world: &World) {
    assert!(!world.forge.observations().iter().any(|observation| matches!(
        observation,
        skein_fake_forge_domain::Observation::Opened { kind: fake::Kind::Pull, .. }
            | skein_fake_forge_domain::Observation::Merged { .. }
    )));
}

#[test]
fn a_stock_chat_cannot_write_its_workspace_in_place_of_a_change() {
    refused_workspace_change(b"write", br##"{"path":"README.md","content":"# widget\n\nTier walk: first.\n"}"##);
}

#[test]
fn a_stock_chat_cannot_edit_its_workspace_in_place_of_a_change() {
    refused_workspace_change(
        b"edit",
        br#"{"path":"README.md","edits":[{"old":"The answer is 42.\n","new":"The answer is 42.\nTier walk: first.\n"}]}"#,
    );
}

fn refused_workspace_change(tool: &[u8], arguments: &[u8]) {
    // The system world is the lowest tier joining Smith tools, Open replies and durable change admission.
    let mut turns =
        vec![call(b"list", br#"{"path":"."}"#), call(b"read", br#"{"path":"README.md"}"#), call(tool, arguments)];
    // Scripted prose does not prove completion; the rail must reject the preceding tool call.
    turns.extend(text_script(b"Added \"Tier walk: first.\" to the end of README.md.").turns);
    let mut world = chat(turns);
    let head = world.forge.branch(b"org/repo", b"main").expect("main exists");
    finish_chat(&mut world);
    assert!(
        world.agent().prompts().iter().all(|prompt| prompt
            .tools
            .iter()
            .all(|tool| { ![b"write".as_slice(), b"edit", b"remove"].contains(&tool.name.as_ref()) })),
        "modifying tools must not be offered"
    );
    assert_refused_tool(&world, tool);
    assert!(world.agent().disk().files().values().any(|bytes| *bytes == b"The answer is 42.\n"));
    assert!(
        !world
            .agent()
            .disk()
            .files()
            .values()
            .any(|bytes| bytes.windows(b"Tier walk: first.".len()).any(|line| line == b"Tier walk: first."))
    );
    assert!(world.agent().host_submissions().is_empty(), "workspace tools made no change call");
    assert_eq!(world.forge.branch(b"org/repo", b"main"), Some(head));
    assert_no_landing(&world);
    assert_prompt(&world);
}

fn assert_refused_tool(world: &World, tool: &[u8]) {
    let prompt = world.agent().prompts().last().expect("feedback reaches the next turn");
    let parts = prompt.messages.iter().flat_map(|message| &message.parts).collect::<Vec<_>>();
    let called = parts
        .iter()
        .find_map(|part| match part {
            Part::ToolCall { id, name, .. } if name.as_ref() == tool => Some(id),
            Part::Text { .. } | Part::ToolCall { .. } | Part::ToolOutput { .. } | Part::Opaque { .. } => None,
        })
        .expect("scripted modifying call");
    assert!(
        parts.iter().any(|part| matches!(part,
            Part::ToolOutput { id, is_error: true, .. } if id == called
        )),
        "the attempted workspace modification must fail"
    );
}

fn main_commit(world: &World) -> String {
    let head = world.forge.branch(b"org/repo", b"main").expect("main exists");
    let mut hex = String::new();
    for byte in skein_fake_forge_domain::object_bytes(head) {
        hex.push(char::from(b"0123456789abcdef"[usize::from(byte >> 4)]));
        hex.push(char::from(b"0123456789abcdef"[usize::from(byte & 15)]));
    }
    hex
}

fn started_change() -> (World, String) {
    let mut world = chat(Vec::new());
    let hex = main_commit(&world);
    let words = format!("{} Read README.md at {hex}: The answer is 42.\\n", String::from_utf8_lossy(REQUEST));
    let turns = vec![
        call(b"read_branch", br#"{"branch":"main"}"#),
        call(b"read_file", format!(r#"{{"head":"{hex}","path":"README.md","max_bytes":4096}}"#).as_bytes()),
        call(b"change", format!(r#"{{"words":"{}"}}"#, words.replace('"', "\\\"")).as_bytes()),
        text_script(b"I started the change.").turns[0].clone(),
    ];
    world.scripts = super::VecDeque::from([Script {
        cue: b"Help the person with their work.".as_slice().into(),
        turns: turns.into_boxed_slice(),
    }]);
    (world, words.replace("\\n", "\n"))
}

#[test]
fn a_stock_chat_reads_then_starts_a_durable_change_without_a_workspace_edit() {
    let (mut world, words) = started_change();
    let head = world.forge.branch(b"org/repo", b"main").expect("main exists");
    finish_chat(&mut world);
    assert_prompt(&world);
    let started = b"I started the change.";
    assert!(
        world.store.rows.values().any(|row| matches!(row,
            engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn)))
                if turn.task == 1 && turn.transcript.windows(started.len()).any(|bytes| bytes == started)
        )),
        "the started-only reply is durable"
    );
    assert!(world.agent().disk().files().values().any(|bytes| *bytes == b"The answer is 42.\n"));
    assert_eq!(
        world.agent().host_submissions().iter().map(|call| call.tool.as_ref()).collect::<Vec<_>>(),
        [b"change".as_slice()]
    );
    let outputs = world
        .agent()
        .prompts()
        .iter()
        .flat_map(|prompt| &prompt.messages)
        .flat_map(|message| &message.parts)
        .filter_map(|part| match part {
            Part::ToolOutput { output, is_error, .. } => {
                assert!(!is_error, "{}", String::from_utf8_lossy(output));
                Some(output.as_ref())
            }
            Part::Text { .. } | Part::ToolCall { .. } | Part::Opaque { .. } => None,
        })
        .collect::<Vec<_>>();
    assert!(
        outputs
            .iter()
            .any(|output| output.windows(b"The answer is 42.".len()).any(|bytes| bytes == b"The answer is 42."))
    );
    assert!(
        world.store.rows.values().any(|row| matches!(row,
            engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))
                if row.spec.words.as_ref() == words.as_bytes() && row.requester == tasks::Party::Task(1)
                    && matches!(row.executor, tasks::Executor::Procedure { code: 2, .. })
        )),
        "the change delegate and its read evidence are durable"
    );
    assert_eq!(world.forge.branch(b"org/repo", b"main"), Some(head));
    assert_no_landing(&world);
}

#[test]
fn a_stock_chats_delegate_reports_the_landed_pull_and_merge_without_invented_checks() {
    let (mut world, _) = started_change();
    world.next_script(Script {
        cue: b"Read only what the change needs".as_slice().into(),
        turns: Box::new([
            call(b"read", br#"{"path":"README.md"}"#),
            call(b"write", br#"{"path":"README.md","content":"The answer is 42.\nTier walk: first.\n"}"#),
            call(b"finish", br#"{"title":"Add tier walk line","body":"Append the requested line."}"#),
        ]),
    });
    let report = landing_report(&mut world);
    let head = world.forge.branch(b"org/repo", b"main").expect("main exists");
    let hex = main_commit(&world);
    let line = format!("Landed: pull request #1 merged into main as {}.", &hex[..12]);
    assert!(report.contains(&line), "{report}");
    assert!(report.contains(": build passed\n"), "the fake CI's build is reported: {report}");
    assert!(report.ends_with("reviews: none"), "no reviewer result is invented: {report}");
    assert_eq!(
        world.forge.api(
            b"org/repo",
            1,
            fake::Op::Read(fake::Read::File { commit: head, path: b"README.md".as_slice().into() })
        ),
        Ok(fake::Answer::File(b"The answer is 42.\nTier walk: first.\n".as_slice().into()))
    );
}

fn landing_report(world: &mut World) -> String {
    for _ in 0..10000 {
        if world.process.answered.is_some()
            && world.worker_events.is_empty()
            && world.root_events.is_empty()
            && !world.committing()
        {
            world.env.now = world.env.now.saturating_add(super::Duration::from_secs(1));
        }
        world.tick();
        let key = engine::Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(2)));
        if let Some(engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            world.store.rows.get(&key)
            && let tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Change { words, .. })) = &row.phase
        {
            return String::from_utf8(words.to_vec()).expect("landing result words");
        }
    }
    panic!("delegate did not report a landing: {:?}", world.observed);
}
