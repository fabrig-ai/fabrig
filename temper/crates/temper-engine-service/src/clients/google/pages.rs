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
pub(super) fn page(
    sheet: Option<&[u8]>,
    heading: &[u8],
    paragraph: &[u8],
    then: &[u8],
    choose: bool,
    back: bool,
) -> Box<[u8]> {
    let sheet = match sheet {
        Some(sheet) => join(&[b"<link rel=\"stylesheet\" href=\"", &escape(sheet), b"\">"]),
        None => Box::new([]),
    };
    let target = super::encode(then);
    let start = join(&[b"/sign-in/google?then=", &target, if choose { b"&account=choose" } else { b"" }]);
    let link = escape(&start);
    let then = escape(then);
    let heading = escape(heading);
    let paragraph = escape(paragraph);
    join(&[
        b"<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\"><title>temper</title>",
        &sheet,
        b"</head><body><header><a href=\"/\">temper</a></header><main><h1>",
        &heading,
        b"</h1><p>",
        &paragraph,
        b"</p><a class=\"primary\" href=\"",
        &link,
        b"\">",
        if choose { b"Use another Google account" } else { b"Continue with Google" },
        b"</a>",
        if back { b"<p class=\"quiet\"><a href=\"" } else { b"" },
        if back { &then } else { b"" },
        if back { b"\">Back to sign in</a></p>" } else { b"" },
        b"</main></body></html>",
    ])
}
pub(super) fn not_member(sheet: Option<&[u8]>, address: &[u8], then: &[u8]) -> Box<[u8]> {
    page(sheet, &join(&[address, " can’t sign in here".as_bytes()]), b"Google signed you in, but no one here signs in with this address. Ask an owner to add it, or use the Google account they added.", then, true, true)
}
pub(super) fn another(sheet: Option<&[u8]>, address: &[u8], then: &[u8]) -> Box<[u8]> {
    page(
        sheet,
        &join(&[address, b" signs in here with another Google account"]),
        "This isn’t the Google account that first signed in with this address. Use that one, or ask an owner."
            .as_bytes(),
        then,
        true,
        true,
    )
}
pub(super) fn unverified(sheet: Option<&[u8]>, address: Option<&[u8]>, then: &[u8]) -> Box<[u8]> {
    page(
        sheet,
        &join(&["Google hasn’t verified ".as_bytes(), address.unwrap_or("this account’s address".as_bytes())]),
        b"temper signs in only an address Google has verified. Verify it with Google, or use another account.",
        then,
        true,
        true,
    )
}
pub(super) fn cancelled(sheet: Option<&[u8]>, error: &[u8], then: &[u8]) -> Box<[u8]> {
    let paragraph = if error == b"access_denied" {
        join(&[b"You cancelled at Google. Nothing was kept."])
    } else {
        join(&[b"Google answered ", error.get(..error.len().min(64)).expect("bounded error"), b". Nothing was kept."])
    };
    page(sheet, "The sign-in didn’t finish".as_bytes(), &paragraph, then, false, true)
}
pub(super) fn expired(sheet: Option<&[u8]>, then: &[u8]) -> Box<[u8]> {
    page(
        sheet,
        "The sign-in didn’t finish".as_bytes(),
        b"It took too long, or temper restarted while you were at Google. Nothing was kept.",
        then,
        false,
        true,
    )
}
pub(super) fn unreachable(sheet: Option<&[u8]>, then: &[u8]) -> Box<[u8]> {
    page(
        sheet,
        "The sign-in didn’t finish".as_bytes(),
        "temper couldn’t reach Google. Nothing was kept: try again in a moment.".as_bytes(),
        then,
        false,
        true,
    )
}
pub(super) fn busy(sheet: Option<&[u8]>, then: &[u8]) -> Box<[u8]> {
    page(
        sheet,
        "temper couldn’t sign you in just now".as_bytes(),
        b"It is starting or stopping, or signing many people in at once. Nothing was kept: try again in a moment.",
        then,
        false,
        false,
    )
}
