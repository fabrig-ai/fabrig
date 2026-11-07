//! The composer's model and effort apply with one press (client/README.md, 3.5).
use skein_browser::boundary::Key;
use skein_fake_person::{Find, Step as PersonStep};
use skein_fake_person_browser::{Harness, Outcome, Script, Step};
use skein_ui::Role;
use temper_client_browser_world::{Tier, WITHIN, start_chat};

#[test]
fn the_model_picker_at_the_composer() {
    let tier = Tier::new("model-picker").with_model();
    let setup = tier.setup();
    // This story needs two offered models and efforts, without reaching an LLM.
    let directory = std::path::Path::new(setup.args.get(2).expect("configuration directory"));
    let path = directory.join("temper.json");
    let mut configuration: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("deployment")).expect("deployment JSON");
    let llm = &mut configuration["engine"]["llm"];
    let mut other = llm["models"][0].clone();
    other["name"] = "other".into();
    other["efforts"] = serde_json::json!(["low", "high"]);
    other["effort"] = "high".into();
    llm["models"].as_array_mut().expect("models").push(other);
    llm["providers"][0]["models"] = serde_json::json!(["fake", "other"]);
    std::fs::write(&path, serde_json::to_vec(&configuration).expect("picker deployment JSON"))
        .expect("picker deployment");

    let see = |find| Step::Person(PersonStep::See { find, within: WITHIN });
    let press = |role, name: &[u8]| Step::Person(PersonStep::Press { find: Find::exactly_named(role, name) });
    let folded = |open| keep(if open { "details.temper-model[open]" } else { "details.temper-model:not([open])" });
    let mut steps = start_chat(b"Choose the next model");
    steps.extend([
        Step::Viewport { width: 420, height: 900 },
        press(Role::Disclosure, b"Model"),
        folded(true),
        see(Find::text(b"A change applies from the next turn.")),
        press(Role::Button, b"other"),
        see(Find::text(b"other \xc2\xb7 high")),
        // A single press changes the line; no Apply control is involved.
        press(Role::Disclosure, b"Model"),
        folded(false),
        press(Role::Disclosure, b"Model"),
        folded(true),
        keep(".temper-model-menu li > button[aria-pressed=true]"),
        press(Role::Button, b"low"),
        see(Find::text(b"other \xc2\xb7 low")),
        keep(".temper-model-effort button[aria-pressed=true]"),
        Step::Viewport { width: 1440, height: 900 },
        press(Role::TextArea, b"Message"),
        folded(false),
        press(Role::Disclosure, b"Model"),
        folded(true),
        Step::Key(Key::Escape),
        folded(false),
    ]);
    let mut script = Script::new(steps.into_boxed_slice());
    assert_eq!(Harness::new(setup).run(&mut [&mut script]), Outcome::Done);
}

fn keep(selector: &str) -> Step {
    Step::Keep {
        selector: selector.as_bytes().into(),
        attribute: Some(b"data-node".as_slice().into()),
        name: b"model-picker".as_slice().into(),
    }
}
