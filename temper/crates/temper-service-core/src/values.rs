//! Shared grant sizing (protocol/services.md, 3).
/// A program source name's byte bound (protocol/credentials.md, 3).
pub const SOURCE_NAME_BYTES: usize = 32;

/// Credentials (including keys), a program source name, or a generated identity.
#[must_use]
pub fn grant_bytes(credential: u32, identity: u32) -> Option<u32> {
    Some(credential.max(u32::try_from(SOURCE_NAME_BYTES).ok()?).max(identity))
}

/// A lent credential's bound: a key's is jig's key bound, the token bound capped by the client wire's
/// (jig-protocol-config's `key_bytes`), so the token bound covers both. The wire's own ceiling also covers a
/// moved account's file, which no worker is lent (protocol/credentials.md, 3).
#[must_use]
pub fn token_credential_bytes(token: u32) -> u32 {
    token
}

