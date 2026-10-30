use jig_tasks_world::{run_story, run_story_choices, stories::Story};

// Stable draw stream for timing/price and fault/cut variations after exhaustive choices.
const DRAW_SOURCE: u64 = 47;
const DRAWN_SEEDS: usize = 8;

#[test]
fn random_faults_and_restart_cuts_reach_every_retained_ending() {
    let mut seen = std::collections::BTreeSet::new();
    let markers = jig_tasks_world::coverage::MARKERS;
    let choices = Story::all();
    assert_eq!(choices.iter().copied().collect::<std::collections::BTreeSet<_>>().len(), 48);
    for (index, choice) in choices.into_iter().enumerate() {
        let timing_seed = u64::try_from(index).expect("48 choice timings");
        let (trace, frozen) = run_story_choices(timing_seed, choice);
        assert_eq!(frozen.results.len(), 5, "seed {timing_seed}, choice {choice:?}");
        seen.extend(trace.coverage.0);
    }
    let mut random = skein_lib::Rng::new(DRAW_SOURCE);
    let mut drawn = std::collections::BTreeSet::new();
    while drawn.len() < DRAWN_SEEDS {
        drawn.insert(random.below(256));
    }
    eprintln!("lifecycle drawn seeds={drawn:?}");
    for seed in drawn {
        let (trace, frozen) = run_story(seed);
        assert_eq!(frozen.results.len(), 5, "seed {seed}");
        seen.extend(trace.coverage.0);
    }
    for marker in markers {
        assert!(seen.contains(&marker), "sweep reaches {marker:?}");
    }
}

#[test]
fn selected_starts_keep_drawn_choices_and_name_the_first_removed_effort() {
    use jig_domain_tasks::{
        self as tasks, Domain, Event, Key, LlmChoice, ModelEffort, Party, RestoreCause, RestoreFailure, RestoreLlmWhy,
    };
    use jig_tasks_world::{EFFORTS, LIMITS, Reply, World, charters, task};
    use skein_lib::{Queue, Rng};
    const CHOICE_SEEDS: [u64; 8] = [131, 241, 353, 467, 571, 683, 797, 911];
    for seed in CHOICE_SEEDS {
        let mut rng = Rng::new(seed);
        let number = 10 + rng.below(100);
        let chosen = ModelEffort { model: 0, effort: Some(u8::try_from(rng.below(2)).expect("two efforts")) };
        let subagent = ModelEffort { model: 1, effort: None };
        let choice = LlmChoice { account: None, main: chosen, subagents: Box::new([subagent]) };
        let mut member = task(number, &[]);
        member.authority.models = tasks::Models::Named(0b11);
        member.llm = Some(choice.clone());
        let mut world = World::new_start_restore(seed, LIMITS);
        assert_eq!(world.make(Party::Person(9), vec![member]), Reply::Made(vec![number]));
        world.restart();
        assert_eq!(world.record(number).llm, Some(choice));

        let mut efforts = EFFORTS;
        efforts[0] = 0;
        let mut restored = Domain::new(&LIMITS, seed, 1, charters(), Box::new(efforts));
        restored.start_restore();
        let mut out = Queue::uncapped();
        for record in world.records.values().filter(|row| row.key().live()) {
            tasks::step(&mut restored, &world.env, Event::Restore { record: record.clone() }, &mut out);
        }
        assert!(out.is_empty());
        tasks::step(&mut restored, &world.env, Event::Check, &mut out);
        assert!(matches!(out.pop(), Some(tasks::Request::RestoreRefused { .. })));
        assert!(out.is_empty());
        let failure = Some(RestoreFailure {
            key: Key::Live(number),
            cause: RestoreCause::LlmChoice { chosen, why: RestoreLlmWhy::Effort },
        });
        assert_eq!(restored.restore_failure(), failure, "seed {seed}");
        tasks::step(&mut restored, &world.env, Event::Restored, &mut out);
        assert!(!restored.checked());
        assert!(out.is_empty());
        assert_eq!(restored.restore_failure(), failure);
    }
}
