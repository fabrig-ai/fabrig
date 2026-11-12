//! Scripted host calls through the root and smith's decoder and renderer
//! (temper/docs/design/testing/checks.md, 4).
use crate::{direct::Driver, protocol};
use jig_domain_core as core;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Event};

enum HostReply {
    Render { to: ReplyTo, answer: smith_domain_run::HostAnswer },
    Sent { body: Box<[u8]>, error: bool },
}

const STEPS: usize = 200; // The direct world's settlement step budget.

/// Send a scripted model's host call and return the answer relayed to its run.
pub fn host_tool(
    driver: &mut Driver,
    run: &engine::Assignment,
    completion: u32,
    tool: &[u8],
    input: &[u8],
) -> (Box<[u8]>, bool) {
    let delivered = driver.delivered.len();
    let mut name = skein_lib::Writer::new(16);
    name.put(&run.attempt.to_be_bytes()).expect("activation");
    name.put(&completion.to_be_bytes()).expect("completion");
    name.put(&0_u32.to_be_bytes()).expect("position");
    let name = name.finish();
    let (to, decoded) = decode(driver, run, completion, tool, input, &name);
    let (to, answer) = match decoded {
        temper_engine_smith::Decoded::Answered(answer) => (to, answer),
        temper_engine_smith::Decoded::Call(body) => match render(driver, to, body, &name, tool, run.task) {
            HostReply::Render { to, answer } => (to, answer),
            HostReply::Sent { body, error } => return (body, error),
        },
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
                down:
                    jig_domain_host::LinkDown::Relayed {
                        call,
                        answer: jig_domain_host::Relayed::Answer { text, error },
                        ..
                    },
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

fn render(driver: &mut Driver, to: ReplyTo, body: engine::Call, name: &[u8], tool: &[u8], task: u64) -> HostReply {
    let delivered = driver.delivered.len();
    driver.send(Event::Worker(engine::WorkerInput::DecodedHostCall { to, body }));
    for _ in 0..STEPS {
        driver.advance(true);
        if let Some((body, error)) = relayed(driver, name, delivered) {
            return HostReply::Sent { body, error };
        }
        if !driver.protocol.pending_renders.is_empty()
            || driver.delivered.iter().any(|delivery| {
                matches!(delivery,
                    Delivery::Host(value) if matches!(value.as_ref(), engine::HostDelivery::Settings { .. } | engine::HostDelivery::See { .. })
                )
            })
        {
            break;
        }
    }
    if let Some((to, answer)) = settings_delivery(driver) {
        HostReply::Render { to, answer }
    } else {
        let (to, rendered_name) = driver.protocol.pending_renders.pop().expect("host answer to render");
        assert_eq!(rendered_name.as_ref(), name);
        let (_, answer) = driver.protocol.answers.last().expect("typed answer");
        let maximum = driver.env.limits.call_answer_bytes.saturating_sub(u32::try_from(tool.len()).expect("tool name"));
        let answer = if let Some(ask) = driver.protocol.asks.get(&rendered_name) {
            temper_engine_smith::answer_for_decided(answer, *ask)
        } else if let Some((page, skip)) = driver.protocol.read_pages.get(&rendered_name) {
            temper_engine_smith::answer_for_page(
                answer,
                tool,
                maximum,
                temper_engine_smith::ForgeReadBounds::STOCK,
                *page,
                *skip,
            )
        } else {
            match driver.protocol.read_jobs.get(&rendered_name) {
                Some(job) => temper_engine_smith::answer_for_job(
                    answer,
                    tool,
                    maximum,
                    temper_engine_smith::ForgeReadBounds::STOCK,
                    *job,
                ),
                None => temper_engine_smith::answer_for_task(
                    answer,
                    tool,
                    maximum,
                    task,
                    temper_engine_smith::ForgeReadBounds::STOCK,
                ),
            }
        };
        HostReply::Render { to, answer }
    }
}

fn relayed(driver: &Driver, name: &[u8], delivered: usize) -> Option<(Box<[u8]>, bool)> {
    driver.delivered[delivered..].iter().rev().find_map(|delivery| {
        if let Delivery::Link {
            down:
                jig_domain_host::LinkDown::Relayed {
                    call, answer: jig_domain_host::Relayed::Answer { text, error }, ..
                },
            ..
        } = delivery
            && call.as_ref() == name
        {
            Some((text.clone(), *error))
        } else {
            None
        }
    })
}

fn decode(
    driver: &mut Driver,
    run: &engine::Assignment,
    completion: u32,
    tool: &[u8],
    input: &[u8],
    name: &[u8],
) -> (ReplyTo, temper_engine_smith::Decoded) {
    driver.protocol.defer_render = true;
    driver.send(protocol::up(
        Token::new(7),
        jig_domain_host::LinkUp::Call {
            run: Token::new(run.task),
            attempt: Token::new(run.attempt),
            call: name.into(),
            tool: tool.into(),
            writes: tool != b"settings" && !tool.starts_with(b"read_"),
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
    assert_eq!(call.name.as_ref(), name);
    let input = smith_domain_run::HostInput::attested(call.input).expect("host input");
    let decoded = temper_engine_smith::call_in(
        smith_domain_run::CallName { activation: run.attempt, completion, position: 0 },
        &call.tool,
        &input,
        driver.root.models(),
        &driver.root.catalogue(),
        &driver.root.repository_identities(run.task),
    )
    .expect("valid host call");
    (to, decoded)
}

fn settings_delivery(driver: &mut Driver) -> Option<(ReplyTo, smith_domain_run::HostAnswer)> {
    let mut settings_delivery = None;
    for (index, delivery) in driver.delivered.iter().enumerate() {
        if let Delivery::Host(delivery) = delivery
            && matches!(delivery.as_ref(), engine::HostDelivery::Settings { .. } | engine::HostDelivery::See { .. })
        {
            settings_delivery = Some(index);
            break;
        }
    }
    if let Some(index) = settings_delivery {
        let Delivery::Host(delivery) = driver.delivered.remove(index) else { panic!("settings delivery") };
        if let engine::HostDelivery::See { to, line, from, answer } = *delivery {
            return Some((
                to,
                temper_engine_smith::see_answer(
                    &line,
                    from,
                    &answer,
                    &driver.root.catalogue(),
                    driver.env.limits.call_answer_bytes.saturating_sub(3),
                    temper_engine_smith::ForgeReadBounds::STOCK,
                ),
            ));
        }
        let engine::HostDelivery::Settings {
            to,
            answer,
            repositories,
            people,
            forge_people,
            offer,
            application,
            from,
            requested,
        } = *delivery
        else {
            panic!("settings read")
        };
        Some((
            to,
            temper_engine_smith::settings_repository_answer(
                &answer,
                from,
                requested,
                &driver.root.catalogue(),
                application.as_ref(),
                &repositories,
                &people,
                &forge_people,
                &offer,
                driver.env.limits.call_answer_bytes.saturating_sub(8),
            ),
        ))
    } else {
        None
    }
}
