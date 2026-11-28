//! Source start tags, never a second DOM (temper/docs/design/client/journey-writer-requests.md, 1.2).

pub(super) struct Tag<'a> {
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) name: &'a str,
    pub(super) classes: &'a str,
    open_at: Option<usize>,
}

pub(super) fn tags(source: &str) -> Vec<Tag<'_>> {
    scan(source, None)
}

pub(super) fn comparable(source: &str) -> String {
    let mut lines = Comparable { source, output: String::with_capacity(source.len()), at: 0 };
    scan(source, Some(&mut lines));
    lines.copy_to(source.len());
    temper_client_walk_lines::print::numbered_ids(&lines.output)
}

fn scan<'a>(source: &'a str, mut comparable: Option<&mut Comparable<'a>>) -> Vec<Tag<'a>> {
    let mut tags = Vec::new();
    let mut cursor = source;
    let mut position = Position { line: 1, column: 0 };
    let mut raw: Option<&str> = None;
    while let Some((before, after)) = cursor.split_once('<') {
        position.advance(before);
        let tail = if let Some(name) = raw {
            if let Some(end) = closing(after, name) {
                raw = None;
                after_tag(end, &mut comparable)
            } else {
                after
            }
        } else if let Some(end) = declaration(after) {
            end
        } else if let Some(end) = after.strip_prefix('/') {
            after_tag(Attributes::new(name_and_tail(end).1).end(), &mut comparable)
        } else if after.as_bytes().first().is_some_and(u8::is_ascii_alphabetic) {
            let (name, rest) = name_and_tail(after);
            let mut attributes = Attributes::new(rest);
            let mut has_open = false;
            let mut classes = None;
            loop {
                let before = attributes.tail;
                let Some((attribute, value)) = attributes.take() else { break };
                if let Some(lines) = comparable.as_deref_mut() {
                    if attribute.eq_ignore_ascii_case("data-node") {
                        lines.remove_attribute(before, attributes.tail);
                    }
                } else {
                    has_open |= attribute.eq_ignore_ascii_case("open");
                    if attribute.eq_ignore_ascii_case("class") && classes.is_none() {
                        classes = Some(value);
                    }
                }
            }
            if !attributes.tail.starts_with('>') {
                break; // No start tag was emitted before EOF.
            }
            if comparable.is_none() {
                let disclosure = name.eq_ignore_ascii_case("details") || name.eq_ignore_ascii_case("dialog");
                let open_at = (disclosure && !has_open).then_some(source.len() - rest.len());
                let Position { line, column } = position;
                tags.push(Tag { line, column, name, classes: classes.unwrap_or_default(), open_at });
            }
            let end = after_tag(attributes.tail, &mut comparable);
            if name.eq_ignore_ascii_case("plaintext") {
                break;
            }
            if ["script", "style", "title", "textarea", "iframe", "xmp", "noembed", "noframes", "noscript"]
                .iter()
                .any(|tag| name.eq_ignore_ascii_case(tag))
            {
                raw = Some(name);
            }
            end
        } else {
            after
        };
        position.column += 1; // The opening '<'.
        position.advance(&after[..after.len() - tail.len()]);
        cursor = tail;
    }
    tags
}

// Keep the doctype, text, comments and quoted values as written. Only the
// boundaries of tags and the printer's unstable node numbers are changed.
struct Comparable<'a> {
    source: &'a str,
    output: String,
    at: usize,
}

impl Comparable<'_> {
    fn copy_to(&mut self, until: usize) {
        self.output.push_str(&self.source[self.at..until]);
        self.at = until;
    }

    fn remove_attribute(&mut self, before: &str, after: &str) {
        let start = before.len() - before.trim_start_matches(|c| space(c) || c == '/').len();
        let leading = before[..start].trim_end_matches(space).len();
        self.copy_to(self.source.len() - before.len() + leading);
        self.at = self.source.len() - after.len();
    }
}

fn after_tag<'a>(end: &'a str, comparable: &mut Option<&mut Comparable<'a>>) -> &'a str {
    let Some(tail) = end.strip_prefix('>') else { return end };
    if let Some(lines) = comparable {
        lines.copy_to(lines.source.len() - end.len());
        if !lines.output.ends_with('\n') {
            lines.output.push('\n');
        }
    }
    tail
}

pub(super) fn opened(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut at = 0;
    for tag in tags(source) {
        if let Some(end) = tag.open_at {
            output.push_str(&source[at..end]);
            output.push_str(" open");
            at = end;
        }
    }
    output.push_str(&source[at..]);
    output
}

fn closing<'a>(after: &'a str, raw: &str) -> Option<&'a str> {
    let (name, tail) = name_and_tail(after.strip_prefix('/')?);
    name.eq_ignore_ascii_case(raw).then(|| Attributes::new(tail).end())
}

fn declaration(after: &str) -> Option<&str> {
    if let Some(comment) = after.strip_prefix("!--") {
        return Some(if let Some(tail) = comment.strip_prefix('>').or_else(|| comment.strip_prefix("->")) {
            tail
        } else {
            ["-->", "--!>"]
                .into_iter()
                .filter_map(|stop| comment.split_once(stop).map(|(_, tail)| tail))
                .max_by_key(|tail| tail.len())
                .unwrap_or_default()
        });
    }
    if let Some(text) = after.strip_prefix("![CDATA[") {
        return Some(text.split_once("]]>").map_or("", |(_, tail)| tail));
    }
    if after.get(..8).is_some_and(|head| head.eq_ignore_ascii_case("!doctype")) {
        return Some(doctype(&after[8..]));
    }
    after.starts_with(['!', '?']).then(|| after.split_once('>').map_or("", |(_, tail)| tail))
}

fn doctype(source: &str) -> &str {
    let mut quote = None;
    for (at, byte) in source.bytes().enumerate() {
        if let Some(end) = quote {
            if byte == end {
                quote = None;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        } else if byte == b'>' {
            return &source[at + 1..];
        }
    }
    ""
}

struct Position {
    line: usize,
    column: usize,
}

impl Position {
    fn advance(&mut self, text: &str) {
        if let Some((before, tail)) = text.rsplit_once('\n') {
            self.line += before.bytes().filter(|&byte| byte == b'\n').count() + 1;
            self.column = tail.len();
        } else {
            self.column += text.len();
        }
    }
}

fn space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c')
}

fn name_and_tail(source: &str) -> (&str, &str) {
    source.split_at(source.find(|c| space(c) || matches!(c, '/' | '>')).unwrap_or(source.len()))
}

struct Attributes<'a> {
    tail: &'a str,
}

impl<'a> Attributes<'a> {
    fn new(tail: &'a str) -> Self {
        Self { tail }
    }

    fn take(&mut self) -> Option<(&'a str, &'a str)> {
        let tail = self.tail.trim_start_matches(|c| space(c) || c == '/');
        self.tail = tail;
        if tail.is_empty() || tail.starts_with('>') {
            return None;
        }
        // A stray leading '=' is part of the name in HTML's attribute tokenizer.
        let from = usize::from(tail.starts_with('='));
        let end = tail[from..].find(|c| space(c) || matches!(c, '/' | '>' | '=')).map_or(tail.len(), |at| at + from);
        let (name, tail) = tail.split_at(end);
        let Some(value) = tail.trim_start_matches(space).strip_prefix('=') else {
            self.tail = tail;
            return Some((name, ""));
        };
        let value = value.trim_start_matches(space);
        let (value, tail) = if let Some(quoted) = value.strip_prefix(['\'', '"']) {
            let delimiter = value.as_bytes()[0];
            quoted.split_once(char::from(delimiter)).unwrap_or((quoted, ""))
        } else {
            value.split_at(value.find(|c| space(c) || c == '>').unwrap_or(value.len()))
        };
        self.tail = tail;
        Some((name, value))
    }

    fn end(mut self) -> &'a str {
        while self.take().is_some() {}
        self.tail
    }
}
