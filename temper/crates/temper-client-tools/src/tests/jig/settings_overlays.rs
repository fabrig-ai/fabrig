//! Ctl overlays, using sets owned by these tests (client/ctl/overlays.md).
use super::*;
use crate::jig::call::SettingsFile;
use settings_form::overlay::{Directory, Markdown};
fn file(number: u32, overlay: &str) -> Vec<u8> {
    format!(r#"{{"items":[{{"setting":{{"kind":"charter","number":{number}}},"overlay":{overlay}}}]}}"#).into_bytes()
}
fn directory(path: &str, files: &[(&str, &str)]) -> Directory {
    Directory { path: path.as_bytes().into(), files: list(files.iter().map(|(name, text)| Markdown { name: name.as_bytes().into(), text: text.as_bytes().into() })) }
}
fn fixture(base: &str, path: &str) -> Directory {
    let root = std::path::Path::new("src/tests/fixtures/overlays").join(base).join(path);
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root).expect("fixture directory") {
        let entry = entry.expect("entry");
        if entry.path().extension().is_some_and(|e| e == "md") {
            files.push(Markdown { name: entry.file_name().as_encoded_bytes().into(), text: std::fs::read(entry.path()).expect("fixture Markdown").into_boxed_slice() });
        }
    }
    files.sort_by(|a,b| a.name.cmp(&b.name));
    Directory { path: path.as_bytes().into(), files: list(files) }
}
fn input(name: &str, bytes: &[u8], directories: Vec<Directory>) -> Value {
    Value::SettingsFile(Box::new(SettingsFile { name: name.as_bytes().into(), bytes: bytes.into(), directories: list(directories) }))
}
fn submit(files: Vec<Value>) -> (fixtures::Harness, u32) {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let mut call = fixtures::call(&["settings", "apply", "first.json", "--against", "5", "--key", "overlay"]);
    call.values[0] = Some(Values::Many(files.into_boxed_slice()));
    let number = root.submit_prepared(call, false, None);
    (root, number)
}
fn change(root: &fixtures::Harness, call: u32) -> &c::ChangeSettings {
    let (_, _, jig_client::Ask::Jig(ask)) = root.sends.get(&call).expect("settings sent") else { panic!("jig ask") };
    let c::Ask::ChangeSettings(change) = ask.as_ref() else { panic!("settings change") }; change
}
fn overlay(item: &c::SettingItem) -> &c::Overlay {
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set") };
    let c::SettingValue::Overlay(overlay) = &set.value else { panic!("overlay") }; overlay
}
#[test]
fn overlay_directories_assemble_in_order() {
    let (root, call) = submit(vec![input("settings.json", &file(2, r#"{"charters":["a","b"]}"#), vec![fixture("", "a"), fixture("", "b")])]);
    let value = overlay(change(&root, call).items.get(0).expect("implementer"));
    assert_eq!(value.instructions.bytes(), b"A\n\nI\n\nJ"); assert!(value.parts.is_empty());
    let (root, call) = submit(vec![input("settings.json", &file(5, r#"{"charters":["."]}"#), vec![fixture("deployment", ".")])]);
    let value = overlay(change(&root, call).items.get(0).expect("goal"));
    assert_eq!(value.instructions.bytes(), b"common\n\ngoal");
    assert_eq!(value.parts, list([c::CharterPart { on: 1, instructions: words("designer") }, c::CharterPart { on: 3, instructions: words("questions") }, c::CharterPart { on: 1 << 8, instructions: words("results") }]));
}
#[test]
fn overlay_inputs_refuse_bad_sets_and_shapes_without_sending() {
    for (files, expected) in [
        (vec![("notes.md", "notes")], "notes.md: unknown Markdown file"),
        (vec![("coordinator.md", "before\n## part, on: first\ntext")], "coordinator.md:1: text before first section"),
        (vec![("coordinator.md", "## part, on: \ntext")], "coordinator.md:1: empty reason"),
        (vec![("coordinator.md", "## part, on: first, first\ntext")], "coordinator.md:1: repeated reason first"),
        (vec![("coordinator.md", "## part, on: unknown\ntext")], "coordinator.md:1: unknown reason unknown"),
    ] {
        let (mut root, call) = submit(vec![input("settings.json", &file(2, r#"{"charters":["bad"]}"#), vec![directory("bad", &files)])]);
        assert!(root.sends.is_empty()); let answer = root.answer(call).expect("refused input");
        let text = core::str::from_utf8(&answer.stderr).expect("words");
        assert!(text.contains("settings file item 1: overlay.charters[0]: \"bad\":"), "{text}"); assert!(text.contains(expected), "{text}");
    }
    for (number, value, expected) in [
        (5, r#"{"charters":["a"],"instructions":"literal"}"#, "charters or instructions and parts, not both"),
        (5, r#"{"charters":["a"],"parts":[]}"#, "charters or instructions and parts, not both"),
        (3, r#"{"charters":["a"]}"#, "sets are charters 5 and 2's"),
        (5, r#"{"charters":[]}"#, "nonempty list required"),
        (5, r#"{"charters":["missing"]}"#, "\"missing\": directory: could not be read"),
    ] {
        let (mut root, call) = submit(vec![input("settings.json", &file(number, value), vec![])]);
        assert!(root.sends.is_empty()); let answer = root.answer(call).expect("refused");
        let text = core::str::from_utf8(&answer.stderr).expect("words"); assert!(text.contains(expected), "{text}");
    }
}
#[test]
fn literal_overlay_preserves_text_and_only_named_fields() {
    let bytes = file(5, r#"{"instructions":"\nLiteral\n","parts":[{"on":["first"],"instructions":"part"}],"shell":true,"documents":[],"turns":800}"#);
    let (root, call) = submit(vec![input("literal.json", &bytes, vec![])]);
    let v = overlay(change(&root, call).items.get(0).expect("literal"));
    assert_eq!(v.instructions.bytes(), b"\nLiteral\n"); assert_eq!(v.parts.get(0).expect("part").instructions.bytes(), b"part");
    assert_eq!((v.shell, v.turns), (Some(true), Some(800))); assert_eq!(v.documents, Some(list([])));
    assert_eq!(v.model, None); assert_eq!(v.resume, None); assert_eq!(v.spend, None); assert_eq!(v.agents, None);
    let json = setting_json(c::Setting::Charter(c::SettingNumber { number: 5 }), c::SettingValue::Overlay(Box::new(v.clone())));
    let json = core::str::from_utf8(&json).expect("JSON");
    for absent in ["model", "resume", "spend", "agents", "charters"] { assert!(!json.contains(&format!("\"{absent}\":")), "{json}"); }
}
#[test]
fn deployment_shaped_set_applies_both_roles_and_the_goal_template() {
    let bytes = include_bytes!("../fixtures/overlays/deployment/settings.json");
    let (mut root, call) = submit(vec![input("settings.json", bytes, vec![fixture("deployment", ".")])]);
    assert!(root.sends.contains_key(&call), "{:?}", root.answer(call));
    let items = &change(&root, call).items; assert_eq!(items.len(), 3);
    let goal = overlay(items.get(0).expect("goal")); let implementer = overlay(items.get(1).expect("implementer"));
    assert_eq!(goal.instructions.bytes(), b"common\n\ngoal"); assert_eq!(goal.parts.len(), 3);
    assert_eq!(implementer.instructions.bytes(), b"common\n\nimplementer"); assert!(implementer.parts.is_empty());
    assert_eq!(goal.model.as_ref().expect("goal model").name.bytes(), b"claude-opus-5-5");
    assert_eq!(goal.model.as_ref().expect("goal model").effort.as_ref().expect("effort").bytes(), b"xhigh");
    let sol = goal.alternatives.as_ref().expect("alternative").get(0).expect("Sol");
    assert_eq!(sol.name.bytes(), b"gpt-6-1-sol"); assert_eq!(sol.effort.as_ref().expect("effort").bytes(), b"high");
    assert_eq!(implementer.model.as_ref(), Some(sol));
    for (value, agents, resume, waiting) in [(goal, true, false, 0), (implementer, false, true, 10)] {
        assert_eq!((value.shell,value.agents,value.resume,value.waiting), (Some(true),Some(agents),Some(resume),Some(Duration::from_secs(waiting))));
        assert_eq!((value.turns,value.time,value.reserve_turns,value.reserve_time,value.spend), (Some(800),Some(Duration::from_secs(86400)),Some(1),Some(Duration::from_secs(300)),Some(1_000_000_000)));
    }
    let c::SettingEdit::Set(set) = &items.get(2).expect("template").edit else { panic!("set") };
    let c::SettingValue::Template(template) = &set.value else { panic!("template") };
    assert_eq!((template.number, template.executor), (4, c::TemplateExecutor::Charter(5)));
    let choice = template.llm.as_ref().expect("template choice");
    assert_eq!(choice.main.model.bytes(), b"claude-opus-5-5"); assert_eq!(choice.main.effort.as_ref().expect("effort").bytes(), b"xhigh");
    assert_eq!(choice.subagents.get(0).expect("subagent").model.bytes(), b"gpt-6-1-sol");
    assert_eq!(choice.later.as_ref().expect("later").main.effort.as_ref().expect("effort").bytes(), b"high");
}
#[test]
fn stacked_overlays_join_and_conflicts_name_both_files_and_items() {
    let a = file(5, r#"{"instructions":"A","shell":true,"turns":40,"parts":[{"on":["first"],"instructions":"a"}]}"#);
    let b = file(5, r#"{"instructions":"B","turns":80,"parts":[{"on":["question"],"instructions":"b"}]}"#);
    let (root, call) = submit(vec![input("a.json", &a, vec![]), input("b.json", &b, vec![])]);
    let items = &change(&root, call).items; assert_eq!(items.len(), 1);
    let v = overlay(items.get(0).expect("joined")); assert_eq!(v.instructions.bytes(), b"A\n\nB"); assert_eq!((v.shell,v.turns), (Some(true),Some(80)));
    assert_eq!(v.parts.iter().map(|p| p.instructions.bytes()).collect::<Vec<_>>(), vec![b"a".as_slice(),b"b"]);
    let whole = charter_file("[]");
    let whole = String::from_utf8(whole).expect("charter").replace("\"number\":3", "\"number\":5");
    for second in [whole.into_bytes(), br#"{"items":[{"setting":{"kind":"charter","number":5},"edit":"removed"}]}"#.to_vec(), br#"{"items":[{"setting":{"kind":"charter","number":5},"edit":"reset"}]}"#.to_vec()] {
        let (mut root, call) = submit(vec![input("a.json", &a, vec![]),input("b.json", &second, vec![])]);
        assert!(root.sends.is_empty()); let answer=root.answer(call).expect("refused"); let text=core::str::from_utf8(&answer.stderr).expect("words");
        assert!(text.contains("charter 5:") && text.contains("b.json item 1") && text.contains("an overlay in a.json item 1"), "{text}");
    }
    let reset = br#"{"items":[{"setting":{"kind":"rules"},"edit":"reset"}]}"#;
    let (mut root,call)=submit(vec![input("a.json",reset,vec![]),input("b.json",reset,vec![])]);
    assert!(root.sends.is_empty()); let answer=root.answer(call).expect("duplicate"); assert!(String::from_utf8_lossy(&answer.stderr).contains("reset in b.json item 1 beside reset in a.json item 1"));
    let (mut root,call)=submit(vec![input("a.json",&a,vec![]),input("bad.json",br#"{"items":[{"oops":true}]}"#,vec![])]);
    assert!(root.sends.is_empty()); let answer=root.answer(call).expect("bad file"); assert!(String::from_utf8_lossy(&answer.stderr).contains("settings file bad.json item 1: oops: unknown member"));
}
#[test]
fn shown_overlay_marks_and_literal_json_apply_back_and_inventory_origin() {
    let (root,call)=submit(vec![input("set.json",&file(5,r#"{"charters":["."],"shell":true,"turns":800}"#),vec![fixture("deployment",".")])]);
    let v=overlay(change(&root,call).items.get(0).expect("overlay")).clone();
    let (setting,c::SettingValue::Charter(mut charter))=values().remove(0) else {panic!("charter")};
    charter.resume=false; charter.shell=true; charter.turns=800;
    charter.instructions=words("Default\n\ncommon\n\ngoal");
    charter.parts=list([c::CharterPart{on:1,instructions:words("default part")}].into_iter().chain(v.parts.iter().cloned()));
    let document=c::ReadValue::Setting(Box::new(c::SettingDocument{setting,revision:5,value:Some(c::SettingValue::Charter(charter)),overlay:Some(Box::new(v.clone())),application:None,proposed:None}));
    let text=read(&["settings","show","charter","3"],document.clone());
    assert!(String::from_utf8_lossy(&text.stdout).contains("parts[1]: on first; 8 bytes; overlay\n"));
    golden("settings_charter_overlay",&text.stdout);
    let json=read(&["settings","show","charter","3","--json"],document);
    let json=core::str::from_utf8(&json.stdout).expect("JSON");
    assert!(json.contains("\"value\":"));assert!(!json.contains("charters"));
    let literal=json.rsplit_once(",\"overlay\":").expect("overlay export").1.strip_suffix('}').expect("document end");
    let (root,call)=submit(vec![input("copied.json",&file(3,literal),vec![])]);
    assert_eq!(overlay(change(&root,call).items.get(0).expect("copied")),&v);
    let inventory=c::ReadValue::Settings(Box::new(c::SettingsDocument{newest:5,settings:list([c::SettingLine{setting,name:None,follows:c::SettingFollows::Overlay,last:Some(5),application:None}]),held:list([]),pending:list([])}));
    let text=read(&["settings","show"],inventory.clone());assert!(String::from_utf8_lossy(&text.stdout).contains("charter 3: overlay\n"));
    let json=read(&["settings","show","--json"],inventory);assert!(String::from_utf8_lossy(&json.stdout).contains("\"follows\":\"overlay\""));
}
