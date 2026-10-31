//! Absent native usage remains absent at the root boundary
//! (smith/docs/design/protocol/llm.md, sections 6 and 10).
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_fake_llm_protocol::documents::{UsageField, UsageFields};
use smith_domain::run;
use smith_protocol_llm_world::{CompletionTerminal, Job, World, fixture, wire};

const ITERATIONS: u32 = 100_000;

fn script() -> Box<[Script]> {
    Box::new([Script {
        cue: b"@report".as_slice().into(),
        turns: Box::new([
            Turn {
                lines: Box::new([Line::Text { text: b"First native answer.".as_slice().into() }]),
                finish: Finish::Stop,
                tokens: 3,
            },
            Turn {
                lines: Box::new([Line::Call {
                    name: b"finish".as_slice().into(),
                    arguments: br#"{"report":"Native answer.","source":"peer"}"#.as_slice().into(),
                }]),
                finish: Finish::ToolCalls,
                tokens: 7,
            },
        ]),
    }])
}

#[test]
fn optional_native_usage_stays_absent_through_the_root() {
    for mut configuration in wire::configurations() {
        configuration.usage_fields = UsageFields::NONE.with(UsageField::Input).with(UsageField::Output);
        let mut settings = fixture::settings(712);
        settings.job = Job::Reporting;
        let mut bounds = fixture::limits();
        bounds.client.http.request = 16_384;
        bounds.client.request = 16_384;
        let receiving = smith_protocol_llm::fixture::receiving(&bounds.client, settings.limits.decoded_call_bytes)
            .expect("fixture receiving bound");
        settings.limits.session.completion_bytes = receiving.max_completion_bytes;
        settings.limits.session.completion_blocks = receiving.max_completion_blocks;
        settings.limits.session.failure_bytes = receiving.max_failure_bytes;
        let mut world = World::with_wire(settings, None, configuration, bounds, script());
        world.run(ITERATIONS);
        assert_eq!(world.completions().len(), 2);
        for (observed, output) in world.completions().iter().zip([3, 7]) {
            let (_, CompletionTerminal::Completed(usage)) = observed.terminal.expect("one completion terminal") else {
                panic!("scripted peer completed")
            };
            assert_eq!([usage.input_tokens, usage.output_tokens], [Some(0), Some(output)]);
            assert_eq!([usage.cache_read_tokens, usage.cache_write_tokens, usage.reasoning_tokens], [None; 3]);
        }
        let run::Answer::Accepted { spent, turns, .. } = world.answer() else { panic!("root accepted the report") };
        assert_eq!((*turns, spent.output), (2, Some(10)));
        assert_eq!([spent.cache_read, spent.cache_write, spent.reasoning], [None; 3]);
        assert!(world.turns().iter().all(|turn| {
            turn.usage.cache_read_tokens.is_none()
                && turn.usage.cache_write_tokens.is_none()
                && turn.usage.reasoning_tokens.is_none()
        }));
        assert_ne!(run::Spend { cache_write: Some(0), ..*spent }, *spent, "absent differs from reported zero");
    }
}
