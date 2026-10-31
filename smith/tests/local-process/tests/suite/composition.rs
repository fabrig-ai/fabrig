//! Behaviour at the local shell, agent and fake-machine composition.
use skein_fake_llm_domain::api;
use skein_io::kernel;
use smith_events::Event;
use smith_local_process_world::{
    Files, Placement, World,
    referee::{Ending, review},
};

fn write_report(name: &str) -> Box<[api::Script]> {
    Box::new([api::Script {
        cue: b"@local-shell".as_slice().into(),
        turns: [
            ("write", format!(r#"{{"path":"/{name}/notes.txt","content":"noted\n"}}"#)),
            ("finish", r#"{"form":"report","text":"Wrote it","fields":{}}"#.into()),
        ]
        .into_iter()
        .map(|(name, arguments)| api::Turn {
            lines: Box::new([api::Line::Call {
                name: name.as_bytes().into(),
                arguments: arguments.into_bytes().into(),
            }]),
            finish: api::Finish::ToolCalls,
            tokens: 1,
        })
        .collect(),
    }])
}

fn directory(seed: u64, files: &Files, placement: Placement, name: &str, writable: bool) -> World {
    let mut world = World::plain(seed, files, placement, name.as_bytes(), writable);
    world.scenario.scripts = Some(write_report(name));
    world.scenario.launch.traced = true;
    world
}

#[test]
fn a_named_writable_root_receives_the_agents_write() {
    for placement in [Placement::Spawned, Placement::InProcess] {
        let files = Files::new();
        let mut world = directory(81, &files, placement, "plan", true);
        world.settle();
        review(&world.seen(), &world, Ending::Report(b"Wrote it".to_vec())).assert_passed(81);
        assert_eq!(world.exit(), Some(kernel::Exit::Code(0)));
        assert_eq!(world.file(b"extra/notes.txt").as_deref(), Some(b"noted\n".as_slice()));
        assert_eq!(world.file(b"repo/plan/notes.txt"), None);
        assert_eq!(world.file(b"plan/notes.txt"), None, "the root name is not a relative directory");
    }
}

#[test]
fn a_named_read_only_root_refuses_the_agents_write() {
    for placement in [Placement::Spawned, Placement::InProcess] {
        let files = Files::new();
        let mut world = directory(82, &files, placement, "kit", false);
        world.settle();
        review(&world.seen(), &world, Ending::Report(b"Wrote it".to_vec())).assert_passed(82);
        let records = match placement {
            Placement::Spawned => world.agent_records(),
            Placement::InProcess => world.records(),
        };
        let verdicts: Vec<_> = records
            .iter()
            .filter_map(|record| match &record.event {
                Event::ToolCompleted(tool) if tool.tool.as_ref() == b"write" => Some(&tool.verdict),
                _ => None,
            })
            .collect();
        assert_eq!(verdicts, [&smith_events::Verdict::ReadOnly], "{placement:?}");
        assert_eq!(world.file(b"extra/notes.txt"), None);
    }
}

#[test]
fn a_plain_snapshot_at_and_past_its_bound_answers() {
    // smith-local-service's process::plain_limits().file_bytes.
    const SNAPSHOT_FILE_BYTES: usize = 65_536;
    for bytes in [SNAPSHOT_FILE_BYTES, SNAPSHOT_FILE_BYTES + 1] {
        for placement in [Placement::Spawned, Placement::InProcess] {
            let files = Files::new();
            let mut world = directory(83, &files, placement, "plan", true);
            world.fixture_file(b"extra/large", &vec![b'x'; bytes]);
            world.settle();
            review(&world.seen(), &world, Ending::Report(b"Wrote it".to_vec())).assert_passed(83);
            assert_eq!(world.exit(), Some(kernel::Exit::Code(0)));
            assert_eq!(world.file(b"extra/notes.txt").as_deref(), Some(b"noted\n".as_slice()));
            assert!(!String::from_utf8_lossy(world.errors()).contains("could not be saved"));
        }
    }
}

#[test]
fn an_accepted_effort_change_reaches_the_next_response() {
    use smith_local_process_world::terminal::Action;
    let files = Files::new();
    let mut world = World::new(84, &files, Placement::InProcess, b"Second answer");
    world.interactive();
    world.scenario.launch.choices = true;
    world.scenario.launch.traced = true;
    world.scenario.commands = vec![
        Action::Send(b"/effort low\nhello\n".as_slice().into()),
        Action::Wait(b"First answer".as_slice().into()),
        Action::Send(b"/effort high\nnext\n".as_slice().into()),
        Action::Eof,
    ];
    world.settle();
    review(&world.seen(), &world, Ending::Report(b"Second answer".to_vec())).assert_passed(84);
    let queries = world.queries();
    // The script has one finish response for each of the two activations.
    const RESPONSES: usize = 2;
    assert_eq!(queries.len(), RESPONSES);
    let efforts: Vec<_> = world
        .records()
        .into_iter()
        .filter_map(|record| match record.event {
            Event::ResponseStarted(response) => Some(response.model.effort),
            _ => None,
        })
        .collect();
    assert_eq!(efforts, [Some(b"low".as_slice().into()), Some(b"high".as_slice().into())]);
    assert_eq!(world.exit(), Some(kernel::Exit::Code(0)));
}

#[test]
fn an_older_store_refuses_with_operator_words_and_unchanged_files() {
    const OPERATOR_LINE: &str = concat!(
        "smith: Chat `main` was kept by an older build of smith, from before chats recorded their layouts, ",
        "and this start has changed nothing in its directory. Before the first release a chat is not translated, ",
        "so every start refuses this one until it is moved. To start afresh, stop each smith using it, ",
        "move its directory under smith's state directory aside, whole, and start again: nothing is deleted, ",
        "and the next start makes a new, empty chat, without this chat's history or its deliveries' answers."
    );
    for placement in [Placement::Spawned, Placement::InProcess] {
        let files = Files::new();
        let mut world = World::new(85, &files, placement, b"unused");
        let state = smith_local_domain::ChatState {
            identity: smith_domain::run::Identity::new([42; 16]),
            activation: 9,
            next_message: 1,
            read: None,
        };
        for name in [b"chat/state".as_slice(), b"chat/fresh"] {
            world.fixture_file(name, &smith_local_protocol::save_state(state).bytes);
        }
        world.fixture_file(b"chat/0000000001.turn", b"old local transcript");
        world.scenario.ending = Ending::OlderStore;
        world.scenario.commands = vec![smith_local_process_world::terminal::Action::Send(b"hello\n".as_slice().into())];
        let before = world.store_snapshot();
        world.settle();
        assert_eq!(String::from_utf8_lossy(world.errors()).lines().last(), Some(OPERATOR_LINE));
        assert_eq!(world.exit(), Some(kernel::Exit::Code(1)));
        assert!(world.queries().is_empty());
        assert!(world.facts().is_empty());
        assert_eq!(world.store_snapshot(), before);
    }
}
