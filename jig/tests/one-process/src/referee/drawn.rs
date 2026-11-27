//! Evidence from the fake provider's last conversation and the final durable turn
//! (jig/docs/design/protocol/service-testing.md, 1).
use crate::fixtures::drawn::{DELEGATE, NEVER_ISSUED, ROUNDS};
use jig_channel_kit::world::store::Store;
use jig_domain_core::{CoreRecord, Record};
use skein_fake_llm_domain::api::{Part, Role};
use skein_fake_peers::llm::Observation;
use smith_shapes::{Answer, Call, Class, Reached, Reference, Sample, Tool};
use std::collections::{BTreeMap, BTreeSet};

enum Reply {
    Sent,
    Subscribed,
    MissingReference,
    SubscriptionRefused,
    QuestionRefused,
    Unrecognized,
}

const DELEGATED: &[u8] = b"Delegated tasks: 2 at spec revision 0";
const SUBSCRIPTION_REFUSAL: &[u8] =
    b"Refused: the target must be a task this one references or a live task of its project its tree's requester reads";

pub(crate) fn check(observations: &[Observation], store: &Store) -> Result<(), String> {
    let mut queries = observations.iter().filter_map(|observation| match observation {
        Observation::Query { query, .. } => Some(query),
        _ => None,
    });
    let last = queries.next_back().ok_or("drawn chat never reached the model")?;
    let mut outputs = BTreeMap::new();
    for part in last.messages.iter().flat_map(|message| &message.parts) {
        if let Part::ToolOutput { id, output, .. } = part {
            let entry = outputs.entry(id.as_ref()).or_insert((output.as_ref(), 0));
            entry.1 += 1;
        }
    }
    // The last query asks for the scripted finish. Each preceding tool answer
    // appears in a message of its own before the matching outputs.
    let batches =
        last.messages.iter().filter(|message| message.role == Role::Assistant).map(|message| {
            message.parts.iter().filter(|part| matches!(part, Part::ToolCall { .. })).collect::<Vec<_>>()
        });
    let cancel = ROUNDS + 1;
    let mut evidence = Evidence::default();
    let mut completions = Vec::new();
    for (batch, calls) in batches.enumerate() {
        let mut completion = Vec::new();
        let script = match batch {
            0 => Some((b"delegate".as_slice(), DELEGATED)),
            index if index == cancel => Some((b"cancel".as_slice(), b"Done".as_slice())),
            _ => None,
        };
        if calls.len() != 1 {
            evidence.problems.push(format!("round {batch} returned {} calls instead of one", calls.len()));
        }
        for call in calls {
            let Part::ToolCall { id, name, arguments } = call else { unreachable!("selected only tool calls") };
            let answer = outputs.get(id.as_ref()).copied();
            if !evidence.ids.insert(id.as_ref()) || !matches!(answer, Some((_, 1))) {
                evidence
                    .problems
                    .push(format!("call {} did not occur and get answered exactly once", String::from_utf8_lossy(id)));
            }
            let output = answer.map_or(b"".as_slice(), |(output, _)| output);
            if let Some((tool, want)) = script {
                if name.as_ref() != tool || output != want {
                    evidence.problems.push(format!(
                        "script round {batch}: expected {} → {}, got {} → {}",
                        String::from_utf8_lossy(tool),
                        String::from_utf8_lossy(want),
                        String::from_utf8_lossy(name),
                        String::from_utf8_lossy(output)
                    ));
                }
                completion.push(Call { tool: Tool::Delegated, answer: Answer::Delegated, references: Vec::new() });
            } else {
                completion.push(evidence.drawn(batch > cancel, name, arguments, output));
            }
        }
        completions.push(completion);
    }
    let expected = 2 * (ROUNDS + 1);
    let earlier = queries.clone().count();
    if completions.len() != expected || earlier != expected {
        evidence.problems.push(format!(
            "expected {expected} tool rounds before finish; saw {} in the last query and {earlier} earlier queries",
            completions.len()
        ));
    }
    // A call echoed in an earlier query must still be in the conversation at finish.
    for part in queries.flat_map(|q| q.messages.iter().flat_map(|m| &m.parts)) {
        if let Part::ToolCall { id, .. } = part
            && !evidence.ids.contains(id.as_ref())
        {
            evidence.problems.push(format!("call {} disappeared before finish", String::from_utf8_lossy(id)));
        }
    }
    evidence.check(&completions)?;
    let mut reached = Reached::new();
    for completion in &completions {
        reached.completion(completion);
    }
    // No later query can echo the finish; take it from the durable transcript.
    let terminal = store
        .rows
        .values()
        .find_map(|row| match row {
            jig_test_domain::Record::Core(Record::Core(CoreRecord::Turn(turn)))
                if usize::try_from(turn.turn).ok() == Some(expected + 1) =>
            {
                Some(turn)
            }
            _ => None,
        })
        .ok_or("drawn chat did not record its final completion")?;
    let config = crate::fixtures::drawn::agent_configuration();
    let turn = smith_protocol_channel::decode_turn(
        &terminal.transcript,
        terminal.turn,
        &config.limits.channel.transcript,
        &config.channel_endpoints,
    )
    .map_err(|why| format!("drawn chat's final turn: {why:?}"))?;
    reached.turns(&[turn]);
    let mut sample = Sample::new("a_person_s_chat_is_answered_by_the_in_process_worker");
    sample.ran(&reached)?;
    sample.write();
    Ok(())
}

#[derive(Default)]
struct Evidence<'a> {
    problems: Vec<String>,
    ids: BTreeSet<&'a [u8]>,
}

impl Evidence<'_> {
    fn drawn(&mut self, cancelled: bool, name: &[u8], arguments: &[u8], output: &[u8]) -> Call {
        let tool = match name {
            b"message" => Tool::Message,
            b"subscribe" => Tool::Subscribe,
            _ => {
                self.problems.push(format!("stretch drew another tool: {}", String::from_utf8_lossy(name)));
                Tool::Unknown
            }
        };
        let body = serde_json::from_slice::<serde_json::Value>(arguments).ok();
        let target = match body.as_ref().and_then(|body| body.get("target")).and_then(serde_json::Value::as_u64) {
            Some(DELEGATE) if cancelled => Class::Ended,
            Some(DELEGATE) => Class::Live,
            Some(NEVER_ISSUED) => Class::Unissued,
            _ => {
                self.problems.push(format!("unexpected drawn target: {}", String::from_utf8_lossy(arguments)));
                Class::Unknown
            }
        };
        let reply = classify(output);
        let fits = matches!(
            (target, tool, &reply),
            (Class::Live, Tool::Message, Reply::Sent | Reply::QuestionRefused)
                | (Class::Live, Tool::Subscribe, Reply::Subscribed)
                | (Class::Ended | Class::Unissued, Tool::Message, Reply::MissingReference)
                | (Class::Ended | Class::Unissued, Tool::Subscribe, Reply::SubscriptionRefused)
        );
        if !fits {
            self.problems.push(format!(
                "{target:?}: {} {} → {}",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(arguments),
                String::from_utf8_lossy(output)
            ));
        }
        let answer = match reply {
            Reply::Sent => Answer::Sent,
            Reply::Subscribed => Answer::Subscribed,
            Reply::MissingReference | Reply::SubscriptionRefused => Answer::NoReference,
            Reply::QuestionRefused => Answer::Refused,
            Reply::Unrecognized => Answer::Unknown,
        };
        let mut references = vec![(Reference::Task, target)];
        if tool == Tool::Message
            && let Some(body) = body
            && body["form"] == "answer"
        {
            let class = if body["question"].as_u64() == Some(NEVER_ISSUED) { Class::Unissued } else { Class::Unknown };
            references.push((Reference::Question, class));
        }
        Call { tool, answer, references }
    }

    fn check(mut self, completions: &[Vec<Call>]) -> Result<(), String> {
        for target in [Class::Unissued, Class::Ended] {
            if !completions
                .iter()
                .flatten()
                .any(|call| call.answer == Answer::NoReference && call.references.contains(&(Reference::Task, target)))
            {
                self.problems.push(format!("no refusal for {target:?} targets"));
            }
        }
        if self.problems.is_empty() { Ok(()) } else { Err(self.problems.join("; ")) }
    }
}

fn classify(output: &[u8]) -> Reply {
    // Smith puts the failure marker in the text of the provider's tool output.
    match output.strip_prefix(b"Error: ") {
        Some(b"Refused; missing reference") => Reply::MissingReference,
        Some(SUBSCRIPTION_REFUSAL) => Reply::SubscriptionRefused,
        Some(b"Refused: reference") => Reply::QuestionRefused,
        None if numbered(output, b"Sent message ") => Reply::Sent,
        None if numbered(output, b"Subscribed ") => Reply::Subscribed,
        _ => Reply::Unrecognized,
    }
}

fn numbered(output: &[u8], prefix: &[u8]) -> bool {
    output.strip_prefix(prefix).is_some_and(|id| !id.is_empty() && id.iter().all(u8::is_ascii_digit))
}
