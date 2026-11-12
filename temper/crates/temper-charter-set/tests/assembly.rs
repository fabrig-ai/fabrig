#![cfg(test)]
#![expect(clippy::disallowed_methods, clippy::disallowed_macros, clippy::disallowed_types, reason = "ordinary assembly tests generate texts independently of the parser")]
use temper_charter_set::{File, GoalAgent, Part, Refusal, Set, file, goal_agent, implementer};
fn reason(name: &str) -> Option<&'static str> {
    match name { "first" => Some("first"), "question" => Some("question"), "words" => Some("words"), "news" => Some("news"), "timer" => Some("timer"), _ => None }
}
fn empty() -> Set<'static> { Set { common: None, goal: None, designer: None, coordinator: None, implementer: None } }
#[test]
fn files_assemble_in_order_without_eating_inner_headings() {
    let set = Set { common: Some("A\n"), goal: Some("\nB\n\n"), designer: Some("D"), coordinator: Some("## part, on: question,  words\n# Q\nq\n## part, on: news\nn\n"), implementer: Some("\nI\n") };
    assert_eq!(goal_agent(&set, reason).unwrap(), GoalAgent { instructions: b"A\n\nB".as_slice().into(), parts: Box::new([
        Part { reasons: Box::new(["first"]), text: b"D".as_slice().into() },
        Part { reasons: Box::new(["question", "words"]), text: b"# Q\nq".as_slice().into() },
        Part { reasons: Box::new(["news"]), text: b"n".as_slice().into() },
    ]) });
    assert_eq!(implementer(&set).as_ref(), b"A\n\nI");
}
#[test]
fn only_boundary_line_feeds_disappear_and_empty_sections_stay() {
    for absent in [None, Some(""), Some("\n\n")] {
        let set = Set { common: absent, goal: absent, designer: absent, coordinator: absent, implementer: absent };
        assert_eq!(goal_agent(&set, reason).unwrap(), GoalAgent { instructions: Box::new([]), parts: Box::new([]) });
        assert!(implementer(&set).is_empty());
        let set = Set { common: Some("\n A\n\nZ \n"), ..set };
        assert_eq!(goal_agent(&set, reason).unwrap().instructions.as_ref(), b" A\n\nZ ");
        assert_eq!(implementer(&set).as_ref(), b" A\n\nZ ");
        let set = Set { common: absent, goal: Some("G"), implementer: Some("I"), ..set };
        assert_eq!(goal_agent(&set, reason).unwrap().instructions.as_ref(), b"G");
        assert_eq!(implementer(&set).as_ref(), b"I");
    }
    let set = Set { designer: Some("\nD\n\n"), coordinator: Some("\n## part, on: news\n\nx\n\ny\n\n## part, on: timer"), ..empty() };
    assert_eq!(goal_agent(&set, reason).unwrap().parts.as_ref(), [
        Part { reasons: Box::new(["first"]), text: b"D".as_slice().into() },
        Part { reasons: Box::new(["news"]), text: b"x\n\ny".as_slice().into() },
        Part { reasons: Box::new(["timer"]), text: Box::new([]) },
    ]);
}
#[test]
fn directory_names_refuse_only_unknown_markdown() {
    for known in [File::Common, File::Goal, File::Designer, File::Coordinator, File::Implementer] { assert_eq!(file(known.name()), Ok(Some(known))); }
    for ignored in ["settings.json", "common.txt", "other.MD", ""] { assert_eq!(file(ignored), Ok(None)); }
    let refusal = file("other.md").unwrap_err();
    assert_eq!(refusal, Refusal::UnknownFile { name: "other.md".into() });
    assert_eq!(refusal.to_string(), "other.md: unknown Markdown file");
}
#[test]
fn text_before_the_first_section_is_refused_at_its_line() {
    for text in ["\n# heading\n## part, on: news\nx", "\n \n", "\ntext"] {
        let refusal = goal_agent(&Set { coordinator: Some(text), ..empty() }, reason).unwrap_err();
        assert_eq!(refusal, Refusal::BeforeSection { file: File::Coordinator, line: 2 });
        assert_eq!(refusal.to_string(), "coordinator.md:2: text before first section");
    }
}
#[test]
fn empty_reasons_are_refused() {
    for heading in ["", " ", "news,,timer", ",news", "news, "] {
        let text = format!("\n## part, on: {heading}\nx");
        let refusal = goal_agent(&Set { coordinator: Some(&text), ..empty() }, reason).unwrap_err();
        assert_eq!(refusal, Refusal::EmptyReason { file: File::Coordinator, line: 2 });
        assert_eq!(refusal.to_string(), "coordinator.md:2: empty reason");
    }
}
#[test]
fn repeated_reasons_are_refused_with_name_and_line() {
    let refusal = goal_agent(&Set { coordinator: Some("## part, on: news, news\nx"), ..empty() }, reason).unwrap_err();
    assert_eq!(refusal, Refusal::RepeatedReason { file: File::Coordinator, line: 1, name: "news".into() });
    assert_eq!(refusal.to_string(), "coordinator.md:1: repeated reason news");
    assert_eq!(goal_agent(&Set { coordinator: Some("## part, on: news\nx\n## part, on: news\ny"), ..empty() }, reason).unwrap().parts.len(), 2);
}
#[test]
fn unknown_reasons_include_always_offered_kinds_and_designer_first() {
    for name in ["answer", "unknown"] {
        let text = format!("## part, on: {name}\nx");
        let refusal = goal_agent(&Set { coordinator: Some(&text), ..empty() }, reason).unwrap_err();
        assert_eq!(refusal, Refusal::UnknownReason { file: File::Coordinator, line: 1, name: name.into() });
        assert_eq!(refusal.to_string(), format!("coordinator.md:1: unknown reason {name}"));
    }
    let refusal = goal_agent::<()>(&Set { designer: Some("D"), ..empty() }, |_| None).unwrap_err();
    assert_eq!(refusal, Refusal::UnknownReason { file: File::Designer, line: 1, name: "first".into() });
}
mod fuzzy {
    use super::*;
    use skein_lib::Rng;
    fn generated(seed: u64) -> (String, Option<Vec<Part<&'static str>>>) {
        let mut rng = Rng::new(seed);
        let mut text = String::new();
        let mut sections: Vec<(Vec<&str>, String)> = Vec::new();
        let mut valid = true;
        for _ in 0..rng.below(32) {
            match rng.below(16) {
                0 => { text.push_str("## part, on: answer\n"); valid = false; }
                1 => { text.push_str("## part, on:\n"); valid = false; }
                2 => { text.push_str("## part, on: news, news\n"); valid = false; }
                3 => { text.push_str("## part, on: news,,timer\n"); valid = false; }
                4..=8 => { text.push_str("## part, on: question,  words\n"); sections.push((vec!["question", "words"], String::new())); }
                9..=11 => { text.push_str("## part, on: news\n"); sections.push((vec!["news"], String::new())); }
                choice => {
                    let line = match choice { 12 => "text λ\n", 13 => "# A question\n", 14 => "### part, on: unknown\n", _ => "\n" };
                    text.push_str(line);
                    if let Some((_, body)) = sections.last_mut() { body.push_str(line); } else if line != "\n" { valid = false; }
                }
            }
        }
        let expected = valid.then(|| sections.into_iter().map(|(reasons, body)| Part { reasons: reasons.into_boxed_slice(), text: body.trim_matches('\n').as_bytes().into() }).collect());
        (text, expected)
    }
    #[test]
    fn section_texts_and_reasons_replay_from_the_seed() {
        let mut accepted = 0_u32; let mut refused = 0_u32;
        for seed in 0..1024 {
            let (text, expected) = generated(seed);
            let replay = generated(seed); assert_eq!(text, replay.0); assert_eq!(expected, replay.1);
            let result = goal_agent(&Set { coordinator: Some(&text), ..empty() }, reason);
            match expected {
                Some(parts) => { accepted += 1; assert_eq!(result.unwrap().parts.as_ref(), parts, "seed {seed}: {text}"); }
                None => { refused += 1; assert!(result.is_err(), "seed {seed}: {text}"); }
            }
        }
        assert!(accepted > 20 && refused > 20);
    }
}
