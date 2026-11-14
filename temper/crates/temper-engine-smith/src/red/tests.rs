//! Red decoding, refusal feedback and the whole-line answer bound (forge/reds.md, 3).
use super::{Problem, answer, decided, engine};
use engine::{RedCall as Call, RedToolAnswer as Answer};
use temper_engine_domain_forge as f;

#[test]
fn the_red_declaration_preserves_the_stock_brief_allowance() {
    let spec = crate::red::SPEC;
    let bytes = spec.name.len() + spec.description.len() + spec.schema.len();
    assert!(bytes <= 600, "red declaration: {bytes} bytes, at most 600 for the stock brief");
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::manual_string_new,
    reason = "fixtures match refusal text without naming the banned String type"
)]
fn red_acts_decode_and_refuse_wrong_members_and_text_before_deciding() {
    let head = "0123456789abcdef0123456789abcdef01234567";
    let call =
        decided(b"red", format!(r#"{{"act":"record","key":"test a::t x","at":"{head}","why":"base"}}"#).as_bytes())
            .unwrap();
    assert_eq!(
        call.tool,
        engine::Tool::Red(Call::Record {
            key: b"test a::t x".as_slice().into(),
            at: Box::new([1, 35, 69, 103, 137, 171, 205, 239, 1, 35, 69, 103, 137, 171, 205, 239, 1, 35, 69, 103]),
            why: b"base".as_slice().into()
        })
    );
    assert_eq!(
        decided(b"red", br#"{"act":"end","key":"k","why":"fixed"}"#).unwrap().tool,
        engine::Tool::Red(Call::End { key: b"k".as_slice().into(), why: b"fixed".as_slice().into() })
    );
    assert_eq!(decided(b"red", br#"{"act":"list"}"#).unwrap().tool, engine::Tool::Red(Call::List));
    for (input, member) in [
        (r#"{"act":"end","key":"k","why":"fixed","at":"…"}"#, "at"),
        (r#"{"act":"list","why":"x"}"#, "why"),
        (r#"{"act":"list","key":"x"}"#, "key"),
        (r#"{"act":"record","goal":7}"#, "goal"),
    ] {
        assert_eq!(
            decided(b"red", input.as_bytes()),
            Err(Problem::ActMember { tool: b"red", member: member.as_bytes().into() })
        );
    }
    for (input, member) in [
        (r"{}", b"act".as_slice()),
        (r#"{"act":"end"}"#, b"key"),
        (r#"{"act":"end","key":"k"}"#, b"why"),
        (r#"{"act":"record","key":"k","why":"w"}"#, b"at"),
    ] {
        assert_eq!(decided(b"red", input.as_bytes()), Err(Problem::Missing(member)));
    }
    for (input, member) in [
        (r#"{"act":0}"#, b"act".as_slice()),
        (r#"{"act":"end","key":0,"why":"w"}"#, b"key"),
        (r#"{"act":"end","key":"k","why":0}"#, b"why"),
        (r#"{"act":"record","key":"k","why":"w","at":0}"#, b"at"),
    ] {
        assert_eq!(decided(b"red", input.as_bytes()), Err(Problem::Type(member)));
    }
    for (field, bound) in [("key", 512), ("why", 256)] {
        for (text, fits) in [
            ("x".repeat(bound), true),
            ("x".repeat(bound + 1), false),
            ("é".repeat(bound / 2), true),
            ("é".repeat(bound / 2 + 1), false),
            ("".into(), false),
            (r"a\nb".into(), false),
            (r"a\rb".into(), false),
            (r"a\tb".into(), false),
            (r"a\u0000b".into(), false),
            (r"a\u007fb".into(), false),
            (r"a\u0085b".into(), false),
        ] {
            let (key, why) = if field == "key" { (text.as_str(), "w") } else { ("k", text.as_str()) };
            let decoded = decided(b"red", format!(r#"{{"act":"end","key":"{key}","why":"{why}"}}"#).as_bytes());
            assert_eq!(decoded.is_ok(), fits, "{field}: {text:?}");
            if !fits {
                let Err(Problem::Range(words)) = decoded else { panic!("named text refusal") };
                assert!(words.windows(field.len()).any(|word| word == field.as_bytes()));
            }
        }
    }
    for key in [" k", "k ", "\u{a0}k", "k\u{a0}"] {
        decided(b"red", format!(r#"{{"act":"end","key":"{key}","why":"w"}}"#).as_bytes()).unwrap_err();
    }
    for (at, fits) in [
        ("a".repeat(40), true),
        ("f".repeat(64), true),
        ("a".repeat(39), false),
        ("a".repeat(41), false),
        ("a".repeat(63), false),
        ("a".repeat(65), false),
        ("A".repeat(40), false),
        ("g".repeat(40), false),
        ("0".repeat(38) + "é", false),
    ] {
        let decoded = decided(b"red", format!(r#"{{"act":"record","key":"k","why":"w","at":"{at}"}}"#).as_bytes());
        assert_eq!(decoded.is_ok(), fits, "at: {at}");
        if !fits {
            assert_eq!(decoded, Err(Problem::Range(b"red: at must be 40 or 64 lowercase hexadecimal digits")));
        }
    }
}
fn row(key: &[u8]) -> f::Red {
    f::Red {
        goal: 7,
        key: key.into(),
        head: Box::new([0xab; 20]),
        why: "base é".as_bytes().into(),
        by: f::Recorder::Task(7),
        at: 1_709_251_140_000_000_000,
        generation: 1,
    }
}
fn write(result: Result<f::RedOutcome, f::RedRefusal>) -> engine::CallAnswer {
    engine::CallAnswer::Red(Box::new(Answer::Write(f::RedReply {
        goal: 7,
        key: b"test a::t x".as_slice().into(),
        result,
    })))
}
#[test]
#[expect(clippy::manual_string_new, reason = "fixtures infer their empty text without naming the banned String type")]
fn red_answers_use_the_tools_words_and_mark_refusals_as_errors() {
    let head = "ab".repeat(20);
    for (replaced, tail) in [
        (None, "".into()),
        (Some(row(b"test a::t x")), format!(" It replaced the one recorded at {head} (generation 1).")),
    ] {
        let rendered = answer(&write(Ok(f::RedOutcome::Recorded {
            head: Box::new([0xcd; 32]),
            recorded: f::RedRecorded { generation: 2, replaced },
        })));
        assert_eq!(
            rendered.text(),
            format!(
                "Recorded red test a::t x at {} (generation 2); a check whose base contains {} passes it.{tail}",
                "cd".repeat(32),
                "cd".repeat(32)
            )
            .as_bytes()
        );
        assert!(!rendered.error());
    }
    for (by, who) in [(f::Recorder::Task(7), "task 7"), (f::Recorder::Party(8), "party 8")] {
        let rendered = answer(&write(Ok(f::RedOutcome::Ended(f::Red { by, ..row(b"test a::t x") }))));
        assert_eq!(
            rendered.text(),
            format!("Ended red test a::t x, recorded at {head} by {who} on 2024-02-29 23:59: base é.").as_bytes()
        );
        assert!(!rendered.error());
    }
    for (why, expected) in [
        (f::RedRefusal::NoGoal, "red: task 7 holds no goal branch; only a goal's run keeps reds"),
        (f::RedRefusal::NoSuch, "red: goal 7 holds no red test a::t x; red list shows what it holds"),
        (
            f::RedRefusal::Full { bound: 2 },
            "red: goal 7 holds its most reds (at most 2); end one, or look at what its branch carries",
        ),
    ] {
        let rendered = answer(&write(Err(why)));
        assert_eq!(rendered.text(), expected.as_bytes());
        assert!(rendered.error());
    }
    let rendered = answer(&engine::CallAnswer::Red(Box::new(Answer::NoGoal { task: 7 })));
    assert_eq!(rendered.text(), b"red: task 7 holds no goal branch; only a goal's run keeps reds");
    assert!(rendered.error());
}
#[test]
#[expect(clippy::disallowed_methods, reason = "the test checks that bounded replies preserve UTF-8")]
fn red_list_fits_whole_lines_and_counts_only_omitted_reds() {
    let answer = engine::CallAnswer::Red(Box::new(Answer::List {
        goal: 7,
        bound: 3,
        reds: Box::new([row(b"a"), row(b"b"), row(b"c")]),
    }));
    let first = format!(
        "Goal 7 holds 3 reds (at most 3):\na at {}, generation 1, by task 7, 2024-02-29 23:59: base é\nand 2 more",
        "ab".repeat(20)
    );
    let rendered = crate::answer_bounded(&answer, u32::try_from(first.len()).unwrap());
    assert_eq!(rendered.text(), first.as_bytes());
    assert!(!rendered.error());
    let whole = crate::answer(&answer);
    assert_eq!(core::str::from_utf8(whole.text()).unwrap().lines().count(), 4);
    for bound in 0..=whole.text().len() {
        let rendered = crate::answer_bounded(&answer, u32::try_from(bound).unwrap());
        assert!(rendered.text().len() <= bound);
        core::str::from_utf8(rendered.text()).unwrap();
        assert!(!rendered.error());
    }
    assert_eq!(crate::answer_bounded(&answer, 10).text(), b"and 3 more");
    assert_eq!(
        crate::answer(&engine::CallAnswer::Red(Box::new(Answer::List { goal: 7, bound: 3, reds: Box::default() })))
            .text(),
        b"Goal 7 holds no reds."
    );
}
