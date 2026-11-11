//! Root assignments as the hub's records, with application items beside them.
use alloc::boxed::Box;
use jig_domain_core as core;
use jig_domain_host as host;
use jig_domain_link::Grant;
use jig_domain_tasks as tasks;
use jig_protocol_engine_workers as kit;
use skein_lib::{List, Time, Token, Writer};
use temper_engine_domain as root;
use temper_protocol_worker_wire as wire;

/// A worker's charter allowance where the engine doesn't know it: smith's
/// `run_bytes` at its floor, which the stock agent's `tool_payload` doesn't
/// raise (smith/docs/design/protocol/limits.md, 3.10;
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
}

/// A transcript tail as text alone. jig's tail copies the retained turns'
/// encoded bytes, so a long one reaches their binary headers, which Smith's
/// brief refuses as not UTF-8: keep what follows the last byte that is not
/// text, behind a cut marker, never longer than the tail given. jig's own fix
/// renders the turns as text.
#[expect(clippy::disallowed_methods, reason = "the worker protocol checks a tail's text before Smith's brief")]
pub(crate) fn text_tail(tail: &[u8]) -> Box<[u8]> {
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

/// An assignment and its application parts, or none when it cannot encode.
#[must_use]
pub fn down(
    assignment: root::Assignment,
    forges: &crate::forges::Forges,
    encoding: &Encoding,
    values: &crate::values::Values,
    channel: Token,
    now: Time,
) -> Option<kit::Event> {
    let items = crate::translate::items(&assignment, forges, &values.tokens, &encoding.limits).ok()?;
    let mut charter = temper_engine_smith::charter(&assignment);
    for (section, from) in charter.brief.sections.iter_mut().zip(&assignment.sections) {
        if from.kind == root::BriefKind::Core(jig_domain_brief::Core::TranscriptTail) {
            section.text = text_tail(&section.text);
        }
    }
    let charter = fitted_charter(charter, encoding)?;
    let identity = jig_protocol_charter::run_identity(&encoding.deployment, assignment.task, assignment.conversation);
    let mut parts = List::with_capacity(1);
    let workspace = match items {
        Some(items) => {
            let token = Token::new(0);
            parts
                .push(jig_protocol_worker_wire::Part { token, bytes: wire::items(&items) })
                .expect("one workspace part");
            Some(host::Workspace { workstream: assignment.workstream, items: token })
        }
        None => None,
    };
    let grants = grants(&assignment.grants);
    let filled = values.grants(&grants, now);
    Some(kit::Event::Down {
        channel,
        down: host::LinkDown::Assign {
            assignment: host::Assignment {
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
                answered: answered(&assignment.settled),
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
            bytes: temper_protocol_worker_wire::items(&items),
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
fn fitted_charter(mut charter: smith_domain_run::Charter, encoding: &Encoding) -> Option<Box<[u8]>> {
    let bound = encoding.charter_bytes;
    if charter.owned_bytes()? <= bound {
        let encoded =
            smith_protocol_translate::encode_charter(&charter, &smith_charter::CEILINGS, &encoding.endpoints).ok()?;
        if skein_lib::len_u64(encoded.len()) <= bound {
            return Some(encoded);
        }
    }
    let brief = ::core::mem::replace(&mut charter.brief.sections, Box::new([]));
    let fixed_owned = charter.owned_bytes()?;
    let fixed_encoded =
        smith_protocol_translate::encode_charter(&charter, &smith_charter::CEILINGS, &encoding.endpoints).ok()?;
    let fixed_encoded = skein_lib::len_u64(fixed_encoded.len());
    if fixed_owned > bound || fixed_encoded > bound {
        return None;
    }
    charter.brief.sections = brief;
    let owned = bound.saturating_sub(fixed_encoded.saturating_sub(fixed_owned));
    temper_protocol_brief::fit_brief_bytes(&mut charter, owned);
    let encoded =
        smith_protocol_translate::encode_charter(&charter, &smith_charter::CEILINGS, &encoding.endpoints).ok()?;
    if skein_lib::len_u64(encoded.len()) > bound || charter.owned_bytes()? > bound {
        return None;
    }
    Some(encoded)
}

fn messages(task: u64, words: &[tasks::Word]) -> Box<[host::StartMessage]> {
    let mut messages = List::with_capacity(u32::try_from(words.len()).expect("bounded messages"));
    for word in words {
        messages
            .push(host::StartMessage {
                name: Token::new(word.number),
                sender: root::inbox_sender(word),
                words: root::inbox_words(task, word),
                finish_requested: word.finish_requested(),
            })
            .expect("bounded messages");
    }
    messages.into_boxed()
}
fn grants(grants: &[Grant]) -> Box<[Grant]> {
    Box::from(grants)
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
                tool: call.tool.clone(),
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
        assert!(decoded.brief.sections[0].text.ends_with(b" bytes; whole: read_pull/read_pull_files]\n"));
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
        assert_eq!(messages[0].sender.as_ref(), b"2 asks (question 4)");
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
