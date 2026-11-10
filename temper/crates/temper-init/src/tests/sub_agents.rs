//! Stock concurrency through Smith's domain (temper/docs/design/protocol/setup/init.md, 1;
//! smith/docs/design/domain/session.md, 5; smith/docs/design/protocol/limits.md, 2.3).

#![expect(clippy::disallowed_methods, reason = "ordinary Rust test code traverses fake provider observations")]

use super::fixtures::{self, DIRECTORIES, call, content, parse};
use serde_json::{Value, json};
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Line, Message, Part, Script, Turn};
use skein_lib::Token;
use smith_agent_world::{CompletionTerminal, World, fixture, scripted_charter};
use smith_domain::run;

// The recorded parent response has two inspect-only calls, each with this allowance.
const CHILDREN: usize = 2;
const CHILD_TURNS: u32 = 3;
const CHILD_SECONDS: u32 = 30;
// Each child reads and replies; the parent delegates and finishes.
const COMPLETIONS: usize = CHILDREN * 2 + 2;
const FILES: [(&str, &[u8]); CHILDREN] =
    [("src/main.rs", b"fn main() {}"), ("README.md", b"A small example application.")];
const REPORT: &str = "src/main.rs defines main; README.md describes the application.";

fn calls(lines: Box<[Line]>) -> Turn {
    fixture::calls(lines, 1)
}

fn scripts() -> Box<[Script]> {
    let [first, second] = FILES.map(|(path, finding)| Script {
        cue: path.as_bytes().into(),
        turns: Box::new([
            calls(Box::new([call(b"read", &json!({"path": path}))])),
            fixture::says(Box::new([Line::Text { text: finding.into() }]), 1),
        ]),
    });
    Box::new([
        Script {
            cue: b"@parent".as_slice().into(),
            turns: Box::new([
                calls(
                    FILES
                        .map(|(path, _)| {
                            call(
                                b"sub_agent",
                                &json!({
                                    "brief": path, "tools": ["inspect"],
                                    "max_turns": CHILD_TURNS, "max_seconds": CHILD_SECONDS
                                }),
                            )
                        })
                        .into(),
                ),
                calls(Box::new([call(b"finish", &json!({"report": REPORT, "source": "workspace"}))])),
            ]),
        },
        first,
        second,
    ])
}

fn world(document: Value) -> World {
    let settings = fixtures::agent_settings(document);
    let mut charter = scripted_charter(&settings);
    charter.models = Box::new([]);
    charter.instructions = b"@parent".as_slice().into();
    let mut disk = Checkout::new();
    disk.mkdir(b"work/src");
    for (path, bytes) in FILES {
        let path = [b"work/", path.as_bytes()].concat();
        disk.write(&path, bytes);
    }
    let workspace = run::Workspace {
        directories: Box::new([run::Directory {
            host: b"/host".as_slice().into(),
            name: b"work".as_slice().into(),
            root: Token::new(disk.root(b"work")),
            writable: false,
            git: false,
            conflicts: Box::new([]),
        }]),
    };
    World::with_workspace_scripts_charter(settings, None, Some(workspace), disk, scripts(), charter)
}

fn output_contains(messages: &[Message], finding: &[u8]) -> bool {
    for message in messages {
        for part in &message.parts {
            let Part::ToolOutput { output, is_error, .. } = part else { continue };
            if !is_error && skein_lib::bytes::find(output, finding).is_some() {
                return true;
            }
        }
    }
    false
}

fn check_results(world: &World) -> bool {
    let run::Answer::Accepted { outcome: run::outcome::Declared::Report(report), .. } = world.answer() else {
        panic!("parent report: {:?}", world.answer());
    };
    assert_eq!(report.text.as_ref(), REPORT.as_bytes());
    assert_eq!(world.completions().len(), COMPLETIONS);
    let parent = world.prompts().last().expect("parent receives both findings");
    for (_, finding) in FILES {
        assert!(output_contains(&parent.messages, finding), "child finding reaches parent");
        // The domain world's typed adapter renders owned read results with Debug.
        let read_bytes = format!("{finding:?}");
        assert!(
            world.prompts()[1..COMPLETIONS - 1]
                .iter()
                .any(|prompt| { output_contains(&prompt.messages, read_bytes.as_bytes()) }),
            "child read returns the assigned file"
        );
    }
    for completion in world.completions() {
        let Some((_, CompletionTerminal::Completed(_))) = completion.terminal else {
            panic!("successful completion: {completion:?}");
        };
    }
    let first = &world.completions()[1];
    let second = world
        .completions()
        .iter()
        .find(|completion| completion.owner != first.owner && completion.owner != world.completions()[0].owner)
        .expect("second child");
    second.started < first.terminal.expect("first child read settled").0
}

#[test]
fn stock_profile_runs_two_inspect_only_children_side_by_side() {
    let generated = crate::files(&fixtures::facts(), &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("init files");
    let mut world = world(parse(content(&generated, b"agent.json")));
    let before = world.disk().tree(b"work");
    world.run(10_000);
    let overlap = check_results(&world);
    assert_eq!(world.disk().tree(b"work"), before, "inspect-only checkout is unchanged");
    assert!(overlap, "both children start before either completes");
}
