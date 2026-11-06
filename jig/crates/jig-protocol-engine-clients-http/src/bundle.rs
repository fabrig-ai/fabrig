//! Bundle helpers (jig/docs/design/protocol/service.md, 11.4; skein/docs/design/ui-web.md, 4.4).
use crate::server::{Routed, Server};
use alloc::boxed::Box;
use skein_http::{Header, server as http};
use skein_lib::stream::{Down, Read};
use skein_lib::{List, Map, Queue, Token, bytes};
/// One digest-directory file, typed by the closed extension list.
#[derive(Debug)]
pub struct File {
    pub kind: &'static [u8],
    pub bytes: Box<[u8]>,
    pub gzip: Option<Box<[u8]>>,
}
/// The file a compressed form belongs to (skein/docs/design/ui-web.md, 4.4).
#[must_use]
pub fn compressed_of(path: &[u8]) -> Option<&[u8]> {
    let plain = path.strip_suffix(b".gz")?;
    if plain.ends_with(b".wasm") || plain.ends_with(b".js") || plain.ends_with(b".css") || plain.ends_with(b".svg") {
        Some(plain)
    } else {
        None
    }
}
#[must_use]
pub fn kind(path: &[u8]) -> Option<&'static [u8]> {
    let mut dot = None;
    for (at, byte) in path.iter().enumerate() {
        if *byte == b'.' {
            dot = Some(at);
        }
        if *byte == b'/' {
            dot = None;
        }
    }
    let dot = dot?;
    match path.get(dot.checked_add(1)?..)? {
        b"html" => Some(b"text/html; charset=utf-8"),
        b"js" => Some(b"text/javascript"),
        b"wasm" => Some(b"application/wasm"),
        b"css" => Some(b"text/css"),
        b"svg" => Some(b"image/svg+xml"),
        b"png" => Some(b"image/png"),
        b"webp" => Some(b"image/webp"),
        b"ico" => Some(b"image/x-icon"),
        b"woff2" => Some(b"font/woff2"),
        b"txt" => Some(b"text/plain; charset=utf-8"),
        b"webmanifest" => Some(b"application/manifest+json"),
        _ => None,
    }
}
#[must_use]
pub fn target(call: &http::Call) -> &[u8] {
    match bytes::find(&call.target, b"?") {
        Some(at) => call.target.get(..at).expect("query position"),
        None => &call.target,
    }
}
#[must_use]
pub fn file<'a>(files: &'a Map<Box<[u8]>, File>, digest: &[u8], path: &[u8]) -> Option<&'a File> {
    let name = if path == b"/robots.txt" {
        b"files/robots.txt".as_slice()
    } else {
        let rest = path.strip_prefix(b"/bundle/")?;
        let slash = bytes::find(rest, b"/")?;
        if rest.get(..slash)? != digest {
            return None;
        }
        rest.get(slash.checked_add(1)?..)?
    };
    files.get(name)
}
pub fn field(headers: &mut List<Header>, name: &[u8], value: &[u8]) {
    headers.push(Header { name: bytes::copy_of(name), value: bytes::copy_of(value) }).expect("answer fields");
}
pub fn demand(server: &mut Server, call: Token, left: usize, send: u32, out: &mut Queue<Routed>) {
    let room = send.min(u32::try_from(left).expect("bounded file"));
    server.answer(call, http::Request::Reply(Down::Demand { read: Read::Nothing, room }), out);
}
pub fn chunk(server: &mut Server, call: Token, body: &[u8], start: usize, send: u32, out: &mut Queue<Routed>) -> usize {
    let end = start.saturating_add(skein_lib::length::usize_of(send)).min(body.len());
    let bytes = bytes::copy_of(body.get(start..end).expect("reply slice"));
    let left = body.len().checked_sub(end).expect("reply cursor");
    server.answer(call, http::Request::Reply(Down::Send(bytes)), out);
    if left == 0 {
        server.answer(call, http::Request::Reply(Down::Finish), out);
    } else {
        demand(server, call, left, send, out);
    }
    end
}
#[cfg(test)]
mod tests;
