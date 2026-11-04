//! The root scripts settings decisions across restarts (jig/docs/design/domain/connectors.md, section 17).
use jig_test_connector::{
    Event, RecordKey, Request, SettingDefault, SettingEdit, SettingItem, SettingJudged, SettingsKind,
};
use jig_test_connector_world::{LIMITS, World, config};
use skein_lib::Token;

const KIND: u32 = 41;
const TOKEN: Token = Token::new(1);
fn world() -> World {
    let mut configuration = config(8);
    configuration.settings = Some(SettingsKind {
        kind: KIND,
        project: false,
        value_bytes: 32,
        per_scope: 4,
        charters: false,
        defaults: Box::new([(0, 1, Box::from(&b"default"[..]))]),
    });
    World::with_config(8, configuration, LIMITS)
}
fn item(number: u32, value: &[u8]) -> SettingItem {
    SettingItem { kind: KIND, scope: 0, number, edit: SettingEdit::Set(Box::from(value)) }
}
fn judge(proposal: Option<u64>, items: Box<[SettingItem]>) -> Event {
    Event::JudgeSettings { token: TOKEN, revision: 9, proposal, items }
}
fn read(number: u32) -> Event {
    Event::ReadSetting { token: TOKEN, kind: KIND, scope: 0, number }
}

#[test]
fn settings_change_is_judged_kept_and_read_after_restart() {
    let mut world = world();
    assert_eq!(
        world.event(judge(None, Box::new([item(1, b"changed")]))),
        vec![Request::SettingsJudged {
            token: TOKEN,
            judged: Box::new([SettingJudged::Accept { size: 7, charters: Box::new([]) }])
        }]
    );
    let commits = world.commits;
    world.event(Event::KeepSettings { revision: 9, proposal: None, items: Box::new([item(1, b"changed")]) });
    assert_eq!(world.commits, commits + 1, "all settings writes belong to one decision");
    world.cold_restart();
    assert_eq!(
        world.event(read(1)),
        vec![Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"changed"[..])) }]
    );
    assert_eq!(
        world.event(judge(None, Box::new([item(1, b"changed")]))),
        vec![Request::SettingsJudged { token: TOKEN, judged: Box::new([SettingJudged::Same]) }]
    );
}

#[test]
fn settings_refusal_carries_its_words_and_keeps_nothing() {
    let mut world = world();
    assert_eq!(
        world.event(judge(None, Box::new([item(1, b"!not today")]))),
        vec![Request::SettingsJudged {
            token: TOKEN,
            judged: Box::new([SettingJudged::Refuse { why: Box::from(&b"refused: not today"[..]) }])
        }]
    );
    assert!(world.records.is_empty());
    world.cold_restart();
    assert_eq!(
        world.event(read(1)),
        vec![Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"default"[..])) }]
    );
}

#[test]
fn settings_proposal_acceptance_uses_kept_bytes_and_withdrawal_drops_them() {
    let mut world = world();
    world.event(Event::KeepProposalSettings { proposal: 11, items: Box::new([item(1, b"kept")]) });
    world.event(Event::KeepProposalSettings { proposal: 12, items: Box::new([item(2, b"withdrawn")]) });
    world.cold_restart();
    assert_eq!(
        world.event(judge(Some(11), Box::new([item(1, b"!ignored")]))),
        vec![Request::SettingsJudged {
            token: TOKEN,
            judged: Box::new([SettingJudged::Accept { size: 4, charters: Box::new([]) }])
        }]
    );
    let commits = world.commits;
    world.event(Event::KeepSettings { revision: 9, proposal: Some(11), items: Box::new([item(1, b"!ignored")]) });
    assert_eq!(world.commits, commits + 2, "one event saves the change and erases the proposal");
    assert!(!world.records.contains_key(&RecordKey::SettingProposal { proposal: 11, kind: KIND, scope: 0, number: 1 }));
    world.event(Event::DropProposalSettings { proposal: 12 });
    world.cold_restart();
    assert_eq!(
        world.event(read(1)),
        vec![Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"kept"[..])) }]
    );
    assert_eq!(world.event(read(2)), vec![Request::SettingValue { token: TOKEN, value: None }]);
    assert_eq!(
        world.event(judge(Some(12), Box::new([item(2, b"replacement")]))),
        vec![Request::SettingsJudged {
            token: TOKEN,
            judged: Box::new([SettingJudged::Refuse { why: Box::from(&b"proposal setting missing"[..]) }])
        }]
    );
}

#[test]
fn settings_restart_compares_changed_defaults_until_the_release_saves_them() {
    let mut world = world();
    world.event(Event::SaveSettingsDefaults { revision: None });
    world.configuration.settings.as_mut().expect("settings kind").defaults =
        Box::new([(0, 1, Box::from(&b"new default"[..]))]);
    world.cold_restart();
    let default = SettingDefault { kind: KIND, scope: 0, number: 1, size: 11, charters: Box::new([]), differs: true };
    assert_eq!(
        world.event(Event::SettingsDefaults),
        vec![Request::SettingsDefaults { defaults: Box::new([default.clone()]) }]
    );
    assert_eq!(
        world.event(read(1)),
        vec![Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"new default"[..])) }]
    );
    world.cold_restart();
    assert_eq!(
        world.event(Event::SettingsDefaults),
        vec![Request::SettingsDefaults { defaults: Box::new([default.clone()]) }]
    );
    world.event(Event::SaveSettingsDefaults { revision: Some(10) });
    world.cold_restart();
    assert_eq!(
        world.event(Event::SettingsDefaults),
        vec![Request::SettingsDefaults { defaults: Box::new([SettingDefault { differs: false, ..default }]) }]
    );
}
