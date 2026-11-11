use std::path::Path;
/// A single map for every invocation in a capture.
pub struct Mask {
    scratch: String,
    ports: Vec<u16>,
    ids: Vec<(String, usize)>,
}
impl Mask {
    #[must_use]
    pub fn new(scratch: &Path, port: u16) -> Self {
        Self { scratch: scratch.to_string_lossy().into_owned(), ports: vec![port], ids: Vec::new() }
    }
    pub(crate) fn port(&mut self, port: u16) {
        self.ports.push(port);
    }
    #[must_use]
    pub fn bytes(&mut self, bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut cursor = 0;
        for i in 0..bytes.len() {
            if i < cursor {
                continue;
            }
            if bytes[i..].starts_with(self.scratch.as_bytes()) {
                out.extend_from_slice(b"/scratch");
                cursor = i + self.scratch.len();
                continue;
            }
            if bytes[i..].starts_with(b"after ") {
                let digits = bytes[i + 6..].iter().take_while(|b| b.is_ascii_digit()).count();
                if digits > 0 && bytes[i + 6 + digits..].starts_with(b" ms") {
                    out.extend_from_slice(b"after 0 ms");
                    cursor = i + 9 + digits;
                    continue;
                }
            }
            if bytes[i..].starts_with(b"port ")
                && let Some(port) =
                    self.ports.iter().map(u16::to_string).find(|p| bytes[i + 5..].starts_with(p.as_bytes()))
            {
                out.extend_from_slice(b"port 3000");
                cursor = i + 5 + port.len();
                continue;
            }
            if bytes[i] == b':'
                && let Some(port) = self.ports.iter().map(u16::to_string).find(|p| {
                    bytes[i + 1..].starts_with(p.as_bytes())
                        && bytes.get(i + 1 + p.len()).is_none_or(|b| !b.is_ascii_digit())
                })
            {
                out.extend_from_slice(b":3000");
                cursor = i + 1 + port.len();
                continue;
            }
            if bytes[i].is_ascii_hexdigit()
                && !bytes[i].is_ascii_uppercase()
                && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
            {
                let len = bytes[i..].iter().take_while(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()).count();
                if (7..=64).contains(&len) && bytes.get(i + len).is_none_or(|b| !b.is_ascii_alphanumeric()) {
                    if bytes[i..i + len].iter().all(|b| *b == b'0') {
                        out.extend_from_slice(&bytes[i..i + len]);
                        cursor = i + len;
                        continue;
                    }
                    let id = String::from_utf8_lossy(&bytes[i..i + len]).into_owned();
                    let found = self.ids.iter().find(|(old, _)| old.starts_with(&id) || id.starts_with(old));
                    let number = found.map_or(self.ids.len() + 1, |(_, n)| *n);
                    if found.is_none() {
                        self.ids.push((id, number));
                    }
                    out.extend_from_slice(format!("{number:0len$x}").as_bytes());
                    cursor = i + len;
                    continue;
                }
            }
            out.push(bytes[i]);
            cursor = i + 1;
        }
        out
    }
}
