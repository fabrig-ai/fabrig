//! A close refused while the pull is open (forge/change/README.md, 4).
use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use temper_engine_domain_forge as top;
use temper_engine_forge_world::{World, changes};

#[test]
fn a_refused_close_of_an_open_pull_holds_with_words() {
    let mut world = World::new(93);
    world.adopt();
    world.produce(b"temper/51");
    let pull = world.open_pull(b"temper/51", b"main");
    let mut row = changes::change_row();
    row.pull = Some(pull);
    world.event(top::Event::Change { row });
    // The pull belongs to user 2; the connector's user 1 cannot close it with read permission.
    world.permission(raw::Permission::Read);
    world.take_seen();
    world.take_calls();
    world.event(top::Event::Release {
        task: 51,
        root: 51,
        ending: top::ReleaseEnding::Cancelled,
        reason: None,
        entry: 10,
    });
    world.event(top::Event::Committed { entry: 10 });
    // The fake answers within this five-second horizon, with no pending comment before the close.
    world.run_for(5);
    assert!(
        world
            .take_calls()
            .iter()
            .any(|op| matches!(op, api::Op::Write(api::Write::Close { number }) if *number == pull))
    );
    let raw::Answer::Pull(found) = world.inspect(&raw::Read::Pull { number: pull }) else { panic!("pull") };
    assert_eq!(found.state, raw::State::Open);
    let Some(top::Stored::Release(release)) = world.stored().get(&top::Key::Release(51)) else { panic!("release") };
    assert!(release.failed, "the refused close holds cleanup");
    let account = world
        .seen()
        .iter()
        .find_map(|request| match request {
            top::Request::ReleaseFailed { task: 51, account, .. } => account.as_ref(),
            _ => None,
        })
        .expect("a refused close holds with an account");
    super::words_name(&account.words, &["close", "permission"]);
    assert!(String::from_utf8_lossy(&account.words).contains(&pull.to_string()), "{:?}", account.words);
}
