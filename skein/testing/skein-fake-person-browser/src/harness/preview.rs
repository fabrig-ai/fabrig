//! Bound both the diagnostic and the work to format it.

use std::fmt::{self, Write};

const CHARS: usize = 200;

pub(super) fn record(output: &mut String, request: &dyn fmt::Debug) {
    output.clear();
    let mut preview = Preview { output, left: CHARS };
    // The writer returns an error to stop formatting after filling the preview.
    let _full = write!(preview, "{request:?}");
}

struct Preview<'a> {
    output: &'a mut String,
    left: usize,
}

impl Write for Preview<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for (at, _) in text.char_indices() {
            if self.left == 0 {
                self.output.push_str(&text[..at]);
                return Err(fmt::Error);
            }
            self.left -= 1;
        }
        self.output.push_str(text);
        if self.left == 0 { Err(fmt::Error) } else { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::{CHARS, record};
    use std::fmt::{self, Write};

    #[test]
    fn a_full_request_preview_does_not_format_the_tail() {
        struct LongRequest;
        impl fmt::Debug for LongRequest {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                for _ in 0..CHARS {
                    formatter.write_char('é')?;
                }
                panic!("formatted beyond the preview");
            }
        }
        let mut output = String::new();
        record(&mut output, &LongRequest);
        assert_eq!(output, "é".repeat(CHARS));
    }
}
