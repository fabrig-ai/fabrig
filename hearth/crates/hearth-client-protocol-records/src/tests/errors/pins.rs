//! Literal printed-form pins (hearth/docs/design/domain/render/contracts.md, 4).
use crate::{
    errors::{Error, Kind},
    form::Form,
    tests::fixtures,
};

fn plain(kind: Kind) -> Error {
    let mut error = fixtures::duplicate();
    error.kind = kind;
    error.at = None;
    error.also = None;
    error
}

fn assert_prints(error: &Error, text: &str, json: &str) {
    for (form, expected) in [(Form::Text, text), (Form::JsonLines, json)] {
        assert_eq!(
            fixtures::print_error(form, error.clone()).as_deref(),
            Ok(expected.as_bytes()),
            "{form:?}: {error:?}"
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
