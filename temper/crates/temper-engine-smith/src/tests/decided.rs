//! The decision tool's bounds and words (temper/docs/design/domain/agent/tools.md).
use super::{Problem, decided, engine};
use temper_engine_domain_forge as forge;

fn input(question: &str, options: &[&str], took: u64, why: &str, kind: &str) -> Box<[u8]> {
    format!(r#"{{"question":"{question}","options":{options:?},"took":{took},"why":"{why}","kind":"{kind}"}}"#)
        .into_bytes()
        .into_boxed_slice()
}

fn with_fields(fields: &str) -> Box<[u8]> {
    format!(r#"{{"question":"Q","options":["A","B"],"took":0,"why":"W","kind":"tests",{fields}}}"#)
        .into_bytes()
        .into_boxed_slice()
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the decoder test converts the kit's ASCII kind names into JSON strings")]
fn decided_decodes() {
    for expected in temper_protocol_reaches::Kind::ALL {
        let kind = core::str::from_utf8(expected.name()).expect("kind name");
        let call = decided(b"decided", &input("Which?", &["A", "B"], 1, "Fits", kind)).expect("decision decodes");
        assert_eq!(
            call.tool,
            engine::Tool::Decided(forge::DecidedCall {
                question: Box::from(b"Which?".as_slice()),
                options: Box::new([Box::from(b"A".as_slice()), Box::from(b"B".as_slice())]),
                took: 1,
                why: Box::from(b"Fits".as_slice()),
                kind: expected,
                paths: Box::new([]),
                under: 0,
            })
        );
    }
    assert_eq!(
        decided(b"decided", &input("Q", &["A", "B"], 0, "W", "unknown")),
        Err(Problem::Range(
            b"kind: wording, retry, implementation, tests, dependency, departure, money, irreversible, rules or other"
        ))
    );
    let original = r#"{"question":"Q","options":["A","B"],"took":0,"why":"W","kind":"retry"}"#;
    for (field, member) in [
        (b"question".as_slice(), "\"question\":\"Q\","),
        (b"options", "\"options\":[\"A\",\"B\"],"),
        (b"took", "\"took\":0,"),
        (b"why", "\"why\":\"W\","),
        (b"kind", ",\"kind\":\"retry\""),
    ] {
        assert_eq!(decided(b"decided", original.replace(member, "").as_bytes()), Err(Problem::Missing(field)));
    }
    let mut schema = None;
    for spec in &crate::calls::SPECS {
        if spec.name == b"decided" {
            schema = Some(spec.schema);
        }
    }
    let schema = schema.expect("decided is declared");
    let schema = crate::json::parse(schema).expect("schema JSON");
    assert_eq!(schema.required(b"additionalProperties").unwrap(), &crate::json::Value::Boolean(false));
    let properties = schema.required(b"properties").unwrap();
    let kinds = properties.required(b"kind").unwrap().required(b"enum").unwrap().array(b"enum").unwrap();
    assert_eq!(kinds.len(), temper_protocol_reaches::Kind::ALL.len());
    for (value, kind) in kinds.iter().zip(temper_protocol_reaches::Kind::ALL) {
        assert_eq!(value.text(b"kind").unwrap().as_ref(), kind.name());
    }
    let paths = properties.required(b"paths").unwrap();
    assert_eq!(paths.required(b"maxItems").unwrap().number(b"maxItems").unwrap(), 16);
    assert_eq!(paths.required(b"items").unwrap().required(b"type").unwrap().text(b"type").unwrap().as_ref(), b"string");
    let under = properties.required(b"under").unwrap();
    assert_eq!(under.required(b"minimum").unwrap().number(b"minimum").unwrap(), 1);
    assert_eq!(under.required(b"maximum").unwrap().number(b"maximum").unwrap(), 20);
    for under in [1, 20] {
        let fields = format!(r#""paths":{:?},"under":{under}"#, vec!["x".repeat(256); 16]);
        let decoded = decided(b"decided", &with_fields(&fields)).expect("at path and item bounds");
        let engine::Tool::Decided(call) = decoded.tool else { panic!("decided") };
        assert_eq!(call.paths.len(), 16);
        assert_eq!(call.paths[0].as_ref(), "x".repeat(256).as_bytes());
        assert_eq!(u64::from(call.under), under);
    }
}

#[test]
fn decided_refuses_past_its_bounds() {
    let question = "q".repeat(forge::DECIDED_QUESTION_BYTES);
    let option = "o".repeat(forge::DECIDED_OPTION_BYTES);
    let why = "w".repeat(forge::DECIDED_WHY_BYTES);
    for options in [vec![option.as_str(); 2], vec![option.as_str(); 6]] {
        decided(b"decided", &input(&question, &options, 0, &why, "retry")).expect("at all byte and count bounds");
    }
    for (question, options, why, expected) in [
        (format!("{question}q"), vec![option.as_str(); 2], why.clone(), b"question: at most 400 bytes".as_slice()),
        (question.clone(), vec![option.as_str(); 1], why.clone(), b"options: two to six options"),
        (question.clone(), vec![option.as_str(); 7], why.clone(), b"options: two to six options"),
        (question.clone(), vec![option.as_str(); 2], format!("{why}w"), b"why: at most 400 bytes"),
    ] {
        assert_eq!(decided(b"decided", &input(&question, &options, 0, &why, "retry")), Err(Problem::Range(expected)));
    }
    assert_eq!(
        decided(b"decided", &input("Q", &[&format!("{option}o"), "B"], 0, "W", "retry")),
        Err(Problem::Range(b"options: each at most 120 bytes"))
    );
    // Two-byte UTF-8 characters count as bytes.
    assert_eq!(
        decided(b"decided", &input(&"é".repeat(201), &["A", "B"], 0, "W", "retry")),
        Err(Problem::Range(b"question: at most 400 bytes"))
    );
    for (fields, reason) in [
        (r#""paths":[""]"#.into(), b"paths: a path is empty".as_slice()),
        (format!(r#""paths":["{}"]"#, "é".repeat(129)), b"paths: each at most 256 bytes"),
        (r#""paths":["/src/core.rs"]"#.into(), b"paths: repository-relative paths, no leading /"),
        (format!(r#""paths":{:?}"#, vec!["a"; 17]), b"paths: at most 16 paths"),
        (r#""under":0"#.into(), b"under: item number 1 to 20"),
        (r#""under":21"#.into(), b"under: item number 1 to 20"),
        (format!(r#""under":{}"#, u64::MAX), b"under: item number 1 to 20"),
    ] {
        assert_eq!(decided(b"decided", &with_fields(&fields)), Err(Problem::Range(reason)));
    }
}

#[test]
fn decided_refuses_a_took_outside_its_options() {
    for took in [2, 6, 256, u64::MAX] {
        assert_eq!(
            decided(b"decided", &input("Q", &["A", "B"], took, "W", "retry")),
            Err(Problem::Range(b"took: index into options, starting at zero"))
        );
    }
}

#[test]
fn its_answers_read_as_the_design_says() {
    use temper_protocol_reaches::{Class, Kind, Reach, Reason};
    for (took, reach, reason) in [
        (Some(Box::from(b"A".as_slice())), None, ""),
        (None, None, "no person is asked through you"),
        (None, Some((Kind::Money, Class::Wait, Reason::Kind, true)), "money always waits for your person"),
        (
            None,
            Some((Kind::Implementation, Class::AskAway, Reason::Kind, true)),
            "implementation asks your person, even when away",
        ),
        (
            None,
            Some((Kind::Implementation, Class::Ask, Reason::Kind, false)),
            "implementation asks your person while they are here",
        ),
        (
            None,
            Some((Kind::Wording, Class::AskAway, Reason::Set(2), true)),
            "changes to path set 2 ask your person, even when away",
        ),
        (
            None,
            Some((Kind::Wording, Class::Ask, Reason::Set(1), false)),
            "changes to path set 1 ask your person while they are here",
        ),
        (None, Some((Kind::Wording, Class::AskAway, Reason::Always, true)), "your words say Asks: always"),
        (
            None,
            Some((Kind::Wording, Class::Decide, Reason::Kind, false)),
            "your person's decided list is full for today",
        ),
        (
            None,
            Some((Kind::Implementation, Class::Ask, Reason::Kind, true)),
            "your person's decided list is full for today",
        ),
    ] {
        let reach = match reach {
            Some((kind, class, reason, away)) => {
                Some(Reach { kind, class, reason, under: 0, sets: 0, matched: 0, away })
            }
            None => None,
        };
        for how in [engine::AskHow::Message, engine::AskHow::Person(17), engine::AskHow::None] {
            let words = if took.is_some() {
                "Decided: carry on with A.".into()
            } else {
                match how {
                    engine::AskHow::Message => {
                        format!("Ask: {reason}. Ask your requester with message, as a question.")
                    }
                    engine::AskHow::Person(person) => {
                        format!("Ask: {reason}. Ask person {person} with delegate, a person task addressed to them.")
                    }
                    engine::AskHow::None => format!(
                        "Not decided: {reason}. Nothing you hold asks anyone: decide within your brief, or leave the choice and your recommendation in your result."
                    ),
                }
            };
            let answer = crate::answer_for_decided(&engine::CallAnswer::Decided { took: took.clone(), reach }, how);
            assert_eq!(answer.text(), words.as_bytes());
            assert!(!answer.error());
        }
    }
}
