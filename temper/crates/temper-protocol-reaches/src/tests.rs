//! Tables for each area of the judgement (temper/docs/design/domain/forge/attention.md, 6).
#![expect(clippy::disallowed_methods, reason = "ordinary table fixtures build and inspect bounded tables")]
#![expect(clippy::disallowed_types, reason = "ordinary table fixtures collect named rows")]
#![expect(clippy::disallowed_macros, reason = "ordinary table fixtures use bounded vectors")]
use super::*;
use alloc::{vec, vec::Vec};

fn item(subject: Subject, words: &[u8], class: Class) -> Item {
    Item { subject, words: words.into(), class }
}
fn own(kind: Kind, class: Class) -> Item {
    item(Subject::Kind(kind), b"", class)
}
fn set(number: u8, patterns: &[&[u8]]) -> PathSet {
    PathSet { number, name: b"core".as_slice().into(), patterns: patterns.iter().map(|p| Box::from(*p)).collect() }
}
fn table(items: &[Item], sets: &[PathSet]) -> Classes {
    Classes { items: items.into(), sets: sets.into() }
}
fn ask(paths: &[Box<[u8]>]) -> Ask<'_> {
    Ask { kind: Kind::Wording, paths, under: 0, away: false, always: false }
}

#[test]
fn check_names_the_first_fault() {
    let base = own(Kind::Wording, Class::Ask);
    let core = set(1, &[b"src/**"]);
    let mut rows = vec![
        ("you approve", Classes::stock(Way::YouApprove), Ok(())),
        ("agents review", Classes::stock(Way::AgentsReview), Ok(())),
        ("draft first", Classes::stock(Way::DraftFirst), Ok(())),
        ("items", table(&vec![base.clone(); ITEMS + 1], &[]), Err(Refusal::Items)),
        ("sets", table(&[], &vec![core.clone(); SETS + 1]), Err(Refusal::Sets)),
        ("fixed", table(&[own(Kind::Money, Class::Ask)], &[]), Err(Refusal::Fixed { item: 1 })),
        ("wait", table(&[own(Kind::Wording, Class::Wait)], &[]), Err(Refusal::Wait { item: 1 })),
        (
            "words",
            table(&[item(Subject::Kind(Kind::Wording), &[b'a'; WORDS_BYTES + 1], Class::Ask)], &[]),
            Err(Refusal::Words { item: 1 }),
        ),
        ("no set", table(&[item(Subject::Set(1), b"", Class::Ask)], &[]), Err(Refusal::NoSet { item: 1 })),
        ("twice", table(&[base.clone(), base.clone()], &[]), Err(Refusal::Twice { item: 2 })),
        ("zero number", table(&[], &[set(0, &[b"a"])]), Err(Refusal::Number { set: 1 })),
        ("large number", table(&[], &[set(7, &[b"a"])]), Err(Refusal::Number { set: 1 })),
        ("repeated number", table(&[], &[core.clone(), core.clone()]), Err(Refusal::Number { set: 2 })),
        ("no patterns", table(&[], &[set(1, &[])]), Err(Refusal::Patterns { set: 1 })),
        ("many patterns", table(&[], &[set(1, &[b"a".as_slice(); PATTERNS + 1])]), Err(Refusal::Patterns { set: 1 })),
        ("empty pattern", table(&[], &[set(1, &[b""])]), Err(Refusal::Pattern { set: 1, pattern: 1 })),
        (
            "long pattern",
            table(&[], &[set(1, &[&[b'a'; PATTERN_BYTES + 1]])]),
            Err(Refusal::Pattern { set: 1, pattern: 1 }),
        ),
        ("bad pattern", table(&[], &[set(1, &[b"a/**b"])]), Err(Refusal::Pattern { set: 1, pattern: 1 })),
        (
            "fixed before wait and words",
            table(&[item(Subject::Kind(Kind::Money), &[b'a'; 121], Class::Wait)], &[]),
            Err(Refusal::Fixed { item: 1 }),
        ),
        (
            "first item before later fixed",
            table(&[own(Kind::Wording, Class::Wait), own(Kind::Money, Class::Decide)], &[]),
            Err(Refusal::Wait { item: 1 }),
        ),
        (
            "item before bad set",
            table(&[own(Kind::Wording, Class::Wait)], &[set(0, &[])]),
            Err(Refusal::Wait { item: 1 }),
        ),
        (
            "worded duplicates permitted",
            table(&vec![item(Subject::Kind(Kind::Wording), b"narrow", Class::Ask); 2], &[]),
            Ok(()),
        ),
    ];
    for (name, bytes) in [("empty name", Box::<[u8]>::default()), ("long name", Box::from([b'a'; NAME_BYTES + 1]))] {
        let mut bad = core.clone();
        bad.name = bytes;
        rows.push((name, table(&[], &[bad]), Err(Refusal::Name { set: 1 })));
    }
    for (name, classes, expected) in rows {
        assert_eq!(classes.check(), expected, "{name}");
    }
}

#[test]
fn patterns_match_segments() {
    for (name, pattern, path, formed, expected) in [
        ("literal", b"a/b".as_slice(), b"a/b".as_slice(), true, true),
        ("literal differs", b"a/b", b"a/c", true, false),
        ("star bytes", b"a*b", b"axyzb", true, true),
        ("star empty", b"a*b", b"ab", true, true),
        ("star stops at slash", b"a*b", b"ax/yb", true, false),
        ("stars backtrack", b"a*b*c", b"axbybzc", true, true),
        ("double start", b"**/a", b"x/y/a", true, true),
        ("double start empty", b"**/a", b"a", true, true),
        ("double middle", b"a/**/b", b"a/x/y/b", true, true),
        ("double middle empty", b"a/**/b", b"a/b", true, true),
        ("double end", b"a/**", b"a/x/y", true, true),
        ("double end empty", b"a/**", b"a", true, true),
        ("header exact", b"include/vsr.h", b"include/vsr.h", true, true),
        ("header extension", b"include/vsr.h", b"include/vsr.hh", true, false),
        ("protocol extension", b"src/protocol.*", b"src/protocol.c", true, true),
        ("protocol directory", b"src/protocol.*", b"src/protocol/x.c", true, false),
        ("protocol docs root", b"docs/protocol/**", b"docs/protocol", true, true),
        ("protocol docs descendants", b"docs/protocol/**", b"docs/protocol/a/b.md", true, true),
        ("double alone empty", b"**", b"", true, true),
        ("empty", b"", b"", false, false),
        ("empty segment", b"a//b", b"a//b", false, false),
        ("leading slash", b"/a", b"/a", false, false),
        ("trailing slash", b"a/", b"a/", false, false),
        ("embedded double", b"a**b", b"ab", false, false),
        ("triple", b"***", b"abc", false, false),
    ] {
        assert_eq!(well_formed(pattern), formed, "{name}: formed");
        assert_eq!(matches(pattern, path), expected, "{name}: matches");
    }
}

#[test]
fn stocks_and_away_follow_the_way() {
    use temper_client_protocol_wire::v1;
    for classes in [
        Classes::stock(Way::YouApprove),
        Classes::stock(Way::AgentsReview),
        Classes::stock(Way::DraftFirst),
        largest(),
        table(&[item(Subject::Set(6), b"keep the wire format", Class::Ask)], &[set(6, &[b"src/**", b"docs/*.md"])]),
    ] {
        assert_eq!(classes.check(), Ok(()));
        let record = classes_record(&classes).unwrap();
        assert_eq!(classes_from(&record), classes);
        let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut writer).unwrap();
        assert_eq!(
            v1::ProjectClasses::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&writer.finish())).unwrap(),
            record
        );
    }
    for subject in Kind::ALL.map(Subject::Kind).into_iter().chain([Subject::Set(6)]) {
        let record = subject_record(subject);
        assert_eq!(subject_from(&record), subject);
        assert_eq!(subject_record(subject_from(&record)), record);
    }
    for (reason, good) in [
        (Reason::Kind, true),
        (Reason::Set(1), true),
        (Reason::Set(6), true),
        (Reason::Always, true),
        (Reason::Set(0), false),
        (Reason::Set(7), false),
    ] {
        let reach = Reach {
            kind: Kind::Dependency,
            class: Class::AskAway,
            reason,
            under: 20,
            sets: 63,
            matched: 33,
            away: true,
        };
        assert_eq!(reach_from(&reach_record(&reach)), good.then_some(reach));
    }

    let expected = [
        own(Kind::Wording, Class::Decide),
        own(Kind::Retry, Class::Decide),
        own(Kind::Tests, Class::Decide),
        own(Kind::Implementation, Class::Ask),
        own(Kind::Dependency, Class::Ask),
        own(Kind::Departure, Class::Ask),
    ];
    for (name, way, items, away) in [
        ("you approve", Way::YouApprove, &[][..], false),
        ("agents review", Way::AgentsReview, expected.as_slice(), true),
        ("draft first", Way::DraftFirst, expected.as_slice(), true),
    ] {
        let classes = Classes::stock(way);
        assert_eq!(classes.items.as_ref(), items, "{name}: order");
        assert!(classes.sets.is_empty(), "{name}: no sets");
        assert_eq!(classes.away_may_decide(), away, "{name}: away");
    }
    let names: [&[u8]; 10] = [
        b"wording",
        b"retry",
        b"implementation",
        b"tests",
        b"dependency",
        b"departure",
        b"money",
        b"irreversible",
        b"rules",
        b"other",
    ];
    for (kind, name) in Kind::ALL.into_iter().zip(names) {
        assert_eq!(kind.name(), name);
        assert_eq!(Kind::named(name), Some(kind));
    }
    assert_eq!(Kind::named(b"unknown"), None);
    assert_eq!([Class::Decide, Class::Ask, Class::AskAway, Class::Wait].map(Class::strictness), [0, 1, 2, 3]);
}

#[test]
fn judgement_uses_the_strictest_subject() {
    let paths: Box<[Box<[u8]>]> = [Box::from(b"src/a".as_slice())].into();
    let base = ask(&paths);
    let sets = [set(4, &[b"src/**", b"include/**"]), set(2, &[b"src/*"])];
    let kinds = table(&[own(Kind::Wording, Class::Decide)], &[]);
    let mut rows = Vec::new();
    // Expected: class, reason, under, touched sets, matched patterns, decided.
    for class in [Class::Decide, Class::Ask, Class::AskAway] {
        for away in [false, true] {
            rows.push((
                "kind class and away",
                table(&[own(Kind::Wording, class)], &[]),
                Ask { away, ..base },
                (class, Reason::Kind, 0, 0, 0, class == Class::Decide || (class == Class::Ask && away)),
            ));
        }
    }
    for kind in [Kind::Money, Kind::Irreversible, Kind::Rules, Kind::Other] {
        for away in [false, true] {
            rows.push((
                "fixed ignores unchecked item",
                table(&[own(kind, Class::Decide)], &sets),
                Ask { kind, away, ..base },
                (Class::Wait, Reason::Kind, 0, 10, 0, false),
            ));
        }
    }
    rows.extend([
        ("missing kind", Classes::default(), base, (Class::AskAway, Reason::Kind, 0, 0, 0, false)),
        ("set stricter", table(&kinds.items, &sets[..1]), base, (Class::AskAway, Reason::Set(4), 0, 8, 1, false)),
        (
            "several matched patterns",
            table(&kinds.items, &[set(4, &[b"other/**", b"src/**", b"src/*"])]),
            base,
            (Class::AskAway, Reason::Set(4), 0, 8, 6, false),
        ),
        (
            "worded set asks",
            table(&[own(Kind::Wording, Class::Decide), item(Subject::Set(4), b"safe", Class::Ask)], &sets[..1]),
            Ask { under: 2, ..base },
            (Class::Ask, Reason::Set(4), 2, 8, 1, false),
        ),
        ("set tie lowest number", table(&kinds.items, &sets), base, (Class::AskAway, Reason::Set(2), 0, 10, 1, false)),
        ("kind wins tie", table(&[], &sets), base, (Class::AskAway, Reason::Kind, 0, 10, 0, false)),
        (
            "set less strict",
            table(&[own(Kind::Wording, Class::Ask), item(Subject::Set(4), b"", Class::Decide)], &sets[..1]),
            base,
            (Class::Ask, Reason::Kind, 0, 8, 0, false),
        ),
        (
            "worded kind",
            table(
                &[own(Kind::Wording, Class::AskAway), item(Subject::Kind(Kind::Wording), b"safe", Class::Decide)],
                &[],
            ),
            Ask { under: 2, ..base },
            (Class::Decide, Reason::Kind, 2, 0, 0, true),
        ),
        (
            "worded set",
            table(&[own(Kind::Wording, Class::Decide), item(Subject::Set(4), b"safe", Class::Decide)], &sets[..1]),
            Ask { under: 2, ..base },
            (Class::Decide, Reason::Kind, 2, 8, 0, true),
        ),
        ("under own ignored", kinds.clone(), Ask { under: 1, ..base }, (Class::Decide, Reason::Kind, 0, 0, 0, true)),
        (
            "under other kind ignored",
            table(&[own(Kind::Wording, Class::Decide), item(Subject::Kind(Kind::Retry), b"safe", Class::AskAway)], &[]),
            Ask { under: 2, ..base },
            (Class::Decide, Reason::Kind, 0, 0, 0, true),
        ),
        (
            "under untouched set ignored",
            table(
                &[own(Kind::Wording, Class::Decide), item(Subject::Set(4), b"safe", Class::Decide)],
                &[set(4, &[b"other/**"])],
            ),
            Ask { under: 2, ..base },
            (Class::Decide, Reason::Kind, 0, 0, 0, true),
        ),
        (
            "under past end ignored",
            kinds.clone(),
            Ask { under: 255, ..base },
            (Class::Decide, Reason::Kind, 0, 0, 0, true),
        ),
        (
            "always lifts decide",
            kinds.clone(),
            Ask { always: true, ..base },
            (Class::AskAway, Reason::Always, 0, 0, 0, false),
        ),
        (
            "always lifts ask",
            table(&[own(Kind::Wording, Class::Ask)], &[]),
            Ask { always: true, away: true, ..base },
            (Class::AskAway, Reason::Always, 0, 0, 0, false),
        ),
        (
            "always fixed",
            kinds.clone(),
            Ask { kind: Kind::Money, always: true, ..base },
            (Class::Wait, Reason::Kind, 0, 0, 0, false),
        ),
        (
            "always existing set",
            table(&kinds.items, &sets[..1]),
            Ask { always: true, ..base },
            (Class::AskAway, Reason::Set(4), 0, 8, 1, false),
        ),
        (
            "first own item on unchecked table",
            table(&[own(Kind::Wording, Class::Ask), own(Kind::Wording, Class::Decide)], &[]),
            base,
            (Class::Ask, Reason::Kind, 0, 0, 0, false),
        ),
    ]);
    for (name, classes, ask, (class, reason, under, sets, matched, decided)) in rows {
        let reach = judge(&classes, &ask);
        assert_eq!(reach, Reach { kind: ask.kind, class, reason, under, sets, matched, away: ask.away }, "{name}");
        assert_eq!(reach.decided(), decided, "{name}: decision");
    }
}

#[test]
fn stop_asking_makes_the_narrowest_table() {
    let paths: Box<[Box<[u8]>]> = [Box::from(b"src/a".as_slice())].into();
    let base = ask(&paths);
    let own_ask = table(&[own(Kind::Wording, Class::Ask)], &[]);
    let worded =
        table(&[own(Kind::Wording, Class::AskAway), item(Subject::Kind(Kind::Wording), b"safe", Class::Ask)], &[]);
    let core = set(4, &[b"src/**", b"include/**"]);
    let with_set = table(
        &[own(Kind::Wording, Class::Decide), item(Subject::Set(4), b"", Class::AskAway)],
        core::slice::from_ref(&core),
    );
    let mut full = with_set.clone();
    full.items = (0..ITEMS).map(|_| item(Subject::Kind(Kind::Retry), b"words", Class::Ask)).collect();
    let mut full_kind = own_ask.clone();
    full_kind.items = core::iter::once(own(Kind::Wording, Class::Ask))
        .chain((1..ITEMS).map(|_| item(Subject::Kind(Kind::Retry), b"words", Class::Ask)))
        .collect();
    for (name, classes, ask, expected) in [
        ("kind", own_ask.clone(), base, vec![Offer::Decide(Subject::Kind(Kind::Wording))]),
        ("worded kind", worded.clone(), Ask { under: 2, ..base }, vec![Offer::Decide(Subject::Kind(Kind::Wording))]),
        ("missing kind", Classes::default(), base, vec![Offer::Decide(Subject::Kind(Kind::Wording))]),
        ("partial set", with_set.clone(), base, vec![Offer::Words(4), Offer::Split(4)]),
        (
            "one pattern",
            table(&[own(Kind::Wording, Class::Decide)], &[set(4, &[b"src/**"])]),
            base,
            vec![Offer::Words(4)],
        ),
        (
            "all matched",
            table(&[own(Kind::Wording, Class::Decide)], &[set(4, &[b"src/**", b"src/*"])]),
            base,
            vec![Offer::Words(4)],
        ),
        ("full items", full.clone(), base, vec![]),
        ("full missing kind", full.clone(), Ask { kind: Kind::Tests, paths: &[], ..base }, vec![]),
        ("full existing kind", full_kind, base, vec![Offer::Decide(Subject::Kind(Kind::Wording))]),
        (
            "full sets",
            table(&with_set.items, &(1..=6).map(|n| set(n, &[b"src/**", b"include/**"])).collect::<Vec<_>>()),
            base,
            vec![Offer::Words(1)],
        ),
        ("fixed", own_ask.clone(), Ask { kind: Kind::Money, ..base }, vec![]),
        ("always", own_ask.clone(), Ask { always: true, ..base }, vec![]),
        ("decided", own_ask.clone(), Ask { away: true, ..base }, vec![]),
    ] {
        let reach = judge(&classes, &ask);
        assert_eq!(offers(&classes, &reach).as_ref(), expected, "{name}: offers");
        for offer in expected {
            let narrowed = narrowed(&classes, &reach, offer, b"safe fixes").expect("offered table");
            assert_eq!(narrowed.check(), Ok(()), "{name}: check");
            let mut next = ask;
            if let Offer::Words(_) = offer {
                next.under = u8::try_from(narrowed.items.len()).expect("item number");
            }
            if name != "full sets" {
                assert!(judge(&narrowed, &next).decided(), "{name}: decides");
            }
            match offer {
                Offer::Decide(_) => {
                    if name == "worded kind" {
                        assert_eq!(narrowed.items.first(), classes.items.first(), "own item retained");
                    }
                    if name == "missing kind" {
                        assert_eq!(narrowed.items.len(), 1, "appended");
                    }
                }
                Offer::Words(_) => {
                    assert_eq!(narrowed.items.last().expect("new item").words.as_ref(), b"safe fixes");
                }
                Offer::Split(_) => {
                    assert_eq!(narrowed.set(1).expect("lowest free").name.as_ref(), b"core, in part");
                    assert_eq!(narrowed.set(4).expect("kept").patterns.as_ref(), [Box::from(b"include/**".as_slice())]);
                    assert_eq!(
                        narrowed.items.get(..classes.items.len()),
                        Some(classes.items.as_ref()),
                        "set items retained"
                    );
                }
            }
        }
    }
    let reach = judge(&with_set, &base);
    for (name, words) in [("empty words", b"".as_slice()), ("long words", &[b'a'; WORDS_BYTES + 1])] {
        assert_eq!(narrowed(&with_set, &reach, Offer::Words(4), words), Err(Refusal::Words { item: 3 }), "{name}");
    }
    // A lower class may still ask, and a Words offer replaces the call's
    // single under. These are remaining blockers after narrowing one subject.
    for (name, classes, ask, offer) in [
        (
            "lower Ask remains",
            table(&[own(Kind::Wording, Class::Ask)], core::slice::from_ref(&core)),
            base,
            Offer::Words(4),
        ),
        ("always remains", table(&[], &[]), Ask { always: true, ..base }, Offer::Decide(Subject::Kind(Kind::Wording))),
        (
            "Words replaces kind under",
            table(
                &[own(Kind::Wording, Class::AskAway), item(Subject::Kind(Kind::Wording), b"safe", Class::Decide)],
                core::slice::from_ref(&core),
            ),
            Ask { under: 2, ..base },
            Offer::Words(4),
        ),
    ] {
        let reach = judge(&classes, &ask);
        assert!(offers(&classes, &reach).contains(&offer), "{name}: offered");
        let next = narrowed(&classes, &reach, offer, b"safe fixes").expect("narrowing");
        let under =
            if let Offer::Words(_) = offer { u8::try_from(next.items.len()).expect("new item") } else { ask.under };
        assert!(!judge(&next, &Ask { under, ..ask }).decided(), "{name}: other blocker retained");
    }
    let mut occupied = with_set.clone();
    occupied.sets = vec![core.clone(), set(1, &[b"elsewhere/**"])].into_boxed_slice();
    let reach = judge(&occupied, &base);
    let split = narrowed(&occupied, &reach, Offer::Split(4), b"").expect("split with occupied number");
    assert_eq!(split.sets.last().expect("appended set").number, 2, "lowest unoccupied number");
    for (name, original, expected) in [
        (
            "suffix cut after character",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaé",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaé,",
        ),
        ("complete character", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaé", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaé"),
    ] {
        let mut classes = with_set.clone();
        classes.sets.first_mut().expect("set").name = original.as_bytes().into();
        let reach = judge(&classes, &base);
        let result = narrowed(&classes, &reach, Offer::Split(4), b"").expect("split");
        assert_eq!(result.set(1).expect("new set").name.as_ref(), expected.as_bytes(), "{name}");
    }
}
