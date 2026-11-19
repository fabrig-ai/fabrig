//! Completed tier reports within the task contract (temper/docs/design/domain/after-landing.md, 10).
use super::{Box, List};

/// Fit a forge-composed tier verdict to the task's report bound.
/// Keep whole landing attributions first, then report names, before the explicit remainder.
#[must_use]
pub fn tier_report_words(words: &[u8], bound: u32) -> Box<[u8]> {
    let room = skein_lib::usize_of(bound);
    if words.len() <= room {
        return Box::from(words);
    }
    let parsed = report_lines(words);
    let mut attributions = List::uncapped();
    let mut names = None;
    for line in parsed.as_slice() {
        if line.starts_with(b"Its report names: ") {
            names = Some(*line);
        } else {
            attributions.push(*line).expect("attribution line");
        }
    }
    let lines = attributions.as_slice();
    let mut kept = List::uncapped();
    let mut lost = List::uncapped();
    let summary = lines.first().expect("verdict summary");
    let mut size = summary.len();
    kept.push(*summary).expect("summary");
    for at in 1..lines.len() {
        let mut possible = lost.clone();
        let after = at.checked_add(1).expect("attribution cursor");
        for line in lines.get(after..).expect("remaining attributions") {
            possible.push(*line).expect("remaining tests");
        }
        let remainder = report_remainder(possible.as_slice());
        let line = lines.get(at).expect("attribution cursor");
        let length = line.len().checked_add(1).expect("attribution separator");
        let needed = size.checked_add(length).expect("bounded verdict");
        let tail = if remainder.is_empty() { 0 } else { remainder.len().checked_add(1).expect("remainder separator") };
        if needed.checked_add(tail).expect("bounded verdict") <= room {
            kept.push(*line).expect("whole test attribution");
            size = needed;
        } else {
            lost.push(*line).expect("omitted attribution");
        }
    }
    let remainder = report_remainder(lost.as_slice());
    let tail = if remainder.is_empty() { 0 } else { remainder.len().saturating_add(1) };
    let names = match names {
        Some(line) => fit_names(line, room.saturating_sub(size).saturating_sub(tail).saturating_sub(1)),
        None => None,
    };
    if let Some(names) = &names {
        kept.push(names.as_ref()).expect("many names before remainder");
    }
    if !remainder.is_empty() {
        kept.push(&remainder).expect("explicit remainder");
    }
    fit_report(kept.as_slice(), bound)
}

// The forge's prefix distinguishes this optional display line from whole landing attributions.
fn fit_names(line: &[u8], room: usize) -> Option<Box<[u8]>> {
    let body = line.strip_prefix(b"Its report names: ").expect("many names marker");
    let mut ids: List<Box<[u8]>> = List::uncapped();
    let mut start = None;
    let mut escaped = false;
    let mut end = 0;
    for (at, byte) in body.iter().enumerate() {
        if escaped {
            escaped = false;
        } else if *byte == b'\\' {
            escaped = true;
        } else if *byte == b'"' {
            match start.take() {
                Some(from) => {
                    end = at.checked_add(1).expect("closing quote");
                    ids.push(Box::from(body.get(from..end).expect("whole quoted id"))).expect("many names");
                }
                None => start = Some(at),
            }
        }
    }
    let tail = body.get(end..).expect("names remainder").strip_suffix(b".").expect("names full stop");
    let cut_suffix = b", its report cut at the read's bound";
    let cut = tail.ends_with(cut_suffix);
    let tail = if cut { tail.strip_suffix(cut_suffix).expect("cut suffix") } else { tail };
    let more = if tail.is_empty() || tail == b" and more" {
        0
    } else {
        let digits = tail.strip_prefix(b" and ").expect("names remainder").strip_suffix(b" more").expect("more suffix");
        u32::try_from(skein_lib::bytes::decimal(digits).expect("many remainder count")).expect("bounded remainder")
    };
    let many = temper_engine_domain_forge_tiers::ManyTests { tests: Box::new([]), more, cut };
    let count = ids.len();
    let mut prefix = Box::from(b"Its report names: ".as_slice());
    let mut fitted = None;
    for kept in 0..=count {
        if kept != 0 {
            let at = kept.checked_sub(1).expect("previous name");
            let test = ids.as_slice().get(skein_lib::usize_of(at)).expect("whole quoted name");
            prefix = skein_lib::bytes::concat(&[&prefix, if kept == 1 { b"" } else { b", " }, test]);
        }
        let tail = match crate::many_remainder(&many, count.checked_sub(kept).expect("unshown names")) {
            Some(tail) => skein_lib::bytes::concat(&[if kept == 0 { b"" } else { b" " }, &tail]),
            None => Box::new([]),
        };
        let next = skein_lib::bytes::concat(&[&prefix, &tail, b"."]);
        if next.len() <= room {
            fitted = Some(next);
        }
    }
    fitted
}

fn fit_report(lines: &[&[u8]], bound: u32) -> Box<[u8]> {
    let mut size = lines.len().saturating_sub(1);
    for line in lines {
        size = size.checked_add(line.len()).expect("bounded report");
    }
    let mut summary = *lines.first().expect("verdict summary");
    if size > skein_lib::usize_of(bound)
        && let Some(at) = skein_lib::bytes::find(summary, b": red,")
    {
        let end = at.checked_add(b": red".len()).expect("summary cursor");
        summary = summary.get(..end).expect("run identity");
    }
    let mut result = List::uncapped();
    for byte in summary {
        result.push(*byte).expect("summary words");
    }
    for line in lines.get(1..).expect("verdict attributions") {
        result.push(b'\n').expect("verdict separator");
        for byte in *line {
            result.push(*byte).expect("whole verdict line");
        }
    }
    result.into_boxed()
}

fn report_remainder(lines: &[&[u8]]) -> Box<[u8]> {
    if lines.is_empty() {
        return Box::new([]);
    }
    let mut many = false;
    let mut continuation: &[u8] = b"";
    for line in lines {
        let reference = report_reference(line);
        let end = line.len().saturating_sub(reference.len());
        many |= line.get(..end).expect("attribution without reference").ends_with(b": many tests");
        if continuation.is_empty() {
            continuation = reference;
        }
    }
    let count = skein_lib::Decimal::of(skein_lib::len_u64(lines.len()));
    let mut result = List::uncapped();
    let suffix: &[u8] = if !many && lines.len() == 1 { b" more failing test" } else { b" more failing tests" };
    for part in [b"and ".as_slice(), if many { b"many" } else { count.as_bytes() }, suffix, continuation] {
        for byte in part {
            result.push(*byte).expect("remainder words");
        }
    }
    result.into_boxed()
}

fn report_reference(line: &[u8]) -> &[u8] {
    let mut reference: &[u8] = b"";
    for at in 0..line.len() {
        if let Some(number) = line.get(at..).expect("reference cursor").strip_prefix(b"; change show ") {
            let mut decimal = !number.is_empty();
            for byte in number {
                decimal &= byte.is_ascii_digit();
            }
            if decimal {
                reference = line.get(at..).expect("whole reference");
            }
        }
    }
    reference
}

fn report_lines(words: &[u8]) -> List<&[u8]> {
    let mut lines = List::uncapped();
    let mut from = 0;
    for (at, byte) in words.iter().enumerate() {
        if *byte == b'\n' {
            lines.push(words.get(from..at).expect("whole attribution")).expect("verdict lines");
            from = at.checked_add(1).expect("bounded words");
        }
    }
    lines.push(words.get(from..).expect("last attribution")).expect("last verdict line");
    lines
}
