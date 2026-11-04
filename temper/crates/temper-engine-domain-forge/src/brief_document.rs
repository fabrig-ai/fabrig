//! Charter documents gathered at one branch head and retained for line cuts
//! (temper/docs/design/protocol/configuration.md, 13.7).

use alloc::boxed::Box;
use skein_forge::api::{Answer, Commit, Error, Read, Repository};
use skein_lib::{Decimal, Queue, Time, Token, Writer};

use crate::{BriefDocument, BriefRead, Domain, Request};

#[derive(Debug)]
pub(crate) struct Fetch {
    section: Token,
    document: Box<BriefDocument>,
    repository: Repository,
    read: Read,
    head: Option<Commit>,
    max_bytes: u32,
    refused: Option<u64>,
    deadline: Option<Time>,
}

#[derive(Debug)]
pub(crate) struct Section {
    prefix: Box<[u8]>,
    text: Box<[u8]>,
    truncated: bool,
}

impl Section {
    pub(crate) fn render(&self, budget: u32) -> Option<Box<[u8]>> {
        let room = usize::try_from(budget).ok()?.checked_sub(self.prefix.len())?;
        if !self.truncated && self.text.len() <= room {
            return Some(words(&[&self.prefix, &self.text]));
        }
        let mut kept = 0;
        if marker_size(self.text.len(), self.truncated) > room {
            return None;
        }
        for (index, byte) in self.text.iter().enumerate() {
            if *byte == b'\n' {
                let end = index.saturating_add(1);
                let lost = self.text.len().saturating_sub(end);
                if end.saturating_add(marker_size(lost, self.truncated)) <= room {
                    kept = end;
                }
            }
            if index >= room {
                break;
            }
        }
        let notice = marker(self.text.len().saturating_sub(kept), self.truncated);
        Some(words(&[&self.prefix, self.text.get(..kept).expect("measured line end"), &notice]))
    }
}

fn marker_size(lost: usize, truncated: bool) -> usize {
    let digits = Decimal::of(u64::try_from(lost).expect("bounded file size"));
    let start = if truncated { b"[more than ".len() } else { b"[".len() };
    start.saturating_add(digits.as_bytes().len()).saturating_add(b" bytes cut]\n".len())
}

pub(crate) fn call(section: Token) -> Token {
    Token::new(section.raw() | (1_u64 << 61_u32) | (1_u64 << 59_u32))
}

fn marker(lost: usize, truncated: bool) -> Box<[u8]> {
    let count = Decimal::of(u64::try_from(lost).expect("bounded file size"));
    let start = if truncated { b"[more than ".as_slice() } else { b"[".as_slice() };
    words(&[start, count.as_bytes(), b" bytes cut]\n"])
}

fn words(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded document words");
    }
    let mut writer = Writer::new(size);
    for part in parts {
        writer.put(part).expect("measured document words");
    }
    writer.finish()
}

fn repository(domain: &Domain, document: &BriefDocument) -> Option<Repository> {
    for (provider, row) in &domain.repositories {
        if row.host == document.host
            && row.owner == document.owner
            && row.name == document.name
            && !domain.forge_ended(provider.forge)
        {
            return Some(*provider);
        }
    }
    None
}

pub(crate) fn gather(domain: &mut Domain, section: Token, parts: u32, max_bytes: u32, out: &mut Queue<Request>) {
    let Some(document) = domain.brief_documents.remove(&section) else {
        crate::held::ready(domain, section, BriefRead::Failed, out);
        return;
    };
    if parts == 0 || domain.document_fetches.len() == domain.document_fetches.capacity() {
        crate::held::ready(domain, section, BriefRead::Failed, out);
        return;
    }
    let Some(repository) = repository(domain, &document) else {
        reason(
            domain,
            section,
            &[b"Repository ", &document.owner, b"/", &document.name, b" on ", &document.host, b" is not adopted."],
            out,
        );
        return;
    };
    let read = Read::Branch { branch: document.branch.clone() };
    let fetch = Fetch {
        section,
        document,
        repository,
        read: read.clone(),
        head: None,
        max_bytes,
        refused: None,
        deadline: None,
    };
    let owner = call(section);
    domain.document_fetches.insert(owner, fetch).expect("bounded document read");
    out.push(Request::BriefClient { event: skein_forge_client::Event::Read { owner, repository, read } });
}

fn reason(domain: &mut Domain, section: Token, parts: &[&[u8]], out: &mut Queue<Request>) {
    crate::held::ready_document(
        domain,
        section,
        Section { prefix: Box::new([]), text: words(parts), truncated: false },
        out,
    );
}

pub(crate) fn answer(
    domain: &mut Domain,
    owner: Token,
    result: Result<Answer, Error>,
    out: &mut Queue<Request>,
) -> bool {
    let Some(mut fetch) = domain.document_fetches.remove(&owner) else { return false };
    let section = fetch.section;
    if result == Err(Error::Busy) {
        fetch.refused = Some(domain.brief_reclaim);
        domain.document_fetches.insert(owner, fetch).expect("same document read room");
        return true;
    }
    let doc = &fetch.document;
    match result {
        Ok(Answer::Commit(head)) if fetch.head.is_none() => {
            fetch.head = Some(head);
            fetch.read = Read::File { head, path: doc.path.clone(), max_bytes: fetch.max_bytes };
            out.push(Request::BriefClient {
                event: skein_forge_client::Event::Read {
                    owner,
                    repository: fetch.repository,
                    read: fetch.read.clone(),
                },
            });
            domain.document_fetches.insert(owner, fetch).expect("same document read room");
        }
        Ok(Answer::File { head, path, bytes, truncated }) if fetch.head == Some(head) && path == doc.path => {
            let (text, truncated) = match &doc.heading {
                Some(heading) => match heading_part(&bytes, heading) {
                    Some((start, end, ended)) => {
                        (bytes.get(start..end).expect("heading part within file"), truncated && !ended)
                    }
                    None => {
                        reason(
                            domain,
                            section,
                            &[
                                b"No heading \"",
                                heading,
                                b"\" in ",
                                &doc.path,
                                b" at ",
                                &doc.branch,
                                b"'s head ",
                                &crate::commit_text(&head),
                                b".",
                            ],
                            out,
                        );
                        return true;
                    }
                },
                None => (bytes.as_ref(), truncated),
            };
            let prefix = words(&[b"At ", &doc.branch, b"'s head ", &crate::commit_text(&head), b"\n"]);
            let document = Section { prefix, text: Box::from(text), truncated };
            crate::held::ready_document(domain, section, document, out);
        }
        Err(Error::Missing) if fetch.head.is_none() => {
            reason(domain, section, &[b"No branch ", &doc.branch, b" in ", &doc.owner, b"/", &doc.name, b"."], out);
        }
        Err(Error::Missing) | Ok(Answer::Directory { .. }) if fetch.head.is_some() => {
            let head = fetch.head.expect("file read pinned at head");
            reason(
                domain,
                section,
                &[
                    b"No file ",
                    &doc.path,
                    b" in ",
                    &doc.owner,
                    b"/",
                    &doc.name,
                    b" at ",
                    &doc.branch,
                    b"'s head ",
                    &crate::commit_text(&head),
                    b".",
                ],
                out,
            );
        }
        Ok(_) | Err(_) => crate::held::ready(domain, section, BriefRead::Failed, out),
    }
    true
}

/// A heading starts at the first hash; fenced code cannot delimit a section.
fn heading_part(text: &[u8], wanted: &[u8]) -> Option<(usize, usize, bool)> {
    let mut offset = 0_usize;
    let mut start = None;
    let mut level = 0;
    let mut fence = None;
    for _ in 0..text.len() {
        if offset >= text.len() {
            break;
        }
        let end = match skein_lib::bytes::find_from(text, b"\n", offset) {
            Some(index) => index.saturating_add(1),
            None => text.len(),
        };
        let raw = text.get(offset..end).expect("line within file");
        let line = raw.strip_suffix(b"\n").unwrap_or(raw);
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if let Some((mark, count)) = fence {
            if fence_line(line, mark, count, true).is_some() {
                fence = None;
            }
        } else if let Some(open) = fence_line(line, b'`', 3, false).or(fence_line(line, b'~', 3, false)) {
            fence = Some(open);
        } else {
            let mut hashes = 0_usize;
            for byte in line {
                if *byte != b'#' {
                    break;
                }
                hashes = hashes.saturating_add(1);
            }
            if (1..=6).contains(&hashes) && line.get(hashes) == Some(&b' ') {
                if let Some(begin) = start {
                    if hashes <= level {
                        return Some((begin, offset, true));
                    }
                } else if line.get(hashes.saturating_add(1)..) == Some(wanted) {
                    start = Some(offset);
                    level = hashes;
                }
            }
        }
        offset = offset.saturating_add(raw.len());
    }
    Some((start?, text.len(), false))
}

fn fence_line(line: &[u8], mark: u8, minimum: usize, closing: bool) -> Option<(u8, usize)> {
    let mut indent = 0_usize;
    for byte in line {
        if *byte != b' ' {
            break;
        }
        indent = indent.saturating_add(1);
    }
    if indent > 3 {
        return None;
    }
    let line = line.get(indent..)?;
    let mut count = 0_usize;
    for byte in line {
        if *byte != mark {
            break;
        }
        count = count.saturating_add(1);
    }
    if count < minimum {
        return None;
    }
    if closing {
        for byte in line.get(count..)? {
            if *byte != b' ' && *byte != b'\t' {
                return None;
            }
        }
    }
    Some((mark, count))
}

pub(crate) fn set_deadline(domain: &mut Domain, section: Token, deadline: Time) {
    if let Some(fetch) = domain.document_fetches.get_mut(&call(section)) {
        fetch.deadline = Some(deadline);
    }
}

pub(crate) fn next_deadline(domain: &Domain) -> Option<Time> {
    let mut next: Option<Time> = None;
    for (_, fetch) in &domain.document_fetches {
        if fetch.refused.is_some()
            && let Some(deadline) = fetch.deadline
        {
            next = Some(match next {
                Some(previous) => previous.min(deadline),
                None => deadline,
            });
        }
    }
    next
}

fn waiting_section(domain: &Domain) -> Option<Token> {
    for (section, fetch) in &domain.document_fetches {
        if let Some(reclaim) = fetch.refused
            && reclaim != domain.brief_reclaim
        {
            return Some(*section);
        }
    }
    None
}

pub(crate) fn waiting(domain: &Domain) -> bool {
    waiting_section(domain).is_some()
}

pub(crate) fn retry(domain: &mut Domain, out: &mut Queue<Request>) {
    let Some(section) = waiting_section(domain) else { return };
    let fetch = domain.document_fetches.get_mut(&section).expect("waiting document read");
    fetch.refused = None;
    out.push(Request::BriefClient {
        event: skein_forge_client::Event::Read {
            owner: section,
            repository: fetch.repository,
            read: fetch.read.clone(),
        },
    });
}

pub(crate) fn expire(domain: &mut Domain, now: Time, out: &mut Queue<Request>) {
    for _ in 0..domain.document_fetches.capacity() {
        let mut expired = None;
        for (section, fetch) in &domain.document_fetches {
            if fetch.refused.is_some()
                && let Some(deadline) = fetch.deadline
                && deadline <= now
            {
                expired = Some(*section);
                break;
            }
        }
        let Some(section) = expired else { break };
        let fetch = domain.document_fetches.remove(&section).expect("expired document");
        crate::held::ready(domain, fetch.section, BriefRead::Failed, out);
    }
}
