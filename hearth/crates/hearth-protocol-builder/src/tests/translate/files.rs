use crate::{records, render::errors};

const FAULTS: &[(errors::Repository, records::errors::Repository)] = &[
    (errors::Repository::NoTemplate, records::errors::Repository::NoTemplate),
    (errors::Repository::Unused, records::errors::Repository::Unused),
    (errors::Repository::NoTree, records::errors::Repository::NoTree),
    (errors::Repository::LinkTarget, records::errors::Repository::LinkTarget),
];
#[test]
fn each_file_repository_fault_translates_to_the_records() {
    for (row, (input, expected)) in FAULTS.iter().enumerate() {
        assert_eq!(crate::translate::errors::repository(input.clone()), *expected, "row {row}");
    }
}
#[test]
fn a_source_fault_names_both_fields_in_the_record() {
    assert_eq!(
        crate::translate::errors::schema(errors::Schema::Source {
            first: crate::render::revision::declaration::Name::Link,
            second: crate::render::revision::declaration::Name::Directory
        }),
        records::errors::Schema::Source { first: b"link".as_slice().into(), second: b"directory".as_slice().into() }
    );
}
