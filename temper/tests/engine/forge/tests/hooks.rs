use skein_forge::api as client;
use temper_engine_forge_world::{World, translate};

#[test]
fn a_jobs_end_reaches_the_connector_as_a_hint_for_its_commit() {
    let mut world = World::hooked(23);
    world.adopt();
    world.produce(b"temper/51");
    world.run_for(1);
    let head = translate::commit(world.branch(b"temper/51"));
    let ended = world.hints().iter().filter(|hint| hint.change == client::Change::Commit(head)).count();
    assert_eq!(ended, 1, "one job end for the head's context, none for its pending report");
}

#[test]
fn an_issues_body_key_reaches_the_connector_in_its_hint() {
    let mut world = World::hooked(23);
    world.adopt();
    let number = world.issue(&translate::keyed(b"an issue", b"A task"));
    world.run_for(1);
    let hint = world.hints().iter().find(|hint| hint.change == client::Change::Item(number)).expect("the issue's hint");
    assert_eq!(hint.key.as_deref(), Some(&b"an issue"[..]));
}
