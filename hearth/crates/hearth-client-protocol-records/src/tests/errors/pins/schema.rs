use super::assert_prints;
use super::plain;
use crate::errors::{ExportFault, Format, Kind, Root, Schema, TypeName};
use skein_lib::bytes::copy_of;

#[expect(clippy::too_many_lines, reason = "literal text and JSON pins keep each error beside its bytes")]
#[test]
fn schema_bytes() {
    let cases = [
        (
            plain(Kind::Schema(Schema::Source { first: copy_of(b"one"), second: copy_of(b"two") })),
            "two sources one and two\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"first":"one","kind":"source","second":"two"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::DataName })),
            "malformed data name\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"data name","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Sha256 })),
            "malformed sha256\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"sha256","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Link })),
            "malformed link\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"link","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Mode })),
            "malformed mode\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"mode","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Duration })),
            "malformed duration\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"duration","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Secret })),
            "malformed secret\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"secret","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::File })),
            "malformed file\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"file","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Key })),
            "malformed key\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"key","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Timestamp })),
            "malformed timestamp\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"timestamp","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Debian })),
            "malformed debian\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"debian","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Preseed })),
            "malformed preseed\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"preseed","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Module })),
            "malformed module\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"module","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Malformed { format: Format::Param })),
            "malformed param\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"format":"param","kind":"malformed"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Export { fault: ExportFault::Form })),
            "an export's value must be one {{ }} tag\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"export","fault":"form"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Export { fault: ExportFault::Root { root: Root::Debian } })),
            "an export reads settings and target alone, not debian\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"export","fault":"root","root":"debian"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Dynamic { root: Root::Debian })),
            "debian takes only literal names\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"dynamic","root":"debian"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Debian,
                task: copy_of(b"core"),
                setting: None,
            })),
            "debian.core is not in motd's reads\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"unlisted","reader":"motd","root":"debian","setting":null,"task":"core"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Debian,
                task: copy_of(b"core"),
                setting: Some(copy_of(b"domain")),
            })),
            "debian.core.domain is not in motd's reads\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"unlisted","reader":"motd","root":"debian","setting":"domain","task":"core"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Export { fault: ExportFault::Root { root: Root::Tasks } })),
            "an export reads settings and target alone, not tasks\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"export","fault":"root","root":"tasks"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Unlisted {
                reader: copy_of(b"motd"),
                root: Root::Tasks,
                task: copy_of(b"core"),
                setting: None,
            })),
            "tasks.core is not an optional read of motd\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"unlisted","reader":"motd","root":"tasks","setting":null,"task":"core"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Export { fault: ExportFault::Root { root: Root::Exports } })),
            "an export reads settings and target alone, not exports\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"kind":"export","fault":"root","root":"exports"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Bool })),
            "not a bool\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"bool","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::String })),
            "not a string\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"string","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Enum })),
            "not an enum\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"enum","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::List })),
            "not a list\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"list","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Map })),
            "not a map\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"map","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Object })),
            "not an object\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"object","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Path })),
            "not a path\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"path","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Mode })),
            "not a mode\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"mode","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Duration })),
            "not a duration\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"duration","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Secret })),
            "not a secret\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"secret","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::Target })),
            "not a target\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"target","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Wrong { expected: TypeName::File })),
            "not a file\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"expected":"file","kind":"wrong"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Beneath { allowed: Box::new([copy_of(b"/srv"), copy_of(b"/var")]) })),
            "path must be beneath /srv, /var\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"allowed":["/srv","/var"],"kind":"beneath"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Component {
                package: copy_of(b"firmware"),
                component: copy_of(b"non-free-firmware"),
            })),
            "firmware needs component non-free-firmware, which debian.components does not enable\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"component":"non-free-firmware","kind":"component","package":"firmware"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Undeclared { setting: copy_of(b"adress"), declared: Box::new([]) })),
            "undeclared setting \"adress\"; task declares no settings\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"declared":[],"kind":"undeclared","setting":"adress"},"class":"configuration"}"#,
                "\n"
            ),
        ),
        (
            plain(Kind::Schema(Schema::Unread {
                task: copy_of(b"core"),
                setting: copy_of(b"domian"),
                declared: Box::new([]),
            })),
            "reads core.domian, which core does not declare; core declares no settings\n",
            concat!(
                r#"{"record":"error","kind":"schema","schema":{"declared":[],"kind":"unread","setting":"domian","task":"core"},"class":"configuration"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}
