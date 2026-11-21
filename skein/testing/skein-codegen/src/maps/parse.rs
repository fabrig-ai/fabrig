//! Line-aware maps grammar (data/codec/maps.md, 2).

use super::{Item, Mapping, Maps, Mode, Shape, error};
use crate::{Error, Schema};

#[derive(Clone)]
struct Token {
    text: String,
    line: usize,
}

struct Parser<'a> {
    schema: &'a Schema,
    tokens: Vec<Token>,
    at: usize,
}

impl Parser<'_> {
    fn peek(&self) -> &str {
        self.tokens.get(self.at).map_or("", |token| token.text.as_str())
    }

    fn line(&self) -> usize {
        self.tokens.get(self.at).or_else(|| self.tokens.last()).map_or(1, |token| token.line)
    }

    fn take(&mut self) -> Result<Token, Error> {
        let token = self.tokens.get(self.at).cloned().ok_or_else(|| self.fail("unexpected end of maps"))?;
        self.at = self.at.checked_add(1).expect("maps token count fits usize");
        Ok(token)
    }

    fn fail(&self, message: &str) -> Error {
        error(self.schema, self.line(), message)
    }

    fn eat(&mut self, text: &str) -> bool {
        if self.peek() == text {
            self.at = self.at.checked_add(1).expect("maps token count fits usize");
            true
        } else {
            false
        }
    }

    fn expect(&mut self, text: &str) -> Result<(), Error> {
        if self.eat(text) { Ok(()) } else { Err(self.fail(&format!("expected {text}, found {}", self.peek()))) }
    }

    fn word(&mut self) -> Result<String, Error> {
        if matches!(self.peek(), "" | "\n" | "=" | "{" | "}" | ",") {
            return Err(self.fail("expected a name or path"));
        }
        Ok(self.take()?.text)
    }

    fn separators(&mut self) {
        while self.eat("\n") || self.eat(",") {}
    }

    fn end_line(&mut self) -> Result<(), Error> {
        if !matches!(self.peek(), "" | "\n") {
            return Err(self.fail("expected end of line"));
        }
        self.eat("\n");
        Ok(())
    }

    fn block(&mut self, enumeration: bool) -> Result<Vec<Item>, Error> {
        let mut items = Vec::new();
        self.expect("{")?;
        self.separators();
        while !self.eat("}") {
            let line = self.line();
            let name = self.word()?;
            let rename = if self.eat("=") { Some(self.word()?) } else { None };
            let via = if !enumeration && self.eat("via") { Some(self.word()?) } else { None };
            let tuple = enumeration && self.eat("tuple");
            let shape = if !enumeration && self.eat("slice") {
                Shape::Slice
            } else if self.eat("boxed") {
                Shape::Boxed
            } else {
                Shape::Same
            };
            let block = if enumeration && self.peek() == "{" { Some(self.block(false)?) } else { None };
            items.push(Item { line, name, rename, via, tuple, shape, block });
            if !matches!(self.peek(), "}" | "\n" | ",") {
                return Err(self.fail("expected comma, newline or block end"));
            }
            self.separators();
        }
        Ok(items)
    }

    fn mapping(&mut self) -> Result<Mapping, Error> {
        let start = self.take()?;
        let enumeration = start.text == "enum";
        let name = self.word()?;
        let mode = if self.eat("=") {
            Mode::Mirror(self.word()?)
        } else if self.eat("stands") && !enumeration {
            self.expect("for")?;
            let field = self.word()?;
            let via = if self.eat("via") { Some(self.word()?) } else { None };
            Mode::Stands { field, via }
        } else if self.eat("by") {
            self.expect("hand")?;
            Mode::Hand
        } else if !enumeration {
            Mode::Payload
        } else {
            return Err(self.fail("expected enum mapping or by hand"));
        };
        // The block may start on the declaration's next nonblank line.
        while self.eat("\n") {}
        let items = if self.peek() == "{" {
            if !matches!(mode, Mode::Mirror(_) | Mode::Payload) {
                return Err(self.fail("only a mirrored declaration or payload has a block"));
            }
            let items = self.block(enumeration)?;
            self.end_line()?;
            items
        } else {
            if !matches!(self.peek(), "" | "record" | "enum") {
                return Err(self.fail("expected block or next declaration"));
            }
            Vec::new()
        };
        Ok(Mapping { line: start.line, name, enumeration, mode, items })
    }
}

fn tokens(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let line_number = index.checked_add(1).expect("maps line count fits usize");
        let text = line.split('#').next().expect("line before comment");
        let mut word = String::new();
        for character in text.chars() {
            if character.is_whitespace() || "{}=,".contains(character) {
                if !word.is_empty() {
                    tokens.push(Token { text: core::mem::take(&mut word), line: line_number });
                }
                if !character.is_whitespace() {
                    tokens.push(Token { text: character.to_string(), line: line_number });
                }
            } else {
                word.push(character);
            }
        }
        if !word.is_empty() {
            tokens.push(Token { text: word, line: line_number });
        }
        tokens.push(Token { text: "\n".into(), line: line_number });
    }
    tokens
}

pub(super) fn parse(schema: &Schema, source: &str) -> Result<Maps, Error> {
    let mut parser = Parser { schema, tokens: tokens(source), at: 0 };
    let mut maps = Maps::default();
    let mut context_seen = false;
    parser.separators();
    while !matches!(parser.peek(), "" | "record" | "enum") {
        let token = parser.take()?;
        match token.text.as_str() {
            "records" if maps.records.is_empty() => maps.records = parser.word()?,
            "use" => {
                let alias = parser.word()?;
                parser.expect("=")?;
                let path = maps.path(&parser.word()?);
                if maps.aliases.insert(alias, path).is_some() {
                    return Err(error(schema, token.line, "duplicate alias"));
                }
            }
            "by" if maps.hand.is_none() => {
                parser.expect("hand")?;
                maps.hand = Some(parser.word()?);
            }
            "context" if !context_seen => {
                let mut words = Vec::new();
                while !matches!(parser.peek(), "" | "\n") {
                    words.push(parser.take()?.text);
                }
                if words.is_empty() {
                    return Err(error(schema, token.line, "expected context type"));
                }
                maps.context = words.join(" ");
                context_seen = true;
            }
            _ => return Err(error(schema, token.line, "unknown or repeated header")),
        }
        parser.end_line()?;
        parser.separators();
    }
    if maps.records.is_empty() {
        return Err(error(schema, parser.line(), "header requires records"));
    }
    if !context_seen {
        maps.context = "()".into();
    }
    while parser.peek() != "" {
        if !matches!(parser.peek(), "record" | "enum") {
            return Err(parser.fail("expected record or enum"));
        }
        maps.declarations.push(parser.mapping()?);
        parser.separators();
    }
    Ok(maps)
}
