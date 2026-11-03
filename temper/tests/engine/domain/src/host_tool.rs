//! Scripted host calls through the root and smith's decoder and renderer
//! (temper/docs/design/domain/testing.md, 6.4).
use crate::direct::Driver;
use jig_domain_core as core;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Event};

const STEPS: usize = 200; // The direct world's settlement step budget.

/// Send a scripted model's host call and return the answer relayed to its run.
pub fn host_tool(
    driver: &mut Driver,
    run: &engine::Assignment,
    completion: u32,
    tool: &[u8],
    input: &[u8],
) -> (Box<[u8]>, bool) {
    use crate::protocol;
    use jig_domain_host as host;
    use smith_domain_run as smith;
    let delivered = driver.delivered.len();
    let mut name = skein_lib::Writer::new(16);
    name.put(&run.attempt.to_be_bytes()).expect("activation");
    name.put(&completion.to_be_bytes()).expect("completion");
    name.put(&0_u32.to_be_bytes()).expect("position");
    let name = name.finish();
    driver.protocol.defer_render = true;
    driver.send(protocol::up(
        Token::new(7),
        host::LinkUp::Call {
            run: Token::new(run.task),
            attempt: Token::new(run.attempt),
            call: name.clone(),
            tool: tool.into(),
            writes: tool != b"settings",
            input: input.into(),
            deadline: Duration::from_secs(10),
        },
        engine::Parts::none(),
    ));
    for _ in 0..STEPS {
        driver.advance(true);
        if !driver.host_requests.is_empty() {
            break;
        }
    }
    let request = driver.host_requests.pop().expect("host decoder called");
    let engine::HostRequest::Decode { to, call, .. } = *request else { panic!("decode") };
    assert_eq!(call.name, name);
    let input = smith::HostInput::attested(call.input).expect("host input");
    let decoded = temper_engine_smith::call_configured(
        smith::CallName { activation: run.attempt, completion, position: 0 },
        &call.tool,
        &input,
        driver.root.models(),
    )
    .expect("valid host call");
    let (to, answer) = match decoded {
        temper_engine_smith::Decoded::Answered(answer) => (to, answer),
        temper_engine_smith::Decoded::Call(body) => render(driver, to, body, &name),
    };
    let (body, error) = answer.into_parts();
    driver.send(Event::Worker(engine::WorkerInput::RenderedHostCall {
        to,
        answer: core::SettledAnswer::Host { error, body },
    }));
    for _ in 0..STEPS {
        driver.advance(true);
        for delivery in driver.delivered[delivered..].iter().rev() {
            if let Delivery::Link {
                down: host::LinkDown::Relayed { call, answer: host::Relayed::Answer { text, error }, .. },
                ..
            } = delivery
                && *call == name
            {
                return (text.clone(), *error);
            }
        }
    }
    panic!("settings tool answer reached the run");
}

fn render(
    driver: &mut Driver,
    to: ReplyTo,
    body: engine::Call,
    name: &[u8],
) -> (ReplyTo, smith_domain_run::HostAnswer) {
    driver.send(Event::Worker(engine::WorkerInput::DecodedHostCall { to, body }));
    for _ in 0..STEPS {
        driver.advance(true);
        if !driver.protocol.pending_renders.is_empty()
            || driver.delivered.iter().any(|delivery| {
                matches!(delivery,
                    Delivery::Host(value) if matches!(value.as_ref(), engine::HostDelivery::Settings { .. })
                )
            })
        {
            break;
        }
    }
    let mut settings_delivery = None;
    for (index, delivery) in driver.delivered.iter().enumerate() {
        if let Delivery::Host(delivery) = delivery
            && matches!(delivery.as_ref(), engine::HostDelivery::Settings { .. })
        {
            settings_delivery = Some(index);
            break;
        }
    }
    if let Some(index) = settings_delivery {
        let Delivery::Host(delivery) = driver.delivered.remove(index) else { panic!("settings delivery") };
        let engine::HostDelivery::Settings { to, answer, application, from, requested } = *delivery else {
            panic!("settings read")
        };
        (to, temper_engine_smith::settings_answer(&answer, from, requested, driver.root.models(), application.as_ref()))
    } else {
        let (to, rendered_name) = driver.protocol.pending_renders.pop().expect("host answer to render");
        assert_eq!(rendered_name.as_ref(), name);
        let (_, answer) = driver.protocol.answers.last().expect("typed answer");
        (to, temper_engine_smith::answer(answer))
    }
}
