//! File metadata for contents reads at a commit (skein/docs/design/testing/fakes/forge.md, 4).
use crate::{Limits, response::Reply, routes};
use alloc::boxed::Box;
use skein_fake_forge_domain::api::Error;
use skein_json::writer::Encoder;
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
    len_u64, usize_of,
};

pub(super) fn file(limits: &Limits, path: &[u8], content: &[u8]) -> Result<Reply, Error> {
    let sha = skein_forge::forgejo::id::write(&routes::tree_object(skein_fake_forge_domain::blob_id(content)));
    let length = base64::encoded_len(Alphabet::Standard, u32::try_from(content.len()).or(Err(Error::TooLarge))?)
        .ok_or(Error::TooLarge)?;
    if length > limits.documents.writer.length {
        return Err(Error::TooLarge);
    }
    let mut encoded = Writer::new(usize_of(length));
    base64::encode(Alphabet::Standard, content, &mut encoded).expect("measured base64");
    let encoded = encoded.finish();
    let mut json = Encoder::measure(&limits.documents.writer);
    write(&mut json, path, &sha, &encoded, len_u64(content.len()));
    let length = json.measured().or(Err(Error::TooLarge))?;
    let mut json = Encoder::write(length, &limits.documents.writer);
    write(&mut json, path, &sha, &encoded, len_u64(content.len()));
    Ok(Reply { status: 200, headers: Box::new([]), body: json.finish() })
}
fn write(json: &mut Encoder, path: &[u8], sha: &[u8], content: &[u8], size: u64) {
    json.object_start();
    json.key(b"type");
    json.string(b"file");
    json.key(b"path");
    json.string(path);
    json.key(b"sha");
    json.string(sha);
    json.key(b"size");
    json.unsigned(size);
    json.key(b"encoding");
    json.string(b"base64");
    json.key(b"content");
    json.string(content);
    json.object_end();
}
