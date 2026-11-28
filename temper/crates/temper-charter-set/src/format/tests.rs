use super::*;

#[test]
fn format_one_body_is_exact() {
    for (text, expected) in [
        ("Goal text.\n", "Goal text.\n"),
        ("+++\nformat = 1\n+++\nGoal text.\n", "Goal text.\n"),
        ("+++\nformat=1\n+++\n\nGoal text.\n+++\n", "\nGoal text.\n+++\n"),
        ("++x\nformat = 2\n+++\n", "++x\nformat = 2\n+++\n"),
        ("+++\r\nformat  =  1\r\n+++\r\nGoal text.\r\n", "Goal text.\r\n"),
        ("+++\nformat = 001\n+++", ""),
    ] {
        assert_eq!(body(File::Goal, Some(text)), Ok(Some(expected)));
    }
}

#[test]
fn front_matter_must_be_closed_and_hold_only_format() {
    for text in
        ["+++\nformat = 1\nbase = \"x\"\n+++\n", "+++\nformat = 1\n", "+++\nbase = 1\n+++\n", "+++\n+++\n", "+++"]
    {
        assert_eq!(body(File::Goal, Some(text)), Err(Refusal::FrontMatter { file: File::Goal }));
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "build front matter around each candidate value")]
fn formats_are_positive_decimal_and_only_one_is_read() {
    for value in ["0", "000", "x", "", "+1", "-1", "1.0", "1_0", "１"] {
        let text = alloc::format!("+++\nformat = {value}\n+++\n");
        assert_eq!(body(File::Goal, Some(&text)), Err(Refusal::BadFormat { file: File::Goal }));
    }
    for number in ["2", "1000000000000000000000000000000000000000"] {
        let text = alloc::format!("+++\nformat = {number}\n+++\n");
        assert_eq!(
            body(File::Goal, Some(&text)),
            Err(Refusal::NewerFormat { file: File::Goal, number: number.into() })
        );
    }
}

#[test]
fn design_source_lines_include_its_front_matter() {
    let set = Set {
        common: None,
        goal: None,
        design: Some("+++\nformat = 1\n+++\ndesign\n"),
        designer: None,
        coordinator: None,
        implementer: None,
    };
    let refusal = Refusal::BeforeSection { file: File::Design, line: 1 };
    assert_eq!(refusal.in_source(&set), Refusal::BeforeSection { file: File::Design, line: 4 });
}
