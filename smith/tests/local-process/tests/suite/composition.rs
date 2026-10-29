//! Behaviour at the local shell, agent and fake-machine composition.
use skein_fake_llm_domain::api;
use skein_io::kernel;
use smith_events::Event;
use smith_local_process_world::{Files, Placement, World, referee::{Ending, review}};

fn write_report(name: &str) -> Box<[api::Script]> {
    Box::new([api::Script {
        cue: b"@local-shell".as_slice().into(),
        turns: [
            ("write", format!(r#"{{"path":"/{name}/notes.txt","content":"noted\n"}}"#)),
            ("finish", r#"{"form":"report","text":"Wrote it","fields":{}}"#.into()),
        ].into_iter().map(|(name, arguments)| api::Turn {
            lines: Box::new([api::Line::Call { name: name.as_bytes().into(), arguments: arguments.into_bytes().into() }]),
            finish: api::Finish::ToolCalls,
            tokens: 1,
        }).collect(),
    }])
}

fn directory(seed: u64, files: &Files, placement: Placement, name: &str, writable: bool) -> World {
    let mut world = World::plain(seed, files, placement, name.as_bytes(), writable);
    world.scenario.scripts = Some(write_report(name));
    world.scenario.launch.traced = true;
    world
}

/// Lower test of smith's end_to_end::a_dir_is_a_named_directory_the_run_writes_unless_it_is_read_only (top tiers test wiring only).
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

/// Lower test of smith's end_to_end::a_dir_is_a_named_directory_the_run_writes_unless_it_is_read_only (top tiers test wiring only).
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
        let verdicts: Vec<_> = records.iter().filter_map(|record| match &record.event {
            Event::ToolCompleted(tool) if tool.tool.as_ref() == b"write" => Some(&tool.verdict),
            _ => None,
        }).collect();
        assert_eq!(verdicts, [&smith_events::Verdict::ReadOnly], "{placement:?}");
        assert_eq!(world.file(b"extra/notes.txt"), None);
    }
}
