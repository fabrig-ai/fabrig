#![cfg(test)]
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "step tests build fixtures and render the resulting faults"
)]
use temper_charter_set::{CharterFault, CharterFile, FilePart, Set, WritePart, read_charter, write_charter};

const REASONS: &[&str] = &["first", "question", "result"];
const FILE_BYTES: usize = 65_536; // Protocol temper-dir, 3.3, including front matter.
const NAME_BYTES: usize = 32; // Protocol temper-dir, 4; the same bound as jig's is_name.
const CAREFUL: &str =
    "+++\nsettings = \"implementer\"\n+++\n# Careful\n\nCheck twice.\n\n## part, on: first\nRead the spec first.";
const INSTRUCTIONS: &str = "Check twice.";

fn front(keys: &str, body: &str) -> String {
    format!("+++\n{keys}+++\n{body}")
}

fn words(fault: CharterFault) -> (u32, String) {
    (fault.line, fault.to_string())
}

fn assert_read_fault(text: &str, line: u32, why: &str) {
    assert_eq!(read_charter(text, REASONS).map_err(words), Err((line, why.into())), "file: {text:?}");
}

#[test]
fn front_matter_is_optional_and_toml_keys_are_unordered() {
    for (keys, settings) in [
        (None, None),
        (Some(""), None),
        (Some("format = 1\n"), None),
        (Some("settings = \"implementer\"\n"), Some("implementer")),
        (Some("format = 1\nsettings = \"implementer\"\n"), Some("implementer")),
        (Some("settings = \"implementer\"\nformat = 1\n"), Some("implementer")),
        (Some("# comment\n\n  settings = 'implementer' # comment\n'format' = 1\n"), Some("implementer")),
    ] {
        let text = keys.map_or_else(|| INSTRUCTIONS.into(), |keys| front(keys, INSTRUCTIONS));
        assert_eq!(
            read_charter(&text, REASONS),
            Ok(CharterFile {
                format: 1,
                settings: settings.map(Into::into),
                instructions: INSTRUCTIONS.into(),
                parts: Box::new([]),
            }),
            "{keys:?}"
        );
    }
}

#[test]
fn newer_format_is_reported_before_unknown_keys_or_body() {
    let text = front("model = 'x'\nformat = 2\n", "## part, on: frist");
    assert_read_fault(
        &text,
        3,
        "format 2; this temper-engine reads format 1: upgrade it, or write the file at format 1",
    );
}

#[test]
fn format_must_be_a_positive_decimal_toml_integer() {
    for value in ["0", "-1", "1.0", "'1'", "true", "0x1", "[1]"] {
        assert_read_fault(&front(&format!("model = 'x'\nformat = {value}\n"), ""), 3, "format: a whole number from 1");
    }
}

#[test]
fn unknown_key_names_its_line_and_what_belongs_here() {
    assert_read_fault(
        &front("format = 1\nmodel = \"x\"\n", ""),
        3,
        "unknown key `model`: a charter's file holds words alone; models, tools, wakes and budgets are settings",
    );
}

#[test]
fn unclosed_front_matter_points_to_the_opening_delimiter() {
    for text in ["+++", "+++\nsettings = 'implementer'\n## part, on: first\n", "+++\r\nformat = 1\r\n"] {
        assert_read_fault(text, 1, "front matter: no closing `+++`");
    }
}

#[test]
fn settings_accepts_deployment_names_at_jigs_bound() {
    let longest = "a".repeat(NAME_BYTES);
    for name in ["goal", "draft-goal", "a1", &longest] {
        let text = front(&format!("settings = '{name}'\n"), "");
        assert_eq!(read_charter(&text, REASONS).unwrap().settings.as_deref(), Some(name));
    }
}

#[test]
fn invalid_settings_names_are_refused_at_the_key() {
    let too_long = "a".repeat(NAME_BYTES + 1);
    for name in ["", "Goal", "1goal", "-goal", "goal_x", "12", &too_long, "Hand-over", "é"] {
        assert_read_fault(
            &front(&format!("# comment\nsettings = '{name}'\n"), ""),
            3,
            &format!("`{name}` is no name: 1 to 32 lowercase letters, digits and `-`, from a letter"),
        );
    }
}

#[test]
fn non_string_settings_names_report_the_expected_type() {
    for value in ["1", "true", "['implementer']", "{ name = 'implementer' }"] {
        assert_read_fault(&front(&format!("settings = {value}\n"), ""), 2, "`settings`: a charter name");
    }
}

#[test]
fn example_reads_with_its_original_heading_line() {
    assert_eq!(
        read_charter(CAREFUL, REASONS),
        Ok(CharterFile {
            format: 1,
            settings: Some("implementer".into()),
            instructions: "# Careful\n\nCheck twice.".into(),
            parts: Box::new([FilePart {
                line: 8,
                reasons: Box::new(["first".into()]),
                text: "Read the spec first.".into()
            }]),
        })
    );
}

#[test]
fn parts_keep_heading_order_and_trim_only_boundary_line_feeds() {
    let text = "\n\n  Instructions. \n\n## part, on: first,  question \n\n### Inner\n\n Text λ. \r\n\n## part, on: result, first\n\n";
    assert_eq!(
        read_charter(text, REASONS),
        Ok(CharterFile {
            format: 1,
            settings: None,
            instructions: "  Instructions. ".into(),
            parts: Box::new([
                FilePart {
                    line: 5,
                    reasons: Box::new(["first".into(), "question".into()]),
                    text: "### Inner\n\n Text λ. \r".into()
                },
                FilePart { line: 11, reasons: Box::new(["result".into(), "first".into()]), text: "".into() },
            ]),
        })
    );
}

#[test]
fn bad_reasons_name_the_offending_heading_and_its_line() {
    for (heading, why) in [
        ("## part, on: frist", "`frist` is no reason a part is selected by"),
        ("## part, on:", "a part needs at least one reason"),
        ("## part, on: first,,result", "a reason is empty"),
        ("## part, on: first, question, first", "`first` is repeated"),
    ] {
        let text = front("# comment\n\nformat = 1\n", &format!("{INSTRUCTIONS}\n\n{heading}\n"));
        assert_read_fault(&text, 8, &format!("`{heading}`: {why}"));
    }
}

#[test]
fn writer_emits_only_the_canonical_charter_body() {
    let parts = [
        WritePart { reasons: &["first", "result"], text: "\n\nPart.\n" },
        WritePart { reasons: &["question"], text: "" },
    ];
    assert_eq!(
        write_charter("\nCheck twice.\n\n", &parts).as_deref(),
        Ok("Check twice.\n\n## part, on: first, result\nPart.\n\n## part, on: question")
    );
}

#[test]
fn writer_round_trips_empty_texts_and_a_literal_front_delimiter() {
    let empty = [WritePart { reasons: &["first"], text: "" }];
    let full = [WritePart { reasons: REASONS, text: "# Inner heading\n\nText." }];
    for (instructions, parts) in [
        (INSTRUCTIONS, [].as_slice()),
        ("", full.as_slice()),
        (INSTRUCTIONS, empty.as_slice()),
        ("+++\n\nInstructions.", empty.as_slice()),
        ("+++", [].as_slice()),
    ] {
        assert_round_trip(instructions, parts);
    }
}

fn assert_round_trip(instructions: &str, parts: &[WritePart<'_>]) {
    let written = write_charter(instructions, parts).unwrap();
    let expected: Vec<_> = parts.iter().map(|part| (part.reasons.to_vec(), part.text)).collect();
    let read = read_charter(&written, REASONS).unwrap_or_else(|fault| panic!("{written:?}:{}: {fault}", fault.line));
    let contents: Vec<_> =
        read.parts.iter().map(|part| (part.reasons.iter().map(AsRef::as_ref).collect(), &*part.text)).collect();
    assert_eq!((read.format, read.settings, &*read.instructions, contents), (1, None, instructions, expected));
}

#[test]
fn writer_names_the_text_and_line_of_nested_headings() {
    let bad = "\nA line.\n## part, on: first";
    let parts = [WritePart { reasons: &["first"], text: "" }, WritePart { reasons: &["question"], text: bad }];
    for (instructions, parts, place) in
        [(bad, [].as_slice(), "instructions"), (INSTRUCTIONS, parts.as_slice(), "part 2")]
    {
        assert_eq!(
            write_charter(instructions, parts).map_err(words),
            Err((3, format!("{place}: a line begins `## part, on:`")))
        );
    }
}

#[test]
fn reader_counts_front_matter_toward_the_file_bound() {
    let keys = "settings = 'implementer'\n";
    let shell = front(keys, "");
    let body_bytes = FILE_BYTES - shell.len();
    let too_long = (0, "past what temper reads, 65,536 bytes".to_owned());
    for (length, expected) in [(body_bytes, Ok(body_bytes)), (body_bytes + 1, Err(too_long))] {
        let text = front(keys, &"a".repeat(length));
        assert_eq!(read_charter(&text, REASONS).map(|file| file.instructions.len()).map_err(words), expected);
    }
}

#[test]
fn writer_refuses_to_make_a_file_past_the_bound() {
    let too_long = (0, "past what temper reads, 65,536 bytes".to_owned());
    for (length, expected) in [(FILE_BYTES, Ok(FILE_BYTES)), (FILE_BYTES + 1, Err(too_long))] {
        assert_eq!(write_charter(&"a".repeat(length), &[]).map(|text| text.len()).map_err(words), expected);
    }
}

#[test]
fn overlay_refusal_names_the_project_only_key_and_line() {
    for (keys, why) in [
        ("settings = 'implementer'\n", "goal.md:2: `settings` is a project's charter file's key"),
        (
            "format = 1\n# comment\n'settings' = 'implementer'\n",
            "goal.md:4: `settings` is a project's charter file's key",
        ),
    ] {
        let text = front(keys, INSTRUCTIONS);
        let set = Set { goal: Some(&text), common: None, designer: None, coordinator: None, implementer: None };
        assert_eq!(set.overlay_bodies().unwrap_err().to_string(), why);
    }
}

mod fuzzy {
    use super::*;
    use skein_lib::Rng;

    fn generated_text(rng: &mut Rng) -> String {
        const FRAGMENTS: &[&str] = &["+++", "\n", "\n\n", "\t", "\r\n", "  ", "## Inner heading", " λ "];
        const FRAGMENT_LIMIT: u64 = 24;
        let mut text = String::new();
        for _ in 0..rng.below(FRAGMENT_LIMIT) {
            if rng.below(2) == 0 {
                let at = usize::try_from(rng.below(FRAGMENTS.len().try_into().unwrap())).unwrap();
                text.push_str(FRAGMENTS[at]);
            } else if let Some(ch) = char::from_u32(rng.below(0x11_0000).try_into().unwrap()) {
                text.push(ch);
            }
        }
        text.trim_matches('\n').into()
    }

    fn generated_reasons(rng: &mut Rng) -> Vec<&'static str> {
        const NONEMPTY_MASKS: u64 = (1 << REASONS.len()) - 1;
        let mask = rng.below(NONEMPTY_MASKS).checked_add(1).unwrap();
        REASONS.iter().copied().enumerate().filter_map(|(at, name)| (mask & (1 << at) != 0).then_some(name)).collect()
    }

    #[test]
    fn generated_charters_round_trip_without_changing_words() {
        const SEEDS: u64 = 1024;
        const PART_LIMIT: u64 = 8;
        for seed in 0..SEEDS {
            let mut rng = Rng::new(seed);
            let instructions = generated_text(&mut rng);
            let parts: Vec<_> =
                (0..rng.below(PART_LIMIT)).map(|_| (generated_reasons(&mut rng), generated_text(&mut rng))).collect();
            let parts: Vec<_> = parts.iter().map(|(reasons, text)| WritePart { reasons, text }).collect();
            assert_round_trip(&instructions, &parts);
        }
    }
}
