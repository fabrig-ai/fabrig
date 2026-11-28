//! Focused whole-document cases (skein/docs/design/data/toml.md, 8).

use crate::{DateTimeKind, Document, Limit, Limits, Refusal, Table, TableKind, Value, ValueKind, Why, parse};

// Room for each fixture's bytes, at most four dotted segments or four open
// values, and at most the scalar fixture's 23 root entries plus four descendants.
const ROOM: Limits = Limits { input_bytes: 1024, depth: 4, entries: 27 };

type Entry = (&'static str, u64, u64, Form);
type Element = (u64, Form);

#[derive(Debug)]
enum Form {
    Boolean(bool),
    Integer(i64),
    Float(&'static str),
    String(&'static str),
    DateTime(DateTimeKind, &'static str),
    Table(TableKind, &'static [Entry]),
    Array(&'static [Element]),
    ArrayOfTables(&'static [Element]),
}

fn check_table(document: &Document, table: Table, kind: TableKind, entries: &[Entry]) {
    assert_eq!(document.table_kind(table), Some(kind));
    let len = u32::try_from(entries.len()).expect("fixture entries fit u32");
    assert_eq!(document.table_len(table), Some(len));
    for (index, (key, key_line, value_line, value)) in (0..len).zip(entries) {
        let entry = document.entry(table, index).expect("fixture entry at index");
        assert_eq!(document.text(entry.key), Some(key.as_bytes()));
        assert_eq!(document.get(table, key.as_bytes()), Some(entry));
        assert_eq!(entry.line, *key_line, "key {key}");
        check_value(document, entry.value, *value_line, value);
    }
    assert_eq!(document.entry(table, len), None);
    assert_eq!(document.get(table, b"key not in the fixture"), None);
}

fn check_value(document: &Document, value: Value, line: u64, expected: &Form) {
    assert_eq!(value.line, line, "{expected:?}");
    match (value.kind, expected) {
        (ValueKind::Boolean(flag), Form::Boolean(want)) => assert_eq!(flag, *want),
        (ValueKind::Integer(number), Form::Integer(want)) => assert_eq!(number, *want),
        (ValueKind::Float(text), Form::Float(want)) | (ValueKind::String(text), Form::String(want)) => {
            assert_eq!(document.text(text), Some(want.as_bytes()));
        }
        (ValueKind::DateTime { kind, text }, Form::DateTime(wanted_kind, want)) => {
            assert_eq!(kind, *wanted_kind);
            assert_eq!(document.text(text), Some(want.as_bytes()));
        }
        (ValueKind::Table(table), Form::Table(kind, entries)) => check_table(document, table, *kind, entries),
        (ValueKind::Array(array), Form::Array(values))
        | (ValueKind::ArrayOfTables(array), Form::ArrayOfTables(values)) => {
            let len = u32::try_from(values.len()).expect("fixture elements fit u32");
            assert_eq!(document.array_len(array), Some(len));
            for (index, (value_line, expected)) in (0..len).zip(*values) {
                let value = document.element(array, index).expect("fixture element at index");
                check_value(document, value, *value_line, expected);
            }
            assert_eq!(document.element(array, len), None);
        }
        _ => panic!("line {line}: wanted {expected:?}, got {:?}", value.kind),
    }
}

#[test]
fn reads_toml_10_forms_into_ordered_lined_values() {
    const SCALARS: &str = concat!(
        "\u{feff}# a comment\twith \u{feff}\r\n",
        "bare-key9 = true\r\n",
        "'literal.key' = false\n",
        "\"quoted\\u002Ekey\" = \"\\b\\t\\n\\f\\r\\\"\\\\\\u03bb\\U0001F642\"\n",
        "\"\" = 'C:\\temp\\x'\n",
        "mb = \"\"\"\r\njoined\\ \t\r\n    \n \tline\r\nquote: \"\"\"\"\"\n",
        "ml = '''\r\nback\\slash\nquote: ''''\n",
        "dec = +1_002\n",
        "min = -9_223_372_036_854_775_808\n",
        "max = 9223372036854775807\n",
        "hex = 0x00_Af\n",
        "oct = 0o00_77\n",
        "bin = 0b0_101\n",
        "frac = -1_234.5_6e+00_7\n",
        "exp = 0E-001\n",
        "inf = inf\n",
        "neginf = -inf\n",
        "nan = +nan\n",
        "datetime = 1979-05-27 07:32:00.123456789-07:30\n",
        "utc = 2016-12-31t23:59:60z\n",
        "local = 2000-02-29T00:00:00\n",
        "date = 0000-02-29\n",
        "time = 01:02:03.0123456789\n",
        "inline = { a = [\n  1, # comment inside array\n  true,\n], b = \"\"\"line\nline\"\"\" }",
    );
    const SCALAR_ENTRIES: &[Entry] = &[
        ("bare-key9", 2, 2, Form::Boolean(true)),
        ("literal.key", 3, 3, Form::Boolean(false)),
        ("quoted.key", 4, 4, Form::String("\x08\t\n\x0c\r\"\\λ🙂")),
        ("", 5, 5, Form::String("C:\\temp\\x")),
        ("mb", 6, 6, Form::String("joinedline\nquote: \"\"")),
        ("ml", 11, 11, Form::String("back\\slash\nquote: '")),
        ("dec", 14, 14, Form::Integer(1002)),
        ("min", 15, 15, Form::Integer(i64::MIN)),
        ("max", 16, 16, Form::Integer(i64::MAX)),
        ("hex", 17, 17, Form::Integer(175)),
        ("oct", 18, 18, Form::Integer(63)),
        ("bin", 19, 19, Form::Integer(5)),
        ("frac", 20, 20, Form::Float("-1_234.5_6e+00_7")),
        ("exp", 21, 21, Form::Float("0E-001")),
        ("inf", 22, 22, Form::Float("inf")),
        ("neginf", 23, 23, Form::Float("-inf")),
        ("nan", 24, 24, Form::Float("+nan")),
        ("datetime", 25, 25, Form::DateTime(DateTimeKind::OffsetDateTime, "1979-05-27 07:32:00.123456789-07:30")),
        ("utc", 26, 26, Form::DateTime(DateTimeKind::OffsetDateTime, "2016-12-31t23:59:60z")),
        ("local", 27, 27, Form::DateTime(DateTimeKind::LocalDateTime, "2000-02-29T00:00:00")),
        ("date", 28, 28, Form::DateTime(DateTimeKind::LocalDate, "0000-02-29")),
        ("time", 29, 29, Form::DateTime(DateTimeKind::LocalTime, "01:02:03.0123456789")),
        (
            "inline",
            30,
            30,
            Form::Table(
                TableKind::Inline,
                &[
                    ("a", 30, 30, Form::Array(&[(31, Form::Integer(1)), (32, Form::Boolean(true))])),
                    ("b", 33, 33, Form::String("line\nline")),
                ],
            ),
        ),
    ];
    const TREE: &str = concat!(
        "a . b = 1\n[c]\nd = \"x\"\n",
        "[implicitly.child]\n[implicitly]\nx = { \"with.dot\".y = [], 'with.dot'.z = {} }\n",
        "[a.sibling]\n",
        "[[fruit]]\nname = 'apple'\n[fruit.physical]\ncolor = 'red'\n",
        "[[fruit.variety]]\nname = 'gala'\n[[fruit.variety]]\nname = 'pink lady'\n",
        "[[fruit]]\nname = 'banana'\n[[fruit.variety]]\nname = 'plantain'\n",
    );
    const TREE_ENTRIES: &[Entry] = &[
        (
            "a",
            1,
            1,
            Form::Table(
                TableKind::Dotted,
                &[("b", 1, 1, Form::Integer(1)), ("sibling", 7, 7, Form::Table(TableKind::Header, &[]))],
            ),
        ),
        ("c", 2, 2, Form::Table(TableKind::Header, &[("d", 3, 3, Form::String("x"))])),
        (
            "implicitly",
            4,
            5,
            Form::Table(
                TableKind::Header,
                &[
                    ("child", 4, 4, Form::Table(TableKind::Header, &[])),
                    (
                        "x",
                        6,
                        6,
                        Form::Table(
                            TableKind::Inline,
                            &[(
                                "with.dot",
                                6,
                                6,
                                Form::Table(
                                    TableKind::Dotted,
                                    &[("y", 6, 6, Form::Array(&[])), ("z", 6, 6, Form::Table(TableKind::Inline, &[]))],
                                ),
                            )],
                        ),
                    ),
                ],
            ),
        ),
        (
            "fruit",
            8,
            8,
            Form::ArrayOfTables(&[
                (
                    8,
                    Form::Table(
                        TableKind::Header,
                        &[
                            ("name", 9, 9, Form::String("apple")),
                            (
                                "physical",
                                10,
                                10,
                                Form::Table(TableKind::Header, &[("color", 11, 11, Form::String("red"))]),
                            ),
                            (
                                "variety",
                                12,
                                12,
                                Form::ArrayOfTables(&[
                                    (12, Form::Table(TableKind::Header, &[("name", 13, 13, Form::String("gala"))])),
                                    (
                                        14,
                                        Form::Table(TableKind::Header, &[("name", 15, 15, Form::String("pink lady"))]),
                                    ),
                                ]),
                            ),
                        ],
                    ),
                ),
                (
                    16,
                    Form::Table(
                        TableKind::Header,
                        &[
                            ("name", 17, 17, Form::String("banana")),
                            (
                                "variety",
                                18,
                                18,
                                Form::ArrayOfTables(&[(
                                    18,
                                    Form::Table(TableKind::Header, &[("name", 19, 19, Form::String("plantain"))]),
                                )]),
                            ),
                        ],
                    ),
                ),
            ]),
        ),
    ];
    const MORE_STRINGS: &str = "quoted = \"\"\"\"a\"\"b\\\"c\"\"\"\"\nliteral = ''''a''b'''\nmark = '\u{feff}'\n";
    const MORE_ENTRIES: &[Entry] = &[
        ("quoted", 1, 1, Form::String("\"a\"\"b\"c\"")),
        ("literal", 2, 2, Form::String("'a''b")),
        ("mark", 3, 3, Form::String("\u{feff}")),
    ];
    for (input, entries) in [(SCALARS, SCALAR_ENTRIES), (TREE, TREE_ENTRIES), (MORE_STRINGS, MORE_ENTRIES), ("", &[])] {
        let document = parse(input.as_bytes(), &ROOM).expect(input);
        check_table(&document, document.root(), TableKind::Root, entries);
    }
}

#[test]
fn refuses_whole_documents_at_their_first_fault() {
    let cases: &[(&[u8], u64, Why)] = &[
        (b"a = 1\na = 2", 2, Why::KeyRedefined),
        (b"a.b = 1\na.\"\\u0062\" = 2", 2, Why::KeyRedefined),
        (b"a.b = 1\na = 2", 2, Why::KeyRedefined),
        (b"a = 1\na.b = 2", 2, Why::KeyRedefined),
        (b"[a.b]\n[a]\nb.c=2", 3, Why::KeyRedefined),
        (b"a = { b = { c = 1 }, b.d = 2 }", 1, Why::KeyRedefined),
        (b"[t]\n[t]", 2, Why::TableRedefined),
        (b"a.b = 1\n[a]", 2, Why::TableRedefined),
        (b"x = {a = 1}\n[x]", 2, Why::TableRedefined),
        (b"x = {a = 1}\n[x.b]", 2, Why::TableRedefined),
        (b"x = [{a = 1}]\n[x.b]", 2, Why::TableRedefined),
        (b"x = []\n[[x]]", 2, Why::TableRedefined),
        (b"[x]\n[[x]]", 2, Why::TableRedefined),
        (b"[[x]]\n[x]", 2, Why::TableRedefined),
        (b"[x.child]\n[[x]]", 2, Why::TableRedefined),
        (b"[[x]]\n[x.y]\n[x.y]", 3, Why::TableRedefined),
        (b"n = 9223372036854775808", 1, Why::Range),
        (b"n = -9223372036854775809", 1, Why::Range),
        (b"n = 0x8000_0000_0000_0000", 1, Why::Range),
        (b"d = 2023-02-30", 1, Why::Range),
        (b"d = 1900-02-29", 1, Why::Range),
        (b"d = 2023-13-01", 1, Why::Range),
        (b"d = 2023-00-01", 1, Why::Range),
        (b"d = 2023-01-00", 1, Why::Range),
        (b"d = 2023-01-01T24:00:00Z", 1, Why::Range),
        (b"d = 2023-01-01T23:59:61Z", 1, Why::Range),
        (b"d = 2023-01-01T00:00:00+00:60", 1, Why::Range),
        (b"d = 00:60:00", 1, Why::Range),
        (b"a=1\n# \xff", 2, Why::InvalidUtf8),
        (b"a=1\r\nb='\xf0\x9f'", 2, Why::InvalidUtf8),
        (b"a=1\n\xe2\x82", 2, Why::InvalidUtf8),
        (b"a = nope\n\xff", 1, Why::Grammar),
        (b"a = 1\na = \xff", 2, Why::KeyRedefined),
        (b"a=\"\x00\"", 1, Why::Grammar),
        (b"a=\"\"\"first\ncontrol\x7f\"\"\"", 2, Why::Grammar),
        (b"a='''first\ncontrol\x01'''", 2, Why::Grammar),
        (b"a='\x07'", 1, Why::Grammar),
        (b"a=1\n# control\x1a", 2, Why::Grammar),
        (b"a=1\rb=2", 1, Why::Grammar),
        (b"a='unclosed\n'", 1, Why::Grammar),
        (b"a=\"\"\"unclosed\n", 2, Why::Grammar),
        (b"a='''',,,,,,", 1, Why::Grammar),
        (b"a=\"\\e\"", 1, Why::Grammar),
        (b"a=\"\\x7f\"", 1, Why::Grammar),
        (b"a=\"\\uD800\"", 1, Why::Grammar),
        (b"a=\"\\U00110000\"", 1, Why::Grammar),
        (b"a=\"\"\"not\\   followed by a newline\"\"\"", 1, Why::Grammar),
        (b"a=[1,,2]", 1, Why::Grammar),
        (b"a=[1\n 2]", 2, Why::Grammar),
        (b"a=[1,\n", 2, Why::Grammar),
        (b"a={b=1,}", 1, Why::Grammar),
        (b"a={b=1,\nc=2}", 1, Why::Grammar),
        (b"a={b=1 # no comment here\n}", 1, Why::Grammar),
        (b"a. = 1", 1, Why::Grammar),
        (b"'''long'''=1", 1, Why::Grammar),
        (b"a = 1 b = 2", 1, Why::Grammar),
        (b"[[]]", 1, Why::Grammar),
        (b"[a] b=1", 1, Why::Grammar),
        (b"a=TRUE", 1, Why::Grammar),
        (b"a=infinitude", 1, Why::Grammar),
        (b"a=0_1", 1, Why::Grammar),
        (b"a=+0x1", 1, Why::Grammar),
        (b"a=0o8", 1, Why::Grammar),
        (b"a=0b1_", 1, Why::Grammar),
        (b"a=1__0", 1, Why::Grammar),
        (b"a=01e1", 1, Why::Grammar),
        (b"a=.1", 1, Why::Grammar),
        (b"a=1.", 1, Why::Grammar),
        (b"a=1e+_1", 1, Why::Grammar),
        (b"a=01:02", 1, Why::Grammar),
        (b"\xef\xbb\xbf\xef\xbb\xbfa=1", 1, Why::Grammar),
    ];
    for &(input, line, why) in cases {
        assert_eq!(parse(input, &ROOM).expect_err("the document is refused"), Refusal { line, why }, "{input:?}");
    }
}

const INPUT_AT: &[u8] = b"\xef\xbb\xbfx=1";
const INPUT_PAST: &[u8] = b"\xef\xbb\xbfx=1\n";
const INPUT_EDGE: Limits = Limits { input_bytes: 6, ..ROOM }; // BOM and x=1 are three bytes each.
const DEPTH_EDGE: Limits = Limits { depth: 3, ..ROOM }; // three arrays, or one inline table with a two-part key.
const ENTRY_EDGE: Limits = Limits { entries: 3, ..ROOM }; // two keyed entries and one array element, or an AoT with two elements.

#[test]
fn accepts_documents_exactly_at_each_limit() {
    let cases: &[(&[u8], Limits)] = &[
        (INPUT_AT, INPUT_EDGE),
        (b"x=[\n[\n[0]]\n]", DEPTH_EDGE),
        (b"x={a.b=1}", DEPTH_EDGE),
        (b"[a.b.c]", DEPTH_EDGE),
        (b"a.b=[\ntrue\n]", ENTRY_EDGE),
        (b"[[a]]\n[[a]]", ENTRY_EDGE),
    ];
    for (input, limits) in cases {
        let document = parse(input, limits).expect("the document is exactly at the named limit");
        assert_eq!(document.table_len(document.root()), Some(1));
    }
}

#[test]
fn refuses_documents_one_past_each_limit() {
    let cases: &[(&[u8], Limits, u64, Limit)] = &[
        (INPUT_PAST, INPUT_EDGE, 1, Limit::InputBytes),
        (b"x=[\n[\n[\n[0]]\n]\n]", DEPTH_EDGE, 4, Limit::Depth),
        (b"x={a.b.c=1}", DEPTH_EDGE, 1, Limit::Depth),
        (b"\n[a.b.c.d]", DEPTH_EDGE, 2, Limit::Depth),
        (b"a.b=[\ntrue,\nfalse\n]", ENTRY_EDGE, 3, Limit::Entries),
        (b"[[a]]\n[[a]]\n[[a]]", ENTRY_EDGE, 3, Limit::Entries),
    ];
    for (input, limits, line, limit) in cases {
        let refusal = Refusal { line: *line, why: Why::Limit(*limit) };
        assert_eq!(parse(input, limits).expect_err("one past the bound refuses"), refusal, "{input:?}");
    }
}
