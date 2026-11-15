const RESOLUTION_PROGRESS: &[(crate::builder::vocabulary::Progress, crate::records::records::Record)] = &[
    (
        crate::builder::vocabulary::Progress::Resolving { key: crate::tests::fixtures::KEY },
        crate::records::records::Record::Resolving(crate::records::progress::Resolving { key: [1; 32] }),
    ),
    (
        crate::builder::vocabulary::Progress::Resolved {
            key: crate::tests::fixtures::KEY,
            took: skein_lib::Duration::from_millis(7),
        },
        crate::records::records::Record::Resolved(crate::records::progress::Resolved {
            key: [1; 32],
            took: skein_lib::Duration::from_millis(7),
        }),
    ),
    (
        crate::builder::vocabulary::Progress::Unresolved { key: crate::tests::fixtures::KEY },
        crate::records::records::Record::Unresolved(crate::records::progress::Unresolved { key: [1; 32] }),
    ),
];
#[test]
fn resolution_progress_keeps_its_key_and_duration() {
    for (row, (input, expected)) in RESOLUTION_PROGRESS.iter().enumerate() {
        assert_eq!(&crate::translate::store::progress(input.clone(), b"/store"), expected, "row {row}");
    }
}
