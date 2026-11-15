use super::assert_prints;
use crate::tests::fixtures;
use hearth_domain_render_vocabulary as render;
use render::errors::{Decode, Error, Field, Kind, Limit};

use skein_lib::bytes::copy_of;
fn decode(decode: Decode) -> Error {
    let mut error = fixtures::duplicate();
    error.kind = Kind::Decode(decode);
    error.also = None;
    error
}

#[expect(clippy::too_many_lines, reason = "literal text and JSON pins keep each error beside its bytes")]
#[test]
fn decode_bytes() {
    let cases = [
        (
            decode(Decode::ListNull),
            "envs/lab.json:1:14: null inside a list\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"list_null","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Trailing),
            "envs/lab.json:1:14: not JSON: trailing bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"trailing","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooDeep),
            "envs/lab.json:1:14: JSON nested too deeply\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"too_deep","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::StringTooLong),
            "envs/lab.json:1:14: JSON string too long\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"string_too_long","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::NumberTooLong),
            "envs/lab.json:1:14: JSON number too long\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"number_too_long","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Number),
            "envs/lab.json:1:14: not JSON: invalid number\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"number","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Escape),
            "envs/lab.json:1:14: not JSON: invalid escape\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"escape","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Surrogate),
            "envs/lab.json:1:14: not JSON: invalid surrogate\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"surrogate","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Utf8),
            "envs/lab.json:1:14: not JSON: invalid UTF-8\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"utf8","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Control),
            "envs/lab.json:1:14: not JSON: control byte\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"control","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Truncated),
            "envs/lab.json:1:14: not JSON: truncated document\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"truncated","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::NotObject),
            "envs/lab.json:1:14: a document must be an object\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"not_object","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::NotInteger),
            "envs/lab.json:1:14: a number must be an integer\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"not_integer","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Range),
            "envs/lab.json:1:14: an integer outside i64\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"range","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Nul),
            "envs/lab.json:1:14: U+0000 is not allowed\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"nul","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Keys),
            "envs/lab.json:1:14: too many object keys\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"keys","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Note),
            "envs/lab.json:1:14: _note must be a string\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"note","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Name),
            "envs/lab.json:1:14: invalid task name\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"name","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Placement),
            "envs/lab.json:1:14: placement must be an object\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"placement","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Members),
            "envs/lab.json:1:14: members must be an object or null\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"members","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Member),
            "envs/lab.json:1:14: invalid member\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":"member","class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Settings { task: copy_of(b"motd") }),
            "envs/lab.json:1:14: settings for \"motd\" must be an object or null\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"task":"motd","kind":"settings"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Unknown { field: copy_of(b"bogus") }),
            "envs/lab.json:1:14: unknown field \"bogus\"\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"bogus","kind":"unknown"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Wrong { field: Field::Kind }),
            "envs/lab.json:1:14: wrong value for kind\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"kind","kind":"wrong"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Wrong { field: Field::Firmware }),
            "envs/lab.json:1:14: wrong value for firmware\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"firmware","kind":"wrong"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Wrong { field: Field::Hardware }),
            "envs/lab.json:1:14: wrong value for hardware\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"hardware","kind":"wrong"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Wrong { field: Field::On }),
            "envs/lab.json:1:14: wrong value for on\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"on","kind":"wrong"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Wrong { field: Field::Mutable }),
            "envs/lab.json:1:14: wrong value for mutable\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"mutable","kind":"wrong"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Missing { field: Field::Kind }),
            "envs/lab.json:1:14: missing kind\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"kind","kind":"missing"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Missing { field: Field::Firmware }),
            "envs/lab.json:1:14: missing firmware\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"firmware","kind":"missing"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Missing { field: Field::Hardware }),
            "envs/lab.json:1:14: missing hardware\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"hardware","kind":"missing"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Missing { field: Field::On }),
            "envs/lab.json:1:14: missing on\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"on","kind":"missing"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Missing { field: Field::Mutable }),
            "envs/lab.json:1:14: missing mutable\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"mutable","kind":"missing"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Misplaced { field: Field::Kind }),
            "envs/lab.json:1:14: misplaced kind\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"kind","kind":"misplaced"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Misplaced { field: Field::Firmware }),
            "envs/lab.json:1:14: misplaced firmware\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"firmware","kind":"misplaced"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Misplaced { field: Field::Hardware }),
            "envs/lab.json:1:14: misplaced hardware\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"hardware","kind":"misplaced"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Misplaced { field: Field::On }),
            "envs/lab.json:1:14: misplaced on\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"on","kind":"misplaced"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::Misplaced { field: Field::Mutable }),
            "envs/lab.json:1:14: misplaced mutable\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"field":"mutable","kind":"misplaced"},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Record, max: 64 }),
            "envs/lab.json:1:14: a record past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"record","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Closure, max: 64 }),
            "envs/lab.json:1:14: a closure past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"closure","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Template, max: 64 }),
            "envs/lab.json:1:14: a template past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"template","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Document, max: 1_048_576 }),
            "envs/lab.json:1:14: a document past 1048576 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"document","max":1048576},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Decoded, max: 64 }),
            "envs/lab.json:1:14: decoded documents past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"decoded","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Static, max: 8 }),
            "envs/lab.json:1:14: a static file past 8 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"static","max":8},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Entries, max: 64 }),
            "envs/lab.json:1:14: listing entries past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"entries","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Path, max: 64 }),
            "envs/lab.json:1:14: a path past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"path","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Paths, max: 64 }),
            "envs/lab.json:1:14: listing paths past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"paths","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Tasks, max: 64 }),
            "envs/lab.json:1:14: tasks past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"tasks","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
        (
            decode(Decode::TooLarge { limit: Limit::Targets, max: 64 }),
            "envs/lab.json:1:14: targets past 64 bytes\n",
            concat!(
                r#"{"record":"error","kind":"decode","decode":{"kind":"too_large","limit":"targets","max":64},"class":"configuration","at":{"column":14,"line":1,"path":"envs/lab.json"}}"#,
                "\n"
            ),
        ),
    ];
    for (error, text, json) in cases {
        assert_prints(&error, text, json);
    }
}
