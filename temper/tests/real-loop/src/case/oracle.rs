//! Reference-case checkpoints: effects over observations any world can supply.
use super::{
    A, A_REPORT, Agent, DESIGN, DESIGN_DOC, DESIGN_REPORT, DESIGN_WORDS, DIRECTOR, DIRECTOR_REPORT, DRAFT, DRAFT_WORDS,
    GREEN_LIGHT, OWNER_EFFORT, OWNER_MODEL, PROJECT, QUESTION, acts::Read, draft_commit, kit_tree,
};
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
    /// Command verbs of the submitted ctl argv (reads and waits included).
    pub commands: Vec<String>,
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
    pub model: Box<[u8]>,
    pub credential: Option<u32>,
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
    pub fn new(
        system: &[u8],
        model: &[u8],
        credential: Option<u32>,
        messages: &[skein_fake_llm_domain::api::Message],
    ) -> Option<Self> {
        Some(Self::with_agent(cued(system)?, system, model, credential, messages))
    }

    fn with_agent(
        agent: Agent,
        system: &[u8],
        model: &[u8],
        credential: Option<u32>,
        messages: &[skein_fake_llm_domain::api::Message],
    ) -> Self {
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
        Self {
            system: system.into(),
            history: messages.into(),
            agent,
            words: words.into_boxed_slice(),
            model: model.into(),
            credential,
        }
    }
}
impl Observed {
    #[must_use]
    pub fn from_ran(ran: &Ran, reads: &[(Read, usize)]) -> Self {
        let (mut documents, observed_at) = read_documents(ran.calls(), reads);
        let mut told = Vec::new();
        for observation in ran.llm() {
            let skein_fake_peers::llm::Observation::Query { query, .. } = observation else { continue };
            if let Some(query) = Told::new(&query.system, &query.model, query.credential, &query.messages) {
                told.push(query);
            }
        }
        // The model peer observes the delivered brief even when the run ends
        // before a ctl read could see it. Keep the first query after Release.
        if let Some(fresh) =
            told.iter().find(|query| query.agent == Agent::Rules && contains(&query.system, super::NEW_RULES))
        {
            documents
                .insert(Read::RulesFreshBrief, serde_json::json!({"system": String::from_utf8_lossy(&fresh.system)}));
        }
        Self {
            activity_aliases: reads
                .iter()
                .filter_map(|(read, index)| ran.calls().get(*index).filter(|call| call.tries == 0).map(|_| *read))
                .collect(),
            documents,
            observed_at,
            commands: Vec::new(),
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
impl Observed {
    /// The command verbs of the story's ctl argv, in order, for checks on which acts it made.
    #[must_use]
    pub fn with_commands(mut self, steps: &[calls::Step]) -> Self {
        for step in steps {
            if let calls::Step::Call { argv, .. } | calls::Step::Start { argv, .. } | calls::Step::Until { argv, .. } =
                step
                && let Some(calls::Arg::Lit(command)) = argv.0.first()
            {
                let name = if let Some(calls::Arg::Lit(verb)) = argv.0.get(1)
                    && !matches!(*command, "me" | "inbox" | "sign-in" | "sign-out" | "link" | "pending" | "help")
                {
                    format!("{command} {verb}")
                } else {
                    String::from(*command)
                };
                self.commands.push(name);
            }
        }
        self
    }
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
        // Act 3, step 6.5 (testing/reference-case.md)
        Agent::BudgetDirector,
        Agent::BudgetOwner,
        Agent::BudgetSubowner,
        Agent::BudgetInvestigator,
        Agent::BudgetAdvisor,
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
const DELEGATE: Promise =
    Promise::Must("delegate a goal with instructions, input, priority, host kind, pool and models (exact)");
const AUTHORITY: Promise = Promise::Add("authority bounds A's subtree");
const SPEND: Promise = Promise::Add("spend rolls up to the goal");
const QUESTION_ANSWER: Promise = Promise::Must("a question and its answer (exact)");
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
    let mut docs = Vec::new();
    for (read, expected) in [(Read::DirectorEnded, DIRECTOR), (Read::AEnded, A), (Read::DesignEnded, DESIGN)] {
        let document = &observed.documents.get(&read).unwrap_or(&Value::Null)["task"];
        check(
            &mut misses,
            number(document, "/chip/task") == Some(expected),
            DELEGATE,
            format!("{read:?}: task number must be {expected}"),
        );
        check(
            &mut misses,
            number(document, "/project") == Some(PROJECT),
            DELEGATE,
            format!("{read:?}: project must be {PROJECT}"),
        );
        docs.push(document);
    }
    let [director, a, design] = docs.as_slice() else { unreachable!("three reads") };
    check(&mut misses, number(a, "/priority") == Some(1), DELEGATE, "A.priority must be 1".into());
    check(
        &mut misses,
        text(a, "/llm/main/model") == OWNER_MODEL,
        DELEGATE,
        format!("A.llm.main.model must be {OWNER_MODEL}"),
    );
    check(
        &mut misses,
        text(a, "/llm/main/effort") == OWNER_EFFORT,
        DELEGATE,
        format!("A.llm.main.effort must be {OWNER_EFFORT}"),
    );
    let spec = text(a, "/spec/specification/words/bytes");
    check(
        &mut misses,
        spec.contains(DRAFT) && spec.contains(&draft_commit()),
        DELEGATE,
        "A.spec must name the draft and its first commit".into(),
    );
    for path in ["/parent", "/requester/task"] {
        if let Some(value) = a.pointer(path) {
            check(&mut misses, value.as_u64() == Some(DIRECTOR), DELEGATE, format!("A{path} must be the director"));
        }
    }
    authority(&mut misses, director, a, "A");
    authority(&mut misses, a, design, "Design");
    rolled_up(&mut misses, a, design, "A", true);
    rolled_up(&mut misses, director, a, "Director", false);

    question_answer(&mut misses, observed, a, director);
    markdown(&mut misses, observed);
    misses
}

fn question_answer(misses: &mut Vec<Miss>, observed: &Observed, a: &Value, director: &Value) {
    let mut report_at = None;
    let mut acknowledged = false;
    let mut delivered = false;
    let sent = format!("Message {QUESTION} sent");
    for (position, Told { agent, words, .. }) in observed.told.iter().enumerate() {
        if *agent == Agent::Director && contains(words, DESIGN_REPORT) && report_at.is_none() {
            report_at = Some(position);
        }
        if *agent == Agent::A {
            acknowledged |= contains(words, &sent);
            delivered |= contains(words, GREEN_LIGHT) && matches!(report_at, Some(at) if at < position);
        }
    }
    check(misses, report_at.is_some(), QUESTION_ANSWER, "Director was not told A's design report".into());
    check(misses, acknowledged, QUESTION_ANSWER, format!("A was not told {sent}"));
    check(
        misses,
        delivered,
        QUESTION_ANSWER,
        "A was not told the green light after the director heard its question".into(),
    );
    for (document, expected, label) in [(a, A_REPORT, "A"), (director, DIRECTOR_REPORT, "Director")] {
        check(
            misses,
            text(document, "/result/value/report/bytes") == expected,
            QUESTION_ANSWER,
            format!("{label}.result must be {expected:?}"),
        );
    }
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

fn rolled_up(misses: &mut Vec<Miss>, parent: &Value, child: &Value, label: &str, positive: bool) {
    let below = number(parent, "/budget/numbers/spent_below");
    let spent = number(child, "/budget/numbers/spent");
    let child_below = number(child, "/budget/numbers/spent_below");
    let sum = match (spent, child_below) {
        (Some(spent), Some(below)) => spent.checked_add(below),
        _ => None,
    };
    check(
        misses,
        at_most(sum, below),
        SPEND,
        format!("{label}.budget.spent_below must cover its child's spent + spent_below"),
    );
    if positive {
        check(
            misses,
            matches!(below, Some(below) if below > 0),
            SPEND,
            format!("{label}.budget.spent_below must be positive"),
        );
    }
}
fn authority(misses: &mut Vec<Miss>, parent: &Value, child: &Value, label: &str) {
    let parent = &parent["authority"]["authority"];
    let child = &child["authority"]["authority"];
    for path in ["/spend", "/delegation/tasks", "/delegation/depth"] {
        check(
            misses,
            at_most(number(child, path), number(parent, path)),
            AUTHORITY,
            format!("{label}.authority{path} exceeds its parent or is absent"),
        );
    }
    check(
        misses,
        matches!((number(child, "/tools"), number(parent, "/tools")), (Some(child), Some(parent)) if child & !parent == 0),
        AUTHORITY,
        format!("{label}.authority.tools must be a subset"),
    );
    match (parent["delegation"]["kinds"].as_array(), child["delegation"]["kinds"].as_array()) {
        (Some(parents), Some(children)) => {
            for kind in children {
                check(
                    misses,
                    parents.contains(kind),
                    AUTHORITY,
                    format!("{label}.authority.delegation.kinds: {kind} is not covered"),
                );
            }
        }
        _ => check(misses, false, AUTHORITY, format!("{label}.authority.delegation.kinds is absent")),
    }
    match (parent["grants"].as_array(), child["grants"].as_array()) {
        (Some(parents), Some(children)) => {
            for grant in children {
                let mut covered = false;
                for parent in parents {
                    covered |= covers(parent, grant);
                }
                check(misses, covered, AUTHORITY, format!("{label}.authority.grants: {grant} is not covered"));
            }
        }
        _ => check(misses, false, AUTHORITY, format!("{label}.authority.grants is absent")),
    }
}
fn covers(parent: &Value, child: &Value) -> bool {
    if parent["connector"].as_u64().is_none()
        || parent["connector"] != child["connector"]
        || parent["kind"].as_u64().is_none()
        || parent["kind"] != child["kind"]
    {
        return false;
    }
    let (Some(parents), Some(children)) =
        (parent["pattern"]["segments"].as_array(), child["pattern"]["segments"].as_array())
    else {
        return false;
    };
    if !children.starts_with(parents) {
        return false;
    }
    let terminal = &parent["pattern"]["last"];
    if let Some(exact) = terminal.get("exact").and_then(Value::as_str) {
        return parents == children && child["pattern"]["last"]["exact"].as_str() == Some(exact);
    }
    let Some(prefix) = terminal.get("open").and_then(Value::as_str) else { return false };
    // Segments and terminal bytes are lower-case hex in ctl's JSON. Hex
    // preserves byte prefixes, so compare at whole-byte boundaries.
    let candidate = if let Some(segment) = children.get(parents.len()) {
        segment.as_str()
    } else {
        match child["pattern"]["last"].get("exact") {
            Some(exact) => exact.as_str(),
            None => child["pattern"]["last"]["open"].as_str(),
        }
    };
    matches!(candidate, Some(candidate) if prefix.len().is_multiple_of(2) && candidate.starts_with(prefix))
}
fn at_most(child: Option<u64>, parent: Option<u64>) -> bool {
    matches!((child, parent), (Some(child), Some(parent)) if child <= parent)
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
        A, A_REPORT, Agent, BTreeMap, DELEGATE, DESIGN, DESIGN_DOC, DESIGN_REPORT, DESIGN_WORDS, DIRECTOR,
        DIRECTOR_REPORT, GREEN_LIGHT, Kit, MARKDOWN, OWNER_EFFORT, OWNER_MODEL, Observed, PROJECT, QUESTION,
        QUESTION_ANSWER, Read, Told, Value, assert_met, contains, draft_commit, kit_tree, step_4_1,
    };
    use crate::case::{A_SPEC_PREFIX, A_SPEC_SUFFIX};
    use serde_json::json;

    #[test]
    fn the_first_system_cue_identifies_the_agent_before_delegated_cues() {
        let system = [b"guide\n".as_slice(), Agent::A.cue(), b"\ndelegate ", Agent::Director.cue()].concat();
        let query = Told::new(&system, b"fake", None, &[]).expect("system cue");
        assert_eq!(query.agent, Agent::A);
        let query = Told::new(Agent::RedRepairCoder.cue(), b"fake", None, &[]).expect("opening cue");
        assert_eq!(query.agent, Agent::RedRepairCoder);
        assert!(Told::new(b"guide without a case cue", b"fake", None, &[]).is_none());
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
        let mut documents = BTreeMap::new();
        for (read, task, tools, spend, tasks, depth, spent, below, report) in [
            (Read::DirectorEnded, DIRECTOR, 3, 40, 7, 4, 2, 5, DIRECTOR_REPORT),
            (Read::AEnded, A, 3, 20, 6, 3, 2, 3, A_REPORT),
            (Read::DesignEnded, DESIGN, 1, 10, 4, 2, 1, 2, ""),
        ] {
            documents.insert(read, json!({"task":{
                "chip":{"task":task}, "project":PROJECT, "requester":{"task":DIRECTOR},
                "priority":1, "llm":{"main":{"model":OWNER_MODEL,"effort":OWNER_EFFORT}},
                "spec":{"specification":{"words":{"bytes":format!("{A_SPEC_PREFIX}{}{A_SPEC_SUFFIX}", draft_commit())}}},
                "authority":{"authority":{"tools":tools,"spend":spend,
                    "grants":[{"connector":1,"kind":2,"pattern":{"segments":["666f726765"],"last":{"open":"61636d65"}}}],
                    "delegation":{"kinds":[{"charter":1},{"procedure":2}],"tasks":tasks,"depth":depth}}},
                "budget":{"numbers":{"spent":spent,"spent_below":below}},
                "result":{"value":{"report":{"bytes":report}}}
            }}));
        }
        let mut main = BTreeMap::new();
        for file in kit_tree() {
            main.insert(file.path.to_vec(), file.content.to_vec());
        }
        main.insert(DESIGN_DOC.as_bytes().to_vec(), DESIGN_WORDS.as_bytes().to_vec());
        Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents,
            commands: Vec::new(),
            told: vec![
                Told {
                    system: Box::new([]),
                    history: Box::new([]),
                    agent: Agent::A,
                    words: format!("Message {QUESTION} sent").into_bytes().into_boxed_slice(),
                    model: Box::new([]),
                    credential: Some(0),
                },
                Told {
                    system: Box::new([]),
                    history: Box::new([]),
                    agent: Agent::Director,
                    words: DESIGN_REPORT.as_bytes().into(),
                    model: Box::new([]),
                    credential: Some(0),
                },
                Told {
                    system: Box::new([]),
                    history: Box::new([]),
                    agent: Agent::A,
                    words: GREEN_LIGHT.as_bytes().into(),
                    model: Box::new([]),
                    credential: Some(0),
                },
            ],
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
        const GREEN_LIGHT_QUERIES: usize = 1;
        let mut broken = met().clone();
        broken.documents.get_mut(&Read::AEnded).expect("A document")["task"]["priority"] = Value::Null;
        broken.kit.main.insert(b"src/export.rs".to_vec(), b"fn export() {}\n".to_vec());
        let mut removed = 0;
        for Told { agent, words, .. } in &mut broken.told {
            if *agent == Agent::A && contains(words, GREEN_LIGHT) {
                *words = Box::new([]);
                removed += 1;
            }
        }
        assert_eq!(removed, GREEN_LIGHT_QUERIES);
        let misses = step_4_1(&broken);
        assert_eq!(misses.len(), MISSES);
        let [priority, answer, markdown] = misses.as_slice() else { panic!("three misses") };
        assert_eq!(priority.promise, DELEGATE);
        assert!(priority.what.contains("A.priority"));
        assert_eq!(answer.promise, QUESTION_ANSWER);
        assert!(answer.what.contains("green light"));
        assert_eq!(markdown.promise, MARKDOWN);
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

/// Retained first-edition story (testing/reference-case.md): supported timer, cancel and empty-worker checkpoint.
#[must_use]
pub fn step_4_3(observed: &Observed) -> Vec<Miss> {
    use super::{B, B_REPORT, BENCHMARK, CHECK, CHECK_REASON, CHECK_REPORT};
    const TIMER: Promise = Promise::Must("a timer (exact)");
    const CANCEL: Promise = Promise::Must("cancel (exact)");
    const EMPTY: Promise = Promise::Add("a moved owner leaves nothing running behind");
    let mut misses = Vec::new();
    let director = task_document(observed, Read::CheckEnded);
    let benchmark = task_document(observed, Read::BenchmarkEnded);
    let b = task_document(observed, Read::BEnded);
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
        number(benchmark, "/chip/task") == Some(BENCHMARK)
            && text(benchmark, "/chip/phase") == "ended"
            && number(benchmark, "/result/ending/task/cancelled/by/task") == Some(CHECK)
            && text(benchmark, "/result/ending/task/cancelled/reason/bytes") == CHECK_REASON,
        CANCEL,
        "the ended benchmark must record cancellation by the director with its reason",
    );
    let activity = check_activity(observed, Read::BenchmarkEnded);
    let attempt = number(benchmark, "/chip/details/owner/details/attempt");
    let cancelled = activity.iter().any(|line| {
        text(line, "/line/run_ended/end") == "cancelled"
            && number(line, "/line/run_ended/attempt").zip(attempt).is_some_and(|(ended, current)| ended == current)
    });
    step_check(&mut misses, "4.3", cancelled, CANCEL, "the benchmark's activity must record a cancelled run ending");
    let began: BTreeSet<_> = activity.iter().filter_map(|line| number(line, "/line/run_began/attempt")).collect();
    let ended: BTreeSet<_> = activity.iter().filter_map(|line| number(line, "/line/run_ended/attempt")).collect();
    step_check(
        &mut misses,
        "4.3",
        !began.is_empty()
            && began.is_subset(&ended)
            && document(observed, Read::BenchmarkEnded).pointer("/activity/cut") == Some(&Value::Bool(false))
            && document(observed, Read::BenchmarkEnded).pointer("/activity/before") == Some(&Value::Null)
            && document(observed, Read::BenchmarkEnded).pointer("/activity/after") == Some(&Value::Null)
            && number(benchmark, "/chip/details/owner/details/ended_at").is_some(),
        EMPTY,
        "the benchmark's complete activity must end every begun run, with the task ended",
    );
    // B's journal orders its lines, newest first (../../../../docs/design/client/ctl/testing.md, 3): the
    // cancel heard is older than the run that finished with B's report. The
    // journal's order, not two wall times, so a clock step can't reorder them.
    let b_activity = check_activity(observed, Read::BEnded);
    let reported = b_activity.iter().position(|line| text(line, "/line/run_ended/end") == "finished");
    let heard = b_activity.iter().position(|line| {
        text(line, "/line/heard/kind/result") == "cancelled"
            && number(line, "/line/heard/from/task") == Some(BENCHMARK)
            && text(line, "/line/heard/words") == CHECK_REASON
    });
    let heard_cancel = reported.zip(heard).is_some_and(|(reported, heard)| reported < heard);
    step_check(
        &mut misses,
        "4.3",
        number(b, "/chip/task") == Some(B)
            && text(b, "/result/value/report/bytes") == B_REPORT
            && heard_cancel
            && observed.told.iter().any(|told| {
                told.agent == Agent::B
                    && contains(&told.words, CHECK_REASON)
                    && contains(&told.words, "Messages arrived. Read them before continuing.")
            }),
        EMPTY,
        "B must hear its delegate's cancelled ending before reporting",
    );
    step_check(
        &mut misses,
        "4.3",
        number(director, "/chip/task") == Some(CHECK)
            && text(director, "/result/value/report/bytes") == CHECK_REPORT
            && observed.told.iter().any(|told| told.agent == Agent::Check && contains(&told.words, B_REPORT)),
        EMPTY,
        "the check director must hear B's result and end with its check report",
    );
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
            let told = Told::with_agent(Agent::Check, b"", b"fake", None, &messages);
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
        Told {
            system: Box::new([]),
            agent,
            history: history.into_boxed_slice(),
            words: Box::new([]),
            model: Box::new([]),
            credential: None,
        }
    }
    fn met() -> Observed {
        Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::new(),
            commands: Vec::new(),
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
const RULES_CONTROL: Promise = Promise::Must("Stop and Release on the chat (exact)");
const RULES_INBOX: Promise = Promise::Must("questions wait in its inbox (exact)");
const RULES_CONVENTIONS: Promise = Promise::Must("the rules land as conventions that every later run reads (exact)");
const RULES_FRESH: Promise = Promise::Must("the director's next run starts fresh and reads them (exact)");
const RULES_NO_NOTICE: Promise = Promise::Add("no message telling anyone to re-read a rule");
const RULES_DROPPED: Promise = Promise::Add("a held owner needs no notice that a rule was dropped");

#[must_use]
pub fn step_5_1(observed: &Observed) -> Vec<Miss> {
    let mut misses = Vec::new();
    rules_controls(&mut misses, observed);
    rules_director(&mut misses, observed);
    rules_fresh_tail(&mut misses, observed);
    rules_owner(&mut misses, observed);
    rules_messages_and_endings(&mut misses, observed);
    misses
}

fn rules_controls(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{NEW_RULES, RULES, RULES_FILE, RULES_HOLD_REVISION, SCOPE, SCOPE_QUESTION, SCOPE_QUESTION_NUMBER};
    let waiting = task_document(observed, Read::RulesWaiting);
    let held = task_document(observed, Read::RulesHeld);
    let question = task_document(observed, Read::RulesQuestionWaiting);
    let scope = task_document(observed, Read::ScopeHeld);
    for (met, promise, what) in [
        (
            number(waiting, "/chip/task") == Some(RULES)
                && text(waiting, "/chip/details/owner/details/observation/work") == "waiting"
                && waiting["delegates"]
                    .as_array()
                    .is_some_and(|rows| rows.iter().any(|row| number(row, "/chip/task") == Some(SCOPE))),
            RULES_CONTROL,
            "the director must wait in its run with A delegated",
        ),
        (rules_held(held), RULES_CONTROL, "the chat must be held, stopped by the person at its hold revision"),
        (
            number(question, "/chip/task") == Some(RULES)
                && question["hold"].is_null()
                && check_activity(observed, Read::RulesQuestionWaiting).iter().any(|row| {
                    number(row, "/task") == Some(RULES)
                        && number(row, "/line/words/message") == Some(SCOPE_QUESTION_NUMBER)
                        && number(row, "/line/words/by/task") == Some(SCOPE)
                        && text(row, "/line/words/kind") == "question"
                        && text(row, "/line/words/words") == SCOPE_QUESTION
                }),
            RULES_INBOX,
            "A's numbered question must reach the director after the subtree resumes",
        ),
        (
            number(scope, "/chip/task") == Some(SCOPE)
                && text(scope, "/chip/phase") == "held"
                && number(scope, "/hold/reason/placed/hold") == number(held, "/hold/reason/placed/hold")
                && number(scope, "/hold/revision") == Some(RULES_HOLD_REVISION),
            RULES_INBOX,
            "the same hold must stop A before its question can be sent",
        ),
        (
            observed.kit.main.get(RULES_FILE.as_bytes()).is_some_and(|words| words == NEW_RULES.as_bytes()),
            RULES_CONVENTIONS,
            "kit's main must hold the new guide",
        ),
    ] {
        step_check(misses, "5.1", met, promise, what);
    }
}

fn rules_director(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{DROPPED_RULE, NEW_RULES, OLD_RULES, SCOPE_QUESTION};
    use skein_fake_llm_domain::api::Role;
    let director: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Rules).collect();
    let resumed = director.iter().position(|told| {
        contains(&told.system, NEW_RULES) || user_words(told).any(|words| contains(words, SCOPE_QUESTION))
    });
    let before = resumed.map_or(&director[..], |at| &director[..at]);
    let after = resumed.map_or(&director[director.len()..], |at| &director[at..]);
    for (met, promise, what) in [
        (
            !before.is_empty()
                && before.iter().all(|told| contains(&told.system, OLD_RULES))
                && before.first().is_some_and(|first| before.iter().all(|told| told.system == first.system))
                && before.windows(2).all(|pair| pair[1].history.starts_with(&pair[0].history)),
            RULES_CONVENTIONS,
            "the director's opening run must read the old guide",
        ),
        (
            !after.is_empty()
                && after.iter().all(|told| contains(&told.system, NEW_RULES) && !contains(&told.system, DROPPED_RULE)),
            RULES_CONVENTIONS,
            "every director query told the question must read the new guide without the dropped rule",
        ),
        (
            after.first().is_some_and(|first| first.history.iter().all(|message| message.role != Role::Assistant)),
            RULES_FRESH,
            "the director's first query after release must contain none of its earlier assistant turns",
        ),
        (
            after.first().is_some_and(|first| {
                after.iter().all(|told| told.system == first.system)
                    && after.windows(2).all(|pair| pair[1].history.starts_with(&pair[0].history))
                    && after.iter().any(|told| user_words(told).any(|words| contains(words, SCOPE_QUESTION)))
                    && after.iter().all(|told| {
                        let count = user_words(told)
                            .map(|words| calls::occurrences(words, SCOPE_QUESTION.as_bytes()))
                            .sum::<usize>();
                        count <= 1
                    })
            }),
            RULES_INBOX,
            "the question must be delivered once in one director run's growing history",
        ),
    ] {
        step_check(misses, "5.1", met, promise, what);
    }
}

fn rules_fresh_tail(misses: &mut Vec<Miss>, observed: &Observed) {
    let system = text(document(observed, Read::RulesFreshBrief), "/system");
    let tail = system.split_once("Transcript tail").map(|(_, rest)| rest.split("\n#").next().unwrap_or(rest));
    step_check(
        misses,
        "5.1",
        tail.is_some_and(|tail| {
            tail.contains("call delegate:")
                && tail.contains(super::SCOPE_SPEC)
                && tail.contains(&format!("result: Delegated tasks: {}", super::SCOPE))
                && !tail.contains("[earlier bytes not text, cut]")
        }),
        RULES_FRESH,
        "the director's fresh brief must have a Transcript tail holding its earlier committed turn text",
    );
}

fn rules_owner(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{DROPPED_RULE, NEW_RULES, SCOPE_ANSWER};
    let a: Vec<_> = observed.told.iter().filter(|told| told.agent == Agent::Scope).collect();
    step_check(
        misses,
        "5.1",
        a.first().is_some_and(|told| contains(&told.system, DROPPED_RULE)),
        RULES_DROPPED,
        "A's opening query must read the dropped rule while it is in force",
    );
    let answered: Vec<_> =
        a.iter().filter(|told| user_words(told).any(|words| contains(words, SCOPE_ANSWER))).collect();
    step_check(
        misses,
        "5.1",
        answered.len() == 1
            && answered.iter().all(|told| {
                contains(&told.system, NEW_RULES)
                    && !contains(&told.system, DROPPED_RULE)
                    && user_words(told).map(|words| calls::occurrences(words, SCOPE_ANSWER.as_bytes())).sum::<usize>()
                        == 1
            }),
        RULES_DROPPED,
        "A must hear the answer once in a run reading the new guide without the dropped rule",
    );
}

fn rules_messages_and_endings(misses: &mut Vec<Miss>, observed: &Observed) {
    use super::{DROPPED_RULE, NEW_RULES, RULES, RULES_REPORT, SCOPE, SCOPE_REPORT};
    step_check(
        misses,
        "5.1",
        observed.told.iter().all(|told| {
            message_words(told).all(|words| {
                NEW_RULES
                    .lines()
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .all(|line| !contains(words, line))
                    && !contains(words, "re-read")
                    && !contains(words, "reread")
            })
        }),
        RULES_NO_NOTICE,
        "no query's messages may carry the new rules or a request to re-read them",
    );
    step_check(
        misses,
        "5.1",
        observed.told.iter().all(|told| message_words(told).all(|words| !contains(words, DROPPED_RULE))),
        RULES_DROPPED,
        "no message may tell an owner the dropped rule",
    );
    for read in [Read::RulesActivity, Read::ScopeActivity] {
        let endings: Vec<_> =
            check_activity(observed, read).into_iter().filter_map(|row| row.pointer("/line/run_ended")).collect();
        step_check(
            misses,
            "5.1",
            !endings.is_empty()
                && endings
                    .iter()
                    .all(|ended| ended.pointer("/end/failed/class").and_then(Value::as_str) != Some("invalid")),
            RULES_CONTROL,
            "no run of the director or A may start invalid: a fresh start refused and retried would pass unseen",
        );
    }
    for (read, task, report) in [(Read::RulesEnded, RULES, RULES_REPORT), (Read::ScopeEnded, SCOPE, SCOPE_REPORT)] {
        let ended = task_document(observed, read);
        step_check(
            misses,
            "5.1",
            number(ended, "/chip/task") == Some(task)
                && text(ended, "/chip/phase") == "ended"
                && text(ended, "/result/value/report/bytes") == report,
            RULES_CONTROL,
            "the original director and A must end with their reports",
        );
    }
}

fn rules_held(task: &Value) -> bool {
    use super::{RULES, RULES_HOLD_REVISION};
    number(task, "/chip/task") == Some(RULES)
        && text(task, "/chip/phase") == "held"
        && number(task, "/hold/reason/placed/hold").is_some_and(|hold| hold != 0)
        && number(task, "/hold/revision") == Some(RULES_HOLD_REVISION)
}

fn message_words(told: &Told) -> impl Iterator<Item = &[u8]> {
    use skein_fake_llm_domain::api::Part;
    told.history.iter().flat_map(|message| &message.parts).filter_map(|part| match part {
        Part::Text { text } | Part::ToolOutput { output: text, .. } => Some(text.as_ref()),
        Part::ToolCall { arguments, .. } => Some(arguments.as_ref()),
        Part::Opaque { .. } => None,
    })
}

#[cfg(test)]
mod rules_tests {
    use super::*;
    use crate::case::{
        DROPPED_RULE, NEW_RULES, OLD_RULES, RULES, RULES_FILE, RULES_HOLD_REVISION, RULES_REPORT, SCOPE, SCOPE_ANSWER,
        SCOPE_QUESTION, SCOPE_QUESTION_NUMBER, SCOPE_REPORT,
    };
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    const OPENING: usize = 0;
    const OPENING_FOLLOWUP: usize = 1;
    const FRESH: usize = 2;
    const LATER: usize = 3;
    const A_OPENING: usize = 4;
    const A_ANSWERED: usize = 5;

    fn user(words: &str) -> Message {
        Message { role: Role::User, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn assistant(words: &str) -> Message {
        Message { role: Role::Assistant, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn told(agent: Agent, guide: &str, history: &[Message]) -> Told {
        let mut system = agent.cue().to_vec();
        system.extend_from_slice(guide.as_bytes());
        Told::new(&system, b"fake", Some(0), history).expect("case cue")
    }
    fn held() -> Value {
        json!({"chip":{"task":RULES,"phase":"held"}, "hold":{
            "reason":{"placed":{"hold":1}},"revision":RULES_HOLD_REVISION}})
    }
    fn ended(task: u64, report: &str) -> Value {
        json!({"task":{"chip":{"task":task,"phase":"ended"},"result":{"value":{"report":{"bytes":report}}}}})
    }
    fn endings(task: u64, ends: &[Value]) -> Value {
        let rows: Vec<Value> = ends
            .iter()
            .map(|end| json!({"event":{"owner":{"source":{"task":task,"line":{"run_ended":{"end":end}}}}}}))
            .collect();
        json!({"task":{"chip":{"task":task}},"activity":{"rows":rows}})
    }
    fn fresh_tail() -> String {
        format!(
            "## Transcript tail\n\ncall delegate: {}\nresult: Delegated tasks: {}\n",
            super::super::SCOPE_SPEC,
            SCOPE
        )
    }
    fn met() -> Observed {
        let fresh = vec![user(&format!(
            "{SCOPE} asks (question {SCOPE_QUESTION_NUMBER}; answer with message target={SCOPE}, form=answer, question={SCOPE_QUESTION_NUMBER}): {SCOPE_QUESTION}"
        ))];
        let mut later = fresh.clone();
        later.extend([assistant("Answer A."), user("Answer sent")]);
        Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::from([
                (
                    Read::RulesWaiting,
                    json!({"chip":{"task":RULES,"details":{"owner":{"details":{
                    "observation":{"work":"waiting"}}}}},"delegates":[{"chip":{"task":SCOPE}}]}),
                ),
                (Read::RulesHeld, held()),
                (
                    Read::ScopeHeld,
                    json!({"chip":{"task":SCOPE,"phase":"held"},"hold":{"reason":{"placed":{"hold":1}},"revision":RULES_HOLD_REVISION}}),
                ),
                (
                    Read::RulesQuestionWaiting,
                    json!({"task":{"chip":{"task":RULES,"phase":"running"},"hold":null},"activity":{"rows":[{"event":{"owner":{"source":{
                    "task":RULES,"line":{"words":{"message":SCOPE_QUESTION_NUMBER,"by":{"task":SCOPE},
                        "kind":"question","words":SCOPE_QUESTION}}}}}}]}}),
                ),
                (Read::RulesFreshBrief, json!({"system": fresh_tail()})),
                (Read::RulesEnded, ended(RULES, RULES_REPORT)),
                (Read::ScopeEnded, ended(SCOPE, SCOPE_REPORT)),
                (Read::RulesActivity, endings(RULES, &[json!("finished"), json!({"failed":{"class":"agent"}})])),
                (Read::ScopeActivity, endings(SCOPE, &[json!("finished"), json!("parked")])),
            ]),
            told: vec![
                told(Agent::Rules, OLD_RULES, &[user("Begin the work your brief describes.")]),
                told(
                    Agent::Rules,
                    OLD_RULES,
                    &[user("Begin the work your brief describes."), assistant("Delegate A.")],
                ),
                told(Agent::Rules, NEW_RULES, &fresh),
                told(Agent::Rules, NEW_RULES, &later),
                told(Agent::Scope, OLD_RULES, &[user("Begin the work your brief describes.")]),
                told(Agent::Scope, NEW_RULES, &[user(SCOPE_ANSWER)]),
            ],
            commands: Vec::new(),
            kit: Kit {
                records: Vec::new(),
                pulls: Vec::new(),
                main: BTreeMap::from([(RULES_FILE.as_bytes().to_vec(), NEW_RULES.as_bytes().to_vec())]),
                merged_green: false,
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
    fn held_chat_and_fresh_rules_meet_the_checkpoint() {
        assert_met(&step_5_1(&met()));
    }
    #[test]
    fn the_wait_and_both_hold_reads_are_checked() {
        let mut passing = met();
        for (read, path, promise, what) in [
            (Read::RulesWaiting, "/chip/task", RULES_CONTROL, "wait in its run"),
            (Read::RulesWaiting, "/chip/details/owner/details/observation/work", RULES_CONTROL, "wait in its run"),
            (Read::RulesWaiting, "/delegates/0/chip/task", RULES_CONTROL, "wait in its run"),
            (Read::RulesHeld, "/chip/task", RULES_CONTROL, "stopped by the person"),
            (Read::RulesHeld, "/chip/phase", RULES_CONTROL, "stopped by the person"),
            (Read::RulesHeld, "/hold/reason/placed/hold", RULES_CONTROL, "stopped by the person"),
            (Read::RulesHeld, "/hold/revision", RULES_CONTROL, "stopped by the person"),
            (Read::RulesQuestionWaiting, "/task/chip/task", RULES_INBOX, "numbered question"),
            (Read::ScopeHeld, "/chip/phase", RULES_INBOX, "same hold"),
            (Read::ScopeHeld, "/hold/revision", RULES_INBOX, "same hold"),
        ] {
            let saved = passing.documents.get_mut(&read).expect("document").pointer_mut(path).expect("field").take();
            rejects(&passing, promise, what);
            *passing.documents.get_mut(&read).expect("document").pointer_mut(path).expect("field") = saved;
        }
    }
    #[test]
    fn the_waiting_question_has_its_recipient_sender_number_and_words() {
        let mut passing = met();
        for path in ["/task", "/line/words/message", "/line/words/by/task", "/line/words/kind", "/line/words/words"] {
            let saved = passing.documents.get_mut(&Read::RulesQuestionWaiting).expect("question")["activity"]["rows"]
                [0]["event"]["owner"]["source"]
                .pointer_mut(path)
                .expect("question field")
                .take();
            rejects(&passing, RULES_INBOX, "numbered question");
            *passing.documents.get_mut(&Read::RulesQuestionWaiting).expect("question")["activity"]["rows"][0]
                ["event"]["owner"]["source"].pointer_mut(path).expect("question field") = saved;
        }
    }
    #[test]
    fn main_and_every_relevant_query_must_have_the_right_system_guide() {
        let passing = met();
        let mut observed = passing.clone();
        observed.kit.main.clear();
        rejects(&observed, RULES_CONVENTIONS, "kit's main");
        for at in [OPENING, FRESH, LATER, A_OPENING, A_ANSWERED] {
            let mut observed = passing.clone();
            observed.told[at].system = Box::new([]);
            let (promise, what) = match at {
                OPENING => (RULES_CONVENTIONS, "opening run"),
                A_OPENING => (RULES_DROPPED, "while it is in force"),
                A_ANSWERED => (RULES_DROPPED, "hear the answer once"),
                _ => (RULES_CONVENTIONS, "every director query"),
            };
            rejects(&observed, promise, what);
        }
        for at in [FRESH, LATER, A_ANSWERED] {
            let mut observed = passing.clone();
            let mut system = observed.told[at].system.to_vec();
            system.extend_from_slice(DROPPED_RULE.as_bytes());
            observed.told[at].system = system.into();
            rejects(
                &observed,
                if at == A_ANSWERED { RULES_DROPPED } else { RULES_CONVENTIONS },
                if at == A_ANSWERED { "hear the answer once" } else { "every director query" },
            );
        }
    }
    #[test]
    fn the_directors_history_must_be_fresh_then_grow_in_one_run() {
        let passing = met();
        let mut observed = passing.clone();
        let mut history = observed.told[FRESH].history.to_vec();
        history.push(assistant("Delegate A."));
        observed.told[FRESH].history = history.into();
        rejects(&observed, RULES_FRESH, "earlier assistant turns");
        let mut observed = passing.clone();
        observed.told[LATER].system = OLD_RULES.as_bytes().into();
        rejects(&observed, RULES_INBOX, "one director run");
        let mut observed = passing.clone();
        observed.told[LATER].history = Box::new([user("Another run"), user(SCOPE_QUESTION)]);
        rejects(&observed, RULES_INBOX, "one director run");
        for replacement in ["No question", &format!("{SCOPE_QUESTION} {SCOPE_QUESTION}")] {
            let mut observed = passing.clone();
            observed.told[FRESH].history = Box::new([user(replacement)]);
            rejects(&observed, RULES_INBOX, "delivered once");
        }
    }
    #[test]
    fn the_opening_queries_must_also_belong_to_one_run() {
        let passing = met();
        let mut observed = passing.clone();
        observed.told[OPENING_FOLLOWUP].system = format!("{OLD_RULES}another run").into_bytes().into();
        rejects(&observed, RULES_CONVENTIONS, "opening run");
        let mut observed = passing.clone();
        observed.told[OPENING_FOLLOWUP].history = Box::new([user("Another opening")]);
        rejects(&observed, RULES_CONVENTIONS, "opening run");
    }

    #[test]
    fn a_must_receive_the_answer_once() {
        let passing = met();
        let mut observed = passing.clone();
        observed.told[A_ANSWERED].history = Box::new([]);
        rejects(&observed, RULES_DROPPED, "hear the answer once");
        let mut observed = passing.clone();
        observed.told.push(observed.told[A_ANSWERED].clone());
        rejects(&observed, RULES_DROPPED, "hear the answer once");
        let mut observed = passing.clone();
        observed.told[A_ANSWERED].history = Box::new([user(&format!("{SCOPE_ANSWER} {SCOPE_ANSWER}"))]);
        rejects(&observed, RULES_DROPPED, "hear the answer once");
    }
    #[test]
    fn messages_cannot_supply_rules_or_reread_notices() {
        let passing = met();
        for words in NEW_RULES
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .chain(["Please re-read the guide", "Please reread the guide"])
        {
            let mut observed = passing.clone();
            observed.told[A_ANSWERED].history = Box::new([user(words), user(SCOPE_ANSWER)]);
            rejects(&observed, RULES_NO_NOTICE, "no query's messages");
        }
        let mut observed = passing.clone();
        observed.told[A_ANSWERED].history = Box::new([user(DROPPED_RULE), user(SCOPE_ANSWER)]);
        rejects(&observed, RULES_DROPPED, "no message");
    }
    #[test]
    fn a_fresh_tail_must_hold_committed_turn_text_without_a_binary_marker() {
        for system in [
            String::new(),
            "Transcript tail\n[earlier bytes not text, cut]\n".into(),
            "Transcript tail\n[a turn not readable as text]\n".into(),
            format!("{}[earlier bytes not text, cut]\n", fresh_tail()),
            fresh_tail().replace("Transcript tail", "Task"),
            fresh_tail().replace(super::super::SCOPE_SPEC, "other task"),
            fresh_tail().replace("result: Delegated tasks: 2", "result: delegation refused"),
            format!("## Transcript tail\n\n## Notes\n{}", fresh_tail()),
        ] {
            let mut observed = met();
            observed.documents.get_mut(&Read::RulesFreshBrief).expect("fresh brief")["system"] = system.into();
            rejects(&observed, RULES_FRESH, "Transcript tail");
        }
        let mut observed = met();
        observed.documents.remove(&Read::RulesFreshBrief);
        rejects(&observed, RULES_FRESH, "Transcript tail");
    }

    #[test]
    fn no_run_may_start_invalid_and_hide_behind_its_retry() {
        let passing = met();
        for read in [Read::RulesActivity, Read::ScopeActivity] {
            let mut observed = passing.clone();
            let rows =
                observed.documents.get_mut(&read).expect("activity")["activity"]["rows"].as_array_mut().expect("rows");
            rows.insert(
                1,
                json!({"event":{"owner":{"source":{"line":{"run_ended":{"end":{"failed":{"class":"invalid"}}}}}}}}),
            );
            rejects(&observed, RULES_CONTROL, "start invalid");
            let mut observed = passing.clone();
            observed.documents.remove(&read);
            rejects(&observed, RULES_CONTROL, "start invalid");
        }
    }
    #[test]
    fn both_original_tasks_must_end_with_their_reports() {
        let mut passing = met();
        for read in [Read::RulesEnded, Read::ScopeEnded] {
            for path in ["/task/chip/task", "/task/chip/phase", "/task/result/value/report/bytes"] {
                let saved = passing.documents.get_mut(&read).expect("ended").pointer_mut(path).expect("field").take();
                rejects(&passing, RULES_CONTROL, "original director and A");
                *passing.documents.get_mut(&read).expect("ended").pointer_mut(path).expect("field") = saved;
            }
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
        Told::new(&[agent.cue(), b"\n", brief.as_bytes()].concat(), b"fake", Some(0), messages).expect("case cue")
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
            commands: Vec::new(),
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
            model: Box::new([]),
            credential: None,
        };
        Observed {
            activity_aliases: BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents,
            commands: Vec::new(),
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

// Match actual tool outputs to their assistant call identities, never to prose
// repeating the expected words. A replayed prefix is still one completed call.
fn plan_tool_results(observed: &Observed, agent: Agent, tool: &[u8]) -> Vec<(usize, Value, Vec<u8>)> {
    use skein_fake_llm_domain::api::{Part, Role};
    let mut seen = BTreeSet::new();
    let mut results = Vec::new();
    let mut previous: &[skein_fake_llm_domain::api::Message] = &[];
    let mut calls = BTreeMap::new();
    for (at, told) in observed.told.iter().enumerate().filter(|(_, told)| told.agent == agent) {
        // Reuse calls only when every message in the previous history agrees.
        // A changed or truncated conversation gets its own lookup; seen ids
        // continue to identify completed results across conversations.
        // An empty lookup has no prefix state to reuse.
        let offset = if !calls.is_empty() && told.history.starts_with(previous) {
            previous.len()
        } else {
            calls.clear();
            0
        };
        for message in &told.history[offset..] {
            for part in &message.parts {
                match part {
                    Part::ToolCall { id, name, arguments }
                        if message.role == Role::Assistant && name.as_ref() == tool =>
                    {
                        calls.insert(id.as_ref(), arguments);
                    }
                    Part::ToolOutput { id, output, .. } if message.role == Role::User => {
                        if !seen.contains(id.as_ref())
                            && let Some(arguments) = calls.get(id.as_ref())
                            && let Ok(args) = serde_json::from_slice(arguments)
                            && seen.insert(id.to_vec())
                        {
                            results.push((at, args, output.to_vec()));
                        }
                    }
                    Part::Text { .. } | Part::Opaque { .. } | Part::ToolCall { .. } | Part::ToolOutput { .. } => {}
                }
            }
        }
        previous = &told.history;
    }
    results
}

fn plan_fresh(told: &Told) -> bool {
    use skein_fake_llm_domain::api::{Part, Role};
    !told.history.is_empty()
        && told.history.iter().all(|message| {
            message.role == Role::User && message.parts.iter().all(|part| matches!(part, Part::Text { .. }))
        })
}

fn plan_note_recalled(observed: &Observed, agent: Agent, note: u64, description: &str, body: &str) -> bool {
    let writes = plan_tool_results(observed, agent, b"note");
    let recalls = plan_tool_results(observed, agent, b"recall");
    writes.iter().any(|(written, args, output)| {
        args["scope"] == serde_json::json!({"kind":"project","project":PROJECT})
            && args["description"] == description
            && args["body"] == body
            && contains(output, &format!("Note {note} revision 1 written · 1 lines"))
            && recalls.iter().any(|(recalled, args, output)| {
                written < recalled
                    && args["kind"] == "name"
                    && args["name"] == note
                    && contains(output, &format!("Note {note} revision 1: {description}\n{body}\n"))
            })
    })
}

fn plan_attempts(observed: &Observed, reads: &[Read], task: u64, minimum: usize) -> bool {
    let rows: Vec<_> = joined_activity(observed, reads);
    plan_attempt_rows(observed, reads, task, minimum, &rows)
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

#[cfg(test)]
mod plan_tests {
    use super::*;
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    fn tool(id: &str, name: &str, args: &Value, output: &str) -> [Message; 2] {
        [
            Message {
                role: Role::Assistant,
                parts: Box::new([Part::ToolCall {
                    id: id.as_bytes().into(),
                    name: name.as_bytes().into(),
                    arguments: serde_json::to_vec(args).expect("args").into(),
                }]),
            },
            Message {
                role: Role::User,
                parts: Box::new([Part::ToolOutput {
                    id: id.as_bytes().into(),
                    output: output.as_bytes().into(),
                    is_error: false,
                }]),
            },
        ]
    }
    fn told(agent: Agent, history: &[Message]) -> Told {
        Told::with_agent(agent, agent.cue(), b"fake", None, history)
    }
    fn met() -> Observed {
        Observed {
            activity_aliases: BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::new(),
            commands: Vec::new(),
            kit: Kit { records: Vec::new(), pulls: Vec::new(), main: BTreeMap::new(), merged_green: true },
            told: Vec::new(),
        }
    }
    #[test]
    fn tool_results_follow_growing_histories_and_reset_changed_or_truncated_prefixes() {
        let mut observed = met();
        let pending = tool("pending", "note", &json!({"version":1}), "first");
        let old = tool("changed", "note", &json!({"version":2}), "old");
        let new = tool("changed", "note", &json!({"version":3}), "new");
        let orphan = tool("orphan", "note", &json!({"version":4}), "orphan");
        let last = tool("last", "note", &json!({"version":5}), "last");
        observed.told = vec![
            told(Agent::A, &pending[..1]),
            told(Agent::Director, &pending),
            told(Agent::A, &pending),
            told(Agent::A, &pending),
            told(Agent::A, &[pending.to_vec(), old[..1].to_vec()].concat()),
            told(Agent::A, &[pending.to_vec(), new.to_vec()].concat()),
            told(Agent::A, &[new.to_vec(), orphan[..1].to_vec()].concat()),
            told(Agent::A, &orphan[1..]),
            told(Agent::A, &last),
        ];
        assert_eq!(
            plan_tool_results(&observed, Agent::A, b"note"),
            vec![
                (2, json!({"version":1}), b"first".to_vec()),
                (5, json!({"version":3}), b"new".to_vec()),
                (8, json!({"version":5}), b"last".to_vec()),
            ]
        );

        // A bad argument never consumes an id; the last matching call wins,
        // even across a replay prefix. Roles and tool names remain significant.
        let mut invalid = tool("same", "note", &json!({"version":0}), "invalid");
        if let Part::ToolCall { arguments, .. } = &mut invalid[0].parts[0] {
            *arguments = b"{".as_slice().into();
        }
        let valid = tool("same", "note", &json!({"version":6}), "valid");
        let wrong_tool = tool("wrong", "other", &json!({}), "wrong");
        let mut wrong_role = tool("role", "note", &json!({}), "wrong");
        wrong_role[0].role = Role::User;
        observed.told = vec![
            told(Agent::A, &invalid),
            told(Agent::A, &[invalid.to_vec(), valid[..1].to_vec()].concat()),
            told(Agent::A, &[invalid.to_vec(), valid.to_vec(), wrong_tool.to_vec(), wrong_role.to_vec()].concat()),
            told(Agent::A, &valid),
        ];
        assert_eq!(plan_tool_results(&observed, Agent::A, b"note"), vec![(2, json!({"version":6}), b"valid".to_vec())]);
    }
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

const DIRECTION_AMENDED: Promise = Promise::Must("amendment (exact)");
const DIRECTION_CANCELLED: Promise = Promise::Must("cancel, with work kept under its closed pull");
const DIRECTION_OVERLAP: Promise = Promise::Must("grants by convention, with footprints as detection");
const DIRECTION_NEWS: Promise = Promise::Add("B's overlap news wakes it once released");

#[must_use]
pub fn step_5_4(observed: &Observed) -> Vec<Miss> {
    use super::{
        B_FILE, DEFERRED_NOTE, DEFERRED_NOTE_NUMBER, DEFERRED_WORDS, DIRECTION, DIRECTION_BRIEF, DIRECTION_REASON,
        DIRECTION_REPORT, DIRECTION_REVISION, OVERLAP_OWNER, OVERLAP_OWNER_REPORT, PLANNED, PLANNED_CANCEL_REASON,
        STORY_OWNER, STORY_OWNER_REPORT, STORY_WORDS, X3, X3_CANCEL_REASON, X3_FILE,
    };
    let mut misses = Vec::new();
    direction_identities(observed, &mut misses);
    let a = task_document(observed, Read::StoryOwnerEnded);
    step_check(
        &mut misses,
        "5.4",
        text(a, "/instructions/bytes") == DIRECTION_BRIEF
            && a.pointer("/instructions/cut") == Some(&Value::Bool(false))
            && number(a, "/spec_revision/value") == Some(DIRECTION_REVISION)
            && plan_deliveries(observed, Agent::StoryOwner, DIRECTION_BRIEF) == 1,
        DIRECTION_AMENDED,
        "A must hold the whole new direction at its revision and hear the amendment once",
    );
    step_check(
        &mut misses,
        "5.4",
        plan_tool_results(observed, Agent::Direction, b"amend").iter().any(|(_, args, _)| {
            args["target"] == STORY_OWNER
                && args["revision"] == DIRECTION_REVISION - 1
                && args["amendment"]["instructions"] == DIRECTION_BRIEF
                && args["amendment"]["reason"] == DIRECTION_REASON
        }),
        DIRECTION_AMENDED,
        "the director must amend A with the new direction and its reason",
    );
    for (read, task, by, reason) in [
        (Read::X3Ended, X3, STORY_OWNER, X3_CANCEL_REASON),
        (Read::PlannedEnded, PLANNED, OVERLAP_OWNER, PLANNED_CANCEL_REASON),
    ] {
        let doc = task_document(observed, read);
        step_check(
            &mut misses,
            "5.4",
            text(doc, "/chip/phase") == "ended"
                && number(doc, "/result/ending/task/cancelled/by/task") == Some(by)
                && text(doc, "/result/ending/task/cancelled/reason/bytes") == reason
                && number(doc, "/chip/details/owner/details/live_delegates") == Some(0),
            DIRECTION_CANCELLED,
            format_args!("task {task}'s subtree must end cancelled by {by} with its reason"),
        );
    }
    step_check(
        &mut misses,
        "5.4",
        plan_note_recalled(observed, Agent::StoryOwner, DEFERRED_NOTE_NUMBER, DEFERRED_NOTE, DEFERRED_WORDS)
            && DEFERRED_WORDS.contains(&format!("Task {X3} (x3)")),
        DIRECTION_CANCELLED,
        "A must write a project note naming x3 and its deferred plan, then recall it",
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
    direction_subtree(observed, &mut misses);
    direction_news(observed, &mut misses);
    direction_attempts(observed, &mut misses);
    direction_messages(observed, &mut misses);
    for (read, task, report) in [
        (Read::StoryOwnerEnded, STORY_OWNER, STORY_OWNER_REPORT),
        (Read::DirectionBEnded, OVERLAP_OWNER, OVERLAP_OWNER_REPORT),
        (Read::DirectionEnded, DIRECTION, DIRECTION_REPORT),
    ] {
        let doc = task_document(observed, read);
        step_check(
            &mut misses,
            "5.4",
            number(doc, "/chip/task") == Some(task)
                && number(doc, "/project") == Some(PROJECT)
                && text(doc, "/chip/phase") == "ended"
                && text(doc, "/result/value/report/bytes") == report,
            DIRECTION_NEWS,
            format_args!("task {task} must finish its report"),
        );
    }
    misses
}

fn direction_identities(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{DIRECTION, OVERLAP_OWNER, PLANNED, PLANNED_CODER, STORY_CHANGE, STORY_OWNER, X3};
    for (read, task, parent) in [
        (Read::X3Open, X3, STORY_OWNER),
        (Read::X3Ended, X3, STORY_OWNER),
        (Read::PlannedRunning, PLANNED, OVERLAP_OWNER),
        (Read::PlannedCoderRunning, PLANNED_CODER, PLANNED),
        (Read::PlannedHeld, PLANNED, OVERLAP_OWNER),
        (Read::PlannedCoderHeld, PLANNED_CODER, PLANNED),
        (Read::PlannedEnded, PLANNED, OVERLAP_OWNER),
        (Read::StoryLanded, STORY_CHANGE, STORY_OWNER),
        (Read::DirectionBHeld, OVERLAP_OWNER, DIRECTION),
        (Read::DirectionBPending, OVERLAP_OWNER, DIRECTION),
        (Read::DirectionBEnded, OVERLAP_OWNER, DIRECTION),
        (Read::StoryOwnerEnded, STORY_OWNER, DIRECTION),
        (Read::DirectionNoteWritten, STORY_OWNER, DIRECTION),
    ] {
        let doc = task_document(observed, read);
        step_check(
            misses,
            "5.4",
            number(doc, "/chip/task") == Some(task)
                && number(doc, "/project") == Some(PROJECT)
                && number(doc, "/requester/task") == Some(parent),
            DIRECTION_OVERLAP,
            format_args!("{read:?} must identify task {task} under {parent} in project {PROJECT}"),
        );
    }
}

fn direction_pulls(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{DIRECTION, STORY_CHANGE, STORY_PULL_NUMBER, X3, X3_CODER, X3_PULL_NUMBER};
    let x3_branch = format!("temper/{DIRECTION}/c{X3}");
    let a1_branch = format!("temper/{DIRECTION}/c{STORY_CHANGE}");
    let x3 = observed.kit.pulls.iter().find(|pull| pull.branch == x3_branch.as_bytes());
    let a1 = observed.kit.pulls.iter().find(|pull| pull.branch == a1_branch.as_bytes());
    step_check(
        misses,
        "5.4",
        observed.kit.pulls.len() == 2
            && x3.is_some_and(|pull| {
                pull.number == X3_PULL_NUMBER
                    && pull.closed
                    && !pull.merged
                    && pull.landed.is_none()
                    && pull.merged_head.is_none()
                    && pull.pushed == Some(pull.opened)
                    && pull.deleted == pull.pushed
            })
            && a1.is_some_and(|pull| {
                pull.number == STORY_PULL_NUMBER
                    && pull.merged
                    && pull.pushed == Some(pull.opened)
                    && pull.merged_head == pull.pushed
                    && pull.landed.is_some_and(|commit| {
                        landing_commit(task_document(observed, Read::StoryLanded)) == direction_commit(commit)
                    })
            }),
        DIRECTION_CANCELLED,
        "only x3 and a1 must open pulls; x3's unmerged pull must close and its branch be deleted at its opening head",
    );
    direction_closing_comment(observed, misses);
    let produced = task_document(observed, Read::X3Produced);
    step_check(
        misses,
        "5.4",
        number(produced, "/chip/task") == Some(X3_CODER)
            && number(produced, "/requester/task") == Some(X3)
            && number(produced, "/project") == Some(PROJECT)
            && text(produced, "/chip/phase") == "ended"
            && number(produced, "/result/value/change/kind") == Some(2)
            && x3.is_some_and(|pull| pull.pushed == Some(pull.opened)),
        DIRECTION_CANCELLED,
        "x3's coder must finish a change, whose pull opens at its observed pushed head",
    );
}

fn direction_comment_says(body: &[u8], reason: &[u8]) -> bool {
    let Some(key) = skein_forge::marker::decode(b"temper", body) else { return false };
    skein_forge::marker::encode(b"temper", &key, reason, u32::MAX).as_deref() == Some(body)
}

fn direction_closing_comment(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{KIT, X3_CANCEL_REASON, X3_PULL_NUMBER};
    use skein_fake_forge_domain::Observation;
    let comments: Vec<_> = observed
        .kit
        .records
        .iter()
        .enumerate()
        .filter_map(|(at, record)| {
            if let Observation::Commented { repository, number, body, .. } = record
                && repository.as_ref() == KIT.as_bytes()
                && *number == X3_PULL_NUMBER
            {
                Some((at, body.as_ref()))
            } else {
                None
            }
        })
        .collect();
    let closed = observed.kit.records.iter().position(|record| {
        matches!(record,
        Observation::Closed { repository, number, .. }
            if repository.as_ref() == KIT.as_bytes() && *number == X3_PULL_NUMBER)
    });
    let reason = X3_CANCEL_REASON.as_bytes();
    step_check(
        misses,
        "5.4",
        comments.iter().filter(|(_, body)| direction_comment_says(body, reason)).count() == 1
            && closed.is_some_and(|closed| {
                comments
                    .iter()
                    .rev()
                    .find(|(at, _)| *at < closed)
                    .is_some_and(|(_, body)| direction_comment_says(body, reason))
            })
            && comments
                .iter()
                .filter(|(_, body)| direction_comment_says(body, reason))
                .all(|(at, _)| closed.is_some_and(|closed| *at < closed)),
        DIRECTION_CANCELLED,
        "x3's last comment before closing must say A's cancel reason, once",
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
fn direction_subtree(observed: &Observed, misses: &mut Vec<Miss>) {
    let root = task_document(observed, Read::DirectionBHeld);
    let hold = number(root, "/hold/reason/placed/hold");
    for (before, held) in
        [(Read::PlannedRunning, Read::PlannedHeld), (Read::PlannedCoderRunning, Read::PlannedCoderHeld)]
    {
        let previous = task_document(observed, before);
        let doc = task_document(observed, held);
        step_check(
            misses,
            "5.4",
            hold.is_some_and(|hold| hold != 0)
                && text(doc, "/chip/phase") == "held"
                && number(doc, "/hold/reason/placed/hold") == hold
                && doc.pointer("/result") == Some(&Value::Null)
                && number(previous, "/chip/details/owner/details/attempt").is_some()
                && number(previous, "/chip/details/owner/details/attempt")
                    == number(doc, "/chip/details/owner/details/attempt"),
            DIRECTION_NEWS,
            format_args!(
                "{held:?}: B's subtree must share its placed hold without finishing or starting another attempt"
            ),
        );
    }
}

fn direction_news(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{B_FILE, OVERLAP_HOLD_REVISION, OVERLAP_OWNER, OVERLAP_SUBSCRIPTION, PLANNED};
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
    let planned = task_document(observed, Read::PlannedRunning);
    step_check(
        misses,
        "5.4",
        planned.pointer("/spec/specification/parameters").and_then(Value::as_array).is_some_and(|parameters| {
            parameters.iter().any(|parameter| {
                number(parameter, "/bytes/name") == Some(6) && text(parameter, "/bytes/value/bytes") == B_FILE
            })
        }) && number(planned, "/chip/task") == Some(PLANNED),
        DIRECTION_OVERLAP,
        "h1 must name B's file as its planned footprint before it produces a pull",
    );
    let pending_rows = check_activity(observed, Read::DirectionBPending);
    let news: Vec<_> = pending_rows
        .iter()
        .filter(|row| {
            text(row, "/line/heard/kind/news/class") == "wakes"
                && text(row, "/line/heard/words").starts_with("forge landing ")
        })
        .collect();
    let stopped = number(held, "/activity");
    let ended = task_document(observed, Read::DirectionBEnded);
    let rows: Vec<_> = joined_activity(observed, &[Read::DirectionBEnded, Read::DirectionBFirst]);
    let release = rows
        .iter()
        .find(|row| number(row, "/line/decided/revision") == Some(OVERLAP_HOLD_REVISION))
        .and_then(|row| number(row, "/position"));
    let same_attempt = number(held, "/chip/details/owner/details/attempt").is_some()
        && number(held, "/chip/details/owner/details/attempt")
            == number(pending, "/chip/details/owner/details/attempt");
    step_check(
        misses,
        "5.4",
        same_attempt
            && release.zip(number(pending, "/activity")).is_some_and(|(release, head)| release > head)
            && stopped.zip(release).is_some_and(|(stop, release)| {
                stop < release
                    && rows.iter().all(|row| {
                        !number(row, "/position").is_some_and(|at| at > stop && at < release)
                            || row.pointer("/line/run_began").is_none()
                    })
            })
            && rows.iter().any(|row| {
                row.pointer("/line/run_began").is_some()
                    && number(row, "/position").zip(release).is_some_and(|(at, release)| at > release)
            })
            && ended["hold"].is_null(),
        DIRECTION_NEWS,
        "B must run no new attempt during its hold and begin its next attempt only after Release",
    );
    let landing = text(task_document(observed, Read::StoryLanded), "/result/value/change/words/bytes");
    step_check(
        misses,
        "5.4",
        news.len() == 1
            && news.first().is_some_and(|row| {
                let words = text(row, "/line/heard/words");
                let suffix = landing.lines().next().map(|commit| format!(" -> {commit}"));
                direction_news_identity(&rows, row)
                    && number(row, "/line/heard/from/task") == Some(OVERLAP_OWNER)
                    && number(row, "/line/heard/hits") == Some(1)
                    && number(row, "/line/heard/kind/news/subscription") == Some(OVERLAP_SUBSCRIPTION)
                    && suffix.is_some_and(|suffix| words.ends_with(&suffix))
                    && number(row, "/position").zip(stopped).is_some_and(|(at, stop)| at > stop)
                    && plan_deliveries(observed, Agent::OverlapOwner, words) == 1
            }),
        DIRECTION_NEWS,
        "a1's overlapping landing must wake B once released, waiting in its held inbox and being told exactly once",
    );
    step_check(
        misses,
        "5.4",
        plan_attempts(
            observed,
            &direction_pages(observed, Read::DirectionBEnded, Read::DirectionBFirst),
            OVERLAP_OWNER,
            2,
        ),
        DIRECTION_NEWS,
        "B's complete journal must show its two non-overlapping attempts",
    );
}

// Fresh numbers are shared with other notices. Identify B's news by the same
// immutable event in both reads, and reject another copy anywhere in its journal.
fn direction_news_identity(rows: &[&Value], news: &Value) -> bool {
    let Some(message) = number(news, "/line/heard/number") else { return false };
    if message == 0 {
        return false;
    }
    let words = text(news, "/line/heard/words");
    let mut identities = 0;
    let mut landings = 0;
    for row in rows {
        if number(row, "/line/heard/number") == Some(message) {
            if *row != news {
                return false;
            }
            identities += 1;
        }
        if row.pointer("/line/heard/kind/news").is_some() && text(row, "/line/heard/words") == words {
            landings += 1;
        }
    }
    identities == 1 && landings == 1
}

fn direction_messages(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{
        DIRECTION, DIRECTION_AMENDMENT_MESSAGE, DIRECTION_BRIEF, DIRECTION_MESSAGE, DIRECTION_PERSON_MESSAGE,
        DIRECTION_REVISION,
    };
    step_check(
        misses,
        "5.4",
        check_activity(observed, Read::StoryOwnerFirst).iter().any(|row| {
            number(row, "/line/heard/number") == Some(DIRECTION_AMENDMENT_MESSAGE)
                && number(row, "/line/heard/from/task") == Some(DIRECTION)
                && number(row, "/line/heard/kind/amendment/revision") == Some(DIRECTION_REVISION)
                && text(row, "/line/heard/words") == DIRECTION_BRIEF
        }) && check_activity(observed, Read::DirectionEnded).iter().any(|row| {
            number(row, "/line/words/message") == Some(DIRECTION_PERSON_MESSAGE)
                && number(row, "/line/words/by/person") == Some(1)
                && text(row, "/line/words/words") == DIRECTION_MESSAGE
        }),
        DIRECTION_AMENDED,
        "the person's direction and A's amendment must have their fresh message identities and revision",
    );
}

// Full journals, including pages with the starts of earlier runs. Procedures
// have journal lines but no agent attempts. A refused placement recheck is
// allowed only for B's first admission, at zero spend (jig's domain/engine/executors.md,
// 1: queued placement losing its selected slot returns the claim without a
// failed try; jig-domain-fleet's attempt.rs, resume, and jig-domain-core's
// routing/children.rs translate Recheck to the journal's refused ending).
// B and the director now fit on one page. Retain support for an earlier
// page in oracle fixtures: plan_attempts still requires complete, ordered
// journals, and rejects an absent page whenever the latest has a cursor.
fn direction_pages(observed: &Observed, latest: Read, earlier: Read) -> Vec<Read> {
    let mut reads = vec![latest];
    if observed.documents.contains_key(&earlier) {
        reads.push(earlier);
    }
    reads
}
fn direction_attempts(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{DIRECTION, OVERLAP_OWNER, OVERLAP_RECHECK_ATTEMPT, PLANNED, STORY_CHANGE, STORY_OWNER, X3, X3_CODER};
    let b_reads = direction_pages(observed, Read::DirectionBEnded, Read::DirectionBFirst);
    let director_reads = direction_pages(observed, Read::DirectionEnded, Read::DirectionFirst);
    let mut owner_reads = vec![Read::StoryOwnerEnded, Read::StoryOwnerEarlier, Read::StoryOwnerFirst];
    if pause_activity(observed, Read::StoryOwnerFirst)["before"] != Value::Null {
        owner_reads.push(Read::StoryOwnerInitial);
    }
    for (task, reads, minimum) in [
        (X3, &[Read::X3Ended][..], 0),
        (X3_CODER, &[Read::X3Produced][..], 1),
        (PLANNED, &[Read::PlannedEnded][..], 0),
        (STORY_CHANGE, &[Read::StoryLanded][..], 0),
        (STORY_OWNER, owner_reads.as_slice(), 1),
        (OVERLAP_OWNER, b_reads.as_slice(), 2),
        (DIRECTION, director_reads.as_slice(), 1),
    ] {
        let rows: Vec<_> = joined_activity(observed, reads);
        step_check(
            misses,
            "5.4",
            plan_attempt_rows(observed, reads, task, minimum, &rows)
                && {
                    rows.last().is_some_and(|row| number(row, "/position") == Some(1))
                        && rows.windows(2).all(|pair| {
                            number(pair[0], "/position")
                                .zip(number(pair[1], "/position"))
                                .is_some_and(|(new, old)| new.checked_sub(1) == Some(old))
                        })
                }
                && rows.iter().all(|row| {
                    let Some(end) = row.pointer("/line/run_ended") else { return true };
                    matches!(end["end"].as_str(), Some("finished" | "parked"))
                        || (task == OVERLAP_OWNER
                            && end["end"] == "refused"
                            && number(end, "/attempt") == Some(OVERLAP_RECHECK_ATTEMPT)
                            && number(end, "/spent") == Some(0))
                }),
            DIRECTION_NEWS,
            format_args!(
                "task {task}'s complete journal must end every counted attempt without failure or an unexplained refusal"
            ),
        );
    }
}

#[cfg(test)]
mod direction_tests {
    use super::*;
    use crate::case::*;
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    const OVERLAP_LANDING_MESSAGE: u64 = 18;

    fn input(words: &str) -> Message {
        Message { role: Role::User, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    fn tool(id: &str, name: &str, args: &Value, output: &str) -> [Message; 2] {
        [
            Message {
                role: Role::Assistant,
                parts: Box::new([Part::ToolCall {
                    id: id.as_bytes().into(),
                    name: name.as_bytes().into(),
                    arguments: serde_json::to_vec(args).expect("arguments").into(),
                }]),
            },
            Message {
                role: Role::User,
                parts: Box::new([Part::ToolOutput {
                    id: id.as_bytes().into(),
                    output: output.as_bytes().into(),
                    is_error: false,
                }]),
            },
        ]
    }
    fn told(agent: Agent, history: &[Message]) -> Told {
        Told::with_agent(agent, agent.cue(), b"fake", None, history)
    }
    fn journal(task: u64, attempts: usize, first: u64) -> Value {
        let lines = if attempts == 0 {
            vec![json!({"milestone":{"position":1}})]
        } else {
            vec![
                json!({"run_began":{"attempt":first}}),
                json!({"run_ended":{"attempt":first,"end":"finished","spent":1}}),
            ]
        };
        let rows: Vec<_> = lines
            .into_iter()
            .enumerate()
            .rev()
            .map(
                |(at, line)| json!({"event":{"owner":{"source":{"task":task,"position":first+at as u64,"line":line}}}}),
            )
            .collect();
        json!({"rows":rows,"cut":false,"before":null,"after":null})
    }
    fn documents() -> BTreeMap<Read, Value> {
        let mut documents = BTreeMap::new();
        for (read, task, parent) in [
            (Read::X3Open, X3, STORY_OWNER),
            (Read::X3Produced, X3_CODER, X3),
            (Read::X3Ended, X3, STORY_OWNER),
            (Read::PlannedRunning, PLANNED, OVERLAP_OWNER),
            (Read::PlannedCoderRunning, PLANNED_CODER, PLANNED),
            (Read::PlannedHeld, PLANNED, OVERLAP_OWNER),
            (Read::PlannedCoderHeld, PLANNED_CODER, PLANNED),
            (Read::PlannedEnded, PLANNED, OVERLAP_OWNER),
            (Read::StoryLanded, STORY_CHANGE, STORY_OWNER),
            (Read::DirectionBHeld, OVERLAP_OWNER, DIRECTION),
            (Read::DirectionBPending, OVERLAP_OWNER, DIRECTION),
            (Read::DirectionBEnded, OVERLAP_OWNER, DIRECTION),
            (Read::StoryOwnerEnded, STORY_OWNER, DIRECTION),
            (Read::DirectionNoteWritten, STORY_OWNER, DIRECTION),
            (Read::DirectionEnded, DIRECTION, 0),
        ] {
            documents.insert(read, json!({"task":{
                "chip":{"task":task,"phase":"ended","details":{"owner":{"details":{"attempt":1,"live_delegates":0}}}},
                "project":PROJECT,"requester":{"task":parent},"hold":null,
                "result":{"value":{}}
            }}));
        }
        for (read, by, reason) in
            [(Read::X3Ended, STORY_OWNER, X3_CANCEL_REASON), (Read::PlannedEnded, OVERLAP_OWNER, PLANNED_CANCEL_REASON)]
        {
            documents.get_mut(&read).expect("cancelled")["task"]["result"]["ending"] =
                json!({"task":{"cancelled":{"by":{"task":by},"reason":{"bytes":reason}}}});
        }
        for (read, report) in [
            (Read::StoryOwnerEnded, STORY_OWNER_REPORT),
            (Read::DirectionBEnded, OVERLAP_OWNER_REPORT),
            (Read::DirectionEnded, DIRECTION_REPORT),
        ] {
            documents.get_mut(&read).expect("report")["task"]["result"]["value"] = json!({"report":{"bytes":report}});
        }
        let a = &mut documents.get_mut(&Read::StoryOwnerEnded).expect("A")["task"];
        a["instructions"] = json!({"bytes":DIRECTION_BRIEF,"cut":false});
        a["spec_revision"] = json!({"value":DIRECTION_REVISION});
        documents.get_mut(&Read::X3Produced).expect("coder")["task"]["result"]["value"] =
            json!({"change":{"kind":2,"words":{"bytes":"The deferred plan."}}});
        let a1 = &mut documents.get_mut(&Read::StoryLanded).expect("a1")["task"];
        a1["result"] = json!({"ending":{"task":{"done":{"change":{"kind":1}}}},
            "value":{"change":{"part":{"connector":1,"kind":"result"},"words":{"bytes":direction_commit(9)}}}});
        documents.get_mut(&Read::PlannedRunning).expect("h1")["task"]["spec"] = json!({"specification":{
            "parameters":[{"bytes":{"name":6,"value":{"bytes":B_FILE,"cut":false}}}]
        }});
        for read in [Read::PlannedHeld, Read::PlannedCoderHeld] {
            let doc = &mut documents.get_mut(&read).expect("held delegate")["task"];
            doc["chip"]["phase"] = json!("held");
            doc["result"] = Value::Null;
            doc["hold"] = json!({"revision":OVERLAP_HOLD_REVISION,"reason":{"placed":{"hold":1}}});
        }
        let news = format!("forge landing {} -> {}", direction_commit(8), direction_commit(9));
        let rows: Vec<_> = [
            json!({"run_began":{"attempt":1}}),
            json!({"run_ended":{"attempt":1,"end":"parked","spent":1}}),
            json!({"decided":{"revision":OVERLAP_HOLD_REVISION}}),
            json!({"heard":{"number":OVERLAP_LANDING_MESSAGE,"from":{"task":OVERLAP_OWNER},"hits":1,"kind":{"news":{"class":"wakes","subscription":OVERLAP_SUBSCRIPTION}},"words":news}}),
            json!({"decided":{"revision":OVERLAP_HOLD_REVISION}}),
            json!({"run_began":{"attempt":2}}),
            json!({"run_ended":{"attempt":2,"end":"finished","spent":1}}),
        ]
        .into_iter()
        .enumerate()
        .map(|(at, line)| {
            json!({"event":{"owner":{"source":{
                "task":OVERLAP_OWNER,"position":at+1,"line":line
            }}}})
        })
        .rev()
        .collect();
        for (read, head) in [(Read::DirectionBHeld, 3), (Read::DirectionBPending, 4)] {
            let doc = documents.get_mut(&read).expect("held");
            doc["task"]["chip"]["phase"] = json!("held");
            doc["task"]["activity"] = json!(head);
            doc["task"]["hold"] = json!({"revision":OVERLAP_HOLD_REVISION,"reason":{"placed":{"hold":1}}});
            doc["activity"] = json!({"rows":if head == 4 { vec![rows[3].clone()] } else { Vec::new() }});
        }
        documents.get_mut(&Read::DirectionBEnded).expect("B")["activity"] =
            json!({"cut":false,"after":null,"before":{"value":"older"},"rows":rows[..3]});
        documents.insert(
            Read::DirectionBFirst,
            json!({"cut":false,"before":null,"after":{"value":"newer"},"rows":rows[3..]}),
        );
        journals(&mut documents);
        documents
    }

    fn journals(documents: &mut BTreeMap<Read, Value>) {
        for (read, task, minimum) in [
            (Read::X3Ended, X3, 0),
            (Read::X3Produced, X3_CODER, 1),
            (Read::PlannedEnded, PLANNED, 0),
            (Read::StoryLanded, STORY_CHANGE, 0),
        ] {
            documents.get_mut(&read).expect("task")["activity"] = journal(task, minimum, 1);
        }
        let mut newest = journal(STORY_OWNER, 1, 6);
        newest["before"] = json!({"value":"older"});
        documents.get_mut(&Read::StoryOwnerEnded).expect("A")["activity"] = newest;
        let mut middle = journal(STORY_OWNER, 1, 4);
        middle["before"] = json!({"value":"first"});
        documents.insert(Read::StoryOwnerEarlier, middle);
        documents.insert(Read::StoryOwnerFirst, journal(STORY_OWNER, 1, 1));
        let first =
            documents.get_mut(&Read::StoryOwnerFirst).expect("A's first page")["rows"].as_array_mut().expect("rows");
        first.insert(
            0,
            json!({"event":{"owner":{"source":{"task":STORY_OWNER,"position":3,"line":{
                "heard":{"number":DIRECTION_AMENDMENT_MESSAGE,"from":{"task":DIRECTION},
                "kind":{"amendment":{"revision":DIRECTION_REVISION}},"words":DIRECTION_BRIEF}
            }}}}}),
        );
        let mut newest = journal(DIRECTION, 1, 4);
        newest["rows"].as_array_mut().expect("rows").push(json!({"event":{"owner":{"source":{
            "task":DIRECTION,"position":3,"line":{"words":{"message":DIRECTION_PERSON_MESSAGE,
            "by":{"person":1},"words":DIRECTION_MESSAGE}}
        }}}}));
        newest["before"] = json!({"value":"first"});
        documents.get_mut(&Read::DirectionEnded).expect("director")["activity"] = newest;
        documents.insert(Read::DirectionFirst, journal(DIRECTION, 1, 1));
    }

    pub(super) fn met() -> Observed {
        let documents = documents();
        let news = format!("forge landing {} -> {}", direction_commit(8), direction_commit(9));
        let amendment = tool(
            "amend",
            "amend",
            &json!({"target":STORY_OWNER,"revision":DIRECTION_REVISION-1,
            "amendment":{"instructions":DIRECTION_BRIEF,"reason":DIRECTION_REASON}}),
            "Amended",
        );
        let mut a = vec![input(DIRECTION_BRIEF)];
        a.extend(tool(
            "note",
            "note",
            &json!({"scope":{"kind":"project","project":PROJECT},
            "description":DEFERRED_NOTE,"body":DEFERRED_WORDS}),
            &format!("Note {DEFERRED_NOTE_NUMBER} revision 1 written · 1 lines"),
        ));
        let noted = told(Agent::StoryOwner, &a);
        a.extend(tool(
            "recall",
            "recall",
            &json!({"kind":"name","name":DEFERRED_NOTE_NUMBER}),
            &format!("Note {DEFERRED_NOTE_NUMBER} revision 1: {DEFERRED_NOTE}\n{DEFERRED_WORDS}\n"),
        ));
        Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents,
            commands: Vec::new(),
            told: vec![
                told(Agent::Direction, &amendment),
                noted,
                told(Agent::StoryOwner, &a),
                told(Agent::OverlapOwner, &[input("Begin B's work.")]),
                told(Agent::OverlapOwner, &[input("Begin B's work."), input(&news)]),
            ],
            kit: Kit {
                records: forge_records(),
                main: BTreeMap::from([(B_FILE.as_bytes().to_vec(), STORY_WORDS.as_bytes().to_vec())]),
                merged_green: true,
                pulls: vec![
                    Pull {
                        repository: crate::case::KIT.as_bytes().to_vec(),
                        number: X3_PULL_NUMBER,
                        branch: format!("temper/{DIRECTION}/c{X3}").into_bytes(),
                        opened: 7,
                        pushed: Some(7),
                        landed: None,
                        merged_head: None,
                        merged: false,
                        closed: true,
                        deleted: Some(7),
                    },
                    Pull {
                        repository: crate::case::KIT.as_bytes().to_vec(),
                        number: STORY_PULL_NUMBER,
                        branch: format!("temper/{DIRECTION}/c{STORY_CHANGE}").into_bytes(),
                        opened: 8,
                        pushed: Some(8),
                        landed: Some(9),
                        merged_head: Some(8),
                        merged: true,
                        closed: false,
                        deleted: None,
                    },
                ],
            },
        }
    }
    #[test]
    fn new_direction_and_deferred_overlap_meet_the_checkpoint() {
        assert_met(&step_5_4(&met()));
    }
    #[test]
    fn overlap_news_identity_is_read_from_both_journals() {
        for message in [19, 42] {
            let mut observed = met();
            for read in [Read::DirectionBPending, Read::DirectionBEnded, Read::DirectionBFirst] {
                let doc = observed.documents.get_mut(&read).expect("B's journal");
                let page = if read == Read::DirectionBFirst { doc } else { &mut doc["activity"] };
                for row in page["rows"].as_array_mut().expect("rows") {
                    let Some(number) = row.pointer_mut("/event/owner/source/line/heard/number") else { continue };
                    if *number == OVERLAP_LANDING_MESSAGE {
                        *number = json!(message);
                    }
                }
            }
            assert_met(&step_5_4(&observed));
            observed.documents.get_mut(&Read::DirectionBPending).expect("pending")["activity"]["rows"][0]["event"]["owner"]
                ["source"]["line"]["heard"]["number"] = json!(message + 1);
            assert!(!step_5_4(&observed).is_empty(), "the two reads must name the same event");
        }
    }
    #[test]
    fn overlap_news_rejects_another_landing_notice_in_the_complete_journal() {
        let mut observed = met();
        let page = observed.documents.get_mut(&Read::DirectionBFirst).expect("B's earlier journal");
        let mut duplicate = page["rows"][0]["event"]["owner"]["source"]["line"].clone();
        duplicate["heard"]["number"] = json!(OVERLAP_LANDING_MESSAGE + 1);
        page["rows"][1]["event"]["owner"]["source"]["line"] = duplicate;
        assert!(!step_5_4(&observed).is_empty(), "the landing must reach B once in its whole journal");
    }
    #[test]
    fn complete_single_page_journals_keep_the_same_checks() {
        let mut observed = met();
        for (latest, earlier) in
            [(Read::DirectionBEnded, Read::DirectionBFirst), (Read::DirectionEnded, Read::DirectionFirst)]
        {
            let older = observed.documents.remove(&earlier).expect("earlier page");
            let page = &mut observed.documents.get_mut(&latest).expect("latest page")["activity"];
            page["rows"].as_array_mut().expect("rows").extend(older["rows"].as_array().expect("older rows").clone());
            page["before"] = Value::Null;
        }
        assert_met(&step_5_4(&observed));
        for latest in [Read::DirectionBEnded, Read::DirectionEnded] {
            let mut missing = observed.clone();
            missing.documents.get_mut(&latest).expect("latest")["activity"]["before"] = json!({"value":"missing"});
            assert!(!step_5_4(&missing).is_empty(), "{latest:?} must not omit an earlier page");
            let mut gap = observed.clone();
            gap.documents.get_mut(&latest).expect("latest")["activity"]["rows"].as_array_mut().expect("rows").remove(1);
            assert!(!step_5_4(&gap).is_empty(), "{latest:?} must have no journal gap");
        }
    }

    #[test]
    fn every_required_document_field_rejects_a_mutation() {
        let passing = met();
        for (read, paths) in [
            (
                Read::StoryOwnerEnded,
                vec!["/task/instructions/bytes", "/task/instructions/cut", "/task/spec_revision/value"],
            ),
            (
                Read::X3Produced,
                vec![
                    "/task/chip/task",
                    "/task/chip/phase",
                    "/task/requester/task",
                    "/task/project",
                    "/task/result/value/change/kind",
                ],
            ),
            (
                Read::X3Ended,
                vec![
                    "/task/result/ending/task/cancelled/by/task",
                    "/task/result/ending/task/cancelled/reason/bytes",
                    "/task/chip/details/owner/details/live_delegates",
                ],
            ),
            (
                Read::PlannedEnded,
                vec![
                    "/task/result/ending/task/cancelled/by/task",
                    "/task/result/ending/task/cancelled/reason/bytes",
                    "/task/chip/details/owner/details/live_delegates",
                ],
            ),
            (
                Read::PlannedRunning,
                vec![
                    "/task/spec/specification/parameters/0/bytes/name",
                    "/task/spec/specification/parameters/0/bytes/value/bytes",
                ],
            ),
            (
                Read::StoryLanded,
                vec![
                    "/task/result/ending/task/done/change/kind",
                    "/task/result/value/change/part/connector",
                    "/task/result/value/change/part/kind",
                    "/task/result/value/change/words/bytes",
                ],
            ),
            (
                Read::DirectionBHeld,
                vec![
                    "/task/chip/phase",
                    "/task/hold/revision",
                    "/task/hold/reason/placed/hold",
                    "/task/chip/details/owner/details/attempt",
                    "/task/activity",
                ],
            ),
            (
                Read::DirectionBPending,
                vec![
                    "/task/chip/phase",
                    "/task/hold/revision",
                    "/task/hold/reason/placed/hold",
                    "/task/chip/details/owner/details/attempt",
                    "/task/activity",
                    "/activity/rows/0/event/owner/source/line/heard/kind/news/class",
                    "/activity/rows/0/event/owner/source/line/heard/words",
                ],
            ),
            (
                Read::DirectionBEnded,
                vec!["/activity/cut", "/activity/after", "/activity/rows/2/event/owner/source/line/decided/revision"],
            ),
            (Read::DirectionBFirst, vec!["/cut", "/before"]),
        ] {
            for path in paths {
                let mut observed = passing.clone();
                let value = observed.documents.get_mut(&read).expect("document").pointer_mut(path).expect("field");
                *value = if value.is_null() { json!("unexpected") } else { Value::Null };
                assert!(!step_5_4(&observed).is_empty(), "{read:?}{path}");
            }
        }
    }
    #[test]
    fn descendants_must_share_bs_hold_without_advancing_or_finishing() {
        let passing = met();
        let mut changed = passing.clone();
        changed.documents.get_mut(&Read::DirectionBPending).expect("still held")["task"]["hold"]["reason"]["placed"]
            ["hold"] = json!(99);
        assert!(!step_5_4(&changed).is_empty(), "B keeps the same hold until release");
        for read in [Read::PlannedHeld, Read::PlannedCoderHeld] {
            for (path, replacement) in [
                ("/task/chip/phase", json!("running")),
                ("/task/result", json!({"value":{}})),
                ("/task/hold/reason/placed/hold", json!(99)),
                ("/task/chip/details/owner/details/attempt", json!(99)),
            ] {
                let mut observed = passing.clone();
                *observed.documents.get_mut(&read).expect("held descendant").pointer_mut(path).expect("field") =
                    replacement;
                assert!(!step_5_4(&observed).is_empty(), "{read:?}{path}");
            }
        }
    }

    #[test]
    fn identities_and_reports_reject_a_mutation() {
        let mut passing = met();
        for (read, _, _) in [
            (Read::X3Open, X3, STORY_OWNER),
            (Read::X3Ended, X3, STORY_OWNER),
            (Read::PlannedRunning, PLANNED, OVERLAP_OWNER),
            (Read::PlannedCoderRunning, PLANNED_CODER, PLANNED),
            (Read::PlannedHeld, PLANNED, OVERLAP_OWNER),
            (Read::PlannedCoderHeld, PLANNED_CODER, PLANNED),
            (Read::PlannedEnded, PLANNED, OVERLAP_OWNER),
            (Read::StoryLanded, STORY_CHANGE, STORY_OWNER),
            (Read::DirectionBHeld, OVERLAP_OWNER, DIRECTION),
            (Read::DirectionBPending, OVERLAP_OWNER, DIRECTION),
            (Read::DirectionBEnded, OVERLAP_OWNER, DIRECTION),
            (Read::StoryOwnerEnded, STORY_OWNER, DIRECTION),
            (Read::DirectionNoteWritten, STORY_OWNER, DIRECTION),
        ] {
            for path in ["/task/chip/task", "/task/project", "/task/requester/task"] {
                let saved = passing.documents.get_mut(&read).expect("read").pointer_mut(path).expect("field").take();
                assert!(!step_5_4(&passing).is_empty(), "{read:?}{path}");
                *passing.documents.get_mut(&read).expect("read").pointer_mut(path).expect("field") = saved;
            }
        }
        for read in [Read::StoryOwnerEnded, Read::DirectionBEnded, Read::DirectionEnded] {
            for path in ["/task/chip/task", "/task/project", "/task/chip/phase", "/task/result/value/report/bytes"] {
                let saved = passing.documents.get_mut(&read).expect("read").pointer_mut(path).expect("field").take();
                assert!(!step_5_4(&passing).is_empty(), "{read:?}{path}");
                *passing.documents.get_mut(&read).expect("read").pointer_mut(path).expect("field") = saved;
            }
        }
    }
    #[test]
    fn the_closed_pull_must_keep_the_pushed_head_and_h1_must_open_nothing() {
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
                7 => observed.kit.pulls.push(Pull {
                    repository: crate::case::KIT.as_bytes().to_vec(),
                    number: 3,
                    branch: format!("temper/{DIRECTION}/c{PLANNED}").into_bytes(),
                    opened: 9,
                    pushed: Some(9),
                    landed: None,
                    merged_head: None,
                    merged: false,
                    closed: true,
                    deleted: Some(9),
                }),
                _ => unreachable!(),
            }
            assert!(!step_5_4(&observed).is_empty(), "mutation {mutation}");
        }
        let mut red = passing.clone();
        red.kit.merged_green = false;
        let mut absent = passing.clone();
        absent.kit.main.clear();
        let mut old = passing.clone();
        old.kit.main.insert(X3_FILE.as_bytes().to_vec(), X3_WORDS.as_bytes().to_vec());
        for observed in [red, absent, old] {
            assert!(!step_5_4(&observed).is_empty());
        }
    }
    #[test]
    fn news_cannot_be_doubled_or_lost_and_a_run_cannot_start_during_the_hold() {
        let passing = met();
        let mut doubled = passing.clone();
        let extra = doubled.told[4].history[1].clone();
        doubled.told[4].history = [doubled.told[4].history.to_vec(), vec![extra]].concat().into();
        let mut lost = passing.clone();
        lost.told.pop();
        let mut during = passing.clone();
        during.documents.get_mut(&Read::DirectionBFirst).expect("older")["rows"][0]["event"]["owner"]["source"]["line"] =
            json!({"run_began":{"attempt":2}});
        let mut early = passing.clone();
        early.documents.get_mut(&Read::DirectionBEnded).expect("B")["activity"]["rows"][2]["event"]["owner"]["source"]
            ["position"] = json!(2);
        for observed in [doubled, lost, during, early] {
            assert!(!step_5_4(&observed).is_empty());
        }
    }
    #[test]
    fn amendment_and_note_must_be_real_completed_tools() {
        let passing = met();
        for at in [0, 2] {
            let mut observed = passing.clone();
            observed.told.remove(at);
            assert!(!step_5_4(&observed).is_empty());
        }
        let mut missing = passing.clone();
        missing.told[1].history = Box::new([]);
        missing.told[2].history = Box::new([]);
        let mut duplicate = passing.clone();
        duplicate.told[2].history = [duplicate.told[2].history.to_vec(), vec![input(DIRECTION_BRIEF)]].concat().into();
        let mut prose = passing.clone();
        prose.told[0].history = vec![input(DIRECTION_BRIEF)].into();
        for observed in [missing, duplicate, prose] {
            assert!(!step_5_4(&observed).is_empty());
        }
    }
    fn ending_mutations(read: Read, task: u64) {
        for ending in [json!({"failed":"invalid"}), json!("refused"), json!("exceeded")] {
            let mut observed = met();
            let doc = observed.documents.get_mut(&read).expect("journal");
            let page = if doc.get("activity").is_some() { &mut doc["activity"] } else { doc };
            let rows = page["rows"].as_array_mut().expect("rows");
            let position = rows[0]["event"]["owner"]["source"]["position"].as_u64().expect("position");
            // Add a later, fully paired attempt, so a successful final result
            // cannot hide an invalid earlier activation.
            rows.insert(
                0,
                json!({"event":{"owner":{"source":{"task":task,"position":position+1,
                "line":{"run_began":{"attempt":99}}}}}}),
            );
            rows.insert(
                0,
                json!({"event":{"owner":{"source":{"task":task,"position":position+2,
                "line":{"run_ended":{"attempt":99,"end":ending,"spent":0}}}}}}),
            );
            assert!(
                step_5_4(&observed)
                    .iter()
                    .any(|miss| miss.what.starts_with(&format!("task {task}'s complete journal")))
            );
        }
    }
    #[test]
    fn x3_cannot_hide_a_failed_or_refused_attempt() {
        ending_mutations(Read::X3Ended, X3);
    }
    #[test]
    fn x3s_coder_cannot_hide_a_failed_or_refused_attempt() {
        ending_mutations(Read::X3Produced, X3_CODER);
    }
    #[test]
    fn h1_cannot_hide_a_failed_or_refused_attempt() {
        ending_mutations(Read::PlannedEnded, PLANNED);
    }
    #[test]
    fn a1_cannot_hide_a_failed_or_refused_attempt() {
        ending_mutations(Read::StoryLanded, STORY_CHANGE);
    }
    #[test]
    fn a_cannot_hide_a_failed_or_refused_attempt() {
        ending_mutations(Read::StoryOwnerEnded, STORY_OWNER);
    }
    #[test]
    fn b_cannot_hide_a_failed_or_refused_attempt() {
        ending_mutations(Read::DirectionBEnded, OVERLAP_OWNER);
    }
    #[test]
    fn the_director_cannot_hide_a_failed_or_refused_attempt() {
        ending_mutations(Read::DirectionEnded, DIRECTION);
    }

    #[test]
    fn only_bs_zero_spend_first_placement_recheck_is_allowed() {
        let mut observed = met();
        for read in [Read::DirectionBEnded, Read::DirectionBFirst, Read::DirectionBPending, Read::DirectionBHeld] {
            let doc = observed.documents.get_mut(&read).expect("B");
            if let Some(head) = doc.pointer_mut("/task/activity") {
                *head = json!(head.as_u64().expect("head") + 2);
            }
            let page = if doc.get("activity").is_some() { &mut doc["activity"] } else { doc };
            for row in page["rows"].as_array_mut().expect("rows") {
                let position = &mut row["event"]["owner"]["source"]["position"];
                *position = json!(position.as_u64().expect("position") + 2);
            }
        }
        let rows =
            observed.documents.get_mut(&Read::DirectionBFirst).expect("B first")["rows"].as_array_mut().expect("rows");
        rows.push(json!({"event":{"owner":{"source":{"task":OVERLAP_OWNER,"position":2,
            "line":{"run_ended":{"attempt":OVERLAP_RECHECK_ATTEMPT,"end":"refused","spent":0}}}}}}));
        rows.push(json!({"event":{"owner":{"source":{"task":OVERLAP_OWNER,"position":1,
            "line":{"run_began":{"attempt":OVERLAP_RECHECK_ATTEMPT}}}}}}));
        assert_met(&step_5_4(&observed));
        for field in ["spent", "attempt"] {
            let mut changed = observed.clone();
            let rows = changed.documents.get_mut(&Read::DirectionBFirst).expect("B first")["rows"]
                .as_array_mut()
                .expect("rows");
            let at = rows.len() - 2;
            rows[at]["event"]["owner"]["source"]["line"]["run_ended"][field] = json!(99);
            assert!(!step_5_4(&changed).is_empty(), "{field}");
        }
    }

    #[test]
    fn every_counted_journal_must_be_complete_and_ordered() {
        let passing = met();
        for read in [
            Read::X3Ended,
            Read::X3Produced,
            Read::PlannedEnded,
            Read::StoryLanded,
            Read::StoryOwnerEnded,
            Read::StoryOwnerEarlier,
            Read::StoryOwnerFirst,
            Read::DirectionBEnded,
            Read::DirectionBFirst,
            Read::DirectionEnded,
            Read::DirectionFirst,
        ] {
            let mut missing = passing.clone();
            missing.documents.remove(&read);
            assert!(!step_5_4(&missing).is_empty(), "missing {read:?}");
            for mutation in 0..3 {
                let mut observed = passing.clone();
                let doc = observed.documents.get_mut(&read).expect("journal");
                let page = if doc.get("activity").is_some() { &mut doc["activity"] } else { doc };
                match mutation {
                    0 => page["cut"] = json!(true),
                    1 => page["rows"][0]["event"]["owner"]["source"]["task"] = json!(99),
                    2 => {
                        let duplicate = page["rows"][0].clone();
                        page["rows"].as_array_mut().expect("rows").push(duplicate);
                    }
                    _ => unreachable!(),
                }
                assert!(!step_5_4(&observed).is_empty(), "{read:?} mutation {mutation}");
            }
        }
    }

    #[test]
    fn fresh_message_numbers_senders_and_amendment_revision_are_checked() {
        let mut passing = met();
        for (read, paths) in [
            (
                Read::StoryOwnerFirst,
                vec![
                    "/rows/0/event/owner/source/line/heard/number",
                    "/rows/0/event/owner/source/line/heard/from/task",
                    "/rows/0/event/owner/source/line/heard/kind/amendment/revision",
                    "/rows/0/event/owner/source/line/heard/words",
                ],
            ),
            (
                Read::DirectionEnded,
                vec![
                    "/activity/rows/2/event/owner/source/line/words/message",
                    "/activity/rows/2/event/owner/source/line/words/by/person",
                    "/activity/rows/2/event/owner/source/line/words/words",
                ],
            ),
            (
                Read::DirectionBPending,
                vec![
                    "/activity/rows/0/event/owner/source/line/heard/number",
                    "/activity/rows/0/event/owner/source/line/heard/from/task",
                    "/activity/rows/0/event/owner/source/line/heard/hits",
                    "/activity/rows/0/event/owner/source/line/heard/kind/news/subscription",
                ],
            ),
        ] {
            for path in paths {
                let saved = passing.documents.get_mut(&read).expect("doc").pointer_mut(path).expect("field").take();
                assert!(!step_5_4(&passing).is_empty(), "{read:?}{path}");
                *passing.documents.get_mut(&read).expect("doc").pointer_mut(path).expect("field") = saved;
            }
        }
    }

    #[test]
    fn both_pull_heads_and_the_merge_commit_come_from_forge_records() {
        let passing = met();
        let mut multiline = passing.clone();
        let words = multiline
            .documents
            .get_mut(&Read::StoryLanded)
            .expect("landed task")
            .pointer_mut("/task/result/value/change/words/bytes")
            .expect("landing words");
        *words = format!("{}\npull request #1 of acme/app on forge.example", words.as_str().expect("merge id")).into();
        assert_met(&step_5_4(&multiline));
        for pull in 0..2 {
            for mutation in 0..5 {
                let mut observed = passing.clone();
                let pull = &mut observed.kit.pulls[pull];
                match mutation {
                    0 => pull.number = 99,
                    1 => pull.branch = b"unrelated".to_vec(),
                    2 => pull.pushed = None,
                    3 => pull.pushed = Some(99),
                    4 => pull.landed = Some(99),
                    _ => unreachable!(),
                }
                assert!(!step_5_4(&observed).is_empty());
            }
        }
        let mut observed = passing.clone();
        observed.documents.get_mut(&Read::X3Produced).expect("coder")["task"]["result"]["value"]["change"]["words"] =
            json!({"bytes":"arbitrary result prose"});
        assert_met(&step_5_4(&observed));
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
                records.push(Observation::Commented {
                    repository: repository.clone(),
                    number,
                    id: 1,
                    body: skein_forge::marker::encode(b"temper", b"reason-key", X3_CANCEL_REASON.as_bytes(), u32::MAX)
                        .expect("comment"),
                    by: 1,
                });
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
    fn the_closing_reason_must_be_exact_once_and_last_before_close() {
        use skein_fake_forge_domain::Observation;
        for mutation in 0..5 {
            let mut observed = met();
            let records = &mut observed.kit.records;
            match mutation {
                0 => {
                    records.remove(2);
                }
                1 => {
                    if let Observation::Commented { body, .. } = &mut records[2] {
                        *body = b"wrong reason".as_slice().into();
                    }
                }
                2 => {
                    records.insert(3, records[2].clone());
                }
                3 => {
                    records.swap(2, 3);
                }
                4 => {
                    let mut later = records[2].clone();
                    if let Observation::Commented { body, .. } = &mut later {
                        *body = b"a later comment".as_slice().into();
                    }
                    records.insert(3, later);
                }
                _ => unreachable!(),
            }
            assert!(!step_5_4(&observed).is_empty(), "comment mutation {mutation}");
        }
    }

    #[test]
    fn pull_digest_correlates_push_open_close_delete_and_merge_in_record_order() {
        let mut observed = met();
        observed.kit.pulls = pull_digest(&forge_records());
        assert_met(&step_5_4(&observed));
        assert_eq!(observed.kit.pulls[0].pushed, Some(7));
        assert_eq!(observed.kit.pulls[1].pushed, Some(8));
        assert_eq!(observed.kit.pulls[1].merged_head, Some(8));
        assert_eq!(observed.kit.pulls[1].landed, Some(9));
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
                    if let Observation::Closed { number, .. } = &mut records[3] {
                        *number = 99;
                    }
                }
                4 => {
                    if let Observation::Deleted { at, .. } = &mut records[4] {
                        *at = 99;
                    }
                }
                5 => {
                    if let Observation::Merged { head, .. } = &mut records[7] {
                        *head = 99;
                    }
                }
                6 => {
                    if let Observation::Merged { commit, .. } = &mut records[7] {
                        *commit = 99;
                    }
                }
                7 => {
                    records.remove(5);
                }
                8 => {
                    if let Observation::Opened { branches: Some(branches), .. } = &mut records[6] {
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

fn batch_writing(observed: &Observed, read: Read, task: u64, ended: Read) -> bool {
    let doc = task_document(observed, read);
    number(doc, "/chip/task") == Some(task)
        && text(doc, "/chip/phase") == "running"
        && text(doc, "/chip/details/owner/details/observation/work") == "writing"
        && number(doc, "/chip/details/owner/phase/active/running/attempt").is_some_and(|attempt| {
            check_activity(observed, ended).iter().any(|row| number(row, "/line/run_began/attempt") == Some(attempt))
        })
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

#[cfg(test)]
mod batch_tests {
    use super::*;
    use serde_json::json;
    use skein_fake_llm_domain::api::{Message, Part, Role};

    pub(super) fn input(words: &str) -> Message {
        Message { role: Role::User, parts: Box::new([Part::Text { text: words.as_bytes().into() }]) }
    }
    pub(super) fn tool(id: &str, name: &str, args: &Value, output: &str) -> [Message; 2] {
        [
            Message {
                role: Role::Assistant,
                parts: Box::new([Part::ToolCall {
                    id: id.as_bytes().into(),
                    name: name.as_bytes().into(),
                    arguments: serde_json::to_vec(args).expect("arguments").into(),
                }]),
            },
            Message {
                role: Role::User,
                parts: Box::new([Part::ToolOutput {
                    id: id.as_bytes().into(),
                    output: output.as_bytes().into(),
                    is_error: false,
                }]),
            },
        ]
    }
    pub(super) fn told(agent: Agent, history: &[Message]) -> Told {
        Told::with_agent(agent, agent.cue(), b"fake", None, history)
    }
    pub(super) fn journal(task: u64, lines: &[Value]) -> Value {
        let rows: Vec<_> = lines
            .iter()
            .enumerate()
            .rev()
            .map(|(at, line)| json!({"event":{"owner":{"source":{"task":task,"position":at+1,"line":line}}}}))
            .collect();
        json!({"rows":rows,"cut":false,"before":null,"after":null})
    }
}

fn blocker_delivery(observed: &Observed, agent: Agent, words: &str) -> Option<usize> {
    observed.told.iter().position(|t| t.agent == agent && user_words(t).any(|w| contains(w, words)))
}
// Act 3, step 6.5 (testing/reference-case.md)
const BUDGET_MESSAGES: Promise = Promise::Must("the budget question climbs two owners and its answers come down once");
const BUDGET_INBOX: Promise = Promise::Must("the person's inbox keeps the question until its answer");
const BUDGET_CHECKPOINT: Promise = Promise::Must("a5 is made after A1 hears the answer");
const BUDGET_EFFORT: Promise = Promise::Must("a5's effort is raised for its next try");
const BUDGET_ONESHOT: Promise = Promise::Must("one advisor reads a5's report on the owner's model");
const BUDGET_RESULT: Promise = Promise::Add("the advisor's report reaches A1 once");
const BUDGET_ATTEMPTS: Promise = Promise::Add("every attempt ended, none failed");

fn budget_tasks() -> [(Read, u64, u64); 5] {
    use super::{BUDGET_A5, BUDGET_ADVISOR, BUDGET_DIRECTOR, BUDGET_OWNER, BUDGET_SUBOWNER};
    [
        (Read::BudgetDirectorEnded, BUDGET_DIRECTOR, 0),
        (Read::BudgetOwnerEnded, BUDGET_OWNER, BUDGET_DIRECTOR),
        (Read::BudgetSubownerEnded, BUDGET_SUBOWNER, BUDGET_OWNER),
        (Read::BudgetA5Ended, BUDGET_A5, BUDGET_SUBOWNER),
        (Read::BudgetAdvisorEnded, BUDGET_ADVISOR, BUDGET_SUBOWNER),
    ]
}

fn budget_pages(read: Read) -> Vec<Read> {
    if read == Read::BudgetDirectorEnded {
        vec![read, Read::BudgetDirectorFirst]
    } else if read == Read::BudgetOwnerEnded {
        vec![read, Read::BudgetOwnerFirst]
    } else if read == Read::BudgetSubownerEnded {
        vec![read, Read::BudgetSubownerFirst]
    } else {
        vec![read]
    }
}
fn budget_rows(observed: &Observed, read: Read) -> Vec<&Value> {
    joined_activity(observed, &budget_pages(read))
}
fn budget_journal(observed: &Observed, read: Read, task: u64) -> bool {
    let pages = budget_pages(read);
    if pages.len() == 1 {
        return batch_journal(observed, read, task);
    }
    let rows = budget_rows(observed, read);
    // Apply the attempt rule across complete adjoining cursor pages.
    plan_attempt_rows(observed, &pages, task, 1, &rows)
        && rows.last().is_some_and(|r| number(r, "/position") == Some(1))
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
                    && number(end, "/attempt")
                        .is_some_and(|attempt| !rows.iter().any(|r| number(r, "/line/turn/attempt") == Some(attempt)))
                    && rows[..at].iter().any(|r| text(r, "/line/run_ended/end") == "finished"))
        })
}

#[must_use]
pub fn step_6_5(observed: &Observed) -> Vec<Miss> {
    use super::{A_DEADLINE, A1_DEADLINE, BUDGET_OWNER, PROJECT};
    let mut misses = Vec::new();
    for (read, task, parent) in budget_tasks() {
        let doc = task_document(observed, read);
        step_check(
            &mut misses,
            "6.5",
            number(doc, "/chip/task") == Some(task)
                && number(doc, "/project") == Some(PROJECT)
                && (parent == 0 || number(doc, "/requester/task") == Some(parent))
                && text(doc, "/chip/phase") == "ended"
                && doc.pointer("/result/ending/task/done").is_some_and(Value::is_object),
            BUDGET_CHECKPOINT,
            format_args!("{read:?} must identify done task {task} under {parent}"),
        );
        step_check(
            &mut misses,
            "6.5",
            budget_journal(observed, read, task),
            BUDGET_ATTEMPTS,
            format_args!(
                "task {task}'s whole ordered journal must end all attempts, allowing only unspent uncommitted rechecks"
            ),
        );
    }
    let a = task_document(observed, Read::BudgetOwnerEnded);
    let a1 = task_document(observed, Read::BudgetSubownerEnded);
    step_check(
        &mut misses,
        "6.5",
        number(a, "/goal") == Some(BUDGET_OWNER)
            && number(a, "/authority/authority/deadline") == Some(A_DEADLINE)
            && number(a1, "/authority/authority/deadline") == Some(A1_DEADLINE)
            && number(a1, "/authority/authority/delegation/depth") == Some(1),
        BUDGET_CHECKPOINT,
        "A must be tracked and A1 must carry M1's deadline and depth one",
    );
    budget_messages(observed, &mut misses);
    budget_person_answer(observed, &mut misses);
    budget_inbox(observed, &mut misses);
    budget_investigation(observed, &mut misses);
    budget_effort(observed, &mut misses);
    budget_advisor(observed, &mut misses);
    misses
}

fn budget_messages(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{
        BUDGET_ANSWER, BUDGET_ANSWER_NUMBER, BUDGET_DIRECTOR, BUDGET_OWNER, BUDGET_PERSON_ANSWER,
        BUDGET_PERSON_ANSWER_NUMBER, BUDGET_QUESTION, BUDGET_QUESTION_NUMBER, BUDGET_SUBOWNER, BUDGET_UP_QUESTION,
        BUDGET_UP_QUESTION_NUMBER,
    };
    let mut previous = None;
    for (agent, receiver, by, target, message, question, words, read) in [
        (
            Agent::BudgetSubowner,
            Agent::BudgetOwner,
            BUDGET_SUBOWNER,
            BUDGET_OWNER,
            BUDGET_QUESTION_NUMBER,
            None,
            BUDGET_QUESTION,
            Read::BudgetOwnerEnded,
        ),
        (
            Agent::BudgetOwner,
            Agent::BudgetDirector,
            BUDGET_OWNER,
            BUDGET_DIRECTOR,
            BUDGET_UP_QUESTION_NUMBER,
            None,
            BUDGET_UP_QUESTION,
            Read::BudgetDirectorEnded,
        ),
        (
            Agent::BudgetDirector,
            Agent::BudgetOwner,
            BUDGET_DIRECTOR,
            BUDGET_OWNER,
            BUDGET_PERSON_ANSWER_NUMBER,
            Some(BUDGET_UP_QUESTION_NUMBER),
            BUDGET_PERSON_ANSWER,
            Read::BudgetOwnerEnded,
        ),
        (
            Agent::BudgetOwner,
            Agent::BudgetSubowner,
            BUDGET_OWNER,
            BUDGET_SUBOWNER,
            BUDGET_ANSWER_NUMBER,
            Some(BUDGET_QUESTION_NUMBER),
            BUDGET_ANSWER,
            Read::BudgetSubownerEnded,
        ),
    ] {
        let delivery = blocker_delivery(observed, receiver, words);
        let ordered = delivery.is_some_and(|at| previous.is_none_or(|before| before < at));
        previous = delivery;
        let form = if question.is_some() { "answer" } else { "question" };
        let tools = plan_tool_results(observed, agent, b"message");
        let tools: Vec<_> = tools.iter().filter(|(_, args, _)| args["words"] == words).collect();
        let sent = (if agent == Agent::BudgetDirector { tools.is_empty() } else { tools.len() == 1 })
            && tools.iter().all(|(_, args, out)| {
                args["target"] == target
                    && args["form"] == form
                    && question.is_none_or(|q| args["question"] == q)
                    && contains(out, &format!("Message {message} sent"))
            });
        let recorded = budget_rows(observed, read)
            .iter()
            .filter(|row| {
                let kind = question
                    .map_or_else(|| Value::String(form.into()), |q| serde_json::json!({"answer":{"question":q}}));
                number(row, "/line/words/message") == Some(message)
                    && number(row, "/line/words/by/task") == Some(by)
                    && row.pointer("/line/words/kind") == Some(&kind)
                    && text(row, "/line/words/words") == words
            })
            .count()
            == 1;
        step_check(
            misses,
            "6.5",
            sent && ordered && recorded && plan_deliveries(observed, receiver, words) == 1,
            BUDGET_MESSAGES,
            format_args!("message {message} must reach {receiver:?} once in the question/answer chain"),
        );
    }
}

fn budget_person_answer(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{
        BUDGET_NOTICE_NUMBER, BUDGET_PERSON_ANSWER, BUDGET_PERSON_ANSWER_NUMBER, BUDGET_UP_QUESTION,
        BUDGET_UP_QUESTION_NUMBER,
    };
    let question = blocker_delivery(observed, Agent::BudgetDirector, BUDGET_UP_QUESTION);
    let owner = super::BUDGET_OWNER;
    let notice = format!(
        "Person answered task {owner}'s question {BUDGET_UP_QUESTION_NUMBER} in their inbox; task {owner} has the answer and does the work:\n{BUDGET_PERSON_ANSWER}"
    );
    let answer = blocker_delivery(observed, Agent::BudgetOwner, BUDGET_PERSON_ANSWER);
    step_check(
        misses,
        "6.5",
        question.zip(answer).is_some_and(|(question, answer)| question < answer)
            && plan_deliveries(observed, Agent::BudgetDirector, &notice) == 1
            && plan_deliveries(observed, Agent::BudgetDirector, BUDGET_PERSON_ANSWER) == 1
            && plan_deliveries(observed, Agent::BudgetOwner, BUDGET_PERSON_ANSWER) == 1
            && budget_rows(observed, Read::BudgetDirectorEnded)
                .iter()
                .filter(|r| {
                    number(r, "/line/heard/number") == Some(BUDGET_NOTICE_NUMBER)
                        && number(r, "/line/heard/from/task") == Some(super::BUDGET_OWNER)
                        && number(r, "/line/heard/kind/question_answered/question") == Some(BUDGET_UP_QUESTION_NUMBER)
                        && text(r, "/line/heard/words") == BUDGET_PERSON_ANSWER
                })
                .count()
                == 1
            && budget_rows(observed, Read::BudgetOwnerEnded)
                .iter()
                .filter(|r| {
                    number(r, "/line/words/message") == Some(BUDGET_PERSON_ANSWER_NUMBER)
                        && number(r, "/line/words/by/task") == Some(super::BUDGET_DIRECTOR)
                        && number(r, "/line/words/kind/answer/question") == Some(BUDGET_UP_QUESTION_NUMBER)
                        && text(r, "/line/words/words") == BUDGET_PERSON_ANSWER
                })
                .count()
                == 1,
        BUDGET_MESSAGES,
        "the person's answer reaches A once, as its requester's answer, with no chat relay",
    );
}

fn budget_inbox(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{BUDGET_DIRECTOR, BUDGET_UP_QUESTION_NUMBER, PROJECT};
    let rows = document(observed, Read::BudgetAsked).pointer("/needs/rows").and_then(Value::as_array);
    let ended = document(observed, Read::BudgetInboxAnswered).pointer("/needs/rows").and_then(Value::as_array);
    // The identified reads are the acts' JSON documents. Its command order
    // proves the first inbox read precedes answer and the final read follows.
    let commands: Vec<_> = observed.commands.iter().map(String::as_str).collect();
    step_check(
        misses,
        "6.5",
        rows.is_some_and(|rows| {
            rows.len() == 1
                && rows[0]["reply"].is_null()
                && number(&rows[0], "/task/live/task") == Some(BUDGET_DIRECTOR)
                && number(&rows[0], "/project") == Some(PROJECT)
                && number(&rows[0], "/object/question/message") == Some(BUDGET_UP_QUESTION_NUMBER)
        }) && document(observed, Read::BudgetAsked)["needs"]["cut"] == false
            && document(observed, Read::BudgetInboxAnswered)["needs"]["cut"] == false
            && ended.is_some_and(Vec::is_empty)
            && commands
                .iter()
                .position(|c| *c == "inbox")
                .zip(commands.iter().position(|c| *c == "task answer"))
                .is_some_and(|(read, answer)| read < answer)
            && commands
                .iter()
                .position(|c| *c == "task answer")
                .zip(commands.iter().rposition(|c| *c == "inbox"))
                .is_some_and(|(answer, read)| answer < read)
            && commands.iter().filter(|c| **c == "task answer").count() == 1,
        BUDGET_INBOX,
        "the read before ctl task answer must hold A's sole question, absent in the final inbox read",
    );
}

fn budget_investigation(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{BUDGET_A5, BUDGET_ANSWER, BUDGET_MEDIUM_NUMBER, DEFAULT_MODEL_NUMBER};
    let answer = blocker_delivery(observed, Agent::BudgetSubowner, BUDGET_ANSWER);
    let delegates = plan_tool_results(observed, Agent::BudgetSubowner, b"delegate");
    step_check(
        misses,
        "6.5",
        delegates.len() == 2
            && delegates.iter().filter(|(_, _, out)| contains(out, &format!("Delegated tasks: {BUDGET_A5}"))).count()
                == 1
            && delegates.iter().any(|(at, args, out)| {
                answer.is_some_and(|answer| answer < *at)
                && contains(out, &format!("Delegated tasks: {BUDGET_A5}"))
                && args["batch"].as_array().is_some_and(|batch| {
                    batch.len() == 1
                        && batch[0]["llm"]
                            == serde_json::json!({"main":{"model":DEFAULT_MODEL_NUMBER,"effort":BUDGET_MEDIUM_NUMBER}})
                })
            }),
        BUDGET_CHECKPOINT,
        "a5's singleton delegation on the default model at medium must follow A1 hearing the answer",
    );
}

fn budget_finding(observed: &Observed) -> bool {
    use super::{BUDGET_A5, BUDGET_FINDING, BUDGET_FINDING_NUMBER};
    plan_deliveries(observed, Agent::BudgetSubowner, BUDGET_FINDING) == 1
        && budget_rows(observed, Read::BudgetSubownerEnded)
            .iter()
            .filter(|r| {
                number(r, "/line/words/message") == Some(BUDGET_FINDING_NUMBER)
                    && number(r, "/line/words/by/task") == Some(BUDGET_A5)
                    && text(r, "/line/words/kind") == "words"
                    && text(r, "/line/words/words") == BUDGET_FINDING
            })
            .count()
            == 1
}

fn budget_effort(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{
        BUDGET_A5, BUDGET_A5_REPORT, BUDGET_AMENDMENT_NUMBER, BUDGET_HIGH, BUDGET_HIGH_NUMBER, BUDGET_MEDIUM,
        BUDGET_REASON, BUDGET_REVISION, BUDGET_SUBOWNER, DEFAULT_MODEL, DEFAULT_MODEL_NUMBER,
    };
    let medium = task_document(observed, Read::BudgetMedium);
    let high = task_document(observed, Read::BudgetHigh);
    let ended = task_document(observed, Read::BudgetA5Ended);
    let amendments = plan_tool_results(observed, Agent::BudgetSubowner, b"amend");
    let choices = [
        (medium, BUDGET_MEDIUM, BUDGET_REVISION - 1),
        (high, BUDGET_HIGH, BUDGET_REVISION),
        (ended, BUDGET_HIGH, BUDGET_REVISION),
    ];
    step_check(
        misses,
        "6.5",
        budget_finding(observed)
            && choices.iter().all(|(doc, effort, revision)| {
                number(doc, "/chip/task") == Some(BUDGET_A5)
                    && text(doc, "/llm/main/model") == DEFAULT_MODEL
                    && text(doc, "/llm/main/effort") == *effort
                    && number(doc, "/spec_revision/value") == Some(*revision)
            })
            && text(medium, "/chip/phase") == "idle"
            && medium["result"].is_null()
            && amendments.len() == 1
            && amendments.iter().all(|(_, args, out)| {
                args["target"] == BUDGET_A5
                    && args["revision"] == BUDGET_REVISION - 1
                    && args["amendment"]["llm"]
                        == serde_json::json!({"main":{"model":DEFAULT_MODEL_NUMBER,"effort":BUDGET_HIGH_NUMBER}})
                    && args["amendment"]["reason"] == BUDGET_REASON
                    && contains(out, &format!("Done: task {BUDGET_A5} is at spec revision {BUDGET_REVISION}"))
            }),
        BUDGET_EFFORT,
        "a5 must change from medium to high at the amendment's next spec revision",
    );
    let before = check_activity(observed, Read::BudgetMedium);
    let after = check_activity(observed, Read::BudgetHigh);
    let reason: Vec<_> =
        after.iter().filter(|r| number(r, "/line/heard/number") == Some(BUDGET_AMENDMENT_NUMBER)).collect();
    let running = number(high, "/chip/details/owner/phase/active/running/attempt");
    step_check(
        misses,
        "6.5",
        batch_journal(observed, Read::BudgetMedium, BUDGET_A5)
            && batch_writing(observed, Read::BudgetHigh, BUDGET_A5, Read::BudgetA5Ended)
            && before.iter().all(|row| after.contains(row))
            && pause_activity(observed, Read::BudgetHigh)["cut"] == false
            && pause_activity(observed, Read::BudgetHigh)["before"].is_null()
            && after.iter().all(|r| number(r, "/task") == Some(BUDGET_A5))
            && after.windows(2).all(|pair| {
                number(pair[0], "/position")
                    .zip(number(pair[1], "/position"))
                    .is_some_and(|(new, old)| new.checked_sub(1) == Some(old))
            })
            && after.iter().filter(|r| number(r, "/line/decided/revision") == Some(BUDGET_REVISION)).count() == 1
            && reason.len() == 1
            && reason.iter().all(|r| {
                number(r, "/line/heard/from/task") == Some(BUDGET_SUBOWNER)
                    && number(r, "/line/heard/kind/amendment/revision") == Some(BUDGET_REVISION)
                    && text(r, "/line/heard/words") == BUDGET_REASON
                    && number(r, "/position").is_some_and(|amended| {
                        before.first().and_then(|r| number(r, "/position")).is_some_and(|read| read < amended)
                            && after.iter().any(|r| {
                                number(r, "/line/run_began/attempt") == running
                                    && running.is_some()
                                    && number(r, "/position").is_some_and(|began| {
                                        began > amended
                                            && after.iter().any(|decision| {
                                                number(decision, "/line/decided/revision") == Some(BUDGET_REVISION)
                                                    && number(decision, "/position")
                                                        .is_some_and(|decided| amended < decided && decided < began)
                                            })
                                            && !after.iter().any(|between| {
                                                between.pointer("/line/run_began").is_some()
                                                    && number(between, "/position")
                                                        .is_some_and(|at| amended < at && at < began)
                                            })
                                    })
                            })
                    })
            })
            && plan_deliveries(observed, Agent::BudgetInvestigator, BUDGET_REASON) == 1
            && text(ended, "/result/value/report/bytes") == BUDGET_A5_REPORT,
        BUDGET_EFFORT,
        "the next a5 run after the amendment must hear its reason once and report none again",
    );
}

fn budget_report(observed: &Observed, task: u64, message: u64, words: &str) -> bool {
    budget_rows(observed, Read::BudgetSubownerEnded)
        .iter()
        .filter(|r| {
            number(r, "/line/heard/number") == Some(message)
                && number(r, "/line/heard/from/task") == Some(task)
                && text(r, "/line/heard/kind/result") == "report"
                && text(r, "/line/heard/words") == words
        })
        .count()
        == 1
}

fn budget_advisor(observed: &Observed, misses: &mut Vec<Miss>) {
    use super::{
        BUDGET_A5, BUDGET_A5_REPORT, BUDGET_A5_REPORT_NUMBER, BUDGET_ADVISOR, BUDGET_ADVISOR_REPORT,
        BUDGET_ADVISOR_REPORT_NUMBER, BUDGET_ADVISOR_SPEC, OWNER_EFFORT_NUMBER, OWNER_MODEL, OWNER_MODEL_NUMBER,
    };
    let report_at = blocker_delivery(observed, Agent::BudgetSubowner, BUDGET_A5_REPORT);
    let delegated = plan_tool_results(observed, Agent::BudgetSubowner, b"delegate");
    let advisor = task_document(observed, Read::BudgetAdvisorEnded);
    let queries: Vec<_> = observed.told.iter().filter(|t| t.agent == Agent::BudgetAdvisor).collect();
    let rows = check_activity(observed, Read::BudgetAdvisorEnded);
    step_check(
        misses,
        "6.5",
        budget_report(observed, BUDGET_A5, BUDGET_A5_REPORT_NUMBER, BUDGET_A5_REPORT)
            && plan_deliveries(observed, Agent::BudgetSubowner, BUDGET_A5_REPORT) == 1
            && delegated
                .iter()
                .filter(|(_, _, out)| contains(out, &format!("Delegated tasks: {BUDGET_ADVISOR}")))
                .count()
                == 1
            && delegated.iter().any(|(at, args, out)| {
                report_at.is_some_and(|report| report < *at)
                    && contains(out, &format!("Delegated tasks: {BUDGET_ADVISOR}"))
                    && args["batch"].as_array().is_some_and(|batch| {
                        batch.len() == 1
                            && batch[0]["spec"]["words"] == BUDGET_ADVISOR_SPEC
                            && batch[0]["llm"]
                                == serde_json::json!({"main":{"model":OWNER_MODEL_NUMBER,"effort":OWNER_EFFORT_NUMBER}})
                    })
            })
            && text(advisor, "/spec/specification/words/bytes") == BUDGET_ADVISOR_SPEC
            && text(advisor, "/llm/main/model") == OWNER_MODEL
            && text(advisor, "/result/value/report/bytes") == BUDGET_ADVISOR_REPORT
            && queries.len() == 1
            && queries.iter().all(|t| {
                t.model.as_ref() == OWNER_MODEL.as_bytes() && contains(&t.system, BUDGET_A5_REPORT) && plan_fresh(t)
            })
            && rows.iter().filter(|r| r.pointer("/line/run_began").is_some()).count() == 1
            && rows.iter().filter(|r| text(r, "/line/run_ended/end") == "finished").count() == 1
            && rows.iter().filter(|r| r.pointer("/line/turn").is_some()).count() == 1,
        BUDGET_ONESHOT,
        "after a5 reports, one advisor must read its report/evidence on owner and end on its first turn and attempt",
    );
    step_check(
        misses,
        "6.5",
        budget_report(observed, BUDGET_ADVISOR, BUDGET_ADVISOR_REPORT_NUMBER, BUDGET_ADVISOR_REPORT)
            && plan_deliveries(observed, Agent::BudgetSubowner, BUDGET_ADVISOR_REPORT) == 1
            && report_at
                .zip(blocker_delivery(observed, Agent::BudgetSubowner, BUDGET_ADVISOR_REPORT))
                .is_some_and(|(a5, advisor)| a5 < advisor),
        BUDGET_RESULT,
        "A1 must hear the advisor's cause once, after a5's report",
    );
}

#[cfg(test)]
mod budget_tests {
    use super::batch_tests::{input, journal, told, tool};
    use super::*;
    use crate::case::*;
    use serde_json::json;
    use skein_fake_llm_domain::api::{Part, Role};

    fn met() -> Observed {
        let mut observed = Observed {
            activity_aliases: std::collections::BTreeSet::new(),
            observed_at: BTreeMap::new(),
            documents: BTreeMap::new(),
            told: Vec::new(),
            commands: vec![
                "sign-in",
                "chat start",
                "task show",
                "task show",
                "task show",
                "inbox",
                "task answer",
                "task show",
                "task show",
                "task show",
                "task show",
                "task show",
                "task show",
                "task show",
                "task show",
                "task show",
                "task show",
                "inbox",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            kit: Kit { records: Vec::new(), main: BTreeMap::new(), pulls: Vec::new(), merged_green: false },
        };
        budget_documents(&mut observed);
        investigation_documents(&mut observed);
        budget_queries(&mut observed);
        for (latest, earlier, _) in [
            (Read::BudgetDirectorEnded, Read::BudgetDirectorFirst, BUDGET_DIRECTOR),
            (Read::BudgetOwnerEnded, Read::BudgetOwnerFirst, BUDGET_OWNER),
            (Read::BudgetSubownerEnded, Read::BudgetSubownerFirst, BUDGET_SUBOWNER),
        ] {
            let page = &mut observed.documents.get_mut(&latest).expect("latest")["activity"];
            let rows = page["rows"].as_array_mut().expect("rows");
            let older = rows.split_off(3);
            page["before"] = json!({"value":"older"});
            observed
                .documents
                .insert(earlier, json!({"rows":older,"cut":false,"before":null,"after":{"value":"newer"}}));
        }
        observed
    }

    fn budget_documents(observed: &mut Observed) {
        for (read, task, parent) in budget_tasks() {
            observed.documents.insert(read,json!({"task":{
                "chip":{"task":task,"phase":"ended"},"project":PROJECT,"requester":{"task":parent},"goal":BUDGET_OWNER,
                "authority":{"authority":{"deadline":if task == BUDGET_OWNER {A_DEADLINE} else {A1_DEADLINE},"delegation":{"depth":1}}},
                "result":{"ending":{"task":{"done":{}}}}
            },"activity":journal(task,&[
                json!({"run_began":{"attempt":1}}), json!({"turn":{"attempt":1}}),
                json!({"run_ended":{"attempt":1,"end":"finished","spent":1}})
            ])}));
        }
        let mut lines = BTreeMap::new();
        for (read, task, _) in budget_tasks() {
            lines.insert(read, (task, vec![json!({"run_began":{"attempt":1}}), json!({"turn":{"attempt":1}})]));
        }
        for (read, message, by, kind, words) in [
            (
                Read::BudgetOwnerEnded,
                BUDGET_QUESTION_NUMBER,
                json!({"task":BUDGET_SUBOWNER}),
                json!("question"),
                BUDGET_QUESTION,
            ),
            (
                Read::BudgetDirectorEnded,
                BUDGET_UP_QUESTION_NUMBER,
                json!({"task":BUDGET_OWNER}),
                json!("question"),
                BUDGET_UP_QUESTION,
            ),
            (
                Read::BudgetOwnerEnded,
                BUDGET_PERSON_ANSWER_NUMBER,
                json!({"task":BUDGET_DIRECTOR}),
                json!({"answer":{"question":BUDGET_UP_QUESTION_NUMBER}}),
                BUDGET_PERSON_ANSWER,
            ),
            (
                Read::BudgetSubownerEnded,
                BUDGET_ANSWER_NUMBER,
                json!({"task":BUDGET_OWNER}),
                json!({"answer":{"question":BUDGET_QUESTION_NUMBER}}),
                BUDGET_ANSWER,
            ),
        ] {
            lines
                .get_mut(&read)
                .expect("task")
                .1
                .push(json!({"words":{"message":message,"by":by,"kind":kind,"words":words}}));
        }
        lines.get_mut(&Read::BudgetDirectorEnded).expect("director").1.push(json!({"heard":{"number":BUDGET_NOTICE_NUMBER,"from":{"task":BUDGET_OWNER},"kind":{"question_answered":{"question":BUDGET_UP_QUESTION_NUMBER}},"words":BUDGET_PERSON_ANSWER,"eligible":true}}));
        lines.get_mut(&Read::BudgetSubownerEnded).expect("A1").1.push(json!({"words":{"message":BUDGET_FINDING_NUMBER,"by":{"task":BUDGET_A5},"kind":"words","words":BUDGET_FINDING}}));
        for (task, message, words) in [
            (BUDGET_A5, BUDGET_A5_REPORT_NUMBER, BUDGET_A5_REPORT),
            (BUDGET_ADVISOR, BUDGET_ADVISOR_REPORT_NUMBER, BUDGET_ADVISOR_REPORT),
        ] {
            lines.get_mut(&Read::BudgetSubownerEnded).expect("A1").1.push(
                json!({"heard":{"number":message,"from":{"task":task},"kind":{"result":"report"},"words":words}}),
            );
        }
        for (read, (task, mut rows)) in lines {
            rows.push(json!({"run_ended":{"attempt":1,"end":"finished","spent":1}}));
            observed.documents.get_mut(&read).expect("doc")["activity"] = journal(task, &rows);
        }
        observed.documents.insert(Read::BudgetAsked,json!({"needs":{"cut":false,"rows":[{"task":{"live":{"task":BUDGET_DIRECTOR}},"project":PROJECT,"object":{"question":{"message":BUDGET_UP_QUESTION_NUMBER}},"reply":null}]}}));
        observed.documents.insert(Read::BudgetInboxAnswered, json!({"needs":{"cut":false,"rows":[]}}));
    }

    fn investigation_documents(observed: &mut Observed) {
        let before = vec![
            json!({"run_began":{"attempt":1}}),
            json!({"turn":{"attempt":1}}),
            json!({"run_ended":{"attempt":1,"end":"parked","spent":1}}),
        ];
        let mut high = before.clone();
        high.extend([
            json!({"heard":{"number":BUDGET_AMENDMENT_NUMBER,"from":{"task":BUDGET_SUBOWNER},"kind":{"amendment":{"revision":BUDGET_REVISION}},"words":BUDGET_REASON}}),
            json!({"decided":{"revision":BUDGET_REVISION}}),
            json!({"run_began":{"attempt":2,"why":{"message":{"message":BUDGET_AMENDMENT_NUMBER}}}}),
        ]);
        for (read, effort, revision, phase, rows) in [
            (Read::BudgetMedium, BUDGET_MEDIUM, BUDGET_REVISION - 1, "idle", &before),
            (Read::BudgetHigh, BUDGET_HIGH, BUDGET_REVISION, "running", &high),
        ] {
            observed.documents.insert(read,json!({"task":{"chip":{"task":BUDGET_A5,"phase":phase,"details":{"owner":{"phase":{"active":{"running":{"attempt":2}}},"details":{"observation":{"work":"writing"}}}}},"llm":{"main":{"model":DEFAULT_MODEL,"effort":effort}},"spec_revision":{"value":revision},"result":null},"activity":journal(BUDGET_A5,rows)}));
        }
        high.extend([
            json!({"turn":{"attempt":2,"read":BUDGET_AMENDMENT_NUMBER}}),
            json!({"run_ended":{"attempt":2,"end":"finished","spent":1}}),
        ]);
        let a5 = observed.documents.get_mut(&Read::BudgetA5Ended).expect("a5");
        a5["activity"] = journal(BUDGET_A5, &high);
        a5["task"]["llm"] = json!({"main":{"model":DEFAULT_MODEL,"effort":BUDGET_HIGH}});
        a5["task"]["spec_revision"] = json!({"value":BUDGET_REVISION});
        a5["task"]["result"]["value"] = json!({"report":{"bytes":BUDGET_A5_REPORT}});
        let advisor = observed.documents.get_mut(&Read::BudgetAdvisorEnded).expect("advisor");
        advisor["task"]["spec"] = json!({"specification":{"words":{"bytes":BUDGET_ADVISOR_SPEC}}});
        advisor["task"]["llm"] = json!({"main":{"model":OWNER_MODEL,"effort":OWNER_EFFORT}});
        advisor["task"]["result"]["value"] = json!({"report":{"bytes":BUDGET_ADVISOR_REPORT}});
    }

    fn budget_queries(observed: &mut Observed) {
        let exchange = |id, name, args: Value, output: String| tool(id, name, &args, &output).to_vec();
        observed.told = vec![
            told(
                Agent::BudgetSubowner,
                &exchange(
                    "q",
                    "message",
                    json!({"target":BUDGET_OWNER,"form":"question","words":BUDGET_QUESTION}),
                    format!("Message {BUDGET_QUESTION_NUMBER} sent"),
                ),
            ),
            told(Agent::BudgetOwner, &[input(BUDGET_QUESTION)]),
            told(
                Agent::BudgetOwner,
                &exchange(
                    "up",
                    "message",
                    json!({"target":BUDGET_DIRECTOR,"form":"question","words":BUDGET_UP_QUESTION}),
                    format!("Message {BUDGET_UP_QUESTION_NUMBER} sent"),
                ),
            ),
            told(Agent::BudgetDirector, &[input(BUDGET_UP_QUESTION)]),
            told(Agent::BudgetOwner, &[input(BUDGET_PERSON_ANSWER)]),
            told(
                Agent::BudgetOwner,
                &exchange(
                    "answer",
                    "message",
                    json!({"target":BUDGET_SUBOWNER,"form":"answer","question":BUDGET_QUESTION_NUMBER,"words":BUDGET_ANSWER}),
                    format!("Message {BUDGET_ANSWER_NUMBER} sent"),
                ),
            ),
            told(Agent::BudgetSubowner, &[input(BUDGET_ANSWER)]),
            told(
                Agent::BudgetSubowner,
                &exchange(
                    "a5",
                    "delegate",
                    json!({"batch":[{"llm":{"main":{"model":DEFAULT_MODEL_NUMBER,"effort":BUDGET_MEDIUM_NUMBER}}}]}),
                    format!("Delegated tasks: {BUDGET_A5}"),
                ),
            ),
            told(
                Agent::BudgetSubowner,
                &exchange(
                    "raise",
                    "amend",
                    json!({"target":BUDGET_A5,"revision":BUDGET_REVISION-1,"amendment":{"llm":{"main":{"model":DEFAULT_MODEL_NUMBER,"effort":BUDGET_HIGH_NUMBER}},"reason":BUDGET_REASON}}),
                    format!("Done: task {BUDGET_A5} is at spec revision {BUDGET_REVISION}"),
                ),
            ),
            told(Agent::BudgetInvestigator, &[input(BUDGET_REASON)]),
            told(Agent::BudgetSubowner, &[input(BUDGET_A5_REPORT)]),
            told(
                Agent::BudgetSubowner,
                &exchange(
                    "advisor",
                    "delegate",
                    json!({"batch":[{"spec":{"words":BUDGET_ADVISOR_SPEC},"llm":{"main":{"model":OWNER_MODEL_NUMBER,"effort":OWNER_EFFORT_NUMBER}}}]}),
                    format!("Delegated tasks: {BUDGET_ADVISOR}"),
                ),
            ),
            Told::new(BUDGET_ADVISOR_SPEC.as_bytes(), OWNER_MODEL.as_bytes(), None, &[input("Begin")])
                .expect("advisor"),
            told(Agent::BudgetSubowner, &[input(BUDGET_ADVISOR_REPORT)]),
        ];
        let mut finding = vec![input(BUDGET_FINDING)];
        finding.extend(observed.told[8].history.iter().cloned());
        observed.told[8].history = finding.into();
        observed.told.insert(
            4,
            told(
                Agent::BudgetDirector,
                &[input(&format!("Person answered task 2's question 2 in their inbox; task 2 has the answer and does the work:\n{BUDGET_PERSON_ANSWER}"))],
            ),
        );
    }

    fn rejects(observed: &Observed, promise: Promise) {
        let misses = step_6_5(observed);
        assert!(
            misses.iter().any(|m| m.step == "6.5" && m.promise == promise),
            "expected {promise:?}; got {}",
            misses.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n")
        );
    }
    fn changed_tool(observed: &mut Observed, at: usize, path: &str) {
        let Part::ToolCall { arguments, .. } = observed.told[at]
            .history
            .iter_mut()
            .flat_map(|m| &mut m.parts)
            .find(|p| matches!(p, Part::ToolCall { .. }))
            .expect("tool")
        else {
            unreachable!()
        };
        let mut args: Value = serde_json::from_slice(arguments).expect("args");
        *args.pointer_mut(path).expect("field") = Value::Null;
        *arguments = serde_json::to_vec(&args).expect("args").into();
    }

    #[test]
    fn budget_checkpoint_met() {
        assert_met(&step_6_5(&met()));
    }

    #[test]
    fn each_task_identity_parent_project_and_ending_is_required() {
        let mut passing = met();
        for (read, _, parent) in budget_tasks() {
            let mut paths =
                vec!["/task/chip/task", "/task/project", "/task/chip/phase", "/task/result/ending/task/done"];
            if parent != 0 {
                paths.push("/task/requester/task");
            }
            for path in paths {
                let saved = passing.documents.get_mut(&read).expect("task").pointer_mut(path).expect("field").take();
                rejects(&passing, BUDGET_CHECKPOINT);
                *passing.documents.get_mut(&read).expect("task").pointer_mut(path).expect("field") = saved;
            }
        }
        for (read, path) in [
            (Read::BudgetOwnerEnded, "/task/goal"),
            (Read::BudgetOwnerEnded, "/task/authority/authority/deadline"),
            (Read::BudgetSubownerEnded, "/task/authority/authority/deadline"),
            (Read::BudgetSubownerEnded, "/task/authority/authority/delegation/depth"),
        ] {
            let saved = passing.documents.get_mut(&read).expect("doc").pointer_mut(path).expect("field").take();
            rejects(&passing, BUDGET_CHECKPOINT);
            *passing.documents.get_mut(&read).expect("doc").pointer_mut(path).expect("field") = saved;
        }
    }

    #[test]
    fn every_chain_message_must_be_delivered_once_and_in_order() {
        let passing = met();
        for at in [1, 3, 4, 5, 7] {
            let mut missing = passing.clone();
            missing.told.remove(at);
            rejects(&missing, BUDGET_MESSAGES);
            let mut duplicated = passing.clone();
            let more = duplicated.told[at].history[0].clone();
            let mut history = duplicated.told[at].history.to_vec();
            history.push(more);
            duplicated.told[at].history = history.into();
            rejects(&duplicated, BUDGET_MESSAGES);
        }
        for (left, right) in [(1, 3), (3, 5), (5, 7)] {
            let mut observed = passing.clone();
            observed.told.swap(left, right);
            rejects(&observed, BUDGET_MESSAGES);
        }
        for (at, paths) in [
            (0, vec!["/target", "/form", "/words"]),
            (2, vec!["/target", "/form", "/words"]),
            (6, vec!["/target", "/form", "/question", "/words"]),
        ] {
            for path in paths {
                let mut observed = passing.clone();
                changed_tool(&mut observed, at, path);
                rejects(&observed, BUDGET_MESSAGES);
            }
            let mut observed = passing.clone();
            observed.told[at].history = observed.told[at].history[..1].into();
            rejects(&observed, BUDGET_MESSAGES);
        }
        for read in [Read::BudgetOwnerEnded, Read::BudgetDirectorEnded, Read::BudgetSubownerEnded] {
            let rows = budget_rows(&passing.clone(), read)
                .into_iter()
                .filter(|r| number(r, "/line/words/message").is_some_and(|message| message <= BUDGET_ANSWER_NUMBER))
                .cloned()
                .collect::<Vec<_>>();
            for row in rows {
                for field in ["message", "by", "kind", "words"] {
                    let mut observed = passing.clone();
                    for page in budget_pages(read) {
                        let doc = observed.documents.get_mut(&page).expect("doc");
                        let page = if doc.get("activity").is_some() { &mut doc["activity"] } else { doc };
                        if let Some(changed) = page["rows"]
                            .as_array_mut()
                            .expect("rows")
                            .iter_mut()
                            .find(|r| r["event"]["owner"]["source"] == row)
                        {
                            changed["event"]["owner"]["source"]["line"]["words"][field] = Value::Null;
                        }
                    }
                    rejects(&observed, BUDGET_MESSAGES);
                }
            }
        }
    }

    #[test]
    fn inbox_question_must_survive_until_the_read_before_the_answer() {
        let passing = met();
        for path in ["/needs/rows/0/task/live/task", "/needs/rows/0/project", "/needs/rows/0/object/question/message"] {
            let mut observed = passing.clone();
            *observed.documents.get_mut(&Read::BudgetAsked).expect("inbox").pointer_mut(path).expect("field") =
                Value::Null;
            rejects(&observed, BUDGET_INBOX);
        }
        for mutation in 0..7 {
            let mut observed = passing.clone();
            match mutation {
                0 => observed.documents.get_mut(&Read::BudgetAsked).expect("inbox")["needs"]["rows"] = json!([]),
                1 => {
                    observed.documents.get_mut(&Read::BudgetInboxAnswered).expect("inbox")["needs"]["rows"] =
                        json!([{}]);
                }
                2 => observed.commands.swap(5, 6),
                3 => observed.commands.push("task answer".into()),
                4 => observed.documents.get_mut(&Read::BudgetAsked).expect("inbox")["needs"]["cut"] = json!(true),
                5 => {
                    observed.documents.get_mut(&Read::BudgetInboxAnswered).expect("inbox")["needs"]["cut"] =
                        json!(true);
                }
                6 => {
                    observed.documents.get_mut(&Read::BudgetAsked).expect("inbox")["needs"]["rows"][0]["reply"] =
                        json!({"answered":true});
                }
                _ => unreachable!(),
            }
            rejects(&observed, BUDGET_INBOX);
        }
    }

    #[test]
    fn a5_cannot_be_made_before_a1s_answer_or_on_another_choice() {
        let passing = met();
        let mut observed = passing.clone();
        observed.told.swap(7, 8);
        rejects(&observed, BUDGET_CHECKPOINT);
        for path in ["/batch/0/llm/main/model", "/batch/0/llm/main/effort"] {
            let mut observed = passing.clone();
            changed_tool(&mut observed, 8, path);
            rejects(&observed, BUDGET_CHECKPOINT);
        }
        let mut observed = passing.clone();
        observed.told.remove(8);
        rejects(&observed, BUDGET_CHECKPOINT);
    }

    #[test]
    fn medium_high_and_the_successful_amendment_revision_are_required() {
        let mut passing = met();
        for read in [Read::BudgetMedium, Read::BudgetHigh, Read::BudgetA5Ended] {
            for path in
                ["/task/chip/task", "/task/llm/main/model", "/task/llm/main/effort", "/task/spec_revision/value"]
            {
                let saved = passing.documents.get_mut(&read).expect("a5").pointer_mut(path).expect("field").take();
                rejects(&passing, BUDGET_EFFORT);
                *passing.documents.get_mut(&read).expect("a5").pointer_mut(path).expect("field") = saved;
            }
        }
        for path in
            ["/target", "/revision", "/amendment/llm/main/model", "/amendment/llm/main/effort", "/amendment/reason"]
        {
            let mut observed = passing.clone();
            changed_tool(&mut observed, 9, path);
            rejects(&observed, BUDGET_EFFORT);
        }
        let mut observed = passing.clone();
        observed.told.remove(9);
        rejects(&observed, BUDGET_EFFORT);
    }

    #[test]
    fn the_next_run_must_follow_the_amendment_and_hear_the_reason_once() {
        let mut passing = met();
        for path in [
            "/task/chip/phase",
            "/activity/rows/1/event/owner/source/line/decided/revision",
            "/task/chip/details/owner/phase/active/running/attempt",
            "/task/chip/details/owner/details/observation/work",
            "/activity/rows/2/event/owner/source/line/heard/number",
            "/activity/rows/2/event/owner/source/line/heard/from/task",
            "/activity/rows/2/event/owner/source/line/heard/kind/amendment/revision",
            "/activity/rows/2/event/owner/source/line/heard/words",
        ] {
            let saved =
                passing.documents.get_mut(&Read::BudgetHigh).expect("a5").pointer_mut(path).expect("field").take();
            rejects(&passing, BUDGET_EFFORT);
            *passing.documents.get_mut(&Read::BudgetHigh).expect("a5").pointer_mut(path).expect("field") = saved;
        }
        let mut observed = passing.clone();
        observed.told.remove(10);
        rejects(&observed, BUDGET_EFFORT);
        let mut observed = passing.clone();
        observed.told[10].history = vec![input(BUDGET_REASON), input(BUDGET_REASON)].into();
        rejects(&observed, BUDGET_EFFORT);
        let mut observed = passing.clone();
        observed.documents.get_mut(&Read::BudgetHigh).expect("a5")["activity"]["rows"]
            .as_array_mut()
            .expect("rows")
            .swap(0, 1);
        rejects(&observed, BUDGET_EFFORT);
    }

    #[test]
    fn advisor_requires_the_ended_investigations_evidence_owner_model_and_one_attempt() {
        let passing = met();
        for path in ["/batch/0/spec/words", "/batch/0/llm/main/model", "/batch/0/llm/main/effort"] {
            let mut observed = passing.clone();
            changed_tool(&mut observed, 12, path);
            rejects(&observed, BUDGET_ONESHOT);
        }
        for path in ["/task/spec/specification/words/bytes", "/task/llm/main/model", "/task/result/value/report/bytes"]
        {
            let mut observed = passing.clone();
            *observed
                .documents
                .get_mut(&Read::BudgetAdvisorEnded)
                .expect("advisor")
                .pointer_mut(path)
                .expect("field") = Value::Null;
            rejects(&observed, BUDGET_ONESHOT);
        }
        for mutation in 0..6 {
            let mut observed = passing.clone();
            match mutation {
                0 => observed.told.swap(11, 12),
                1 => observed.told[13].model = DEFAULT_MODEL.as_bytes().into(),
                2 => observed.told[13].system = Agent::BudgetAdvisor.cue().into(),
                3 => {
                    let extra = observed.told[13].clone();
                    observed.told.push(extra);
                }
                4 => observed.told[13].history[0].role = Role::Assistant,
                5 => {
                    observed.documents.get_mut(&Read::BudgetAdvisorEnded).expect("advisor")["activity"] = journal(
                        BUDGET_ADVISOR,
                        &[
                            json!({"run_began":{"attempt":1}}),
                            json!({"turn":{"attempt":1}}),
                            json!({"run_ended":{"attempt":1,"end":"parked"}}),
                            json!({"run_began":{"attempt":2}}),
                            json!({"turn":{"attempt":2}}),
                            json!({"run_ended":{"attempt":2,"end":"finished"}}),
                        ],
                    );
                }
                _ => unreachable!(),
            }
            rejects(&observed, BUDGET_ONESHOT);
        }
    }

    #[test]
    fn advisor_report_must_be_delivered_once_after_the_investigations_report() {
        let passing = met();
        let mut observed = passing.clone();
        observed.told.remove(14);
        rejects(&observed, BUDGET_RESULT);
        let mut observed = passing.clone();
        observed.told[14].history = vec![input(BUDGET_ADVISOR_REPORT), input(BUDGET_ADVISOR_REPORT)].into();
        rejects(&observed, BUDGET_RESULT);
        let mut observed = passing.clone();
        observed.told.swap(11, 14);
        rejects(&observed, BUDGET_RESULT);
    }

    #[test]
    fn no_failed_unended_or_spent_refused_attempt_or_incomplete_journal_is_hidden() {
        let passing = met();
        for (read, task, _) in budget_tasks() {
            for mutation in 0..7 {
                let mut observed = passing.clone();
                let page = &mut observed.documents.get_mut(&read).expect("doc")["activity"];
                match mutation {
                    0 => {
                        page["rows"][0]["event"]["owner"]["source"]["line"]["run_ended"]["end"] =
                            json!({"failed":{"class":"permanent"}});
                    }
                    1 => page["rows"][0]["event"]["owner"]["source"]["line"]["run_ended"]["end"] = json!("refused"),
                    2 => {
                        page["rows"].as_array_mut().expect("rows").remove(0);
                    }
                    3 => page["cut"] = json!(true),
                    4 => page["after"] = json!(1),
                    5 => page["rows"][0]["event"]["owner"]["source"]["task"] = json!(task + 99),
                    6 => {
                        let duplicate = page["rows"][0].clone();
                        page["rows"].as_array_mut().expect("rows").push(duplicate);
                    }
                    _ => unreachable!(),
                }
                rejects(&observed, BUDGET_ATTEMPTS);
            }
        }
    }

    #[test]
    fn every_identified_read_is_required() {
        let passing = met();
        for (read, _) in crate::case::acts::READS_6_5 {
            let mut observed = passing.clone();
            observed.documents.remove(read);
            assert!(!step_6_5(&observed).is_empty(), "{read:?}");
        }
    }
    #[test]
    fn result_messages_require_their_number_sender_kind_and_words() {
        let passing = met();
        for (message, promise) in
            [(BUDGET_A5_REPORT_NUMBER, BUDGET_ONESHOT), (BUDGET_ADVISOR_REPORT_NUMBER, BUDGET_RESULT)]
        {
            for field in ["number", "from", "kind", "words"] {
                let mut observed = passing.clone();
                for read in budget_pages(Read::BudgetSubownerEnded) {
                    let doc = observed.documents.get_mut(&read).expect("page");
                    let page = if doc.get("activity").is_some() { &mut doc["activity"] } else { doc };
                    for row in page["rows"].as_array_mut().expect("rows") {
                        let heard = &mut row["event"]["owner"]["source"]["line"]["heard"];
                        if heard["number"] == message {
                            heard[field] = Value::Null;
                        }
                    }
                }
                rejects(&observed, promise);
            }
        }
    }

    #[test]
    fn earlier_pages_cannot_hide_gaps_missing_starts_or_failed_attempts() {
        let passing = met();
        for read in [Read::BudgetDirectorFirst, Read::BudgetOwnerFirst, Read::BudgetSubownerFirst] {
            for mutation in 0..4 {
                let mut observed = passing.clone();
                let page = observed.documents.get_mut(&read).expect("page");
                match mutation {
                    0 => page["cut"] = json!(true),
                    1 => page["before"] = json!({"value":"another page"}),
                    2 => {
                        page["rows"].as_array_mut().expect("rows").pop();
                    }
                    3 => page["rows"][0]["event"]["owner"]["source"]["position"] = json!(99),
                    _ => unreachable!(),
                }
                rejects(&observed, BUDGET_ATTEMPTS);
            }
        }
    }

    #[test]
    fn only_unspent_uncommitted_rechecks_followed_by_a_finished_attempt_are_allowed() {
        let passing = met();
        for (read, task, _) in budget_tasks() {
            let mut accepted = passing.clone();
            let mut lines = vec![
                json!({"run_began":{"attempt":99}}),
                json!({"run_ended":{"attempt":99,"end":"refused","spent":0}}),
            ];
            lines.extend(budget_rows(&accepted, read).iter().rev().map(|r| r["line"].clone()));
            let mut page = journal(task, &lines);
            let pages = budget_pages(read);
            if pages.len() == 2 {
                let older = page["rows"].as_array_mut().expect("rows").split_off(3);
                page["before"] = json!({"value":"older"});
                accepted
                    .documents
                    .insert(pages[1], json!({"rows":older,"cut":false,"before":null,"after":{"value":"newer"}}));
            }
            accepted.documents.get_mut(&read).expect("doc")["activity"] = page;
            assert!(!step_6_5(&accepted).iter().any(|m| m.promise == BUDGET_ATTEMPTS));
            for mutation in 0..3 {
                let mut changed = accepted.clone();
                let last = *pages.last().expect("page");
                let doc = changed.documents.get_mut(&last).expect("doc");
                let page = if doc.get("activity").is_some() { &mut doc["activity"] } else { doc };
                let rows = page["rows"].as_array_mut().expect("rows");
                let recheck = rows
                    .iter_mut()
                    .find(|r| r.pointer("/event/owner/source/line/run_ended/attempt") == Some(&json!(99)))
                    .expect("recheck");
                match mutation {
                    0 => recheck["event"]["owner"]["source"]["line"]["run_ended"]["spent"] = json!(1),
                    1 => {
                        recheck["event"]["owner"]["source"]["line"]["run_ended"]["end"] =
                            json!({"failed":{"class":"transient"}});
                    }
                    2 => recheck["event"]["owner"]["source"]["line"]["turn"] = json!({"attempt":99}),
                    _ => unreachable!(),
                }
                rejects(&changed, BUDGET_ATTEMPTS);
            }
        }
    }
    #[test]
    fn the_first_finding_requires_its_number_sender_words_and_unique_delivery() {
        let passing = met();
        for mutation in 0..5 {
            let mut observed = passing.clone();
            if mutation == 4 {
                observed.told[9].history = observed.told[9].history[1..].into();
            } else {
                for read in budget_pages(Read::BudgetSubownerEnded) {
                    let doc = observed.documents.get_mut(&read).expect("page");
                    let page = if doc.get("activity").is_some() { &mut doc["activity"] } else { doc };
                    for row in page["rows"].as_array_mut().expect("rows") {
                        let words = &mut row["event"]["owner"]["source"]["line"]["words"];
                        if words["message"] == BUDGET_FINDING_NUMBER {
                            words[["message", "by", "kind", "words"][mutation]] = Value::Null;
                        }
                    }
                }
            }
            rejects(&observed, BUDGET_EFFORT);
        }
    }
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
            commands: Vec::new(),
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
            commands: vec![],
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
