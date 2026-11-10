//! Seeded judgements keep their rules (temper/docs/design/domain/forge/attention.md, 6).
#![expect(clippy::disallowed_methods, reason = "ordinary random fixtures build and inspect bounded tables")]
#![expect(clippy::disallowed_types, reason = "ordinary random fixtures collect bounded items and paths")]
#![expect(clippy::disallowed_macros, reason = "ordinary random fixtures use bounded vectors")]
use skein_lib::Rng;
use temper_protocol_reaches::{
    Ask, Class, Classes, ITEMS, Item, Kind, NAME_BYTES, Offer, PATHS, PATTERNS, PathSet, Reach, Reason, Refusal, SETS,
    Subject, WORDS_BYTES, Way, judge, narrowed, offers,
};

fn pick_class(rng: &mut Rng) -> Class {
    [Class::Decide, Class::Ask, Class::AskAway]
        .get(usize::try_from(rng.below(3)).expect("small draw"))
        .copied()
        .expect("class draw")
}
fn pattern(rng: &mut Rng) -> Box<[u8]> {
    let segments: [&[u8]; 9] = [b"src", b"docs", b"a", b"b", b"*", b"**", b"a*b", b"x.c", b"include"];
    let mut bytes = Vec::new();
    for index in 0..rng.below(4).saturating_add(1) {
        if index != 0 {
            bytes.push(b'/');
        }
        bytes
            .extend_from_slice(segments.get(usize::try_from(rng.below(9)).expect("small draw")).expect("segment draw"));
    }
    bytes.into_boxed_slice()
}
fn random_table(rng: &mut Rng) -> Classes {
    let mut sets: Vec<PathSet> = Vec::new();
    let count = usize::try_from(rng.below(u64::try_from(SETS + 1).expect("bound"))).expect("set count");
    let mut numbers = [1, 2, 3, 4, 5, 6];
    for index in 0..SETS {
        let other = usize::try_from(rng.below(6)).expect("number draw");
        numbers.swap(index, other);
    }
    for number in numbers.into_iter().take(count) {
        sets.push(PathSet {
            number,
            name: vec![
                b'n';
                usize::try_from(rng.below(u64::try_from(NAME_BYTES).expect("bound")))
                    .expect("name length")
                    .saturating_add(1)
            ]
            .into_boxed_slice(),
            patterns: (0..=rng.below(u64::try_from(PATTERNS).expect("bound"))).map(|_| pattern(rng)).collect(),
        });
    }
    let mut items: Vec<Item> = Vec::new();
    for _item in 0..rng.below(u64::try_from(ITEMS + 1).expect("bound")) {
        let subject = if !sets.is_empty() && rng.chance(500) {
            Subject::Set(
                sets.get(usize::try_from(rng.below(u64::try_from(sets.len()).expect("set count"))).expect("set draw"))
                    .expect("drawn set")
                    .number,
            )
        } else {
            Subject::Kind(*Kind::ALL.get(usize::try_from(rng.below(6)).expect("kind draw")).expect("movable kind"))
        };
        let length = if rng.chance(500) && !items.iter().any(|item| item.subject == subject && item.words.is_empty()) {
            0
        } else {
            usize::try_from(rng.below(u64::try_from(WORDS_BYTES).expect("bound")))
                .expect("word count")
                .saturating_add(1)
        };
        items.push(Item { subject, words: vec![b'w'; length].into_boxed_slice(), class: pick_class(rng) });
    }
    Classes { items: items.into_boxed_slice(), sets: sets.into_boxed_slice() }
}
fn random_paths(classes: &Classes, rng: &mut Rng) -> Box<[Box<[u8]>]> {
    (0..rng.below(u64::try_from(PATHS + 1).expect("bound")))
        .map(|_| {
            if classes.sets.is_empty() || rng.chance(250) {
                return Box::from(b"elsewhere/unrelated".as_slice());
            }
            let set = classes
                .sets
                .get(
                    usize::try_from(rng.below(u64::try_from(classes.sets.len()).expect("set count")))
                        .expect("set draw"),
                )
                .expect("drawn set");
            let pattern = set
                .patterns
                .get(
                    usize::try_from(rng.below(u64::try_from(set.patterns.len()).expect("pattern count")))
                        .expect("pattern draw"),
                )
                .expect("drawn pattern");
            let mut parts = Vec::new();
            for segment in pattern.split(|b| *b == b'/') {
                if segment == b"**" && rng.chance(500) {
                    continue;
                }
                let literal: Vec<u8> = segment.iter().copied().filter(|b| *b != b'*').collect();
                parts.push(if literal.is_empty() { b"a".to_vec() } else { literal });
            }
            if parts.is_empty() {
                parts.push(b"a".to_vec());
            }
            parts.join(&b'/').into_boxed_slice()
        })
        .collect()
}
fn subject_class(classes: &Classes, subject: Subject, under: u8) -> Class {
    match subject {
        Subject::Kind(Kind::Money | Kind::Irreversible | Kind::Rules | Kind::Other) => return Class::Wait,
        Subject::Kind(
            Kind::Wording | Kind::Retry | Kind::Implementation | Kind::Tests | Kind::Dependency | Kind::Departure,
        )
        | Subject::Set(_) => {}
    }
    if let Some(index) = under.checked_sub(1)
        && let Some(item) = classes.items.get(usize::from(index))
        && !item.words.is_empty()
        && item.subject == subject
    {
        return item.class;
    }
    classes
        .items
        .iter()
        .find(|item| item.subject == subject && item.words.is_empty())
        .map_or(Class::AskAway, |item| item.class)
}
fn decides(class: Class, away: bool) -> bool {
    class == Class::Decide || (class == Class::Ask && away)
}
fn invariants(classes: &Classes, ask: &Ask<'_>, reach: Reach, seed: u64) {
    let mut subjects = vec![(Subject::Kind(ask.kind), subject_class(classes, Subject::Kind(ask.kind), ask.under))];
    let mut touched = 0;
    let mut used_under = 0;
    for set in &classes.sets {
        if set.matched(ask.paths) != 0 {
            touched |= 1_u8 << set.number.saturating_sub(1);
            subjects.push((Subject::Set(set.number), subject_class(classes, Subject::Set(set.number), ask.under)));
        }
    }
    assert_eq!(reach.sets, touched, "seed {seed}: all touched sets");
    if let Some(index) = ask.under.checked_sub(1)
        && let Some(item) = classes.items.get(usize::from(index))
        && !item.words.is_empty()
        && subjects.iter().any(|(subject, _)| *subject == item.subject)
    {
        used_under = ask.under;
    }
    assert_eq!(reach.under, used_under, "seed {seed}: used under");
    assert_eq!(reach.away, ask.away, "seed {seed}: away retained");
    for (subject, class) in &subjects {
        assert!(reach.class.strictness() >= class.strictness(), "seed {seed}: {subject:?} strictness");
    }
    let mut strictest = subjects.iter().map(|(_, class)| class.strictness()).max().expect("kind subject");
    if ask.always {
        strictest = strictest.max(Class::AskAway.strictness());
    }
    assert_eq!(reach.class.strictness(), strictest, "seed {seed}: no spurious lift");
    assert_eq!(reach.decided(), decides(reach.class, ask.away), "seed {seed}: decision");
    if ask.always {
        assert!(!reach.decided(), "seed {seed}: always asks");
    }
    match ask.kind {
        Kind::Money | Kind::Irreversible | Kind::Rules | Kind::Other => {
            assert_eq!(reach.class, Class::Wait, "seed {seed}: fixed wait");
            assert!(!reach.decided(), "seed {seed}: fixed never decided");
        }
        Kind::Wording | Kind::Retry | Kind::Implementation | Kind::Tests | Kind::Dependency | Kind::Departure => {}
    }
    let reason = match reach.reason {
        Reason::Kind => {
            assert_eq!(reach.matched, 0, "seed {seed}: kind matched");
            Some(Subject::Kind(ask.kind))
        }
        Reason::Set(number) => {
            assert_ne!(reach.sets & (1 << number.saturating_sub(1)), 0, "seed {seed}: reason touched");
            assert_eq!(
                reach.matched,
                classes.set(number).expect("reason set").matched(ask.paths),
                "seed {seed}: matched patterns"
            );
            Some(Subject::Set(number))
        }
        Reason::Always => {
            assert!(
                ask.always && subjects.iter().all(|(_, class)| class.strictness() < Class::AskAway.strictness()),
                "seed {seed}: only lift"
            );
            assert_eq!(reach.matched, 0, "seed {seed}: always matched");
            None
        }
    };
    if let Some(subject) = reason {
        assert_eq!(subject_class(classes, subject, ask.under), reach.class, "seed {seed}: reason class");
        let tied: Vec<_> =
            subjects.iter().filter(|(_, class)| *class == reach.class).map(|(subject, _)| *subject).collect();
        let expected = if tied.contains(&Subject::Kind(ask.kind)) {
            Subject::Kind(ask.kind)
        } else {
            Subject::Set(
                tied.iter()
                    .filter_map(|s| if let Subject::Set(n) = s { Some(*n) } else { None })
                    .min()
                    .expect("set reason"),
            )
        };
        assert_eq!(subject, expected, "seed {seed}: tie order");
    }
    if !classes.away_may_decide() {
        assert_eq!(
            judge(classes, &Ask { away: true, ..*ask }).decided(),
            judge(classes, &Ask { away: false, ..*ask }).decided(),
            "seed {seed}: away without Ask"
        );
    }
    assert!(!judge(&Classes::stock(Way::YouApprove), ask).decided(), "seed {seed}: You approve");
    for offer in offers(classes, &reach) {
        let next = narrowed(classes, &reach, offer, b"safe fixes").expect("offered table");
        assert_eq!(next.check(), Ok(()), "seed {seed}: offer checks");
        let under =
            if let Offer::Words(_) = offer { u8::try_from(next.items.len()).expect("new item") } else { ask.under };
        // Narrowing decides when all other subjects permit it under the next
        // call's override, and the caller has not chosen Asks: always.
        let others_decide = subjects
            .iter()
            .all(|(subject, _)| Some(*subject) == reason || decides(subject_class(&next, *subject, under), ask.away));
        if !ask.always && others_decide {
            assert!(
                judge(&next, &Ask { under, ..*ask }).decided(),
                "seed {seed}: sole blocking subject narrowed ({offer:?})"
            );
        }
    }
}
#[test]
fn the_judgement_keeps_its_rules() {
    for seed in 0..64 {
        let mut rng = Rng::new(seed);
        let classes = random_table(&mut rng);
        assert_eq!(classes.check(), Ok(()), "seed {seed}: generated table checks");
        for _ask in 0..32_u8 {
            let paths = random_paths(&classes, &mut rng);
            let kind = *Kind::ALL.get(usize::try_from(rng.below(10)).expect("kind draw")).expect("kind");
            let under = if rng.chance(500) {
                u8::try_from(rng.below(256)).expect("any under")
            } else {
                u8::try_from(rng.below(u64::try_from(classes.items.len() + 1).expect("item count")))
                    .expect("under item")
            };
            let ask = Ask { kind, paths: &paths, under, away: rng.chance(500), always: rng.chance(500) };
            invariants(&classes, &ask, judge(&classes, &ask), seed);
        }
        // Unchecked tables cannot make a fixed kind movable.
        let unchecked = Classes {
            items: Box::new([Item {
                subject: Subject::Kind(Kind::Money),
                words: Box::default(),
                class: Class::Decide,
            }]),
            sets: Box::default(),
        };
        assert_eq!(unchecked.check(), Err(Refusal::Fixed { item: 1 }), "seed {seed}: fixed item refused");
        let ask = Ask { kind: Kind::Money, paths: &[], under: 0, away: true, always: false };
        assert_eq!(judge(&unchecked, &ask).class, Class::Wait, "seed {seed}: unchecked money waits");
    }
}
