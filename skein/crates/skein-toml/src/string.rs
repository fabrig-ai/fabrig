//! Basic and literal strings, including quoted keys (skein/docs/design/data/toml.md, 2).

use skein_lib::{List, bytes};

use crate::Refusal;
use crate::cursor::Cursor;
use crate::document::Text;

pub(crate) fn read(cursor: &mut Cursor, input: &[u8], text: &mut List<u8>, key: bool) -> Result<Text, Refusal> {
    let start = text.len();
    let quote = cursor.take(input)?;
    let multiline = !key && cursor.peek(input) == Some(quote) && cursor.ahead(input, 1) == Some(quote);
    if multiline {
        cursor.need(input, quote)?;
        cursor.need(input, quote)?;
        if at_newline(cursor, input) {
            cursor.newline(input)?;
        }
    }
    for _ in cursor.at..input.len() {
        match cursor.peek(input) {
            Some(byte) if byte == quote => {
                if quotes(cursor, input, text, quote, multiline)? {
                    return Ok(Text::since(start, text));
                }
            }
            Some(b'\\') if quote == b'"' => escape(cursor, input, text, multiline)?,
            Some(b'\n' | b'\r') if multiline => {
                cursor.newline(input)?;
                put(text, b'\n');
            }
            None | Some(b'\x00'..=b'\x08' | b'\x0a'..=b'\x1f' | b'\x7f') => return Err(cursor.grammar(input)),
            Some(byte) => {
                cursor.take(input)?;
                put(text, byte);
            }
        }
    }
    Err(cursor.grammar(input))
}

// One/two quotes can appear within a multiline string, including immediately
// before the closing triple. A sixth consecutive quote is outside the value.
fn quotes(cursor: &mut Cursor, input: &[u8], text: &mut List<u8>, quote: u8, multiline: bool) -> Result<bool, Refusal> {
    cursor.need(input, quote)?;
    if !multiline {
        return Ok(true);
    }
    if !cursor.eat(input, quote) {
        put(text, quote);
        return Ok(false);
    }
    if !cursor.eat(input, quote) {
        put(text, quote);
        put(text, quote);
        return Ok(false);
    }
    for _ in 0..2_u8 {
        if !cursor.eat(input, quote) {
            break;
        }
        put(text, quote);
    }
    Ok(true)
}

fn escape(cursor: &mut Cursor, input: &[u8], text: &mut List<u8>, multiline: bool) -> Result<(), Refusal> {
    cursor.need(input, b'\\')?;
    let escaped = match cursor.peek(input) {
        Some(b'b') => b'\x08',
        Some(b't') => b'\t',
        Some(b'n') => b'\n',
        Some(b'f') => b'\x0c',
        Some(b'r') => b'\r',
        Some(b'"') => b'"',
        Some(b'\\') => b'\\',
        Some(b'u') => return unicode(cursor, input, text, 4),
        Some(b'U') => return unicode(cursor, input, text, 8),
        Some(b' ' | b'\t' | b'\n' | b'\r') if multiline => return continuation(cursor, input),
        Some(_) | None => return Err(cursor.grammar(input)),
    };
    cursor.take(input)?;
    put(text, escaped);
    Ok(())
}

fn unicode(cursor: &mut Cursor, input: &[u8], text: &mut List<u8>, digits: u32) -> Result<(), Refusal> {
    cursor.take(input)?;
    let mut number = 0_u32;
    for _ in 0..digits {
        let Some(byte) = cursor.peek(input) else { return Err(cursor.grammar(input)) };
        let Some(digit) = bytes::hex_value(byte) else { return Err(cursor.grammar(input)) };
        cursor.take(input)?;
        number = (number << 4_u32) | u32::from(digit);
    }
    let Some(character) = char::from_u32(number) else { return Err(cursor.grammar(input)) };
    let mut utf8 = [0; 4];
    for &byte in character.encode_utf8(&mut utf8).as_bytes() {
        put(text, byte);
    }
    Ok(())
}

fn at_newline(cursor: &Cursor, input: &[u8]) -> bool {
    cursor.peek(input) == Some(b'\n') || cursor.peek(input) == Some(b'\r')
}

fn continuation(cursor: &mut Cursor, input: &[u8]) -> Result<(), Refusal> {
    cursor.horizontal(input);
    cursor.newline(input)?;
    for _ in cursor.at..input.len() {
        match cursor.peek(input) {
            Some(b' ' | b'\t') => cursor.horizontal(input),
            Some(b'\r' | b'\n') => cursor.newline(input)?,
            Some(_) | None => break,
        }
    }
    Ok(())
}

fn put(text: &mut List<u8>, byte: u8) {
    text.push(byte).expect("string decoding never expands the admitted input");
}
