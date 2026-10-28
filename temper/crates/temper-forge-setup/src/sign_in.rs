//! Sign-in's private files and printable instructions (protocol/setup.md, 6.8).
use crate::{
    check::joined,
    place::{File, Place},
};
use alloc::boxed::Box;
use skein_lib::{
    Secret, Writer,
    base64::{self, Alphabet},
};

/// Settings already written by an owned forge's preparation.
pub const SETTINGS: &[u8] = b"[oauth2_client]\nENABLE_AUTO_REGISTRATION = true\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n[service]\nDISABLE_REGISTRATION = true\nALLOW_ONLY_EXTERNAL_REGISTRATION = true\n";
/// The key generator's arguments; its stdout goes only to a private partial file.
pub const KEY_ARGUMENTS: &[&[u8]] = &[b"genpkey", b"-algorithm", b"RSA", b"-pkeyopt", b"rsa_keygen_bits:3072"];

/// Encode exactly 32 bytes drawn by the host, without padding or a newline.
#[must_use]
pub fn secret(random: &[u8; 32]) -> Secret {
    let mut out = Writer::new(43);
    base64::encode(Alphabet::Url, random, &mut out).expect("32 bytes encode to 43 characters");
    Secret::new(out.finish())
}

/// A POSIX shell word, including paths containing apostrophes or substitutions.
#[must_use]
pub fn quote(value: &[u8]) -> Box<[u8]> {
    let mut size = value.len().checked_add(2).expect("bounded word");
    for byte in value {
        if *byte == b'\'' {
            size = size.checked_add(3).expect("bounded word");
        }
    }
    let mut out = Writer::new(size);
    out.put(b"'").expect("measured word");
    for byte in value {
        out.put(if *byte == b'\'' { b"'\\''" } else { core::slice::from_ref(byte) }).expect("measured word");
    }
    out.put(b"'").expect("measured word");
    out.finish()
}

/// The command takes only a secret's path: its contents cannot reach these bytes.
#[must_use]
pub fn instructions(
    program: &[u8],
    place: Option<&Place>,
    client: &[u8],
    secret_path: &[u8],
    discovery: &[u8],
) -> Box<[u8]> {
    let owned = match place {
        Some(place) => {
            joined(&[b" --work-path ", &quote(&place.work_path()), b" --config ", &quote(&place.path(File::AppIni))])
        }
        None => Box::new([]),
    };
    joined(&[
        &quote(program),
        &owned,
        b" admin auth add-oauth --name temper --provider openidConnect --key ",
        &quote(client),
        b" --secret \"$(cat ",
        &quote(secret_path),
        b")\" --auto-discover-url ",
        &quote(discovery),
        b"\n\n# Forgejo app.ini settings:\n",
        SETTINGS,
        b"\n# Once the source works, [service] ENABLE_INTERNAL_SIGNIN = false makes it the web's only sign-in.\n",
    ])
}
