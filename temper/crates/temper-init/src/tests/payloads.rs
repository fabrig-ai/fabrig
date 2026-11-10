//! Design edits through the shipped profile (temper/docs/design/protocol/configuration/settings.md, 7).
#![expect(clippy::disallowed_methods, reason = "ordinary test code inspects the fake checkout's tree")]

use super::fixtures::{self, DIRECTORIES, call, content, parse};
use serde_json::json;
use skein_fake_checkout::Checkout;
use skein_fake_llm_domain::api::{Part, Script};
use skein_lib::Token;
use smith_agent_world::{CompletionTerminal, World, fixture, scripted_charter};
use smith_domain::run;

const EDIT_BYTES: usize = 4487;
const BEFORE: &str = "Design draft.\n";

fn edit_world(document: serde_json::Value, replacement: &str) -> World {
    let mut settings = fixtures::agent_settings(document);
    settings.writable = true;
    let mut charter = scripted_charter(&settings);
    charter.models = Box::new([]);
    charter.instructions = b"@design".as_slice().into();
    let mut disk = Checkout::new();
    disk.mkdir(b"work/docs");
    disk.write(b"work/docs/design.md", BEFORE.as_bytes());
    let workspace = run::Workspace {
        directories: Box::new([run::Directory {
            host: b"/host".as_slice().into(),
            name: b"work".as_slice().into(),
            root: Token::new(disk.root(b"work")),
            writable: true,
            git: false,
            conflicts: Box::new([]),
        }]),
    };
    let scripts = Box::new([Script {
        cue: b"@design".as_slice().into(),
        turns: Box::new([
            fixture::calls(Box::new([call(b"read", &json!({"path": "docs/design.md"}))]), 1),
            fixture::calls(
                Box::new([call(
                    b"edit",
                    &json!({"path": "docs/design.md", "edits": [{"old": BEFORE, "new": replacement}]}),
                )]),
                1,
            ),
            fixture::calls(
                Box::new([call(b"finish", &json!({"report": "Design written.", "source": "workspace"}))]),
                1,
            ),
        ]),
    }]);
    World::with_workspace_scripts_charter(settings, None, Some(workspace), disk, scripts, charter)
}

#[test]
fn stock_and_reference_profiles_accept_a_4487_byte_design_edit() {
    let generated = crate::files(&fixtures::facts(), &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("init files");
    // The reference overlays engine settings and limits, retaining the stock agent.
    // Check the owned profile too: init adds its scripted model to this profile.
    let replacement = "d".repeat(EDIT_BYTES);
    for document in [parse(content(&generated, b"agent.json")), parse(fixtures::AGENT)] {
        let mut world = edit_world(document, &replacement);
        world.run(10_000);
        for prompt in world.prompts() {
            for message in &prompt.messages {
                for part in &message.parts {
                    if let Part::ToolOutput { output, is_error, .. } = part {
                        assert!(!is_error, "tool refused: {}", core::str::from_utf8(output).expect("tool output text"));
                    }
                }
            }
        }
        assert!(
            world
                .disk()
                .tree(b"work")
                .get(b"docs/design.md".as_slice())
                .is_some_and(|text| text == replacement.as_bytes()),
            "the edit must replace the whole design document"
        );
        let run::Answer::Accepted { outcome: run::outcome::Declared::Report(report), .. } = world.answer() else {
            panic!("design run finishes: {:?}", world.answer());
        };
        assert_eq!(report.text.as_ref(), b"Design written.");
        assert_eq!(world.completions().len(), 3, "read, edit and finish");
        for completion in world.completions() {
            let Some((_, CompletionTerminal::Completed(_))) = completion.terminal else {
                panic!("successful completion: {completion:?}");
            };
        }
    }
}
