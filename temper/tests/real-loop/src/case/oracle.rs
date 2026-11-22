//! Reference-case checkpoints: effects over observations any world can supply.
use super::{Agent, DESIGN_DOC, DESIGN_WORDS, DRAFT, DRAFT_WORDS, PROJECT, acts::Read, kit_tree};
use crate::{Exit, Ran, calls};
use serde_json::Value;
use skein_lib::Wall;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Clone)]
pub struct Observed {
    /// ctl's whole JSON document, by the act that read it; absent if that act failed.
    pub documents: BTreeMap<Read, Value>,
    /// Page names served by a collected observation, not another submitted read.
    pub activity_aliases: BTreeSet<Read>,
    /// Client completion/observation wall inputs, never the server read instant.
    pub observed_at: BTreeMap<Read, Wall>,
    /// Each LLM query in order, by its cue: its messages' texts and tool outputs.
    pub told: Vec<Told>,
    /// kit's main after its last observed merge or the person's push.
    pub kit: Kit,
}
#[derive(Clone)]
pub struct Told {
    /// Ordered query messages, preserving roles and tool-call identities across a retry.
    pub history: Box<[skein_fake_llm_domain::api::Message]>,
    pub agent: Agent,
    /// Repository conventions and the brief, separate from conversation messages.
    pub system: Box<[u8]>,
    pub words: Box<[u8]>,
}
#[derive(Clone)]
pub struct Kit {
    /// Forge records in their observed order, including the person's main push.
    pub records: Vec<skein_fake_forge_domain::Observation>,
    pub main: BTreeMap<Vec<u8>, Vec<u8>>,
    pub merged_green: bool,
    pub pulls: Vec<Pull>,
}
impl Told {
    /// The oracle's per-query view, shared with process-tier observation readers.
    #[must_use]
    pub fn new(system: &[u8], messages: &[skein_fake_llm_domain::api::Message]) -> Option<Self> {
        Some(Self::with_agent(cued(system)?, system, messages))
    }

    fn with_agent(agent: Agent, system: &[u8], messages: &[skein_fake_llm_domain::api::Message]) -> Self {
        let mut words = system.to_vec();
        words.push(b'\n');
        for message in messages {
            for part in &message.parts {
                use skein_fake_llm_domain::api::Part;
                match part {
                    Part::Text { text } | Part::ToolOutput { output: text, .. } => {
                        words.extend_from_slice(text);
                        words.push(b'\n');
                    }
                    Part::Opaque { .. } | Part::ToolCall { .. } => {}
                }
            }
        }
        Self { system: system.into(), history: messages.into(), agent, words: words.into_boxed_slice() }
    }
}
impl Observed {
    #[must_use]
    pub fn from_ran(ran: &Ran, reads: &[(Read, usize)]) -> Self {
        let (documents, observed_at) = read_documents(ran.calls(), reads);
        let mut told = Vec::new();
        for observation in ran.llm() {
            let skein_fake_peers::llm::Observation::Query { query, .. } = observation else { continue };
            if let Some(query) = Told::new(&query.system, &query.messages) {
                told.push(query);
            }
        }
        Self {
            activity_aliases: reads
                .iter()
                .filter_map(|(read, index)| ran.calls().get(*index).filter(|call| call.tries == 0).map(|_| *read))
                .collect(),
            documents,
            observed_at,
            told,
            kit: Kit {
                records: ran.forge().observations().to_vec(),
                pulls: pull_digest(ran.forge().observations()),
                main: ran.forge().main().clone(),
                merged_green: ran.forge().merged_green(),
            },
        }
    }
}
fn read_documents(calls: &[calls::Called], reads: &[(Read, usize)]) -> (BTreeMap<Read, Value>, BTreeMap<Read, Wall>) {
    let mut documents = BTreeMap::new();
    let mut observed_at = BTreeMap::new();
    for (read, position) in reads {
        let Some(call) = calls.get(*position) else { continue };
        if call.exit != Some(Exit::Code(0)) {
            continue;
        }
        if let Ok(document) = serde_json::from_slice::<Value>(&call.stdout) {
            documents.insert(*read, document);
            if let Some(wall) = call.observed_at {
                observed_at.insert(*read, wall);
            }
        }
    }
    (documents, observed_at)
}
fn cued(system: &[u8]) -> Option<Agent> {
    let mut first_at = usize::MAX;
    let mut first = None;
    for agent in [
        Agent::Director,
        Agent::A,
        Agent::Design,
        Agent::Review,
        Agent::Check,
        Agent::B,
        Agent::Benchmark,
        Agent::Hang,
        Agent::Phase,
        Agent::Rules,
        Agent::Scope,
        Agent::Pause,
        Agent::Bowner,
        Agent::H1Coder,
        Agent::Shape,
        Agent::Mover,
        Agent::Direction,
        Agent::StoryOwner,
        Agent::OverlapOwner,
        Agent::X3Writer,
        Agent::PlannedWriter,
        Agent::StoryWriter,
        // 6.6
        Agent::WaitingStarter,
        Agent::WaitingKitCoder,
        Agent::WaitingAppCoder,
        // 6.4
        Agent::RedRepairCoder,
        Agent::QaDirector,
        Agent::QaOwner,
        Agent::QaSubowner,
        Agent::QaTester,
        Agent::QaCoder,
    ] {
        if let Some(at) = skein_lib::bytes::find(system, agent.cue()) {
            // No later cue can precede zero; equal positions keep this agent's priority.
            if at == 0 {
                return Some(agent);
            }
            if at < first_at {
                first_at = at;
                first = Some(agent);
            }
        }
    }
    first
}

pub struct Miss {
    pub step: &'static str,
    pub promise: Promise,
    pub what: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Promise {
    Must(&'static str),
    Add(&'static str),
    Step(&'static str),
}
impl fmt::Display for Miss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (kind, words) = match self.promise {
            Promise::Must(words) => ("Must", words),
            Promise::Add(words) => ("Add", words),
            Promise::Step(words) => ("Step", words),
        };
        write!(f, "{} {kind} \"{words}\": {}", self.step, self.what)
    }
}
const MARKDOWN: Promise = Promise::Must("the fast path for Markdown alone (exact)");
fn check(misses: &mut Vec<Miss>, met: bool, promise: Promise, what: String) {
    if !met {
        misses.push(Miss { step: "4.1", promise, what });
    }
}
fn text<'a>(value: &'a Value, path: &str) -> &'a str {
    pointer(value, path).and_then(Value::as_str).unwrap_or_default()
}
// Result words may carry a pull description after the merge id (domain/root.md, 5).
fn landing_commit(value: &Value) -> &str {
    text(value, "/result/value/change/words/bytes").lines().next().unwrap_or_default()
}
fn number(value: &Value, path: &str) -> Option<u64> {
    pointer(value, path).and_then(Value::as_u64)
}
// Oracle paths have plain tokens: borrow them instead of allocating decoded
// tokens at each read. Escaped keys keep the JSON pointer decoder's semantics.
fn pointer<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path.contains('~') {
        return value.pointer(path);
    }
    if path.is_empty() {
        return Some(value);
    }
    let mut at = value;
    for token in path.strip_prefix('/')?.split('/') {
        at = match at {
            Value::Object(members) => members.get(token)?,
            Value::Array(items) => {
                if token.starts_with('+') || (token.starts_with('0') && token.len() != 1) {
                    return None;
                }
                items.get(token.parse::<usize>().ok()?)?
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => return None,
        };
    }
    Some(at)
}
fn contains(words: &[u8], needle: &str) -> bool {
    words.windows(needle.len()).any(|part| part == needle.as_bytes())
}

#[must_use]
pub fn step_4_1(observed: &Observed) -> Vec<Miss> {
    let mut misses = Vec::new();
    markdown(&mut misses, observed);
    misses
}

fn markdown(misses: &mut Vec<Miss>, observed: &Observed) {
    // The fast path itself belongs to the forge's CI and is left out here
    // (testing/reference-case.md's 4.1 row). We check the merge and its Markdown.
    check(misses, observed.kit.merged_green, MARKDOWN, "kit.main must have merged with green CI at its head".into());
    check(
        misses,
        observed.kit.main.get(DESIGN_DOC.as_bytes()).map(Vec::as_slice) == Some(DESIGN_WORDS.as_bytes()),
        MARKDOWN,
        "kit.main must hold A's design words".into(),
    );
    check(
        misses,
        observed.kit.main.get(DRAFT.as_bytes()).map(Vec::as_slice) == Some(DRAFT_WORDS.as_bytes()),
        MARKDOWN,
        "kit's draft must be unchanged".into(),
    );
    let mut before = BTreeMap::new();
    for file in kit_tree() {
        before.insert(file.path.to_vec(), file.content.to_vec());
    }
    let mut paths = BTreeSet::new();
    paths.extend(before.keys());
    paths.extend(observed.kit.main.keys());
    for path in paths {
        if before.get(path) != observed.kit.main.get(path) {
            check(
                misses,
                path.ends_with(b".md"),
                MARKDOWN,
                format!("changed path {:?} is not Markdown", String::from_utf8_lossy(path)),
            );
        }
    }
}

pub fn assert_met(misses: &[Miss]) {
    let mut report = String::new();
    for miss in misses {
        use fmt::Write;
        writeln!(report, "{miss}").expect("miss report");
    }
    assert!(misses.is_empty(), "{report}");
}

#[cfg(test)]
mod tests {
    use super::{
        Agent, BTreeMap, DESIGN_DOC, DESIGN_WORDS, Kit, MARKDOWN, Observed, Told, assert_met, kit_tree, step_4_1,
    };

    #[test]
    fn the_first_system_cue_identifies_the_agent_before_delegated_cues() {
        let system = [b"guide\n".as_slice(), Agent::A.cue(), b"\ndelegate ", Agent::Director.cue()].concat();
        let query = Told::new(&system, &[]).expect("system cue");
        assert_eq!(query.agent, Agent::A);
        let query = Told::new(Agent::RedRepairCoder.cue(), &[]).expect("opening cue");
        assert_eq!(query.agent, Agent::RedRepairCoder);
        assert!(Told::new(b"guide without a case cue", &[]).is_none());
    }

    #[test]
    fn borrowed_paths_keep_json_pointer_semantics() {
        let doc = serde_json::json!({"":0,"a/b":{"~key":[null,{"x":"value"}]},
            "items":[true,{"":2}],"01":"object key","scalar":7});
        for path in [
            "",
            "/",
            "/01",
            "/items",
            "/items/0",
            "/items/1/",
            "/items/01",
            "/items/+1",
            "/items/-1",
            "/items/-",
            "/items/2",
            "/items/",
            "/items/18446744073709551616",
            "/scalar/x",
            "items/0",
            "/missing",
            "/a~1b/~0key/1/x",
            "/a~1b/~0key/00",
            "/~2",
        ] {
            assert_eq!(super::pointer(&doc, path), doc.pointer(path), "{path}");
        }
    }

    pub(super) fn met() -> Observed {
        let mut main = BTreeMap::new();
        for file in kit_tree() {
            main.insert(file.path.to_vec(), file.content.to_vec());
        }
        main.insert(DESIGN_DOC.as_bytes().to_vec(), DESIGN_WORDS.as_bytes().to_vec());
        Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::new(),
            told: Vec::new(),
            kit: Kit { records: Vec::new(), pulls: Vec::new(), main, merged_green: true },
        }
    }

    #[test]
    fn a_met_checkpoint_has_no_misses() {
        assert_met(&step_4_1(&met()));
    }

    #[test]
    fn the_checkpoint_reports_every_independent_miss_together() {
        // Three independent corruptions of otherwise sufficient observations.
        const MISSES: usize = 3;
        let mut broken = met();
        broken.kit.merged_green = false;
        broken.kit.main.remove(DESIGN_DOC.as_bytes());
        broken.kit.main.insert(b"src/export.rs".to_vec(), b"fn export() {}\n".to_vec());
        let misses = step_4_1(&broken);
        assert_eq!(misses.len(), MISSES);
        assert!(misses.iter().all(|miss| miss.promise == MARKDOWN));
        let [green, design, markdown] = misses.as_slice() else { panic!("three misses") };
        assert!(green.what.contains("green CI"));
        assert!(design.what.contains("design words"));
        assert!(markdown.what.contains("src/export.rs"));
    }
}

fn step_check(misses: &mut Vec<Miss>, step: &'static str, met: bool, promise: Promise, what: impl fmt::Display) {
    if !met {
        misses.push(Miss { step, promise, what: what.to_string() });
    }
}
fn document(observed: &Observed, read: Read) -> &Value {
    observed.documents.get(&read).unwrap_or(&Value::Null)
}

fn task_document(observed: &Observed, read: Read) -> &Value {
    let value = document(observed, read);
    value.get("task").filter(|task| task.is_object()).unwrap_or(value)
}

// An alias has no extra journal positions. Calibration observations have no aliases,
// so repeated or unordered positions in a submitted page remain a failed promise.
fn observation_reads(observed: &Observed, reads: &[Read]) -> Vec<Read> {
    reads.iter().copied().filter(|read| !observed.activity_aliases.contains(read)).collect()
}
fn joined_activity<'a>(observed: &'a Observed, reads: &[Read]) -> Vec<&'a Value> {
    observation_reads(observed, reads).into_iter().flat_map(|read| check_activity(observed, read)).collect()
}

// ctl wraps the source journal lines in its decoded activity rows.
fn check_activity(observed: &Observed, read: Read) -> Vec<&Value> {
    let document = document(observed, read);
    document
        .get("activity")
        .filter(|value| value.is_object())
        .unwrap_or(document)
        .get("rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("event")?.get("owner")?.get("source"))
        .collect()
}

/// The real-clock wake and cooperative run cancellation (testing/reference-case.md).
#[must_use]
pub fn step_4_3(observed: &Observed) -> Vec<Miss> {
    const TIMER: Promise = Promise::Must("a timer (exact)");
    const CANCEL: Promise = Promise::Must("cancel (exact)");
    let mut misses = Vec::new();
    let benchmark = task_document(observed, Read::BenchmarkEnded);
    let director_activity = check_activity(observed, Read::DirectorTimer);
    let timer = director_activity
        .iter()
        .find_map(|line| number(line, "/line/heard/kind/timer/subscription").zip(number(line, "/line/heard/number")));
    step_check(
        &mut misses,
        "4.3",
        timer.is_some(),
        TIMER,
        "the director's activity must record the timer's subscription and wake",
    );
    let queries: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Check).collect();
    step_check(
        &mut misses,
        "4.3",
        timer.is_some_and(|(subscription, _message)| {
            queries.iter().any(|told| check_timer_delivery(told, subscription))
        }),
        TIMER,
        "the director must read the timer's empty wake after its subscription result, in that query or a later one",
    );
    step_check(
        &mut misses,
        "4.3",
        text(benchmark, "/chip/phase") == "ended" && benchmark.pointer("/result/ending/task/cancelled").is_some(),
        CANCEL,
        "the benchmark must end cancelled",
    );
    let activity = check_activity(observed, Read::BenchmarkEnded);
    let attempt = number(benchmark, "/chip/details/owner/details/attempt");
    let cancelled = activity.iter().any(|line| {
        text(line, "/line/run_ended/end") == "cancelled"
            && number(line, "/line/run_ended/attempt").zip(attempt).is_some_and(|(ended, current)| ended == current)
    });
    step_check(&mut misses, "4.3", cancelled, CANCEL, "the benchmark's activity must record a cancelled run ending");
    misses
}

// Smith offers queued messages beside settled tool results, including subscribe's
// result (smith/docs/design/domain/run.md, 6). A live wait is only one delivery seam.
fn check_timer_delivery(told: &Told, subscription: u64) -> bool {
    use skein_fake_llm_domain::api::{Part, Role};
    let result = format!("Subscription {subscription} added");
    let wake = format!("{}: ", super::CHECK);
    let mut subscribed = false;
    for message in &told.history {
        if message.role != Role::User {
            continue;
        }
        for part in &message.parts {
            match part {
                Part::ToolOutput { output, is_error: false, .. } if contains(output, &result) => subscribed = true,
                Part::Text { text }
                    if subscribed && text.split(|byte| *byte == b'\n').any(|line| line == wake.as_bytes()) =>
                {
                    return true;
                }
                Part::Text { .. } | Part::ToolOutput { .. } | Part::ToolCall { .. } | Part::Opaque { .. } => {}
            }
        }
    }
    false
}

#[cfg(test)]
mod check_timer_tests {
    use super::{Agent, Told, check_timer_delivery};
    use skein_fake_llm_domain::api::{Message, Part, Role};

    #[test]
    fn timer_delivery_requires_the_subscription_result_before_the_wake() {
        let receipt = Part::ToolOutput {
            id: b"subscribe".as_slice().into(),
            output: b"Subscription 12 added".as_slice().into(),
            is_error: false,
        };
        let wake = Part::Text { text: format!("{}: ", crate::case::CHECK).into_bytes().into() };
        let message = |role, parts: Vec<Part>| Message { role, parts: parts.into() };
        for (name, messages, subscription, met) in [
            ("beside result", vec![message(Role::User, vec![receipt.clone(), wake.clone()])], 12, true),
            (
                "later query",
                vec![
                    message(Role::User, vec![receipt.clone()]),
                    message(Role::Assistant, vec![]),
                    message(Role::User, vec![wake.clone()]),
                ],
                12,
                true,
            ),
            ("wake first", vec![message(Role::User, vec![wake.clone(), receipt.clone()])], 12, false),
            ("no result", vec![message(Role::User, vec![wake.clone()])], 12, false),
            ("no wake", vec![message(Role::User, vec![receipt.clone()])], 12, false),
            ("another subscription", vec![message(Role::User, vec![receipt.clone(), wake.clone()])], 13, false),
            ("assistant words", vec![message(Role::Assistant, vec![receipt, wake])], 12, false),
        ] {
            let told = Told::with_agent(Agent::Check, b"", &messages);
            assert_eq!(check_timer_delivery(&told, subscription), met, "{name}");
        }
    }
}

const ONCE: Promise = Promise::Add("nothing resumes twice");

/// The real worker restart delivers the direct answer once (testing/reference-case.md's 4.6 row).
#[must_use]
pub fn step_4_6(observed: &Observed) -> Vec<Miss> {
    let mut misses = Vec::new();
    step_check(
        &mut misses,
        "4.6",
        restart_answer_once(observed),
        ONCE,
        "A must receive the person's answer once, without a director relay",
    );
    misses
}

fn user_words(told: &Told) -> impl Iterator<Item = &[u8]> {
    use skein_fake_llm_domain::api::{Part, Role};
    told.history.iter().filter(|message| message.role == Role::User).flat_map(|message| &message.parts).filter_map(
        |part| match part {
            Part::Text { text } | Part::ToolOutput { output: text, .. } => Some(text.as_ref()),
            Part::Opaque { .. } | Part::ToolCall { .. } => None,
        },
    )
}

fn restart_answer_once(observed: &Observed) -> bool {
    use super::{PHASE_ANSWER, PHASE_QUESTION_NUMBER};
    use skein_fake_llm_domain::api::Part;
    let mut valid_answers = true;
    let mut received = 0;
    for told in &observed.told {
        if told.agent == Agent::Hang {
            for part in told.history.iter().flat_map(|message| &message.parts) {
                if let Part::ToolCall { name, arguments, .. } = part
                    && name.as_ref() == b"message"
                    && let Ok(args) = serde_json::from_slice::<Value>(arguments)
                    && args["form"] == "answer"
                    && args["question"] == PHASE_QUESTION_NUMBER
                {
                    valid_answers = false;
                }
            }
        }
        if told.agent == Agent::Phase {
            received += user_words(told).map(|words| calls::occurrences(words, PHASE_ANSWER.as_bytes())).sum::<usize>();
        }
    }
    valid_answers && received == 1
}

#[cfg(test)]
mod restart_tests {
    use super::*;
    use crate::case::{PHASE, PHASE_ANSWER, PHASE_QUESTION_NUMBER};
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    fn call(id: &str, name: &str, arguments: &Value) -> Part {
        Part::ToolCall {
            id: id.as_bytes().into(),
            name: name.as_bytes().into(),
            arguments: serde_json::to_vec(arguments).expect("tool arguments").into(),
        }
    }
    fn told(agent: Agent, history: Vec<Message>) -> Told {
        Told { system: Box::new([]), agent, history: history.into_boxed_slice(), words: Box::new([]) }
    }
    fn met() -> Observed {
        Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::new(),
            kit: Kit { records: Vec::new(), pulls: Vec::new(), main: BTreeMap::new(), merged_green: false },
            told: vec![
                told(Agent::Hang, Vec::new()),
                told(
                    Agent::Phase,
                    vec![Message {
                        role: Role::User,
                        parts: vec![Part::ToolOutput {
                            id: b"wait".as_slice().into(),
                            output: PHASE_ANSWER.as_bytes().into(),
                            is_error: false,
                        }]
                        .into_boxed_slice(),
                    }],
                ),
            ],
        }
    }
    #[test]
    fn a_restarted_chat_meets_the_checkpoint() {
        assert_met(&step_4_6(&met()));
    }
    #[test]
    fn replayed_history_is_one_answer_but_a_new_call_is_another() {
        const DIRECTOR_QUERY: usize = 0;
        let mut observed = met();
        observed.told.push(observed.told[DIRECTOR_QUERY].clone());
        assert_met(&step_4_6(&observed));
        let duplicate = call(
            "second-answer",
            "message",
            &json!({"target":PHASE,"form":"answer","question":PHASE_QUESTION_NUMBER,"words":PHASE_ANSWER}),
        );
        observed.told.push(told(
            Agent::Hang,
            vec![Message { role: Role::Assistant, parts: vec![duplicate].into_boxed_slice() }],
        ));
        let misses = step_4_6(&observed);
        assert!(misses.iter().any(|miss| miss.promise == ONCE));
    }
    #[test]
    fn a_missing_or_duplicate_answer_breaks_its_promise() {
        const PHASE_QUERY: usize = 1;
        let mut missing = met();
        missing.told.pop();
        assert!(step_4_6(&missing).iter().any(|miss| miss.promise == ONCE));
        let mut duplicate = met();
        duplicate.told.push(duplicate.told[PHASE_QUERY].clone());
        assert!(step_4_6(&duplicate).iter().any(|miss| miss.promise == ONCE));
    }
}

// Act 2, step 5.1 (testing/reference-case.md)
const RULES_CONVENTIONS: Promise = Promise::Must("the rules land as conventions that every later run reads (exact)");
const RULES_DROPPED: Promise = Promise::Add("a held owner needs no notice that a rule was dropped");

#[must_use]
pub fn step_5_1(observed: &Observed) -> Vec<Miss> {
    use super::{DROPPED_RULE, NEW_RULES, OLD_RULES, RULES_FILE};
    let mut misses = Vec::new();
    step_check(
        &mut misses,
        "5.1",
        observed.kit.main.get(RULES_FILE.as_bytes()).is_some_and(|words| words == NEW_RULES.as_bytes()),
        RULES_CONVENTIONS,
        "kit's main must hold the new guide",
    );
    for (agent, promise) in [(Agent::Rules, RULES_CONVENTIONS), (Agent::Scope, RULES_DROPPED)] {
        let queries: Vec<_> = observed.told.iter().filter(|told| told.agent == agent).collect();
        // System text stays fixed within a run; the fresh checkout starts the next one.
        let fresh = if agent == Agent::Scope {
            usize::from(!queries.is_empty())
        } else {
            queries.first().map_or(0, |first| queries.iter().take_while(|told| told.system == first.system).count())
        };
        let (opening, later) = queries.split_at(fresh);
        step_check(
            &mut misses,
            "5.1",
            !opening.is_empty() && opening.iter().all(|told| contains(&told.system, OLD_RULES)),
            promise,
            "the opening run must read the old guide, including the dropped rule",
        );
        step_check(
            &mut misses,
            "5.1",
            (agent == Agent::Scope || !later.is_empty())
                && later.iter().all(|told| contains(&told.system, NEW_RULES) && !contains(&told.system, DROPPED_RULE)),
            promise,
            "every query after release must read the new guide without the dropped rule",
        );
    }
    misses
}

#[cfg(test)]
mod rules_tests {
    use super::*;
    use crate::case::{DROPPED_RULE, NEW_RULES, OLD_RULES, RULES_FILE};

    fn told(agent: Agent, guide: &str) -> Told {
        let mut system = agent.cue().to_vec();
        system.extend_from_slice(guide.as_bytes());
        Told::new(&system, &[]).expect("case cue")
    }

    fn met() -> Observed {
        Observed {
            activity_aliases: BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::new(),
            told: vec![
                told(Agent::Rules, OLD_RULES),
                told(Agent::Rules, NEW_RULES),
                told(Agent::Rules, NEW_RULES),
                told(Agent::Scope, OLD_RULES),
                told(Agent::Scope, NEW_RULES),
            ],
            kit: Kit {
                records: Vec::new(),
                pulls: Vec::new(),
                merged_green: false,
                main: BTreeMap::from([(RULES_FILE.as_bytes().to_vec(), NEW_RULES.as_bytes().to_vec())]),
            },
        }
    }

    fn rejects(observed: &Observed, promise: Promise, what: &str) {
        let misses = step_5_1(observed);
        assert!(misses.iter().all(|miss| miss.step == "5.1"));
        assert!(
            misses.iter().any(|miss| miss.promise == promise && miss.what.contains(what)),
            "expected {what}: {}",
            misses.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        );
    }

    #[test]
    fn pushed_rules_and_fresh_checkouts_meet_the_checkpoint() {
        assert_met(&step_5_1(&met()));
    }

    #[test]
    fn main_and_every_relevant_query_must_have_the_right_system_guide() {
        let mut observed = met();
        observed.kit.main.clear();
        rejects(&observed, RULES_CONVENTIONS, "kit's main");
        for (at, promise, what) in [
            (0, RULES_CONVENTIONS, "opening run"),
            (1, RULES_CONVENTIONS, "every query after release"),
            (2, RULES_CONVENTIONS, "every query after release"),
            (3, RULES_DROPPED, "opening run"),
            (4, RULES_DROPPED, "every query after release"),
        ] {
            let mut observed = met();
            observed.told[at].system = Box::new([]);
            rejects(&observed, promise, what);
        }
        for (at, promise) in [(1, RULES_CONVENTIONS), (2, RULES_CONVENTIONS), (4, RULES_DROPPED)] {
            let mut observed = met();
            let mut system = observed.told[at].system.to_vec();
            system.extend_from_slice(DROPPED_RULE.as_bytes());
            observed.told[at].system = system.into();
            rejects(&observed, promise, "every query after release");
        }
    }
}

// Act 2, step 5.3 (testing/reference-case.md)
const SHAPE_INSTRUCTIONS: Promise = Promise::Must("instructions amended (exact)");
const SHAPE_HISTORY: Promise = Promise::Add("the amendment is in A's history, with what it replaced");
const SHAPE_ENDINGS: Promise = Promise::Step("the chat and A report under the new brief");

/// The observable before/after, without a deadline's hold (waiting on jig)
/// or a ctl read of the stored amendment history (a client gap).
#[must_use]
pub fn step_5_3(observed: &Observed) -> Vec<Miss> {
    use super::{
        MOVER, MOVER_AMENDMENT_REVISION, MOVER_QUESTION, MOVER_QUESTION_NUMBER, MOVER_REPORT, NEW_BRIEF, OLD_BRIEF,
        SHAPE, SHAPE_REPORT,
    };
    let mut misses = Vec::new();
    let ended = task_document(observed, Read::MoverEnded);
    step_check(
        &mut misses,
        "5.3",
        text(ended, "/instructions/bytes") == NEW_BRIEF
            && ended.pointer("/instructions/cut") == Some(&Value::Bool(false))
            && number(ended, "/spec_revision/value") == Some(MOVER_AMENDMENT_REVISION),
        SHAPE_HISTORY,
        "A's document must hold the whole new brief at the amendment's revision",
    );
    let a: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Mover).collect();
    step_check(
        &mut misses,
        "5.3",
        a.first().is_some_and(|told| contains(&told.system, OLD_BRIEF) && !contains(&told.system, NEW_BRIEF)),
        SHAPE_HISTORY,
        "A's first query must carry the old instructions alone in system text",
    );
    let amended = a
        .iter()
        .position(|told| contains(&told.system, NEW_BRIEF) || user_words(told).any(|words| contains(words, NEW_BRIEF)));
    let after = amended.map_or(&a[a.len()..], |at| &a[at..]);
    step_check(
        &mut misses,
        "5.3",
        amended.is_some_and(|at| at > 0)
            && !after.is_empty()
            && after.iter().all(|told| contains(&told.system, NEW_BRIEF) && !contains(&told.system, OLD_BRIEF)),
        SHAPE_INSTRUCTIONS,
        "every A query from the amendment onward must use the new instructions alone",
    );
    step_check(
        &mut misses,
        "5.3",
        !after.is_empty()
            && after.iter().all(|told| {
                user_words(told).map(|words| calls::occurrences(words, NEW_BRIEF.as_bytes())).sum::<usize>() == 1
            }),
        SHAPE_INSTRUCTIONS,
        "A must be told the amendment's whole message once in each later query's history",
    );
    let chat: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Shape).collect();
    step_check(
        &mut misses,
        "5.3",
        !chat.is_empty()
            && chat.iter().all(|told| !contains(&told.system, OLD_BRIEF) && !contains(&told.system, NEW_BRIEF)),
        SHAPE_INSTRUCTIONS,
        "the chat's charter must carry neither of A's instruction briefs",
    );
    let sent = format!("Message {MOVER_QUESTION_NUMBER} sent");
    let question_at = observed
        .told
        .iter()
        .position(|told| told.agent == Agent::Shape && user_words(told).any(|words| contains(words, MOVER_QUESTION)));
    let amendment_at = observed.told.iter().position(|told| {
        told.agent == Agent::Mover
            && (contains(&told.system, NEW_BRIEF) || user_words(told).any(|words| contains(words, NEW_BRIEF)))
    });
    step_check(
        &mut misses,
        "5.3",
        a.iter().any(|told| contains(&told.words, &sent))
            && question_at.zip(amendment_at).is_some_and(|(question, amendment)| question < amendment),
        SHAPE_INSTRUCTIONS,
        "A's numbered question must reach the chat before it amends A",
    );
    for (read, task, report) in [(Read::MoverEnded, MOVER, MOVER_REPORT), (Read::ShapeEnded, SHAPE, SHAPE_REPORT)] {
        let ended = task_document(observed, read);
        step_check(
            &mut misses,
            "5.3",
            number(ended, "/chip/task") == Some(task)
                && number(ended, "/project") == Some(PROJECT)
                && text(ended, "/chip/phase") == "ended"
                && text(ended, "/result/value/report/bytes") == report,
            SHAPE_ENDINGS,
            "the original chat and A must end with their reports in the project",
        );
    }
    misses
}

#[cfg(test)]
mod shape_tests {
    use super::*;
    use crate::case::{
        MOVER, MOVER_AMENDMENT_REVISION, MOVER_QUESTION, MOVER_QUESTION_NUMBER, MOVER_REPORT, NEW_BRIEF, OLD_BRIEF,
        SHAPE, SHAPE_REPORT,
    };
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    fn user(words: &str) -> Message {
        Message { role: Role::User, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn told(agent: Agent, brief: &str, messages: &[Message]) -> Told {
        Told::new(&[agent.cue(), b"\n", brief.as_bytes()].concat(), messages).expect("case cue")
    }
    fn met() -> Observed {
        let a = json!({"chip":{"task":MOVER,"phase":"ended"},"project":PROJECT,
            "instructions":{"bytes":NEW_BRIEF,"cut":false},"spec_revision":{"value":MOVER_AMENDMENT_REVISION},
            "result":{"value":{"report":{"bytes":MOVER_REPORT}}}});
        let shape = json!({"chip":{"task":SHAPE,"phase":"ended"},"project":PROJECT,
            "result":{"value":{"report":{"bytes":SHAPE_REPORT}}}});
        Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::from([(Read::MoverEnded, a), (Read::ShapeEnded, shape)]),
            told: vec![
                told(Agent::Mover, OLD_BRIEF, &[]),
                told(
                    Agent::Mover,
                    OLD_BRIEF,
                    &[Message {
                        role: Role::User,
                        parts: Box::new([Part::ToolOutput {
                            id: b"question".as_slice().into(),
                            is_error: false,
                            output: format!("Message {MOVER_QUESTION_NUMBER} sent").into_bytes().into(),
                        }]),
                    }],
                ),
                told(Agent::Shape, "", &[user(MOVER_QUESTION)]),
                told(Agent::Mover, NEW_BRIEF, &[user(NEW_BRIEF)]),
                told(Agent::Mover, NEW_BRIEF, &[user(NEW_BRIEF), user("Continue")]),
            ],
            kit: Kit { records: Vec::new(), pulls: Vec::new(), main: BTreeMap::new(), merged_green: false },
        }
    }
    fn rejects(observed: &Observed, promise: Promise, what: &str) {
        let misses = step_5_3(observed);
        assert!(misses.iter().all(|miss| miss.step == "5.3"));
        assert!(
            misses.iter().any(|miss| miss.promise == promise && miss.what.contains(what)),
            "expected {what}: {}",
            misses.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
        );
    }
    #[test]
    fn amendment_and_its_observable_before_and_after_meet_the_checkpoint() {
        assert_met(&step_5_3(&met()));
    }
    #[test]
    fn the_current_instructions_and_their_revision_are_checked() {
        let passing = met();
        for path in ["/instructions/bytes", "/instructions/cut", "/spec_revision/value"] {
            let mut observed = passing.clone();
            *observed
                .documents
                .get_mut(&Read::MoverEnded)
                .expect("A's document")
                .pointer_mut(path)
                .expect("mutated field") = Value::Null;
            rejects(&observed, SHAPE_HISTORY, "A's document");
        }
    }
    #[test]
    fn the_opening_system_must_have_only_the_old_brief() {
        let passing = met();
        for brief in [String::new(), NEW_BRIEF.into(), format!("{OLD_BRIEF}\n{NEW_BRIEF}")] {
            let mut observed = passing.clone();
            observed.told[0].system = brief.into_bytes().into();
            rejects(&observed, SHAPE_HISTORY, "first query");
        }
    }
    #[test]
    fn every_system_after_the_amendment_must_have_only_the_new_brief() {
        let passing = met();
        for at in [3, 4] {
            for brief in [String::new(), OLD_BRIEF.into(), format!("{OLD_BRIEF}\n{NEW_BRIEF}")] {
                let mut observed = passing.clone();
                observed.told[at].system = brief.into_bytes().into();
                rejects(&observed, SHAPE_INSTRUCTIONS, "every A query");
            }
        }
    }
    #[test]
    fn the_amendment_message_must_arrive_once() {
        let passing = met();
        for at in [3, 4] {
            for messages in [vec![], vec![user(NEW_BRIEF), user(NEW_BRIEF)]] {
                let mut observed = passing.clone();
                observed.told[at].history = messages.into();
                rejects(&observed, SHAPE_INSTRUCTIONS, "whole message once");
            }
        }
        let mut observed = passing.clone();
        observed.told.truncate(3);
        rejects(&observed, SHAPE_INSTRUCTIONS, "every A query");
    }
    #[test]
    fn the_chat_never_inherits_either_brief() {
        let passing = met();
        for brief in [OLD_BRIEF, NEW_BRIEF] {
            let mut observed = passing.clone();
            observed.told[2].system = brief.as_bytes().into();
            rejects(&observed, SHAPE_INSTRUCTIONS, "chat's charter");
        }
        let mut observed = passing.clone();
        observed.told.retain(|told| told.agent != Agent::Shape);
        rejects(&observed, SHAPE_INSTRUCTIONS, "chat's charter");
    }
    #[test]
    fn the_numbered_question_and_its_delivery_are_checked() {
        let passing = met();
        let mut observed = passing.clone();
        observed.told[1] = told(Agent::Mover, OLD_BRIEF, &[]);
        rejects(&observed, SHAPE_INSTRUCTIONS, "numbered question");
        let mut observed = passing.clone();
        observed.told[2].history = Box::new([]);
        rejects(&observed, SHAPE_INSTRUCTIONS, "numbered question");
        let mut observed = passing.clone();
        observed.told.swap(2, 3);
        rejects(&observed, SHAPE_INSTRUCTIONS, "numbered question");
    }
    #[test]
    fn both_numbered_tasks_must_report_in_the_project() {
        let passing = met();
        for read in [Read::MoverEnded, Read::ShapeEnded] {
            for path in ["/chip/task", "/project", "/chip/phase", "/result/value/report/bytes"] {
                let mut observed = passing.clone();
                *observed.documents.get_mut(&read).expect("ended document").pointer_mut(path).expect("mutated field") =
                    Value::Null;
                rejects(&observed, SHAPE_ENDINGS, "original chat and A");
            }
            let mut observed = passing.clone();
            observed.documents.remove(&read);
            rejects(&observed, SHAPE_ENDINGS, "original chat and A");
        }
    }
}

// Act 2, step 5.2 (testing/reference-case.md)
const DELEGATES_RESUME: Promise = Promise::Must("a stopped goal holds its delegates and one release resumes them");
const STOPPED_NEWS: Promise =
    Promise::Add("a stopped goal's news waits for its release, so nothing is relayed by hand");

#[must_use]
pub fn step_5_2(observed: &Observed) -> Vec<Miss> {
    use super::{H1, H1_DOC, H1_WORDS};
    let mut misses = Vec::new();
    for read in [Read::BHeld, Read::H1Held, Read::H1ReviewerHeld] {
        step_check(
            &mut misses,
            "5.2",
            text(task_document(observed, read), "/chip/phase") == "held",
            DELEGATES_RESUME,
            format_args!("{read:?} must be held after the person's Stop"),
        );
    }
    let h1 = task_document(observed, Read::H1Ended);
    step_check(
        &mut misses,
        "5.2",
        text(h1, "/chip/phase") == "ended"
            && number(h1, "/result/ending/task/done/change/kind") == Some(1)
            && observed.kit.merged_green
            && observed.kit.main.get(H1_DOC.as_bytes()).is_some_and(|words| words == H1_WORDS.as_bytes()),
        DELEGATES_RESUME,
        "h1 must land green with its Markdown on kit's main after Release",
    );
    let label = format!("Result from task {H1}:\n");
    let queries: Vec<_> = observed.told.iter().filter(|query| query.agent == Agent::Bowner).collect();
    step_check(
        &mut misses,
        "5.2",
        queries.last().is_some_and(|query| calls::occurrences(&query.words, label.as_bytes()) == 1)
            && queries.iter().rev().skip(1).all(|query| !contains(&query.words, &label)),
        STOPPED_NEWS,
        "B must hear h1's ending once, after Release",
    );
    misses
}

fn pause_activity(observed: &Observed, read: Read) -> &Value {
    let doc = document(observed, read);
    doc.get("activity").unwrap_or(doc)
}

#[cfg(test)]
mod pause_tests {
    use super::*;
    use crate::case::{H1, H1_DOC, H1_WORDS};
    use serde_json::json;

    fn met() -> Observed {
        let documents = BTreeMap::from([
            (Read::BHeld, json!({"task":{"chip":{"phase":"held"}}})),
            (Read::H1Held, json!({"task":{"chip":{"phase":"held"}}})),
            (Read::H1ReviewerHeld, json!({"task":{"chip":{"phase":"held"}}})),
            (
                Read::H1Ended,
                json!({"task":{"chip":{"phase":"ended"},"result":{"ending":{"task":{"done":{"change":{"kind":1}}}}}}}),
            ),
        ]);
        let told = |words: String| Told {
            system: Box::new([]),
            agent: Agent::Bowner,
            words: words.into_bytes().into(),
            history: Box::new([]),
        };
        Observed {
            activity_aliases: BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents,
            told: vec![told("Delegated".into()), told(format!("Result from task {H1}:\nlanded"))],
            kit: Kit {
                records: Vec::new(),
                pulls: Vec::new(),
                merged_green: true,
                main: BTreeMap::from([(H1_DOC.as_bytes().to_vec(), H1_WORDS.as_bytes().to_vec())]),
            },
        }
    }

    #[test]
    fn pause_checkpoint_met() {
        assert_met(&step_5_2(&met()));
    }

    #[test]
    fn every_required_document_field_rejects_a_mutation() {
        let passing = met();
        for (read, path) in [
            (Read::BHeld, "/task/chip/phase"),
            (Read::H1Held, "/task/chip/phase"),
            (Read::H1ReviewerHeld, "/task/chip/phase"),
            (Read::H1Ended, "/task/chip/phase"),
            (Read::H1Ended, "/task/result/ending/task/done/change/kind"),
        ] {
            let mut observed = passing.clone();
            *observed.documents.get_mut(&read).expect("document").pointer_mut(path).expect("field") = Value::Null;
            assert!(!step_5_2(&observed).is_empty(), "{read:?}{path}");
        }
        for &(read, _) in crate::case::acts::READS_5_2 {
            let mut observed = passing.clone();
            observed.documents.remove(&read);
            assert!(!step_5_2(&observed).is_empty(), "missing {read:?}");
        }
    }

    #[test]
    fn news_cannot_arrive_early_twice_or_without_landing() {
        let passing = met();
        let mut early = passing.clone();
        early.told[0].words = early.told[1].words.clone();
        let mut missing = passing.clone();
        missing.told[1].words = Box::new([]);
        let mut twice = passing.clone();
        twice.told[1].words = format!("Result from task {H1}:\nResult from task {H1}:\n").into_bytes().into();
        for observed in [early, missing, twice] {
            assert!(!step_5_2(&observed).is_empty());
        }
    }

    #[test]
    fn stop_and_merge_are_checked() {
        let passing = met();
        let mut red = passing.clone();
        red.kit.merged_green = false;
        let mut missing_file = passing.clone();
        missing_file.kit.main.clear();
        for observed in [red, missing_file] {
            assert!(!step_5_2(&observed).is_empty());
        }
    }
}

// Histories repeat earlier turns. Count new user input once within a growing
// history, and count it again if delivered to another fresh conversation.
fn plan_deliveries(observed: &Observed, agent: Agent, words: &str) -> usize {
    use skein_fake_llm_domain::api::{Part, Role};
    let mut previous: &[skein_fake_llm_domain::api::Message] = &[];
    let mut count = 0;
    for told in observed.told.iter().filter(|told| told.agent == agent) {
        let offset = if told.history.starts_with(previous) { previous.len() } else { 0 };
        count += told.history[offset..]
            .iter()
            .filter(|message| message.role == Role::User)
            .flat_map(|message| &message.parts)
            .map(|part| match part {
                Part::Text { text } | Part::ToolOutput { output: text, .. } => {
                    calls::occurrences(text, words.as_bytes())
                }
                Part::Opaque { .. } | Part::ToolCall { .. } => 0,
            })
            .sum::<usize>();
        previous = &told.history;
    }
    count
}

fn plan_attempt_rows(observed: &Observed, reads: &[Read], task: u64, minimum: usize, rows: &[&Value]) -> bool {
    let complete = reads.iter().all(|read| pause_activity(observed, *read)["cut"] == false)
        && reads.last().is_some_and(|read| pause_activity(observed, *read)["before"] == Value::Null)
        && reads.first().is_some_and(|read| pause_activity(observed, *read)["after"] == Value::Null);
    let ordered = rows
        .windows(2)
        .all(|pair| number(pair[0], "/position").zip(number(pair[1], "/position")).is_some_and(|(new, old)| new > old));
    let mut open = None;
    let mut attempts = BTreeSet::new();
    let mut valid = complete && ordered && !rows.is_empty();
    for row in rows.iter().rev() {
        valid &= number(row, "/task") == Some(task);
        if let Some(began) = row["line"].get("run_began") {
            valid &= open.is_none();
            open = number(began, "/attempt");
            valid &= open.is_some_and(|attempt| attempts.insert(attempt));
        }
        if let Some(ended) = row["line"].get("run_ended") {
            valid &= open.is_some() && open.take() == number(ended, "/attempt");
            // A failed start's retry would hide it: no attempt may fail.
            valid &= ended["end"].get("failed").is_none();
        }
    }
    valid && open.is_none() && attempts.len() >= minimum
}

// Act 2, step 5.4 (testing/reference-case.md)
/// The forge's pull lifecycle, beside its ordered comment observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pull {
    pub repository: Vec<u8>,
    pub number: u64,
    pub branch: Vec<u8>,
    pub opened: u64,
    /// Last observed push of this branch before the pull opened.
    pub pushed: Option<u64>,
    /// The forge's merge commit, independent of the coder's result words.
    pub landed: Option<u64>,
    pub merged_head: Option<u64>,
    pub merged: bool,
    pub closed: bool,
    pub deleted: Option<u64>,
}
fn pull_digest(records: &[skein_fake_forge_domain::Observation]) -> Vec<Pull> {
    use skein_fake_forge_domain::Observation;
    let mut pulls = Vec::<Pull>::new();
    let mut heads = BTreeMap::new();
    for record in records {
        match record {
            Observation::Moved { repository, branch, to, .. } => {
                heads.insert((repository.to_vec(), branch.to_vec()), *to);
            }
            Observation::Opened { repository, number, branches: Some(branches), .. } => pulls.push(Pull {
                repository: repository.to_vec(),
                number: *number,
                branch: branches.head.to_vec(),
                opened: branches.commit,
                pushed: heads.get(&(repository.to_vec(), branches.head.to_vec())).copied(),
                landed: None,
                merged_head: None,
                merged: false,
                closed: false,
                deleted: None,
            }),
            Observation::Closed { repository, number, .. } => {
                if let Some(pull) =
                    pulls.iter_mut().find(|pull| pull.repository == repository.as_ref() && pull.number == *number)
                {
                    pull.closed = true;
                }
            }
            Observation::Merged { repository, number, head, commit, .. } => {
                if let Some(pull) =
                    pulls.iter_mut().find(|pull| pull.repository == repository.as_ref() && pull.number == *number)
                {
                    pull.merged = true;
                    pull.landed = Some(*commit);
                    pull.merged_head = Some(*head);
                }
            }
            Observation::Deleted { repository, branch, at, .. } => {
                if let Some(pull) = pulls
                    .iter_mut()
                    .rev()
                    .find(|pull| pull.repository == repository.as_ref() && pull.branch == branch.as_ref())
                {
                    pull.deleted = Some(*at);
                }
            }
            Observation::Opened { branches: None, .. }
            | Observation::Reopened { .. }
            | Observation::Labelled { .. }
            | Observation::Revised { .. }
            | Observation::Depends { .. }
            | Observation::Requested { .. }
            | Observation::Defined { .. }
            | Observation::Commented { .. }
            | Observation::Edited { .. }
            | Observation::Removed { .. }
            | Observation::Reviewed { .. }
            | Observation::Reported { .. }
            | Observation::Enqueued { .. }
            | Observation::Group { .. }
            | Observation::Dequeued { .. }
            | Observation::Refused { .. }
            | Observation::Rejected { .. }
            | Observation::Rebased { .. }
            | Observation::Wiki { .. } => {}
        }
    }
    pulls
}

const DIRECTION_CANCELLED: Promise = Promise::Must("cancel, with work kept under its closed pull");
const DIRECTION_OVERLAP: Promise = Promise::Add("a1 lands on B's file");
const DIRECTION_NEWS: Promise = Promise::Add("B's overlap news wakes it once released");

#[must_use]
pub fn step_5_4(observed: &Observed) -> Vec<Miss> {
    use super::{B_FILE, STORY_WORDS, X3_FILE};
    let mut misses = Vec::new();
    let cancelled = task_document(observed, Read::X3Ended);
    step_check(
        &mut misses,
        "5.4",
        text(cancelled, "/chip/phase") == "ended"
            && cancelled.pointer("/result/ending/task/cancelled").is_some_and(|value| !value.is_null()),
        DIRECTION_CANCELLED,
        "x3 must end cancelled",
    );
    let landed = task_document(observed, Read::StoryLanded);
    step_check(
        &mut misses,
        "5.4",
        text(landed, "/chip/phase") == "ended"
            && number(landed, "/result/ending/task/done/change/kind") == Some(1)
            && number(landed, "/result/value/change/part/connector") == Some(1)
            && text(landed, "/result/value/change/part/kind") == "result"
            && observed.kit.merged_green
            && observed.kit.main.get(B_FILE.as_bytes()).is_some_and(|words| words == STORY_WORDS.as_bytes())
            && !observed.kit.main.contains_key(X3_FILE.as_bytes()),
        DIRECTION_OVERLAP,
        "a1 must land green changing B's file, while main excludes x3's file",
    );
    direction_pulls(observed, &mut misses);
    direction_news(observed, &mut misses);
    misses
}

fn direction_pulls(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{DIRECTION, STORY_CHANGE, X3};
    let x3_branch = format!("temper/{DIRECTION}/c{X3}");
    let a1_branch = format!("temper/{DIRECTION}/c{STORY_CHANGE}");
    let x3 = observed.kit.pulls.iter().find(|pull| pull.branch == x3_branch.as_bytes());
    let a1 = observed.kit.pulls.iter().find(|pull| pull.branch == a1_branch.as_bytes());
    step_check(
        misses,
        "5.4",
        x3.is_some_and(|pull| {
            pull.closed
                && !pull.merged
                && pull.landed.is_none()
                && pull.merged_head.is_none()
                && pull.pushed == Some(pull.opened)
                && pull.deleted == pull.pushed
        }) && a1.is_some_and(|pull| {
            pull.merged
                && pull.pushed == Some(pull.opened)
                && pull.merged_head == pull.pushed
                && pull.landed.is_some_and(|commit| {
                    landing_commit(task_document(observed, Read::StoryLanded)) == direction_commit(commit)
                })
        }),
        DIRECTION_CANCELLED,
        "x3's closed unmerged pull must retain its pushed head, with its branch deleted; a1 must land its pushed head",
    );
    let produced = task_document(observed, Read::X3Produced);
    step_check(
        misses,
        "5.4",
        text(produced, "/chip/phase") == "ended"
            && number(produced, "/result/value/change/kind") == Some(2)
            && x3.is_some_and(|pull| pull.pushed == Some(pull.opened)),
        DIRECTION_CANCELLED,
        "x3's coder must finish a change, whose pull opens at its observed pushed head",
    );
}

fn direction_commit(commit: u64) -> String {
    use std::fmt::Write;
    let mut hex = String::new();
    for byte in &skein_fake_forge_forgejo::routes::object(commit).bytes[..20] {
        write!(hex, "{byte:02x}").expect("commit hex");
    }
    hex
}

fn direction_news(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::OVERLAP_HOLD_REVISION;
    let held = task_document(observed, Read::DirectionBHeld);
    let pending = task_document(observed, Read::DirectionBPending);
    for (read, doc) in [(Read::DirectionBHeld, held), (Read::DirectionBPending, pending)] {
        step_check(
            misses,
            "5.4",
            text(doc, "/chip/phase") == "held"
                && number(doc, "/hold/revision") == Some(OVERLAP_HOLD_REVISION)
                && number(doc, "/hold/reason/placed/hold").is_some_and(|hold| hold != 0)
                && number(doc, "/hold/reason/placed/hold") == number(held, "/hold/reason/placed/hold"),
            DIRECTION_NEWS,
            format_args!("{read:?}: B must remain held at the person's Stop"),
        );
    }
    let rows = check_activity(observed, Read::DirectionBEnded);
    let release = rows
        .iter()
        .find(|row| number(row, "/line/decided/revision") == Some(OVERLAP_HOLD_REVISION))
        .and_then(|row| number(row, "/position"));
    let landing = landing_commit(task_document(observed, Read::StoryLanded));
    let news: Vec<_> = rows
        .iter()
        .filter(|row| {
            text(row, "/line/heard/kind/news/class") == "wakes"
                && text(row, "/line/heard/words").starts_with("forge landing ")
        })
        .collect();
    step_check(
        misses,
        "5.4",
        news.len() == 1
            && news.first().is_some_and(|row| {
                text(row, "/line/heard/words").ends_with(&format!(" -> {landing}"))
                    && plan_deliveries(observed, Agent::OverlapOwner, text(row, "/line/heard/words")) == 1
            })
            && release.zip(number(pending, "/activity")).is_some_and(|(release, head)| release > head)
            && rows.iter().any(|row| {
                row.pointer("/line/run_began").is_some()
                    && number(row, "/position").zip(release).is_some_and(|(at, release)| at > release)
            }),
        DIRECTION_NEWS,
        "a1's overlapping landing must wake B once after release",
    );
}

#[cfg(test)]
mod direction_tests {
    use super::*;
    use crate::case::*;
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    fn met() -> Observed {
        let news = format!("forge landing {} -> {}", direction_commit(8), direction_commit(9));
        let documents = BTreeMap::from([
            (Read::X3Produced, json!({"task":{"chip":{"phase":"ended"},"result":{"value":{"change":{"kind":2}}}}})),
            (Read::X3Ended, json!({"task":{"chip":{"phase":"ended"},"result":{"ending":{"task":{"cancelled":{}}}}}})),
            (
                Read::StoryLanded,
                json!({"task":{"chip":{"phase":"ended"},"result":{
                "ending":{"task":{"done":{"change":{"kind":1}}}},
                "value":{"change":{"part":{"connector":1,"kind":"result"},"words":{"bytes":direction_commit(9)}}}}}}),
            ),
            (
                Read::DirectionBHeld,
                json!({"task":{"chip":{"phase":"held"},"hold":{"revision":1,"reason":{"placed":{"hold":1}}}}}),
            ),
            (
                Read::DirectionBPending,
                json!({"task":{"chip":{"phase":"held"},"activity":4,"hold":{"revision":1,"reason":{"placed":{"hold":1}}}}}),
            ),
            (
                Read::DirectionBEnded,
                json!({"activity":{"rows":[
                    {"event":{"owner":{"source":{"position":7,"line":{"run_began":{"attempt":2}}}}}},
                    {"event":{"owner":{"source":{"position":6,"line":{"decided":{"revision":1}}}}}},
                    {"event":{"owner":{"source":{"position":5,"line":{"heard":{"kind":{"news":{"class":"wakes"}},"words":news}}}}}}
                ]}}),
            ),
        ]);
        let message = Message { role: Role::User, parts: Box::new([Part::Text { text: news.as_bytes().into() }]) };
        let records = forge_records();
        Observed {
            activity_aliases: BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents,
            told: vec![Told::with_agent(Agent::OverlapOwner, Agent::OverlapOwner.cue(), &[message])],
            kit: Kit {
                pulls: pull_digest(&records),
                records,
                main: BTreeMap::from([(B_FILE.as_bytes().to_vec(), STORY_WORDS.as_bytes().to_vec())]),
                merged_green: true,
            },
        }
    }

    #[test]
    fn pushed_cancellation_and_released_overlap_meet_the_checkpoint() {
        assert_met(&step_5_4(&met()));
    }

    #[test]
    fn retained_document_fields_reject_a_mutation() {
        let passing = met();
        for (read, paths) in [
            (Read::X3Produced, vec!["/task/chip/phase", "/task/result/value/change/kind"]),
            (Read::X3Ended, vec!["/task/chip/phase", "/task/result/ending/task/cancelled"]),
            (
                Read::StoryLanded,
                vec![
                    "/task/chip/phase",
                    "/task/result/ending/task/done/change/kind",
                    "/task/result/value/change/part/connector",
                    "/task/result/value/change/part/kind",
                    "/task/result/value/change/words/bytes",
                ],
            ),
            (Read::DirectionBHeld, vec!["/task/chip/phase", "/task/hold/revision", "/task/hold/reason/placed/hold"]),
            (
                Read::DirectionBPending,
                vec!["/task/chip/phase", "/task/hold/revision", "/task/hold/reason/placed/hold", "/task/activity"],
            ),
        ] {
            for path in paths {
                let mut observed = passing.clone();
                *observed.documents.get_mut(&read).expect("read").pointer_mut(path).expect("field") = Value::Null;
                assert!(!step_5_4(&observed).is_empty(), "{read:?}{path}");
            }
        }
    }

    #[test]
    fn news_must_be_told_once_after_release() {
        let passing = met();
        let mut lost = passing.clone();
        lost.told.clear();
        let mut doubled = passing.clone();
        doubled.told[0].history = vec![doubled.told[0].history[0].clone(); 2].into();
        let mut early = passing.clone();
        early.documents.get_mut(&Read::DirectionBPending).expect("held")["task"]["activity"] = json!(8);
        for observed in [lost, doubled, early] {
            assert!(!step_5_4(&observed).is_empty());
        }
    }

    #[test]
    fn the_closed_pull_keeps_the_pushed_head_and_main_excludes_its_file() {
        let passing = met();
        for mutation in 0..8 {
            let mut observed = passing.clone();
            match mutation {
                0 => {
                    observed.kit.pulls.remove(0);
                }
                1 => observed.kit.pulls[0].closed = false,
                2 => observed.kit.pulls[0].merged = true,
                3 => observed.kit.pulls[0].deleted = None,
                4 => observed.kit.pulls[0].deleted = Some(99),
                5 => observed.kit.pulls[0].opened = 99,
                6 => observed.kit.pulls[1].merged = false,
                7 => {
                    observed.kit.main.insert(X3_FILE.as_bytes().to_vec(), X3_WORDS.as_bytes().to_vec());
                }
                _ => unreachable!(),
            }
            assert!(!step_5_4(&observed).is_empty(), "mutation {mutation}");
        }
    }

    fn forge_records() -> Vec<skein_fake_forge_domain::Observation> {
        use skein_fake_forge_domain::{Branches, Observation};
        let repository: Box<[u8]> = b"acme/app".as_slice().into();
        let mut records = Vec::new();
        for (task, number, commit) in [(X3, X3_PULL_NUMBER, 7), (STORY_CHANGE, STORY_PULL_NUMBER, 8)] {
            let branch = format!("temper/{DIRECTION}/c{task}").into_bytes().into_boxed_slice();
            records.push(Observation::Moved {
                repository: repository.clone(),
                branch: branch.clone(),
                from: None,
                to: commit,
                by: 1,
            });
            records.push(Observation::Opened {
                repository: repository.clone(),
                number,
                kind: skein_fake_forge_domain::api::Kind::Pull,
                title: Box::new([]),
                body: Box::new([]),
                labels: Box::new([]),
                branches: Some(Branches { head: branch.clone(), base: b"main".as_slice().into(), commit }),
                by: 1,
            });
            if task == X3 {
                records.push(Observation::Closed { repository: repository.clone(), number, by: 1 });
            } else {
                records.push(Observation::Merged {
                    repository: repository.clone(),
                    number,
                    base: b"main".as_slice().into(),
                    head: commit,
                    commit: 9,
                    by: 1,
                });
            }
            records.push(Observation::Deleted { repository: repository.clone(), branch, at: commit, by: 1 });
        }
        records
    }

    #[test]
    fn missing_reordered_or_changed_forge_observations_fail_the_checkpoint() {
        use skein_fake_forge_domain::Observation;
        let passing = met();
        for mutation in 0..9 {
            let mut records = forge_records();
            match mutation {
                0 => {
                    records.remove(0);
                }
                1 => {
                    if let Observation::Moved { to, .. } = &mut records[0] {
                        *to = 99;
                    }
                }
                2 => records.swap(0, 1),
                3 => {
                    if let Observation::Closed { number, .. } = &mut records[2] {
                        *number = 99;
                    }
                }
                4 => {
                    if let Observation::Deleted { at, .. } = &mut records[3] {
                        *at = 99;
                    }
                }
                5 => {
                    if let Observation::Merged { head, .. } = &mut records[6] {
                        *head = 99;
                    }
                }
                6 => {
                    if let Observation::Merged { commit, .. } = &mut records[6] {
                        *commit = 99;
                    }
                }
                7 => {
                    records.remove(4);
                }
                8 => {
                    if let Observation::Opened { branches: Some(branches), .. } = &mut records[5] {
                        branches.commit = 99;
                    }
                }
                _ => unreachable!(),
            }
            let mut observed = passing.clone();
            observed.kit.pulls = pull_digest(&records);
            assert!(!step_5_4(&observed).is_empty(), "record mutation {mutation}");
        }
    }

    #[test]
    fn pull_digest_correlates_push_open_close_delete_and_merge_in_record_order() {
        let observed = met();
        assert_eq!(observed.kit.pulls[0].pushed, Some(7));
        assert_eq!(observed.kit.pulls[1].pushed, Some(8));
        assert_eq!(observed.kit.pulls[1].merged_head, Some(8));
        assert_eq!(observed.kit.pulls[1].landed, Some(9));
        assert_met(&step_5_4(&observed));
    }
}

#[cfg(test)]
mod qualified_pull_tests {
    use super::{Pull, pull_digest};
    use skein_fake_forge_domain::{Branches, Observation};

    #[test]
    fn a_pulls_facts_stay_in_their_repository() {
        // Equal pull numbers and branch names are local to their repository.
        const KIT: &[u8] = b"acme/kit";
        const APP: &[u8] = b"acme/app";
        const BRANCH: &[u8] = b"temper/1/c3";
        let records = [
            Observation::Moved { repository: KIT.into(), branch: BRANCH.into(), from: None, to: 11, by: 1 },
            Observation::Moved { repository: APP.into(), branch: BRANCH.into(), from: None, to: 22, by: 1 },
            Observation::Opened {
                repository: KIT.into(),
                number: 1,
                kind: skein_fake_forge_domain::api::Kind::Pull,
                title: Box::new([]),
                body: Box::new([]),
                labels: Box::new([]),
                branches: Some(Branches { head: BRANCH.into(), base: b"main".as_slice().into(), commit: 11 }),
                by: 1,
            },
            Observation::Opened {
                repository: APP.into(),
                number: 1,
                kind: skein_fake_forge_domain::api::Kind::Pull,
                title: Box::new([]),
                body: Box::new([]),
                labels: Box::new([]),
                branches: Some(Branches { head: BRANCH.into(), base: b"main".as_slice().into(), commit: 22 }),
                by: 1,
            },
        ];
        // Each row adds one kind of fact. Its expected repository-local
        // state is data: closed, merged (head, commit), deleted-at.
        let rows = [
            ("push/open", None, [(false, None, None), (false, None, None)]),
            (
                "close",
                Some(Observation::Closed { repository: APP.into(), number: 1, by: 1 }),
                [(false, None, None), (true, None, None)],
            ),
            (
                "merge",
                Some(Observation::Merged {
                    repository: APP.into(),
                    number: 1,
                    base: b"main".as_slice().into(),
                    head: 22,
                    commit: 33,
                    by: 1,
                }),
                [(false, None, None), (false, Some((22, 33)), None)],
            ),
            (
                "delete",
                Some(Observation::Deleted { repository: APP.into(), branch: BRANCH.into(), at: 22, by: 1 }),
                [(false, None, None), (false, None, Some(22))],
            ),
        ];
        for (name, tail, states) in rows {
            let mut input = records.to_vec();
            input.extend(tail);
            let expected = [(KIT, 11), (APP, 22)]
                .into_iter()
                .zip(states)
                .map(|((repository, head), (closed, merge, deleted))| Pull {
                    repository: repository.to_vec(),
                    number: 1,
                    branch: BRANCH.to_vec(),
                    opened: head,
                    pushed: Some(head),
                    closed,
                    merged: merge.is_some(),
                    merged_head: merge.map(|(head, _)| head),
                    landed: merge.map(|(_, commit)| commit),
                    deleted,
                })
                .collect::<Vec<_>>();
            assert_eq!(pull_digest(&input), expected, "{name}");
        }
    }
}

fn batch_journal(observed: &Observed, read: Read, task: u64) -> bool {
    batch_journal_pages(observed, &[read], task)
}
fn batch_journal_pages(observed: &Observed, reads: &[Read], task: u64) -> bool {
    let rows: Vec<_> = joined_activity(observed, reads);
    plan_attempt_rows(observed, reads, task, 1, &rows)
        && rows.last().is_some_and(|row| number(row, "/position") == Some(1))
        && rows.windows(2).all(|pair| {
            number(pair[0], "/position")
                .zip(number(pair[1], "/position"))
                .is_some_and(|(new, old)| new.checked_sub(1) == Some(old))
        })
        && rows.iter().enumerate().all(|(at, row)| {
            let Some(end) = row.pointer("/line/run_ended") else { return true };
            matches!(end["end"].as_str(), Some("parked" | "finished"))
                || (end["end"] == "refused"
                    && number(end, "/spent") == Some(0)
                    && number(end, "/attempt").is_some_and(|attempt| {
                        !rows.iter().any(|r| number(r, "/line/turn/attempt") == Some(attempt))
                    })
                    // Rows are newest first: a finished attempt must follow
                    // this zero-spend placement recheck in commit order.
                    && rows[..at].iter().any(|r| text(r, "/line/run_ended/end") == "finished"))
        })
}
fn batch_done(observed: &Observed, read: Read, task: u64) -> bool {
    let doc = task_document(observed, read);
    number(doc, "/chip/task") == Some(task)
        && text(doc, "/chip/phase") == "ended"
        && doc.pointer("/result/ending/task/done").is_some_and(Value::is_object)
        && batch_journal(observed, read, task)
}

// Act 3, step 6.4 (testing/reference-case.md)
const RED_ORDER: Promise = Promise::Add("nothing lands on a red main before its repair");
const RED_DEPLOYMENT: Promise = Promise::Must("one deployment-requested landing-branch repair");
const RED_LANDING: Promise = Promise::Must("a4 lands with green CI after the repair");
const RED_ATTEMPTS: Promise = Promise::Add("every attempt ended, none failed");

fn red_tasks() -> [(Read, u64, u64); 6] {
    use super::{RED_CHANGE, RED_REPAIR, RED_REPAIR_CODER, RED_REPAIR_REVIEW, RED_REVIEW, RED_UPDATED_REVIEW};
    [
        (Read::RedChangeEnded, RED_CHANGE, 0),
        (Read::RedReviewEnded, RED_REVIEW, RED_CHANGE),
        (Read::RedRepairEnded, RED_REPAIR, 0),
        (Read::RedRepairCoderEnded, RED_REPAIR_CODER, RED_REPAIR),
        (Read::RedRepairReviewEnded, RED_REPAIR_REVIEW, RED_REPAIR),
        (Read::RedUpdatedReviewEnded, RED_UPDATED_REVIEW, RED_CHANGE),
    ]
}

#[must_use]
pub fn step_6_4(observed: &Observed) -> Vec<Miss> {
    use super::{PROJECT, RED_CHANGE, RED_REPAIR, RED_REVISION};
    let mut misses = Vec::new();
    for (read, task, parent) in red_tasks() {
        let doc = task_document(observed, read);
        step_check(
            &mut misses,
            "6.4",
            number(doc, "/chip/task") == Some(task)
                && number(doc, "/project") == Some(PROJECT)
                && number(doc, "/spec_revision/value") == Some(RED_REVISION)
                && text(doc, "/chip/phase") == "ended"
                && (parent == 0 || number(doc, "/requester/task") == Some(parent)),
            RED_DEPLOYMENT,
            format_args!("{read:?} identifies ended task {task} under {parent}"),
        );
        if task != RED_CHANGE && task != RED_REPAIR {
            step_check(
                &mut misses,
                "6.4",
                batch_done(observed, read, task),
                RED_ATTEMPTS,
                format_args!("task {task}'s journal ends every attempt; zero-spend rechecks are unbounded"),
            );
        }
    }
    red_landings(observed, &mut misses);
    let pending = document(observed, Read::RedRootsPending);
    step_check(
        &mut misses,
        "6.4",
        pending.pointer("/tasks/more") == Some(&Value::Bool(false))
            && pending.pointer("/tasks/rows").and_then(Value::as_array).is_some_and(|rows| {
                let repairs: Vec<_> =
                    rows.iter().filter(|row| row.pointer("/requester/deployment").is_some()).collect();
                repairs.len() == 1
                    && number(repairs[0], "/chip/task") == Some(RED_REPAIR)
                    && number(repairs[0], "/requester/deployment/project") == Some(PROJECT)
            }),
        RED_DEPLOYMENT,
        "one deployment repair among the pending roots",
    );
    misses
}

fn red_landings(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{
        RED_A4, RED_A4_FILE, RED_A4_PULL, RED_BASE, RED_CHANGE, RED_FILE, RED_FIRST_HEAD, RED_REPAIR, RED_REPAIR_PULL,
        RED_UPDATED_HEAD,
    };
    use skein_fake_forge_domain::{Observation, api};
    let records = &observed.kit.records;
    let pushes: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, record)| {
            let Observation::Moved { repository, branch, to, by, .. } = record else { return None };
            (repository.as_ref() == b"acme/app" && branch.as_ref() == b"main" && *by == crate::forge::OTHER)
                .then_some((at, *to))
        })
        .collect();
    let merges: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, record)| {
            let Observation::Merged { repository, base, number, head, commit, .. } = record else { return None };
            Some((at, repository.as_ref(), base.as_ref(), *number, *head, *commit))
        })
        .collect();
    let green = |at: usize, head: u64| {
        records[..at].iter().rev().find_map(|record| {
            let Observation::Reported { repository, commit, context, state, .. } = record else { return None };
            (repository.as_ref() == b"acme/app" && *commit == head && context.as_ref() == b"check").then_some(*state)
        }) == Some(api::Check::Passed)
    };
    let pull_for = |task: u64, root: u64| {
        observed.kit.pulls.iter().find(|pull| pull.branch == format!("temper/{root}/c{task}").as_bytes())
    };
    let repair = pull_for(RED_REPAIR, RED_REPAIR);
    let a4 = pull_for(RED_CHANGE, RED_CHANGE);
    let opened_before_push = a4.is_some_and(|pull| {
        pushes.first().is_some_and(|(pushed, _)| {
            records[..*pushed].iter().any(|record| {
                matches!(record,
            Observation::Opened { number, branches: Some(_), .. } if *number == pull.number)
            })
        })
    });
    step_check(
        misses,
        "6.4",
        pushes.len() == 1
            && merges.len() == 2
            && opened_before_push
            && pushes.first().zip(merges.first()).is_some_and(|((pushed, red), first)| {
                *pushed < first.0
                    && records[*pushed..first.0].iter().any(|record| {
                        matches!(record,
                Observation::Reported { commit, state: api::Check::Failed, .. } if commit == red)
                    })
            })
            && repair.is_some_and(|pull| {
                pull.number == RED_REPAIR_PULL
                    && merges.first().is_some_and(|m| m.3 == pull.number && pull.landed == Some(m.5))
            })
            && a4.is_some_and(|pull| {
                pull.number == RED_A4_PULL
                    && merges.get(1).is_some_and(|m| m.3 == pull.number && pull.landed == Some(m.5))
            }),
        RED_ORDER,
        "the person's failed main push must precede exactly two forge-ordered merges: repair, then a4",
    );
    step_check(
        misses,
        "6.4",
        merges.len() == 2
            && observed.kit.pulls.len() == 2
            && observed.kit.merged_green
            && a4.is_some_and(|pull| pull.opened == RED_FIRST_HEAD && pull.merged_head == Some(RED_UPDATED_HEAD))
            && merges.iter().all(|m| m.1 == b"acme/app" && m.2 == b"main" && green(m.0, m.4)),
        RED_LANDING,
        "each merge must use its green head after a4 updates onto repaired main",
    );
    step_check(
        misses,
        "6.4",
        observed.kit.main.get(RED_FILE.as_bytes()).is_some_and(|bytes| bytes == RED_BASE.as_bytes())
            && observed.kit.main.get(RED_A4_FILE.as_bytes()).is_some_and(|bytes| bytes == RED_A4.as_bytes()),
        RED_LANDING,
        "main holds the repair and a4's story files",
    );
}

#[cfg(test)]
mod red_tests {
    use super::*;
    use crate::case::*;
    use serde_json::json;
    use skein_fake_forge_domain::{Observation, api};

    fn journal(task: u64, lines: &[Value]) -> Value {
        let rows: Vec<_> = lines
            .iter()
            .enumerate()
            .rev()
            .map(|(at, line)| json!({"event":{"owner":{"source":{"task":task,"position":at+1,"line":line}}}}))
            .collect();
        json!({"rows":rows,"cut":false,"before":null,"after":null})
    }
    fn met_documents() -> BTreeMap<Read, Value> {
        let mut documents = BTreeMap::new();
        for (read, task, parent) in red_tasks() {
            documents.insert(
                read,
                json!({"task":{
                    "chip":{"task":task,"phase":"ended"},"project":PROJECT,"spec_revision":{"value":RED_REVISION},
                    "requester":{"task":parent},"result":{"ending":{"task":{"done":{}}}}
                },"activity":journal(task, &[
                    json!({"run_began":{"attempt":1}}),json!({"run_ended":{"attempt":1,"end":"finished","spent":1}})
                ])}),
            );
        }
        documents.insert(
            Read::RedRootsPending,
            json!({"tasks":{"more":false,"rows":[{
                "chip":{"task":RED_REPAIR},"requester":{"deployment":{"project":PROJECT}}
            }]}}),
        );
        documents
    }
    fn met_kit() -> Kit {
        let repo: Box<[u8]> = b"acme/app".as_slice().into();
        let mut records = vec![
            Observation::Moved {
                repository: repo.clone(),
                branch: b"main".as_slice().into(),
                from: Some(1),
                to: 3,
                by: crate::forge::OTHER,
            },
            Observation::Reported {
                repository: repo.clone(),
                commit: 3,
                context: b"check".as_slice().into(),
                state: api::Check::Failed,
                by: 1,
            },
            Observation::Reported {
                repository: repo.clone(),
                commit: 4,
                context: b"check".as_slice().into(),
                state: api::Check::Passed,
                by: 1,
            },
            Observation::Merged {
                repository: repo.clone(),
                base: b"main".as_slice().into(),
                number: RED_REPAIR_PULL,
                head: 4,
                commit: 5,
                by: 1,
            },
            Observation::Reported {
                repository: repo.clone(),
                commit: 6,
                context: b"check".as_slice().into(),
                state: api::Check::Passed,
                by: 1,
            },
            Observation::Merged {
                repository: repo,
                base: b"main".as_slice().into(),
                number: RED_A4_PULL,
                head: 6,
                commit: 7,
                by: 1,
            },
        ];
        records.insert(
            0,
            Observation::Opened {
                repository: b"acme/app".as_slice().into(),
                number: RED_A4_PULL,
                kind: api::Kind::Pull,
                title: Box::new([]),
                body: Box::new([]),
                labels: Box::new([]),
                branches: Some(skein_fake_forge_domain::Branches {
                    head: format!("temper/{RED_CHANGE}/c{RED_CHANGE}").into_bytes().into(),
                    base: b"main".as_slice().into(),
                    commit: 2,
                }),
                by: 1,
            },
        );
        let pulls = [(RED_CHANGE, RED_CHANGE, RED_A4_PULL, 6, 7), (RED_REPAIR, RED_REPAIR, RED_REPAIR_PULL, 4, 5)]
            .into_iter()
            .map(|(task, root, number, head, merge)| Pull {
                repository: crate::case::KIT.as_bytes().to_vec(),
                number,
                branch: format!("temper/{root}/c{task}").into_bytes(),
                opened: if task == RED_CHANGE { RED_FIRST_HEAD } else { head },
                pushed: Some(head),
                landed: Some(merge),
                merged_head: Some(head),
                merged: true,
                closed: false,
                deleted: None,
            })
            .collect();
        Kit {
            records,
            pulls,
            merged_green: true,
            main: BTreeMap::from([
                (RED_FILE.as_bytes().to_vec(), RED_BASE.as_bytes().to_vec()),
                (RED_A4_FILE.as_bytes().to_vec(), RED_A4.as_bytes().to_vec()),
            ]),
        }
    }
    fn met() -> Observed {
        Observed {
            documents: met_documents(),
            kit: met_kit(),
            activity_aliases: BTreeSet::new(),
            observed_at: BTreeMap::new(),
            told: Vec::new(),
        }
    }
    fn rejects(observed: &Observed, promise: Promise) {
        assert!(step_6_4(observed).iter().any(|miss| miss.promise == promise), "mutation must fail {promise:?}");
    }
    #[test]
    fn red_observation_meets_the_step() {
        assert_met(&step_6_4(&met()));
    }
    #[test]
    fn missing_reversed_extra_or_premature_merges_fail() {
        let fixture = met();
        for mutation in 0..6 {
            let mut bad = fixture.clone();
            match mutation {
                0 => {
                    bad.kit.records.remove(0);
                }
                1 => bad.kit.records.swap(4, 6),
                2 => bad.kit.records.insert(3, bad.kit.records[6].clone()),
                3 => bad.kit.records.swap(1, 4),
                4 => {
                    bad.kit.records.remove(2);
                }
                _ => bad.kit.pulls[1].landed = Some(99),
            }
            rejects(&bad, RED_ORDER);
        }
    }
    #[test]
    fn each_head_must_have_passed_ci_before_its_own_merge() {
        let fixture = met();
        for at in [3, 5] {
            let mut bad = fixture.clone();
            if let Observation::Reported { state, .. } = &mut bad.kit.records[at] {
                *state = api::Check::Failed;
            }
            rejects(&bad, RED_LANDING);
            let mut bad = fixture.clone();
            bad.kit.records.swap(at, at + 1);
            rejects(&bad, RED_LANDING);
        }
        let mut bad = fixture.clone();
        bad.kit.main.clear();
        rejects(&bad, RED_LANDING);
    }
    #[test]
    fn another_deployment_repair_or_a_partial_task_list_fails() {
        let mut bad = met();
        let extra = bad.documents[&Read::RedRootsPending]["tasks"]["rows"][0].clone();
        bad.documents.get_mut(&Read::RedRootsPending).expect("pending")["tasks"]["rows"]
            .as_array_mut()
            .expect("roots")
            .push(extra);
        rejects(&bad, RED_DEPLOYMENT);
        let mut bad = met();
        bad.documents.get_mut(&Read::RedRootsPending).expect("pending")["tasks"]["more"] = json!(true);
        rejects(&bad, RED_DEPLOYMENT);
    }
    #[test]
    fn failed_or_incomplete_agent_attempts_are_rejected() {
        for end in [json!({"failed":{}}), json!("refused"), Value::Null] {
            let mut bad = met();
            bad.documents.get_mut(&Read::RedReviewEnded).expect("review")["activity"] = journal(
                RED_REVIEW,
                &[json!({"run_began":{"attempt":1}}), json!({"run_ended":{"attempt":1,"end":end,"spent":1}})],
            );
            rejects(&bad, RED_ATTEMPTS);
        }
        let mut bad = met();
        bad.documents.get_mut(&Read::RedReviewEnded).expect("review")["activity"]["cut"] = json!(true);
        rejects(&bad, RED_ATTEMPTS);
    }
    #[test]
    fn zero_spend_rechecks_have_no_count_bound() {
        let mut accepted = met();
        let mut lines = Vec::new();
        for attempt in 1..=8 {
            lines.push(json!({"run_began":{"attempt":attempt}}));
            lines.push(json!({"run_ended":{"attempt":attempt,"end":"refused","spent":0}}));
        }
        lines.push(json!({"run_began":{"attempt":9}}));
        lines.push(json!({"run_ended":{"attempt":9,"end":"finished","spent":1}}));
        accepted.documents.get_mut(&Read::RedReviewEnded).expect("review")["activity"] = journal(RED_REVIEW, &lines);
        assert_met(&step_6_4(&accepted));
        lines[1]["run_ended"]["spent"] = json!(1);
        accepted.documents.get_mut(&Read::RedReviewEnded).expect("review")["activity"] = journal(RED_REVIEW, &lines);
        rejects(&accepted, RED_ATTEMPTS);
    }
}

// Act 3, step 6.7 (testing/reference-case.md)
fn complete_call_part<'a>(call: &'a Value, part: &str) -> Option<&'a str> {
    let part = call.get(part)?;
    (part["from"] == 0 && part["bytes_left"] == 0).then(|| part["text"].as_str()).flatten()
}

// Act 3, step 6.6 (testing/reference-case.md).
const WAITING_LANDING: Promise = Promise::Must("each repository merges a green head");
const WAITING_ORDER: Promise = Promise::Must("kit merge precedes repin's first push and app merge");
const WAITING_PIN: Promise = Promise::Must("app pins the actual kit merge");

// A completed tool is an assistant call plus the numbered person result in
// the same committed turn. Resumed LLM histories can contain only a recap;
// the decoded journal retains the original arguments and acknowledgement.
fn waiting_tools(observed: &Observed, read: Read) -> Vec<(u64, Value, String, String)> {
    let mut tools = Vec::new();
    let mut identities = BTreeSet::new();
    let Some(rows) = document(observed, read).pointer("/activity/rows").and_then(Value::as_array) else { return tools };
    for row in rows.iter().rev() {
        let source = &row["event"]["owner"]["source"];
        let decoded = &row["event"]["owner"]["decoded"]["turn"]["decoded"];
        let Some(messages) = decoded["messages"].as_array() else { continue };
        if decoded["cut"] != false {
            continue;
        }
        for message in messages.iter().filter(|m| m["speaker"] == "assistant") {
            let Some(blocks) = message["blocks"].as_array() else { continue };
            for block in blocks {
                let Some(call) = block.get("call") else { continue };
                let (Some(input), Some(outcome), Some(name), Some(attempt), Some(turn), Some(at)) = (
                    complete_call_part(call, "input"),
                    complete_call_part(call, "outcome"),
                    number(call, "/number"),
                    number(source, "/line/turn/attempt"),
                    number(source, "/line/turn/turn"),
                    number(source, "/position"),
                ) else {
                    continue;
                };
                let acknowledged = messages
                    .iter()
                    .filter(|m| m["speaker"] == "person")
                    .filter_map(|m| m["blocks"].as_array())
                    .flatten()
                    .filter_map(|b| b.get("result"))
                    .filter(|r| {
                        number(r, "/call") == Some(name)
                            && r["failed"] == false
                            && complete_call_part(r, "summary") == Some(outcome)
                    })
                    .count()
                    == 1;
                if call["failed"] == false
                    && acknowledged
                    && identities.insert((attempt, turn, name))
                    && let Ok(input) = serde_json::from_str(input)
                {
                    tools.push((at, input, text(call, "/tool").into(), outcome.into()));
                }
            }
        }
    }
    tools
}
fn waiting_parameters(doc: &Value, repository: u32, path: &str, parameter: u64) -> bool {
    let expected = serde_json::json!([
        {"resource":{"connector":1,"name":1,"resource":temper_engine_domain_forge::tag(1, repository)}},
        {"bytes":{"name":2,"value":{"bytes":"main","cut":false}}},
        {"bytes":{"name":parameter,"value":{"bytes":path,"cut":false}}}
    ]);
    doc.pointer("/spec/specification/parameters") == Some(&expected)
}
fn waiting_batch_member(member: &Value, repository: u32, spec: &str, path: &str) -> bool {
    member["executor"] == serde_json::json!({"kind":"procedure","connector":1,"code":2})
        && member.pointer("/spec/words") == Some(&Value::String(spec.into()))
        && member.pointer("/spec/parameters")
            == Some(&serde_json::json!([
                {"name":1,"kind":"resource","connector":1,"resource":temper_engine_domain_forge::tag(1,repository)},
                {"name":2,"kind":"bytes","value":"main"},{"name":6,"kind":"bytes","value":path}
            ]))
        && member["contract"] == serde_json::json!({"kind":"change","connector":1,"change_kind":1,"words":128})
}
fn waiting_delivery(observed: &Observed, read: Read, from: u64, words: &str, agent: Agent) -> bool {
    let rows = check_activity(observed, read);
    let heard: Vec<_> = rows
        .iter()
        .filter(|r| number(r, "/line/heard/from/task") == Some(from) && r.pointer("/line/heard/kind/result").is_some())
        .collect();
    heard.len() == 1
        && text(heard[0], "/line/heard/words") == words
        && number(heard[0], "/line/heard/number").is_some_and(|n| n > 0)
        && plan_deliveries(observed, agent, &format!("Result from task {from}:\n{words}")) == 1
        && number(heard[0], "/position").is_some_and(|at| {
            waiting_tools(observed, read).iter().any(|(end, input, tool, output)| {
                *end >= at && tool == "finish" && input["form"] == "report" && output == "accepted"
            })
        })
}

fn waiting_head(head: u64) -> String {
    use std::fmt::Write;
    let mut hex = String::new();
    for byte in &skein_fake_forge_forgejo::routes::object(head).bytes[..20] {
        write!(hex, "{byte:02x}").expect("head hex");
    }
    hex
}
fn waiting_review(observed: &Observed, read: Read, task: u64, head: u64) -> bool {
    let review = task_document(observed, read);
    let cue = format!("Pull request at head {}", waiting_head(head));
    let requester = format!("[Requested by task {task}]");
    number(review, "/result/value/owner_verdict/verdict") == Some(1)
        && text(review, "/result/value/owner_verdict/words/bytes") == super::REVIEWED
        && observed.told.iter().any(|t| {
            t.agent == Agent::Review
                && std::str::from_utf8(&t.system)
                    .is_ok_and(|s| s.lines().any(|l| l == cue) && s.lines().any(|l| l == requester))
        })
        && waiting_tools(observed, read).iter().any(|(_, args, tool, output)| {
            tool == "finish"
                && args
                    == &serde_json::json!({"fields":{},"form":"verdict","items":[],"label":"1","text":super::REVIEWED})
                && output == "accepted"
        })
}
#[must_use]
pub fn step_6_6(observed: &Observed) -> Vec<Miss> {
    use super::waiting::{APP, KIT, REPIN, STARTER};
    use skein_fake_forge_domain::{Observation, api};
    let records = &observed.kit.records;
    let merges: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(at, row)| {
            if let Observation::Merged { repository, base, head, commit, .. } = row {
                Some((at, repository.as_ref(), base.as_ref(), *head, *commit))
            } else {
                None
            }
        })
        .collect();
    let kit = merges.iter().find(|merge| merge.1 == KIT.as_bytes());
    let app = merges.iter().find(|merge| merge.1 == APP.as_bytes());
    let first_push = records.iter().position(|row| {
        matches!(row,
        Observation::Moved { repository, branch, .. }
        if repository.as_ref() == APP.as_bytes() && branch.as_ref() == format!("temper/{STARTER}/c{REPIN}").as_bytes())
    });
    let mut misses = Vec::new();
    step_check(
        &mut misses,
        "6.6",
        merges.len() == 2
            && kit.zip(app).is_some_and(|(kit, app)| first_push.is_some_and(|push| kit.0 < push && push < app.0)),
        WAITING_ORDER,
        "actual kit merge < first app repin push < actual app merge",
    );
    for repository in [KIT, APP] {
        let green = merges.iter().find(|merge| merge.1 == repository.as_bytes()).is_some_and(|merge| {
            merge.2 == b"main"
                && records[..merge.0].iter().rev().find_map(|row| {
                    if let Observation::Reported { repository: repo, commit, context, state, .. } = row {
                        (repo.as_ref() == repository.as_bytes() && *commit == merge.3 && context.as_ref() == b"check")
                            .then_some(*state)
                    } else {
                        None
                    }
                }) == Some(api::Check::Passed)
        });
        step_check(&mut misses, "6.6", green, WAITING_LANDING, format_args!("{repository}: green head at merge"));
    }
    step_check(
        &mut misses,
        "6.6",
        kit.is_some_and(|merge| {
            observed
                .kit
                .main
                .get(super::waiting::PIN.as_bytes())
                .is_some_and(|pin| pin == direction_commit(merge.4).as_bytes())
        }),
        WAITING_PIN,
        "app's pin bytes equal the independently rendered actual kit merge",
    );
    misses
}

#[cfg(test)]
mod waiting_tests {
    use super::*;
    use crate::case::waiting::*;
    use skein_fake_forge_domain::{Observation, api};
    // Independent forge identities, unlike the scripts' 3/4/5/6.
    const KIT_HEAD: u64 = 11;
    const KIT_MERGE: u64 = 12;
    const APP_HEAD: u64 = 21;
    const APP_MERGE: u64 = 22;

    fn records() -> Vec<Observation> {
        let mut records = Vec::new();
        for (repository, task, head, commit) in [(KIT, E1, KIT_HEAD, KIT_MERGE), (APP, REPIN, APP_HEAD, APP_MERGE)] {
            let branch = format!("temper/{STARTER}/c{task}").into_bytes().into_boxed_slice();
            records.push(Observation::Moved {
                repository: repository.as_bytes().into(),
                branch: branch.clone(),
                from: None,
                to: head,
                by: 1,
            });
            records.push(Observation::Reported {
                repository: repository.as_bytes().into(),
                commit: head,
                context: b"check".as_slice().into(),
                state: api::Check::Passed,
                by: 1,
            });
            records.push(Observation::Merged {
                repository: repository.as_bytes().into(),
                number: 1,
                base: b"main".as_slice().into(),
                head,
                commit,
                by: 1,
            });
        }
        records
    }
    fn met() -> Observed {
        let records = records();
        Observed {
            activity_aliases: BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::new(),
            told: vec![],
            kit: Kit {
                pulls: vec![],
                records,
                merged_green: true,
                main: BTreeMap::from([(PIN.as_bytes().to_vec(), direction_commit(KIT_MERGE).into_bytes())]),
            },
        }
    }
    fn rejects(observed: &Observed, promise: Promise) {
        let misses = step_6_6(observed);
        assert!(
            misses.iter().any(|miss| miss.promise == promise),
            "expected {promise:?}: {}",
            misses.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
        );
    }
    #[test]
    fn two_repository_waiting_fixture_meets_its_promises() {
        assert_met(&step_6_6(&met()));
    }
    fn wrong_ci(record: &mut Observation) -> bool {
        if let Observation::Reported { repository, state, .. } = record
            && repository.as_ref() == KIT.as_bytes()
        {
            *state = api::Check::Failed;
            true
        } else {
            false
        }
    }
    fn wrong_ci_repository(record: &mut Observation) -> bool {
        if let Observation::Reported { repository, .. } = record
            && repository.as_ref() == KIT.as_bytes()
        {
            *repository = APP.as_bytes().into();
            true
        } else {
            false
        }
    }
    fn wrong_repository(record: &mut Observation) -> bool {
        if let Observation::Merged { repository, .. } = record
            && repository.as_ref() == KIT.as_bytes()
        {
            *repository = b"other/repo".as_slice().into();
            true
        } else {
            false
        }
    }
    fn wrong_head(record: &mut Observation) -> bool {
        if let Observation::Merged { repository, head, .. } = record
            && repository.as_ref() == APP.as_bytes()
        {
            *head = 99;
            true
        } else {
            false
        }
    }
    fn changed_actual_merge(record: &mut Observation) -> bool {
        if let Observation::Merged { repository, commit, .. } = record
            && repository.as_ref() == KIT.as_bytes()
        {
            *commit = 99;
            true
        } else {
            false
        }
    }
    #[test]
    fn typed_forge_mutations_break_qualified_heads_ci_and_actual_pin() {
        type ForgeEdit = fn(&mut Observation) -> bool;
        let passing = met();
        let rows: [(&str, ForgeEdit, Promise); 5] = [
            ("kit CI", wrong_ci, WAITING_LANDING),
            ("qualified CI", wrong_ci_repository, WAITING_LANDING),
            ("repository", wrong_repository, WAITING_ORDER),
            ("head", wrong_head, WAITING_LANDING),
            ("actual kit merge", changed_actual_merge, WAITING_PIN),
        ];
        for (name, edit, promise) in rows {
            let mut changed = passing.clone();
            let mut count = 0;
            for record in &mut changed.kit.records {
                count += usize::from(edit(record));
            }
            assert_eq!(count, 1, "{name}");
            rejects(&changed, promise);
        }
    }
    #[test]
    fn an_earlier_repin_push_or_wrong_pin_is_rejected() {
        let mut changed = met();
        let positions: Vec<_> = changed
            .kit
            .records
            .iter()
            .enumerate()
            .filter_map(|(at, r)| {
                matches!(r,Observation::Moved{repository,..} if repository.as_ref()==APP.as_bytes()).then_some(at)
            })
            .collect();
        assert_eq!(positions.len(), 1, "one app push moved");
        let at = positions[0];
        let push = changed.kit.records.remove(at);
        changed.kit.records.insert(0, push);
        rejects(&changed, WAITING_ORDER);
        let mut changed = met();
        *changed.kit.main.get_mut(PIN.as_bytes()).expect("pin") = b"script-counter-is-not-truth".to_vec();
        rejects(&changed, WAITING_PIN);
    }

    #[test]
    fn the_latest_qualified_ci_at_each_merge_is_green() {
        let mut changed = met();
        let at = changed
            .kit
            .records
            .iter()
            .position(|row| {
                matches!(row,
            Observation::Merged { repository, .. } if repository.as_ref() == KIT.as_bytes())
            })
            .expect("kit merge");
        changed.kit.records.insert(
            at,
            Observation::Reported {
                repository: KIT.as_bytes().into(),
                commit: KIT_HEAD,
                context: b"check".as_slice().into(),
                state: api::Check::Failed,
                by: 1,
            },
        );
        rejects(&changed, WAITING_LANDING);
    }
}

// Act 3, step 6.8 (testing/reference-case.md).
const QA_ADD: Promise = Promise::Add("the tester's tests land only through A1's change");
const QA_NO_WRITE: Promise = Promise::Must("the tester writes nothing to the forge");

fn qa_named_tools(observed: &Observed, read: Read, name: &str) -> Vec<(u64, Value, String, String)> {
    waiting_tools(observed, read).into_iter().filter(|(_, _, tool, _)| tool == name).collect()
}
fn qa_heard<'a>(observed: &'a Observed, read: Read, from: u64, words: &str) -> Vec<&'a Value> {
    check_activity(observed, read)
        .into_iter()
        .filter(|r| number(r, "/line/heard/from/task") == Some(from) && text(r, "/line/heard/words") == words)
        .collect()
}
fn qa_no_write(observed: &Observed) -> bool {
    use super::qa::{C, DIRECTOR, STATUS_CONTEXT};
    use skein_fake_forge_domain::Observation;
    observed.kit.records.iter().all(|r| {
        if let Observation::Reported { context, .. } = r {
            context.as_ref() != STATUS_CONTEXT.as_bytes()
        } else if let Observation::Moved { repository, branch, to, .. } = r {
            branch.as_ref() == format!("temper/{DIRECTOR}/c{C}").as_bytes()
                || (branch.as_ref() == b"main" && observed.kit.records.iter().any(|r| {
                    matches!(r, Observation::Merged {repository:repo,commit,..} if repo == repository && commit == to)
                }))
        } else if let Observation::Deleted { branch, .. } = r {
            branch.as_ref() == format!("temper/{DIRECTOR}/c{C}").as_bytes()
        } else if let Observation::Opened { branches: Some(branches), .. } = r {
            branches.head.as_ref() == format!("temper/{DIRECTOR}/c{C}").as_bytes()
        } else {
            true
        }
    })
}
fn qa_landing(observed: &Observed) -> bool {
    use super::qa::{A1, C, CHANGE_SPEC, DIRECTOR, QA_TEST, QA_TEST_PATH, T, TEST_REPORT};
    use skein_fake_forge_domain::{Observation, api};
    let change = task_document(observed, Read::QaChangeEnded);
    let delegates = qa_named_tools(observed, Read::QaSubownerEnded, "delegate");
    let changes: Vec<_> = delegates.iter().filter(|(_, _, _, out)| out == &format!("Delegated tasks: {C}")).collect();
    let merges: Vec<_> = observed
        .kit
        .records
        .iter()
        .enumerate()
        .filter_map(|(at, r)| {
            let Observation::Merged { repository, number, base, head, commit, .. } = r else { return None };
            Some((at, repository, number, base, head, commit))
        })
        .collect();
    let writes = qa_named_tools(observed, Read::QaCoderEnded, "write");
    text(task_document(observed, Read::QaTesterEnded), "/result/value/report/bytes") == TEST_REPORT
        && TEST_REPORT == format!("{QA_TEST_PATH}\n{QA_TEST}")
        && waiting_delivery(observed, Read::QaSubownerEnded, T, TEST_REPORT, Agent::QaSubowner)
        && changes.len() == 1
        && changes.iter().all(|(at, args, _, _)| {
            args["batch"].as_array().is_some_and(|b| {
                b.len() == 1
                    && waiting_batch_member(&b[0], crate::forge::ID, CHANGE_SPEC, QA_TEST_PATH)
                    && b[0]["spec"]["inputs"] == serde_json::json!([T])
            }) && qa_heard(observed, Read::QaSubownerEnded, T, TEST_REPORT)
                .iter()
                .any(|r| number(r, "/position").is_some_and(|p| p < *at))
        })
        && change.pointer("/spec/specification/inputs") == Some(&serde_json::json!([T]))
        && number(change, "/requester/task") == Some(A1)
        && waiting_parameters(change, crate::forge::ID, QA_TEST_PATH, 6)
        && waiting_delivery(
            observed,
            Read::QaSubownerEnded,
            C,
            text(change, "/result/value/change/words/bytes"),
            Agent::QaSubowner,
        )
        && writes.len() == 1
        && writes[0].1 == serde_json::json!({"path":QA_TEST_PATH,"content":QA_TEST})
        && !(super::fixture_6_8().setup)().tree.iter().any(|f| f.path.as_ref() == QA_TEST_PATH.as_bytes())
        && observed.kit.main.get(QA_TEST_PATH.as_bytes()).is_some_and(|words| words == QA_TEST.as_bytes())
        && merges.len() == 1
        && observed.kit.pulls.len() == 1
        && observed.kit.pulls == pull_digest(&observed.kit.records)
        && merges.iter().all(|(at, repo, pull, base, head, commit)| {
            let green = observed.kit.records[..*at].iter().rev().find_map(|r| {
                let Observation::Reported { repository, commit, context, state, .. } = r else { return None };
                (repository == *repo && commit == *head && context.as_ref() == b"check").then_some(*state)
            }) == Some(api::Check::Passed);
            let p = &observed.kit.pulls[0];
            repo.as_ref() == super::KIT.as_bytes()
                && base.as_ref() == b"main"
                && green
                && observed.kit.merged_green
                && p.branch.as_slice() == format!("temper/{DIRECTOR}/c{C}").as_bytes()
                && p.number == **pull
                && p.pushed == Some(**head)
                && p.opened == **head
                && p.merged_head == Some(**head)
                && p.landed == Some(**commit)
                && p.merged
                && !p.closed
                && number(change, "/result/value/change/part/key/number") == Some(**pull)
                && landing_commit(change) == direction_commit(**commit)
                && waiting_review(observed, Read::QaReviewerEnded, C, **head)
        })
}
#[must_use]
pub fn step_6_8(observed: &Observed) -> Vec<Miss> {
    let mut misses = Vec::new();
    step_check(
        &mut misses,
        "6.8",
        qa_no_write(observed),
        QA_NO_WRITE,
        "no tester status, push or pull among the forge records",
    );
    step_check(
        &mut misses,
        "6.8",
        qa_landing(observed),
        QA_ADD,
        "T's actual result is c's spec input and c's sole green, reviewed landing adds the exact test",
    );
    misses
}

#[cfg(test)]
mod calibration_tests;
