use jig_domain_tasks::{ClaimSources, Event, Guide, Key, Stored};
use jig_tasks_world::{LIMITS, PERSON_PARTY, Reply, World, task};
use skein_lib::Rng;

const TASKS: usize = 8;
const ATTEMPT: u64 = 1;
const SEEDS: u64 = 8;
const STEPS: u32 = 100;

#[test]
fn reports_follow_claims_and_ends_across_arbitrary_restart_cuts() {
    for seed in 1..=SEEDS {
        let mut world = World::new(seed, LIMITS);
        world.open_period(0, 10_000);
        assert!(matches!(world.make(PERSON_PARTY, (1..=TASKS as u64).map(|n| task(n, &[])).collect()), Reply::Made(_)));
        let mut draws = Rng::new(seed);
        let mut opened = [false; TASKS];
        let mut closed = [false; TASKS];
        let mut reported = [None; TASKS];
        for step in 0..STEPS {
            let index = usize::try_from(draws.next_u64() % TASKS as u64).expect("fixture index");
            let task = u64::try_from(index).expect("fixture task number") + 1;
            match draws.next_u64() % 5 {
                0 if !opened[index] => {
                    let sources =
                        ClaimSources { build: "seed-build".into(), settings: 1, source: Box::new([]), guide_limit: 1 };
                    assert_eq!(world.claim_prepared(task, ATTEMPT, 10, Box::new([]), Some(sources)), Reply::Done);
                    opened[index] = true;
                }
                1 if opened[index] && !closed[index] => {
                    world.finish(task);
                    closed[index] = true;
                }
                2 => world.restart(),
                op @ (3 | 4) => {
                    let attempt = if op == 3 { ATTEMPT } else { ATTEMPT + 1 };
                    let guide = Guide {
                        repository: "shop".into(),
                        commit: "abc".into(),
                        path: "AGENTS.md".into(),
                        digest: [0; 32],
                        bytes: step,
                        whole: true,
                    };
                    world.send(Event::RunGuides { task, attempt, guides: Box::new([guide]) });
                    if opened[index] && !closed[index] && attempt == ATTEMPT && reported[index].is_none() {
                        reported[index] = Some(step);
                    }
                }
                _ => {}
            }
            let key = Key::Attempt { task, attempt: ATTEMPT };
            if opened[index] {
                let Some(Stored::Attempt(row)) = world.records.get(&key) else {
                    panic!("seed {seed} step {step} missing attempt")
                };
                let guides = row.sources.as_ref().expect("prepared sources").guides.as_ref();
                assert_eq!(
                    guides.map(|guides| guides[0].bytes),
                    reported[index],
                    "seed {seed} step {step} task {task}"
                );
            } else {
                assert!(!world.records.contains_key(&key), "seed {seed} step {step} unclaimed attempt");
            }
            assert!(!world.records.contains_key(&Key::Attempt { task, attempt: ATTEMPT + 1 }));
        }
    }
}
