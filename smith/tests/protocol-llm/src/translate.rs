//! Fixture finish grammar for direct-client stories (smith/docs/design/protocol/llm.md, section 10).
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
    if name == b"sub_agent"
        && (asker == agent::Asker::Child || skein_lib::bytes::find(arguments, b"\"agents\"").is_some())
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
        let has = |word: &[u8]| skein_lib::bytes::find(arguments, word).is_some();
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
    let limits = smith_protocol_llm::fixture::small(skein_llm_world::limits(), crate::adapter::stated());
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
    let children: Box<[Item]> = if skein_lib::bytes::find(arguments, b"\"children\"").is_some() {
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
    let start = skein_lib::bytes::find(arguments, &prefix)? + prefix.len();
    let rest = &arguments[start..];
    let end = rest.iter().position(|byte| *byte == b'"')?;
    Some(rest[..end].into())
}

fn object(document: &[u8]) -> Option<()> {
    let length = u32::try_from(document.len()).ok()?;
    if document.len() > run::HostInput::CAPACITY {
        return None;
    }
    let limits = skein_json::tokenizer::Limits { depth: 32, string: length, number: length, chunk: 1024, length };
    let retained = skein_json::document::Limits { tokens: length, text: length };
    let parsed = skein_json::parse(document, &limits, &retained).ok()?;
    (parsed.token(0)?.kind == skein_json::Kind::ObjectStart).then_some(())
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
