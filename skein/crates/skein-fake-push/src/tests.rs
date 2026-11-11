//! Step checks of testing/fakes/push.md, 3, one fault per request.
#![expect(clippy::disallowed_types, reason = "tests edit bounded request fixtures")]
#![expect(clippy::disallowed_methods, reason = "tests inspect and edit bounded request fixtures using ordinary Rust")]
use crate::{RECEIVED, Request, Script, Service};
use alloc::{boxed::Box, vec::Vec};
use skein_http::Header;
use skein_lib::{
    Wall, Writer,
    base64::{self, Alphabet},
};
use skein_push::{Push, Subscription, Urgency, Vapid};
mod checks;
mod records;
// RFC 8291 Appendix A's public test key, in OpenSSL PKCS#8 v1 form.
const PEM: &[u8] = b"-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgyfWPiYE+n46HLnH0KqZOF1fJJU3MYrct3AELtAQ+oRyhRANCAAT+M/SrDepxkU21WCP3O1SUj0EwbZIHMtu5pZpTKGSCIA5Zent7wmC6HCJ5mFgJkuk5cwAvMBKiiujwa7t45ewP\n-----END PRIVATE KEY-----\n";
const ORIGIN: &[u8] = b"http://push.test:4000";
const WALL: Wall = Wall::from_nanos(1_700_000_000_000_000_000);
const GOOD: &[u8] = b"{\"aud\":\"http://push.test:4000\",\"exp\":1700043200,\"sub\":\"mailto:p@example.test\"}";
fn fixture() -> (Service, Request) {
    let vapid = Vapid::from_pkcs8_pem(PEM).expect("public RFC key");
    let p256dh = *vapid.public();
    let subscription =
        Subscription { endpoint: Box::from(b"http://push.test:4000/push/alice".as_slice()), p256dh, auth: [2; 16] };
    let sealed = skein_push::seal(
        &vapid,
        b"mailto:p@example.test",
        &subscription,
        &Push { payload: b"hello", ttl: 60, urgency: Urgency::High, topic: Some(b"updates") },
        WALL,
    )
    .expect("sealed request");
    let headers: Vec<Header> = sealed.headers.into_iter().map(|(name, value)| Header { name, value }).collect();
    (
        Service::new(Box::from(ORIGIN), p256dh),
        Request {
            method: Box::from(b"POST".as_slice()),
            target: sealed.target,
            headers: headers.into_boxed_slice(),
            body: sealed.body,
        },
    )
}
fn field(request: &mut Request, name: &[u8], value: &[u8]) {
    let header = request.headers.iter_mut().find(|header| header.is(name)).expect("header fixture");
    header.value = Box::from(value);
}
fn encoded(bytes: &[u8]) -> Box<[u8]> {
    let len = base64::encoded_len(Alphabet::Url, u32::try_from(bytes.len()).expect("bounded")).expect("encoded length");
    let mut out = Writer::new(skein_lib::usize_of(len));
    base64::encode(Alphabet::Url, bytes, &mut out).expect("encoding");
    out.finish()
}
fn authorization(header: &[u8], claims: &[u8], corrupt: bool) -> Box<[u8]> {
    let pem = core::str::from_utf8(PEM).expect("pem");
    let der = base64::decode(Alphabet::Standard, pem.lines().nth(1).expect("key body").as_bytes(), 256).expect("DER");
    let key = skein_tls::es256_public(&der).expect("key");
    let mut input = Vec::from(encoded(header).as_ref());
    input.push(b'.');
    input.extend_from_slice(&encoded(claims));
    let mut signature = skein_tls::sign_es256(&der, &input).expect("sign");
    if corrupt {
        signature[0] ^= 1;
    }
    let mut value = Vec::from(b"vapid t=".as_slice());
    value.extend_from_slice(&input);
    value.push(b'.');
    value.extend_from_slice(&encoded(&signature));
    value.extend_from_slice(b", k=");
    value.extend_from_slice(&encoded(&key));
    value.into_boxed_slice()
}
