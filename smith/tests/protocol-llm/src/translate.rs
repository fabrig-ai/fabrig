//! Fixture finish grammar for direct-client stories (protocol/llm.md, section 10).
use crate::adapter;
use smith_domain::{llm as agent, run, tools};

pub(crate) fn decode(
    name: &[u8],
    arguments: &[u8],
    grants: tools::Grants,
    served: &[agent::Served],
    asker: agent::Asker,
) -> agent::Decoded {
    let invalid = || agent::Decoded::Invalid { problem: agent::Problem::UnknownTool };
    if name == b"sub_agent" && (asker == agent::Asker::Child || arguments.windows(8).any(|part| part == b"\"agents\""))
    {
        return agent::Decoded::Invalid { problem: agent::Problem::Nesting };
    }
    for tool in served {
        match tool {
            agent::Served::Host(tool) if tool.name.as_ref() == name => {
                return object(arguments).and_then(|()| run::HostInput::attested(arguments.into())).map_or_else(
                    invalid,
                    |input| agent::Decoded::Served {
                        ask: run::Ask::Host { tool: name.into(), effect: tool.effect, input },
                    },
                );
            }
            agent::Served::Host(_)
            | agent::Served::Deliver
            | agent::Served::Finish
            | agent::Served::SubAgent
            | agent::Served::Wait => {}
        }
    }
    if name == b"deliver" && served.contains(&agent::Served::Deliver) {
        let Some(value) = field(arguments, b"ticket") else {
            return invalid();
        };
        return agent::Decoded::Served {
            ask: run::Ask::Deliver {
                change: run::outcome::Change {
                    fields: Box::new([run::outcome::Field { name: b"ticket".as_slice().into(), value }]),
                },
            },
        };
    }
    if name == b"finish" && served.contains(&agent::Served::Finish) {
        return finish(arguments)
            .map_or_else(invalid, |outcome| agent::Decoded::Served { ask: run::Ask::Finish { outcome } });
    }
    if name == b"wait" && served.contains(&agent::Served::Wait) {
        let no_arguments: Vec<u8> = arguments.iter().copied().filter(|byte| !byte.is_ascii_whitespace()).collect();
        return if no_arguments == b"{}" { agent::Decoded::Served { ask: run::Ask::Wait } } else { invalid() };
    }
    if name == b"sub_agent" && served.contains(&agent::Served::SubAgent) {
        let Some(brief) = field(arguments, b"brief") else { return invalid() };
        let has = |word: &[u8]| arguments.windows(word.len()).any(|part| part == word);
        let families = run::charter::Families {
            tools: run::charter::Tools {
                inspect: has(b"\"inspect\""),
                modify: has(b"\"modify\""),
                shell: has(b"\"shell\""),
            },

            agents: false,
        };
        return agent::Decoded::Served {
            ask: run::Ask::SubAgent { brief, families, llm: field(arguments, b"llm"), share: run::Share::UNASKED },
        };
    }
    let limits = adapter::Limits {
        client: skein_llm_world::limits(),
        tool_bytes: 32768,
        history_bytes: u64::from(skein_llm_world::limits().request),
        rendered_result: skein_llm_world::limits().strings,
        stated: crate::adapter::stated(),
    };
    adapter::decode(name, arguments, grants, served, asker, &limits)
}

fn finish(arguments: &[u8]) -> Option<run::outcome::Declared> {
    use run::outcome::{Change, Declared, DeclaredFailure, Field, Item, Report, Verdict};
    if !arguments.starts_with(b"{") || !arguments.ends_with(b"}") {
        return None;
    }
    if let Some(text) = field(arguments, b"report") {
        let fields: Box<[Field]> = match field(arguments, b"source") {
            Some(value) => Box::new([Field { name: b"source".as_slice().into(), value }]),
            None => Box::new([]),
        };
        return Some(Declared::Report(Report { text, fields }));
    }
    if let Some(reason) = field(arguments, b"failure") {
        let fields: Box<[Field]> = match field(arguments, b"cause") {
            Some(value) => Box::new([Field { name: b"cause".as_slice().into(), value }]),
            None => Box::new([]),
        };
        return Some(Declared::Failure(DeclaredFailure { reason, fields }));
    }
    let body = field(arguments, b"body")?;
    if let Some(title) = field(arguments, b"title") {
        return Some(Declared::Change(Change {
            fields: Box::new([
                smith_domain::run::outcome::Field { name: b"title".as_slice().into(), value: title },
                smith_domain::run::outcome::Field { name: b"body".as_slice().into(), value: body },
            ]),
        }));
    }
    let name = field(arguments, b"verdict")?;
    let children: Box<[Item]> = if arguments.windows(10).any(|part| part == b"\"children\"") {
        let child = Item {
            kind: field(arguments, b"kind")?,
            fields: Box::new([
                Field { name: b"path".as_slice().into(), value: field(arguments, b"path")? },
                Field { name: b"body".as_slice().into(), value: b"43".as_slice().into() },
            ]),
        };
        Box::new([child])
    } else {
        Box::new([])
    };
    Some(Declared::Verdict(Verdict { name, text: body, items: children, fields: Box::new([]) }))
}

/// The fixture scripts use unescaped string fields. This recognizes that finite
/// fixture language, never arbitrary JSON or production provider documents.
fn field(arguments: &[u8], name: &[u8]) -> Option<Box<[u8]>> {
    let prefix = [b"\"", name, b"\":\""].concat();
    let start = arguments.windows(prefix.len()).position(|part| part == prefix)? + prefix.len();
    let rest = &arguments[start..];
    let end = rest.iter().position(|byte| *byte == b'"')?;
    Some(rest[..end].into())
}

fn object(document: &[u8]) -> Option<()> {
    use skein_json::{Token, tokenizer as json};
    use skein_lib::{
        Env, Intake, Queue, Time, Wall,
        stream::{Down, Up},
    };
    let length = u32::try_from(document.len()).ok()?;
    if document.len() > run::HostInput::CAPACITY {
        return None;
    }
    let limits = json::Limits { depth: 32, string: length, number: length, chunk: 1024, length };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut tokenizer = json::Tokenizer::new(&limits);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut intake = Intake::with_capacity(length.max(json::largest_demand(&limits)));
    intake.append(document).ok()?;
    let mut demand = None;
    let mut first = true;
    for _ in 0..document.len().checked_mul(4)?.checked_add(8)? {
        match demand.take() {
            None => json::down(&mut tokenizer, &env, json::Request::Next, &mut above, &mut below),
            Some(read) => {
                let event = intake.meet(read).map_or(Up::End, Up::Bytes);
                json::up(&mut tokenizer, &env, event, &mut above, &mut below);
            }
        }
        match below.pop() {
            Some(Down::Demand { read, room: 0 }) => demand = Some(read),
            None => {}
            Some(Down::Demand { .. } | Down::Send(_) | Down::Finish) => {
                panic!("JSON tokenizer only demands bounded reads")
            }
        }
        match above.pop() {
            Some(json::Event::Token(token)) => {
                if first && token != Token::ObjectStart {
                    return None;
                }
                first = false;
            }
            Some(json::Event::Done) => return (!first).then_some(()),
            Some(json::Event::Failed(_)) => return None,
            Some(json::Event::Long(_) | json::Event::Skipped(_)) => {
                unreachable!("fixture decoder requests only Next tokens")
            }
            Some(json::Event::Closed) => panic!("parser is not closed by this translator"),
            None => {}
        }
        assert!(above.is_empty() && below.is_empty());
    }
    panic!("bounded JSON completes within linear steps");
}

#[cfg(test)]
mod host_tests {
    use super::*;
    #[test]
    fn only_a_complete_json_object_is_attested_and_bytes_are_opaque() {
        let tool = agent::Served::Host(run::HostTool {
            name: b"opaque".as_slice().into(),
            description: b"Opaque operation".as_slice().into(),
            schema: b"{}".as_slice().into(),
            effect: run::HostEffect::Write,
            timeout: skein_lib::Duration::from_secs(1),
        });
        let granted = tools::Grants { inspect: false, modify: false, shell: false };
        let raw = br#" {"nested":[1,true,{"unchanged":"\u0041"}],"policy":"unknown"} "#;
        let decoded = decode(b"opaque", raw, granted, std::slice::from_ref(&tool), agent::Asker::Main);
        let agent::Decoded::Served { ask: run::Ask::Host { input, effect, .. } } = decoded else {
            panic!("valid opaque object is relayed");
        };
        assert_eq!(input.bytes(), raw);
        assert_eq!(effect, run::HostEffect::Write);
        for malformed in [b"{bad}".as_slice(), b"{}{}", b"[]", b"{\"key\":}", b"{\"key\":NaN}", b"{\"key\":\"\xff\"}"] {
            assert!(matches!(
                decode(b"opaque", malformed, granted, std::slice::from_ref(&tool), agent::Asker::Main),
                agent::Decoded::Invalid { .. }
            ));
        }
    }
}
