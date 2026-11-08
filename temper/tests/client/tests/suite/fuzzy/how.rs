//! Parts follow their facts and drawings keep every stop
//! (temper/docs/design/client/README.md, 5.13; temper/docs/design/draft/surfaces.md, 12).
use jig_client::contract::TemplateBudget;
use skein_lib::{List, Rng};
use skein_ui::{Builder, Role};
use std::{collections::BTreeSet, panic::catch_unwind};
use temper_client_domain::{
    forge,
    pages::how::{self, Budget, Facts, Head, Path, Read},
};
use temper_client_protocol_wire::v1 as w;
use temper_client_view::paths;
use temper_client_world::fixtures;

// Match public_address's independently replayable 2,000 draws.
const SEEDS: u64 = 2_000;
// A draw follows sixteen settings moves, mixing each of the six moves below.
const MOVES: u32 = 16;
// Four reads, two Ask offerings, two Goals settings and five budget forms.
const FACT_COMBINATIONS: usize = 4 * 2 * 2 * 5;

fn amount(rng: &mut Rng) -> u64 {
    match rng.below(3) {
        0 => 0,
        1 => u64::MAX,
        _ => rng.next_u64(),
    }
}

fn budget(rng: &mut Rng) -> Option<TemplateBudget> {
    match rng.below(5) {
        0 => None,
        1 => Some(TemplateBudget::Given),
        2 => Some(TemplateBudget::Amount(amount(rng))),
        3 => Some(TemplateBudget::AsAllowed { cap: None }),
        _ => Some(TemplateBudget::AsAllowed { cap: Some(amount(rng)) }),
    }
}

fn current(rng: &mut Rng) -> Read {
    // The three stock ways and a way of the project's own.
    let index = rng.below(4);
    Read::Shown { current: if index < 3 { Some(u32::try_from(index).expect("stock index")) } else { None } }
}

fn facts(rng: &mut Rng) -> Facts {
    Facts {
        roles: false,
        asking: rng.chance(500),
        read: match rng.below(4) {
            0 => Read::Reading,
            1 => Read::Ended,
            2 => Read::Shown { current: None },
            _ => Read::Shown { current: Some(u32::try_from(rng.below(3)).expect("stock index")) },
        },
        goals: rng.chance(500),
        budget: budget(rng),
    }
}

// The oracle reads 5.13's rules independently: no figure means no Budgets row,
// the template alone offers Ask, and only an answered read names a way.
fn check_parts(facts: &Facts, seed: u64, step: u32) {
    let shown = how::parts(facts);
    assert_eq!(shown.ask, facts.asking, "seed {seed}, step {step}, {facts:?}: Ask follows its template");
    let (head, path) = match facts.read {
        Read::Reading => (Head::Reading, Path::Reading),
        Read::Ended => (Head::Reading, Path::Ended),
        Read::Shown { current: None } => (Head::Own, Path::Shown),
        Read::Shown { current: Some(index) } => (Head::Way(index), Path::Shown),
    };
    assert_eq!(shown.head, head, "seed {seed}, step {step}, {facts:?}: heading follows the read");
    assert_eq!(shown.path, path, "seed {seed}, step {step}, {facts:?}: path follows the read");
    let figure = match facts.budget {
        Some(TemplateBudget::Amount(value)) if facts.goals => Some(Budget::Amount(value)),
        Some(TemplateBudget::AsAllowed { cap: Some(value) }) if facts.goals => Some(Budget::UpTo(value)),
        Some(TemplateBudget::Amount(_) | TemplateBudget::AsAllowed { cap: Some(_) | None } | TemplateBudget::Given)
        | None => None,
    };
    assert_eq!(shown.budget, figure, "seed {seed}, step {step}, {facts:?}: Budgets needs Goals and a figure");
}

fn path(rng: &mut Rng, limits: &w::Limits) -> w::BranchPath {
    // The fixture uses the wire's bound of 24 stops (forge.md, 8.13).
    let count = u32::try_from(rng.between(0, u64::from(limits.branch_path_stops))).expect("wire count");
    let mut stops = List::with_capacity(count);
    for index in 0..count {
        let (actor, step) = match rng.below(4) {
            0 => (w::Actor::Person, w::PathStep::Ask),
            1 => (w::Actor::Agent, w::PathStep::Produce),
            2 => (w::Actor::Code, w::PathStep::Check),
            _ => (w::Actor::Bar, w::PathStep::Branch),
        };
        let words = rng.next_u64().to_string().into_bytes().into_boxed_slice();
        let card = w::Card::new(
            limits,
            w::CardParts { who: Box::new([]), name: words.clone(), facts: List::with_capacity(0) },
        )
        .expect("short card within fixture limits");
        stops
            .push(
                w::Stop::new(
                    limits,
                    w::StopParts {
                        needed: actor != w::Actor::Bar && index % 2 == 0,
                        over: if actor == w::Actor::Agent {
                            b"tests, review".as_slice().into()
                        } else {
                            Box::default()
                        },
                        actor,
                        step,
                        words,
                        opens: w::Opens::Nothing,
                        card,
                    },
                )
                .expect("stop within fixture limits"),
            )
            .expect("one stop per draw");
    }
    w::BranchPath::new(limits, w::BranchPathParts { repository: None, branch: b"main".as_slice().into(), stops })
        .expect("path within fixture limits")
}

fn check_drawing(path: &w::BranchPath, bounds: &skein_ui::Limits, seed: u64, step: u32) {
    let stops: Vec<_> = path
        .stops()
        .iter()
        .map(|stop| paths::Stop {
            mark: match stop.actor() {
                w::Actor::Person => paths::Mark::Person,
                w::Actor::Agent => paths::Mark::Agent,
                w::Actor::Code => paths::Mark::Code,
                w::Actor::Bar => paths::Mark::Bar,
            },
            words: stop.words(),
            needed: stop.needed(),
            over: stop.over(),
        })
        .collect();
    let mut builder = Builder::new(bounds);
    let result = paths::path(&mut builder, b"The way a change goes", &stops, None);
    let tree = builder.finish();
    if path.stops().is_empty() {
        assert_eq!(result, Err(paths::Problem::Stops));
        assert!(tree.nodes().is_empty(), "seed {seed}, step {step}: empty path has no fallback lines");
        return;
    }
    result.expect("wire path fits drawing");
    let lines: Vec<_> = tree
        .nodes()
        .iter()
        .filter(|node| node.role == Role::Item)
        .map(|node| node.name.as_deref().expect("fallback line"))
        .collect();
    assert_eq!(
        lines.len(),
        usize::try_from(path.stops().len()).expect("wire stop count"),
        "seed {seed}, step {step}: every stop exactly once"
    );
    for (index, (line, stop)) in lines.iter().zip(path.stops()).enumerate() {
        let actor = match stop.actor() {
            w::Actor::Person => "a person",
            w::Actor::Agent => "an agent",
            w::Actor::Code => "code",
            w::Actor::Bar => "a branch",
        };
        let needed = if stop.needed() { ", when needed" } else { "" };
        let over = if stop.over().is_empty() {
            String::new()
        } else {
            format!(" ({})", std::str::from_utf8(stop.over()).expect("small words"))
        };
        let expected = format!("{}, {actor}{needed}{over}", std::str::from_utf8(stop.words()).expect("stop words"));
        assert_eq!(
            *line,
            expected.as_bytes(),
            "seed {seed}, step {step}, stop {index}: fallback keeps path order and actor"
        );
    }
}

fn combination(facts: &Facts) -> (u8, bool, bool, u8) {
    let read = match facts.read {
        Read::Reading => 0,
        Read::Ended => 1,
        Read::Shown { current: None } => 2,
        Read::Shown { current: Some(_) } => 3,
    };
    let budget = match facts.budget {
        None => 0,
        Some(TemplateBudget::Given) => 1,
        Some(TemplateBudget::Amount(_)) => 2,
        Some(TemplateBudget::AsAllowed { cap: None }) => 3,
        Some(TemplateBudget::AsAllowed { cap: Some(_) }) => 4,
    };
    (read, facts.asking, facts.goals, budget)
}

fn sweep(
    seed: u64,
    limits: &w::Limits,
    bounds: &skein_ui::Limits,
    combinations: &mut BTreeSet<(u8, bool, bool, u8)>,
    lengths: &mut BTreeSet<u32>,
) {
    let mut rng = Rng::new(seed);
    let initial = facts(&mut rng);
    let mut settings = initial;
    let mut route = path(&mut rng, limits);
    check_parts(&settings, seed, 0);
    check_drawing(&route, bounds, seed, 0);
    combinations.insert(combination(&settings));
    lengths.insert(route.stops().len());
    for step in 1..=MOVES {
        match rng.below(6) {
            0 => {
                // Using a way replaces its settings and its entire path.
                settings = facts(&mut rng);
                settings.read = current(&mut rng);
                route = path(&mut rng, limits);
            }
            1 => settings.goals = !settings.goals,
            2 => settings.budget = budget(&mut rng),
            3 => settings.read = Read::Ended,
            4 => settings.read = Read::Reading,
            _ => settings.asking = !settings.asking,
        }
        check_parts(&settings, seed, step);
        check_drawing(&route, bounds, seed, step);
        // An unrelated project's read cannot change this project's parts.
        check_parts(&initial, seed, step);
        combinations.insert(combination(&settings));
        lengths.insert(route.stops().len());
    }
}

#[test]
fn parts_and_drawing_follow_only_the_current_facts() {
    let limits = forge::wire_limits(&fixtures::domain_limits().forge).expect("world wire limits");
    let domain = fixtures::domain_limits();
    let bounds = fixtures::view_bounds(&domain, &fixtures::phrasebooks(&domain)).tree;
    let mut combinations = BTreeSet::new();
    let mut lengths = BTreeSet::new();
    for seed in 1..=SEEDS {
        assert!(
            catch_unwind(std::panic::AssertUnwindSafe(|| sweep(
                seed,
                &limits,
                &bounds,
                &mut combinations,
                &mut lengths
            )))
            .is_ok(),
            "seed {seed}: replay with sweep({seed})"
        );
    }
    assert_eq!(combinations.len(), FACT_COMBINATIONS, "every combination of parts' facts was drawn");
    assert_eq!(
        lengths,
        (0..=limits.branch_path_stops).collect(),
        "every path length, including empty and full, was drawn"
    );
}
