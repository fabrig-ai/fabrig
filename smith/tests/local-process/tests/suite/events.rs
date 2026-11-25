use smith_agent_process_world::events;
use smith_events::{Event, Mode};
use smith_local_process_world::world::{Files, Placement, World};

#[test]
fn the_products_header_carries_its_local_limits() {
    for placement in [Placement::InProcess, Placement::Spawned] {
        let files = Files::new();
        let mut world = World::traced(23, &files, placement, b"First answer");
        world.settle();
        let records = world.records();
        let Event::SessionStarted(header) = &records[0].event else { panic!("header first") };
        let local = header.limits.iter().find(|limit| limit.name.as_ref() == b"local.line_bytes");
        let memory = header.limits.iter().find(|limit| limit.name.as_ref() == b"declared.memory");
        assert_eq!(local.map(|limit| limit.value), Some(smith_events::LimitValue::Integer(248)));
        assert_eq!(memory.map(|limit| limit.value), Some(smith_events::LimitValue::Integer(2_147_483_648)));
        assert!(!header.limits.iter().any(|limit| limit.name.as_ref() == b"inline.memory"));
        let inline = header.limits.iter().any(|limit| limit.name.as_ref() == b"inline.window.bytes");
        let host = header.limits.iter().any(|limit| limit.name.as_ref() == b"host.process.io.sockets");
        assert_eq!(inline, matches!(placement, Placement::InProcess));
        assert_eq!(host, matches!(placement, Placement::Spawned));
    }
}

#[test]
fn an_inline_chat_writes_a_complete_stream_through_its_answer() {
    let files = Files::new();
    let mut world = World::traced(11, &files, Placement::InProcess, b"First answer");
    world.settle();
    let records = world.records();
    events::check(&records).expect("conforming events");
    let Event::SessionStarted(header) = &records[0].event else { panic!("header first") };
    assert_eq!(header.mode, Mode::Chat);
    assert_eq!(header.chat.as_deref(), Some(b"main".as_slice()));
}

#[test]
fn a_slowed_destination_loses_no_record_under_complete() {
    let files = Files::new();
    let mut world = World::traced(111, &files, Placement::InProcess, b"First answer");
    world.settle();
    events::check(&world.records()).expect("conforming events");
}

#[test]
fn one_response_completed_per_provider_response_of_an_inline_chat() {
    let files = Files::new();
    let mut world = World::traced(13, &files, Placement::InProcess, b"First answer");
    world.settle();
    let responses = world.records().iter().filter(|record| matches!(record.event, Event::ResponseCompleted(_))).count();
    assert_eq!(responses, world.queries().len());
    assert!(responses > 0);
}

#[test]
fn a_spawned_chat_writes_its_answer_and_end_to_the_product_stream() {
    let files = Files::new();
    let mut world = World::traced(17, &files, Placement::Spawned, b"First answer");
    world.settle();
    let records = world.records();
    assert!(matches!(records.first().map(|r| &r.event), Some(Event::SessionStarted(_))));
    assert!(matches!(records.last().map(|r| &r.event), Some(Event::SessionEnded(_))));
    assert_eq!(records.iter().filter(|r| matches!(r.event, Event::RunCompleted(_))).count(), 1);
    assert_eq!(records.iter().filter(|r| matches!(r.event, Event::SessionEnded(_))).count(), 1);
}

#[test]
fn the_spawned_agents_records_stay_in_its_own_trace() {
    let files = Files::new();
    let mut world = World::traced(19, &files, Placement::Spawned, b"First answer");
    world.settle();
    let product = world.records();
    let agent = world.agent_records();
    events::check(&agent).expect("conforming events");
    assert!(agent.iter().any(|r| matches!(r.event, Event::RunStarted(_))));
    assert!(agent.iter().any(|r| matches!(r.event, Event::ResponseCompleted(_))));
    assert!(!product.iter().any(|r| matches!(r.event, Event::RunStarted(_) | Event::ResponseCompleted(_))));
}

#[test]
fn a_spawned_agent_that_exits_without_answering_completes_as_agent_ended() {
    let files = Files::new();
    let mut world = World::traced(23, &files, Placement::Spawned, b"First answer");
    world.scenario.ending = smith_local_process_world::referee::Ending::AgentEnded;
    world.scenario.commands = vec![smith_local_process_world::terminal::Action::Send(b"hello\n".as_slice().into())];
    world.settle();
    let records = world.records();
    let completed: Vec<_> = records
        .iter()
        .filter_map(|record| if let Event::RunCompleted(completed) = &record.event { Some(completed) } else { None })
        .collect();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].status, smith_events::Status::Failed);
    let failure = completed[0].failure.as_ref().expect("unanswering agent");
    assert_eq!(failure.class, smith_events::AnswerClass::Agent);
    assert_eq!(failure.ended, Some(smith_events::AgentExit::Code(7)));
    assert_eq!(completed[0].result, None);
    assert_eq!(world.exit(), Some(skein_io::kernel::Exit::Code(1)));
}

#[test]
fn a_forced_exit_is_a_host_notice_in_the_product_stream() {
    let files = Files::new();
    let mut world = World::dropped(12, &files, Placement::Spawned);
    world.scenario.launch.traced = true;
    world.settle();
    let records = world.records();
    let notices: Vec<_> = records
        .iter()
        .filter_map(|record| {
            if let Event::Notice(notice) = &record.event
                && notice.kind == smith_events::NoticeKind::ForcedExit
            {
                Some(notice)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(notices.len(), 0);
    assert!(matches!(records.last().map(|r| &r.event), Some(Event::SessionEnded(_))));
}

#[test]
fn a_program_exec_traces_its_run_to_the_agent_file() {
    use smith_events::Block;
    use smith_fake_claude::{Step, Story, Turn};

    let files = Files::new();
    let prompt = b"Ask first";
    const ANSWER: &str = "Program answer";
    let mut world = World::traced(19, &files, Placement::Spawned, ANSWER.as_bytes());
    world.exec(prompt, ANSWER.as_bytes());
    world.on_program(Story {
        turns: vec![Turn::new(vec![Step::Outcome {
            input: serde_json::json!({"outcome":{"form":"report","text":ANSWER,"fields":{}}}),
        }])],
        ..Story::default()
    });
    world.settle();
    let product = assert_exec_accepted(&mut world);
    assert!(matches!(product.first().map(|record| &record.event), Some(Event::SessionStarted(_))));
    assert!(matches!(product.last().map(|record| &record.event), Some(Event::SessionEnded(_))));
    assert!(
        product.iter().all(|record| matches!(
            record.event,
            Event::SessionStarted(_) | Event::SessionEnded(_) | Event::RunCompleted(_) | Event::Notice(_)
        )),
        "detailed records belong to the child process"
    );
    let agent = world.agent_records();
    assert_run_started(&agent, false);
    assert!(
        agent.iter().any(|record| matches!(&record.event, Event::ResponseStarted(start)
        if start.prompt.as_ref().is_some_and(|capture| capture.messages.iter().any(|message|
            message.blocks.iter().any(|block| matches!(block, Block::Text { text }
                if skein_lib::bytes::find(text, prompt).is_some())))))),
        "the task reached the agent trace"
    );
    assert!(
        agent.iter().any(|record| matches!(&record.event, Event::ToolCompleted(tool)
        if tool.tool.as_ref() == b"finish" && tool.verdict == smith_events::Verdict::Result)),
        "the agent records the finish completion"
    );
}

pub(super) fn assert_exec_accepted(world: &mut World) -> Vec<smith_events::Record> {
    assert_eq!(world.exit(), Some(skein_io::kernel::Exit::Code(0)));
    let records = world.records();
    let completed = super::run_completed(&records);
    assert_eq!(completed.len(), 1, "one product answer");
    assert_eq!(completed[0].status, smith_events::Status::Accepted);
    assert!(completed[0].result.is_some(), "accepted result");
    let ends: Vec<_> = records
        .iter()
        .filter_map(|record| match &record.event {
            Event::SessionEnded(end) => Some(end),
            _ => None,
        })
        .collect();
    assert_eq!(ends.len(), 1, "one product terminal");
    assert_eq!(ends[0].exit, 0);
    assert!(ends[0].answered && !ends[0].aborted, "the product answered and drained");
    records
}

pub(super) fn assert_run_started(records: &[smith_events::Record], resumed: bool) {
    let starts: Vec<_> = records
        .iter()
        .filter_map(|record| match &record.event {
            Event::RunStarted(start) => Some(start.resumed),
            _ => None,
        })
        .collect();
    assert_eq!(starts, [resumed], "one run starts with the expected resume state");
}
