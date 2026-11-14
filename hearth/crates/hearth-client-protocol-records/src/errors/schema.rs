use super::type_name::{type_name_article, type_name_name};
use hearth_domain_render_vocabulary::errors::{Format, TypeName};
use super::{Box, Encoder, Lines, Schema, field, json_names, names, signed, validation};


pub(super) fn format_name(item: Format) -> &'static [u8] {
    match item {
        Format::DataName => b"data name",
        Format::Link => b"link",
        Format::Path => b"path",
        Format::Mode => b"mode",
        Format::Duration => b"duration",
        Format::Secret => b"secret",
        Format::File => b"file",
        Format::Key => b"key",
        Format::Timestamp => b"timestamp",
        Format::Debian => b"debian",
        Format::Preseed => b"preseed",
        Format::Module => b"module",
        Format::Param => b"param",
        Format::Sha256 => b"sha256",
    }
}


pub(super) fn schema_text(item: &Schema, lines: &mut Lines, task: Option<&[u8]>) {
    match item {
        Schema::Export { fault } => export_text(lines, *fault),
        Schema::Unlisted { reader, root, task, setting } => {
            unlisted_text(lines, reader, *root, task, setting.as_deref());
        }
        Schema::Source { first, second } => {
            lines.put(b"two sources ");
            lines.put(first.key());
            lines.put(b" and ");
            lines.put(second.key());
        }
        Schema::Dynamic { root } => dynamic_text(lines, *root),
        Schema::Malformed { format } => {
            lines.put(b"malformed ");
            lines.put(format_name(*format));
        }
        Schema::Target { name, environment } => {
            lines.put(b"no target ");
            lines.put(name);
            lines.put(b" in ");
            lines.put(environment);
        }
        Schema::File { path } => {
            lines.put(b"no file ");
            lines.put(path);
            lines.put(b" at this revision");
        }
        Schema::Large { path, max } => large_text(lines, path, *max),
        Schema::Component { package, component } => component_text(lines, package, component),
        Schema::Beneath { allowed } => {
            lines.put(b"path must be beneath ");
            names(lines, allowed);
        }
        Schema::Main => lines.put(b"debian.components must keep main on"),
        Schema::Security => lines.put(b"debian.snapshot.security is required with its pocket"),

        Schema::Undeclared { setting, declared } => {
            validation::undeclared(lines, task.unwrap_or(b"task"), setting, declared);
        }
        Schema::Missing { setting } => {
            lines.put(b"missing required setting ");
            lines.put(setting);
        }
        Schema::Absent { task } => {
            lines.put(b"reads ");
            lines.put(task);
            lines.put(b", which this target lacks");
        }
        Schema::Wrong { expected } => {
            lines.put(b"not ");
            lines.put(type_name_article(expected));
            lines.put(type_name_name(expected));
        }
        Schema::Enum { value, listed } => {
            lines.quote(value);
            lines.put(b" is not one of ");
            names(lines, listed);
        }
        Schema::Below { min } => {
            lines.put(b"below its min ");
            signed(lines, *min);
        }
        Schema::Above { max } => {
            lines.put(b"past its max ");
            signed(lines, *max);
        }
        Schema::Length { max_length } => {
            lines.put(b"past its max_length ");
            signed(lines, *max_length);
        }
        Schema::Null => lines.put(b"null in a default"),
        Schema::UndeclaredField { field, declared } => {
            lines.put(b"undeclared field ");
            lines.quote(field);
            lines.put(b"; the object declares ");
            names(lines, declared);
        }
        Schema::MissingField { field } => {
            lines.put(b"missing required field ");
            lines.put(field);
        }

        Schema::Task { task } => {
            lines.put(b"reads ");
            lines.put(task);
            lines.put(b", which tasks/ lacks");
        }
        Schema::Unread { task, setting, declared } => unread_text(lines, task, setting, declared),
    }
}
pub(super) fn schema_json(item: &Schema, json: &mut Encoder) {
    json.object_start();
    match item {
        Schema::Export { fault } => export_json(json, *fault),
        Schema::Unlisted { reader, root, task, setting } => {
            unlisted_json(json, reader, *root, task, setting.as_deref());
        }
        Schema::Source { first, second } => {
            field(json, b"first", first.key());
            field(json, b"kind", b"source");
            field(json, b"second", second.key());
        }
        Schema::Dynamic { root } => dynamic_json(json, *root),
        Schema::Malformed { format } => {
            field(json, b"format", format_name(*format));
            field(json, b"kind", b"malformed");
        }
        Schema::Target { name, environment } => {
            field(json, b"environment", environment);
            field(json, b"kind", b"target");
            field(json, b"name", name);
        }
        Schema::File { path } => {
            field(json, b"kind", b"file");
            field(json, b"path", path);
        }
        Schema::Large { path, max } => large_json(json, path, *max),
        Schema::Beneath { allowed } => {
            json.key(b"allowed");
            json_names(json, allowed);
            field(json, b"kind", b"beneath");
        }
        Schema::Component { package, component } => component_json(json, package, component),
        Schema::Main => field(json, b"kind", b"main"),
        Schema::Security => field(json, b"kind", b"security"),

        Schema::Undeclared { setting, declared } => {
            json.key(b"declared");
            json_names(json, declared);
            field(json, b"kind", b"undeclared");
            field(json, b"setting", setting);
        }
        Schema::Missing { setting } => {
            field(json, b"kind", b"missing");
            field(json, b"setting", setting);
        }
        Schema::Absent { task } => {
            field(json, b"kind", b"absent");
            field(json, b"task", task);
        }
        Schema::Wrong { expected } => {
            field(json, b"expected", type_name_name(expected));
            field(json, b"kind", b"wrong");
        }
        Schema::Enum { value, listed } => {
            field(json, b"kind", b"enum");
            json.key(b"listed");
            json_names(json, listed);
            field(json, b"value", value);
        }
        Schema::Below { min } => {
            field(json, b"kind", b"below");
            json.key(b"min");
            json.signed(*min);
        }
        Schema::Above { max } => {
            field(json, b"kind", b"above");
            json.key(b"max");
            json.signed(*max);
        }
        Schema::Length { max_length } => {
            field(json, b"kind", b"length");
            json.key(b"max_length");
            json.signed(*max_length);
        }
        Schema::Null => field(json, b"kind", b"null"),
        Schema::UndeclaredField { field: name, declared } => {
            json.key(b"declared");
            json_names(json, declared);
            field(json, b"field", name);
            field(json, b"kind", b"undeclared_field");
        }
        Schema::MissingField { field: name } => {
            field(json, b"field", name);
            field(json, b"kind", b"missing_field");
        }

        Schema::Task { task } => {
            field(json, b"kind", b"task");
            field(json, b"task", task);
        }
        Schema::Unread { task, setting, declared } => {
            json.key(b"declared");
            json_names(json, declared);
            field(json, b"kind", b"unread");
            field(json, b"setting", setting);
            field(json, b"task", task);
        }
    }
    json.object_end();
}



pub(super) fn root_name(item: Root) -> &'static [u8] {
    match item {
        Root::Tasks => b"tasks",
        Root::Debian => b"debian",
        Root::Exports => b"exports",
    }
}

fn unlisted_text(lines: &mut Lines, reader: &[u8], root: super::Root, task: &[u8], setting: Option<&[u8]>) {
    lines.put(root_name(root));
    lines.put(b".");
    lines.put(task);
    if let Some(setting) = setting {
        lines.put(b".");
        lines.put(setting);
    }
    if root == super::Root::Tasks && setting.is_none() {
        lines.put(b" is not an optional read of ");
        lines.put(reader);
    } else {
        lines.put(b" is not in ");
        lines.put(reader);
        lines.put(b"'s reads");
    }
}
fn unlisted_json(json: &mut Encoder, reader: &[u8], root: super::Root, task: &[u8], setting: Option<&[u8]>) {
    field(json, b"kind", b"unlisted");
    field(json, b"reader", reader);
    field(json, b"root", root_name(root));
    json.key(b"setting");
    if let Some(setting) = setting {
        json.string(setting);
    } else {
        json.null();
    }
    field(json, b"task", task);
}

fn dynamic_text(lines: &mut Lines, root: super::Root) {
    lines.put(root_name(root));
    lines.put(b" takes only literal names");
}

fn dynamic_json(json: &mut Encoder, root: super::Root) {
    field(json, b"kind", b"dynamic");
    field(json, b"root", root_name(root));
}

fn export_text(lines: &mut Lines, fault: super::ExportFault) {
    match fault {
        super::ExportFault::Form => lines.put(b"an export's value must be one {{ }} tag"),
        super::ExportFault::Root { root } => {
            lines.put(b"an export reads settings and target alone, not ");
            lines.put(root_name(root));
        }
    }
}
fn export_json(json: &mut Encoder, fault: super::ExportFault) {
    field(json, b"kind", b"export");
    match fault {
        super::ExportFault::Form => field(json, b"fault", b"form"),
        super::ExportFault::Root { root } => {
            field(json, b"fault", b"root");
            field(json, b"root", root_name(root));
        }
    }
}

fn unread_text(lines: &mut Lines, task: &[u8], setting: &[u8], declared: &[Box<[u8]>]) {
    lines.put(b"reads ");
    lines.put(task);
    lines.put(b".");
    lines.put(setting);
    lines.put(b", which ");
    lines.put(task);
    lines.put(b" does not declare; ");
    lines.put(task);
    if declared.is_empty() {
        lines.put(b" declares no settings");
    } else {
        lines.put(b" declares ");
        names(lines, declared);
    }
}
fn component_text(lines: &mut Lines, package: &[u8], component: &[u8]) {
    lines.put(package);
    lines.put(b" needs component ");
    lines.put(component);
    lines.put(b", which debian.components does not enable");
}
fn large_text(lines: &mut Lines, path: &[u8], max: u64) {
    lines.put(path);
    lines.put(b" past ");
    lines.decimal(max);
    lines.put(b" bytes");
}

fn component_json(json: &mut Encoder, package: &[u8], component: &[u8]) {
    field(json, b"component", component);
    field(json, b"kind", b"component");
    field(json, b"package", package);
}
fn large_json(json: &mut Encoder, path: &[u8], max: u64) {
    field(json, b"kind", b"large");
    json.key(b"max");
    json.unsigned(max);
    field(json, b"path", path);
}
