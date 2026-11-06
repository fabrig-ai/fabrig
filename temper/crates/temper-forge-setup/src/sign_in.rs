//! Sign-in's private files and printable instructions (protocol/setup.md, 6.8).
use crate::{
    check::joined,
    place::{File, Place},
};
use alloc::boxed::Box;
use skein_lib::{
    Secret, Writer,
    base64::{self, Alphabet},
    bytes,
};

/// What a person's first sign-in through temper does on their forge
/// (temper/docs/design/protocol/sign-in.md, 9.4; temper/docs/design/draft/dogfood-hacks.md, 2.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstSignIn {
    /// Makes their account.
    MakesAccount,
    /// Makes none: an account already there links once, by its password.
    LinksOnly,
    /// A box makes or links by name and address, without a password (sign-in.md, 9.9).
    LinksByName,
}
impl FirstSignIn {
    /// The settings `instructions` prints.
    #[must_use]
    pub const fn settings(self) -> &'static [u8] {
        match self {
            Self::MakesAccount => SETTINGS,
            Self::LinksOnly => LINKS_ONLY_SETTINGS,
            Self::LinksByName => LINKS_BY_NAME_SETTINGS,
        }
    }
}
/// Settings for linking an existing account without registration.
pub const LINKS_ONLY_SETTINGS: &[u8] = b"[oauth2_client]\nENABLE_AUTO_REGISTRATION = false\nUSERNAME = preferred_username\nACCOUNT_LINKING = login\n[service]\nDISABLE_REGISTRATION = true\n";
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
    first: FirstSignIn,
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
        first.settings(),
        b"\n# Once the source works, [service] ENABLE_INTERNAL_SIGNIN = false makes it the web's only sign-in.\n",
    ])
}

/// A box's owned Forgejo alone links the accounts temper made (sign-in.md, 9.9).
pub const LINKS_BY_NAME_SETTINGS: &[u8] = b"[oauth2_client]\nENABLE_AUTO_REGISTRATION = true\nUSERNAME = preferred_username\nACCOUNT_LINKING = auto\n[service]\nDISABLE_REGISTRATION = true\nALLOW_ONLY_EXTERNAL_REGISTRATION = true\n";

/// Names and addresses a box's person may not take through automatic linking.
#[must_use]
pub fn refused_person(login: &[u8], email: &[u8]) -> bool {
    login.eq_ignore_ascii_case(b"temper")
        || login.eq_ignore_ascii_case(crate::spec::ADMIN)
        || login.eq_ignore_ascii_case(b"work")
        || email.eq_ignore_ascii_case(b"temper@temper.invalid")
        || email.eq_ignore_ascii_case(crate::spec::ADMIN_EMAIL)
}

/// The box retains this outcome for the group's next start request (setup.md, 7.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Added,
    Failed(Box<[u8]>),
}
impl Source {
    /// The source command writes exactly one bounded line, without credentials.
    #[must_use]
    pub fn read(line: &[u8]) -> Option<Self> {
        let line = line.strip_suffix(b"\n").unwrap_or(line);
        if line == b"source added" {
            return Some(Self::Added);
        }
        let why = line.strip_prefix(b"source failed ")?;
        if why.is_empty() || why.len() > 1024 {
            return None;
        }
        for byte in why {
            if byte.is_ascii_control() {
                return None;
            }
        }
        Some(Self::Failed(why.into()))
    }
    #[must_use]
    pub fn line(&self) -> Box<[u8]> {
        match self {
            Self::Added => b"source added\n".as_slice().into(),
            Self::Failed(why) => joined(&[b"source failed ", why, b"\n"]),
        }
    }
}

/// Forgejo's auth list is a table: its second column is the exact source name.
#[must_use]
pub fn source_present(table: &[u8]) -> bool {
    let mut rest = table;
    while !rest.is_empty() {
        let end = bytes::find(rest, b"\n").unwrap_or(rest.len());
        let line = rest.get(..end).expect("line boundary");
        let (_, columns) = column(line);
        if column(columns).0 == b"temper" {
            return true;
        }
        rest = rest.get(end..).expect("line boundary").strip_prefix(b"\n").unwrap_or_default();
    }
    false
}

fn column(input: &[u8]) -> (&[u8], &[u8]) {
    let mut rest = input;
    while let Some((byte, tail)) = rest.split_first() {
        if !byte.is_ascii_whitespace() {
            break;
        }
        rest = tail;
    }
    let mut end = 0_usize;
    for byte in rest {
        if byte.is_ascii_whitespace() {
            break;
        }
        end = end.checked_add(1).expect("column within input");
    }
    rest.split_at_checked(end).expect("column within input")
}
