//! Whole-output pins (hearth/docs/design/domain/render/contracts.md, 4).
use super::fixtures;
use crate::{
    form::{self, Form},
    limits,
    records::Record,
};
use alloc::boxed::Box;
use hearth_domain_render_vocabulary as render;
use render::{
    diff::PartChange,
    parts::{
        Part, PartRecord,
        config::{Change as ConfigChange, Config},
    },
};

// Only literal fragments enter these macros: the expectation never uses the printer.
macro_rules! spec_text {
    ($($part:literal),* $(,)?) => { concat!(
        "target       x1-prod   base x1   environment prod\n",
        "placement    metal efi 7th-gen\n",
        "spec         9a0c5e1f2b3d\n", $($part),*
    ) };
}
macro_rules! spec_json {
    ($($part:literal),* $(,)?) => { concat!(
        r#"{"record":"version","client":1}"#, "\n",
        r#"{"record":"spec","target":"x1-prod","commit":"1a2b3c4d00000000000000000000000000000000","digest":"9a0c5e1f2b3d0000000000000000000000000000000000000000000000000000"}"#, "\n",
        r#"{"record":"part","target":"x1-prod","name":"meta","digest":"c4d3a0b1e2f50000000000000000000000000000000000000000000000000000","meta":{"base":"x1","environment":"prod","placement":{"firmware":"efi","hardware":"7th-gen","kind":"metal"},"target":"x1-prod"}}"#, "\n", $($part),*
    ) };
}
macro_rules! change_text {
    ($($part:literal),* $(,)?) => { concat!(
        "west-prod                                           nothing to build\n", $($part),*
    ) };
}
macro_rules! change_json {
    ($($part:literal),* $(,)?) => { concat!(
        r#"{"record":"change","target":"west-prod","kind":"changed","changed":{"parts":["#,
        $($part),*, r#"],"rebuild":{"kind":"nothing"}}}"#, "\n"
    ) };
}

fn check(rows: &[(Record, &str, &str)]) {
    for (row, (record, text, json)) in rows.iter().enumerate() {
        for (format, expected, after) in [(Form::Text, *text, None), (Form::JsonLines, *json, Some(record.kind()))] {
            // Specs include the version line; changes are subsequent stream records.
            let after = if let Record::Spec(_) = record { None } else { after };
            assert_eq!(
                form::print(format, record, after, &limits::SHIPPED).as_deref(),
                Ok(expected.as_bytes()),
                "row {row}, {format:?}"
            );
        }
    }
}
fn spec(part: PartRecord) -> Record {
    let mut spec = fixtures::x1_prod();
    spec.parts = Box::new([Part { digest: hearth_lib::encode::Digest([0; 32]), record: part }]);
    Record::Spec(spec)
}
fn config() -> Config {
    let mut spec = fixtures::x1_prod();
    let config = fixtures::config_mut(&mut spec);
    config.tree = Box::new([]);
    config.clone()
}
fn config_change() -> ConfigChange {
    ConfigChange {
        presets: Box::new([]),
        tmpfiles: Box::new([]),
        scripts: None,
        groups: Box::new([]),
        users: Box::new([]),
        members: Box::new([]),
        tree: Box::new([]),
    }
}
fn change(part: PartChange) -> Record {
    fixtures::change_record(part)
}

mod changes;
mod config;
mod packages;
mod scripts;
mod specs;
