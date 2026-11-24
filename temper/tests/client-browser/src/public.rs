//! Public reading on the box's demo (temper/docs/design/client/web/testing.md, 3).
use crate::{Tier, bundle, configuration, forgery, scripted, temper};
use serde_json::{Value, json};
use skein_fake_person_browser::{Script, Step};
use std::{ffi::OsString, path::Path};

const CSV_CHAT: &str = "Add CSV export to widget";
const CSV_TITLE: &str = "Export widget's table as CSV";
const BOARD_TITLE: &str =
    "Export widget's table as CSV with visible columns, localized headings and a readable filename for monthly reports";

impl Tier {
    /// The CSV demo's live public goal with a long title for responsive layout.
    #[must_use]
    pub fn public_demo(name: &str) -> Self {
        let scratch = skein_scratch::Scratch::new(name);
        configuration::public_demo(scratch.path(), &temper(), &bundle());
        scripted::write(scratch.path(), &board_scripts());
        Self { scratch, forged: forgery::request() }
    }

    /// Make and publish Demo through ctl alone, keeping its printed page in `answer`.
    #[must_use]
    pub fn ctl_public_board(&self, answer: &Path) -> Script {
        let mut steps = self.ctl_chat(CSV_CHAT, &[]);
        // T1 has delegated procedure T2. Watch its producer T3 until it parks.
        // Until T2 creates T3 the watch answers `unknown`; only that refusal retries.
        steps.extend(through_shell(
            self.ctl(&["task", "show", "3", "--until", "phase:idle"]),
            concat!(
                "while :; do ",
                "result=$(\"$@\" 2>&1) && exit 0; code=$?; ",
                "if [ \"$code\" != 2 ] || [ \"$result\" != unknown ]; then ",
                "printf '%s\\n' \"$result\" >&2; exit \"$code\"; fi; ",
                "sleep 0.05; done"
            ),
            None,
        ));
        // The browser harness does not expose command stdout to Play.
        steps.extend(through_shell(
            self.ctl(&["project", "amend", "Demo", "--visibility", "public", "--key", "public-board"]),
            "umask 077; answer=$1; shift; exec \"$@\" > \"$answer\"",
            Some(answer),
        ));
        self.ctl_only(steps)
    }
}

fn through_shell(mut steps: [Step; 2], script: &str, answer: Option<&Path>) -> [Step; 2] {
    let Step::Run(command) = &mut steps[0] else { unreachable!("ctl starts with a command") };
    let mut args: Vec<OsString> = vec!["-c".into(), script.into(), "seed-public-board".into()];
    if let Some(answer) = answer {
        args.push(answer.into());
    }
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

    let original = format!("## Task\n\n{CSV_TITLE}");
    let producer = value["cued"]
        .as_array_mut()
        .expect("cued scripts")
        .iter_mut()
        .find(|script| script["cue"] == original)
        .expect("CSV producer");
    producer["cue"] = format!("## Task\n\n{BOARD_TITLE}").into();
    producer["turns"] = json!([{"lines":[{"call":"wait","arguments":{}}],"finish":"tool_calls","tokens":2}]);
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
        let [turn] = producer["turns"].as_array().expect("producer turns").as_slice() else {
            panic!("the delegate must park on its only turn");
        };
        for turn in [waiting, turn] {
            assert_eq!(turn["finish"], "tool_calls");
            assert_eq!(turn["lines"], json!([{"call":"wait","arguments":{}}]), "no work after delegation");
        }
    }
}
