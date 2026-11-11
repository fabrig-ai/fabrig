//! The welcome's paths to a first chat on `box --fake-backends`, sent as the acts its buttons send.
//! First sign-in block: temper/docs/design/protocol/setup/box.md, 2.
use crate::box_link;
use crate::support::{
    self, Ended, boxed,
    ctl::{self, Call, Step},
};
use serde_json::Value;
use skein_io::kernel::Exit;
use std::path::{Path, PathBuf};

/// One fake box on an empty configuration, as a person starts it.
struct Walk {
    scratch: support::Scratch,
    paths: [PathBuf; 3],
    bundle: PathBuf,
}

enum Act {
    Now(Call),
    After(ctl::NextCall),
    /// Follow the preceding act's printed address through the box's supervised peers.
    ProgramLogin(&'static str),
    Mark(&'static str),
}

impl Walk {
    fn new(case: &str) -> Self {
        let scratch = support::Scratch::new(case);
        let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
        for path in &paths {
            super::private(path);
        }
        let bundle = scratch.path().join("loopback-bundle");
        // This process story sends acts through ctl; only the bundle's file shape is needed.
        super::bundle(&bundle, super::SOURCE_PAGE);
        Self { scratch, paths, bundle }
    }

    fn owned(case: &str) -> Self {
        let walk = Self::new(case);
        let _cached = support::forge_land::cache_forgejo_binaries(&walk.paths[2]);
        walk
    }

    fn sandbox(&self) -> PathBuf {
        self.paths[1].join("box/fake-backends")
    }

    fn key(&self) -> PathBuf {
        self.scratch.path().join("model-key-paste")
    }

    /// The Model step's key way: copy the key the box leaves for the person, and add it.
    fn add_key(&self) -> Act {
        let source = self.sandbox().join("data/llm.key");
        let paste = self.key();
        Act::After(Box::new(move |_| {
            super::write(&paste, &std::fs::read(&source).expect("box's fake model key"));
            Call::new(&["account", "add-key", "1", path(&paste), "--deployment", "--key", "walk-model", "--json"])
        }))
    }

    /// Runs the acts, then `check` while the box still serves, given the configuration
    /// directory and the client root that ctl signed in under.
    fn run_then(&self, acts: Vec<Act>, check: impl FnOnce(&Path, &Path)) -> Seen {
        let count = acts.iter().filter(|act| !matches!(act, Act::Mark(_))).count();
        let mut steps = vec![Step::Run(Call::sign_in())];
        steps.extend(acts.into_iter().map(|act| match act {
            Act::Now(call) => Step::Run(call),
            Act::ProgramLogin(name) => Step::ProgramLogin(name),
            Act::Mark(name) => Step::Mark(name),
            Act::After(next) => Step::Then(Box::new(move |seen| {
                let (_, answers) = seen.split_first().expect("signed in before the welcome's acts");
                next(answers)
            })),
        }));
        let outcome = boxed::run_ctl(
            self.scratch.path(),
            super::flags(&self.paths[0], &self.paths[1], &self.paths[2], Some(&self.bundle)),
            self.scratch.path(),
            steps,
            crate::BOX_DEADLINE,
            |_binary| check(&self.sandbox().join("config"), self.scratch.path()),
        );
        let diagnostic = String::from_utf8_lossy(&outcome.stderr);
        assert_first_block(&outcome.stdout, &outcome.stderr);
        assert!(!diagnostic.contains("Logs:"), "the logs notice belongs to box.log");
        // A refused restart must fail on the box's terminal, before missing
        // ctl answers obscure the reason it never printed a new sign-in link.
        assert_eq!(outcome.exit, Some(Exit::Code(0)), "healthy box stop: {diagnostic}");
        let last = outcome.calls.iter().rev().find(|call| call.exit == Some(Exit::Code(0)));
        let last = last.map_or_else(String::new, |call| String::from_utf8_lossy(&call.stdout).into_owned());
        for (index, call) in outcome.calls.iter().enumerate() {
            assert_eq!(
                call.exit,
                Some(Exit::Code(0)),
                "call {index}: stdout {}; stderr {}; last successful answer: {last}; box: {diagnostic}",
                String::from_utf8_lossy(&call.stdout),
                String::from_utf8_lossy(&call.stderr)
            );
        }
        assert_eq!(outcome.calls.len(), count + 1, "all welcome acts completed; box: {diagnostic}");
        assert!(outcome.checked.is_some(), "live checkpoint reached; box: {diagnostic}");
        assert_eq!(outcome.left, Some(Vec::new()), "no child outlives its box: {diagnostic}");
        Seen { answers: outcome.calls.into_iter().skip(1).collect() }
    }
}

fn assert_first_block(stdout: &[u8], stderr: &[u8]) {
    let diagnostic = String::from_utf8_lossy(stderr);
    let link = support::printed_link(stdout).expect("first printed sign-in link");
    let address = box_link::address(link);
    // The walk's PATH has only system programs, and its environment has no HOME.
    let binary = std::fs::canonicalize(skein_scratch::paths::binary("temper-box")).expect("test binary");
    let command = format!("In a terminal: {} http://{address}\n", binary.display());
    let block = [box_link::PRELUDE, box_link::FOOTER, command.as_bytes()].concat();
    let after_block = stderr
        .strip_prefix(block.as_slice())
        .unwrap_or_else(|| panic!("the sign-in block is first on stderr: {diagnostic}"));
    assert!(
        after_block.starts_with(b"box: effective configuration "),
        "the effective directories follow the block: {diagnostic}"
    );
}

fn path(path: &Path) -> &str {
    path.to_str().expect("fixture path")
}

fn usable(call: &Ended) -> bool {
    let document = ctl::json(call);
    document["accounts"]["usable"].as_bool().expect("account usability")
}

// A first run keeps no forge until SetUp or Connect makes its record
// (temper/docs/design/protocol/forge/engine.md, 2).
fn forge_rows(call: &Ended) -> Vec<Value> {
    let document = ctl::json(call);
    let document = if document.get("met").is_some() { &document["forges"] } else { &document };
    document["forges"].as_array().expect("forge rows").clone()
}

fn only_forge(call: &Ended) -> Value {
    let rows = forge_rows(call);
    let [row] = rows.as_slice() else { panic!("one forge made by this walk: {rows:?}") };
    row.clone()
}

fn number(row: &Value) -> u16 {
    u16::try_from(row["number"].as_u64().expect("forge number")).expect("bounded forge number")
}

fn adoption(call: &Ended) -> Value {
    // SetUp and Connect print progress before the terminal adoption answer.
    serde_json::Deserializer::from_slice(&call.stdout)
        .into_iter::<Value>()
        .last()
        .expect("adoption answer")
        .expect("adoption JSON")
}

fn adopted_forge(call: &Ended) -> u16 {
    u16::try_from(adoption(call)["outcome"]["adopted"]["forge"].as_u64().expect("adopted forge"))
        .expect("bounded forge number")
}

struct Seen {
    answers: Vec<Ended>,
}

mod here_key;
mod open_default;
mod program_here;
mod sign_in;

fn assert_reply(seen: &Seen) {
    let [.., waited, conversation] = seen.answers.as_slice() else { panic!("reply and conversation reads") };
    assert_committed_reply(waited, conversation);
}

fn assert_committed_reply(waited: &Ended, conversation: &Ended) {
    assert_eq!(ctl::json(waited)["met"], true, "chat replied before the wait ended");
    let page = ctl::json(conversation);
    assert_eq!(page["activity"]["cut"], false, "whole conversation page");
    let turns: Vec<_> = page["activity"]["rows"]
        .as_array()
        .expect("conversation rows")
        .iter()
        .filter(|row| row["event"]["owner"]["source"]["line"]["turn"].is_object())
        .map(|row| {
            let turn = &row["event"]["owner"]["decoded"]["turn"]["decoded"];
            assert!(turn.is_object(), "committed turn loaded and decoded: {row}");
            assert_eq!(turn["cut"], false, "whole committed turn");
            turn
        })
        .collect();
    assert_eq!(turns.len(), 1, "first reply committed once: {page}");
    let replies: Vec<_> = turns
        .iter()
        .flat_map(|turn| turn["messages"].as_array().expect("turn messages"))
        .filter(|message| message["speaker"] == "assistant")
        .flat_map(|message| message["blocks"].as_array().expect("assistant blocks"))
        .filter_map(|block| block["text"]["text"].as_str())
        .collect();
    assert_eq!(replies, ["Hello from the local fake model."]);
}
