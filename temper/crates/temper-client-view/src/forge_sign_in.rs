//! Bounded page-owned forge facts (protocol/sign-in.md, 9.3).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use skein_lib::List;
use skein_ui::phrases::Hole;

pub const FORGES: u32 = 32;
pub const META_BYTES: usize = 1408;
pub const ROOT_BYTES: usize = 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forge {
    pub number: u16,
    pub client: Box<[u8]>,
    pub root: Box<[u8]>,
    pub owned: bool,
}
#[derive(Debug)]
pub struct Forges {
    entries: List<Forge>,
}
impl Default for Forges {
    fn default() -> Self {
        Self::new()
    }
}
impl Forges {
    #[must_use]
    pub fn new() -> Self {
        Self { entries: List::with_capacity(FORGES) }
    }
    /// Malformed, oversized and repeated entries are dropped.
    pub fn read(&mut self, content: &str) {
        if self.entries.len() == FORGES || content.len() > META_BYTES {
            return;
        }
        let mut parts = content.split_ascii_whitespace();
        let (Some(number), Some(client), Some(root), Some(mode), None) =
            (parts.next(), parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return;
        };
        let Some(number) = skein_ui::number(number.as_bytes()) else { return };
        let Ok(number) = u16::try_from(number) else { return };
        if number == 0 || client.is_empty() || client.len() > 256 || !graphic(client.as_bytes()) {
            return;
        }
        if root.len() > ROOT_BYTES || !skein_ui::markdown::safe_url(root.as_bytes()) || host(root.as_bytes()).is_none()
        {
            return;
        }
        let owned = match mode {
            "owned" => true,
            "other" => false,
            _ => return,
        };
        for entry in &self.entries {
            if entry.number == number || *entry.client == *client.as_bytes() {
                return;
            }
        }
        self.entries
            .push(Forge { number, client: client.as_bytes().into(), root: root.as_bytes().into(), owned })
            .expect("forge facts bounded");
    }
    #[must_use]
    pub fn entries(&self) -> &List<Forge> {
        &self.entries
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "the programming model uses bounded loops without closures")]
    pub fn client(&self, query: &[u8]) -> Option<&Forge> {
        let mut client = None;
        let mut rest = query;
        for _ in 0..query.len() {
            let end = skein_lib::bytes::find(rest, b"&").unwrap_or(rest.len());
            let field = rest.get(..end).expect("field boundary");
            rest = rest.get(end..).expect("field boundary").strip_prefix(b"&").unwrap_or(b"");
            let Some(value) = field.strip_prefix(b"client_id=") else { continue };
            if client.is_some() {
                return None;
            }
            client = decode(value);
        }
        let client = client?;
        for entry in &self.entries {
            if entry.client == client {
                return Some(entry);
            }
        }
        None
    }
    #[must_use]
    #[expect(clippy::manual_find, reason = "the programming model uses bounded loops without closures")]
    pub fn number(&self, number: u16) -> Option<&Forge> {
        for entry in &self.entries {
            if entry.number == number {
                return Some(entry);
            }
        }
        None
    }
}
fn decode(value: &[u8]) -> Option<Box<[u8]>> {
    if value.len() > 768 {
        return None;
    }
    let mut out = List::with_capacity(256);
    let mut rest = value;
    for _ in 0..value.len() {
        let Some((byte, tail)) = rest.split_first() else { break };
        let byte = match byte {
            b'+' => b' ',
            b'%' => {
                let (Some(a), Some(b)) = (tail.first(), tail.get(1)) else { return None };
                let (a, b) = (hex(*a)?, hex(*b)?);
                rest = tail.get(2..)?;
                out.push((a << 4_u8) | b).ok()?;
                continue;
            }
            byte => *byte,
        };
        out.push(byte).ok()?;
        rest = tail;
    }
    Some(out.into_boxed())
}
fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'a'..=b'f' => byte.checked_sub(b'a')?.checked_add(10),
        b'A'..=b'F' => byte.checked_sub(b'A')?.checked_add(10),
        _ => None,
    }
}
fn graphic(bytes: &[u8]) -> bool {
    for byte in bytes {
        if !byte.is_ascii_graphic() {
            return false;
        }
    }
    true
}
fn host(root: &[u8]) -> Option<&[u8]> {
    let rest = match root.strip_prefix(b"https://") {
        Some(rest) => rest,
        None => root.strip_prefix(b"http://")?,
    };
    let mut end = rest.len();
    for (index, byte) in rest.iter().enumerate() {
        if b"/?#".contains(byte) {
            end = index;
            break;
        }
    }
    let host = rest.get(..end)?;
    if host.is_empty() || host.len() > 260 || host.contains(&b'@') || rest.contains(&b'?') || rest.contains(&b'#') {
        return None;
    }
    Some(host)
}
impl Forge {
    #[must_use]
    pub fn name(&self, words: &Words) -> Box<[u8]> {
        if self.owned {
            words.say(Word::ForgeHere, &[])
        } else {
            words.say(Word::SignInForgeAt, &[Hole::Name(host(&self.root).expect("checked forge root").into())])
        }
    }
    #[must_use]
    pub fn open(&self) -> Box<[u8]> {
        let root = self.root.strip_suffix(b"/").unwrap_or(&self.root);
        let tail = b"/user/oauth2/temper";
        let mut out = skein_lib::Writer::new(root.len().checked_add(tail.len()).expect("forge URL bound"));
        out.put(root).expect("sized URL");
        out.put(tail).expect("sized URL");
        out.finish()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::disallowed_macros, reason = "tests construct bounded synthetic metadata rows")]
    use super::*;
    #[test]
    fn malformed_and_duplicate_meta_facts_are_dropped() {
        let mut facts = Forges::new();
        for value in [
            "",
            "1 a https://forge.example",
            "0 a https://forge.example other",
            "65536 a https://forge.example other",
            "1 a javascript:alert(1) other",
            "1 a https://u@forge.example other",
            "1 a https://forge.example odd",
            "1 a https://forge.example other extra",
        ] {
            facts.read(value);
        }
        assert!(facts.entries().is_empty());
        facts.read("1 forgejo https://forge.example:3000 other");
        facts.read("1 duplicate https://else.example other");
        facts.read("2 forgejo https://else.example other");
        assert_eq!(facts.entries().len(), 1);
        assert_eq!(facts.client(b"client_id=f%6Frgejo").expect("decoded client").number, 1);
        assert!(facts.client(b"client_id=forgejo&client_id=forgejo").is_none());
        for number in 2_u16..=40_u16 {
            facts.read(&std::format!("{number} c{number} https://forge.example other"));
        }
        assert_eq!(facts.entries().len(), FORGES);
    }
    #[test]
    fn names_and_the_open_link_come_from_the_page_facts() {
        let words = crate::tests::fixtures::browser_words();
        let mut facts = Forges::new();
        facts.read("1 forgejo https://forge.example:3000 other");
        facts.read("2 owned http://127.0.0.1:3000 owned");
        assert_eq!(facts.number(1).unwrap().name(&words).as_ref(), b"Forgejo at forge.example:3000");
        assert_eq!(facts.number(2).unwrap().name(&words).as_ref(), b"Forgejo on this machine");
        assert_eq!(facts.number(1).unwrap().open().as_ref(), b"https://forge.example:3000/user/oauth2/temper");
    }
}
