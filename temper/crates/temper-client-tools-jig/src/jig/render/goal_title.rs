//! A long goal title warns beside a successful answer (client/ctl/areas.md), area 2.
use super::Text;
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits};

fn first_line_characters(text: &[u8]) -> u64 {
    let mut count = 0_u64;
    // Loaded TEXT is UTF-8: every non-continuation byte starts one scalar value.
    for byte in text {
        if *byte == b'\n' {
            break;
        }
        if byte & 0xc0 != 0x80 {
            count = count.checked_add(1).expect("bounded TEXT character count fits");
        }
    }
    count
}

pub(crate) fn warn(text: &[u8], json: bool, stderr: &mut Box<[u8]>) {
    let count = first_line_characters(text);
    if count <= 60 {
        return;
    }
    let mut measure = Text::measure();
    words(count, &mut measure);
    let mut out = Text::writing_exact(measure);
    words(count, &mut out);
    let words = out.finish();
    let notice = if json {
        // One fixed sentence with a u64 count in one string in one object.
        let bounds = Limits { depth: 2, length: 256 };
        let mut measure = Encoder::measure(&bounds);
        warning(&words, &mut measure);
        let size = measure.measured().expect("goal title warning fits");
        let mut out = Encoder::write(size, &bounds);
        warning(&words, &mut out);
        out.finish()
    } else {
        words
    };
    let mut measure = Text::measure();
    joined(stderr, &notice, json, &mut measure);
    let mut out = Text::writing_exact(measure);
    joined(stderr, &notice, json, &mut out);
    *stderr = out.finish();
}

fn words(count: u64, out: &mut Text) {
    out.put(b"TEXT's first line, the title the pages show, is ");
    out.number(count);
    out.put(b" characters, past 60; temper ctl help goal start");
}

fn warning(words: &[u8], out: &mut Encoder) {
    out.object_start();
    out.key(b"warning");
    out.string(words);
    out.object_end();
}

fn joined(stderr: &[u8], notice: &[u8], json: bool, out: &mut Text) {
    out.put(stderr);
    if !json {
        out.put(b"warning: ");
    }
    out.put(notice);
    out.put(b"\n");
}
