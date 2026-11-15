//! Errors in the answer (hearth/docs/design/domain/render/errors.md, 1 to 3).
mod ids;
mod validation;
use crate::form::{json::field, text::Lines};
use alloc::boxed::Box;
use collisions::collision_json;
use collisions::collision_text;
use composition::{composition_json, composition_text};
use hearth_domain_render_vocabulary::{
    errors::{
        Cause, Class, Composition, Declared, Decode, Error, Expected, ExportFault, Exported, Field, Kind, Limit,
        Object, Place, Schema, Takes,
    },
    revision::declaration::Map,
};
use ids::{ids_json, ids_text};
use repository::{repository_json, repository_text, tool_json, tool_text};
use reserved::{reserved_json, reserved_text};
use resolution::{resolution_json, resolution_text};
use schema::{schema_json, schema_text};
use syntax::{syntax_json, syntax_text};
use validation::{fault_kind_message, fault_kind_name, required_name};
mod store;
use skein_json::writer::Encoder;
use store::{store_error_json, store_error_text};
use validation::{json_names, names, signed};

pub(crate) fn text(item: &Error, lines: &mut Lines) {
    if let Some(side) = item.side {
        lines.put(crate::diff::side(side));
        lines.put(b": ");
    }
    if let Some(at) = &item.at {
        place_text(at, lines);
        lines.put(b": ");
    }
    if let Some(target) = &item.target {
        lines.put(target);
        lines.put(b": ");
    }
    if let Some(task) = &item.task
        && item.kind != Kind::Composition(Composition::Task)
    {
        lines.put(task);
        lines.put(b": ");
    }
    match &item.kind {
        Kind::Exported(exported) => exported_text(exported, lines),
        Kind::Ids(ids) => ids_text(ids, lines),
        Kind::Reserved(reserved) => reserved_text(reserved, lines),
        Kind::Collision(collision) => collision_text(collision, lines),
        Kind::Resolution(error) => resolution_text(error, lines),
        Kind::Store(error) => store_error_text(error, lines),
        Kind::Decode(decode) => decode_text(decode, lines),
        Kind::Repository(why) => repository_text(why, lines),
        Kind::Tool(tool) => tool_text(tool, lines),
        Kind::Composition(why) => composition_text(why, lines, item.target.as_deref(), item.task.as_deref()),
        Kind::Schema(schema) => schema_text(schema, lines, item.task.as_deref()),
        Kind::Evaluation(kind) => lines.put(fault_kind_message(*kind)),
        Kind::Guard(guard) => {
            lines.put(b"requires ");
            lines.put(&guard.name);
            lines.put(b": ");
            lines.put(&guard.message);
        }
    }
    if let Some(also) = &item.also {
        lines.put(b" (");
        place_text(also, lines);
        lines.put(b")");
    }
    lines.put(b"\n");
    if let Some(tool) = item.kind.tool() {
        for line in &tool.tail {
            lines.put(b"  ");
            lines.put(line);
            lines.put(b"\n");
        }
    }
}
pub(crate) fn json(item: &Error, json: &mut Encoder) {
    field(json, b"record", b"error");
    if let Some(side) = item.side {
        field(json, b"side", crate::diff::side(side));
    }
    match &item.kind {
        Kind::Reserved(reserved) => {
            field(json, b"kind", b"reserved");
            json.key(b"reserved");
            reserved_json(reserved, json);
        }
        Kind::Ids(ids) => {
            field(json, b"kind", b"ids");
            json.key(b"ids");
            ids_json(ids, json);
        }
        Kind::Exported(exported) => {
            field(json, b"kind", b"exported");
            json.key(b"exported");
            exported_json(exported, json);
        }
        Kind::Collision(collision) => {
            field(json, b"kind", b"collision");
            json.key(b"collision");
            collision_json(collision, json);
        }
        Kind::Resolution(error) => {
            field(json, b"kind", b"resolution");
            json.key(b"resolution");
            resolution_json(error, json);
        }
        Kind::Store(error) => {
            field(json, b"kind", b"store");
            store_error_json(error, json);
        }
        Kind::Evaluation(kind) => {
            field(json, b"kind", b"evaluation");
            field(json, b"evaluation", fault_kind_name(*kind));
        }
        Kind::Schema(schema) => {
            field(json, b"kind", b"schema");
            json.key(b"schema");
            schema_json(schema, json);
        }
        Kind::Repository(why) => {
            field(json, b"kind", b"repository");
            json.key(b"repository");
            repository_json(why, json);
        }
        Kind::Composition(why) => {
            field(json, b"kind", b"composition");
            json.key(b"composition");
            composition_json(why, json);
        }
        Kind::Tool(tool) => {
            field(json, b"kind", b"tool");
            json.key(b"tool");
            tool_json(tool, json);
        }
        Kind::Decode(decode) => {
            field(json, b"kind", b"decode");
            json.key(b"decode");
            decode_json(decode, json);
        }
        Kind::Guard(guard) => {
            field(json, b"kind", b"guard");
            json.key(b"guard");
            json.object_start();
            field(json, b"message", &guard.message);
            field(json, b"name", &guard.name);
            json.object_end();
        }
    }
    field(
        json,
        b"class",
        match item.class {
            Class::Configuration => b"configuration",
            Class::Operational => b"operational",
        },
    );
    if let Some(at) = &item.at {
        json.key(b"at");
        place_json(at, json);
    }
    if let Some(also) = &item.also {
        json.key(b"also");
        place_json(also, json);
    }
    if let Some(target) = &item.target {
        field(json, b"target", target);
    }
    if let Some(task) = &item.task {
        field(json, b"task", task);
    }
}

pub(super) fn place_text(item: &Place, lines: &mut Lines) {
    lines.put(&item.path);
    if let Some(position) = item.position {
        lines.put(b":");
        lines.decimal(u64::from(position.line));
        lines.put(b":");
        lines.decimal(u64::from(position.column));
    }
}
pub(super) fn place_json(item: &Place, json: &mut Encoder) {
    json.object_start();
    if let Some(position) = item.position {
        json.key(b"column");
        json.unsigned(u64::from(position.column));
        json.key(b"line");
        json.unsigned(u64::from(position.line));
    }
    field(json, b"path", &item.path);
    json.object_end();
}

pub(super) fn decode_text(item: &Decode, lines: &mut Lines) {
    match item {
        Decode::Syntax(syntax) => syntax_text(syntax, lines),
        Decode::Declaration(declared) => declared_text(*declared, lines),
        Decode::Unexpected => lines.put(b"not JSON: unexpected byte"),
        Decode::Trailing => lines.put(b"not JSON: trailing bytes"),
        Decode::TooDeep => lines.put(b"JSON nested too deeply"),
        Decode::StringTooLong => lines.put(b"JSON string too long"),
        Decode::NumberTooLong => lines.put(b"JSON number too long"),
        Decode::Number => lines.put(b"not JSON: invalid number"),
        Decode::Escape => lines.put(b"not JSON: invalid escape"),
        Decode::Surrogate => lines.put(b"not JSON: invalid surrogate"),
        Decode::Utf8 => lines.put(b"not JSON: invalid UTF-8"),
        Decode::Control => lines.put(b"not JSON: control byte"),
        Decode::Truncated => lines.put(b"not JSON: truncated document"),
        Decode::NotObject => lines.put(b"a document must be an object"),
        Decode::NotInteger => lines.put(b"a number must be an integer"),
        Decode::Range => lines.put(b"an integer outside i64"),
        Decode::Nul => lines.put(b"U+0000 is not allowed"),
        Decode::Keys => lines.put(b"too many object keys"),
        Decode::Note => lines.put(b"_note must be a string"),
        Decode::ListNull => lines.put(b"null inside a list"),
        Decode::Name => lines.put(b"invalid task name"),
        Decode::Placement => lines.put(b"placement must be an object"),
        Decode::Members => lines.put(b"members must be an object or null"),
        Decode::Member => lines.put(b"invalid member"),
        Decode::Duplicate { key } => {
            lines.put(b"duplicate key ");
            lines.quote(key);
        }
        Decode::Settings { task } => {
            lines.put(b"settings for ");
            lines.quote(task);
            lines.put(b" must be an object or null");
        }
        Decode::Unknown { field } => {
            lines.put(b"unknown field ");
            lines.quote(field);
        }
        Decode::Wrong { field } => {
            lines.put(b"wrong value for ");
            lines.put(field_name(*field));
        }
        Decode::Missing { field } => {
            lines.put(b"missing ");
            lines.put(field_name(*field));
        }
        Decode::Misplaced { field } => {
            lines.put(b"misplaced ");
            lines.put(field_name(*field));
        }
        Decode::TooLarge { limit, max } => {
            lines.put(match limit {
                Limit::Record => b"a record past ",
                Limit::Closure => b"a closure past ",
                Limit::Template => b"a template past ",
                Limit::Document => b"a document past ",
                Limit::Decoded => b"decoded documents past ",
                Limit::Static => b"a static file past ",
                Limit::Entries => b"listing entries past ",
                Limit::Path => b"a path past ",
                Limit::Paths => b"listing paths past ",
                Limit::Tasks => b"tasks past ",
                Limit::Targets => b"targets past ",
            });
            lines.decimal(*max);
            lines.put(b" bytes");
        }
    }
}
pub(super) fn decode_json(item: &Decode, json: &mut Encoder) {
    match item {
        Decode::Syntax(syntax) => syntax_json(syntax, json),
        Decode::Declaration(declared) => {
            json.object_start();
            json.key(b"declared");
            declared_json(*declared, json);
            field(json, b"kind", b"declaration");
            json.object_end();
        }
        Decode::Unexpected => json.string(b"unexpected"),
        Decode::Trailing => json.string(b"trailing"),
        Decode::TooDeep => json.string(b"too_deep"),
        Decode::StringTooLong => json.string(b"string_too_long"),
        Decode::NumberTooLong => json.string(b"number_too_long"),
        Decode::Number => json.string(b"number"),
        Decode::Escape => json.string(b"escape"),
        Decode::Surrogate => json.string(b"surrogate"),
        Decode::Utf8 => json.string(b"utf8"),
        Decode::Control => json.string(b"control"),
        Decode::Truncated => json.string(b"truncated"),
        Decode::NotObject => json.string(b"not_object"),
        Decode::NotInteger => json.string(b"not_integer"),
        Decode::Range => json.string(b"range"),
        Decode::Nul => json.string(b"nul"),
        Decode::Keys => json.string(b"keys"),
        Decode::Note => json.string(b"note"),
        Decode::ListNull => json.string(b"list_null"),
        Decode::Name => json.string(b"name"),
        Decode::Placement => json.string(b"placement"),
        Decode::Members => json.string(b"members"),
        Decode::Member => json.string(b"member"),
        Decode::Duplicate { key } => {
            json.object_start();
            field(json, b"key", key);
            field(json, b"kind", b"duplicate");
            json.object_end();
        }
        Decode::Settings { task } => {
            json.object_start();
            field(json, b"task", task);
            field(json, b"kind", b"settings");
            json.object_end();
        }
        Decode::Unknown { field: name } => {
            json.object_start();
            field(json, b"field", name);
            field(json, b"kind", b"unknown");
            json.object_end();
        }
        Decode::Wrong { field: name } => {
            json.object_start();
            field(json, b"field", field_name(*name));
            field(json, b"kind", b"wrong");
            json.object_end();
        }
        Decode::Missing { field: name } => {
            json.object_start();
            field(json, b"field", field_name(*name));
            field(json, b"kind", b"missing");
            json.object_end();
        }
        Decode::Misplaced { field: name } => {
            json.object_start();
            field(json, b"field", field_name(*name));
            field(json, b"kind", b"misplaced");
            json.object_end();
        }
        Decode::TooLarge { limit, max } => {
            json.object_start();
            field(json, b"kind", b"too_large");
            field(
                json,
                b"limit",
                match limit {
                    Limit::Record => b"record",
                    Limit::Closure => b"closure",
                    Limit::Template => b"template",
                    Limit::Document => b"document",
                    Limit::Decoded => b"decoded",
                    Limit::Static => b"static",
                    Limit::Entries => b"entries",
                    Limit::Path => b"path",
                    Limit::Paths => b"paths",
                    Limit::Tasks => b"tasks",
                    Limit::Targets => b"targets",
                },
            );
            json.key(b"max");
            json.unsigned(*max);
            json.object_end();
        }
    }
}

pub(super) fn field_name(item: Field) -> &'static [u8] {
    match item {
        Field::Kind => b"kind",
        Field::Firmware => b"firmware",
        Field::Hardware => b"hardware",
        Field::On => b"on",
        Field::Mutable => b"mutable",
    }
}

pub(super) fn declared_text(item: Declared, lines: &mut Lines) {
    match item {
        Declared::Unknown { of } => {
            lines.put(b"no such key in a ");
            lines.put(object_name(of));
        }
        Declared::Wrong { expected } => {
            lines.put(b"not ");
            lines.put(expected_name(expected));
        }
        Declared::Missing { key } => {
            lines.put(b"missing ");
            lines.put(required_name(key));
        }
        Declared::Later => lines.put(b"a key for a later milestone"),
        Declared::Name => lines.put(b"invalid declaration name"),
        Declared::Doc => lines.put(b"doc must be one line within its length limit"),
        Declared::Order => lines.put(b"order must be from 0 to 999"),
        Declared::Optional => lines.put(b"optional beside a default"),
        Declared::Bound => lines.put(b"a bound on a type it does not bound"),
        Declared::Above => lines.put(b"min above max"),
        Declared::Enum => lines.put(b"an enum must list distinct values"),
        Declared::Settings => lines.put(b"too many settings"),
        Declared::Secret => lines.put(b"an export type holds secret"),
    }
}
pub(super) fn declared_json(item: Declared, json: &mut Encoder) {
    json.object_start();
    match item {
        Declared::Unknown { of } => {
            field(json, b"kind", b"unknown");
            field(json, b"of", object_name(of));
        }
        Declared::Wrong { expected } => {
            field(json, b"kind", b"wrong");
            field(json, b"expected", expected_name(expected));
        }
        Declared::Missing { key } => {
            field(json, b"kind", b"missing");
            field(json, b"key", required_name(key));
        }
        Declared::Later => field(json, b"kind", b"later"),
        Declared::Name => field(json, b"kind", b"name"),
        Declared::Doc => field(json, b"kind", b"doc"),
        Declared::Order => field(json, b"kind", b"order"),
        Declared::Optional => field(json, b"kind", b"optional"),
        Declared::Bound => field(json, b"kind", b"bound"),
        Declared::Above => field(json, b"kind", b"above"),
        Declared::Enum => field(json, b"kind", b"enum"),
        Declared::Settings => field(json, b"kind", b"settings"),
        Declared::Secret => field(json, b"kind", b"secret"),
    }
    json.object_end();
}

pub(super) fn object_name(item: Object) -> &'static [u8] {
    match item {
        Object::Declaration => b"declaration",
        Object::Read => b"read",
        Object::Guard => b"guard",
        Object::Script => b"script",
        Object::Bless => b"bless",
        Object::Range => b"range",
        Object::Spec => b"spec",
        Object::Export => b"export",
        Object::Type => b"type",
        Object::Entry(map) => map_name(map),
    }
}

pub(super) fn map_name(item: Map) -> &'static [u8] {
    match item {
        Map::Packages => b"packages entry",
        Map::Preseeds => b"preseeds entry",
        Map::Initramfs => b"initramfs entry",
        Map::Software => b"software entry",
        Map::Files => b"files entry",
        Map::Users => b"users entry",
        Map::Groups => b"groups entry",
        Map::Presets => b"presets entry",
        Map::Tmpfiles => b"tmpfiles entry",
        Map::Data => b"data entry",
        Map::Secrets => b"secrets entry",
        Map::Identity => b"identity entry",
        Map::Cmdline => b"cmdline entry",
        Map::Live => b"live entry",
    }
}

pub(super) fn expected_name(item: Expected) -> &'static [u8] {
    match item {
        Expected::Object => b"an object",
        Expected::String => b"a string",
        Expected::Strings => b"a list of strings",
        Expected::Bool => b"a bool",
        Expected::Int => b"an int",
        Expected::True => b"true",
        Expected::Type => b"a type",
        Expected::Field(takes) => takes_name(takes),
    }
}

pub(super) fn takes_name(item: Takes) -> &'static [u8] {
    match item {
        Takes::Text => b"a string",
        Takes::Int => b"an int",
        Takes::Bool => b"a bool",
        Takes::Strings => b"a list of strings",
        Takes::Range => b"a range",
    }
}

mod schema;

mod type_name;

mod repository;

mod composition;
mod syntax;

mod collisions;

pub(super) fn exported_text(item: &Exported, lines: &mut Lines) {
    lines.put(b"exports.");
    lines.put(&item.task);
    lines.put(b" of ");
    lines.put(&item.exporter);
    lines.put(b": ");
    match &item.cause {
        Cause::Decode(value) => decode_text(value, lines),
        Cause::Repository(value) => repository_text(value, lines),
        Cause::Composition(value) => composition_text(value, lines, Some(&item.exporter), Some(&item.task)),
        Cause::Schema(value) => schema_text(value, lines, Some(&item.task)),
        Cause::Evaluation(value) => lines.put(fault_kind_message(*value)),
    }
}
pub(super) fn exported_json(item: &Exported, json: &mut Encoder) {
    json.object_start();
    json.key(b"cause");
    json.object_start();
    match &item.cause {
        Cause::Decode(value) => {
            json.key(b"decode");
            decode_json(value, json);
        }
        Cause::Repository(value) => {
            json.key(b"repository");
            repository_json(value, json);
        }
        Cause::Composition(value) => {
            json.key(b"composition");
            composition_json(value, json);
        }
        Cause::Schema(value) => {
            json.key(b"schema");
            schema_json(value, json);
        }
        Cause::Evaluation(value) => field(json, b"evaluation", fault_kind_name(*value)),
    }
    json.object_end();
    field(json, b"exporter", &item.exporter);
    field(json, b"task", &item.task);
    json.object_end();
}

mod resolution;

mod reserved;
