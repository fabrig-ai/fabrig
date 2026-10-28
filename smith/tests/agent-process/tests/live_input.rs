//! The main conversation's tool input crosses the real channel before its turn.
use skein_fake_llm_domain::api;
use skein_world::domain::heap::Counting;
use smith_agent_process_world::{TraceCase, World, charter};

#[global_allocator]
static HEAP: Counting = Counting;

const SEED: u64 = 37;
const FRAMES: u32 = 32;
const LIVE: u16 = 0x0112;
const TURN: u16 = 0x0109;

struct InputStream {
    turn: u32,
    id: Vec<u8>,
    tool: Vec<u8>,
    input: Vec<u8>,
}

#[test]
fn three_calls_and_a_finish_stream_input_in_each_dialect() {
    for anthropic in [false, true] {
        let mut world = World::new(SEED, &charter());
        if anthropic {
            world.anthropic();
        }
        world.observe_facts(TraceCase::ThreeChildren, FRAMES);
        world.ask(false, false, true);
        world.check_memory();
        assert!(world.settle());
        assert_eq!(world.lost_live_bytes(), 0);
        let pieces = world.pieces();
        let mut streams: Vec<InputStream> = Vec::new();
        for piece in &pieces {
            assert_eq!(*piece.kind(), smith_channel::LiveKind::Input);
            assert_eq!(piece.retry(), 0);
            let call = piece.call().as_ref().expect("call");
            assert!(!call.id().is_empty());
            if streams.last().is_none_or(|stream| stream.turn != piece.turn() || stream.id.as_slice() != call.id()) {
                assert_eq!(piece.offset(), 0);
                assert!(streams.iter().all(|stream| stream.id.as_slice() != call.id()), "distinct call identities");
                streams.push(InputStream {
                    turn: piece.turn(),
                    id: call.id().to_vec(),
                    tool: call.tool().to_vec(),
                    input: Vec::new(),
                });
            }
            let stream = streams.last_mut().expect("current stream");
            assert_eq!(usize::try_from(piece.offset()).expect("fixture offset"), stream.input.len());
            stream.input.extend_from_slice(piece.bytes());
        }
        assert_eq!(streams.len(), 4, "three main calls and finish; children give none");
        for (index, InputStream { turn, tool, input, .. }) in streams.iter().enumerate() {
            if index < 3 {
                assert_eq!(*turn, 1);
                assert_eq!(tool, b"sub_agent");
                assert_eq!(
                    input.as_slice(),
                    format!("{{\"brief\":\"@event-child-{index}\",\"tools\":[],\"max_turns\":2,\"max_seconds\":30}}")
                        .as_bytes()
                );
            } else {
                assert_eq!(tool, b"finish");
                assert_eq!(input, br#"{"form":"report","text":"done","fields":{}}"#);
            }
        }
        let mut last_turn = 0;
        for frame in world.observed() {
            if frame.kind == TURN {
                last_turn += 1;
            }
            if frame.kind == LIVE {
                let piece =
                    smith_channel::Live::decode(&smith_channel::CEILINGS, &mut skein_lib::Reader::new(&frame.body))
                        .expect("live frame");
                assert!(piece.turn() > last_turn, "no piece follows its turn");
            }
        }
    }
}

#[test]
fn kinds_not_asked_give_no_piece() {
    for (case, text, input) in [(TraceCase::Accepted, true, false), (TraceCase::Failed, false, true)] {
        let mut world = World::new(SEED, &charter());
        world.observe_facts(case, FRAMES);
        world.ask(text, false, input);
        assert!(world.settle());
        assert!(world.pieces().is_empty());
    }
}

#[test]
fn a_channel_with_no_room_counts_every_input_byte_and_still_sends_the_turn() {
    for length in [16_384, 16_385] {
        let mut world = World::new(SEED, &charter());
        world.anthropic();
        world.observe_facts(TraceCase::Failed, 1);
        world.output_tokens(4096);
        world.ask(false, false, true);
        world.script(Box::new([api::Turn {
            lines: Box::new([api::Line::Call {
                name: b"finish".as_slice().into(),
                arguments: vec![b' '; length].into(),
            }]),
            finish: api::Finish::ToolCalls,
            tokens: 1,
        }]));
        assert!(world.settle());
        assert_eq!(world.lost_live_bytes(), length as u64);
        assert!(world.pieces().is_empty());
        assert_eq!(world.observed().iter().filter(|frame| frame.kind == TURN).count(), 1);
    }
}
