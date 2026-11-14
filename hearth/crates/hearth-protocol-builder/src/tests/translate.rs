use crate::{
    config::document as source,
    git_machine::listing,
    render::revision as target,
    translate::{documents, listing as entries},
};

const MODES: &[(listing::Mode, target::Mode)] = &[
    (listing::Mode::File, target::Mode::File),
    (listing::Mode::Executable, target::Mode::Executable),
    (listing::Mode::Link, target::Mode::Link),
    (listing::Mode::Submodule, target::Mode::Submodule),
];
fn mode_back(mode: target::Mode) -> listing::Mode {
    match mode {
        target::Mode::File => listing::Mode::File,
        target::Mode::Executable => listing::Mode::Executable,
        target::Mode::Link => listing::Mode::Link,
        target::Mode::Submodule => listing::Mode::Submodule,
    }
}
#[test]
fn each_mode_translates_and_back() {
    for (source, target) in MODES {
        assert_eq!(entries::mode_to_render(*source), *target, "mode {source:?}");
        assert_eq!(mode_back(*target), *source, "mode {target:?}");
    }
}
const KINDS: &[(source::Runs, target::Runs)] = &[
    (source::Runs::Metal, target::Runs::Metal),
    (source::Runs::Vm, target::Runs::Vm),
    (source::Runs::Container, target::Runs::Container),
];
fn kind_back(kind: target::Runs) -> source::Runs {
    match kind {
        target::Runs::Metal => source::Runs::Metal,
        target::Runs::Vm => source::Runs::Vm,
        target::Runs::Container => source::Runs::Container,
    }
}
#[test]
fn each_placement_kind_translates_and_back() {
    for (source, target) in KINDS {
        assert_eq!(documents::runs(*source), *target, "kind {source:?}");
        assert_eq!(kind_back(*target), *source, "kind {target:?}");
    }
}
const FIRMWARE: &[(source::Firmware, target::Firmware)] =
    &[(source::Firmware::Efi, target::Firmware::Efi), (source::Firmware::Bios, target::Firmware::Bios)];
fn firmware_back(firmware: target::Firmware) -> source::Firmware {
    match firmware {
        target::Firmware::Efi => source::Firmware::Efi,
        target::Firmware::Bios => source::Firmware::Bios,
    }
}
#[test]
fn each_firmware_translates_and_back() {
    for (source, target) in FIRMWARE {
        assert_eq!(documents::boot(*source), *target, "firmware {source:?}");
        assert_eq!(firmware_back(*target), *source, "firmware {target:?}");
    }
}

#[test]
fn a_containers_placement_keeps_its_host_position_and_mutability() {
    let input = source::Placement {
        at: crate::config::Position { offset: 7, line: 2, column: 4 },
        kind: source::Runs::Container,
        firmware: None,
        hardware: None,
        on: Some(source::On {
            target: skein_lib::bytes::copy_of(b"x1-prod"),
            at: crate::config::Position { offset: 17, line: 3, column: 8 },
        }),
        mutable: true,
    };
    assert_eq!(
        documents::placement_to_render(input),
        target::Placement {
            at: target::Position { line: 2, column: 4 },
            kind: target::Runs::Container,
            firmware: None,
            hardware: None,
            on: Some(target::On {
                target: skein_lib::bytes::copy_of(b"x1-prod"),
                at: target::Position { line: 3, column: 8 }
            }),
            mutable: true,
        },
        "container placement fields"
    );
}
#[test]
fn a_document_without_placement_keeps_its_position_and_absence() {
    let input = source::Document {
        members: None,
        at: crate::config::Position { offset: 7, line: 2, column: 4 },
        layer: source::Layer { tasks: Box::new([]), debian: None, placement: None },
    };
    assert_eq!(
        documents::document(input),
        target::Document {
            members: None,
            at: target::Position { line: 2, column: 4 },
            layer: target::Layer { debian: None, placement: None, tasks: Box::new([]) }
        },
        "no placement is introduced"
    );
}

use crate::{records, render, translate::answers};
/// A build's unavailable step is the client's outcome alone, never render's.
fn outcome_back(outcome: records::outcome::Outcome) -> Option<render::vocabulary::Outcome> {
    Some(match outcome {
        records::outcome::Outcome::Done => render::vocabulary::Outcome::Done,
        records::outcome::Outcome::Failed => render::vocabulary::Outcome::Failed,
        records::outcome::Outcome::Usage => render::vocabulary::Outcome::Usage,
        records::outcome::Outcome::TryAgain => render::vocabulary::Outcome::TryAgain,
        records::outcome::Outcome::Negative => render::vocabulary::Outcome::Negative,
        records::outcome::Outcome::Unavailable => return None,
    })
}
const CLIENT_OUTCOMES: &[(render::vocabulary::Outcome, records::outcome::Outcome)] = &[
    (render::vocabulary::Outcome::Done, records::outcome::Outcome::Done),
    (render::vocabulary::Outcome::Failed, records::outcome::Outcome::Failed),
    (render::vocabulary::Outcome::Usage, records::outcome::Outcome::Usage),
    (render::vocabulary::Outcome::TryAgain, records::outcome::Outcome::TryAgain),
    (render::vocabulary::Outcome::Negative, records::outcome::Outcome::Negative),
];
#[test]
fn each_client_outcome_translates_and_back() {
    for (source, expected) in CLIENT_OUTCOMES {
        assert_eq!(answers::outcome(*source), *expected, "outcome {source:?}");
        assert_eq!(outcome_back(*expected), Some(*source), "outcome {expected:?}");
    }
}

const VALUES: &[(crate::config::json::Kind, target::Kind)] = &[
    (crate::config::json::Kind::Null, target::Kind::Null),
    (crate::config::json::Kind::Bool(true), target::Kind::Bool(true)),
    (crate::config::json::Kind::Int(42), target::Kind::Int(42)),
];
#[test]
fn scalar_configuration_values_keep_their_kind_and_position() {
    for (kind, expected) in VALUES {
        let value = documents::value(crate::config::json::Value {
            at: crate::config::Position { offset: 17, line: 3, column: 8 },
            kind: kind.clone(),
        });
        assert_eq!(value.kind, *expected);
        assert_eq!(value.at, target::Position { line: 3, column: 8 });
    }
}
#[test]
fn nested_configuration_values_keep_their_keys_lists_and_string_bytes() {
    let at = crate::config::Position { offset: 0, line: 1, column: 1 };
    let value = crate::config::json::Value {
        at,
        kind: crate::config::json::Kind::Map(Box::new([crate::config::json::Entry {
            key: skein_lib::bytes::copy_of(b"a"),
            at,
            value: crate::config::json::Value {
                at,
                kind: crate::config::json::Kind::List(Box::new([crate::config::json::Value {
                    at,
                    kind: crate::config::json::Kind::String(skein_lib::bytes::copy_of(b"hello")),
                }])),
            },
        }])),
    };
    assert_eq!(
        documents::value(value),
        target::Value {
            at: target::Position { line: 1, column: 1 },
            kind: target::Kind::Map(Box::new([target::Pair {
                key: skein_lib::bytes::copy_of(b"a"),
                at: target::Position { line: 1, column: 1 },
                value: target::Value {
                    at: target::Position { line: 1, column: 1 },
                    kind: target::Kind::List(Box::new([target::Value {
                        at: target::Position { line: 1, column: 1 },
                        kind: target::Kind::String(skein_lib::bytes::copy_of(b"hello"))
                    }]))
                },
            }])),
        }
    );
}

use crate::render::diff;
const STORE_RECORDS: &[(crate::tree::store::Store, crate::builder::store::vocabulary::StoreRecord)] = &[
    (
        crate::tree::store::Store { layout: 1, made: skein_lib::Wall::from_nanos(77), range: None, uid: 1000 },
        crate::builder::store::vocabulary::StoreRecord {
            layout: 1,
            made: skein_lib::Wall::from_nanos(77),
            range: None,
            uid: 1000,
        },
    ),
    (
        crate::tree::store::Store {
            layout: 2,
            made: skein_lib::Wall::from_nanos(99),
            range: Some(crate::tree::store::Range { base: 100_000, count: 65_536 }),
            uid: 1001,
        },
        crate::builder::store::vocabulary::StoreRecord {
            layout: 2,
            made: skein_lib::Wall::from_nanos(99),
            range: Some(crate::builder::store::vocabulary::Range { base: 100_000, count: 65_536 }),
            uid: 1001,
        },
    ),
];
#[test]
fn each_store_record_field_translates_and_back() {
    for &(record, translated) in STORE_RECORDS {
        assert_eq!(crate::translate::store::store_record(record), translated, "{record:?}");
        assert_eq!(crate::translate::store::tree_store(translated), record, "{translated:?}");
    }
}
mod errors;

mod back {
    use crate::render::evaluate as source;
    use hearth_protocol_template as target;
    use skein_lib::List;
    #[must_use]
    pub fn expression(expression: source::Expression) -> target::Expression {
        let source::Expression { operations, values } = expression;
        let mut translated = List::with_capacity(u32::try_from(operations.len()).expect("bounded operations"));
        for item in operations {
            translated.push(operation(item)).expect("operation room");
        }
        target::Expression { operations: translated.into_boxed(), values }
    }
    #[must_use]
    pub fn operation(operation: source::Operation) -> target::Operation {
        let source::Operation { at, op: value } = operation;
        target::Operation { at, op: op(value) }
    }
    #[must_use]
    pub fn op(op: source::Op) -> target::Op {
        match op {
            source::Op::String(v) => target::Op::String(v),
            source::Op::Integer(v) => target::Op::Integer(v),
            source::Op::Boolean(v) => target::Op::Boolean(v),
            source::Op::Name { name, missing: mark } => target::Op::Name { name, missing: missing(mark) },
            source::Op::Key { name, missing: mark } => target::Op::Key { name, missing: missing(mark) },
            source::Op::Item { missing: mark } => target::Op::Item { missing: missing(mark) },
            source::Op::List { length } => target::Op::List { length },
            source::Op::Filter(value) => target::Op::Filter(filter(value)),
            source::Op::Not => target::Op::Not,
            source::Op::Equal => target::Op::Equal,
            source::Op::NotEqual => target::Op::NotEqual,
            source::Op::Less => target::Op::Less,
            source::Op::LessOrEqual => target::Op::LessOrEqual,
            source::Op::Greater => target::Op::Greater,
            source::Op::GreaterOrEqual => target::Op::GreaterOrEqual,
            source::Op::In => target::Op::In,
            source::Op::NotIn => target::Op::NotIn,
            source::Op::Concat => target::Op::Concat,
            source::Op::Add => target::Op::Add,
            source::Op::Subtract => target::Op::Subtract,
            source::Op::Defined => target::Op::Defined,
            source::Op::NotDefined => target::Op::NotDefined,
            source::Op::JumpIfFalseKeeping { over } => target::Op::JumpIfFalseKeeping { over },
            source::Op::JumpIfTrueKeeping { over } => target::Op::JumpIfTrueKeeping { over },
            source::Op::JumpIfFalse { over } => target::Op::JumpIfFalse { over },
            source::Op::Jump { over } => target::Op::Jump { over },
        }
    }
    #[must_use]
    pub fn missing(missing: source::Missing) -> target::Missing {
        match missing {
            source::Missing::Error => target::Missing::Error,
            source::Missing::Undefined => target::Missing::Undefined,
        }
    }
    #[must_use]
    pub fn filter(filter: source::Filter) -> target::Filter {
        match filter {
            source::Filter::Default => target::Filter::Default,
            source::Filter::Join => target::Filter::Join,
            source::Filter::Split => target::Filter::Split,
            source::Filter::First => target::Filter::First,
            source::Filter::Last => target::Filter::Last,
            source::Filter::Length => target::Filter::Length,
            source::Filter::Replace => target::Filter::Replace,
            source::Filter::Items => target::Filter::Items,
            source::Filter::Keys => target::Filter::Keys,
            source::Filter::Values => target::Filter::Values,
            source::Filter::Sort => target::Filter::Sort,
            source::Filter::Tojson => target::Filter::Tojson,
            source::Filter::ShellQuote => target::Filter::ShellQuote,
            source::Filter::SystemdEscape => target::Filter::SystemdEscape,
            source::Filter::SystemdEscapePath => target::Filter::SystemdEscapePath,
            source::Filter::Basename => target::Filter::Basename,
            source::Filter::Dirname => target::Filter::Dirname,
        }
    }
}

use crate::translate::operations;
use hearth_protocol_template as template;
#[test]
fn each_operation_translates_to_render_and_back() {
    for (row, op) in super::fixtures::every_op().into_iter().enumerate() {
        let back = back::op(operations::op(op.clone()));
        assert_eq!(back, op, "row {row}: hearth 05's Op through render's and back");
    }
}
const MISSING: &[template::Missing] = &[template::Missing::Error, template::Missing::Undefined];
#[test]
fn missing_translates_to_render_and_back() {
    for &missing in MISSING {
        assert_eq!(back::missing(operations::missing(missing)), missing, "{missing:?}");
    }
}
const FILTERS: &[template::Filter] = &[
    template::Filter::Default,
    template::Filter::Join,
    template::Filter::Split,
    template::Filter::First,
    template::Filter::Last,
    template::Filter::Length,
    template::Filter::Replace,
    template::Filter::Items,
    template::Filter::Keys,
    template::Filter::Values,
    template::Filter::Sort,
    template::Filter::Tojson,
    template::Filter::ShellQuote,
    template::Filter::SystemdEscape,
    template::Filter::SystemdEscapePath,
    template::Filter::Basename,
    template::Filter::Dirname,
];
#[test]
fn each_filter_translates_to_render_and_back() {
    for &filter in FILTERS {
        assert_eq!(back::filter(operations::filter(filter)), filter, "{filter:?}");
    }
}
#[test]
fn an_expressions_offsets_and_values_cross_unchanged() {
    let expression = template::Expression {
        values: 2,
        operations: Box::new([
            template::Operation { at: 7, op: template::Op::String(skein_lib::bytes::copy_of(b"text")) },
            template::Operation { at: 19, op: template::Op::Integer(-17) },
        ]),
    };
    assert_eq!(
        back::expression(operations::expression(expression)),
        template::Expression {
            values: 2,
            operations: Box::new([
                template::Operation { at: 7, op: template::Op::String(skein_lib::bytes::copy_of(b"text")) },
                template::Operation { at: 19, op: template::Op::Integer(-17) },
            ]),
        }
    );
}

fn git_kind_back(kind: diff::LineKind) -> crate::git_machine::hunks::Kind {
    use crate::git_machine::hunks::Kind;
    match kind {
        diff::LineKind::Hunk => Kind::Hunk,
        diff::LineKind::Context => Kind::Context,
        diff::LineKind::Removed => Kind::Removed,
        diff::LineKind::Added => Kind::Added,
        diff::LineKind::NoNewline => Kind::NoNewline,
    }
}

const HUNK_KINDS: &[(crate::git_machine::hunks::Kind, diff::LineKind)] = &[
    (crate::git_machine::hunks::Kind::Hunk, diff::LineKind::Hunk),
    (crate::git_machine::hunks::Kind::Context, diff::LineKind::Context),
    (crate::git_machine::hunks::Kind::Removed, diff::LineKind::Removed),
    (crate::git_machine::hunks::Kind::Added, diff::LineKind::Added),
    (crate::git_machine::hunks::Kind::NoNewline, diff::LineKind::NoNewline),
];
#[test]
fn each_hunk_line_kind_translates_and_back() {
    for (row, (git, render)) in HUNK_KINDS.iter().enumerate() {
        let translated = crate::translate::hunks::kind(*git);
        assert_eq!(translated, *render, "row {row}: git kind");
        assert_eq!(git_kind_back(translated), *git, "row {row}: git kind and back");
    }
}
#[test]
fn retained_hunk_text_and_counts_translate_from_git() {
    let hunks = crate::git_machine::hunks::Hunks {
        lines: Box::new([crate::git_machine::hunks::Line {
            kind: crate::git_machine::hunks::Kind::NoNewline,
            text: skein_lib::bytes::copy_of(b" No newline at end of file"),
        }]),
        left_out: 401,
        binary: false,
    };
    assert_eq!(
        crate::translate::hunks::hunks(hunks),
        diff::Hunks {
            lines: Box::new([diff::Line {
                kind: diff::LineKind::NoNewline,
                text: skein_lib::bytes::copy_of(b" No newline at end of file")
            }]),
            left_out: 401,
            binary: false
        }
    );
}

mod composition;
mod declarations;
mod repository;
mod store_arrivals;

mod evaluation;
mod instructions;

mod forms;

mod guards;
mod meta;

mod expansion;

mod debian;
mod fields;

mod additions;

mod exports;
mod files;

mod resolution;

mod exported;

mod resolve;

mod record_store;

mod ids;

mod reserved;
