//! One authenticated contents commit (temper/docs/design/client/drive.md, 3.5).
use super::{Encoded, Refusal, items::Target};
use crate::forgejo::{
    limits::Limits,
    request::{FileChange, Method, Request},
};
use skein_json::writer::{self, Encoder};
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
    usize_of,
};

pub(super) fn encode(request: &Request, limits: &Limits) -> Result<Encoded, Refusal> {
    let Request::ChangeFiles { at, branch, message, files } = request else { unreachable!("change-files dispatch") };
    let mut target = Target { length: 0, writer: None };
    target.repo(at);
    target.put(b"/contents");
    if target.length > usize_of(limits.target_bytes) {
        return Err(Refusal::TooLong);
    }
    let mut json = Encoder::measure(&limits.writer);
    body(&mut json, branch, message, files, limits)?;
    let length = match json.measured() {
        Ok(length) => length,
        Err(why) => return Err(refusal(why)),
    };
    let mut json = Encoder::write(length, &limits.writer);
    body(&mut json, branch, message, files, limits)?;
    let mut out = Target { length: 0, writer: Some(Writer::new(target.length)) };
    out.repo(at);
    out.put(b"/contents");
    Ok(Encoded {
        method: Method::Post,
        target: out.writer.expect("measured target").finish(),
        body: Some(json.finish()),
    })
}
fn refusal(why: writer::Refusal) -> Refusal {
    match why {
        writer::Refusal::Text => Refusal::Text,
        writer::Refusal::TooLong | writer::Refusal::TooDeep | writer::Refusal::Number => Refusal::TooLong,
    }
}
fn body(
    json: &mut Encoder,
    branch: &[u8],
    message: &[u8],
    files: &[FileChange],
    limits: &Limits,
) -> Result<(), Refusal> {
    json.object_start();
    json.key(b"branch");
    json.string(branch);
    json.key(b"message");
    json.string(message);
    json.key(b"files");
    json.array_start();
    for file in files {
        let length =
            base64::encoded_len(Alphabet::Standard, u32::try_from(file.content.len()).or(Err(Refusal::TooLong))?)
                .ok_or(Refusal::TooLong)?;
        if length > limits.writer.length {
            return Err(Refusal::TooLong);
        }
        let mut content = Writer::new(usize_of(length));
        base64::encode(Alphabet::Standard, &file.content, &mut content).expect("measured base64");
        json.object_start();
        json.key(b"operation");
        json.string(if file.sha.is_some() { b"update" } else { b"create" });
        json.key(b"path");
        json.string(&file.path);
        if let Some(sha) = &file.sha {
            json.key(b"sha");
            json.string(sha);
        }
        json.key(b"content");
        json.string(&content.finish());
        json.object_end();
    }
    json.array_end();
    json.object_end();
    Ok(())
}
