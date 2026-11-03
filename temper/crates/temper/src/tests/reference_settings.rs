//! The reference's settings file (temper/docs/design/protocol/configuration.md, 13.9).
use settings_form::view::{
    ResultsWake, Setting, SettingEdit, SettingItem, SettingNumber, SettingValue, TemplateExecutor, WakePolicy, WakeRule,
};
use skein_lib::{Duration, List};

const COORDINATOR: u32 = 5;
const CODER: u32 = 2;
const FOREMAN: u32 = 6;
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
        (CODER, b"gpt-6.1-sol".as_slice()),
        (FOREMAN, b"gpt-6.1-sol".as_slice()),
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
    }
    let SettingValue::Charter(coordinator) = value(&items, Setting::Charter(SettingNumber { number: COORDINATOR }))
    else {
        panic!("coordinator charter")
    };
    let instructions =
        std::str::from_utf8(coordinator.instructions.bytes()).expect("coordinator instructions are UTF-8");
    assert!(instructions.contains(r#""branch": "mine""#));
    assert!(instructions.contains(r#""questions": "foreman""#));
    assert!(instructions.contains("decide"));
    assert!(coordinator.instructions.bytes().ends_with(
        b"If you finish without a landing, say why in your report; your goal branch is kept for a person to read."
    ));
    assert!(!coordinator.resume());
    assert_eq!(coordinator.waiting, Duration::ZERO);
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
    let SettingValue::Charter(coder) = value(&items, Setting::Charter(SettingNumber { number: CODER })) else {
        panic!("coding charter")
    };
    assert_eq!(coder.wake, WakePolicy::DEFAULT);
    assert!(coder.instructions.bytes().starts_with(include_bytes!("../../../temper-service/src/stock/coding.txt")));
    assert!(
        std::str::from_utf8(coder.instructions.bytes())
            .expect("coding instructions are UTF-8")
            .contains("in the question form")
    );
    let SettingValue::Charter(foreman) = value(&items, Setting::Charter(SettingNumber { number: FOREMAN })) else {
        panic!("foreman charter")
    };
    assert!(foreman.inspect());
    assert!(!foreman.modify());
    assert!(!foreman.shell());
    assert!(!foreman.agents());
    assert!(foreman.instructions.bytes().starts_with(include_bytes!("../../../temper-service/src/stock/foreman.txt")));
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
