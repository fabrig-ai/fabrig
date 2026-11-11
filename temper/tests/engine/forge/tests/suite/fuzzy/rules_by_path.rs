//! Coverage agrees with the sets' patterns, independently of its bit mask (change/gates.md, 2).
use skein_lib::Rng;
use temper_engine_domain_forge::{self as forge, settings::LandingRule};
use temper_engine_forge_world::{REPO, World};
use temper_protocol_reaches::{Classes, PathSet, matches};

fn word(rng: &mut Rng, pattern: bool) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for part in 0..rng.between(1, 3) {
        if part != 0 {
            bytes.push(b'/');
        }
        if pattern && rng.below(4) == 0 {
            bytes.extend_from_slice(b"**");
        } else {
            for _ in 0..rng.between(1, 3) {
                bytes.push(if pattern && rng.below(4) == 0 {
                    b'*'
                } else if rng.below(2) == 0 {
                    b'a'
                } else {
                    b'b'
                });
            }
        }
    }
    // Adjacent stars within a segment are not patterns; ** names a whole segment.
    if pattern && !temper_protocol_reaches::well_formed(&bytes) {
        return b"a*".as_slice().into();
    }
    bytes.into_boxed_slice()
}

#[test]
fn path_rules_agree_with_the_independent_pattern_statement() {
    let mut world = World::new(0);
    world.adopt();
    let Some(forge::Stored::Repository(repository)) = world.stored().get(&forge::Key::Repository(REPO)) else {
        panic!("adopted fixture")
    };
    for seed in 0..64 {
        let mut rng = Rng::new(seed);
        for _ in 0..64 {
            let mut sets = Vec::new();
            for number in 1..=6 {
                if rng.below(2) == 0 {
                    let patterns = (0..rng.between(1, 6)).map(|_| word(&mut rng, true)).collect();
                    sets.push(PathSet { number, name: b"set".as_slice().into(), patterns });
                }
            }
            let classes = Classes { items: Box::new([]), sets: sets.into_boxed_slice() };
            let paths: Box<[_]> = (0..rng.below(7)).map(|_| word(&mut rng, false)).collect();
            let named: Box<[_]> = (1..=6).filter(|_| rng.below(2) == 0).collect();
            let whole = rng.below(2) != 0;
            let rule = LandingRule {
                sets: named,
                pattern: forge::GatePattern {
                    segments: Box::new([b"branch".as_slice().into()]),
                    last: forge::GateLast::Exact(b"main".as_slice().into()),
                },
                enforce: true,
                ci: true,
                up_to_date: true,
                gates: Box::new([]),
                approvals: Box::new([]),
            };
            let expected = rule.sets.is_empty()
                || !whole
                || rule.sets.iter().any(|number| {
                    classes.sets.iter().find(|set| set.number == *number).is_none_or(|set| {
                        set.patterns.iter().any(|pattern| paths.iter().any(|path| matches(pattern, path)))
                    })
                });
            let touched = forge::touched(&classes, &paths, whole);
            assert_eq!(
                forge::covers(&rule, repository, b"main", touched),
                expected,
                "seed {seed}: {classes:?}, {paths:?}, {rule:?}, whole {whole}"
            );
        }
    }
}
