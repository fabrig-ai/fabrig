//! The mapping between the agent's vocabulary and the provider's: what the two
//! protocol layers and the wire between them do, without the bytes. The
//! agent's side owns the tools' schemas, decodes the arguments the LLM writes
//! into typed calls, and renders owned outcomes as the product does
//! (smith/docs/design/protocol/llm.md, sections 5 and 8).

use smith_domain_session::Event;
use smith_domain_session::llm as agent;

use crate::tickets::{Ticketed, Tickets};
use skein_fake_llm_domain::api as provider;
use skein_lib::Token;
use smith_domain_tools::{Effect, Grants, Namespace};
use smith_protocol_llm::{Error, render_outcome};

/// The product's result bound; this world has no separate host tool or message replies.
pub(crate) fn render_bound(limits: &smith_domain_session::Limits) -> u32 {
    smith_protocol_llm::render_bound(&limits.tools, 0, limits.delegated_result_bytes)
        .expect("fixture limits give a result bound")
}

/// The tools the agent's side offers, by name: the family that grants each,
/// and its schema.
const TOOLS: [(&[u8], Family, &[u8]); 8] = [
    (b"read", Family::Inspect, br#"{"path":"string","offset":"integer","window":"integer"}"#),
    (b"list", Family::Inspect, br#"{"path":"string","depth":"integer","glob":"string"}"#),
    (b"search", Family::Inspect, br#"{"path":"string","pattern":"string","glob":"string"}"#),
    (b"write", Family::Modify, br#"{"path":"string","content":"string"}"#),
    (b"edit", Family::Modify, br#"{"path":"string","edits":[{"old":"string","new":"string","all":"boolean"}]}"#),
    (b"remove", Family::Modify, br#"{"path":"string"}"#),
    (
        b"shell",
        Family::Shell,
        br#"{"command":"string","timeout_seconds":"integer","output_bytes":"integer","yield_seconds":"integer"}"#,
    ),
    (b"job", Family::Shell, br#"{"job":"integer","wait_seconds":"integer","output_bytes":"integer","stop":"boolean"}"#),
];

#[derive(Clone, Copy)]
enum Family {
    Inspect,
    Modify,
    Shell,
}

/// The provider's query for an agent's prompt, its tickets resolved in
/// `tickets`: the live tools the opener serves. Uses the calm fixture's result
/// bound. There is one provider, so the endpoint names nothing.
#[must_use]
pub fn query(prompt: agent::Prompt, tickets: &Tickets) -> provider::Query {
    query_bounded(prompt, tickets, render_bound(&crate::Settings::calm(0).agent))
        .expect("scripted fixture prompt renders")
}

pub(crate) fn query_bounded(prompt: agent::Prompt, tickets: &Tickets, maximum: u32) -> Result<provider::Query, Error> {
    let agent::Prompt {
        purpose: _,
        endpoint: _,
        affinity,
        protocol_allowance: _,
        model,
        effort: _,
        system,
        tools,
        namespace,
        delegated,
        choice: _,
        messages,
        max_tokens,
    } = prompt;
    let mut offered = offer(tools).into_vec();
    for descriptor in &delegated {
        let Ticketed::Tool { name, effect, schema } = tickets.resolve(descriptor.ticket) else {
            panic!("a descriptor's ticket names a tool");
        };
        assert_eq!(*effect, descriptor.effect, "a descriptor says what its tool does");
        offered.push(provider::ToolSpec {
            name: (*name).into(),
            description: b"Served.".as_slice().into(),
            parameters: (*schema).into(),
        });
    }
    let messages = messages
        .into_iter()
        .map(|message| translate_message(message, &namespace, maximum))
        .collect::<Result<_, _>>()?;
    let caching = provider::Caching::Scope(affinity.key);
    Ok(provider::Query {
        credential: None,
        caching,
        model,
        system,
        tools: offered.into(),
        choice: provider::ToolChoice::Auto,
        messages,
        max_tokens,
    })
}

/// The agent's terminal event for the provider's answer to the call of `owner`,
/// a call of the session of `opener` that offered the tools of `served`. A
/// call to one of those is kept in `tickets`, as the top level would keep it.
#[must_use]
pub fn outcome(
    owner: Token,
    result: Result<provider::Answer, provider::Error>,
    tickets: &mut Tickets,
    opener: u64,
    served: &[agent::Descriptor],
) -> Event {
    match result {
        Ok(answer) => Event::Completed { owner, completion: completion(answer, tickets, opener, served) },
        Err(error) => Event::Failed {
            retry_ok: true,
            owner,
            failure: failure(error),
            evidence: smith_domain_session::llm::Evidence::Unknown,
            detail: Box::default(),
        },
    }
}

/// The schemas of the tools `grants` allows.
fn offer(grants: Grants) -> Box<[provider::ToolSpec]> {
    let granted = |family: Family| match family {
        Family::Inspect => grants.inspect,
        Family::Modify => grants.modify,
        Family::Shell => grants.shell,
    };
    TOOLS
        .iter()
        .filter(|(_, family, _)| granted(*family))
        .map(|(name, _, schema)| provider::ToolSpec {
            name: (*name).into(),
            description: b"A tool.".as_slice().into(),
            parameters: (*schema).into(),
        })
        .collect()
}

fn translate_message(message: agent::Message, namespace: &Namespace, maximum: u32) -> Result<provider::Message, Error> {
    let role = match message.role {
        agent::Role::User => provider::Role::User,
        agent::Role::Assistant => provider::Role::Assistant,
    };
    let parts = message.content.into_iter().map(|block| part(block, namespace, maximum)).collect::<Result<_, _>>()?;
    Ok(provider::Message { role, parts })
}

fn part(block: agent::Block, namespace: &Namespace, maximum: u32) -> Result<provider::Part, Error> {
    Ok(match block {
        agent::Block::Text { text, .. } | agent::Block::Refusal { text, .. } => provider::Part::Text { text },
        agent::Block::Opaque { bytes } => provider::Part::Opaque { bytes },
        // The call goes back as the LLM wrote it.
        agent::Block::ToolCall(agent::ToolCall { id, name, input, .. }) => {
            provider::Part::ToolCall { id, name, arguments: input }
        }
        agent::Block::ToolResult { id, result } => {
            let (output, is_error) = render_bounded(&result, namespace, maximum)?;
            provider::Part::ToolOutput { id, output, is_error }
        }
    })
}

fn completion(
    answer: provider::Answer,
    tickets: &mut Tickets,
    opener: u64,
    served: &[agent::Descriptor],
) -> agent::Completion {
    let stop = match answer.finish {
        provider::Finish::Stop => agent::Stop::EndTurn,
        provider::Finish::ToolCalls => agent::Stop::ToolUse,
        provider::Finish::Length => agent::Stop::MaxTokens,
        provider::Finish::ContentFilter => agent::Stop::Refusal,
    };
    let provider::Usage { input, cache_read, cache_write, output, reasoning } = answer.usage;
    let usage = agent::Usage {
        input_tokens: input,
        output_tokens: output,
        cache_read_tokens: cache_read,
        cache_write_tokens: cache_write,
        reasoning_tokens: reasoning,
    };
    let content = answer.parts.into_iter().map(|part| block(part, tickets, opener, served)).collect();
    agent::Completion { reasoning_dropped: Box::default(), content, stop, usage }
}

fn block(part: provider::Part, tickets: &mut Tickets, opener: u64, served: &[agent::Descriptor]) -> agent::Block {
    match part {
        provider::Part::Text { text } => agent::Block::Text { text, replay: None },
        provider::Part::Opaque { bytes } => agent::Block::Opaque { bytes },
        provider::Part::ToolCall { id, name, arguments } => {
            let call = match delegated(&name, served, tickets) {
                Some((tool, effect, descriptor)) => {
                    let ticket = tickets.issue(opener, Ticketed::Call { tool, arguments: arguments.clone() });
                    agent::Decoded::Delegated {
                        descriptor,
                        source: smith_domain_session::ToolSource::Run,
                        ticket,
                        effect,
                    }
                }
                None => decode(&name, &arguments),
            };
            agent::Block::ToolCall(agent::ToolCall { id, name, input: arguments, call, replay: None })
        }
        provider::Part::ToolOutput { .. } => unreachable!("the fake answers with text and tool calls"),
    }
}

/// The tool of `served` named `name`, if the call is to one of them.
fn delegated(name: &[u8], served: &[agent::Descriptor], tickets: &Tickets) -> Option<(&'static [u8], Effect, Token)> {
    served.iter().find_map(|descriptor| match tickets.resolve(descriptor.ticket) {
        Ticketed::Tool { name: tool, effect, .. } => (*tool == name).then_some((*tool, *effect, descriptor.ticket)),
        Ticketed::Call { .. } => panic!("a descriptor's ticket names a tool"),
    })
}

fn failure(error: provider::Error) -> agent::Failure {
    match error {
        provider::Error::Spent => agent::Failure::Exhausted { retry_after: None },
        provider::Error::Window { .. } => agent::Failure::RateLimited { retry_after: skein_lib::Duration::ZERO },
        provider::Error::Cut(_) => agent::Failure::Protocol,
        provider::Error::Overloaded => agent::Failure::Overloaded,
        provider::Error::RateLimited { retry_after } => agent::Failure::RateLimited { retry_after },
        provider::Error::Unavailable => agent::Failure::Unavailable,
        provider::Error::ContextTooLong => agent::Failure::ContextTooLong,
        provider::Error::SignedOut | provider::Error::Unauthorized => agent::Failure::Unauthorized,
        provider::Error::Exhausted { retry_after } => agent::Failure::Exhausted { retry_after },
        provider::Error::Refused => agent::Failure::Refused,
        provider::Error::Unsupported => agent::Failure::Unsupported,
        provider::Error::Unserved | provider::Error::InvalidRequest => agent::Failure::Invalid,
    }
}

/// The call the LLM made to the tool `name` with `arguments`, as the agent's
/// protocol layer decodes it: or what keeps it from being one.
#[must_use]
pub fn decode(name: &[u8], arguments: &[u8]) -> agent::Decoded {
    use smith_domain::llm;

    let client = skein_llm_kit::limits();
    let stated = smith_protocol_llm::Stated::from_tools(&smith_domain_tools_fixtures::SMALL, client.input)
        .expect("finite workspace fixture");
    let limits = smith_protocol_llm_fixtures::small(client, stated);
    let grants = Grants { inspect: true, modify: true, shell: true };
    match smith_protocol_llm::decode(name, arguments, grants, &[], llm::Asker::Main, &limits) {
        llm::Decoded::Owned { call } => agent::Decoded::Owned { call },
        llm::Decoded::Invalid { problem } => agent::Decoded::Invalid { descriptor: None, problem },
        llm::Decoded::Builtin { .. } | llm::Decoded::Served { .. } => unreachable!("only workspace tools are granted"),
    }
}

/// The text the LLM reads for what came of a call, and whether it failed.
#[must_use]
pub fn render(result: &agent::Returned) -> (Box<[u8]>, bool) {
    render_with_namespace(result, &Namespace::One)
}

/// Render a tool result in the conversation's fixed namespace at the calm fixture's bound.
#[must_use]
pub fn render_with_namespace(result: &agent::Returned, namespace: &Namespace) -> (Box<[u8]>, bool) {
    render_bounded(result, namespace, render_bound(&crate::Settings::calm(0).agent))
        .expect("scripted fixture result renders")
}

fn render_bounded(result: &agent::Returned, namespace: &Namespace, maximum: u32) -> Result<(Box<[u8]>, bool), Error> {
    Ok(match result {
        agent::Returned::Text { text, error, replay: _ } => (text.clone(), *error),
        agent::Returned::Owned { outcome } => return render_outcome(outcome, namespace, maximum),
        agent::Returned::Invalid { problem } => (format!("malformed call: {problem:?}").into_bytes().into(), true),
        agent::Returned::Withdrawn => (b"withdrawn".as_slice().into(), true),
        agent::Returned::NotRun { why: agent::NotRun::OutsideChoice { .. } } => {
            (b"not run: outside the narrowed choice".as_slice().into(), true)
        }
        agent::Returned::NotRun { why: agent::NotRun::AfterFailedWrite { call } } => {
            (format!("not run: after failed call {call}").into_bytes().into(), true)
        }
        agent::Returned::NotRun { why: agent::NotRun::Stopped } => {
            (b"not run: the answer stopped first".as_slice().into(), true)
        }
    })
}
