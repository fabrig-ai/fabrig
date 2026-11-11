//! Step tests (net/push.md, 6). RFC 8291 Appendix A's public test key in OpenSSL PKCS#8 form.
#![expect(clippy::disallowed_types, reason = "tests edit owned requests and collect decoded token parts")]
#![expect(clippy::disallowed_methods, reason = "tests inspect bounded wire fixtures using ordinary Rust")]
use crate::*;
use alloc::{boxed::Box, vec::Vec};
use skein_lib::{
    Wall,
    base64::{self, Alphabet},
};
mod endpoint;
mod request;
const PEM: &[u8] = b"-----BEGIN PRIVATE KEY-----\nMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgyfWPiYE+n46HLnH0KqZOF1fJJU3MYrct3AELtAQ+oRyhRANCAAT+M/SrDepxkU21WCP3O1SUj0EwbZIHMtu5pZpTKGSCIA5Zent7wmC6HCJ5mFgJkuk5cwAvMBKiiujwa7t45ewP\n-----END PRIVATE KEY-----\n";
const UA: &[u8] = b"BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4";
const WALL: Wall = Wall::from_nanos(1_700_000_000_000_000_000);
fn subscription() -> Subscription {
    Subscription {
        endpoint: Box::from(b"https://fcm.googleapis.com/fcm/send/abc".as_slice()),
        p256dh: base64::decode(Alphabet::Url, UA, 65).expect("RFC ua").as_ref().try_into().expect("point"),
        auth: [1; 16],
    }
}
fn push() -> Push<'static> {
    Push { payload: b"hello", ttl: 60, urgency: Urgency::High, topic: Some(b"updates_1") }
}
