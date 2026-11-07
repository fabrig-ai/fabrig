//! An independent HTTP chunk/SSE parser retains only complete received records.
#[derive(Default)]
pub(super) struct Records {
    received: usize,
    searched_head: usize,
    offset: Option<usize>,
    searched_chunk: usize,
    chunk: Option<(usize, usize)>,
    bytes: Vec<u8>,
    line: usize,
    searched_line: usize,
    pub values: Vec<Box<[u8]>>,
}
impl Records {
    pub fn feed(&mut self, http: &[u8]) {
        if self.received == http.len() {
            return;
        }
        self.received = http.len();
        let at = if let Some(at) = self.offset {
            at
        } else if let Some(at) = super::head_end(http, &mut self.searched_head) {
            self.offset = Some(at);
            self.searched_chunk = at;
            at
        } else {
            return;
        };
        let mut at = at;
        while let Some(chunks) = http.get(at..) {
            let (start, next) = if let Some(chunk) = self.chunk {
                chunk
            } else {
                let Some(end) = http[self.searched_chunk..].windows(2).position(|b| b == b"\r\n") else {
                    self.searched_chunk = http.len().saturating_sub(1).max(at);
                    break;
                };
                let end = self.searched_chunk + end;
                let Ok(text) = std::str::from_utf8(&http[at..end]) else { break };
                let Ok(length) = usize::from_str_radix(text, 16) else { break };
                let chunk = (end + 2, end + 4 + length);
                self.chunk = Some(chunk);
                chunk
            };
            if chunks.len() < next - at {
                break;
            }
            self.bytes.extend_from_slice(&http[start..next - 2]);
            at = next;
            self.chunk = None;
            self.searched_chunk = at;
        }
        self.offset = Some(at);
        while let Some(end) = self.bytes[self.searched_line..].iter().position(|b| *b == b'\n') {
            let end = self.searched_line + end;
            let event = &self.bytes[self.line..end];
            self.line = end + 1;
            self.searched_line = self.line;
            if let Some(value) = event.strip_prefix(b"data: ") {
                let decoded = skein_lib::base64::decode(
                    skein_lib::base64::Alphabet::Standard,
                    value.strip_suffix(b"\r").unwrap_or(value),
                    65536,
                )
                .expect("a complete encoded stream record");
                self.values.push(decoded);
            }
        }
        self.searched_line = self.bytes.len();
    }
}
