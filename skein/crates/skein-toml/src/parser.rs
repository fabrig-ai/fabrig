//! Whole-document grammar and table definitions (skein/docs/design/data/toml.md, 2, 3 and 6).

use skein_lib::{Stack, usize_of};

use crate::cursor::Cursor;
use crate::document::{Build, Stored, TableState, Text};
use crate::{Document, Limit, Limits, Refusal, Why, atom, string};

/// Reads one whole TOML 1.0 document under `limits`, or returns its first fault.
/// No input is retained by reference and no partial document is returned.
pub fn parse(input: &[u8], limits: &Limits) -> Result<Document, Refusal> {
    if input.len() > usize_of(limits.input_bytes) {
        return Err(Refusal { line: 1, why: Why::Limit(Limit::InputBytes) });
    }
    let (cursor, input) = Cursor::new(input);
    let mut parser = Parser { cursor, build: Build::new(limits), frames: Stack::with_capacity(limits.depth) };
    parser.read(input)?;
    Ok(parser.build.finish())
}

#[derive(Clone, Copy)]
enum Path {
    Key,
    Header,
    ArrayHeader,
}

impl Path {
    const fn redefined(self) -> Why {
        match self {
            Self::Key => Why::KeyRedefined,
            Self::Header | Self::ArrayHeader => Why::TableRedefined,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    ArrayItem,
    ArrayTail,
    InlineFirst,
    InlineEntry,
    InlineTail,
}

#[derive(Clone, Copy, Debug)]
struct Frame {
    node: u32,
    phase: Phase,
}

struct Parser {
    cursor: Cursor,
    build: Build,
    frames: Stack<Frame>,
}

impl Parser {
    fn read(&mut self, input: &[u8]) -> Result<(), Refusal> {
        let mut table = 0;
        // A statement reads at least one byte; one extra step checks the end.
        for _ in 0..=input.len() {
            self.cursor.trivia(input)?;
            if self.cursor.at == input.len() {
                return self.cursor.end(input);
            }
            if self.cursor.eat(input, b'[') {
                let path = if self.cursor.eat(input, b'[') { Path::ArrayHeader } else { Path::Header };
                table = self.path(input, 0, path)?;
            } else {
                let node = self.path(input, table, Path::Key)?;
                self.start_value(input, node)?;
                self.containers(input)?;
            }
            self.cursor.statement_end(input)?;
        }
        self.cursor.end(input)
    }

    fn path(&mut self, input: &[u8], mut parent: u32, path: Path) -> Result<u32, Refusal> {
        let mut depth = self.frames.len();
        for _ in self.cursor.at..=input.len() {
            self.cursor.horizontal(input);
            let line = self.cursor.line;
            let start = self.build.text.len();
            let Some(first) = self.cursor.peek(input) else { return Err(self.cursor.grammar(input)) };
            if !bare(first) && first != b'\'' && first != b'"' {
                return Err(self.cursor.grammar(input));
            }
            if depth == self.frames.capacity() {
                return Err(self.cursor.refuse(Why::Limit(Limit::Depth)));
            }
            depth = depth.saturating_add(1);
            let key = self.segment(input, start)?;
            self.cursor.horizontal(input);
            let present = self.build.find(parent, key);
            if present.is_some() {
                // A later spelling of a segment already in the tree is scratch.
                self.build.text.truncate(start);
            }
            if self.cursor.eat(input, b'.') {
                parent = self.prefix(parent, path, key, line, present)?;
            } else {
                return self.end_path(input, parent, path, key, line, present);
            }
        }
        Err(self.cursor.grammar(input))
    }

    fn segment(&mut self, input: &[u8], start: u32) -> Result<Text, Refusal> {
        if let Some(b'\'' | b'"') = self.cursor.peek(input) {
            return string::read(&mut self.cursor, input, &mut self.build.text, true);
        }
        for _ in self.cursor.at..input.len() {
            let Some(byte) = self.cursor.peek(input) else { break };
            if !bare(byte) {
                break;
            }
            self.cursor.take(input)?;
            self.build.text.push(byte).expect("bare keys occupy at most their admitted input bytes");
        }
        Ok(Text::since(start, &self.build.text))
    }

    fn prefix(&mut self, parent: u32, path: Path, key: Text, line: u64, present: Option<u32>) -> Result<u32, Refusal> {
        let Some(node) = present else {
            let table = match path {
                Path::Key => TableState::Dotted,
                Path::Header | Path::ArrayHeader => TableState::Implicit,
            };
            return self.append(parent, key, line, Stored::Table(table));
        };
        let kind = self.build.kind(node);
        match path {
            Path::Key => match kind.table() {
                Some(TableState::Dotted) => Ok(node),
                Some(TableState::Implicit) => {
                    self.build.set_kind(node, Stored::Table(TableState::Dotted));
                    Ok(node)
                }
                Some(TableState::Root | TableState::Header | TableState::Inline) | None => {
                    Err(Refusal { line, why: path.redefined() })
                }
            },
            Path::Header | Path::ArrayHeader => match kind {
                Stored::Table(TableState::Implicit | TableState::Header | TableState::Dotted) => Ok(node),
                Stored::ArrayOfTables => Ok(self.build.last(node)),
                Stored::Table(TableState::Root | TableState::Inline)
                | Stored::Pending
                | Stored::String(_)
                | Stored::Integer { .. }
                | Stored::Float(_)
                | Stored::Boolean(_)
                | Stored::DateTime { .. }
                | Stored::Array => Err(Refusal { line, why: path.redefined() }),
            },
        }
    }

    fn end_path(
        &mut self,
        input: &[u8],
        parent: u32,
        path: Path,
        key: Text,
        line: u64,
        present: Option<u32>,
    ) -> Result<u32, Refusal> {
        let redefined = Refusal { line, why: path.redefined() };
        match path {
            Path::Key => {
                self.cursor.need(input, b'=')?;
                if present.is_some() {
                    return Err(redefined);
                }
                self.append(parent, key, line, Stored::Pending)
            }
            Path::Header => {
                self.cursor.need(input, b']')?;
                let Some(node) = present else {
                    return self.append(parent, key, line, Stored::Table(TableState::Header));
                };
                if self.build.kind(node).table() != Some(TableState::Implicit) {
                    return Err(redefined);
                }
                self.build.set_kind(node, Stored::Table(TableState::Header));
                self.build.set_line(node, line);
                Ok(node)
            }
            Path::ArrayHeader => {
                self.cursor.need(input, b']')?;
                self.cursor.need(input, b']')?;
                let array = if let Some(node) = present {
                    match self.build.kind(node) {
                        Stored::ArrayOfTables => node,
                        Stored::Table(_)
                        | Stored::Pending
                        | Stored::String(_)
                        | Stored::Integer { .. }
                        | Stored::Float(_)
                        | Stored::Boolean(_)
                        | Stored::DateTime { .. }
                        | Stored::Array => return Err(redefined),
                    }
                } else {
                    self.append(parent, key, line, Stored::ArrayOfTables)?
                };
                self.append(array, Text::EMPTY, line, Stored::Table(TableState::Header))
            }
        }
    }

    fn append(&mut self, parent: u32, key: Text, line: u64, kind: Stored) -> Result<u32, Refusal> {
        self.build.append(parent, key, line, kind).ok_or(Refusal { line, why: Why::Limit(Limit::Entries) })
    }

    fn start_value(&mut self, input: &[u8], node: u32) -> Result<(), Refusal> {
        self.cursor.horizontal(input);
        self.build.set_line(node, self.cursor.line);
        let kind = match self.cursor.peek(input) {
            Some(b'[') => {
                self.open(input, node, b'[', Phase::ArrayItem)?;
                Stored::Array
            }
            Some(b'{') => {
                self.open(input, node, b'{', Phase::InlineFirst)?;
                Stored::Table(TableState::Inline)
            }
            Some(b'\'' | b'"') => Stored::String(string::read(&mut self.cursor, input, &mut self.build.text, false)?),
            Some(_) | None => atom::read(&mut self.cursor, input, &mut self.build.text)?,
        };
        self.build.set_kind(node, kind);
        Ok(())
    }

    fn open(&mut self, input: &[u8], node: u32, byte: u8, phase: Phase) -> Result<(), Refusal> {
        let refusal = self.cursor.refuse(Why::Limit(Limit::Depth));
        self.frames.push(Frame { node, phase }).or(Err(refusal))?;
        self.cursor.need(input, byte)
    }

    fn close(&mut self) {
        self.frames.pop().expect("closing the current frame");
    }

    fn phase(&mut self, phase: Phase) {
        self.frames.top_mut().expect("changing the current frame").phase = phase;
    }

    // Every iteration with a frame either consumes at least one byte or
    // refuses. A value that opens a container pushes rather than recursing.
    fn containers(&mut self, input: &[u8]) -> Result<(), Refusal> {
        for _ in self.cursor.at..=input.len() {
            let Some(&frame) = self.frames.top() else { return Ok(()) };
            match frame.phase {
                Phase::ArrayItem => {
                    self.cursor.trivia(input)?;
                    if self.cursor.eat(input, b']') {
                        self.close();
                    } else {
                        match self.cursor.peek(input) {
                            Some(
                                b'[' | b'{' | b'\'' | b'"' | b't' | b'f' | b'i' | b'n' | b'+' | b'-' | b'0'..=b'9',
                            ) => {}
                            Some(_) | None => return Err(self.cursor.grammar(input)),
                        }
                        let node = self.append(frame.node, Text::EMPTY, self.cursor.line, Stored::Pending)?;
                        self.phase(Phase::ArrayTail);
                        self.start_value(input, node)?;
                    }
                }
                Phase::ArrayTail => {
                    self.cursor.trivia(input)?;
                    if self.cursor.eat(input, b']') {
                        self.close();
                    } else {
                        self.cursor.need(input, b',')?;
                        self.phase(Phase::ArrayItem);
                    }
                }
                Phase::InlineFirst | Phase::InlineEntry => {
                    self.cursor.horizontal(input);
                    if is_inline_first(frame.phase) && self.cursor.eat(input, b'}') {
                        self.close();
                    } else {
                        let node = self.path(input, frame.node, Path::Key)?;
                        self.phase(Phase::InlineTail);
                        self.start_value(input, node)?;
                    }
                }
                Phase::InlineTail => {
                    self.cursor.horizontal(input);
                    if self.cursor.eat(input, b'}') {
                        self.close();
                    } else {
                        self.cursor.need(input, b',')?;
                        self.phase(Phase::InlineEntry);
                    }
                }
            }
        }
        Err(self.cursor.grammar(input))
    }
}

const fn is_inline_first(phase: Phase) -> bool {
    match phase {
        Phase::InlineFirst => true,
        Phase::ArrayItem | Phase::ArrayTail | Phase::InlineEntry | Phase::InlineTail => false,
    }
}

const fn bare(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}
