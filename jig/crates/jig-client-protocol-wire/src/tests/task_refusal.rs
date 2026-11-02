//! Refusals travel inside whole task documents; wrappers add no framing.
use crate::v19 as w;
use skein_lib::{Reader, Writer};

#[test]
fn refused_task_documents_reject_truncated_or_unknown_nested_tags() {
    let limits = super::fixtures::limits();
    // Outer refusal, hello arm, worker kind, receipt time, reserved-channel cause.
    let wait_bytes = [3, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0, 1];
    let wait = w::TaskRunWait::decode(&limits, &mut Reader::new(&wait_bytes)).expect("hello wait");
    assert_eq!(wait.measure(), 12, "one-field records have no separate header");
    let mut parts = super::fixtures::task(&limits).into_parts();
    parts.run_wait = Some(wait);
    parts.lineage = skein_lib::List::with_capacity(1);
    parts.lineage.push(77).expect("one ancestor");
    let record = w::Task::new(&limits, parts).expect("whole task");
    let mut out = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut out).expect("measured writer");
    let bytes = out.finish();
    let mut reader = Reader::new(&bytes);
    assert_eq!(w::Task::decode(&limits, &mut reader), Ok(record));
    assert_eq!(reader.remaining(), 0);
    // The task's dependency facts, an empty list's four-byte count, follow its wait.
    let offset = bytes.len() - wait_bytes.len() - 4;
    assert_eq!(&bytes[offset..offset + wait_bytes.len()], wait_bytes);
    assert_eq!(&bytes[offset + wait_bytes.len()..], [0; 4]);
    for end in offset..bytes.len() {
        assert_eq!(
            w::Task::decode(&limits, &mut Reader::new(&bytes[..end])).expect_err("truncated refusal").reason,
            skein_codec::Reason::Short
        );
    }
    for tag in [0, 1, 2, 11] {
        let mut invalid = bytes.clone();
        invalid[offset + tag] = 255;
        assert_eq!(
            w::Task::decode(&limits, &mut Reader::new(&invalid)).expect_err("unknown nested tag").reason,
            skein_codec::Reason::Tag
        );
    }
    // Bound validation still applies before a refusal-bearing document is admitted.
    let smaller = w::Limits { task_lineage: 0, ..limits };
    assert_eq!(
        w::Task::decode(&smaller, &mut Reader::new(&bytes)).expect_err("receiver lineage bound").reason,
        skein_codec::Reason::Bound
    );
}
