use smith_local_process_world::fixture::contains;
use smith_local_process_world::{Files, Lending, World, referee::Ending};

#[test]
fn an_env_account_lends_its_variable() {
    let files = Files::new();
    let mut world = World::lending(44, &files, Lending::Env);
    world.settle();
    assert!(!world.queries().is_empty(), "the inherited variable's value reaches the provider");
    world.assert_ended(Ending::Report(b"First answer".to_vec()));
}

#[test]
fn a_key_account_lends_its_key_to_the_run() {
    let files = Files::new();
    let mut world = World::lending(45, &files, Lending::Key);
    world.settle();
    assert!(!world.queries().is_empty(), "the key grant reaches the provider");
    assert!(!contains(world.shown(), b"key-for-work"), "no key in output");
    world.assert_ended(Ending::Report(b"First answer".to_vec()));
}
