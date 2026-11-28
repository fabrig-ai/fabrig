//! Tab-separated render verdicts (temper/docs/design/client/journey-writer-requests.md, 1.2, step 8).

use super::{Why, copies::Page};

const LISTED: usize = 50;

#[derive(Default)]
pub(super) struct Report {
    output: Vec<u8>,
    compared: bool,
    differs: bool,
}

impl Report {
    pub(super) fn uncompared(&mut self, page: &Page, why: Why) {
        line(&mut self.output, &[b"uncompared", page.item().as_bytes(), why.words().as_bytes()]);
    }

    pub(super) fn compared(&mut self, page: &Page, report: PageReport) {
        self.compared = true;
        let item = page.item();
        if report.count == 0 {
            line(&mut self.output, &[b"identical", item.as_bytes()]);
            return;
        }
        self.differs = true;
        self.output.extend(report.output);
        if report.count > LISTED {
            let more = (report.count - LISTED).to_string();
            line(&mut self.output, &[b"more", item.as_bytes(), more.as_bytes()]);
        }
    }

    pub(super) fn finish(mut self) -> Vec<u8> {
        let result: &[u8] = if self.differs {
            b"differs"
        } else if self.compared {
            b"identical"
        } else if self.output.is_empty() {
            b"nothing to render"
        } else {
            b"not compared"
        };
        line(&mut self.output, &[b"result", result]);
        self.output
    }
}

#[derive(Default)]
pub(super) struct PageReport {
    output: Vec<u8>,
    count: usize,
}

impl PageReport {
    pub(super) fn difference(
        &mut self,
        page: &str,
        width: &str,
        element: &str,
        property: &[u8],
        values: (&[u8], &[u8]),
    ) {
        self.count += 1;
        if self.count <= LISTED {
            line(
                &mut self.output,
                &[b"differs", page.as_bytes(), width.as_bytes(), element.as_bytes(), property, values.0, values.1],
            );
        }
    }
}

pub(crate) fn line(out: &mut Vec<u8>, fields: &[&[u8]]) {
    for (at, field) in fields.iter().enumerate() {
        if at != 0 {
            out.push(b'\t');
        }
        for &byte in *field {
            match byte {
                b'\\' => out.extend_from_slice(b"\\\\"),
                b'\t' => out.extend_from_slice(b"\\t"),
                b'\r' => out.extend_from_slice(b"\\r"),
                b'\n' => out.extend_from_slice(b"\\n"),
                byte => out.push(byte),
            }
        }
    }
    out.push(b'\n');
}
