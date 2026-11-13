//! Brief sections retained by the forge connector (jig's domain/connectors.md,
//! section 9). The parent sees a token and rendered size while the connector
//! owns the words. Taking or dropping a token ends that ownership.

pub fn edge(text: &[u8], wanted: usize, tail: bool) -> &[u8] {
    if tail {
        let mut start = text.len().saturating_sub(wanted);
        for _ in 0..3 {
            if continues(text.get(start)) {
                start = start.saturating_add(1);
            }
        }
        text.get(start..).expect("within section")
    } else {
        let mut end = wanted;
        for _ in 0..3 {
            if continues(text.get(end)) {
                end = end.saturating_sub(1);
            }
        }
        text.get(..end).expect("within section")
    }
}

fn continues(byte: Option<&u8>) -> bool {
    match byte {
        Some(byte) => byte & 0b1100_0000 == 0b1000_0000,
        None => false,
    }
}
