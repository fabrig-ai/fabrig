//! Literal printed-form pins (hearth/docs/design/client/README.md, 3.2).
use crate::{form::Form, tests::fixtures};
use hearth_domain_render_vocabulary as render;
use render::errors::{Error, Kind};

fn plain(kind: Kind) -> Error {
    let mut error = fixtures::duplicate();
    error.kind = kind;
    error.at = None;
    error.also = None;
    error
}

fn assert_prints(error: &Error, text: &str, json: &str) {
    assert_record_prints(&crate::records::Record::Error(error.clone()), text, json);
}
fn assert_record_prints(record: &crate::records::Record, text: &str, json: &str) {
    for (form, expected) in [(Form::Text, text), (Form::JsonLines, json)] {
        assert_eq!(
            crate::form::print(form, record, Some(crate::records::Type::Error), &crate::limits::SHIPPED).as_deref(),
            Ok(expected.as_bytes()),
            "{form:?}: {record:?}"
        );
    }
}

mod collision;
mod declared;
mod decode;
mod envelope;
mod exported;
mod ids;
mod repository;
mod reserved;
mod resolution;
mod schema;
mod store;
mod syntax;
mod tool;
