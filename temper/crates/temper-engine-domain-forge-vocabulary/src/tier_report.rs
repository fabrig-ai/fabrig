//! Bounded completed tier reports (temper/docs/design/domain/after-landing-storage.md, 1).
use alloc::boxed::Box;
use skein_lib::List;

// Each verdict line holds one whole escaped ID and its retained change read.
pub fn words(words: &[u8], bound: u32) -> Box<[u8]> {
    let lines = report_lines(words);
    let mut offset = 0_usize;
    for line in lines.as_slice() {
        if line.ends_with(b" red filings wait for room in the project's findings.")
            || line.ends_with(b" red filings were dropped for want of room.")
        {
            let end = offset.saturating_sub(1);
            let tail = words.get(end..).expect("report tail");
            let reserve = u32::try_from(tail.len()).expect("bounded filing counts");
            let head = report_attributions(words.get(..end).expect("report head"), bound.saturating_sub(reserve));
            let mut result = List::uncapped();
            for part in [head.as_ref(), tail] {
                for byte in part {
                    result.push(*byte).expect("report words");
                }
            }
            return result.into_boxed();
        }
        offset = offset.saturating_add(line.len()).saturating_add(1);
    }
    report_attributions(words, bound)
}
fn report_attributions(words: &[u8], bound: u32) -> Box<[u8]> {
    let room = skein_lib::usize_of(bound);
    if words.len() <= room {
        return Box::from(words);
    }
    let parsed = report_lines(words);
    let lines = parsed.as_slice();
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
    if !remainder.is_empty() {
        kept.push(&remainder).expect("explicit remainder");
    }
    fit_report(kept.as_slice(), bound)
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
