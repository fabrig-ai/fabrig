//! Demand checks for a world's stream below (skein/docs/design/testing/README.md, 6).

use skein_lib::stream::Read;

/// Whether `bytes` hold what `read` asks for, from their start.
#[must_use]
pub fn meets(read: Read, bytes: &[u8]) -> bool {
    match read {
        Read::Nothing => false,
        Read::Fill(n) => bytes.len() >= usize::try_from(n).expect("fits a usize"),
        Read::Arrived { .. } => !bytes.is_empty(),
        Read::Scan { until, max } => {
            let max = usize::try_from(max).expect("fits a usize");
            let window = &bytes[..max.min(bytes.len())];
            bytes.len() >= max || window.windows(until.as_bytes().len()).any(|found| found == until.as_bytes())
        }
        Read::Line { max } => {
            let max = usize::try_from(max).expect("fits a usize");
            bytes.len() >= max || bytes[..max.min(bytes.len())].iter().any(|&byte| byte == b'\r' || byte == b'\n')
        }
    }
}

/// Whether `bytes` meet `read` exactly, or with what has arrived.
#[must_use]
pub fn delivers(read: Read, bytes: &[u8]) -> bool {
    match read {
        Read::Nothing => false,
        Read::Fill(n) => bytes.len() == usize::try_from(n).expect("fits a usize"),
        Read::Arrived { max } => !bytes.is_empty() && bytes.len() <= usize::try_from(max).expect("fits a usize"),
        Read::Scan { until, max } => {
            let delimiter = until.as_bytes();
            let first = bytes.windows(delimiter.len()).position(|window| window == delimiter);
            match first {
                Some(at) => at + delimiter.len() == bytes.len(),
                None => bytes.len() == usize::try_from(max).expect("fits a usize"),
            }
        }
        Read::Line { max } => match bytes.iter().position(|&byte| byte == b'\r' || byte == b'\n') {
            Some(at) => at + 1 == bytes.len(),
            None => bytes.len() == usize::try_from(max).expect("fits a usize"),
        },
    }
}
