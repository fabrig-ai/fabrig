//! Collector fixtures and fake ctl: `temper/docs/design/testing/checks-stage-one.md`, 1.3.
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output};

use serde_json::{Value, json};
use skein_fake_machine::{How, Item as File, Listed, Machine, Opened, Refusal};
use skein_scratch::{Scratch, paths};

use crate::collect::{self, CollectArgs, CtlOutputs};
use crate::evidence;
use crate::golden::{Edit, Item, ProjectRef, SettingKey, SettingsFile};
use crate::testing::{self, EvidenceSink, EvidenceSource};

/// Revision 3 and inventory reads for the project's ways and questions, selected independently.
pub(super) fn project_settings_outputs(project: u32, accepted: &[usize], listed: &[usize]) -> (CtlOutputs, Vec<Item>) {
    let settings = vec![
        Item {
            setting: SettingKey {
                kind: "project-questions".into(),
                number: None,
                project: Some(ProjectRef::Number(project)),
            },
            edit: Edit::Set(json!("up")),
        },
        Item {
            setting: SettingKey {
                kind: "project-ways".into(),
                number: None,
                project: Some(ProjectRef::Number(project)),
            },
            edit: Edit::Set(json!({"drafts":true,"goals":true,"changes":true,"way":4})),
        },
    ];
    let mut recorded = outputs();
    recorded.project = project;
    recorded.revision = 3;
    let mut project_read = value("project_show_json.txt");
    project_read["number"] = json!(project);
    recorded.by_command.insert(words(&format!("project show {project} --json")), project_read.to_string());
    let edits: Vec<_> = accepted
        .iter()
        .map(|at| {
            let item = &settings[*at];
            let Edit::Set(value) = &item.edit else { panic!("whole setting") };
            json!({"setting":item.setting,"set":value})
        })
        .collect();
    let inventory: Vec<_> = listed.iter().map(|at| json!({"setting":settings[*at].setting})).collect();
    recorded.by_command.insert(words("revision show 3 --json"), json!({"head":{"number":3},"items":edits}).to_string());
    recorded.by_command.insert(words("settings show --json"), json!({"settings":inventory}).to_string());
    for item in &settings {
        let Edit::Set(value) = &item.edit else { panic!("whole setting") };
        recorded.by_command.insert(
            words(&format!("settings show {} {project} --json", item.setting.kind)),
            json!({"setting":item.setting,"value":value}).to_string(),
        );
    }
    (recorded, settings)
}

#[test]
fn assemble_preserves_project_ways_and_questions() {
    for (accepted, listed) in [
        (vec![0], vec![0, 1]),
        (vec![1], vec![0, 1]),
        (vec![0, 1], vec![0, 1]),
        (vec![], vec![0, 1]),
        (vec![], vec![0]),
        (vec![], vec![1]),
        (vec![0], vec![]),
        (vec![1], vec![]),
    ] {
        let (recorded, settings) = project_settings_outputs(1, &accepted, &listed);
        let (items, current) = collect::assemble(&recorded).expect("collect project ways and questions");
        let expected: Vec<_> = accepted.iter().map(|at| settings[*at].clone()).collect();
        assert_eq!(items.items, expected, "accepted {accepted:?}, listed {listed:?}");
        let mut expected: Vec<_> = listed.iter().map(|at| settings[*at].clone()).collect();
        expected.push(Item {
            setting: SettingKey { kind: "project-policy".into(), number: None, project: Some(ProjectRef::Number(1)) },
            edit: edit(&current, "project-policy", None, Some(1)),
        });
        assert_eq!(current.items, expected, "accepted {accepted:?}, listed {listed:?}");
    }
}

macro_rules! transcript {
    ($call:literal, $file:literal) => {
        ($call, $file, include_str!(concat!("fixtures/ctl/", $file)))
    };
}

const READS: [(&str, &str, &str); 13] = [
    transcript!("revision show 5 --json", "revision.json"),
    transcript!("settings show charter 3 --json", "settings_value_0_json.txt"),
    transcript!("settings show charter 6 --revision 5 --json", "charter-6.json"),
    transcript!("settings show charter 6 --json", "charter-6.json"),
    transcript!("settings show template 4 --json", "settings_value_1_json.txt"),
    transcript!("settings show policy 5 --json", "settings_value_2_json.txt"),
    transcript!("settings show rules --json", "settings_value_3_json.txt"),
    transcript!("settings show landing-rule 2 --project 1 --json", "landing-rule-project.json"),
    transcript!("settings show landing-rule 1 --json", "landing-rule.json"),
    transcript!("settings show forge-charter 1 --json", "forge-charter.json"),
    transcript!("settings show forge-charter 2 --json", "forge-charter-removed.json"),
    transcript!("settings show --json", "inventory.json"),
    transcript!("project show 1 --json", "project_show_json.txt"),
];
fn fixture(name: &str) -> PathBuf {
    paths::package().join("src/tests/fixtures/ctl").join(name)
}
fn text(name: &str) -> String {
    fs::read_to_string(fixture(name)).expect("ctl fixture")
}
fn value(name: &str) -> Value {
    serde_json::from_str(&text(name)).expect("JSON fixture")
}
fn words(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_owned).collect()
}
fn outputs() -> CtlOutputs {
    let mut outputs = CtlOutputs { project: 1, revision: 5, ..CtlOutputs::default() };
    for (call, name, _) in READS {
        outputs.by_command.insert(words(call), text(name));
    }
    outputs
}
fn edit(file: &SettingsFile, kind: &str, number: Option<u32>, project: Option<u32>) -> Edit {
    file.items
        .iter()
        .find(|item| {
            item.setting.kind == kind
                && item.setting.number == number
                && item.setting.project == project.map(crate::golden::ProjectRef::Number)
        })
        .expect("setting present")
        .edit
        .clone()
}
#[test]
fn assemble_takes_values_whole_from_the_revision() {
    let mut outputs = outputs();
    outputs.by_command.insert(words("settings show charter 3 --json"), text("charter-6.json"));
    // The revision's charter is independent of the newest read.
    outputs.by_command.insert(words("settings show --json"), json!({"settings": []}).to_string());
    let (items, _) = collect::assemble(&outputs).expect("assemble");
    for (kind, number, fixture) in [("charter", 3, 0), ("template", 4, 1), ("policy", 5, 2)] {
        assert_eq!(
            edit(&items, kind, Some(number), None),
            Edit::Set(value(&format!("settings_revision_value_{fixture}_json.txt"))["items"][0]["set"].clone())
        );
    }
}
#[test]
fn assemble_reads_a_too_large_value_again() {
    let mut outputs = outputs();
    let (items, _) = collect::assemble(&outputs).expect("assemble");
    assert_eq!(edit(&items, "charter", Some(6), None), Edit::Set(value("charter-6.json")["value"].clone()));
    outputs.by_command.remove(&words("settings show charter 6 --revision 5 --json"));
    assert!(
        collect::assemble(&outputs)
            .expect_err("held charter read required")
            .to_string()
            .contains("charter 6 --revision 5")
    );
    let mut revision = value("revision.json");
    let row = &mut revision["items"][2];
    row.as_object_mut().expect("item").remove("set");
    row["too_large"] = json!(1_000_000);
    outputs.by_command.insert(words("revision show 5 --json"), revision.to_string());
    outputs.by_command.insert(words("settings show charter 6 --revision 5 --json"), text("charter-6.json"));
    let (items, _) = collect::assemble(&outputs).expect("large template reads current");
    assert_eq!(edit(&items, "template", Some(4), None), Edit::Set(value("settings_value_1_json.txt")["value"].clone()));
}
#[test]
fn assemble_reads_temper_kinds_from_settings_show() {
    let (items, _) = collect::assemble(&outputs()).expect("assemble");
    assert_eq!(
        edit(&items, "landing-rule", Some(2), Some(1)),
        Edit::Set(value("landing-rule-project.json")["value"].clone())
    );
    assert_eq!(edit(&items, "forge-charter", Some(1), None), Edit::Set(json!(3)));
    assert_eq!(edit(&items, "forge-charter", Some(2), None), Edit::Removed);
    assert_eq!(edit(&items, "landing-rule", Some(1), None), Edit::Reset);
}
#[test]
fn assemble_keeps_a_removed_item() {
    let (items, _) = collect::assemble(&outputs()).expect("assemble");
    assert_eq!(edit(&items, "charter", Some(9), None), Edit::Removed);
    assert_eq!(edit(&items, "rules", None, None), Edit::Reset);
}
#[test]
fn current_holds_every_listed_setting() {
    let (_, current) = collect::assemble(&outputs()).expect("assemble");
    let listed = value("inventory.json");
    assert_eq!(current.items.len(), listed["settings"].as_array().expect("settings").len() + 1);
    for row in listed["settings"].as_array().expect("settings") {
        assert!(
            current.items.iter().any(|item| serde_json::to_value(&item.setting).expect("setting") == row["setting"])
        );
    }
    assert_eq!(edit(&current, "rules", None, None), Edit::Set(value("settings_value_3_json.txt")["value"].clone()));
    assert_eq!(edit(&current, "forge-charter", Some(2), None), Edit::Removed);
    let Edit::Set(policy) = edit(&current, "project-policy", None, Some(1)) else { panic!("policy set") };
    let project = value("project_show_json.txt");
    assert_eq!(policy["period_spend"], project["policy"]["period_spend"]);
    assert_eq!(policy["roles"][0]["number"], 0);
    assert_eq!(policy["roles"][0]["authority"], project["policy"]["roles"][0]["authority"]["authority"]);
    assert!(policy.get("ceiling").is_none());
}
#[test]
fn a_whole_project_policy_set_is_also_current() {
    let mut outputs = outputs();
    outputs.by_command.insert(words("revision show 5 --json"), text("revision-policy.json"));
    outputs.by_command.remove(&words("project show 1 --json"));
    let (items, current) = collect::assemble(&outputs).expect("whole revision policy needs no project read");
    assert_eq!(edit(&current, "project-policy", None, Some(1)), items.items[0].edit);
    let Edit::Set(policy) = &items.items[0].edit else { panic!("policy") };
    assert!(policy.get("ceiling").is_some());
}
#[test]
fn a_size_only_project_policy_keeps_what_project_show_gives() {
    let mut outputs = outputs();
    let mut revision = value("revision-policy.json");
    revision["items"][0].as_object_mut().expect("item").remove("set");
    revision["items"][0]["too_large"] = json!(2_000_000);
    outputs.by_command.insert(words("revision show 5 --json"), revision.to_string());
    let (items, current) = collect::assemble(&outputs).expect("partial policy");
    assert_eq!(items.items[0].edit, edit(&current, "project-policy", None, Some(1)));
}
#[test]
fn a_projects_ways_and_questions_are_collected_from_their_own_reads() {
    let mut outputs = outputs();
    let inventory = json!({"newest":5,"settings":[
        {"setting":{"kind":"project-ways","project":1},"name":null,"follows":"change","last":5},
        {"setting":{"kind":"project-questions","project":1},"name":null,"follows":"change","last":5},
    ]});
    outputs.by_command.insert(words("settings show --json"), inventory.to_string());
    let revision = json!({"head":{"number":5},"items":[
        {"setting":{"kind":"project-questions","project":1},"set":"up","replaced":"not_offered"},
    ]});
    outputs.by_command.insert(words("revision show 5 --json"), revision.to_string());
    let ways = json!({"setting":{"kind":"project-ways","project":1},"revision":5,
        "value":{"drafts":true,"goals":true,"changes":true,"way":4},
        "application":{"follows":"change","revision":5,"size":7,"charters":[]}});
    outputs.by_command.insert(words("settings show project-ways 1 --json"), ways.to_string());
    let questions = json!({"setting":{"kind":"project-questions","project":1},"revision":5,"value":"up",
        "application":{"follows":"change","revision":5,"size":1,"charters":[]}});
    outputs.by_command.insert(words("settings show project-questions 1 --json"), questions.to_string());
    let (items, current) = collect::assemble(&outputs).expect("assemble");
    assert_eq!(items.items.len(), 1);
    assert_eq!(items.items[0].setting.kind, "project-questions");
    let way = current.items.iter().find(|item| item.setting.kind == "project-ways").expect("ways");
    assert!(matches!(&way.edit, Edit::Set(value) if value["way"] == json!(4)), "{way:?}");
}
struct EvidenceDisk {
    machine: Machine,
    root: Opened,
}

impl EvidenceDisk {
    fn new(items: &[File]) -> Self {
        let mut machine = Machine::new();
        let root = machine.lay(items);
        Self { machine, root }
    }

    fn list(&mut self, path: &Path) -> Result<Vec<Listed>, Refusal> {
        let dir = self.machine.open(self.root, path.as_os_str().as_encoded_bytes(), How::Directory)?;
        let entries = self.machine.list(dir, 128, 4096);
        self.machine.close(dir);
        entries
    }

    fn write(&mut self, path: &str, bytes: &[u8]) {
        self.write_new(Path::new(path), bytes).expect("fixture evidence");
    }
}

fn disk_error(why: Refusal) -> io::Error {
    match why {
        Refusal::NotFound => io::ErrorKind::NotFound.into(),
        Refusal::Exists => io::ErrorKind::AlreadyExists.into(),
        other => io::Error::other(format!("{other:?}")),
    }
}

impl EvidenceSink for EvidenceDisk {
    fn create_dir_all(&mut self, path: &Path) -> io::Result<()> {
        match self.machine.open(self.root, path.as_os_str().as_encoded_bytes(), How::Directory) {
            Ok(dir) => {
                self.machine.close(dir);
                Ok(())
            }
            Err(Refusal::NotFound) => self.create_dir(path),
            Err(why) => Err(disk_error(why)),
        }
    }

    fn check_entry(&mut self, path: &Path) -> io::Result<()> {
        let file =
            self.machine.open(self.root, path.as_os_str().as_encoded_bytes(), How::PathNoFollow).map_err(disk_error)?;
        self.machine.close(file);
        Ok(())
    }

    fn write_new(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let file = self
            .machine
            .open(self.root, path.as_os_str().as_encoded_bytes(), How::Create { mode: 0o600 })
            .map_err(disk_error)?;
        let written = self.machine.write(file, 0, bytes).map_err(disk_error);
        self.machine.close(file);
        written
    }

    fn create_dir(&mut self, path: &Path) -> io::Result<()> {
        let name = path.file_name().expect("directory name").as_encoded_bytes();
        let parent = path.parent().expect("evidence path");
        if parent.as_os_str().is_empty() {
            return self.machine.make_directory(self.root, name, 0o700).map_err(disk_error);
        }
        let dir =
            self.machine.open(self.root, parent.as_os_str().as_encoded_bytes(), How::Directory).map_err(disk_error)?;
        let made = self.machine.make_directory(dir, name, 0o700).map_err(disk_error);
        self.machine.close(dir);
        made
    }
}

impl EvidenceSource for EvidenceDisk {
    fn read_text(&mut self, path: &Path) -> io::Result<String> {
        let file = self.machine.open(self.root, path.as_os_str().as_encoded_bytes(), How::Read).map_err(disk_error)?;
        let size = u32::try_from(self.machine.stat(file).size).expect("fixture length");
        let bytes = self.machine.read(file, 0, size).map_err(disk_error);
        self.machine.close(file);
        String::from_utf8(bytes?).map_err(io::Error::other)
    }

    fn read_documents(
        &mut self,
        dir: &Path,
        _out: &mut BTreeMap<String, String>,
        _recursive: bool,
    ) -> Result<(), evidence::EvidenceError> {
        assert_eq!(self.list(dir).expect("collected document directory"), [] as [Listed; 0]);
        Ok(())
    }
}

struct Script {
    replies: CtlOutputs,
    calls: Vec<String>,
    fail: Option<&'static str>,
}

impl Script {
    fn new() -> Self {
        let mut replies = CtlOutputs { project: 1, revision: 5, ..CtlOutputs::default() };
        for (call, _, text) in READS.into_iter().chain([
            transcript!("proposal show t41/7", "proposal.txt"),
            transcript!("chat show 41", "chat-latest.txt"),
            transcript!("chat show 41 --before aabb", "chat-middle.txt"),
            transcript!("chat show 41 --before ccdd", "chat-first.txt"),
        ]) {
            replies.by_command.insert(words(call), text.into());
        }
        Self { replies, calls: Vec::new(), fail: None }
    }

    fn collect(&mut self, args: &CollectArgs, disk: &mut EvidenceDisk) -> Result<(), collect::CollectError> {
        testing::collect_with(args, disk, &mut |command| {
            assert_eq!(command.get_program(), args.temper.as_os_str());
            let mut arguments = command.get_args();
            assert_eq!(arguments.next(), Some(std::ffi::OsStr::new("ctl")));
            assert_eq!(arguments.next(), Some(args.ctl.as_os_str()));
            let words: Vec<_> = arguments.map(|arg| arg.to_str().expect("ctl word").to_owned()).collect();
            let call = words.join(" ");
            let (status, stdout, stderr) = if self.fail == Some(call.as_str()) {
                (2, Vec::new(), b"read refused\nsecond line\n".to_vec())
            } else {
                let reply = self.replies.by_command.get(&words).unwrap_or_else(|| panic!("unexpected read: {call}"));
                (0, reply.as_bytes().to_vec(), Vec::new())
            };
            self.calls.push(call);
            Ok(Output { status: ExitStatus::from_raw(status << 8), stdout, stderr })
        })
    }
}

fn collect_args() -> CollectArgs {
    CollectArgs::parse(&arguments(Path::new("evidence"))).expect("collect arguments")
}

#[test]
fn collect_writes_evidence_through_a_fake_temper() {
    let args = collect_args();
    let mut disk = EvidenceDisk::new(&[]);
    let mut script = Script::new();
    script.collect(&args, &mut disk).expect("collect");
    disk.write("evidence/confirmed.json", br#"{"box":[]}"#);
    let evidence = testing::read_with(&args.evidence, &mut disk).expect("read collected evidence");
    let (items, current) = collect::assemble(&script.replies).expect("assemble");
    assert_eq!(evidence.items, items);
    assert_eq!(evidence.current, current);
    assert_eq!(evidence.facts.project, 1);
    assert_eq!(evidence.facts.proposal, "t41/7");
    assert_eq!(evidence.proposal, include_str!("fixtures/ctl/proposal.txt"));
    assert_eq!(
        evidence.chat,
        "Person: Configure the workflow from the process documents.\nPerson: Keep running work as it is.\nAgent: The settings are accepted.\n"
    );
    assert_eq!(
        disk.read_text(&args.evidence.join("revision.json")).expect("revision"),
        include_str!("fixtures/ctl/revision.json")
    );
    for name in ["documents", "person"] {
        assert_eq!(disk.list(&args.evidence.join(name)).expect("empty directory").len(), 0);
    }
    let calls = script.calls.join("\n") + "\n";
    assert!(calls.starts_with("revision show 5 --json\nsettings show charter 6 --revision 5 --json\nsettings show landing-rule 2 --project 1 --json\nsettings show forge-charter 1 --json\nsettings show --json\n"));
    assert!(
        calls.ends_with("proposal show t41/7\nchat show 41\nchat show 41 --before aabb\nchat show 41 --before ccdd\n")
    );
}
#[test]
fn a_failing_ctl_call_is_named() {
    let args = collect_args();
    let mut disk = EvidenceDisk::new(&[]);
    let mut script = Script::new();
    script.fail = Some("settings show --json");
    let error = script.collect(&args, &mut disk).expect_err("failed ctl").to_string();
    assert!(error.contains(&format!("{} ctl ctl-dir settings show --json", args.temper.display())), "{error}");
    assert!(error.contains("read refused"), "{error}");
    assert!(!error.contains("second line"), "{error}");
    assert_eq!(
        disk.check_entry(&args.evidence.join("items.json")).expect_err("no output").kind(),
        io::ErrorKind::NotFound
    );
}
#[test]
fn existing_evidence_is_refused_before_any_read_but_confirmations_are_kept() {
    let args = collect_args();
    let mut script = Script::new();
    for name in
        ["box.json", "revision.json", "items.json", "current.json", "proposal.txt", "chat.txt", "documents", "person"]
    {
        let path = args.evidence.join(name);
        let mut disk = EvidenceDisk::new(&[
            File::directory(b"evidence"),
            File::file(path.as_os_str().as_encoded_bytes(), b"keep"),
        ]);
        let error = script.collect(&args, &mut disk).expect_err("refuse overwrite").to_string();
        assert!(error.contains(name), "{error}");
        assert_eq!(disk.read_text(&path).expect("retained"), "keep");
        assert_eq!(script.calls, [] as [String; 0]);
    }
    let mut disk =
        EvidenceDisk::new(&[File::directory(b"evidence"), File::file(b"evidence/confirmed.json", br#"{"box":[]}"#)]);
    script.collect(&args, &mut disk).expect("existing confirmation permitted");
    assert!(testing::read_with(&args.evidence, &mut disk).is_ok());
}
#[test]
fn malformed_and_mismatched_ctl_json_is_named() {
    for (call, replacement) in [
        ("revision show 5 --json", "{"),
        ("revision show 5 --json", r#"{"unknown":true}"#),
        ("settings show --json", r#"{"settings":null}"#),
        ("settings show charter 3 --json", "{}"),
        ("settings show charter 3 --json", r#"{"setting":{"kind":"charter","number":4},"value":{}}"#),
        ("settings show charter 3 --json", r#"{"setting":{"kind":"charter","number":3},"value":null}"#),
        ("project show 1 --json", r#"{"number":2}"#),
    ] {
        let mut outputs = outputs();
        outputs.by_command.insert(words(call), replacement.into());
        let error = collect::assemble(&outputs).expect_err("bad ctl JSON").to_string();
        assert!(error.contains(call), "{error}");
    }
}
fn arguments(evidence: &Path) -> Vec<String> {
    let mut args = words(
        "--temper temper --ctl ctl-dir --project 1 --revision 5 --proposal t41/7 --chat 41 --forge code.example:3000 --repository team/rules",
    );
    args.push(evidence.to_str().expect("evidence path").into());
    args
}
#[test]
fn collect_requires_each_flag_once_and_one_evidence_directory() {
    let args = arguments(Path::new("evidence"));
    let parsed = CollectArgs::parse(&args).expect("all flags");
    assert_eq!(parsed.project, 1);
    assert_eq!(parsed.evidence, Path::new("evidence"));
    for i in (0..16).step_by(2) {
        let mut missing = args.clone();
        missing.drain(i..i + 2);
        assert!(CollectArgs::parse(&missing).expect_err("missing flag").to_string().contains(&args[i]));
        let mut duplicate = args.clone();
        duplicate.extend_from_slice(&args[i..i + 2]);
        assert!(CollectArgs::parse(&duplicate).expect_err("duplicate flag").to_string().contains(&args[i]));
    }
    for invalid in ["0", "-1", "4294967296", "number"] {
        let mut args = args.clone();
        args[5] = invalid.into();
        assert!(CollectArgs::parse(&args).expect_err("invalid number").to_string().contains("--project"));
    }
    assert!(CollectArgs::parse(&args[..16]).is_err());
    let mut extra = args.clone();
    extra.push("extra".into());
    assert!(CollectArgs::parse(&extra).is_err());
    extra = args;
    extra.extend(words("--unknown value"));
    assert!(CollectArgs::parse(&extra).is_err());
}
#[test]
fn a_repeated_chat_cursor_is_refused() {
    let args = collect_args();
    let mut disk = EvidenceDisk::new(&[]);
    let mut script = Script::new();
    script.replies.by_command.insert(
        words("chat show 41 --before aabb"),
        "Agent: Same page.\nread on with: chat show 41 --before aabb\n".into(),
    );
    let error = script.collect(&args, &mut disk).expect_err("cursor cycle").to_string();
    assert!(error.contains("chat show 41 --before aabb: repeated chat cursor"), "{error}");
    assert_eq!(
        disk.check_entry(&args.evidence.join("chat.txt")).expect_err("no output").kind(),
        io::ErrorKind::NotFound
    );
}
#[test]
fn current_keeps_an_unset_temper_default_as_reset() {
    let mut outputs = outputs();
    let mut setting = value("landing-rule.json");
    setting["value"] = Value::Null;
    outputs.by_command.insert(words("settings show landing-rule 1 --json"), setting.to_string());
    let (_, current) = collect::assemble(&outputs).expect("unset default");
    assert_eq!(edit(&current, "landing-rule", Some(1), None), Edit::Reset);
}

#[test]
fn one_accepted_revision_retains_the_designer_and_triage_bindings() {
    let scratch = Scratch::new("stage-one-note-proposal");
    super::check::profile_evidence(scratch.path(), |_: &mut Vec<crate::golden::Item>| {});
    let mut evidence = evidence::read(scratch.path()).expect("reference evidence");
    let accepted = evidence.items.clone();
    let rows: Vec<_> = accepted
        .items
        .iter()
        .map(|item| {
            let Edit::Set(value) = &item.edit else { panic!("accepted whole values") };
            json!({"setting":item.setting,"set":value})
        })
        .collect();
    let (mut recorded, _) = project_settings_outputs(evidence.facts.project, &[], &[]);
    recorded.revision = 5;
    recorded.by_command.insert(words("revision show 5 --json"), json!({"head":{"number":5},"items":rows}).to_string());
    recorded.by_command.insert(words("settings show --json"), json!({"settings":[]}).to_string());
    let (items, _) = collect::assemble(&recorded).expect("one complete accepted revision");
    assert_eq!(items, accepted, "the revision retains every submitted item");
    evidence.items = items;
    let golden = crate::golden::reference().expect("reference golden");
    let profile = crate::golden::profile().expect("reference profile");
    let report = crate::check::check(&golden, &profile, &evidence);
    for (role, number) in [("designer", 7), ("triage", 8)] {
        assert!(report.binding.iter().any(|binding| binding.role == role && binding.number == Some(number)));
    }
    // Independent acceptances leave only the last proposal's edits in its revision.
    for (at, row) in rows.iter().enumerate() {
        let number = at + 3;
        recorded.revision = u32::try_from(number).expect("fixture revision");
        recorded.by_command.insert(
            words(&format!("revision show {number} --json")),
            json!({"head":{"number":number},"items":[row]}).to_string(),
        );
    }
    let (incomplete, _) = collect::assemble(&recorded).expect("the final workaround revision alone");
    assert_eq!(incomplete.items.len(), 1, "current settings cannot fill unrelated accepted edits");
    evidence.items = incomplete;
    let report = crate::check::check(&golden, &profile, &evidence);
    for role in ["designer", "triage"] {
        assert!(
            report.binding.iter().any(|binding| binding.role == role && binding.by == crate::check::BoundBy::Unbound)
        );
    }
}
