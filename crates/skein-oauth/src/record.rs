//! Versioned kept token records and refresh state (oauth.md, sections 1 and 2).
//! A record owns access and optional refresh bytes, expiry and selected metadata;
//! it knows no credential source or store. The codecs enforce bounds and version,
//! and `rotate` prepares a candidate the caller keeps before lending.

use crate::{DecodeError, Limits, TokenResponse, common, documents, jwt, v3};
use alloc::boxed::Box;
use skein_lib::{Duration, Reader, Secret, Wall, Writer};

pub const RECORD_VERSION: u16 = 3;
/// Caller-selected nested JWT claim to keep as opaque metadata.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ClaimSelector {
    pub object: Box<[u8]>,
    pub field: Box<[u8]>,
}
/// Metadata needed for Starting or the next refresh; access tokens are optional
/// in the owner's table and are not required to ask for the first refresh.
#[derive(Clone, PartialEq, Eq, Hash)]
#[expect(missing_debug_implementations, reason = "credential values must never occur in traces")]
pub struct RefreshState {
    pub key: u32,
    pub generation: u64,
    pub refresh_token: Box<[u8]>,
    pub issued_client_id: Option<Box<[u8]>>,
    pub granted_scopes: Option<Box<[u8]>>,
    pub subject: Option<Box<[u8]>>,
    pub id_token: Option<Box<[u8]>>,
}
/// Token record its owner keeps before lending; access-only records cannot refresh.
/// `expires_at` is restart metadata, never a cross-host grant deadline.
#[derive(Clone, PartialEq, Eq, Hash)]
#[expect(missing_debug_implementations, reason = "credential values must never occur in traces")]
pub struct SavedToken {
    pub key: u32,
    pub generation: u64,
    pub access_token: Box<[u8]>,
    pub refresh_token: Option<Box<[u8]>>,
    pub metadata: Option<Box<[u8]>>,
    pub expires_at: Wall,
    pub issued_client_id: Option<Box<[u8]>>,
    pub granted_scopes: Option<Box<[u8]>>,
    pub subject: Option<Box<[u8]>>,
    pub id_token: Option<Box<[u8]>>,
    pub earliest_refresh_at: Option<Wall>,
}
impl SavedToken {
    /// Returns refresh state only for a record that holds a refresh token.
    #[must_use]
    #[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
    pub fn refresh_state(&self) -> Option<RefreshState> {
        match &self.refresh_token {
            Some(refresh_token) => Some(RefreshState {
                key: self.key,
                generation: self.generation,
                refresh_token: refresh_token.clone(),
                issued_client_id: self.issued_client_id.clone(),
                granted_scopes: self.granted_scopes.clone(),
                subject: self.subject.clone(),
                id_token: self.id_token.clone(),
            }),
            None => None,
        }
    }
    /// Call at grant encoding or durability completion, not at refresh start.
    #[must_use]
    pub fn remaining(&self, wall: Wall) -> Duration {
        Duration::from_nanos(self.expires_at.as_nanos().saturating_sub(wall.as_nanos()))
    }
}
/// Builds the next candidate; its owner must keep it successfully before
/// granting it. An omitted `refresh_token` preserves the previous token.
pub fn rotate(
    previous: &RefreshState,
    response: &TokenResponse,
    claim: Option<&ClaimSelector>,
    wall: Wall,
    limits: &Limits,
) -> Result<SavedToken, DecodeError> {
    common::bounded(&previous.refresh_token, limits.token_bytes)?;
    common::text(&previous.refresh_token)?;
    validate_refresh_additions(previous, limits)?;
    complete(
        SavedToken {
            key: previous.key,
            generation: previous.generation,
            access_token: Box::new([]),
            refresh_token: Some(previous.refresh_token.clone()),
            metadata: None,
            expires_at: wall,
            issued_client_id: previous.issued_client_id.clone(),
            granted_scopes: previous.granted_scopes.clone(),
            subject: previous.subject.clone(),
            id_token: previous.id_token.clone(),
            earliest_refresh_at: None,
        },
        response,
        claim,
        wall,
        limits,
    )
}
/// Completes a sign-in or refresh candidate with the answer's tokens and expiry.
pub(crate) fn complete(
    mut candidate: SavedToken,
    response: &TokenResponse,
    claim: Option<&ClaimSelector>,
    wall: Wall,
    limits: &Limits,
) -> Result<SavedToken, DecodeError> {
    documents::validate_response(response, limits)?;
    candidate.generation = candidate.generation.checked_add(1).ok_or(DecodeError::TooLarge)?;
    let nanos = response.expires_in.checked_mul(1_000_000_000).ok_or(DecodeError::TooLarge)?;
    let mut expires_at = Wall::from_nanos(wall.as_nanos().checked_add(nanos).ok_or(DecodeError::TooLarge)?);
    candidate.metadata = match claim {
        None => None,
        Some(selector) => {
            let claims = jwt::read_claims(&response.access_token, selector, wall, limits)?;
            if let Some(expiry) = claims.expires_at {
                expires_at = expires_at.min(expiry);
            }
            Some(claims.metadata)
        }
    };
    candidate.expires_at = expires_at;
    candidate.access_token.clone_from(&response.access_token);
    candidate.refresh_token = response.refresh_token.clone().or(candidate.refresh_token);
    candidate.granted_scopes = response.scope.clone().or(candidate.granted_scopes);
    candidate.earliest_refresh_at = response.earliest_refresh_at;
    validate(&candidate, limits)?;
    let _length = measured(&candidate, limits)?;
    Ok(candidate)
}
fn validate(record: &SavedToken, limits: &Limits) -> Result<(), DecodeError> {
    common::bearer(&record.access_token, limits)?;
    if let Some(refresh_token) = &record.refresh_token {
        common::bounded(refresh_token, limits.token_bytes)?;
        common::text(refresh_token)?;
    }
    if let Some(id) = &record.metadata {
        jwt::check_metadata(id, limits)?;
    }
    for (value, cap) in [
        (&record.issued_client_id, limits.issued_client_bytes),
        (&record.granted_scopes, limits.granted_bytes),
        (&record.subject, limits.subject_bytes),
        (&record.id_token, limits.id_token_bytes),
    ] {
        if let Some(value) = value {
            common::optional_text(value, cap)?;
        }
    }
    Ok(())
}
/// Maps runtime limits to the schema's fields and checks its fixed ceilings.
pub(crate) fn record_limits(limits: &Limits) -> Option<v3::Limits> {
    let limits = v3::Limits {
        saved_token_access_token: limits.token_bytes,
        saved_token_refresh_token_some: limits.token_bytes,
        saved_token_metadata_some: limits.token_bytes,
        saved_token_issued_client_id_some: limits.issued_client_bytes,
        saved_token_granted_scopes_some: limits.granted_bytes,
        saved_token_subject_some: limits.subject_bytes,
        saved_token_id_token_some: limits.id_token_bytes,
    };
    v3::check_limits(&limits).ok()?;
    Some(limits)
}

fn problem(problem: v3::Problem) -> DecodeError {
    match problem.reason {
        skein_codec::Reason::Version => DecodeError::Version,
        skein_codec::Reason::Bound => DecodeError::TooLarge,
        skein_codec::Reason::Short
        | skein_codec::Reason::Tag
        | skein_codec::Reason::Bool
        | skein_codec::Reason::Utf8
        | skein_codec::Reason::Trailing
        | skein_codec::Reason::Malformed => DecodeError::Malformed,
    }
}

#[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
fn secret(value: Option<Box<[u8]>>) -> Option<Secret> {
    match value {
        Some(value) => Some(Secret::new(value)),
        None => None,
    }
}

#[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
fn exposed(value: Option<Secret>) -> Option<Box<[u8]>> {
    match value {
        Some(value) => Some(value.into_exposed()),
        None => None,
    }
}

#[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
fn generated(record: &SavedToken, limits: &Limits) -> Result<v3::SavedToken, DecodeError> {
    let schema_limits = record_limits(limits).ok_or(DecodeError::TooLarge)?;
    validate(record, limits)?;
    let parts = v3::SavedTokenParts {
        key: record.key,
        generation: record.generation,
        expires_at: record.expires_at.as_nanos(),
        access_token: Secret::new(record.access_token.clone()),
        refresh_token: secret(record.refresh_token.clone()),
        metadata: record.metadata.clone(),
        issued_client_id: record.issued_client_id.clone(),
        granted_scopes: record.granted_scopes.clone(),
        subject: record.subject.clone(),
        id_token: secret(record.id_token.clone()),
        earliest_refresh_at: match record.earliest_refresh_at {
            Some(wall) => Some(wall.as_nanos()),
            None => None,
        },
    };
    match v3::SavedToken::new(&schema_limits, parts) {
        Ok(value) => Ok(value),
        Err(error) => Err(problem(error)),
    }
}

fn record_length(len: u32, limits: &Limits) -> Result<u32, DecodeError> {
    if len > limits.record_bytes {
        return Err(DecodeError::TooLarge);
    }
    Ok(len)
}

fn measured(record: &SavedToken, limits: &Limits) -> Result<u32, DecodeError> {
    record_length(generated(record, limits)?.measure(), limits)
}

pub fn encode_record(record: &SavedToken, limits: &Limits) -> Result<Box<[u8]>, DecodeError> {
    let value = generated(record, limits)?;
    let len = record_length(value.measure(), limits)?;
    let mut out = Writer::new(usize::try_from(len).expect("u32 fits usize"));
    value.encode(&mut out).expect("record measured");
    Ok(out.finish())
}

#[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
pub fn decode_record(input: &[u8], limits: &Limits) -> Result<SavedToken, DecodeError> {
    if input.len() > usize::try_from(limits.record_bytes).expect("u32 fits usize") {
        return Err(DecodeError::TooLarge);
    }
    let schema_limits = record_limits(limits).ok_or(DecodeError::TooLarge)?;
    let value = match v3::SavedToken::decode(&schema_limits, &mut Reader::new(input)) {
        Ok(value) => value,
        Err(error) => return Err(problem(error)),
    };
    let _length = record_length(value.measure(), limits)?;
    let parts = value.into_parts();
    let record = SavedToken {
        key: parts.key,
        generation: parts.generation,
        expires_at: Wall::from_nanos(parts.expires_at),
        access_token: parts.access_token.into_exposed(),
        refresh_token: exposed(parts.refresh_token),
        metadata: parts.metadata,
        issued_client_id: parts.issued_client_id,
        granted_scopes: parts.granted_scopes,
        subject: parts.subject,
        id_token: exposed(parts.id_token),
        earliest_refresh_at: match parts.earliest_refresh_at {
            Some(nanos) => Some(Wall::from_nanos(nanos)),
            None => None,
        },
    };
    validate(&record, limits)?;
    Ok(record)
}

/// Checks held identity fields before the machine retains or copies them.
pub(crate) fn validate_refresh_additions(state: &RefreshState, limits: &Limits) -> Result<(), DecodeError> {
    for (value, cap) in [
        (&state.issued_client_id, limits.issued_client_bytes),
        (&state.granted_scopes, limits.granted_bytes),
        (&state.subject, limits.subject_bytes),
        (&state.id_token, limits.id_token_bytes),
    ] {
        if let Some(value) = value {
            common::optional_text(value, cap)?;
        }
    }
    Ok(())
}
