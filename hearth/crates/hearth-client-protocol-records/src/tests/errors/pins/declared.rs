use super::assert_prints;
use super::plain;
use crate::errors::{Declared, Decode, Error, Expected, Kind, Map, Object, Place, Position, Required, Takes};
use skein_lib::bytes::copy_of;
fn declaration(declared: Declared) -> Error {
    let mut error = plain(Kind::Decode(Decode::Declaration(declared)));
    error.at = Some(Place { path: copy_of(b"tasks/motd/task.json"), position: Some(Position { line: 1, column: 2 }) });
    error.task = Some(copy_of(b"motd"));
    error
}

#[expect(clippy::too_many_lines, reason = "literal text and JSON pins keep each error beside its bytes")]
#[test]
fn declared_bytes() {
    let cases = [
        (
            declaration(Declared::Unknown { of: Object::Read }),
            "tasks/motd/task.json:1:2: motd: no such key in a read\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"read"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Guard }),
            "tasks/motd/task.json:1:2: motd: no such key in a guard\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"guard"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Script }),
            "tasks/motd/task.json:1:2: motd: no such key in a script\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"script"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Bless }),
            "tasks/motd/task.json:1:2: motd: no such key in a bless\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"bless"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Range }),
            "tasks/motd/task.json:1:2: motd: no such key in a range\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"range"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Spec }),
            "tasks/motd/task.json:1:2: motd: no such key in a spec\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"spec"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Export }),
            "tasks/motd/task.json:1:2: motd: no such key in a export\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"export"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Type }),
            "tasks/motd/task.json:1:2: motd: no such key in a type\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"type"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Packages) }),
            "tasks/motd/task.json:1:2: motd: no such key in a packages entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"packages entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Preseeds) }),
            "tasks/motd/task.json:1:2: motd: no such key in a preseeds entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"preseeds entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Initramfs) }),
            "tasks/motd/task.json:1:2: motd: no such key in a initramfs entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"initramfs entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Software) }),
            "tasks/motd/task.json:1:2: motd: no such key in a software entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"software entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Files) }),
            "tasks/motd/task.json:1:2: motd: no such key in a files entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"files entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Users) }),
            "tasks/motd/task.json:1:2: motd: no such key in a users entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"users entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Groups) }),
            "tasks/motd/task.json:1:2: motd: no such key in a groups entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"groups entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Presets) }),
            "tasks/motd/task.json:1:2: motd: no such key in a presets entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"presets entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Tmpfiles) }),
            "tasks/motd/task.json:1:2: motd: no such key in a tmpfiles entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"tmpfiles entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Data) }),
            "tasks/motd/task.json:1:2: motd: no such key in a data entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"data entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Secrets) }),
            "tasks/motd/task.json:1:2: motd: no such key in a secrets entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"secrets entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Identity) }),
            "tasks/motd/task.json:1:2: motd: no such key in a identity entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"identity entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Cmdline) }),
            "tasks/motd/task.json:1:2: motd: no such key in a cmdline entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"cmdline entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Unknown { of: Object::Entry(Map::Live) }),
            "tasks/motd/task.json:1:2: motd: no such key in a live entry\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"unknown","of":"live entry"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Object }),
            "tasks/motd/task.json:1:2: motd: not an object\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"an object"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::String }),
            "tasks/motd/task.json:1:2: motd: not a string\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a string"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Strings }),
            "tasks/motd/task.json:1:2: motd: not a list of strings\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a list of strings"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Bool }),
            "tasks/motd/task.json:1:2: motd: not a bool\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a bool"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Int }),
            "tasks/motd/task.json:1:2: motd: not an int\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"an int"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::True }),
            "tasks/motd/task.json:1:2: motd: not true\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"true"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Type }),
            "tasks/motd/task.json:1:2: motd: not a type\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a type"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Field(Takes::Text) }),
            "tasks/motd/task.json:1:2: motd: not a string\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a string"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Field(Takes::Int) }),
            "tasks/motd/task.json:1:2: motd: not an int\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"an int"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Field(Takes::Bool) }),
            "tasks/motd/task.json:1:2: motd: not a bool\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a bool"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Field(Takes::Strings) }),
            "tasks/motd/task.json:1:2: motd: not a list of strings\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a list of strings"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Wrong { expected: Expected::Field(Takes::Range) }),
            "tasks/motd/task.json:1:2: motd: not a range\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"wrong","expected":"a range"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Missing { key: Required::If }),
            "tasks/motd/task.json:1:2: motd: missing if\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"missing","key":"if"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Missing { key: Required::Message }),
            "tasks/motd/task.json:1:2: motd: missing message\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"missing","key":"message"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Missing { key: Required::Type }),
            "tasks/motd/task.json:1:2: motd: missing type\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"missing","key":"type"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Missing { key: Required::Doc }),
            "tasks/motd/task.json:1:2: motd: missing doc\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"missing","key":"doc"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Missing { key: Required::Value }),
            "tasks/motd/task.json:1:2: motd: missing value\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"missing","key":"value"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Later),
            "tasks/motd/task.json:1:2: motd: a key for a later milestone\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"later"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Name),
            "tasks/motd/task.json:1:2: motd: invalid declaration name\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"name"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Doc),
            "tasks/motd/task.json:1:2: motd: doc must be one line within its length limit\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"doc"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Order),
            "tasks/motd/task.json:1:2: motd: order must be from 0 to 999\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"order"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Optional),
            "tasks/motd/task.json:1:2: motd: optional beside a default\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"optional"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Bound),
            "tasks/motd/task.json:1:2: motd: a bound on a type it does not bound\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"bound"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Above),
            "tasks/motd/task.json:1:2: motd: min above max\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"above"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Enum),
            "tasks/motd/task.json:1:2: motd: an enum must list distinct values\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"enum"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Settings),
            "tasks/motd/task.json:1:2: motd: too many settings\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"settings"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
        (
            declaration(Declared::Secret),
            "tasks/motd/task.json:1:2: motd: an export type holds secret\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"declared":{"kind":"secret"},"kind":"declaration"},"class":"configuration","at":{"column":2,"line":1,"path":"tasks/motd/task.json"},"task":"motd"}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}
