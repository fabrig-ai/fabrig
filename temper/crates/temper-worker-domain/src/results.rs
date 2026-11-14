//! The check's bounded text contract (domain/forge/reds.md, 5.5 and 5.6).
use crate::wire;
use alloc::boxed::Box;
use skein_lib::{Intake, List, Set, Writer, bytes, stream};

pub(crate) fn parse(tag: u64, check: u64, bytes: &[u8]) -> Result<wire::CheckResults, &'static str> {
    if bytes.len() > 262_144 {
        return Err("oversized results file");
    }
    if !bytes.ends_with(b"\n") {
        return Err("results line without line feed");
    }
    let (header, rest) = part(bytes, b'\n').ok_or("missing results header")?;
    if header != b"temper-check 1" {
        return Err("wrong results header");
    }
    let (line, mut rest) = part(rest, b'\n').ok_or("missing check line")?;
    let Some(line) = line.strip_prefix(b"check ") else { return Err("wrong check line") };
    let (tree, tail) = part(line, b' ').ok_or("wrong check line")?;
    let tree = object(tree)?;
    let (base, verdict) = part(tail, b' ').ok_or("wrong check line")?;
    let base = if base == b"-" { None } else { Some(object(base)?) };
    let verdict = match verdict {
        b"passed" => wire::Verdict::Passed,
        b"refused" => wire::Verdict::Refused,
        _ => return Err("wrong check verdict"),
    };
    let mut passed: List<Box<[u8]>> = List::with_capacity(256);
    let mut failed: List<Box<[u8]>> = List::with_capacity(256);
    let mut flakes: List<wire::Flake> = List::with_capacity(16);
    let mut keys: Set<Box<[u8]>> = Set::with_capacity(512);
    let mut flake_keys: Set<Box<[u8]>> = Set::with_capacity(16);
    let mut log = Intake::with_capacity(1536);
    let mut log_lines = false;
    for _ in 0..rest.len() {
        if rest.is_empty() {
            break;
        }
        let (line, tail) = part(rest, b'\n').expect("line-feed terminated file");
        rest = tail;
        if line == b"|" || line.starts_with(b"| ") {
            if flakes.is_empty() {
                return Err("log before flake");
            }
            let text = if line == b"|" { b"".as_slice() } else { line.get(2..).expect("log prefix") };
            if log_lines {
                log.append(b"\n").or(Err("flake log too long"))?;
            }
            log.append(text).or(Err("flake log too long"))?;
            log_lines = true;
        } else if let Some(key) = line.strip_prefix(b"flake ") {
            valid_key(key)?;
            if !flake_keys.insert(Box::from(key)).or(Err("too many flakes"))? {
                return Err("duplicate flake");
            }
            finish_log(&mut flakes, &mut log);
            log_lines = false;
            flakes.push(wire::Flake { key: Box::from(key), log: Box::new([]) }).or(Err("too many flakes"))?;
        } else {
            if !flakes.is_empty() {
                return Err("key after flakes");
            }
            let list = if let Some(key) = line.strip_prefix(b"passed ") {
                (key, &mut passed)
            } else if let Some(key) = line.strip_prefix(b"failed ") {
                (key, &mut failed)
            } else {
                return Err("unknown results word");
            };
            let (key, list) = list;
            valid_key(key)?;
            if !keys.insert(Box::from(key)).or(Err("too many keys"))? {
                return Err("duplicate results key");
            }
            if let Some(last) = list.last()
                && last.as_ref() >= key
            {
                return Err("results keys out of order");
            }
            list.push(Box::from(key)).or(Err("too many results keys"))?;
        }
    }
    finish_log(&mut flakes, &mut log);
    Ok(wire::CheckResults {
        tag,
        check,
        tree,
        base,
        verdict,
        passed: passed.into_boxed(),
        failed: failed.into_boxed(),
        flakes: flakes.into_boxed(),
    })
}
fn finish_log(flakes: &mut List<wire::Flake>, log: &mut Intake) {
    if let Some(last) = flakes.len().checked_sub(1) {
        let flake = flakes.get_mut(last).expect("last flake");
        let size = log.len();
        flake.log = log.meet(stream::Read::Fill(size)).expect("buffered log");
    }
}
fn valid_key(key: &[u8]) -> Result<(), &'static str> {
    if key.len() > 512 {
        return Err("results key too long");
    }
    for byte in key {
        if byte.is_ascii_control() {
            return Err("control in results key");
        }
    }
    let (mut tail, separators) = if let Some(tail) = key.strip_prefix(b"test ") {
        (tail, 1_u32)
    } else if let Some(tail) = key.strip_prefix(b"lint ") {
        (tail, 2_u32)
    } else {
        return Err("wrong results key");
    };
    for _ in 0..separators {
        let (word, rest) = part(tail, b' ').ok_or("wrong results key parts")?;
        if word.is_empty() {
            return Err("empty results key part");
        }
        tail = rest;
    }
    if tail.is_empty() || tail.starts_with(b" ") || tail.ends_with(b" ") {
        return Err("empty results key part");
    }
    Ok(())
}
fn object(hex: &[u8]) -> Result<Box<[u8]>, &'static str> {
    if hex.len() != 40 && hex.len() != 64 {
        return Err("wrong check object width");
    }
    let mut out = Writer::new(hex.len().checked_div(2).expect("hex width"));
    for pair in hex.as_chunks::<2>().0 {
        let hi = digit(*pair.first().expect("hex pair"))?;
        let lo = digit(*pair.last().expect("hex pair"))?;
        out.put(&[hi.checked_mul(16).expect("hex nibble") | lo]).expect("measured object");
    }
    Ok(out.finish())
}
fn digit(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte.checked_sub(b'0').expect("digit")),
        b'a'..=b'f' => Ok(byte.checked_sub(b'a').expect("hex letter").checked_add(10).expect("nibble")),
        _ => Err("wrong check object digit"),
    }
}

fn part(bytes: &[u8], delimiter: u8) -> Option<(&[u8], &[u8])> {
    let at = bytes::find(bytes, &[delimiter])?;
    let (word, rest) = bytes.split_at_checked(at)?;
    Some((word, rest.get(1..)?))
}
