//! Local actor admission against the finishing reserve (smith/docs/design/domain/run.md, 9.1).

use smith_local_domain::{ExitStatus, Failure, Notice};
use smith_local_world::World;

// The local world's charter keeps two turns for main's finishing reserve.
const RESERVE_TURNS: u32 = 2;
// The shipped one-turn story leaves less than that reserve.
const ONE_TURN: u32 = 1;
// Admission needs an ordinary turn above the finishing reserve.
const ADMITTED_TURNS: u32 = RESERVE_TURNS + 1;
// The existing headless admission fixtures allow 100 boundary rounds to settle.
const ADMISSION_ROUNDS: u32 = 100;

fn run(seed: u64, turns: u32) -> World {
    let mut world = World::headless_with_turns(seed, b"A refused run", turns);
    assert!(world.drive_to_exit(ADMISSION_ROUNDS), "seed {seed}: the headless host settles");
    world
}

#[test]
fn a_one_turn_budget_refuses_before_any_provider_completion() {
    for turns in [ONE_TURN, RESERVE_TURNS] {
        let world = run(63, turns);
        assert_eq!(world.exit(), Some(ExitStatus::Refused));
        assert_eq!(world.answers(), 1, "one typed actor terminal");
        assert_eq!(
            world.notices().iter().filter(|notice| matches!(notice, Notice::Failure(Failure::Refused(_)))).count(),
            1,
            "one typed refusal notice"
        );
        let refusals: Vec<_> =
            world.shown().iter().map(AsRef::as_ref).filter(|text: &&[u8]| text.starts_with(b"run refused: ")).collect();
        assert_eq!(
            refusals,
            [b"run refused: invalid: budget_reserve_turns\n".as_slice()],
            "the reserve refusal names its part and invents no provider prose"
        );
        assert_eq!(world.completions(), 0, "reserve refusal starts no provider request");
        assert!(world.prompt_texts().is_empty());
    }
}

#[test]
fn a_budget_above_the_reserve_starts_a_provider_completion() {
    let world = run(64, ADMITTED_TURNS);
    assert!(world.completions() > 0, "one ordinary turn above the reserve admits provider work");
}
