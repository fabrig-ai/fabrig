//! The reference's settings file (temper/docs/design/protocol/configuration.md, 13.9).
use settings_form::view::{
    CharterDocument, ResultsWake, Setting, SettingEdit, SettingItem, SettingNumber, SettingValue, TemplateExecutor,
    WakePolicy, WakeRule,
};
use skein_lib::{Duration, List, bytes};

const COORDINATOR: u32 = 5;
const DESIGNER: u32 = 6;
const CODER: u32 = 2;
const GOAL: u32 = 4;
// The reference supplies three charters and one template.
const ITEMS: u32 = 4;
// temper-init/template/agent.json's profile.policy.max_turns.
const TURNS: u32 = 16;

fn value(items: &List<SettingItem>, setting: Setting) -> &SettingValue {
    for item in items {
        if item.setting == setting {
            let SettingEdit::Set(set) = &item.edit else { panic!("reference sets its settings") };
            return &set.value;
        }
    }
    panic!("missing reference setting {setting:?}");
}

fn documents(documents: &List<CharterDocument>, first_path: &[u8], first_heading: Option<&[u8]>, second_path: &[u8]) {
    assert_eq!(documents.len(), 2);
    for (document, (path, heading)) in
        documents.iter().zip([(first_path, first_heading), (second_path, Some(b"1. In one page".as_slice()))])
    {
        assert_eq!(document.path.as_ref(), path);
        assert_eq!(document.heading.as_deref(), heading);
    }
}

#[test]
fn the_reference_settings_file_decodes() {
    let items = settings_form::file::decode(
        include_bytes!("../../../../docs/examples/reference/settings.json"),
        &temper_client_tools::limits::form(),
    )
    .expect("the reference decodes as a ctl settings file");
    assert_eq!(items.len(), ITEMS);
    for (number, model) in [
        (COORDINATOR, b"claude-opus-5-5".as_slice()),
        (DESIGNER, b"claude-opus-5-5".as_slice()),
        (CODER, b"gpt-6.1-sol".as_slice()),
    ] {
        let SettingValue::Charter(charter) = value(&items, Setting::Charter(SettingNumber { number })) else {
            panic!("reference charter")
        };
        assert_eq!(charter.number, number);
        assert_eq!(charter.model.name.bytes(), model);
        assert!(charter.alternatives.is_empty());
        assert_eq!(charter.hosts, settings_form::view::SettingHosts::Workers);
        assert_eq!(charter.turns, TURNS);
        assert_eq!(charter.time, Duration::from_secs(1800));
        if number != CODER {
            assert!(!charter.resume());
            assert_eq!(charter.waiting, Duration::ZERO);
        }
    }
    let SettingValue::Charter(coordinator) = value(&items, Setting::Charter(SettingNumber { number: COORDINATOR }))
    else {
        panic!("coordinator charter")
    };
    documents(
        &coordinator.documents,
        b"AGENTS.md",
        Some(b"1. In one page"),
        b"skein/docs/foundation/programming-model.md",
    );
    assert!(bytes::find(coordinator.instructions.bytes(), b"\"producer\": 6").is_some());
    assert!(bytes::find(coordinator.instructions.bytes(), b"close-out").is_some());
    assert_eq!(
        coordinator.wake,
        WakePolicy {
            words: WakeRule::Immediate,
            notices: WakeRule::Immediate,
            news: WakeRule::Paced { span: Duration::from_secs(1800) },
            results: ResultsWake::Each,
            questions: true,
            answers: true,
            timers: true,
        }
    );
    let SettingValue::Charter(designer) = value(&items, Setting::Charter(SettingNumber { number: DESIGNER })) else {
        panic!("designer charter")
    };
    assert_eq!(designer.model, coordinator.model);
    assert!(designer.inspect());
    assert!(designer.modify());
    assert!(!designer.shell());
    assert!(designer.agents());
    assert_eq!(designer.reserve_turns, coordinator.reserve_turns);
    assert_eq!(designer.reserve_time, coordinator.reserve_time);
    assert_eq!(designer.call_timeout, coordinator.call_timeout);
    assert_eq!(designer.spend, coordinator.spend);
    assert_eq!(designer.wake, WakePolicy::DEFAULT);
    documents(&designer.documents, b"DESIGN.md", None, b"AGENTS.md");
    assert!(bytes::find(designer.instructions.bytes(), b"Not yet").is_some());
    let SettingValue::Charter(coder) = value(&items, Setting::Charter(SettingNumber { number: CODER })) else {
        panic!("coding charter")
    };
    assert_eq!(coder.wake, WakePolicy::DEFAULT);
    assert!(coder.documents.is_empty());
    assert!(coder.instructions.bytes().starts_with(include_bytes!("../../../temper-service/src/stock/coding.txt")));
    let SettingValue::Template(goal) = value(&items, Setting::Template(SettingNumber { number: GOAL })) else {
        panic!("goal template")
    };
    assert_eq!(goal.number, GOAL);
    assert_eq!(goal.executor, TemplateExecutor::Charter(COORDINATOR));
    assert_eq!(goal.name.bytes(), b"goal for a draft");
    assert_eq!(goal.tracked, Some(1));
    assert_eq!(goal.label, 2);
    let llm = goal.llm.as_ref().expect("goal model choice");
    assert_eq!(llm.main.model.bytes(), b"claude-opus-5-5");
    assert_eq!(llm.main.effort.as_ref().expect("coordinator effort").bytes(), b"xhigh");
}
