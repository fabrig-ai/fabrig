use super::*;
use crate::responses::tests::{at, intent, opened, prompt};

fn requesting(
    backend: smith_events::Backend,
    capture: Capture,
) -> (Sinks, Token, Token, Option<smith_protocol_llm::Observation>) {
    let bounds = limits(1);
    let mut sinks = Sinks::new(&bounds, capture, bounds.output, DEADLINE);
    let destination = Token::new(400);
    sinks.add(Destination::Trace, destination, Delivery::Complete).unwrap();
    grant(&mut sinks, destination);
    let mut opening = opened(0, 0);
    opening.model.backend = backend;
    let conversation = Token::new(11);
    let owner = Token::new(99);
    sinks.responses.opened(at(1), conversation, &opening);
    intent(&mut sinks.responses, conversation, smith_domain::session::Choice::Auto, 0);
    let observation = sinks.request(at(2), Time::ZERO, owner, &prompt(0));
    (sinks, destination, owner, observation)
}

#[test]
fn direct_calls_capture_starts_without_a_prompt_observation() {
    let (mut sinks, destination, _, observation) = requesting(smith_events::Backend::Direct, Capture::Calls);
    let records = drain_ready(&mut sinks, destination);
    assert!(observation.is_none());
    assert_eq!(records[0].t_ms, 2);
}

#[test]
fn prompt_timestamps_set_the_backend_duration() {
    use smith_events::Backend;
    for (backend, capture, sent, duration, content) in [
        (Backend::Direct, Capture::Everything, 2, 8, true),
        (Backend::ClaudeCode, Capture::Everything, 7, 3, true),
        (Backend::ClaudeCode, Capture::Calls, 7, 3, false),
        (Backend::ClaudeCode, Capture::None, 7, 3, false),
    ] {
        let (mut sinks, destination, owner, observation) = requesting(backend, capture);
        assert!(observation.is_some());
        assert!(drain_ready(&mut sinks, destination).is_empty());
        sinks.measure(
            Time::ZERO,
            smith_protocol_llm::Measure::Prompt {
                at: at(sent),
                owner,
                prompt: smith_events::Prompt { system: None, tools: None, messages: List::with_capacity(0) },
            },
        );
        let records = drain_ready(&mut sinks, destination);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].t_ms, sent);
        let Event::ResponseStarted(started) = &records[0].event else { panic!("response start") };
        assert_eq!(started.prompt.is_some(), content);
        assert_eq!(sinks.responses.observed(owner), content);
        sinks.measure(
            Time::ZERO,
            smith_protocol_llm::Measure::Completion {
                at: at(9),
                owner,
                completion: smith_events::Completion { blocks: List::with_capacity(0) },
            },
        );
        let records = sinks.responses.retried(at(10), Time::ZERO, owner, Some(503), None);
        let Event::ResponseCompleted(response) = &records.get(0).unwrap().event else { panic!("response") };
        assert_eq!(response.duration_ms, duration);
        assert_eq!(response.completion.is_some(), content);
    }
}
