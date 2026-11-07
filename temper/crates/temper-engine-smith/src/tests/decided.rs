//! The decision tool's bounds and words (temper/docs/design/domain/agent.md, 4.3).
use super::{Problem, decided, engine};
use temper_engine_domain_forge as forge;

fn input(question: &str, options: &[&str], took: u64, why: &str, kind: &str) -> Box<[u8]> {
    format!(r#"{{"question":"{question}","options":{options:?},"took":{took},"why":"{why}","kind":"{kind}"}}"#)
        .into_bytes()
        .into_boxed_slice()
}

#[test]
fn decided_decodes() {
    for (kind, expected) in [
        ("implementation", forge::DecisionKind::Implementation),
        ("retry", forge::DecisionKind::Retry),
        ("wording", forge::DecisionKind::Wording),
        ("dependency", forge::DecisionKind::Dependency),
        ("other", forge::DecisionKind::Other),
    ] {
        let call = decided(b"decided", &input("Which?", &["A", "B"], 1, "Fits", kind)).expect("decision decodes");
        assert_eq!(
            call.tool,
            engine::Tool::Decided(forge::DecidedCall {
                question: Box::from(b"Which?".as_slice()),
                options: Box::new([Box::from(b"A".as_slice()), Box::from(b"B".as_slice())]),
                took: 1,
                why: Box::from(b"Fits".as_slice()),
                kind: expected,
            })
        );
    }
    assert_eq!(
        decided(b"decided", &input("Q", &["A", "B"], 0, "W", "unknown")),
        Err(Problem::Range(b"kind: implementation, retry, wording, dependency or other"))
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
    assert!(skein_lib::bytes::find(schema, br#""additionalProperties":false"#).is_some());
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
    for (took, words) in [
        (Some(Box::from(b"A".as_slice())), b"Decided: carry on with A.".as_slice()),
        (None, b"Ask: your person is here or this kind waits for them. Ask with message, as a question; if your brief says [Requested by person N], with delegate, a person task addressed to person N."),
    ] {
        let answer = crate::answer(&engine::CallAnswer::Decided { took });
        assert_eq!(answer.text(), words);
        assert!(!answer.error());
    }
}
