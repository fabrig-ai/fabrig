use super::assert_prints;
use crate::{
    errors::{
        Cause, Composition, Decode, Error, Evaluation, ExportFault, Format, Kind, Repository, Root, Schema, TypeName,
    },
    tests::fixtures,
};
use skein_lib::bytes::copy_of;
fn exported(cause: Cause) -> Error {
    let mut error = fixtures::exported_error();
    let Kind::Exported(exported) = &mut error.kind else { panic!("exported fixture") };
    exported.cause = cause;
    error
}

#[expect(clippy::too_many_lines, reason = "literal text and JSON pins keep each error beside its bytes")]
#[test]
fn exported_bytes() {
    let cases = [
        (
            exported(Cause::Decode(Decode::Trailing)),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not JSON: trailing bytes\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"decode":"trailing"},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Repository(Repository::NoTree)),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: no tree entry at this path\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"repository":"no_tree"},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Composition(Composition::Duplicate)),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: two targets are named west-prod\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"composition":"duplicate"},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Evaluation(Evaluation::NoKey)),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: a missing key\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"evaluation":"no_key"},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Source { first: copy_of(b"one"), second: copy_of(b"two") })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: two sources one and two\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"first":"one","kind":"source","second":"two"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::DataName })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed data name\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"data name","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Sha256 })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed sha256\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"sha256","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Link })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed link\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"link","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Path })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed path\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"path","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Mode })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed mode\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"mode","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Duration })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed duration\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"duration","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Secret })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed secret\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"secret","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::File })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed file\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"file","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Key })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed key\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"key","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Timestamp })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed timestamp\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"timestamp","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Debian })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed debian\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"debian","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Preseed })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed preseed\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"preseed","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Module })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed module\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"module","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Malformed { format: Format::Param })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: malformed param\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"format":"param","kind":"malformed"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Export { fault: ExportFault::Form })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: an export's value must be one {{ }} tag\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"export","fault":"form"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Export { fault: ExportFault::Root { root: Root::Debian } })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: an export reads settings and target alone, not debian\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"export","fault":"root","root":"debian"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Dynamic { root: Root::Debian })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: debian takes only literal names\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"dynamic","root":"debian"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Debian,
                task: copy_of(b"core"),
                setting: None,
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: debian.core is not in motd's reads\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"unlisted","reader":"motd","root":"debian","setting":null,"task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Debian,
                task: copy_of(b"core"),
                setting: Some(copy_of(b"domain")),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: debian.core.domain is not in motd's reads\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"unlisted","reader":"motd","root":"debian","setting":"domain","task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Export { fault: ExportFault::Root { root: Root::Tasks } })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: an export reads settings and target alone, not tasks\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"export","fault":"root","root":"tasks"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Dynamic { root: Root::Tasks })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: tasks takes only literal names\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"dynamic","root":"tasks"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Tasks,
                task: copy_of(b"core"),
                setting: None,
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: tasks.core is not an optional read of motd\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"unlisted","reader":"motd","root":"tasks","setting":null,"task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Tasks,
                task: copy_of(b"core"),
                setting: Some(copy_of(b"domain")),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: tasks.core.domain is not in motd's reads\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"unlisted","reader":"motd","root":"tasks","setting":"domain","task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Export { fault: ExportFault::Root { root: Root::Exports } })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: an export reads settings and target alone, not exports\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"export","fault":"root","root":"exports"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Dynamic { root: Root::Exports })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: exports takes only literal names\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"dynamic","root":"exports"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Exports,
                task: copy_of(b"core"),
                setting: None,
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: exports.core is not in motd's reads\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"unlisted","reader":"motd","root":"exports","setting":null,"task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Exports,
                task: copy_of(b"core"),
                setting: Some(copy_of(b"domain")),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: exports.core.domain is not in motd's reads\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"unlisted","reader":"motd","root":"exports","setting":"domain","task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Bool })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a bool\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"bool","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Int })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not an int\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"int","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Enum })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not an enum\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"enum","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::List })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a list\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"list","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Map })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a map\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"map","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Object })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not an object\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"object","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Path })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a path\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"path","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Mode })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a mode\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"mode","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Duration })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a duration\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"duration","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Secret })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a secret\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"secret","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::Target })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a target\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"target","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Wrong { expected: TypeName::File })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: not a file\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"expected":"file","kind":"wrong"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Beneath { allowed: Box::new([copy_of(b"/srv"), copy_of(b"/var")]) })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: path must be beneath /srv, /var\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"allowed":["/srv","/var"],"kind":"beneath"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Component {
                package: copy_of(b"firmware"),
                component: copy_of(b"non-free-firmware"),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: firmware needs component non-free-firmware, which debian.components does not enable\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"component":"non-free-firmware","kind":"component","package":"firmware"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Undeclared { setting: copy_of(b"adress"), declared: Box::new([]) })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: undeclared setting \"adress\"; vpn declares no settings\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"declared":[],"kind":"undeclared","setting":"adress"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Undeclared {
                setting: copy_of(b"adress"),
                declared: Box::new([copy_of(b"address")]),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: undeclared setting \"adress\"; vpn declares address\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"declared":["address"],"kind":"undeclared","setting":"adress"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unread {
                task: copy_of(b"core"),
                setting: copy_of(b"domian"),
                declared: Box::new([]),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: reads core.domian, which core does not declare; core declares no settings\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"declared":[],"kind":"unread","setting":"domian","task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Unread {
                task: copy_of(b"core"),
                setting: copy_of(b"domian"),
                declared: Box::new([copy_of(b"domain")]),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: reads core.domian, which core does not declare; core declares domain\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"declared":["domain"],"kind":"unread","setting":"domian","task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Task { task: copy_of(b"core") })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: reads core, which tasks/ lacks\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"task","task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Enum {
                value: copy_of(b"x"),
                listed: Box::new([copy_of(b"main"), copy_of(b"contrib")]),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: \"x\" is not one of main, contrib\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"enum","listed":["main","contrib"],"value":"x"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Below { min: -10 })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: below its min -10\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"below","min":-10}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Above { max: 10 })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: past its max 10\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"above","max":10}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Length { max_length: 8 })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: past its max_length 8\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"length","max_length":8}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Null)),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: null in a default\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"null"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::UndeclaredField {
                field: copy_of(b"shel"),
                declared: Box::new([copy_of(b"shell"), copy_of(b"uid")]),
            })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: undeclared field \"shel\"; the object declares shell, uid\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"declared":["shell","uid"],"field":"shel","kind":"undeclared_field"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::MissingField { field: copy_of(b"uid") })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: missing required field uid\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"field":"uid","kind":"missing_field"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Missing { setting: copy_of(b"address") })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: missing required setting address\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"missing","setting":"address"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Absent { task: copy_of(b"core") })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: reads core, which this target lacks\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"absent","task":"core"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Target { name: copy_of(b"x9-prod"), environment: copy_of(b"prod") })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: no target x9-prod in prod\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"environment":"prod","kind":"target","name":"x9-prod"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::File { path: copy_of(b"lab-ca/lab.crt") })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: no file lab-ca/lab.crt at this revision\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"file","path":"lab-ca/lab.crt"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Large { path: copy_of(b"lab-ca/big.crt"), max: 64 })),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: lab-ca/big.crt past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"large","max":64,"path":"lab-ca/big.crt"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Main)),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: debian.components must keep main on\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"main"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
        (
            exported(Cause::Schema(Schema::Security)),
            "tasks/boot/task.json:22:7: carbon-prod: vpn: exports.vpn of west-prod: debian.snapshot.security is required with its pocket\n",
            concat!(
                r#"{"record":"error","kind":"exported","exported":{"cause":{"schema":{"kind":"security"}},"exporter":"west-prod","task":"vpn"},"class":"configuration","at":{"column":7,"line":22,"path":"tasks/boot/task.json"},"target":"carbon-prod","task":"vpn"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}
