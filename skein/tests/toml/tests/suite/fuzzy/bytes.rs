//! Byte and limit sweeps (skein/docs/design/data/toml.md, 8).

use skein_lib::Rng;
use skein_toml::{Document, Limit, Limits, Table, Text, ValueKind, Why, parse};
use skein_world::{sweep::cases, this_test};

const CASES: u64 = 1024; // Small arbitrary strings plus seeds sampling inside valid grammar.
const BYTES: u32 = 256; // Room for the fixtures and arbitrary inputs of up to this size.
const DEPTH: u32 = 6; // Enough for the fixtures; a sweep also runs with smaller limits.
const ENTRIES: u32 = 24; // Covers all the entries and elements of the fixtures.
const FIXTURES: &[&[u8]] = &[
    b"name='toml'\n[a.'b.c']\nlist = [1, false, { dotted.key = 'val' },]\n",
    b"[[items]]\nname = \"\"\"\r\nThis \\ \r\n   folds\r\nlines.\"\"\"\n[[items]]\na = 2016-12-31T23:59:60Z\n",
    b"[a.b.c]\nla = 0b0_101\n[b]\nlt = [ 01:02:03, 8E2, -9223372036854775808 ]\n",
    b"# \nkey = 'Unicode at this position'\n",
];

#[test]
fn arbitrary_bytes_read_or_refuse_under_all_limits() {
    cases(this_test!("arbitrary_bytes_read_or_refuse_under_all_limits"), 0..CASES, |seed| {
        let mut rng = Rng::new(seed);
        let bytes = input(&mut rng);
        let ceiling = Limits { input_bytes: BYTES, depth: DEPTH, entries: ENTRIES };
        let tiny = Limits {
            input_bytes: u32::try_from(bytes.len()).expect("input within the byte cap").max(1),
            depth: u32::try_from(rng.between(1, u64::from(DEPTH))).expect("depth fits u32"),
            entries: u32::try_from(rng.between(1, u64::from(ENTRIES))).expect("entry count fits u32"),
        };
        let last_line = 1 + u64::from(skein_lib::bytes::count(&bytes, b"\n", BYTES));
        for limits in [ceiling, tiny] {
            match parse(&bytes, &limits) {
                Ok(document) => {
                    assert!(core::str::from_utf8(&bytes).is_ok(), "invalid UTF-8 was read: {bytes:?}");
                    let entries = check_tree(&document, &limits, last_line, bytes.len());
                    if entries > 1 {
                        let too_small = Limits { entries: entries - 1, ..limits };
                        let failure =
                            parse(&bytes, &too_small).expect_err("one fewer entry can no longer hold the read tree");
                        assert_eq!(failure.why, Why::Limit(Limit::Entries), "{bytes:?}");
                    }
                }
                Err(refusal) => {
                    line_in(refusal.line, last_line);
                    assert_ne!(refusal.why, Why::Limit(Limit::InputBytes), "the input is within the byte cap");
                }
            }
        }
        Ok::<(), std::convert::Infallible>(())
    })
    .expect("bytes are either read within bounds or refused");
}

fn input(rng: &mut Rng) -> Vec<u8> {
    let byte = |rng: &mut Rng| u8::try_from(rng.below(256)).expect("a drawn byte fits u8");
    match rng.below(4) {
        0 => (0..rng.between(0, u64::from(BYTES))).map(|_| byte(rng)).collect(),
        1 => {
            // Read arbitrary bytes where all valid UTF-8 literal characters are allowed.
            // No quote, newline, or forbidden ASCII control can hide invalid UTF-8 later.
            let mut input = b"key = '".to_vec();
            for _ in 0..rng.between(1, 24) {
                let sample = byte(rng);
                input.push(if sample < 0x80 { b'x' } else { sample });
            }
            input.push(b'\'');
            input
        }
        mode => {
            let count = u64::try_from(FIXTURES.len()).expect("small fixture count");
            let index = usize::try_from(rng.below(count)).expect("fixture index fits usize");
            let mut input = FIXTURES[index].to_vec();
            if mode == 2 {
                let len = u64::try_from(input.len()).expect("short fixture");
                let at = usize::try_from(rng.below(len)).expect("fixture offset fits usize");
                input[at] = byte(rng);
            }
            input
        }
    }
}

fn check_tree(document: &Document, limits: &Limits, last_line: u64, input_bytes: usize) -> u32 {
    let mut pending = Vec::from([ValueKind::Table(document.root())]);
    let mut retained_text = 0;
    let mut entries = 0;
    for _ in 0..=limits.entries {
        let Some(value) = pending.pop() else {
            assert!(retained_text <= input_bytes, "decoded text expands its source");
            return entries;
        };
        match value {
            ValueKind::Table(table) => {
                let len = document.table_len(table).expect("table handle belongs to the document");
                admit(&mut entries, len, limits.entries);
                retained_text += check_entries(document, table, len, last_line, &mut pending);
            }
            ValueKind::Array(array) | ValueKind::ArrayOfTables(array) => {
                let len = document.array_len(array).expect("array handle belongs to the document");
                admit(&mut entries, len, limits.entries);
                for index in 0..len {
                    let value = document.element(array, index).expect("array element at index");
                    line_in(value.line, last_line);
                    pending.push(value.kind);
                }
            }
            ValueKind::String(text) | ValueKind::Float(text) | ValueKind::DateTime { text, .. } => {
                retained_text += text_len(document, text);
            }
            ValueKind::Boolean(_) | ValueKind::Integer(_) => {}
        }
    }
    assert!(pending.is_empty(), "navigation found more nodes than were admitted");
    assert!(retained_text <= input_bytes, "decoded text expands its source");
    entries
}

fn admit(entries: &mut u32, new: u32, limit: u32) {
    *entries = entries.checked_add(new).expect("records fit u32");
    assert!(*entries <= limit, "retained {entries} records under a limit of {limit}");
}

fn check_entries(document: &Document, table: Table, len: u32, last_line: u64, pending: &mut Vec<ValueKind>) -> usize {
    let mut text = 0;
    for index in 0..len {
        let entry = document.entry(table, index).expect("table entry at index");
        line_in(entry.line, last_line);
        line_in(entry.value.line, last_line);
        text += text_len(document, entry.key);
        pending.push(entry.value.kind);
    }
    text
}

fn text_len(document: &Document, text: Text) -> usize {
    let bytes = document.text(text).expect("text handle belongs to the document");
    core::str::from_utf8(bytes).expect("retained spans are valid UTF-8").len()
}

fn line_in(line: u64, last_line: u64) {
    assert!((1..=last_line).contains(&line), "line {line} outside document ending at {last_line}");
}
