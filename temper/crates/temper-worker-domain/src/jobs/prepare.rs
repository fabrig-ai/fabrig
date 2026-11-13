//! Temper's own request and bounded result (temper/docs/design/protocol/tooling.md, 4;
//! temper/docs/design/domain/worker/jobs.md).
use crate::{checkout, wire};
use alloc::boxed::Box;
use skein_json::{
    Document, Kind,
    writer::{Encoder, Limits},
};

#[derive(Debug)]
pub(super) struct Request {
    pub(super) repository: Box<[u8]>,
    base: checkout::git::Commit,
    from: checkout::git::Commit,
    upstream: Option<checkout::git::Commit>,
    squash: bool,
    members: Box<[Member]>,
}
#[derive(Debug)]
struct Member {
    number: u64,
    head: checkout::git::Commit,
    prefix: Option<Box<[u8]>>,
    message: Box<[u8]>,
    squash_message: Box<[u8]>,
}

pub(super) fn decode(bytes: &[u8]) -> Option<Request> {
    let d = skein_json::parse(
        bytes,
        &skein_json::tokenizer::Limits { depth: 8, string: 16_384, number: 20, chunk: 256, length: 16_384 },
        &skein_json::document::Limits { tokens: 16_384, text: 16_384 },
    )
    .ok()?;
    if text(&d, field(&d, 0, b"version")?, Kind::Number)? != b"1" {
        return None;
    }
    let at = field(&d, 0, b"members")?;
    if d.token(at)?.kind != Kind::ArrayStart {
        return None;
    }
    let mut cursor = at.checked_add(1)?;
    let mut members = skein_lib::List::uncapped();
    for _ in 0..d.len() {
        if d.token(cursor)?.kind == Kind::ArrayEnd {
            break;
        }
        let number = text(&d, field(&d, cursor, b"number")?, Kind::Number)?;
        let mut member_number = 0_u64;
        for byte in number {
            if !byte.is_ascii_digit() {
                return None;
            }
            member_number = member_number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
        }
        if string(&d, cursor, b"branch")?.is_empty() || member_number == 0 {
            return None;
        }
        let message = string(&d, cursor, b"message")?;
        let squash_message = string(&d, cursor, b"squash")?;
        if message.is_empty() || squash_message.is_empty() || message.contains(&0) || squash_message.contains(&0) {
            return None;
        }
        let prefix = match optional_field(&d, cursor, b"prefix").ok()? {
            Some(at) => Some(text(&d, at, Kind::String)?.into()),
            None => None,
        };
        members
            .push(Member {
                number: member_number,
                head: commit(string(&d, cursor, b"head")?)?,
                prefix,
                message: message.into(),
                squash_message: squash_message.into(),
            })
            .ok()?;
        cursor = skip(&d, cursor)?;
    }
    if members.is_empty() || d.token(cursor)?.kind != Kind::ArrayEnd {
        return None;
    }
    // This built kind has no project tooling, upstream repositories or keys.
    for name in [b"upstreams".as_slice(), b"gates".as_slice()] {
        let at = field(&d, 0, name)?;
        if d.token(at)?.kind != Kind::ArrayStart || d.token(at.checked_add(1)?)?.kind != Kind::ArrayEnd {
            return None;
        }
    }
    let squash = match string(&d, 0, b"combine")? {
        b"merge" => false,
        b"squash" => true,
        _ => return None,
    };
    let upstream = match optional_field(&d, 0, b"train_upstream").ok()? {
        Some(at) => Some(commit(text(&d, at, Kind::String)?)?),
        None => None,
    };
    let base = commit(string(&d, 0, b"main")?)?;
    let from = match optional_field(&d, 0, b"from").ok()? {
        Some(at) => commit(text(&d, at, Kind::String)?)?,
        None => base,
    };
    Some(Request {
        repository: string(&d, 0, b"repository")?.into(),
        base,
        from,
        upstream,
        squash,
        members: members.into_boxed(),
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
    for (pair, byte) in bytes.as_chunks::<2>().0.iter().zip(raw.iter_mut()) {
        let high = char::from(*pair.first()?).to_digit(16)?;
        let low = char::from(*pair.get(1)?).to_digit(16)?;
        *byte = u8::try_from(high.checked_mul(16)?.checked_add(low)?).ok()?;
    }
    Some(checkout::git::Commit::new(raw))
}

pub(super) fn check(request: &Request, items: &wire::Workspace, limits: &crate::Limits) -> bool {
    let bound = u64::from(limits.checkout.message_bytes);
    if limits.checkout.conflicts == 0 || limits.checkout.path_bytes == 0 {
        return false;
    }
    for member in &request.members {
        if skein_lib::len_u64(member.message.len()) > bound || skein_lib::len_u64(member.squash_message.len()) > bound {
            return false;
        }
    }
    let [repository] = items.repositories.as_ref() else { return false };
    let starts = match repository.start {
        wire::Start::Commit { commit } => commit == request.from.raw(),
        wire::Start::Base { .. }
        | wire::Start::Branch { .. }
        | wire::Start::Saved { .. }
        | wire::Start::Merge { .. }
        | wire::Start::Merged { .. } => false,
    };
    let access = match &repository.access {
        wire::Access::Prefixes { branches } => {
            if branches.len() != request.members.len() {
                return false;
            }
            for (branch, member) in branches.iter().zip(request.members.iter()) {
                if member.prefix.as_deref() != Some(branch) {
                    return false;
                }
            }
            true
        }
        wire::Access::Writable { push, expected: None, landing: None } => {
            let [member] = request.members.as_ref() else { return false };
            match &member.prefix {
                Some(prefix) => prefix == push,
                None => true,
            }
        }
        wire::Access::Writable { .. } | wire::Access::ReadOnly => false,
    };
    repository.name == request.repository && starts && access
}
pub(super) fn start(request: &Request) -> checkout::Start {
    let mut members = skein_lib::List::uncapped();
    for member in &request.members {
        members
            .push(checkout::CandidateMember {
                number: member.number,
                head: member.head,
                message: message(&member.message),
                squash_message: message(&member.squash_message),
            })
            .expect("admitted members");
    }
    checkout::Start::Prepare {
        request: Box::new(checkout::CandidateSpec {
            base: request.base,
            from: request.from,
            upstream: request.upstream,
            squash: request.squash,
            members: members.into_boxed(),
        }),
    }
}
fn message(bytes: &[u8]) -> checkout::Message {
    checkout::Message { title: bytes.into(), body: Box::new([]) }
}

pub(super) fn result(outcome: &checkout::Candidate, most: u32) -> Option<(Box<[u8]>, u8)> {
    let limits = Limits { depth: 4, length: most };
    let mut kept = 0;
    let mut combined = false;
    for member in &outcome.members {
        match member {
            checkout::Combination::Combined { .. } => combined = true,
            checkout::Combination::Conflict { files, .. } => kept = kept.max(files.len()),
        }
    }
    // A common per-list ceiling cuts every long list, retaining their order.
    loop {
        let mut measure = Encoder::measure(&limits);
        encode(&mut measure, outcome, combined, kept);
        if let Ok(length) = measure.measured() {
            let mut writer = Encoder::write(length, &limits);
            encode(&mut writer, outcome, combined, kept);
            return Some((writer.finish(), if combined { 0 } else { 3 }));
        }
        kept = kept.checked_sub(1)?;
    }
}
fn encode(json: &mut Encoder, outcome: &checkout::Candidate, combined: bool, kept: usize) {
    json.object_start();
    if combined {
        json.key(b"combined");
        json.array_start();
        for member in &outcome.members {
            match member {
                checkout::Combination::Combined { member, squash, .. } => {
                    json.object_start();
                    json.key(b"member");
                    json.unsigned(*member);
                    json.key(b"by");
                    json.string(if *squash { b"squash" } else { b"merge" });
                    json.object_end();
                }
                checkout::Combination::Conflict { .. } => {}
            }
        }
        json.array_end();
    }
    let mut conflicts = false;
    for member in &outcome.members {
        match member {
            checkout::Combination::Combined { .. } => {}
            checkout::Combination::Conflict { .. } => conflicts = true,
        }
    }
    if conflicts {
        json.key(b"conflicts");
        json.array_start();
        for member in &outcome.members {
            match member {
                checkout::Combination::Conflict { member, branch, files, cut } => {
                    json.object_start();
                    json.key(b"member");
                    json.unsigned(*member);
                    json.key(b"with");
                    json.string(if *branch { b"branch" } else { b"member" });
                    json.key(b"files");
                    json.array_start();
                    for file in files.iter().take(kept) {
                        json.string(file);
                    }
                    json.array_end();
                    json.key(b"cut");
                    json.boolean(*cut || kept < files.len());
                    json.object_end();
                }
                checkout::Combination::Combined { .. } => {}
            }
        }
        json.array_end();
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
        let (bytes, code) = result(
            &checkout::Candidate {
                members: Box::new([checkout::Combination::Conflict { member: 12, branch: true, files, cut: false }]),
            },
            100,
        )
        .expect("bounded prefix");
        assert_eq!(code, 3);
        assert_eq!(
            bytes.as_ref(),
            br#"{"conflicts":[{"member":12,"with":"branch","files":["aaaaaaaaaaaaaaaaaaaa"],"cut":true}]}"#
        );
        let (bytes, code) = result(
            &checkout::Candidate {
                members: Box::new([checkout::Combination::Conflict {
                    member: 12,
                    branch: true,
                    files: Box::new([Box::from(&b"code"[..])]),
                    cut: true,
                }]),
            },
            4096,
        )
        .expect("git prefix");
        assert_eq!(code, 3);
        assert_eq!(bytes.as_ref(), br#"{"conflicts":[{"member":12,"with":"branch","files":["code"],"cut":true}]}"#);
        let mut members = skein_lib::List::uncapped();
        for member in 1..=5 {
            let mut files = skein_lib::List::uncapped();
            for _ in 0_u32..200 {
                files.push(Box::from(b"a-conflicting-path-long-enough-to-cut".as_slice())).expect("fixture");
            }
            members
                .push(checkout::Combination::Conflict {
                    member,
                    branch: member != 2,
                    files: files.into_boxed(),
                    cut: false,
                })
                .expect("fixture");
        }
        let (bytes, code) =
            result(&checkout::Candidate { members: members.into_boxed() }, 4096).expect("all lists cut");
        assert_eq!(code, 3);
        assert!(bytes.len() <= 4096);
        assert_eq!(skein_lib::bytes::count(&bytes, b"\"cut\":true", 1000), 5);
        let kept = skein_lib::bytes::count(&bytes, b"a-conflicting-path-long-enough-to-cut", 1000);
        assert!(kept > 0 && kept < 1000);
        let combined = checkout::Candidate {
            members: Box::new([checkout::Combination::Combined {
                member: 1,
                commit: checkout::git::Commit::new([0; 32]),
                squash: false,
            }]),
        };
        assert_eq!(result(&combined, 4096).expect("combined").1, 0);
    }
}
