//! Public reading on the box's demo (temper/docs/design/client/web/testing.md, 3).
use crate::{Tier, bundle, configuration, forgery, scripted, temper};
use serde_json::{Value, json};
use skein_fake_person_browser::{Script, Step};
use std::{ffi::OsString, path::Path};

// jig's ToolFamilies::JIG.subscribe, in ctl's settings and the delegate tool's JSON forms.
pub(crate) const SUBSCRIBE: u64 = 32;
const CSV_CHAT: &str = "Add CSV export to widget";
const CSV_TITLE: &str = "Export widget's table as CSV";
const BOARD_TITLE: &str =
    "Export widget's table as CSV with visible columns, localized headings and a readable filename for monthly reports";
// The empty task window's {"rows":[],"more":false}, SHA-256's first 8 bytes
// (temper/docs/design/client/ctl/tools-waiting.md, 1).
const NO_LIVE_DELEGATES: &str = "832c124041f0d04e";

impl Tier {
    /// The CSV demo's live public goal with a long title for responsive layout.
    #[must_use]
    pub fn public_demo(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::public_demo(scratch.path(), &temper(), &bundle());
        scripted::write(scratch.path(), &board_scripts());
        Self { scratch, forged: forgery::request() }
    }

    /// Run the board's model, letting its producer park as soon as it calls `wait`.
    #[must_use]
    pub fn public_board_model(&self) -> temper_box_kit::parent::Guard {
        // The stock producer holds its slot for 10 s before parking. The board
        // seeds a parked producer, not a measurement of the warm wait.
        self.scripted_model_with_parking(Some(2))
    }

    /// Make and publish Demo through ctl alone, keeping its printed page in `answer`.
    #[must_use]
    pub fn ctl_public_board(&self, answer: &Path) -> Script {
        let mut steps = self.ctl_chat(CSV_CHAT, &[]);
        // T1 has delegated procedure T2. Its only live delegate is producer T3.
        // Watch for its creation before asking the task watch to wait for idle.
        steps.extend(self.ctl(&["task", "list", "--project", "1", "--parent", "2", "--since", NO_LIVE_DELEGATES]));
        steps.extend(self.ctl(&["task", "show", "3", "--until", "phase:idle"]));
        // The browser harness does not expose command stdout to Play.
        steps.extend(through_shell(
            self.ctl(&["project", "amend", "Demo", "--visibility", "public", "--key", "public-board"]),
            "umask 077; answer=$1; shift; exec \"$@\" > \"$answer\"",
            answer,
        ));
        self.ctl_only(steps)
    }
}

fn through_shell(mut steps: [Step; 2], script: &str, answer: &Path) -> [Step; 2] {
    let Step::Run(command) = &mut steps[0] else { unreachable!("ctl starts with a command") };
    let mut args: Vec<OsString> = vec!["-c".into(), script.into(), "seed-public-board".into(), answer.into()];
    args.push(command.program.as_os_str().into());
    args.append(&mut command.args);
    command.args = args;
    command.program = "/bin/sh".into();
    steps
}

fn board_scripts() -> Value {
    let mut value: Value = serde_json::from_slice(temper_fake::demo::SCRIPTS).expect("embedded scripts");
    let chat = value["asked"]
        .as_array_mut()
        .expect("asked scripts")
        .iter_mut()
        .find(|script| script["cue"] == CSV_CHAT)
        .expect("CSV chat");
    let turns = chat["turns"].as_array_mut().expect("CSV turns");
    turns.truncate(2); // Delegate and wait; the producer never reports a change back.
    let delegate = turns[0]["lines"]
        .as_array_mut()
        .expect("CSV lines")
        .iter_mut()
        .find(|line| line["call"] == "delegate")
        .expect("CSV delegate call");
    let batch = delegate["arguments"]["batch"].as_array_mut().expect("delegate batch");
    let [goal] = batch.as_mut_slice() else { panic!("one CSV procedure becomes the public goal") };
    assert_eq!(goal["spec"]["words"], CSV_TITLE);
    goal["tracked"] = json!({"priority": 1});
    goal["spec"]["words"] = BOARD_TITLE.into();
    let tools = &mut goal["authority"]["tools"];
    *tools = json!(tools.as_u64().expect("delegation tool families") | SUBSCRIBE);

    let original = format!("## Task\n\n{CSV_TITLE}");
    let producer = value["cued"]
        .as_array_mut()
        .expect("cued scripts")
        .iter_mut()
        .find(|script| script["cue"] == original)
        .expect("CSV producer");
    producer["cue"] = format!("## Task\n\n{BOARD_TITLE}").into();
    // A result-ended agent with nothing to await wakes as soon as it parks.
    // An armed timer keeps the producer parked throughout the measurement.
    producer["turns"] = json!([
        {"lines":[{"call":"subscribe","arguments":{"kind":"timer","at":u64::MAX}}],"finish":"tool_calls","tokens":10},
        {"lines":[{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":2}
    ]);
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use temper_fake::llm::{Document, ScriptJoin};

    #[test]
    fn the_public_boards_scripts_leave_its_delegate_waiting() {
        let scripts = board_scripts();
        let document = scripted::document("127.0.0.1:8370", Path::new("/test/state/model.key"));
        let document = serde_json::to_vec(&document).expect("model document");
        let mut joined = ScriptJoin::empty();
        joined
            .push(&document, &serde_json::to_vec(&scripts).expect("board scripts"))
            .expect("the product fake admits the public scripts together");
        let bytes = joined.into_bytes();
        Document::decode_with_scripts(&document, Some(&bytes)).expect("the standalone fake admits the joined scripts");
        let joined: Value = serde_json::from_slice(&bytes).expect("joined scripts");
        let chat = joined["asked"]
            .as_array()
            .expect("asked scripts")
            .iter()
            .find(|script| script["cue"] == CSV_CHAT)
            .expect("admitted CSV chat");
        let [delegating, waiting] = chat["turns"].as_array().expect("CSV turns").as_slice() else {
            panic!("the chat only delegates and waits");
        };
        let delegate = delegating["lines"]
            .as_array()
            .expect("CSV lines")
            .iter()
            .find(|line| line["call"] == "delegate")
            .expect("admitted CSV delegate call");
        let [goal] = delegate["arguments"]["batch"].as_array().expect("batch").as_slice() else {
            panic!("the chat delegates one goal");
        };
        assert_eq!(goal["executor"], json!({"kind":"procedure","connector":1,"code":2}));
        assert_eq!(goal["tracked"], json!({"priority":1}));
        assert_eq!(goal["spec"]["words"], BOARD_TITLE);
        let cue = format!("## Task\n\n{BOARD_TITLE}");
        let producer = joined["cued"]
            .as_array()
            .expect("cued scripts")
            .iter()
            .find(|script| script["cue"] == cue)
            .expect("admitted cue for the renamed goal's producer");
        let [timer, turn] = producer["turns"].as_array().expect("producer turns").as_slice() else {
            panic!("the delegate arms a timer and then parks");
        };
        assert_eq!(SUBSCRIBE, jig_domain_core::ToolFamilies::JIG.subscribe.0);
        assert_eq!(goal["authority"]["tools"].as_u64().expect("tool families") & SUBSCRIBE, SUBSCRIBE);
        assert_eq!(timer["finish"], "tool_calls");
        assert_eq!(timer["lines"], json!([{"call":"subscribe","arguments":{"kind":"timer","at":u64::MAX}}]));
        for turn in [waiting, turn] {
            assert_eq!(turn["finish"], "tool_calls");
            assert_eq!(turn["lines"], json!([{"call":"wait","arguments":{}}]), "no work after delegation");
        }
    }
}
