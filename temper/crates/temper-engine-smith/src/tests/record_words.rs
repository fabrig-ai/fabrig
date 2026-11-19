//! Run acts record words, through the message family or a written note.
use super::{decided, note_input};
use crate::{Problem, calls::SPECS};
use temper_engine_domain::{CallAnswer, Tool};

fn spec(name: &[u8]) -> &crate::calls::Spec {
    for spec in &SPECS {
        if spec.name == name {
            return spec;
        }
    }
    panic!("declared tool")
}

#[test]
fn mark_read_takes_positive_nonempty_message_numbers() {
    assert_eq!(
        decided(b"mark_read", br#"{"messages":[77,5]}"#).unwrap().tool,
        Tool::MarkRead { messages: Box::new([77, 5]) }
    );
    for input in [br#"{"messages":[]}"#.as_slice(), br#"{"messages":[0]}"#, br#"{"messages":[-1]}"#] {
        assert!(decided(b"mark_read", input).is_err(), "{input:?}");
    }
    assert_eq!(decided(b"mark_read", b"{}"), Err(Problem::Missing(b"messages")));
    let declaration = spec(b"mark_read");
    let schema = crate::json::parse(declaration.schema).unwrap();
    let messages = schema.required(b"properties").unwrap().required(b"messages").unwrap();
    assert_eq!(messages.required(b"minItems").unwrap().number(b"minItems").unwrap(), 1);
    assert_eq!(messages.required(b"items").unwrap().required(b"minimum").unwrap().number(b"minimum").unwrap(), 1);
    assert_eq!(declaration.effect, smith_domain_run::HostEffect::Write);
    assert_eq!(
        declaration.description,
        b"Record words your task keeps unread, once a spec or a note you wrote holds them."
    );
}

#[test]
fn note_records_are_optional_positive_numbers_on_both_forms() {
    let declaration = spec(b"note");
    for schema in [declaration.schema, declaration.whole()] {
        let schema = crate::json::parse(schema).unwrap();
        let records = schema.required(b"properties").unwrap().required(b"records").unwrap();
        assert_eq!(records.required(b"type").unwrap().text(b"type").unwrap().as_ref(), b"array");
        assert_eq!(records.required(b"items").unwrap().required(b"minimum").unwrap().number(b"minimum").unwrap(), 1);
    }
    for range in ["", r#", "name":20,"recalled":1,"first":1,"remove":0"#] {
        for (suffix, expected) in [("", &[][..]), (r#", "records":[]"#, &[][..]), (r#", "records":[77]"#, &[77][..])] {
            let call = decided(b"note", &note_input(&format!("{range}{suffix}"))).unwrap();
            let records = if range.is_empty() {
                let Tool::Note { records, .. } = call.tool else { panic!("whole note") };
                records
            } else {
                let Tool::NoteLines { records, .. } = call.tool else { panic!("line note") };
                records
            };
            assert_eq!(records.as_ref(), expected);
        }
        assert_eq!(
            decided(b"note", &note_input(&format!(r#"{range}, "records":[0]"#))),
            Err(Problem::Range(b"records"))
        );
    }
}

#[test]
fn recording_answers_name_both_parts_and_omit_empty_parts() {
    for (recorded, unknown, words) in [
        (&[77][..], &[5][..], "recorded: 77\nnamed none of your unrecorded words: 5"),
        (&[77][..], &[][..], "recorded: 77"),
        (&[][..], &[5][..], "named none of your unrecorded words: 5"),
        (&[][..], &[][..], ""),
    ] {
        let marked = CallAnswer::WordsRecorded { recorded: recorded.into(), unknown: unknown.into() };
        let rendered = crate::answer(&marked);
        assert!(!rendered.error());
        assert_eq!(rendered.text(), words.as_bytes());
        assert_eq!(CallAnswer::from_core(&marked.core_part(0)), Some(marked));
        let written = CallAnswer::NoteWritten {
            name: 12,
            revision: 1,
            lines: 1,
            added: 1,
            removed: 0,
            first: 1,
            recorded: recorded.into(),
            unknown: unknown.into(),
        };
        let rendered = crate::answer(&written);
        assert!(!rendered.error());
        let prefix = "Note 12 revision 1 written · 1 lines";
        if words.is_empty() {
            assert_eq!(rendered.text(), prefix.as_bytes());
        } else {
            assert_eq!(rendered.text(), format!("{prefix}\n{words}").as_bytes());
        }
        assert_eq!(CallAnswer::from_core(&written.core_part(0)), Some(written));
    }
}
