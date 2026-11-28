//! Written models against both readers (skein/docs/design/data/toml.md, 8).

use std::fmt::Write;

use skein_lib::Rng;
use skein_toml::{Document, Entry, Limits, Refusal, Table, TableKind, ValueKind, Why, parse};
use skein_world::{sweep::cases, this_test};

// The writer emits two root entries, four fields/child tables, and up to
// four array elements. Inline nesting reaches two, with keys of up to three segments.
const LIMITS: Limits = Limits { input_bytes: 1024, depth: 4, entries: 10 };
const CASES: u64 = 192; // Samples independent string, base, key and array layouts for each table form.
const ARRAY_MAX: u64 = 4;
const WORD_MAX: u64 = 15;
const GLYPHS: &[char] = &['x', 'Z', ' ', '\t', '#', '.', 'é', '中', '🙂']; // Writable in all four forms.

struct Model {
    flag: bool,
    number: i64,
    words: Vec<String>,
    note: String,
}

#[derive(Clone, Copy)]
enum Tables {
    Headers,
    Dotted,
    Inline,
}

struct Source {
    text: String,
    line: u64,
}

impl Source {
    fn emit(&mut self, text: &str) {
        self.line += u64::try_from(text.bytes().filter(|byte| *byte == b'\n').count()).expect("lines fit u64");
        self.text.push_str(text);
    }
}

struct Lines {
    settings: u64,
    number: u64,
    child: u64,
    words: u64,
    elements: Vec<u64>,
    note: u64,
}

#[test]
fn written_models_round_trip_in_each_table_spelling() {
    cases(this_test!("written_models_round_trip_in_each_table_spelling"), 0..CASES, |seed| {
        let mut rng = Rng::new(seed);
        let model = model(&mut rng);
        for tables in [Tables::Headers, Tables::Dotted, Tables::Inline] {
            let (mut source, lines) = write(&model, tables, &mut rng);
            let result = parse(source.text.as_bytes(), &LIMITS);
            let oracle = toml::from_str::<toml::Table>(&source.text);
            assert_eq!(result.is_ok(), oracle.is_ok(), "{:?}: {result:?} vs {oracle:?}", source.text);
            let document = result.expect(&source.text);
            check(&document, &model, tables, &lines);

            // Already defined through headers or dotted keys, or sealed inline.
            source.emit("\n");
            let duplicate = source.line;
            source.emit("[settings.child]\n");
            let result = parse(source.text.as_bytes(), &LIMITS);
            let oracle = toml::from_str::<toml::Table>(&source.text);
            assert_eq!(result.is_ok(), oracle.is_ok(), "{:?}: {result:?} vs {oracle:?}", source.text);
            assert_eq!(
                result.expect_err("a child table is not redefined"),
                Refusal { line: duplicate, why: Why::TableRedefined }
            );
        }
        Ok::<(), std::convert::Infallible>(())
    })
    .expect("the seeded models agree with both readers");
}

fn model(rng: &mut Rng) -> Model {
    let flag = rng.chance(500);
    let magnitude = i64::try_from(rng.next_u64() >> 1).expect("63 bits fit i64");
    let number = if rng.chance(500) { -magnitude } else { magnitude };
    let words = (0..rng.between(0, ARRAY_MAX)).map(|_| word(rng)).collect();
    let note = word(rng);
    Model { flag, number, words, note }
}

fn word(rng: &mut Rng) -> String {
    let alphabet = u64::try_from(GLYPHS.len()).expect("small alphabet");
    (0..rng.between(0, WORD_MAX))
        .map(|_| GLYPHS[usize::try_from(rng.below(alphabet)).expect("glyph index fits usize")])
        .collect()
}

fn write(model: &Model, tables: Tables, rng: &mut Rng) -> (Source, Lines) {
    let mut source = Source { text: String::new(), line: 1 };
    source.emit(if model.flag { "flag = true\n" } else { "flag = false\n" });
    let newline = if rng.chance(500) { "\r\n" } else { "\n" };
    source.emit(newline);
    let settings = source.line;
    let number = integer(model.number, rng);
    match tables {
        Tables::Headers => source.emit("[\"settings\"]\ncount = "),
        Tables::Dotted => source.emit("'settings' . count = "),
        Tables::Inline => source.emit("settings = { \"count\" = "),
    }
    let number_line = source.line;
    source.emit(&number);
    match tables {
        Tables::Headers | Tables::Dotted => source.emit(newline),
        Tables::Inline => source.emit(", "),
    }
    let child = source.line;
    match tables {
        Tables::Headers => source.emit("[settings.'child']\nwords = "),
        Tables::Dotted => source.emit("settings . \"child\" . words = "),
        Tables::Inline => source.emit("'child' = { words = "),
    }
    let words = source.line;
    let mut elements = Vec::new();
    source.emit("[");
    for word in &model.words {
        source.emit(if rng.chance(500) { " " } else { newline });
        elements.push(source.line);
        string(&mut source, word, rng);
        source.emit(",");
    }
    source.emit("]");
    match tables {
        Tables::Headers => source.emit("\nnote = "),
        Tables::Dotted => source.emit("\nsettings.child.note = "),
        Tables::Inline => source.emit(", note = "),
    }
    let note = source.line;
    string(&mut source, &model.note, rng);
    if let Tables::Inline = tables {
        source.emit(" } }");
    }
    (source, Lines { settings, number: number_line, child, words, elements, note })
}

fn integer(number: i64, rng: &mut Rng) -> String {
    let base = rng.below(4);
    let (prefix, digits) = if number < 0 {
        ("-", number.unsigned_abs().to_string())
    } else {
        match base {
            0 => (if rng.chance(500) { "+" } else { "" }, number.to_string()),
            1 => ("0x", format!("{number:x}")),
            2 => ("0o", format!("{number:o}")),
            _ => ("0b", format!("{number:b}")),
        }
    };
    let mut text = String::from(prefix);
    for (offset, character) in digits.chars().enumerate() {
        if offset != 0 && rng.chance(250) {
            text.push('_');
        }
        text.push(character);
    }
    text
}

fn string(source: &mut Source, word: &str, rng: &mut Rng) {
    match rng.below(4) {
        0 => {
            source.emit("'");
            source.emit(word);
            source.emit("'");
        }
        1 => {
            source.emit("'''\r\n");
            source.emit(word);
            source.emit("'''");
        }
        style => {
            let quote = if style == 2 { "\"" } else { "\"\"\"" };
            source.emit(quote);
            if style == 3 {
                source.emit("\n");
            }
            let mut encoded = String::new();
            for character in word.chars() {
                if style == 3 && rng.chance(125) {
                    encoded.push_str("\\  \r\n  \t");
                }
                if rng.chance(500) || character == ' ' || character == '\t' {
                    let code = u32::from(character);
                    if u16::try_from(code).is_ok() && rng.chance(500) {
                        write!(encoded, "\\u{code:04x}").expect("writing into a String works");
                    } else {
                        write!(encoded, "\\U{code:08x}").expect("writing into a String works");
                    }
                } else {
                    encoded.push(character);
                }
            }
            source.emit(&encoded);
            source.emit(quote);
        }
    }
}

fn check(document: &Document, model: &Model, tables: Tables, lines: &Lines) {
    let root = document.root();
    assert_eq!(document.table_len(root), Some(2)); // flag, settings
    let flag = check_entry(document, root, 0, b"flag", 1);
    assert_eq!(flag.value.kind, ValueKind::Boolean(model.flag));
    let settings = check_entry(document, root, 1, b"settings", lines.settings);
    let kind = match tables {
        Tables::Headers => TableKind::Header,
        Tables::Dotted => TableKind::Dotted,
        Tables::Inline => TableKind::Inline,
    };
    let settings = check_table(document, settings, kind);
    assert_eq!(document.table_len(settings), Some(2)); // count, child
    let number = check_entry(document, settings, 0, b"count", lines.number);
    assert_eq!(number.value.kind, ValueKind::Integer(model.number));
    let child = check_entry(document, settings, 1, b"child", lines.child);
    let child = check_table(document, child, kind);
    assert_eq!(document.table_len(child), Some(2)); // words, note
    let words = check_entry(document, child, 0, b"words", lines.words);
    let ValueKind::Array(array) = words.value.kind else { panic!("expected an array, got {words:?}") };
    let len = u32::try_from(model.words.len()).expect("array length fits u32");
    assert_eq!(document.array_len(array), Some(len));
    for (index, (word, line)) in (0..len).zip(model.words.iter().zip(&lines.elements)) {
        let element = document.element(array, index).expect("array element at index");
        assert_eq!(element.line, *line);
        check_string(document, element.kind, word);
    }
    let note = check_entry(document, child, 1, b"note", lines.note);
    check_string(document, note.value.kind, &model.note);
}

fn check_entry(document: &Document, table: Table, index: u32, key: &[u8], line: u64) -> Entry {
    let entry = document.entry(table, index).expect("entry at index");
    assert_eq!(document.get(table, key), Some(entry));
    assert_eq!(document.text(entry.key), Some(key));
    assert_eq!((entry.line, entry.value.line), (line, line));
    entry
}

fn check_table(document: &Document, entry: Entry, kind: TableKind) -> Table {
    let ValueKind::Table(table) = entry.value.kind else { panic!("expected a table, got {entry:?}") };
    assert_eq!(document.table_kind(table), Some(kind));
    table
}

fn check_string(document: &Document, value: ValueKind, word: &str) {
    let ValueKind::String(text) = value else { panic!("expected a string, got {value:?}") };
    assert_eq!(document.text(text), Some(word.as_bytes()));
}
