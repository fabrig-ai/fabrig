//! Stock endings (protocol/sign-in.md, 4.3). Every substituted byte is escaped.
use alloc::boxed::Box;
use skein_lib::Writer;
fn escaped(byte: &u8) -> &[u8] {
    match byte {
        b'&' => b"&amp;",
        b'"' => b"&quot;",
        b'\'' => b"&#39;",
        b'<' => b"&lt;",
        b'>' => b"&gt;",
        _ => core::slice::from_ref(byte),
    }
}
fn join(parts: &[&[u8]]) -> Box<[u8]> {
    let mut size = 0_usize;
    for part in parts {
        size = size.checked_add(part.len()).expect("bounded page");
    }
    let mut out = Writer::new(size);
    for part in parts {
        out.put(part).expect("measured page");
    }
    out.finish()
}
fn escape(value: &[u8]) -> Box<[u8]> {
    let mut size = 0_usize;
    for byte in value {
        size = size.checked_add(escaped(byte).len()).expect("bounded page");
    }
    let mut out = Writer::new(size);
    for byte in value {
        out.put(escaped(byte)).expect("measured escaping");
    }
    out.finish()
}
pub(super) fn page(heading: &[u8], paragraph: &[u8], then: &[u8], choose: bool, back: bool) -> Box<[u8]> {
    let target = super::encode(then);
    let start = join(&[b"/sign-in/google?then=", &target, if choose { b"&account=choose" } else { b"" }]);
    let link = escape(&start);
    let then = escape(then);
    let heading = escape(heading);
    let paragraph = escape(paragraph);
    join(&[
        b"<!doctype html><html><head><meta charset=\"utf-8\"><title>temper</title></head><body><main><h1>",
        &heading,
        b"</h1><p>",
        &paragraph,
        b"</p><a href=\"",
        &link,
        b"\">",
        if choose { b"Use another Google account" } else { b"Continue with Google" },
        b"</a>",
        if back { b" <a href=\"" } else { b"" },
        if back { &then } else { b"" },
        if back { b"\">Back to sign in</a>" } else { b"" },
        b"</main></body></html>",
    ])
}
pub(super) fn not_member(address: &[u8], then: &[u8]) -> Box<[u8]> {
    page(&join(&[address, b" can't sign in here"]), b"Google signed you in, but no one here signs in with this address. Ask an owner to add it, or use the Google account they added.", then, true, true)
}
pub(super) fn another(address: &[u8], then: &[u8]) -> Box<[u8]> {
    page(
        &join(&[address, b" signs in here with another Google account"]),
        b"This isn't the Google account that first signed in with this address. Use that one, or ask an owner.",
        then,
        true,
        true,
    )
}
pub(super) fn unverified(address: Option<&[u8]>, then: &[u8]) -> Box<[u8]> {
    page(
        &join(&[b"Google hasn't verified ", address.unwrap_or(b"this account's address")]),
        b"temper signs in only an address Google has verified. Verify it with Google, or use another account.",
        then,
        true,
        true,
    )
}
pub(super) fn cancelled(error: &[u8], then: &[u8]) -> Box<[u8]> {
    let paragraph = if error == b"access_denied" {
        join(&[b"You cancelled at Google. Nothing was kept."])
    } else {
        join(&[b"Google answered ", error.get(..error.len().min(64)).expect("bounded error"), b". Nothing was kept."])
    };
    page(b"The sign-in didn't finish", &paragraph, then, false, true)
}
pub(super) fn expired(then: &[u8]) -> Box<[u8]> {
    page(
        b"The sign-in didn't finish",
        b"It took too long, or temper restarted while you were at Google. Nothing was kept.",
        then,
        false,
        true,
    )
}
pub(super) fn unreachable(then: &[u8]) -> Box<[u8]> {
    page(
        b"The sign-in didn't finish",
        b"temper couldn't reach Google. Nothing was kept: try again in a moment.",
        then,
        false,
        true,
    )
}
pub(super) fn busy(then: &[u8]) -> Box<[u8]> {
    page(
        b"temper couldn't sign you in just now",
        b"It is starting or stopping, or signing many people in at once. Nothing was kept: try again in a moment.",
        then,
        false,
        false,
    )
}
