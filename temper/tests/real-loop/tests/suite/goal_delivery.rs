//! A goal delivers on r and lands through a change into g (domain/agent.md,
//! 4.3, 4.3.1 and 6; domain/forge.md, 8.1 and 11.1).
use serde_json::{Value, json};
use skein_fake_forge_domain::api;
use skein_fake_llm_domain::api::{Finish, Line, Part, Script, Turn};
use skein_fake_peers::llm::Observation;
use std::{cell::RefCell, net::SocketAddr};
use temper_real_loop_world::{
    self as world, argv,
    calls::{self, D1},
    fixtures, forge, landing, settings,
};

const WORDS: &str = "@goal Write the design";
const REPORT: &str = "The design landed in the goal branch.";
const EDITED: &[u8] = b"# app\nGoal design\n";
const RUN_BRANCH: &[u8] = b"temper/1/r1";
const GOAL_BRANCH: &[u8] = b"temper/1/g1";

thread_local! {
    static FORGE: RefCell<Option<temper_fake_git_hosted::Typed>> = const { RefCell::new(None) };
    static TEMPLATE: RefCell<Option<Value>> = const { RefCell::new(None) };
}

fn configure(document: &mut Value, agent: &mut Value, address: SocketAddr) {
    landing::configure(document, agent, address);
    let tools = jig_domain_core::ToolFamilies::JIG.delegate.0 | jig_domain_core::ToolFamilies::JIG.control.0;
    for project in document["world_settings"]["policies"].as_array_mut().expect("project policies") {
        project["policy"]["ceiling"]["tools"] = tools.into();
        project["policy"]["roles"][0]["authority"]["tools"] = tools.into();
    }
    // The three stock templates and this goal's own-branch template.
    document["engine"]["limits"]["overrides"]["core.templates"] = 4.into();
    // A tracked goal keeps core news, landing news and its delegate's result.
    document["engine"]["limits"]["overrides"]["tasks.inbox_bytes"] = 16384.into();
    let mut template = settings::stock(document)["template"].clone();
    template["number"] = 4.into();
    template["name"] = json!({"bytes":"goal design","cut":false});
    template["contract"] = json!({"report":{"words":128}});
    template["tracked"] = 1.into();
    template["label"] = 2.into();
    template["budget"] = json!({"as_allowed":{"cap":64}});
    template["flags"]["ended_by"] = "result".into();
    template["flags"]["conversation"] = false.into();
    template["flags"]["listed"] = true.into();
    template["resources"] = json!([{"connector":1,"symbol":1}]);
    let mut charter = settings::stock(document)["charter"].clone();
    charter["modify"] = true.into();
    charter["waiting"] = 1_000_000.into();
    TEMPLATE.with(|slot| {
        *slot.borrow_mut() =
            Some(json!([settings::item("charter", 1, charter), settings::item("template", 4, template)]));
    });
}

fn tool(name: &[u8], arguments: &Value) -> Turn {
    Turn {
        lines: Box::new([Line::Call {
            name: name.into(),
            arguments: serde_json::to_vec(arguments).expect("tool JSON").into(),
        }]),
        finish: Finish::ToolCalls,
        tokens: 20,
    }
}

fn scripts(_: SocketAddr) -> Box<[Script]> {
    FORGE.with(|slot| *slot.borrow_mut() = Some(world::git::transport()));
    Box::new([Script {
        cue: WORDS.as_bytes().into(),
        turns: Box::new([
            // The goal's first activation parks while its branch is confirmed.
            tool(b"wait", &json!({})),
            tool(b"edit", &json!({"path":"README.md","edits":[{"old":"# app\n","new":"# app\nGoal design\n"}]})),
            tool(b"deliver", &json!({"title":"Goal design","body":"Write the goal's design."})),
            tool(b"change", &json!({"words":"Land the design","branch":"run","landing":"goal"})),
            tool(b"wait", &json!({})),
            tool(b"finish", &json!({"form":"report","text":REPORT,"fields":{}})),
        ]),
    }])
}

fn steps() -> Vec<calls::Step> {
    let template = TEMPLATE.with(|slot| slot.borrow_mut().take().expect("configured goal template"));
    let mut steps = vec![calls::sign_in(D1)];
    steps.extend(fixtures::change_settings(D1, "goal-template", template));
    steps.extend([
        calls::call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                "1",
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--key",
                "adopt-app"
            ],
        )
        .succeeds(),
        calls::call(D1, argv!["goal", "start", "--project", "1", "--template", "4", WORDS, "--key", "goal-design"])
            .succeeds(),
        calls::call(D1, argv!["task", "show", "1", "--until", "ended", "--json"]),
    ]);
    steps
}

fn branch_file(transport: &temper_fake_git_hosted::Typed, branch: &[u8]) -> Vec<u8> {
    let forge = transport.forge.borrow();
    let api::Answer::Commit(head) = forge
        .domain
        .inspect(&forge.config, b"acme/app", &api::Read::Branch { branch: branch.into() })
        .expect("goal or run branch")
    else {
        panic!("branch commit")
    };
    forge.domain.object(head).expect("branch object").tree.get(b"README.md".as_slice()).expect("branch README").to_vec()
}

#[test]
fn a_goals_run_delivers_and_lands_through_a_change() {
    let ran = world::run_forge(forge::Fixture { configure, ..forge::fixture() }, scripts, steps);
    for call in ran.calls() {
        assert_eq!(
            call.exit,
            Some(world::Exit::Code(0)),
            "ctl: {} {}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
    let transport = FORGE.with(|slot| slot.borrow_mut().take().expect("captured public git transport"));
    assert_eq!(branch_file(&transport, RUN_BRANCH), EDITED);
    assert_eq!(branch_file(&transport, GOAL_BRANCH), EDITED);
    assert_eq!(ran.forge().main_file(b"README.md"), landing::README);
    let task: Value = serde_json::from_slice(&ran.calls().last().expect("ended goal").stdout).expect("task JSON");
    assert_eq!(task["task"]["chip"]["phase"], "ended", "{task}");
    assert!(
        task["task"]["chip"]["details"]["owner"]["details"]["ending"]["done"].is_object(),
        "the goal ends done: {task}"
    );
    assert_eq!(task["task"]["result"]["value"]["report"]["bytes"], REPORT, "{task}");
    let mut delivered = false;
    for observed in ran.llm() {
        if let Observation::Query { query, .. } = observed {
            for message in &query.messages {
                for part in &message.parts {
                    if let Part::ToolOutput { id, output, is_error } = part {
                        assert!(
                            !is_error && !output.starts_with(b"Error:"),
                            "tool refused: {}",
                            String::from_utf8_lossy(output)
                        );
                        let is_delivery = query.messages.iter().flat_map(|message| &message.parts).any(|part| {
                            if let Part::ToolCall { id: called, name, .. } = part {
                                called == id && name.as_ref() == b"deliver"
                            } else {
                                false
                            }
                        });
                        if is_delivery {
                            assert!(
                                output.starts_with(b"delivered\nreceipt directory="),
                                "delivery answer: {}",
                                String::from_utf8_lossy(output)
                            );
                            assert!(
                                output.windows(RUN_BRANCH.len()).any(|bytes| bytes == RUN_BRANCH),
                                "receipt names the run branch: {}",
                                String::from_utf8_lossy(output)
                            );
                            println!("delivery answer: {}", String::from_utf8_lossy(output));
                            delivered = true;
                        }
                    }
                }
            }
        }
    }
    assert!(delivered, "the model received its run branch's delivery receipt");
}
