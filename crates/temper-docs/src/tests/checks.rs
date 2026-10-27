use super::*;

fn refused(entry: &Page, rule: &str, line: usize) -> String {
    format!(
        "{}: {rule}: line {line}: {}",
        entry.name,
        entry.text.lines().nth(line.saturating_sub(1)).unwrap_or("<end>")
    )
}

pub(super) fn head(entry: &Page) -> Result<(), String> {
    let name = entry.name;
    if name.is_empty()
        || name.len() > NAME_BYTES
        || !name.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err(refused(entry, "name: a-z, 0-9 and -, at most 32 bytes", 1));
    }
    for (at, line) in entry.text.split('\n').enumerate() {
        if line.chars().any(char::is_control) {
            return Err(refused(entry, "no control character but \\n", at + 1));
        }
    }
    let mut paragraphs = entry.text.split("\n\n");
    let title = paragraphs.next().unwrap_or("").strip_prefix("# ");
    if title != Some(entry.title) || entry.title.len() > TITLE_BYTES || entry.title.contains('\n') {
        return Err(refused(entry, "title line equal to row, at most 80 bytes", 1));
    }
    if paragraphs.next() != Some(&format!("Kind: {}", entry.kind.word())) {
        return Err(refused(entry, "kind line equal to row", 3));
    }
    let summary = paragraphs.next().unwrap_or("").lines().collect::<Vec<_>>().join(" ");
    if summary != entry.summary || summary.len() > SUMMARY_BYTES || !summary.ends_with('.') {
        return Err(refused(entry, "summary equal to row, at most 200 bytes, ending in a period", 5));
    }
    for (heading, offset, _) in sections(entry).into_iter().skip(1) {
        if heading.len() > HEADING_BYTES {
            return Err(refused(entry, "heading at most 80 bytes", entry.text[..offset].lines().count() + 1));
        }
    }
    Ok(())
}

pub(super) fn template(entry: &Page) -> Result<(), String> {
    if entry.text.len() > entry.kind.bound() {
        return Err(refused(
            entry,
            &format!("{} bound {} bytes", entry.kind.word(), entry.kind.bound()),
            entry.text.lines().count(),
        ));
    }
    let sections = sections(entry);
    let headings = sections.iter().skip(1).map(|s| s.0).collect::<Vec<_>>();
    let valid = match entry.kind {
        Kind::Concept => {
            headings == ["What it is", "How it changes", "What temper guarantees", "Acting on it", "Limits"]
                || headings == ["What it is", "What temper guarantees", "Acting on it", "Limits"]
        }
        Kind::Recipe => headings == ["Goal", "Settings", "Enforced and followed", "Checking it took"],
        Kind::Walk => {
            headings.len() >= 4
                && headings[0] == "Before you start"
                && headings[headings.len() - 2..] == ["Clean up", "Next"]
                && headings[1..headings.len() - 2].iter().enumerate().all(|(n, heading)| {
                    heading.strip_prefix(&format!("{}. ", n + 1)).is_some_and(|rest| !rest.is_empty())
                })
        }
        Kind::Index | Kind::Reference | Kind::Guide => true,
    };
    if !valid {
        return Err(refused(entry, &format!("{} headings in template order", entry.kind.word()), 1));
    }
    if entry.kind == Kind::Walk {
        for (_, offset, section) in sections.iter().skip(2).take(headings.len() - 3) {
            let mut fenced = false;
            let expect = section.lines().any(|line| {
                if line.starts_with("```") {
                    fenced = !fenced;
                }
                !fenced && line.starts_with("Expect:")
            });
            if !expect {
                return Err(refused(entry, "walk step holds Expect:", entry.text[..*offset].lines().count() + 1));
            }
        }
    }
    Ok(())
}

pub(super) fn links(entry: &Page) -> Result<(), String> {
    for (at, line) in entry.text.lines().enumerate() {
        for after in line.split("temper help ").skip(1) {
            let name =
                after.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')).next().unwrap_or("");
            if !name.is_empty() && name != "--search" && find(name.as_bytes()).is_none() {
                return Err(refused(entry, &format!("link {name} names a page"), at + 1));
            }
        }
    }
    Ok(())
}

pub(super) fn written(entry: &Page) -> bool {
    match entry.kind {
        Kind::Index | Kind::Concept | Kind::Walk | Kind::Recipe => true,
        Kind::Reference | Kind::Guide => false,
    }
}

pub(super) fn placeholders(entry: &Page) -> Result<(), String> {
    if !written(entry) {
        return Ok(());
    }
    let mut allowed = vec![
        "DIR",
        "CONFIG",
        "DATA",
        "CACHE",
        "FILE",
        "TEXT",
        "KEY",
        "TASK",
        "PROJECT",
        "FORGE",
        "INSTALLATION",
        "ACCOUNT",
        "REVISION",
        "PROPOSAL",
        "PROVIDER",
    ];
    if entry.kind == Kind::Walk {
        for (heading, _, section) in sections(entry) {
            if heading == "Before you start" {
                for line in section.lines() {
                    if let Some(after) = line.strip_prefix("- `")
                        && let Some((word, _)) = after.split_once('`')
                        && !word.is_empty()
                        && word.starts_with(|c: char| c.is_ascii_uppercase())
                        && word.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                    {
                        allowed.push(word);
                    }
                }
            }
        }
    }
    let mut fenced = false;
    for (at, line) in entry.text.lines().enumerate() {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if !fenced || line.split_ascii_whitespace().next() != Some("temper") {
            continue;
        }
        for word in line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if word.starts_with(|c: char| c.is_ascii_uppercase())
                && word.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                && !allowed.contains(&word)
            {
                return Err(refused(
                    entry,
                    &format!("placeholder {word} is in the set or declared by the walk"),
                    at + 1,
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn voice(entry: &Page) -> Result<(), String> {
    if !written(entry) {
        return Ok(());
    }
    let mut fenced = false;
    for (at, line) in entry.text.lines().enumerate() {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        for prose in line.split('`').step_by(2) {
            if prose.contains(".md")
                || prose.contains("temper-")
                || prose.split(|c: char| !c.is_ascii_alphabetic()).any(|word| ["jig", "smith", "skein"].contains(&word))
            {
                return Err(refused(entry, "voice keeps design words in code", at + 1));
            }
        }
    }
    Ok(())
}
