//! Always steps and groups built independently from their settings files.
use super::demo;
use crate::steps;
use jig_protocol_config as config;
use serde_json::Value;
use temper_forge_setup::{Kind, Stage, Step};
fn fake_entry(fixture: &Value) -> Value {
    let body = serde_json::to_vec(&fixture["application"]["demo"]["forge"]).unwrap();
    serde_json::from_slice(&temper_forge_setup::setup::group_settings(&body, 7).unwrap()).unwrap()
}
fn read_entry(value: &Value) -> temper_forge_setup::setup::Entry {
    let bytes = serde_json::to_vec(value).unwrap();
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    temper_forge_setup::setup::Entry::group(&bytes, &temper_forge_setup::limits::small(), &directories).unwrap()
}
fn group_flags() -> Box<[Box<[u8]>]> {
    ["--config", "C", "--data", "D", "--cache", "K"].iter().map(|flag| Box::from(flag.as_bytes())).collect()
}
#[test]
fn first_steps_precede_serve_without_any_groups() {
    let first = Step::new(b"fake model".as_slice().into(), Box::new([b"model".as_slice().into()]), Kind::Long);
    let commands = steps(&group_flags(), Box::new([]), Box::new([first.clone()]));
    assert_eq!(commands.always.len(), 2);
    assert_eq!(commands.always[0], first);
    assert_eq!(commands.always[1].name(), b"serve");
    assert!(commands.asked.is_empty());
}
#[test]
fn playground_group_is_recorded_and_runs_llm_prepare_forge() {
    let fixture = demo::fixture();
    let entry = read_entry(&fake_entry(&fixture));
    let temper_forge_setup::setup::Mode::Fake(fake) = &entry.mode else { panic!("fake") };
    let demo = demo::read(&fixture).unwrap().unwrap();
    let group = steps::playground_group(7, fake, demo.llm().listen(), &group_flags());
    assert!(group.recorded);
    assert_eq!(group.before, 0);
    assert_eq!(&*group.label, b"Playground");
    assert_eq!(
        group.steps.iter().map(|(stage, step)| (*stage, step.name())).collect::<Vec<_>>(),
        [(Stage::Run, b"fake llm".as_slice()), (Stage::Prepare, b"prepare 7"), (Stage::Run, b"fake forge 7")]
    );
    assert_eq!(group.steps[0].1.listeners(), &[demo.llm().listen()]);
    assert_eq!(group.steps[2].1.listeners(), &[fake.listen]);
}
#[test]
fn group_picks_by_mode() {
    let fixture = demo::fixture();
    let fake = read_entry(&fake_entry(&fixture));
    let owned: Value = serde_json::from_slice(include_bytes!("../../../../docs/examples/owned.json")).unwrap();
    let owned = read_entry(&owned["application"]["setup"][0]);
    let llm = "127.0.0.1:8370".parse().unwrap();
    for listens in [false, true] {
        let temper_forge_setup::setup::Mode::Owned(settings) = &owned.mode else { panic!("owned") };
        assert_eq!(
            steps::group(&owned, None, &group_flags(), listens),
            Some(temper_forge_setup::step::owned_group(owned.forge, settings, &group_flags(), listens))
        );
        let temper_forge_setup::setup::Mode::Fake(settings) = &fake.mode else { panic!("fake") };
        assert_eq!(
            steps::group(&fake, Some(llm), &group_flags(), listens),
            Some(steps::playground_group(fake.forge, settings, llm, &group_flags()))
        );
        assert_eq!(steps::group(&fake, None, &group_flags(), listens), None);
    }
    let mut managed = owned;
    for token in [None, Some(Box::from(b"/data/admin".as_slice()))] {
        managed.mode = temper_forge_setup::setup::Mode::Managed { admin_token: token };
        assert_eq!(steps::group(&managed, Some(llm), &group_flags(), true), None);
    }
}
