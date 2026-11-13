//! Stateless host tools and fenced results (protocol/llm.md, 4 and 5).
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token, Writer};
use smith_domain_run as run;
use temper_engine_domain as engine;
use temper_engine_smith::{Decoded, ForgeReadBounds, Problem};

/// Decode a host call without configured models or repository identities.
#[must_use]
pub fn decode(to: ReplyTo, task: u64, attempt: u64, call: &fleet::Call) -> engine::Event {
    decode_configured(to, task, attempt, call.contract, call, &[], &[], &[])
}
/// Decode a host call using the component's configured models and repository identities.
#[must_use]
#[expect(clippy::too_many_arguments, reason = "the call's contract travels with its configured decoding context")]
pub fn decode_configured(
    to: ReplyTo,
    _task: u64,
    attempt: u64,
    contract: u16,
    call: &fleet::Call,
    models: &[engine::Model],
    catalogue: &[core::settings::CatalogueModel],
    repositories: &[engine::RepositoryIdentity],
) -> engine::Event {
    let Some(name) = jig_protocol_charter::call_key(&call.name, attempt) else {
        return invalid(to, Problem::Malformed);
    };
    if call.input.len() > run::HostInput::CAPACITY {
        return invalid(to, Problem::TooLarge);
    }
    let Some(input) = run::HostInput::attested(call.input.clone()) else {
        return invalid(to, Problem::Malformed);
    };
    let name = run::CallName { activation: name.activation, completion: name.completion, position: name.position };
    match temper_engine_smith::call_for_contract(contract, name, &call.tool, &input, models, catalogue, repositories) {
        Ok(Decoded::Call(body)) => engine::Event::Worker(engine::WorkerInput::DecodedHostCall { to, body }),
        Ok(Decoded::Answered(answer)) => {
            let (body, error) = answer.into_parts();
            engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
                to,
                answer: core::SettledAnswer::Host { error, body },
            })
        }
        Err(problem) => invalid(to, problem),
    }
}
/// Translate an internally authorized whole settings note into a proposal call.
#[must_use]
pub fn settings_note(
    to: ReplyTo,
    call: engine::Call,
    body: &[u8],
    models: &[core::settings::CatalogueModel],
) -> engine::Event {
    match temper_engine_smith::settings_note(call, body, models) {
        Ok(Decoded::Call(body)) => engine::Event::Worker(engine::WorkerInput::DecodedHostCall { to, body }),
        Ok(Decoded::Answered(answer)) => rendered(to, answer),
        Err(problem) => invalid(to, problem),
    }
}
fn invalid(to: ReplyTo, problem: Problem) -> engine::Event {
    let (body, member): (&[u8], Option<&[u8]>) = match problem {
        Problem::Malformed => (b"malformed host call", None),
        Problem::TooLarge => (b"host call too large", None),
        Problem::Missing(member) => (b"host call member missing", Some(member)),
        Problem::Needs { kind, member } => {
            let separator = b" needs ";
            let guidance = b"; read the inventory and name what it lists";
            let mut writer = Writer::new(
                kind.len()
                    .checked_add(separator.len())
                    .expect("fixed error separator")
                    .checked_add(member.len())
                    .expect("static error member")
                    .checked_add(guidance.len())
                    .expect("fixed error guidance"),
            );
            writer.put(kind).expect("static setting kind");
            writer.put(separator).expect("measured separator");
            writer.put(member).expect("static member name");
            writer.put(guidance).expect("measured guidance");
            return engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
                to,
                answer: core::SettledAnswer::Host { error: true, body: writer.finish() },
            });
        }
        Problem::Type(member) => (b"host call member of wrong type", Some(member)),
        Problem::Range(b"full commit identifier") => (
            b"expected a full commit identifier (40 or 64 lowercase hexadecimal digits); use the full head from the review brief",
            None,
        ),
        Problem::Range(member) => (b"host call out of range", Some(member)),
        Problem::Beside(member) => {
            (b"host call member belongs in each change, not beside changes", Some(member))
        }
        Problem::UnknownTool => (b"unknown tool", None),
        Problem::Unasked(_) => (b"host call answer of an unasked kind", None),
    };
    let body = match member {
        None => body.into(),
        Some(member) => {
            let mut writer = Writer::new(
                body.len()
                    .checked_add(2)
                    .expect("fixed error separator")
                    .checked_add(member.len())
                    .expect("static error member"),
            );
            writer.put(body).expect("measured error");
            writer.put(b": ").expect("measured separator");
            writer.put(member).expect("static member name");
            writer.finish()
        }
    };
    engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
        to,
        answer: core::SettledAnswer::Host { error: true, body },
    })
}
/// Render a typed answer as the component's bounded host-call terminal.
#[must_use]
pub fn render(
    to: ReplyTo,
    answer: &engine::CallAnswer,
    tool: &[u8],
    max_bytes: u32,
    forge: ForgeReadBounds,
) -> engine::Event {
    rendered(to, temper_engine_smith::answer_for_tool(answer, tool, max_bytes, forge))
}
/// Render within the core body and receiving agent allowances, preserving the
/// proposing task's identity. `max_bytes` excludes the settled call's tool name.
#[must_use]
#[expect(clippy::too_many_arguments, reason = "host rendering carries the asked job beside both answer bounds")]
pub fn render_for_task(
    to: ReplyTo,
    answer: &engine::CallAnswer,
    tool: &[u8],
    max_bytes: u32,
    task: u64,
    receiving_bytes: u32,
    forge: ForgeReadBounds,
    job: Option<skein_forge::api::JobAttempt>,
) -> engine::Event {
    render_for_task_page(to, answer, tool, max_bytes, task, receiving_bytes, forge, job, None, engine::AskHow::None)
}
#[must_use]
#[expect(clippy::too_many_arguments, reason = "host rendering carries the asked job beside both answer bounds")]
pub fn render_for_task_page(
    to: ReplyTo,
    answer: &engine::CallAnswer,
    tool: &[u8],
    max_bytes: u32,
    task: u64,
    receiving_bytes: u32,
    forge: ForgeReadBounds,
    job: Option<skein_forge::api::JobAttempt>,
    page: Option<(u32, u32)>,
    ask: engine::AskHow,
) -> engine::Event {
    let max_bytes = max_bytes.min(receiving_bytes);
    if let Some((page, skip)) = page {
        return rendered(to, temper_engine_smith::answer_for_page(answer, tool, max_bytes, forge, page, skip));
    }
    if let Some(job) = job {
        return rendered(to, temper_engine_smith::answer_for_job(answer, tool, max_bytes, forge, job));
    }
    if let engine::CallAnswer::Decided { .. } = answer {
        return rendered(to, temper_engine_smith::answer_for_decided(answer, ask));
    }
    if let engine::CallAnswer::Proposed { .. } = answer {
        rendered(to, temper_engine_smith::answer_for_task(answer, tool, max_bytes, task, forge))
    } else {
        render(to, answer, tool, max_bytes, forge)
    }
}
fn rendered(to: ReplyTo, answer: smith_domain_run::HostAnswer) -> engine::Event {
    let (body, error) = answer.into_parts();
    engine::Event::Worker(engine::WorkerInput::RenderedHostCall {
        to,
        answer: core::SettledAnswer::Host { error, body },
    })
}
/// Translate a declared task outcome into the component's result-read terminal.
#[must_use]
pub fn result(payload: Token, outcome: &[u8], change: Option<temper_engine_smith::ChangeResource>) -> engine::Event {
    let end = match smith_protocol_translate::decode_declared(outcome, &smith_charter::CEILINGS) {
        Ok(outcome) => {
            let spent = run::Spend {
                turns: 0,
                input: None,
                output: None,
                cache_read: None,
                cache_write: None,
                reasoning: None,
                units: 0,
            };
            match temper_engine_smith::result(run::Answer::Accepted { outcome, spent, turns: 0 }, change) {
                Ok(terminal) => terminal.end,
                Err(_) => tasks::End::Failed(tasks::Class::Invalid),
            }
        }
        Err(_) => tasks::End::Failed(tasks::Class::Invalid),
    };
    engine::Event::Worker(engine::WorkerInput::ResultRead { payload, end })
}
/// The hub carries a settled workspace delivery as host text.
#[must_use]
pub fn settled(answer: core::SettledAnswer) -> host::Relayed {
    match answer {
        core::SettledAnswer::Host { error, body } => host::Relayed::Answer { error, text: body },
        core::SettledAnswer::Delivery { outcome, evidence } => {
            let (label, error): (&[u8], bool) = match outcome {
                core::DeliveryOutcome::Delivered => (b"delivered", false),
                core::DeliveryOutcome::Nothing => (b"nothing to deliver", false),
                core::DeliveryOutcome::Stale => (b"stale delivery", true),
                core::DeliveryOutcome::Refused => (b"delivery refused", true),
                core::DeliveryOutcome::Invalid => (b"invalid delivery", true),
                core::DeliveryOutcome::Failed => (b"delivery failed", true),
            };
            let bytes = label.len().checked_add(2).expect("bounded delivery label");
            let bytes = bytes.checked_add(evidence.len()).expect("bounded delivery text");
            let mut writer = Writer::new(bytes);
            writer.put(label).expect("measured delivery label");
            writer.put(b": ").expect("measured separator");
            writer.put(&evidence).expect("measured evidence");
            let Some(answer) = run::HostAnswer::new(writer.finish(), error) else {
                return host::Relayed::Answer { error: true, text: b"delivery answer too large".as_slice().into() };
            };
            let (text, error) = answer.into_parts();
            host::Relayed::Answer { error, text }
        }
    }
}
