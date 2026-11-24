//! Access-only conversion keeps credential values only while preparing a record.
//! The source login is read-only; no refresh field is deserialized or copied.
//! `codex` writes a guarded private record; smith uses its own sign-in record.
//! `Borrowed::finish` settles the credential and checks the source digest.
//! Contracts: ../../docs/design/benchmarks/isolation.md, section 1; reliability-plan Credentials (95a9807).

use crate::{Bound, Credential, Fault, FaultKind, bound, credential as login, refuse_if};
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use skein_lib::{Duration, Wall};

use crate::files::{CreateNewError, Files, Kind, Native};
use crate::guard::{Guard, check_credential_record};
use crate::isolate::SetupStep;

pub mod logins;

const MAX_AUTH_BYTES: u64 = 128 * 1024;

#[derive(Deserialize)]
struct Source {
    auth_mode: String,
    tokens: Tokens,
}

#[derive(Deserialize, Serialize)]
struct Tokens {
    id_token: String,
    access_token: String,
    account_id: Option<String>,
}

#[derive(Serialize)]
struct AccessOnly<'a> {
    auth_mode: &'a str,
    tokens: AccessTokens<'a>,
    last_refresh: String,
}

#[derive(Serialize)]
struct AccessTokens<'a> {
    id_token: &'a str,
    access_token: &'a str,
    account_id: &'a Option<String>,
    refresh_token: &'static str,
}

#[derive(Deserialize)]
struct Claims {
    exp: u64,
}

/// Nonsecret evidence needed to remove a borrowed file and verify its source.
#[derive(Clone, Debug)]
pub struct Borrowed {
    pub credential: PathBuf,
    pub expires: Wall,
    source: PathBuf,
    digest: String,
}

/// The read-only source and private-file observations retained after settlement.
#[derive(Debug, Serialize)]
#[expect(clippy::struct_excessive_bools, reason = "independent persisted assertions, not execution states")]
pub struct Proof {
    pub source_sha256_before: String,
    pub source_sha256_after: String,
    pub source_unchanged: bool,
    pub expires_unix_seconds: u64,
    pub credential_removed: bool,
    pub private_refresh_empty: bool,
    pub private_refresh_unusable: bool,
}

fn read_source(files: &dyn Files, path: &Path) -> Result<Vec<u8>, Fault> {
    let metadata = files.symlink_metadata(path).map_err(|_| login("Codex auth.json is unavailable"))?;
    if metadata.kind != Kind::File || metadata.size > MAX_AUTH_BYTES {
        return Err(bound("Codex auth.json must be a bounded ordinary file"));
    }
    let bytes = files.read_bounded(path, MAX_AUTH_BYTES + 1).map_err(|_| login("cannot read Codex auth.json"))?;
    refuse_if(skein_lib::len_u64(bytes.len()) > MAX_AUTH_BYTES, Bound, "Codex auth.json exceeds the credential bound")?;
    Ok(bytes)
}

fn expiry(access: &str) -> Result<Wall, Fault> {
    let mut segments = access.split('.');
    let header = segments.next().ok_or_else(|| login("access token has no JWT header"))?;
    let payload = segments.next().ok_or_else(|| login("access token has no JWT claims"))?;
    let signature = segments.next().ok_or_else(|| login("access token has no JWT signature"))?;
    if header.is_empty() || signature.is_empty() || segments.next().is_some() {
        return Err(login("access token has an invalid JWT shape"));
    }
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| login("access token claims are not base64url"))?;
    let claims: Claims = serde_json::from_slice(&bytes).map_err(|_| login("access token has no valid expiry claim"))?;
    let nanos =
        claims.exp.checked_mul(1_000_000_000).ok_or_else(|| bound("access token expiry is outside the wall clock"))?;
    Ok(Wall::from_nanos(nanos))
}

fn write_record(files: &dyn Files, guard: &Guard, home: &Path, name: &str, encoded: &[u8]) -> Result<PathBuf, Fault> {
    let home = crate::isolate::create_directory_with(files, guard, home)?;
    let credential = guard.write_path_with(files, &home.join(name))?;
    let facts = match files.create_new_with_metadata(&credential, 0o600, encoded) {
        Ok(facts) => facts,
        Err(CreateNewError::Open(_)) => return Err(login("cannot create private credential record")),
        Err(CreateNewError::Write(error)) => {
            files
                .remove_file(&guard.write_path_with(files, &credential)?)
                .map_err(|_| login("cannot remove incomplete credential"))?;
            return Err(login(format!("cannot write private credential record: {}", error.kind())));
        }
        Err(CreateNewError::Metadata(_)) => return Err(login("cannot inspect private credential record")),
    };
    if facts.mode & 0o777 != 0o600 {
        files
            .remove_file(&guard.write_path_with(files, &credential)?)
            .map_err(|_| login("cannot remove incorrectly private credential"))?;
        return Err(login("credential record requires mode 0600"));
    }
    Ok(credential)
}

/// Convert only allowed Codex fields into a new private attempt-owned home.
/// `now` is the shell's wall observation; `lifetime` covers the hard deadline.
pub fn codex(guard: &Guard, source_home: &Path, home: &Path, now: Wall, lifetime: Duration) -> Result<Borrowed, Fault> {
    codex_with(&Native, guard, source_home, home, now, lifetime)
}

/// Stage an access-only Codex record on the supplied filesystem.
pub fn codex_with(
    files: &dyn Files,
    guard: &Guard,
    source_home: &Path,
    home: &Path,
    now: Wall,
    lifetime: Duration,
) -> Result<Borrowed, Fault> {
    logins::codex_with(files, source_home)?;
    let converted = convert(files, guard, source_home, home, now, lifetime);
    converted.map_err(|reason| Fault {
        kind: FaultKind::Setup(SetupStep::Credentials),
        detail: format!("{reason}; run `CODEX_HOME={} codex` once before retrying", source_home.display()),
    })
}

fn convert(
    files: &dyn Files,
    guard: &Guard,
    source_home: &Path,
    home: &Path,
    now: Wall,
    lifetime: Duration,
) -> Result<Borrowed, Fault> {
    let source = source_home.join("auth.json");
    let bytes = read_source(files, &source)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let source_auth: Source =
        serde_json::from_slice(&bytes).map_err(|_| login("Codex auth.json has an invalid shape"))?;
    if source_auth.auth_mode.is_empty() || source_auth.tokens.access_token.is_empty() {
        return Err(login("Codex login has no authentication mode or access token"));
    }
    let expires = expiry(&source_auth.tokens.access_token)?;
    let end = now
        .as_nanos()
        .checked_add(lifetime.as_nanos())
        .ok_or_else(|| bound("attempt lifetime exceeds the wall clock"))?;
    refuse_if(expires.as_nanos() <= end, Credential, "Codex access token expires before the attempt ends")?;
    let last_refresh = time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(now.as_nanos()))
        .map_err(|_| login("credential write time is outside the timestamp range"))?
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| login("credential write time cannot be formatted"))?;
    let record = serde_json::to_value(AccessOnly {
        auth_mode: &source_auth.auth_mode,
        tokens: AccessTokens {
            id_token: &source_auth.tokens.id_token,
            access_token: &source_auth.tokens.access_token,
            account_id: &source_auth.tokens.account_id,
            refresh_token: "",
        },
        last_refresh,
    })
    .map_err(|_| login("cannot encode the access-only record"))?;
    check_credential_record(&record)?;
    let encoded = serde_json::to_vec(&record).map_err(|_| login("cannot encode the access-only record"))?;
    let credential = write_record(files, guard, home, "auth.json", &encoded)?;
    Ok(Borrowed { credential, expires, source, digest })
}

fn check_private(files: &dyn Files, credential: &Path) -> Result<(), Fault> {
    let bytes = read_source(files, credential)?;
    let record: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| login("private Codex auth.json changed shape"))?;
    check_credential_record(&record)?;
    if record.pointer("/tokens/refresh_token").and_then(serde_json::Value::as_str) != Some("") {
        return Err(login("private Codex refresh field is no longer empty"));
    }
    Ok(())
}

impl Borrowed {
    /// Remove the private auth file after settlement and verify the user login.
    pub fn finish(&self, guard: &Guard) -> Result<Proof, Fault> {
        self.finish_with(&Native, guard)
    }

    /// Settle the credential and verify its source on the supplied filesystem.
    pub fn finish_with(&self, files: &dyn Files, guard: &Guard) -> Result<Proof, Fault> {
        let checked = check_private(files, &self.credential);
        files
            .remove_file(&guard.write_path_with(files, &self.credential)?)
            .map_err(|_| login("cannot remove private Codex auth.json"))?;
        checked?;
        let after = format!("{:x}", Sha256::digest(read_source(files, &self.source)?));
        if after != self.digest {
            return Err(login(
                "user Codex auth.json changed during the isolated attempt; stop before further inference",
            ));
        }
        Ok(Proof {
            source_sha256_before: self.digest.clone(),
            source_sha256_after: after,
            source_unchanged: true,
            expires_unix_seconds: self.expires.as_secs(),
            credential_removed: true,
            private_refresh_empty: true,
            private_refresh_unusable: true,
        })
    }
}
