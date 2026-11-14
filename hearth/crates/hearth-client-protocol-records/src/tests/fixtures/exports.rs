use super::{Part, PartRecord, Spec, copy_of, duplicate, x1_prod};
use hearth_domain_render_vocabulary as render;
use render::{
    evaluate::{Entry as Named, Value},
    exports::Exported as TaskExports,
    parts::exports::Exports,
};
use skein_lib::List;

pub const EXPORT_PRINT_DEPTH: u32 = 32;
pub const EXPORT_PRINT_BRACKETS: usize = 32;
pub fn with_exports(mut spec: Spec, task: &[u8], name: &[u8], value: Value) -> Spec {
    spec.parts = Box::new([Part {
        digest: hearth_lib::encode::Digest([0; 32]),
        record: PartRecord::Exports(Exports {
            tasks: Box::new([TaskExports {
                task: copy_of(task),
                values: Box::new([
                    Named { key: copy_of(b"an_absent_export_with_a_long_name"), value: None },
                    Named { key: copy_of(name), value: Some(value) },
                    Named { key: copy_of(b"last_absent"), value: None },
                ]),
            }]),
        }),
    }]);
    spec
}
pub fn export_spec(value: Value) -> Box<Spec> {
    Box::new(with_exports(*x1_prod(), b"vpn", b"nest", value))
}
pub fn lists_around(depth: u32, value: &[u8]) -> Value {
    let mut value = Value::String(copy_of(value));
    for _ in 0..depth {
        value = Value::List(Box::new([value]));
    }
    value
}
pub fn has_run(bytes: &[u8], parts: &[&[u8]]) -> bool {
    skein_lib::bytes::find(bytes, &joined(parts)).is_some()
}
pub fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("fixture length");
    }
    let mut expected = List::with_capacity(u32::try_from(length).expect("printed length"));
    for part in parts {
        for byte in *part {
            expected.push(*byte).expect("printed length");
        }
    }
    expected.into_boxed()
}
pub fn printed_exports(form: crate::form::Form) -> Box<[u8]> {
    crate::form::print(
        form,
        &crate::records::Record::Spec(Box::new(with_exports(*x1_prod(), b"vpn", b"port", Value::Int(51820)))),
        None,
        &crate::limits::SHIPPED,
    )
    .expect("exports fit")
}
pub fn export_error(fault: crate::errors::ExportFault) -> crate::errors::Error {
    let mut error = duplicate();
    error.at = None;
    error.also = None;
    error.kind = crate::errors::Kind::Schema(crate::errors::Schema::Export { fault });
    error
}
pub fn exported_error() -> crate::errors::Error {
    use crate::errors::{Cause, Exported, Kind, Schema, TypeName};
    let mut error = super::modules_guard();
    error.kind = Kind::Exported(Exported {
        exporter: copy_of(b"west-prod"),
        task: copy_of(b"vpn"),
        cause: Cause::Schema(Schema::Wrong { expected: TypeName::String }),
    });
    error.target = Some(copy_of(b"carbon-prod"));
    error.task = Some(copy_of(b"vpn"));
    error
}

pub fn printed_export_change(form: crate::form::Form, before: Option<&[u8]>, after: Option<&[u8]>) -> Box<[u8]> {
    let part = render::diff::PartChange::Exports(render::parts::exports::Change {
        entries: Box::new([render::parts::exports::ExportChange {
            task: copy_of(b"vpn"),
            name: copy_of(b"endpoint"),
            before: optional_string(before),
            after: optional_string(after),
        }]),
    });
    crate::form::print(form, &super::change_record(part), Some(crate::records::Type::Change), &crate::limits::SHIPPED)
        .expect("export change fits")
}

fn optional_string(value: Option<&[u8]>) -> Option<Value> {
    Some(Value::String(copy_of(value?)))
}

pub fn export_map(up: bool) -> Value {
    Value::Map(Box::new([
        Named { key: copy_of(b"first_absent"), value: None },
        Named { key: copy_of(b"up"), value: Some(Value::Bool(up)) },
        Named { key: copy_of(b"last_absent"), value: None },
    ]))
}
