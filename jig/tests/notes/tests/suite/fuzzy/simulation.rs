use jig_domain_notes::{Event, LineRange, Scope};
use jig_notes_world::{World, one_scope, task_entry};
use skein_lib::{Rng, Token};
use skein_world::{
    domain::assert_replays,
    sweep::{Case, cases, gate},
    this_test,
};
use std::convert::Infallible;

fn draw_project(rng: &mut Rng) -> u32 {
    if rng.chance(500) { 7 } else { 8 }
}

fn run(seed: u64) -> (Vec<String>, std::collections::BTreeMap<jig_domain_notes::Key, jig_domain_notes::Record>) {
    let mut rng = Rng::new(seed);
    let mut world = World::default();
    for turn in 0..12u64 {
        let project = draw_project(&mut rng);
        let scope = Scope::Project { project };
        let name = turn.checked_add(100).expect("turn fits");
        let _answer = world.call(Event::Write {
            owner: Token::new(name),
            entry: task_entry(name, scope.clone(), b"warm service", b"wait for warmup", 9),
            recalled: None,
        });
        let _range = world.call(Event::WriteLines {
            owner: Token::new(name.checked_add(500).expect("token fits")),
            entry: task_entry(name, scope.clone(), b"warm service", b"then check", 9),
            recalled: 1,
            range: LineRange { first: if rng.chance(500) { 2 } else { 3 }, remove: 0, join: false },
        });
        let _index = world.call(Event::Index {
            owner: Token::new(name.checked_add(1000).expect("token fits")),
            scopes: one_scope(scope),
            most: 2,
        });
        if rng.chance(250) {
            world.restart();
        }
    }
    let project = draw_project(&mut rng);
    let _dropped = world.call(Event::DropProject { owner: Token::new(2000), project });
    world.restart();
    let _index =
        world.call(Event::Index { owner: Token::new(2001), scopes: one_scope(Scope::Project { project }), most: 2 });
    (world.trace().to_vec(), world.records().clone())
}

fn comparison_seed(seed: u64) -> u64 {
    const PINNED_PAIR_OFFSET: u64 = 100; // The gate's pinned pairs are 100 seeds apart.
    let mut other = seed.wrapping_add(PINNED_PAIR_OFFSET);
    if !gate() {
        // The first write logs its project. Draw again until that write differs.
        let project = draw_project(&mut Rng::new(seed));
        let mut rng = Rng::new(seed);
        while draw_project(&mut Rng::new(other)) == project {
            other = rng.next_u64();
        }
    }
    other
}

#[test]
fn seeded_writes_pages_and_restarts_replay_the_same_durable_entries() -> Result<(), Case<Infallible>> {
    cases(this_test!("seeded_writes_pages_and_restarts_replay_the_same_durable_entries"), 1..17, |seed| {
        let trace = assert_replays(seed, comparison_seed(seed), run);
        assert!(!trace.is_empty(), "seed {seed} exercised the world");
        Ok(())
    })
}
