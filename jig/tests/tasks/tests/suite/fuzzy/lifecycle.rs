use jig_tasks_world::{PERSON_PARTY, run_story, run_story_choices, stories::Story};
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};
use std::{collections::BTreeSet, convert::Infallible};

// Stable draw stream for timing/price and fault/cut variations after exhaustive choices.
const DRAW_SOURCE: u64 = 47;
const DRAWN_SEEDS: usize = 8;

#[test]
fn random_faults_and_restart_cuts_reach_every_retained_ending() -> Result<(), Case<Infallible>> {
    macro_rules! part {
        ($story:literal) => {
            this_test!("random_faults_and_restart_cuts_reach_every_retained_ending", $story)
        };
    }

    // Parts follow Story::all()'s order and keep the same choices at every seed.
    let parts = [
        part!("staged-refused-skip-succeeded-skip"),
        part!("staged-refused-skip-succeeded-restore"),
        part!("staged-refused-skip-failed-skip"),
        part!("staged-refused-skip-failed-restore"),
        part!("staged-refused-restore-succeeded-skip"),
        part!("staged-refused-restore-succeeded-restore"),
        part!("staged-refused-restore-failed-skip"),
        part!("staged-refused-restore-failed-restore"),
        part!("staged-lost-skip-succeeded-skip"),
        part!("staged-lost-skip-succeeded-restore"),
        part!("staged-lost-skip-failed-skip"),
        part!("staged-lost-skip-failed-restore"),
        part!("staged-lost-restore-succeeded-skip"),
        part!("staged-lost-restore-succeeded-restore"),
        part!("staged-lost-restore-failed-skip"),
        part!("staged-lost-restore-failed-restore"),
        part!("staged-none-skip-succeeded-skip"),
        part!("staged-none-skip-succeeded-restore"),
        part!("staged-none-skip-failed-skip"),
        part!("staged-none-skip-failed-restore"),
        part!("staged-none-restore-succeeded-skip"),
        part!("staged-none-restore-succeeded-restore"),
        part!("staged-none-restore-failed-skip"),
        part!("staged-none-restore-failed-restore"),
        part!("committed-refused-skip-succeeded-skip"),
        part!("committed-refused-skip-succeeded-restore"),
        part!("committed-refused-skip-failed-skip"),
        part!("committed-refused-skip-failed-restore"),
        part!("committed-refused-restore-succeeded-skip"),
        part!("committed-refused-restore-succeeded-restore"),
        part!("committed-refused-restore-failed-skip"),
        part!("committed-refused-restore-failed-restore"),
        part!("committed-lost-skip-succeeded-skip"),
        part!("committed-lost-skip-succeeded-restore"),
        part!("committed-lost-skip-failed-skip"),
        part!("committed-lost-skip-failed-restore"),
        part!("committed-lost-restore-succeeded-skip"),
        part!("committed-lost-restore-succeeded-restore"),
        part!("committed-lost-restore-failed-skip"),
        part!("committed-lost-restore-failed-restore"),
        part!("committed-none-skip-succeeded-skip"),
        part!("committed-none-skip-succeeded-restore"),
        part!("committed-none-skip-failed-skip"),
        part!("committed-none-skip-failed-restore"),
        part!("committed-none-restore-succeeded-skip"),
        part!("committed-none-restore-succeeded-restore"),
        part!("committed-none-restore-failed-skip"),
        part!("committed-none-restore-failed-restore"),
    ];
    let mut seen = BTreeSet::new();
    let choices = Story::all();
    assert_eq!(choices.iter().copied().collect::<BTreeSet<_>>().len(), 48);
    assert_eq!(choices.len(), parts.len());
    for (index, (choice, test)) in choices.into_iter().zip(parts).enumerate() {
        let timing_seed = u64::try_from(index).expect("48 choice timings");
        cases(test, [timing_seed], |seed| {
            let (trace, frozen) = run_story_choices(seed, choice);
            assert_eq!(frozen.results.len(), 5, "seed {seed}, choice {choice:?}");
            seen.extend(trace.coverage.0);
            Ok(())
        })?;
    }
    let mut random = skein_lib::Rng::new(DRAW_SOURCE);
    let mut drawn = BTreeSet::new();
    while drawn.len() < DRAWN_SEEDS {
        drawn.insert(random.below(256));
    }
    eprintln!("lifecycle drawn seeds={drawn:?}");
    cases(part!("drawn"), drawn, |seed| {
        let (trace, frozen) = run_story(seed);
        assert_eq!(frozen.results.len(), 5, "seed {seed}");
        seen.extend(trace.coverage.0);
        Ok(())
    })?;
    if gate() {
        for marker in jig_tasks_world::coverage::MARKERS {
            assert!(seen.contains(&marker), "sweep reaches {marker:?}");
        }
    }
    Ok(())
}

#[test]
fn selected_starts_keep_drawn_choices_and_name_the_first_removed_effort() -> Result<(), Case<Infallible>> {
    use jig_domain_tasks::{
        self as tasks, Domain, Event, Key, LlmChoice, ModelEffort, RestoreCause, RestoreFailure, RestoreLlmWhy,
    };
    use jig_tasks_world::{EFFORTS, LIMITS, Reply, World, charters, task};
    use skein_lib::{Queue, Rng};
    const CHOICE_SEEDS: [u64; 8] = [131, 241, 353, 467, 571, 683, 797, 911];
    cases(this_test!("selected_starts_keep_drawn_choices_and_name_the_first_removed_effort"), CHOICE_SEEDS, |seed| {
        let mut rng = Rng::new(seed);
        let number = 10 + rng.below(100);
        let chosen = ModelEffort { model: 0, effort: Some(u8::try_from(rng.below(2)).expect("two efforts")) };
        let subagent = ModelEffort { model: 1, effort: None };
        let choice = LlmChoice { later: None, account: None, main: chosen, subagents: Box::new([subagent]) };
        let mut member = task(number, &[]);
        member.authority.models = tasks::Models::Named(0b11);
        member.llm = Some(choice.clone());
        let mut world = World::new(seed, LIMITS);
        assert_eq!(world.make(PERSON_PARTY, vec![member]), Reply::Made(vec![number]));
        world.restart();
        assert_eq!(world.record(number).llm, Some(choice));

        let mut efforts = EFFORTS;
        efforts[0] = 0;
        let mut restored = Domain::new(
            &LIMITS,
            jig_domain_tasks::Kept { revisions: 4, versions: 8 },
            seed,
            1,
            charters(),
            Box::new(efforts),
        );
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
        Ok(())
    })
}
