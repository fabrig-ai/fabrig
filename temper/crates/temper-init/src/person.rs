//! Forge 3's person (protocol/setup.md, 8.2).
use crate::Person;
use skein_lib::bytes;

// Forgejo 17.0.0, models/user/user.go: reservedUsernames and
// reservedUserPatterns. Forgejo lowercases names before these comparisons;
// each reserved pattern is a leading '*' followed by a literal suffix.
const RESERVED_NAMES: &[&[u8]] = &[
    b".",
    b"..",
    b".well-known",
    b"admin",
    b"api",
    b"assets",
    b"attachments",
    b"avatar",
    b"avatars",
    b"captcha",
    b"commits",
    b"debug",
    b"devtest",
    b"error",
    b"explore",
    b"favicon.ico",
    b"ghost",
    b"issues",
    b"login",
    b"manifest.json",
    b"metrics",
    b"milestones",
    b"new",
    b"notifications",
    b"org",
    b"organizations",
    b"projects",
    b"pulls",
    b"raw",
    b"repo",
    b"repo-avatars",
    b"robots.txt",
    b"search",
    b"serviceworker.js",
    b"ssh_info",
    b"swagger.v1.json",
    b"user",
    b"v2",
];
const RESERVED_PATTERNS: &[&[u8]] = &[b"*.keys", b"*.gpg"];

pub(crate) fn login(found: Option<&[u8]>) -> (&[u8], Person) {
    let Some(login) = found else { return (b"person", Person::FellBack) };
    if !skein_forge_setup::given::login(login) {
        return (b"person", Person::FellBack);
    }
    // The shared grammar bounds the copy to 40 ASCII bytes.
    let mut lower = bytes::copy_of(login);
    lower.make_ascii_lowercase();
    for reserved in RESERVED_NAMES {
        if lower.as_ref() == *reserved {
            return (b"person", Person::FellBack);
        }
    }
    for pattern in RESERVED_PATTERNS {
        if lower.ends_with(pattern.strip_prefix(b"*").expect("each reserved pattern starts with '*'")) {
            return (b"person", Person::FellBack);
        }
    }
    for reserved in [b"temper".as_slice(), b"temper-admin", b"work"] {
        if lower.as_ref() == reserved {
            return (b"person", Person::FellBack);
        }
    }
    (login, Person::Found)
}
