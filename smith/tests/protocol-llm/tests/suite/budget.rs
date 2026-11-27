//! Native usage and serving speed through to the run's posted spend
//! (smith/docs/design/protocol/llm.md, sections 6 and 10).
use skein_fake_llm_domain::api::{Finish, Line, Script, Turn};
use skein_fake_llm_protocol::documents::{UsageField, UsageFields};
use smith_domain::run;
use smith_protocol_llm_world::{
    CompletionTerminal, Job, World, client_peer,
    fixture::{self, Dialect},
};

const ITERATIONS: u32 = 100_000;
const INPUT_OUTPUT: UsageFields = UsageFields::NONE.with(UsageField::Input).with(UsageField::Output);

const FINISH_REPORT: &[u8] = br#"{"form":"report","text":"Native answer.","fields":{"source":"peer"}}"#;

fn script() -> Box<[Script]> {
    Box::new([Script {
        cue: b"@report".as_slice().into(),
        turns: Box::new([
            smith_protocol_llm_world::fixture::says(
                Box::new([Line::Text { text: b"First native answer.".as_slice().into() }]),
                3,
            ),
            smith_protocol_llm_world::fixture::calls(
                Box::new([Line::Call { name: b"finish".as_slice().into(), arguments: FINISH_REPORT.into() }]),
                7,
            ),
        ]),
    }])
}

fn settings_and_bounds() -> (smith_agent_world::Settings, smith_protocol_llm::Limits) {
    let mut settings = fixture::settings(712);
    settings.job = Job::Reporting;
    let mut bounds = fixture::limits();
    bounds.client.http.request = 16_384;
    bounds.client.request = 16_384;
    let receiving = smith_protocol_llm_fixtures::receiving(&bounds.client, settings.limits.decoded_call_bytes)
        .expect("fixture receiving bound");
    settings.limits.session.completion_bytes = receiving.max_completion_bytes;
    settings.limits.session.completion_blocks = receiving.max_completion_blocks;
    settings.limits.session.failure_bytes = receiving.max_failure_bytes;
    (settings, bounds)
}

#[test]
fn optional_native_usage_stays_absent_through_the_root() {
    for mut configuration in client_peer::configurations() {
        configuration.usage_fields = INPUT_OUTPUT;
        let (settings, bounds) = settings_and_bounds();
        let mut world = World::with_protocol(settings, None, configuration, bounds, script());
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

fn served_turn(dialect: Dialect, requested: &[u8], multiplier: u64, served: Option<&[u8]>, finish: Finish) -> World {
    let (mut settings, bounds) = settings_and_bounds();
    settings.limits.run.nudges = 0;
    let mut charter = smith_agent_world::scripted_charter(&settings);
    let prices = run::Prices { input: 1, cached: 1, cache_write: None, output: 20, unit: 100 };
    charter.llm.prices = prices;
    charter.llm.speed = Some(run::charter::Speed {
        name: requested.into(),
        also: if requested == b"fast" { Box::new([b"priority".as_slice().into()]) } else { Box::new([]) },
        prices: run::Prices { input: multiplier, cached: multiplier, output: prices.output * multiplier, ..prices },
    });
    let mut lines = Vec::new();
    if let Some(name) = served {
        lines.push(Line::Served { name: name.into() });
    }
    if finish == Finish::Length {
        lines.push(Line::Text { text: b"Cut short.".as_slice().into() });
    } else {
        lines.push(Line::Call { name: b"finish".as_slice().into(), arguments: FINISH_REPORT.into() });
    }
    let scripts = Box::new([Script {
        cue: b"@report".as_slice().into(),
        turns: Box::new([Turn { lines: lines.into_boxed_slice(), finish, tokens: 20 }]),
    }]);
    let mut configuration = dialect.configuration();
    configuration.usage_fields = INPUT_OUTPUT;
    let mut world = World::with_workspace_protocol_charter(
        settings,
        None,
        None,
        skein_fake_checkout::Checkout::new(),
        (configuration, bounds),
        scripts,
        charter,
    );
    world.run(ITERATIONS);
    world
}

fn assert_spend(world: &World, spent: &run::Spend, units: u64, served: Option<&[u8]>) {
    assert_eq!((spent.input, spent.output), (Some(0), Some(20)));
    assert_eq!(spent.units, units, "reported speed {served:?}");
    let [turn] = world.turns() else { panic!("one posted turn") };
    assert_eq!(turn.spent, units);
}

#[test]
fn responses_post_the_charge_for_the_serving_tier() {
    for (requested, multiplier, served, units) in [
        (b"ultrafast".as_slice(), 6, Some(b"ultrafast".as_slice()), 24),
        (b"fast", 2, Some(b"priority"), 8),
        (b"ultrafast", 6, Some(b"default"), 4),
        (b"ultrafast", 6, Some(b"turbo"), 24),
        (b"ultrafast", 6, None, 24),
    ] {
        let world = served_turn(Dialect::OpenAi, requested, multiplier, served, Finish::ToolCalls);
        let run::Answer::Accepted { spent, turns: 1, .. } = world.answer() else {
            panic!("one report for serving tier {served:?}: {:?}", world.answer());
        };
        assert_spend(&world, spent, units, served);
    }
}

#[test]
fn an_incomplete_response_still_posts_the_serving_tier_charge() {
    for (served, units) in [(b"ultrafast".as_slice(), 24), (b"default".as_slice(), 4)] {
        let world = served_turn(Dialect::OpenAi, b"ultrafast", 6, Some(served), Finish::Length);
        let run::Answer::Failed { failure: run::RunFailure::Model(run::Fault::Truncated), spent, turns: 1 } =
            world.answer()
        else {
            panic!("truncated after one charged completion: {:?}", world.answer());
        };
        assert_spend(&world, spent, units, Some(served));
    }
}

#[test]
fn anthropic_posts_the_charge_for_the_speed_in_usage() {
    for (served, units) in [(b"fast".as_slice(), 8), (b"standard".as_slice(), 4)] {
        let world = served_turn(Dialect::Anthropic, b"fast", 2, Some(served), Finish::ToolCalls);
        let run::Answer::Accepted { spent, turns: 1, .. } = world.answer() else {
            panic!("one report for serving speed {served:?}: {:?}", world.answer());
        };
        assert_spend(&world, spent, units, Some(served));
    }
}
