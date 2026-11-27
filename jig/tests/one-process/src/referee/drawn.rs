//! Evidence from the last conversation the fake provider saw (jig/docs/design/protocol/service-testing.md, 1).
use crate::fixtures::drawn::{DELEGATE, NEVER_ISSUED, ROUNDS};
use skein_fake_llm_domain::api::Part;
use skein_fake_peers::llm::Observation;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Target {
    Live,
    Ended,
    NeverIssued,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Reply {
    Sent,
    Subscribed,
    MissingReference,
    SubscriptionRefused,
    QuestionRefused,
    Unrecognized,
}

type Tally = BTreeMap<(Target, Reply), usize>;

const DELEGATED: &[u8] = b"Delegated tasks: 2 at spec revision 0";
const SUBSCRIPTION_REFUSAL: &[u8] =
    b"Refused: the target must be a task this one references or a live task of its project its tree's requester reads";

pub(crate) fn check(observations: &[Observation]) -> Result<(), String> {
    let mut queries = observations.iter().filter_map(|observation| match observation {
        Observation::Query { query, .. } => Some(query),
        _ => None,
    });
    let last = queries.next_back().ok_or("drawn chat never reached the model; tally: {}")?;
    let mut outputs = BTreeMap::new();
    for part in last.messages.iter().flat_map(|message| &message.parts) {
        if let Part::ToolOutput { id, output, .. } = part {
            let entry = outputs.entry(id.as_ref()).or_insert((output.as_ref(), 0));
            entry.1 += 1;
        }
    }
    // The last query asks for the scripted finish. Each preceding tool answer
    // appears in a message of its own before the matching outputs.
    let batches = last.messages.iter().filter_map(|message| {
        let calls = message.parts.iter().filter(|part| matches!(part, Part::ToolCall { .. })).collect::<Vec<_>>();
        (!calls.is_empty()).then_some(calls)
    });
    let cancel = ROUNDS + 1;
    let mut sample = Sample::default();
    for (batch, calls) in batches.enumerate() {
        sample.batches += 1;
        let script = match batch {
            0 => Some((b"delegate".as_slice(), DELEGATED)),
            index if index == cancel => Some((b"cancel".as_slice(), b"Done".as_slice())),
            _ => None,
        };
        if calls.len() != 1 {
            sample.problems.push(format!("round {batch} returned {} calls instead of one", calls.len()));
        }
        for call in calls {
            let Part::ToolCall { id, name, arguments } = call else { unreachable!("selected only tool calls") };
            let answer = outputs.get(id.as_ref()).copied();
            if !sample.ids.insert(id.as_ref()) || !matches!(answer, Some((_, 1))) {
                sample
                    .problems
                    .push(format!("call {} did not occur and get answered exactly once", String::from_utf8_lossy(id)));
            }
            let output = answer.map_or(b"".as_slice(), |(output, _)| output);
            if let Some((tool, want)) = script {
                if name.as_ref() != tool || output != want {
                    sample.problems.push(format!(
                        "script round {batch}: expected {} → {}, got {} → {}",
                        String::from_utf8_lossy(tool),
                        String::from_utf8_lossy(want),
                        String::from_utf8_lossy(name),
                        String::from_utf8_lossy(output)
                    ));
                }
            } else {
                sample.drawn(batch > cancel, name, arguments, output);
            }
        }
    }
    let expected = 2 * (ROUNDS + 1);
    let earlier = queries.clone().count();
    if sample.batches != expected || earlier != expected {
        sample.problems.push(format!(
            "expected {expected} tool rounds before finish; saw {} in the last query and {} earlier queries",
            sample.batches, earlier
        ));
    }
    // A call echoed in an earlier query must still be in the conversation at finish.
    for part in queries.flat_map(|q| q.messages.iter().flat_map(|m| &m.parts)) {
        if let Part::ToolCall { id, .. } = part
            && !sample.ids.contains(id.as_ref())
        {
            sample.problems.push(format!("call {} disappeared before finish", String::from_utf8_lossy(id)));
        }
    }
    sample.finish()
}

#[derive(Default)]
struct Sample<'a> {
    tally: Tally,
    problems: Vec<String>,
    ids: BTreeSet<&'a [u8]>,
    batches: usize,
}

impl Sample<'_> {
    fn drawn(&mut self, cancelled: bool, name: &[u8], arguments: &[u8], output: &[u8]) {
        if !matches!(name, b"message" | b"subscribe") {
            self.problems.push(format!("stretch drew another tool: {}", String::from_utf8_lossy(name)));
        }
        let body = serde_json::from_slice::<serde_json::Value>(arguments).ok();
        let target = match body.as_ref().and_then(|body| body.get("target")).and_then(serde_json::Value::as_u64) {
            Some(DELEGATE) if cancelled => Target::Ended,
            Some(DELEGATE) => Target::Live,
            Some(NEVER_ISSUED) => Target::NeverIssued,
            _ => {
                self.problems.push(format!("unexpected drawn target: {}", String::from_utf8_lossy(arguments)));
                return;
            }
        };
        let reply = classify(output);
        *self.tally.entry((target, reply)).or_default() += 1;
        let fits = matches!(
            (target, name, reply),
            (Target::Live, b"message", Reply::Sent | Reply::QuestionRefused)
                | (Target::Live, b"subscribe", Reply::Subscribed)
                | (Target::Ended | Target::NeverIssued, b"message", Reply::MissingReference)
                | (Target::Ended | Target::NeverIssued, b"subscribe", Reply::SubscriptionRefused)
        );
        if !fits {
            self.problems.push(format!(
                "{target:?}: {} {} → {}",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(arguments),
                String::from_utf8_lossy(output)
            ));
        }
    }

    fn finish(mut self) -> Result<(), String> {
        for target in [Target::NeverIssued, Target::Ended] {
            if ![Reply::MissingReference, Reply::SubscriptionRefused]
                .iter()
                .any(|&reply| self.tally.contains_key(&(target, reply)))
            {
                self.problems.push(format!("no refusal for {target:?} targets"));
            }
        }
        if self.problems.is_empty() {
            Ok(())
        } else {
            Err(format!("{}; drawn target × reply tally: {:?}", self.problems.join("; "), self.tally))
        }
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
