//! Root assignments as the hub's records, with application items beside them.
//! Text fields are made UTF-8 before fitting (domain/agent/room.md, 2).
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_link::Grant;
use jig_domain_tasks as tasks;
use jig_protocol_engine_workers as kit;
use skein_lib::{List, Time, Token, Writer};
use temper_engine_domain as root;
use temper_protocol_worker_wire::{self as wire, v1::Record};

/// A worker's charter allowance where the engine doesn't know it: smith's
/// `run_bytes` at its floor, which the stock agent's `tool_payload` doesn't
/// raise (smith/docs/design/protocol/limits-derivation.md, 1.10;
/// temper/docs/design/domain/agent/tools.md).
pub const WORKER_CHARTER_BYTES: u64 = 65_536;

/// The bound a charter is fitted to: the hub's, and no more than its
/// worker takes (temper/docs/design/domain/agent/tools.md).
#[must_use]
pub fn charter_bound(hub: u64, agent: Option<u64>) -> u64 {
    hub.min(agent.unwrap_or(WORKER_CHARTER_BYTES))
}

/// The checked channel limits, endpoint names and deployment identity.
#[derive(Debug)]
pub struct Encoding {
    pub limits: wire::v1::Limits,
    pub endpoints: smith_protocol_translate::Endpoints,
    pub deployment: [u8; 16],
    pub charter_bytes: u64,
    /// Canonical host feedback bound for answers recovered before a resumed start.
    pub reply_bytes: u32,
    pub forge_reads: temper_engine_smith::ForgeReadBounds,
}

/// A transcript tail as text alone. jig's tail copies the retained turns'
/// encoded bytes, so a long one reaches their binary headers, which Smith's
/// brief refuses as not UTF-8: keep what follows the last byte that is not
/// text, behind a cut marker, never longer than the tail given. jig's own fix
/// renders the turns as text.
#[expect(clippy::disallowed_methods, reason = "the worker protocol checks a tail's text before Smith's brief")]
#[must_use]
pub fn text_tail(tail: &[u8]) -> Box<[u8]> {
    const MARKER: &[u8] = b"[earlier bytes not text, cut]\n";
    let mut from = 0_usize;
    let mut at = 0_usize;
    while let Some(rest) = tail.get(at..)
        && !rest.is_empty()
    {
        let (valid, bad) = match ::core::str::from_utf8(rest) {
            Ok(valid) => (valid, 0),
            Err(problem) => {
                let (valid, rest) = rest.split_at_checked(problem.valid_up_to()).expect("valid prefix inside the rest");
                let valid = ::core::str::from_utf8(valid).expect("valid prefix");
                (valid, problem.error_len().unwrap_or(rest.len()))
            }
        };
        for (offset, char) in valid.char_indices() {
            let text = match char {
                '\n' | '\t' => true,
                other => !other.is_control(),
            };
            if !text {
                from = at.saturating_add(offset).saturating_add(char.len_utf8());
            }
        }
        at = at.saturating_add(valid.len());
        if bad != 0 {
            at = at.saturating_add(bad);
            from = at;
        }
    }
    let kept = tail.get(from..).expect("text tail");
    if from == 0 || tail.len() <= MARKER.len() {
        return kept.into();
    }
    let kept = ::core::str::from_utf8(kept).expect("text after the last non-text byte");
    let mut skip = kept.len().saturating_sub(tail.len().saturating_sub(MARKER.len()));
    while !kept.is_char_boundary(skip) {
        skip = skip.saturating_add(1);
    }
    let kept = kept.get(skip..).expect("a char boundary");
    let mut text = Writer::new(MARKER.len().saturating_add(kept.len()));
    text.put(MARKER).expect("marker within the tail's room");
    text.put(kept.as_bytes()).expect("tail within its room");
    text.finish()
}

/// An assignment and its application parts, or the field that cannot encode.
pub fn down(
    assignment: root::Assignment,
    forges: &crate::forges::Forges,
    encoding: &Encoding,
    values: &crate::values::Values,
    channel: Token,
    now: Time,
) -> Result<kit::Event, kit::Unencoded> {
    let items = match crate::translate::items(&assignment, forges, &values.tokens, &encoding.limits) {
        Ok(items) => items,
        Err(problem) => {
            return Err(kit::Unencoded { reason: Some(problem.reason), ..unencoded(problem.path.name()) });
        }
    };
    let mut parts = List::with_capacity(1);
    let workspace = match items {
        Some(items) => {
            let bytes = match items.encode(&encoding.limits) {
                Ok(bytes) => bytes,
                Err(problem) => {
                    return Err(kit::Unencoded { reason: Some(problem.reason), ..unencoded(problem.path.name()) });
                }
            };
            let token = Token::new(0);
            parts.push(jig_protocol_worker_wire::Part { token, bytes }).expect("one workspace part");
            Some(host::Workspace { workstream: assignment.workstream, items: token })
        }
        None => None,
    };
    let mut charter = temper_engine_smith::charter(&assignment);
    for (section, from) in charter.brief.sections.iter_mut().zip(&assignment.sections) {
        if from.kind == root::BriefKind::Core(jig_domain_brief::Core::TranscriptTail) {
            section.text = text_tail(&section.text);
        }
        make_text(&mut section.title);
        make_text(&mut section.text);
    }
    let charter = fitted_charter(charter, encoding)?;
    let identity = jig_protocol_charter::run_identity(&encoding.deployment, assignment.task, assignment.conversation);
    let grants = grants(&assignment.grants);
    let filled = values.grants(&grants, now);
    let answered = recovered_answers(&assignment, encoding);
    Ok(kit::Event::Down {
        channel,
        down: host::LinkDown::Assign {
            assignment: host::Assignment {
                parts: assignment.parts,
                tell_parts: false,
                messages: messages(assignment.task, &assignment.messages),
                assignment: host::RunAssignment {
                    identity,
                    live: assignment.live,
                    run: Token::new(assignment.task),
                    attempt: Token::new(assignment.attempt),
                    workspace,
                    save: assignment.run.policy.resume,
                    charter,
                    grants,
                },
                turns: assignment.transcript,
                answered,
            },
        },
        parts: parts.into_boxed(),
        values: filled,
    })
}
/// A job's assignment and its forge workspace as the worker link carries them
/// (temper/docs/design/domain/worker/jobs.md).
#[must_use]
pub fn job_down(
    job: host::JobAssignment,
    workspace: Option<&root::ForgeWorkspace>,
    forges: &crate::forges::Forges,
    encoding: &Encoding,
    values: &crate::values::Values,
    channel: Token,
    now: Time,
) -> Option<kit::Event> {
    let items = match workspace {
        Some(workspace) => {
            crate::translate::workspace_items(workspace, false, forges, &values.tokens, &encoding.limits).ok()?
        }
        None => None,
    };
    let parts: Box<[jig_protocol_worker_wire::Part]> = match items {
        Some(items) => Box::new([jig_protocol_worker_wire::Part {
            token: job.workspace.as_ref()?.items,
            bytes: items.encode(&encoding.limits).ok()?,
        }]),
        None => Box::new([]),
    };
    let values = values.grants(&job.grants, now);
    Some(jig_protocol_engine_workers::Event::Down {
        channel,
        down: jig_domain_host::LinkDown::AssignJob { job },
        parts,
        values,
    })
}

// The wire's section framing is two u32 lengths; a decoded Section owns two
// boxes, so its array cell covers that framing even when sections are omitted.
fn fitted_charter(mut charter: smith_domain_run::Charter, encoding: &Encoding) -> Result<Box<[u8]>, kit::Unencoded> {
    let bound = encoding.charter_bytes;
    if charter.owned_bytes().ok_or(unencoded("charter.ownership"))? <= bound {
        let encoded = encode_charter(&charter, encoding)?;
        if skein_lib::len_u64(encoded.len()) <= bound {
            return Ok(encoded);
        }
    }
    let brief = ::core::mem::replace(&mut charter.brief.sections, Box::new([]));
    let fixed_owned = charter.owned_bytes().ok_or(unencoded("charter.ownership"))?;
    if fixed_owned > bound {
        return Err(charter_size("charter.fixed_owned", fixed_owned, encoding));
    }
    let fixed_encoded = encode_charter(&charter, encoding)?;
    let fixed_encoded = skein_lib::len_u64(fixed_encoded.len());
    if fixed_encoded > bound {
        return Err(charter_size("charter.fixed_encoded", fixed_encoded, encoding));
    }
    charter.brief.sections = brief;
    let owned = bound.saturating_sub(fixed_encoded.saturating_sub(fixed_owned));
    temper_protocol_brief::fit_brief_bytes(&mut charter, owned);
    let owned = charter.owned_bytes().ok_or(unencoded("charter.ownership"))?;
    if owned > bound {
        return Err(charter_size("charter.owned", owned, encoding));
    }
    let encoded = encode_charter(&charter, encoding)?;
    let bytes = skein_lib::len_u64(encoded.len());
    if bytes > bound {
        return Err(charter_size("charter.encoded", bytes, encoding));
    }
    Ok(encoded)
}

// temper/docs/design/domain/agent/room.md, 3: only measured refusals carry figures.
fn unencoded(field: &'static str) -> kit::Unencoded {
    kit::Unencoded { field, reason: None, figure: None, bound: None, ceiling: None }
}

fn charter_size(field: &'static str, figure: u64, encoding: &Encoding) -> kit::Unencoded {
    kit::Unencoded {
        figure: Some(figure),
        bound: Some(encoding.charter_bytes),
        ceiling: Some(u64::from(jig_protocol_worker_wire::v22::CEILINGS.assignment_charter)),
        ..unencoded(field)
    }
}

fn encode_charter(charter: &smith_domain_run::Charter, encoding: &Encoding) -> Result<Box<[u8]>, kit::Unencoded> {
    use smith_protocol_translate::Error;
    let error = match smith_protocol_translate::encode_charter(charter, &smith_charter::CEILINGS, &encoding.endpoints) {
        Ok(encoded) => return Ok(encoded),
        Err(error) => error,
    };
    let field = match error {
        Error::CharterBody(problem) => {
            return Err(kit::Unencoded { reason: Some(problem.reason), ..unencoded(problem.path.name()) });
        }
        Error::Endpoints => "charter.endpoints",
        Error::Capacity => "charter.capacity",
        Error::Write(_) => "charter.write",
        Error::Codec(_) => "charter.codec",
        Error::Channel(_) => "charter.channel",
        Error::InvalidSavedAnswer => "charter.saved_answer",
        Error::InvalidResult => "charter.result",
        Error::Transcript(_) | Error::TranscriptBody(_) => "charter.transcript",
        Error::Order => "charter.order",
        Error::Calls => "charter.calls",
        Error::Window => "charter.window",
        Error::Grants => "charter.grants",
        Error::ChannelBody(_) => "charter.channel_body",
        Error::Frame(_) => "charter.frame",
    };
    Err(unencoded(field))
}

fn make_text(text: &mut Box<[u8]>) {
    if !skein_lib::bytes::is_utf8(text) {
        *text = skein_lib::bytes::utf8_replaced(text);
    }
}

fn messages(task: u64, words: &[tasks::Word]) -> Box<[host::StartMessage]> {
    let mut messages = List::with_capacity(u32::try_from(words.len()).expect("bounded messages"));
    for word in words {
        let mut words = root::inbox_words(task, word);
        make_text(&mut words);
        messages
            .push(host::StartMessage {
                name: Token::new(word.number),
                sender: root::inbox_sender(word),
                words,
                finish_requested: tasks::finish_requested(word),
            })
            .expect("bounded messages");
    }
    messages.into_boxed()
}
fn grants(grants: &[Grant]) -> Box<[Grant]> {
    Box::from(grants)
}
fn recovered_answers(assignment: &root::Assignment, encoding: &Encoding) -> Box<[host::AnsweredCall]> {
    let capacity = assignment.answered.len().saturating_add(assignment.settled.len());
    let mut calls = List::with_capacity(u32::try_from(capacity).expect("bounded retained calls"));
    for settled in &assignment.settled {
        calls.push(settled.clone()).expect("retained calls");
    }
    for record in &assignment.answered {
        let mut known = false;
        for settled in &assignment.settled {
            known |= settled.key == record.key;
        }
        if known {
            continue;
        }
        let call = match &record.settled {
            Some(call) => call.clone(),
            None => {
                let Some(tool) = recovered_tool(assignment, record.key, encoding) else { continue };
                let bound = encoding.reply_bytes.saturating_sub(u32::try_from(tool.len()).expect("bounded tool"));
                let (body, error) =
                    temper_engine_smith::answer_for_tool(&record.answer, &tool, bound, encoding.forge_reads)
                        .into_parts();
                core::SettledCall { serial: 0, tool, answer: core::SettledAnswer::Host { body, error } }
            }
        };
        calls.push(core::Settled { key: record.key, call }).expect("retained calls");
    }
    answered(calls.as_slice())
}

fn recovered_tool(assignment: &root::Assignment, key: core::CallKey, encoding: &Encoding) -> Option<Box<[u8]>> {
    for part in &assignment.parts {
        if part.attempt != key.attempt
            || part.turn != key.completion
            || part.thread != 0
            || part.kind != jig_domain_link::PartKind::Answer
        {
            continue;
        }
        let Ok(turn) =
            smith_protocol_translate::decode_part(&part.body, &smith_transcript::CEILINGS, &encoding.endpoints)
        else {
            continue;
        };
        let message = turn.messages.last()?;
        let block = message.content.get(usize::try_from(key.position).ok()?)?;
        let call = block.tool_call()?;
        return Some(call.name.clone());
    }
    None
}

fn answered(calls: &[core::Settled]) -> Box<[host::AnsweredCall]> {
    let mut answered = List::with_capacity(u32::try_from(calls.len()).expect("bounded answered calls"));
    for core::Settled { key, call } in calls {
        let answer = match &call.answer {
            core::SettledAnswer::Host { error, body } => {
                host::SettledAnswer::Host { error: *error, body: body.clone() }
            }
            core::SettledAnswer::Delivery { outcome, evidence } => host::SettledAnswer::Delivery {
                outcome: match outcome {
                    core::DeliveryOutcome::Delivered => host::DeliveryOutcome::Delivered,
                    core::DeliveryOutcome::Nothing => host::DeliveryOutcome::Nothing,
                    core::DeliveryOutcome::Stale => host::DeliveryOutcome::Stale,
                    core::DeliveryOutcome::Refused | core::DeliveryOutcome::Invalid => host::DeliveryOutcome::Refused,
                    core::DeliveryOutcome::Failed => host::DeliveryOutcome::Failed,
                },
                evidence: evidence.clone(),
            },
        };
        answered
            .push(host::AnsweredCall {
                name: jig_protocol_charter::call_name(&jig_protocol_charter::CallKey {
                    activation: key.attempt,
                    completion: key.completion,
                    position: key.position,
                }),
                tool: skein_lib::bytes::utf8_replaced(&call.tool),
                answer,
            })
            .expect("bounded answered calls");
    }
    answered.into_boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resumed_start_carries_a_decided_effect_the_lost_worker_never_relayed() {
        use crate::tests::fixtures;
        use smith_domain_session::{llm, record};
        let encoding = fixtures::encoding(&fixtures::engine());
        let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
        assignment.attempt = 10;
        let key = core::CallKey { task: assignment.task, attempt: 6, completion: 2, position: 1 };
        let turn = record::Turn {
            version: record::VERSION,
            kind: record::Kind::Ordinary,
            endpoint: Some(llm::Endpoint(assignment.run.policy.model.endpoint)),
            model: assignment.run.policy.model.name.clone(),
            effort: None,
            dialect: assignment.run.policy.model.dialect,
            sequence: 2,
            usage: llm::Usage::ZERO,
            spent: 0,
            jobs: 0,
            messages: Box::new([llm::Message {
                role: llm::Role::Assistant,
                content: Box::new([
                    llm::Block::Text { text: b"make issue".as_slice().into(), replay: None },
                    llm::Block::ToolCall(llm::ToolCall {
                        id: b"call".as_slice().into(),
                        name: b"effect_forge".as_slice().into(),
                        input: b"{}".as_slice().into(),
                        call: llm::Decoded::Historical,
                        replay: None,
                    }),
                ]),
            }]),
        };
        assignment.parts = Box::new([jig_domain_link::KeptPart {
            attempt: 6,
            turn: 2,
            thread: 0,
            call: 0,
            kind: jig_domain_link::PartKind::Answer,
            body: smith_protocol_translate::encode_part(&turn, &smith_transcript::CEILINGS, &encoding.endpoints)
                .unwrap(),
        }]);
        assignment.answered = Box::new([root::CallRecord {
            key,
            answer: root::CallAnswer::ForgeEffect {
                entry: 9,
                deadline: skein_lib::Wall::EPOCH,
                outcome: Some(skein_forge_client::Outcome::Made {
                    made: skein_forge_client::Made::Created(2),
                    found: true,
                }),
            },
            settled: None,
        }]);
        assert!(assignment.settled.is_empty());
        let event = down(
            assignment,
            &fixtures::forge_table(&fixtures::forges()),
            &encoding,
            &crate::values::Values::new(0, 0),
            Token::new(8),
            Time::ZERO,
        )
        .unwrap();
        let kit::Event::Down { down: host::LinkDown::Assign { assignment }, .. } = event else { panic!("start") };
        assert_eq!(assignment.answered.len(), 1);
        assert_eq!(
            assignment.answered[0].name,
            jig_protocol_charter::call_name(&jig_protocol_charter::CallKey {
                activation: 6,
                completion: 2,
                position: 1,
            })
        );
        assert_eq!(assignment.answered[0].tool.as_ref(), b"effect_forge");
        assert_eq!(
            assignment.answered[0].answer,
            host::SettledAnswer::Host { error: false, body: b"Forge effect 9: found 2".as_slice().into() }
        );
    }

    #[test]
    fn encoded_framing_fits_beside_the_fixed_charter() {
        let settings = crate::tests::fixtures::engine();
        let mut encoding = crate::tests::fixtures::encoding(&settings);
        let assignment =
            crate::tests::fixtures::assignment_with(crate::tests::fixtures::repository(7, b"acme", b"shop"));
        let mut charter = temper_engine_smith::charter(&assignment);
        charter.grants.host_tools = Box::new([]);
        charter.models = Box::new([]);
        charter.brief.sections = Box::new([smith_domain_run::Section {
            title: b"Pull request".as_slice().into(),
            text: Box::new([b'x'; 1024]),
        }]);
        let owned = charter.owned_bytes().unwrap();
        let encoded =
            smith_protocol_translate::encode_charter(&charter, &smith_charter::CEILINGS, &encoding.endpoints).unwrap();
        assert!(skein_lib::len_u64(encoded.len()) > owned);
        encoding.charter_bytes = owned;
        let fitted = fitted_charter(charter, &encoding).expect("wire overhead leaves room for a cut brief");
        assert!(skein_lib::len_u64(fitted.len()) <= owned);
        let decoded =
            smith_protocol_translate::decode_charter(&fitted, &smith_charter::CEILINGS, &encoding.endpoints).unwrap();
        assert!(decoded.owned_bytes().unwrap() <= owned);
        assert!(decoded.brief.sections[0].text.ends_with(b" bytes; whole: read_issues]\n"));
    }

    #[test]
    fn a_fixed_charters_ownership_names_its_figure() {
        let settings = crate::tests::fixtures::engine();
        let mut encoding = crate::tests::fixtures::encoding(&settings);
        let assignment =
            crate::tests::fixtures::assignment_with(crate::tests::fixtures::repository(7, b"acme", b"shop"));
        let mut charter = temper_engine_smith::charter(&assignment);
        charter.brief.sections = Box::new([]);
        let owned = charter.owned_bytes().unwrap();
        charter.brief.sections =
            Box::new([smith_domain_run::Section { title: b"Task".as_slice().into(), text: Box::new([b'x'; 1024]) }]);
        encoding.charter_bytes = owned.checked_sub(1).unwrap();
        assert_eq!(
            fitted_charter(charter, &encoding),
            Err(kit::Unencoded {
                field: "charter.fixed_owned",
                reason: None,
                figure: Some(owned),
                bound: Some(encoding.charter_bytes),
                ceiling: Some(u64::from(jig_protocol_worker_wire::v22::CEILINGS.assignment_charter)),
            })
        );
    }

    fn declaration_bytes(total: usize, index: usize) -> usize {
        total.checked_div(42).unwrap().checked_add(usize::from(index < total.checked_rem(42).unwrap())).unwrap()
    }

    #[test]
    fn a_first_goals_fixed_ownership_exceeds_its_room() {
        // temper/docs/design/draft/archive/tools-by-role.md, 9: first wake joins the first three parts.
        let mut settings = crate::tests::fixtures::engine();
        let mut subagent = crate::tests::fixtures::small_model();
        subagent.number = 2;
        subagent.model.name = b"small-subagent".as_slice().into();
        settings.jig.llm.models = Box::new([crate::tests::fixtures::small_model(), subagent]);
        let mut assignment =
            crate::tests::fixtures::assignment_with(crate::tests::fixtures::repository(7, b"acme", b"shop"));
        assignment.charter = 5;
        for charter in &settings.jig.charters {
            if charter.number == 5 {
                assignment.run.policy = charter.policy.clone();
            }
        }
        assignment.run.policy.alternatives = Box::new([settings.jig.llm.models[1].model.clone()]);
        assignment.run.sources = Box::new([Some(1), Some(1)]);
        assignment.run.authority.tools = tasks::Tools(settings.jig.policies[0].policy.ceiling.tools.0);
        let shared = Box::new([b's'; 9_910]);
        let parts: [Box<[u8]>; 4] =
            [Box::new([b'a'; 11_031]), Box::new([b'b'; 3_655]), Box::new([b'c'; 1_597]), Box::new([b'd'; 649])];
        let mut size = shared.len();
        for part in parts.iter().take(3) {
            size = size.checked_add(2).unwrap().checked_add(part.len()).unwrap();
        }
        let mut instructions = Writer::new(size);
        instructions.put(shared.as_slice()).unwrap();
        for part in parts.iter().take(3) {
            instructions.put(b"\n\n").unwrap();
            instructions.put(part).unwrap();
        }
        assignment.run.policy.instructions = instructions.finish();
        assignment.run.policy.modify = true;
        assignment.workspace.repositories[0].delivers = true;
        assignment.sections = Box::new([root::BriefSection {
            kind: root::BriefKind::Core(jig_domain_brief::Core::Task),
            body: root::BriefBody::Text(Box::new([b't'; 2_342])),
        }]);
        let mut charter = temper_engine_smith::charter(&assignment);
        // The draft's declaration bytes are a fixture, independent of the live registry.
        let mut tools = List::with_capacity(42);
        for index in 0_usize..42 {
            let number = skein_lib::Decimal::of(u64::try_from(index).unwrap());
            let mut name = Writer::new(declaration_bytes(406, index));
            name.put(b"tool").unwrap();
            name.put(number.as_bytes()).unwrap();
            for _ in 0..name.room() {
                name.put(b"x").unwrap();
            }
            let mut description = Writer::new(declaration_bytes(4_746, index));
            for _ in 0..description.room() {
                description.put(b"d").unwrap();
            }
            let mut schema = Writer::new(declaration_bytes(31_360, index));
            schema.put(b"{}").unwrap();
            for _ in 0..schema.room() {
                schema.put(b" ").unwrap();
            }
            tools
                .push(smith_domain_run::HostTool {
                    name: name.finish(),
                    description: description.finish(),
                    schema: schema.finish(),
                    reserve: false,
                    effect: smith_domain_run::HostEffect::Read,
                    timeout: skein_lib::Duration::from_secs(1),
                })
                .unwrap();
        }
        charter.grants.host_tools = tools.into_boxed();
        assert_eq!(charter.grants.host_tools.len(), 42);
        let (mut names, mut descriptions, mut schemas) = (0_usize, 0_usize, 0_usize);
        for tool in &charter.grants.host_tools {
            names = names.checked_add(tool.name.len()).unwrap();
            descriptions = descriptions.checked_add(tool.description.len()).unwrap();
            schemas = schemas.checked_add(tool.schema.len()).unwrap();
        }
        assert_eq!((names, descriptions, schemas), (406, 4_746, 31_360));
        assert_eq!(charter.models.len(), 1);
        assert!(charter.grants.deliver.is_some());
        let brief = ::core::mem::replace(&mut charter.brief.sections, Box::new([]));
        let owned = charter.owned_bytes().unwrap();
        charter.brief.sections = brief;
        let encoding = crate::tests::fixtures::encoding(&settings);
        assert_eq!(encoding.charter_bytes, 65_536);
        assert!(owned > encoding.charter_bytes);
        assert_eq!(
            fitted_charter(charter, &encoding),
            Err(kit::Unencoded {
                field: "charter.fixed_owned",
                reason: None,
                figure: Some(owned),
                bound: Some(65_536),
                ceiling: Some(67_108_864),
            })
        );
    }

    #[test]
    fn a_fixed_charters_encoding_names_its_figure() {
        let settings = crate::tests::fixtures::engine();
        let mut encoding = crate::tests::fixtures::encoding(&settings);
        let assignment =
            crate::tests::fixtures::assignment_with(crate::tests::fixtures::repository(7, b"acme", b"shop"));
        let mut charter = temper_engine_smith::charter(&assignment);
        charter.grants.host_tools = Box::new([]);
        charter.models = Box::new([]);
        charter.brief.sections = Box::new([]);
        let encoded =
            smith_protocol_translate::encode_charter(&charter, &smith_charter::CEILINGS, &encoding.endpoints).unwrap();
        let bytes = skein_lib::len_u64(encoded.len());
        encoding.charter_bytes = bytes.checked_sub(1).unwrap();
        assert!(charter.owned_bytes().unwrap() <= encoding.charter_bytes);
        let mut with_brief =
            smith_protocol_translate::decode_charter(&encoded, &smith_charter::CEILINGS, &encoding.endpoints).unwrap();
        with_brief.brief.sections =
            Box::new([smith_domain_run::Section { title: b"Task".as_slice().into(), text: Box::new([b'x'; 1024]) }]);
        assert_eq!(
            fitted_charter(with_brief, &encoding),
            Err(kit::Unencoded {
                field: "charter.fixed_encoded",
                reason: None,
                figure: Some(bytes),
                bound: Some(encoding.charter_bytes),
                ceiling: Some(u64::from(jig_protocol_worker_wire::v22::CEILINGS.assignment_charter)),
            })
        );
        encoding.charter_bytes = bytes;
        assert_eq!(fitted_charter(charter, &encoding), Ok(encoded));
    }

    #[test]
    fn a_run_identity_keeps_its_bytes() {
        use crate::tests::fixtures;

        let mut encoding = fixtures::encoding(&fixtures::engine());
        encoding.deployment = [3; 16];
        let mut assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
        assignment.task = 17;
        assignment.conversation = 29;
        let event = down(
            assignment,
            &fixtures::forge_table(&fixtures::forges()),
            &encoding,
            &crate::values::Values::new(0, 0),
            Token::new(8),
            Time::ZERO,
        )
        .expect("assignment encodes");
        let kit::Event::Down { down: host::LinkDown::Assign { assignment }, .. } = event else {
            panic!("assignment event")
        };
        assert_eq!(
            assignment.assignment.identity,
            [252, 129, 193, 120, 217, 168, 14, 238, 123, 174, 89, 65, 38, 144, 139, 54]
        );
    }

    #[test]
    fn a_host_start_carries_the_question_and_answer_references() {
        let words = [
            tasks::Word {
                number: 4,
                from: tasks::Party::Task(2),
                kind: tasks::MessageKind::Question,
                words: b"The plan?".as_slice().into(),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            },
            tasks::Word {
                number: 9,
                from: tasks::Party::Person(1),
                kind: tasks::MessageKind::Answer { question: 4 },
                words: b"Approved.".as_slice().into(),
                at: skein_lib::Wall::EPOCH,
                hits: 1,
                eligible: true,
            },
        ];
        let messages = messages(1, &words);
        assert_eq!(
            messages[0].sender.as_ref(),
            b"2 asks (question 4; answer with message target=2, form=answer, question=4)"
        );
        assert_eq!(messages[0].words.as_ref(), b"The plan?");
        assert_eq!(messages[1].sender.as_ref(), b"1 answers question 4");
        assert_eq!(messages[1].words.as_ref(), b"Approved.");
    }

    #[test]
    fn a_starting_run_hears_its_met_dependency_by_name() {
        // A run resumed by its last dependency's end hears the Met notice that woke it (jig's
        // domain/tasks/messages.md, section 1), as a live run does.
        let met = tasks::Word {
            number: 2,
            from: tasks::Party::Task(2),
            kind: tasks::MessageKind::Met { dependency: 2, removed: false },
            words: Box::new([]),
            at: skein_lib::Wall::EPOCH,
            hits: 1,
            eligible: true,
        };
        let started = messages(1, &[met]);
        assert_eq!(started[0].sender.as_ref(), b"2");
        assert_eq!(started[0].words.as_ref(), b"T2 is done; the task can go on");
    }
}
