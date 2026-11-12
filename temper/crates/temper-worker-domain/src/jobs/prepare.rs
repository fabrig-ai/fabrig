//! Temper's own request and bounded result (temper/docs/design/protocol/tooling.md, 4;
//! temper/docs/design/domain/worker.md, 4.4).
use crate::{checkout, wire};
use alloc::boxed::Box;
use skein_json::{
    Document, Kind,
    writer::{Encoder, Limits},
};

#[derive(Debug)]
pub(super) struct Request {
    pub(super) repository: Box<[u8]>,
    pub(super) base: checkout::git::Commit,
    pub(super) head: checkout::git::Commit,
    pub(super) upstream: Option<checkout::git::Commit>,
    pub(super) squash: bool,
    pub(super) member: u64,
    message: Box<[u8]>,
    squash_message: Box<[u8]>,
}

pub(super) fn decode(bytes: &[u8]) -> Option<Request> {
    let d = skein_json::parse(
        bytes,
        &skein_json::tokenizer::Limits { depth: 8, string: 16_384, number: 20, chunk: 256, length: 16_384 },
        &skein_json::document::Limits { tokens: 128, text: 16_384 },
    )
    .ok()?;
    if text(&d, field(&d, 0, b"version")?, Kind::Number)? != b"1" {
        return None;
    }
    let members = field(&d, 0, b"members")?;
    if d.token(members)?.kind != Kind::ArrayStart {
        return None;
    }
    let member = members.checked_add(1)?;
    let end = skip(&d, member)?;
    if d.token(end)?.kind != Kind::ArrayEnd {
        return None;
    }
    // This built kind has no project tooling, upstream repositories or keys.
    for name in [b"upstreams".as_slice(), b"gates".as_slice()] {
        let at = field(&d, 0, name)?;
        if d.token(at)?.kind != Kind::ArrayStart || d.token(at.checked_add(1)?)?.kind != Kind::ArrayEnd {
            return None;
        }
    }
    let combine = string(&d, 0, b"combine")?;
    let squash = match combine {
        b"merge" => false,
        b"squash" => true,
        _ => return None,
    };
    let upstream = match optional_field(&d, 0, b"train_upstream").ok()? {
        Some(at) => Some(commit(text(&d, at, Kind::String)?)?),
        None => None,
    };
    let number = text(&d, field(&d, member, b"number")?, Kind::Number)?;
    let mut member_number = 0_u64;
    for byte in number {
        if !byte.is_ascii_digit() {
            return None;
        }
        member_number = member_number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    if string(&d, member, b"branch")?.is_empty() || member_number == 0 {
        return None;
    }
    let message = string(&d, member, b"message")?;
    let squash_message = string(&d, member, b"squash")?;
    if message.is_empty() || squash_message.is_empty() || message.contains(&0) || squash_message.contains(&0) {
        return None;
    }
    Some(Request {
        repository: string(&d, 0, b"repository")?.into(),
        base: commit(string(&d, 0, b"main")?)?,
        head: commit(string(&d, member, b"head")?)?,
        upstream,
        squash,
        member: member_number,
        message: message.into(),
        squash_message: squash_message.into(),
    })
}

fn text(d: &Document, at: u32, kind: Kind) -> Option<&[u8]> {
    let token = d.token(at)?;
    if token.kind == kind { d.text(token) } else { None }
}
fn string<'a>(d: &'a Document, at: u32, key: &[u8]) -> Option<&'a [u8]> {
    text(d, field(d, at, key)?, Kind::String)
}
fn field(d: &Document, at: u32, key: &[u8]) -> Option<u32> {
    optional_field(d, at, key).ok()?
}
fn optional_field(d: &Document, at: u32, key: &[u8]) -> Result<Option<u32>, ()> {
    if d.token(at).ok_or(())?.kind != Kind::ObjectStart {
        return Err(());
    }
    let mut cursor = at.checked_add(1).ok_or(())?;
    let mut found = None;
    while d.token(cursor).ok_or(())?.kind != Kind::ObjectEnd {
        let name = text(d, cursor, Kind::Key).ok_or(())?;
        cursor = cursor.checked_add(1).ok_or(())?;
        if name == key {
            if found.is_some() {
                return Err(());
            }
            found = Some(cursor);
        }
        cursor = skip(d, cursor).ok_or(())?;
    }
    Ok(found)
}
fn skip(d: &Document, at: u32) -> Option<u32> {
    let mut depth = 0_u32;
    let mut cursor = at;
    loop {
        match d.token(cursor)?.kind {
            Kind::ObjectStart | Kind::ArrayStart => depth = depth.checked_add(1)?,
            Kind::ObjectEnd | Kind::ArrayEnd => depth = depth.checked_sub(1)?,
            Kind::Key | Kind::String | Kind::Number | Kind::True | Kind::False | Kind::Null | Kind::Long => {}
        }
        cursor = cursor.checked_add(1)?;
        if depth == 0 {
            return Some(cursor);
        }
    }
}
fn commit(bytes: &[u8]) -> Option<checkout::git::Commit> {
    if bytes.len() != 40 && bytes.len() != 64 {
        return None;
    }
    let mut raw = [0; 32];
    for (pair, byte) in bytes.chunks_exact(2).zip(raw.iter_mut()) {
        let high = char::from(*pair.first()?).to_digit(16)?;
        let low = char::from(*pair.get(1)?).to_digit(16)?;
        *byte = u8::try_from(high.checked_mul(16)?.checked_add(low)?).ok()?;
    }
    Some(checkout::git::Commit::new(raw))
}

pub(super) fn check(request: &Request, items: &wire::Workspace, limits: &crate::Limits) -> bool {
    let bound = u64::from(limits.checkout.message_bytes);
    if limits.checkout.conflicts == 0
        || limits.checkout.path_bytes == 0
        || skein_lib::len_u64(request.message.len()) > bound
        || skein_lib::len_u64(request.squash_message.len()) > bound
    {
        return false;
    }
    let [repository] = items.repositories.as_ref() else { return false };
    repository.name == request.repository
        && match repository.start {
            wire::Start::Commit { commit } => commit == request.base.raw(),
            wire::Start::Base { .. }
            | wire::Start::Branch { .. }
            | wire::Start::Saved { .. }
            | wire::Start::Merge { .. }
            | wire::Start::Merged { .. } => false,
        }
        && match repository.access {
            wire::Access::Writable { expected: None, landing: None, .. } => true,
            wire::Access::Writable { .. } | wire::Access::ReadOnly => false,
        }
}
pub(super) fn start(request: &Request) -> checkout::Start {
    checkout::Start::Prepare {
        request: Box::new(checkout::CandidateSpec {
            base: request.base,
            head: request.head,
            upstream: request.upstream,
            squash: request.squash,
            message: message(&request.message),
            squash_message: message(&request.squash_message),
        }),
    }
}
fn message(bytes: &[u8]) -> checkout::Message {
    checkout::Message { title: bytes.into(), body: Box::new([]) }
}

pub(super) fn result(member: u64, outcome: &checkout::Candidate, most: u32) -> Option<(Box<[u8]>, u8)> {
    let limits = Limits { depth: 4, length: most };
    let (mut kept, total) = match outcome {
        checkout::Candidate::Combined { .. } => (0, 0),
        checkout::Candidate::Conflict { files, .. } => (files.len(), files.len()),
    };
    loop {
        let mut measure = Encoder::measure(&limits);
        encode(&mut measure, member, outcome, kept, kept < total);
        if let Ok(length) = measure.measured() {
            let mut writer = Encoder::write(length, &limits);
            encode(&mut writer, member, outcome, kept, kept < total);
            return Some((
                writer.finish(),
                match outcome {
                    checkout::Candidate::Combined { .. } => 0,
                    checkout::Candidate::Conflict { .. } => 3,
                },
            ));
        }
        kept = kept.checked_sub(1)?;
    }
}
fn encode(json: &mut Encoder, member: u64, outcome: &checkout::Candidate, kept: usize, cut: bool) {
    json.object_start();
    match outcome {
        checkout::Candidate::Combined { squash } => {
            json.key(b"combined");
            json.array_start();
            json.object_start();
            json.key(b"member");
            json.unsigned(member);
            json.key(b"by");
            json.string(if *squash { b"squash" } else { b"merge" });
            json.object_end();
            json.array_end();
        }
        checkout::Candidate::Conflict { files, cut: truncated } => {
            json.key(b"conflict");
            json.object_start();
            json.key(b"member");
            json.unsigned(member);
            json.key(b"with");
            json.string(b"branch");
            json.key(b"files");
            json.array_start();
            for file in files.iter().take(kept) {
                json.string(file);
            }
            json.array_end();
            json.key(b"cut");
            json.boolean(cut || *truncated);
            json.object_end();
        }
    }
    json.object_end();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conflict_results_keep_a_prefix_within_the_bound_and_preserve_git_truncation() {
        let files = Box::new([
            Box::from(&b"aaaaaaaaaaaaaaaaaaaa"[..]),
            Box::from(&b"bbbbbbbbbbbbbbbbbbbb"[..]),
            Box::from(&b"cccccccccccccccccccc"[..]),
        ]);
        let (bytes, code) =
            result(12, &checkout::Candidate::Conflict { files, cut: false }, 100).expect("bounded prefix");
        assert_eq!(code, 3);
        assert_eq!(
            bytes.as_ref(),
            br#"{"conflict":{"member":12,"with":"branch","files":["aaaaaaaaaaaaaaaaaaaa"],"cut":true}}"#
        );
        let (bytes, code) =
            result(12, &checkout::Candidate::Conflict { files: Box::new([Box::from(&b"code"[..])]), cut: true }, 4096)
                .expect("git prefix");
        assert_eq!(code, 3);
        assert_eq!(bytes.as_ref(), br#"{"conflict":{"member":12,"with":"branch","files":["code"],"cut":true}}"#);
    }
}
