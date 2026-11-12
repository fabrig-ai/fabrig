//! Parts follow their facts, drawings keep every stop and columns keep every subject
//! (temper/docs/design/client/pages/settings.md, 2; temper/docs/design/draft/surfaces.md, 12).
use jig_client::contract::TemplateBudget;
use skein_lib::{List, Rng};
use skein_ui::{Builder, Role};
use std::{collections::BTreeSet, panic::catch_unwind};
use temper_client_domain::{
    forge,
    pages::how::{self, Budget, Facts, Head, Line, Path, Read},
};
use temper_client_protocol_wire::v1 as w;
use temper_client_view::paths;
use temper_client_world::fixtures;
use temper_protocol_reaches::{
    Class, Classes, ITEMS, Item, Kind, NAME_BYTES, PATTERNS, PathSet, SETS, Subject, WORDS_BYTES, Way,
};

// Match public_address's independently replayable 2,000 draws.
const SEEDS: u64 = 2_000;
// A draw follows sixteen settings moves, mixing each of the six moves below.
const MOVES: u32 = 16;
// Four reads, two Ask offerings, two Goals settings, five budget forms,
// two classes-read states and two project-count states.
const FACT_COMBINATIONS: usize = 4 * 2 * 2 * 5 * 2 * 2;

type Combination = (u8, bool, bool, u8, bool, bool);

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
        period: None, standing: Box::default(),
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
        classes: rng.chance(500),
        several: rng.chance(500),
    }
}

// The oracle reads 5.13's rules independently: no figure means no Budgets row,
// the template alone offers Ask, and only an answered read names a way.
fn check_parts(facts: &Facts, seed: u64, step: u32) {
    let shown = how::parts(facts);
    assert_eq!(shown.ask, facts.asking, "seed {seed}, step {step}, {facts:?}: Ask follows its template");
    assert_eq!(shown.reaches, facts.classes, "seed {seed}, step {step}, {facts:?}: reaches follows its read");
    assert_eq!(shown.several, facts.several, "seed {seed}, step {step}, {facts:?}: head follows project count");
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
    // The fixture uses the wire's bound of 24 stops (domain/forge/change/branch-path.md).
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

fn combination(facts: &Facts) -> Combination {
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
    (read, facts.asking, facts.goals, budget, facts.classes, facts.several)
}

fn sweep(
    seed: u64,
    limits: &w::Limits,
    bounds: &skein_ui::Limits,
    combinations: &mut BTreeSet<Combination>,
    lengths: &mut BTreeSet<u32>,
) {
    let mut rng = Rng::new(seed);
    let initial = facts(&mut rng);
    let mut settings = initial.clone();
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

fn draw(rng: &mut Rng, bound: usize) -> usize {
    usize::try_from(rng.below(u64::try_from(bound).expect("public table bound"))).expect("bounded draw")
}

fn random_class(rng: &mut Rng) -> Class {
    match rng.below(3) {
        0 => Class::Decide,
        1 => Class::Ask,
        _ => Class::AskAway,
    }
}

fn pattern(rng: &mut Rng) -> Box<[u8]> {
    let segments: [&[u8]; 5] = [b"src", b"docs", b"*", b"**", b"a*b"];
    let first = segments[draw(rng, segments.len())];
    let second = segments[draw(rng, segments.len())];
    [first, second].join(&b'/').into_boxed_slice()
}

fn random_set(number: u8, rng: &mut Rng) -> PathSet {
    PathSet {
        number,
        name: vec![b'n'; 1 + draw(rng, NAME_BYTES)].into_boxed_slice(),
        patterns: (0..=draw(rng, PATTERNS)).map(|_| pattern(rng)).collect(),
    }
}

fn random_subject(sets: &[PathSet], rng: &mut Rng) -> Subject {
    let kinds: Vec<_> = Kind::ALL.into_iter().filter(|kind| !kind.fixed()).collect();
    if !sets.is_empty() && rng.chance(500) {
        Subject::Set(sets[draw(rng, sets.len())].number)
    } else {
        Subject::Kind(kinds[draw(rng, kinds.len())])
    }
}

fn words_item(sets: &[PathSet], rng: &mut Rng) -> Item {
    Item {
        subject: random_subject(sets, rng),
        words: vec![b'w'; 1 + draw(rng, WORDS_BYTES)].into_boxed_slice(),
        class: random_class(rng),
    }
}

fn random_table(rng: &mut Rng) -> Classes {
    let mut numbers: Vec<_> = (1..=u8::try_from(SETS).expect("public set bound")).collect();
    for index in 0..numbers.len() {
        let other = draw(rng, numbers.len());
        numbers.swap(index, other);
    }
    let sets: Vec<_> = numbers.into_iter().take(draw(rng, SETS + 1)).map(|number| random_set(number, rng)).collect();
    // Whole subjects and uniform classes make empty columns and both homogeneous
    // forms of Asks you ordinary draws, alongside missing own items and mixed classes.
    let all_own = rng.chance(500);
    let uniform = if rng.chance(500) { Some(random_class(rng)) } else { None };
    let subjects = Kind::ALL
        .into_iter()
        .filter(|kind| !kind.fixed())
        .map(Subject::Kind)
        .chain(sets.iter().map(|set| Subject::Set(set.number)));
    let mut items = Vec::new();
    for subject in subjects {
        if all_own || rng.chance(500) {
            items.push(Item { subject, words: Box::default(), class: random_class(rng) });
        }
    }
    for _ in 0..draw(rng, ITEMS - items.len() + 1) {
        items.push(words_item(&sets, rng));
    }
    for index in 0..items.len() {
        let other = draw(rng, items.len());
        items.swap(index, other);
    }
    if let Some(class) = uniform {
        for item in &mut items {
            item.class = class;
        }
    }
    Classes { items: items.into_boxed_slice(), sets: sets.into_boxed_slice() }
}

fn moved_table(classes: &Classes, rng: &mut Rng) -> Classes {
    let mut items = classes.items.to_vec();
    let mut sets = classes.sets.to_vec();
    match rng.below(6) {
        0 => {
            let index = draw(rng, items.len());
            if let Some(item) = items.get_mut(index) {
                item.class = random_class(rng);
            }
        }
        1 => items.push(words_item(&sets, rng)),
        2 => {
            let free: Vec<_> = (1..=u8::try_from(SETS).expect("public set bound"))
                .filter(|number| !sets.iter().any(|set| set.number == *number))
                .collect();
            if let Some(number) = free.get(draw(rng, free.len())).copied() {
                sets.push(random_set(number, rng));
                if rng.chance(500) {
                    items.push(Item { subject: Subject::Set(number), words: Box::default(), class: random_class(rng) });
                }
            }
        }
        3 => {
            if !items.is_empty() {
                let index = draw(rng, items.len());
                items.remove(index);
            }
        }
        4 => {
            if !sets.is_empty() {
                let index = draw(rng, sets.len());
                let removed = sets.remove(index);
                items.retain(|item| item.subject != Subject::Set(removed.number));
            }
        }
        _ => {
            return Classes::stock(match rng.below(3) {
                0 => Way::YouApprove,
                1 => Way::AgentsReview,
                _ => Way::DraftFirst,
            });
        }
    }
    Classes { items: items.into_boxed_slice(), sets: sets.into_boxed_slice() }
}

// Build the settings order once (temper/docs/design/client/pages/settings.md, 2),
// independently of reaches' per-column traversal. Item identity includes its
// place: two rules with words may share a subject and still each need a line.
fn settings_order(classes: &Classes) -> Vec<Line> {
    let mut lines: Vec<_> = Kind::ALL
        .into_iter()
        .filter(|kind| !kind.fixed())
        .map(|kind| {
            let subject = Subject::Kind(kind);
            let own = classes.items.iter().position(|item| item.subject == subject && item.words.is_empty());
            own.map_or(Line { subject, item: 0, class: Class::AskAway }, |index| Line {
                subject,
                item: u8::try_from(index + 1).expect("checked item bound"),
                class: classes.items[index].class,
            })
        })
        .collect();
    lines.extend(classes.items.iter().enumerate().filter_map(|(index, item)| {
        let kind_own = match item.subject {
            Subject::Kind(_) => item.words.is_empty(),
            Subject::Set(_) => false,
        };
        (!kind_own).then_some(Line {
            subject: item.subject,
            item: u8::try_from(index + 1).expect("checked item bound"),
            class: item.class,
        })
    }));
    lines.extend(classes.sets.iter().filter_map(|set| {
        let subject = Subject::Set(set.number);
        let own = classes.items.iter().any(|item| item.subject == subject && item.words.is_empty());
        (!own).then_some(Line { subject, item: 0, class: Class::AskAway })
    }));
    lines
}

#[derive(Default)]
struct ReachesCoverage {
    decide: BTreeSet<bool>,
    // AskAway present, Ask present: includes the empty column.
    ask: BTreeSet<(bool, bool)>,
}

fn check_reaches(classes: &Classes, seed: u64, step: u32, coverage: &mut ReachesCoverage) {
    classes.check().expect("only checked tables reach the page");
    let order = settings_order(classes);
    let select = |class| order.iter().copied().filter(move |line| line.class == class);
    let decide: Vec<_> = select(Class::Decide).collect();
    let ask: Vec<_> = select(Class::AskAway).chain(select(Class::Ask)).collect();
    let wait: Vec<_> = Kind::ALL
        .into_iter()
        .filter(|kind| kind.fixed())
        .map(|kind| Line { subject: Subject::Kind(kind), item: 0, class: Class::Wait })
        .collect();
    let shown = how::reaches(classes);
    assert_eq!(shown.decide.as_ref(), decide, "seed {seed}, step {step}, {classes:?}: Decide in settings order");
    assert_eq!(shown.ask.as_ref(), ask, "seed {seed}, step {step}, {classes:?}: AskAway then Ask in settings order");
    assert_eq!(shown.wait.as_ref(), wait, "seed {seed}, step {step}, {classes:?}: fixed kinds alone, in kit order");
    coverage.decide.insert(!shown.decide.is_empty());
    coverage.ask.insert((select(Class::AskAway).next().is_some(), select(Class::Ask).next().is_some()));
}

fn reaches_sweep(seed: u64, coverage: &mut ReachesCoverage) {
    let mut rng = Rng::new(seed);
    let mut classes = random_table(&mut rng);
    check_reaches(&classes, seed, 0, coverage);
    for step in 1..=MOVES {
        let moved = moved_table(&classes, &mut rng);
        if moved.check().is_ok() {
            classes = moved;
        }
        check_reaches(&classes, seed, step, coverage);
    }
}

#[test]
fn what_reaches_you_places_each_subject_once() {
    let mut coverage = ReachesCoverage::default();
    for seed in 1..=SEEDS {
        assert!(
            catch_unwind(std::panic::AssertUnwindSafe(|| reaches_sweep(seed, &mut coverage))).is_ok(),
            "seed {seed}: replay with reaches_sweep({seed})"
        );
    }
    assert_eq!(coverage.decide, BTreeSet::from([false, true]), "with and without a Decide line");
    assert_eq!(
        coverage.ask,
        BTreeSet::from([(false, false), (false, true), (true, false), (true, true)]),
        "Asks you absent, all Ask, all AskAway and mixed"
    );
}
