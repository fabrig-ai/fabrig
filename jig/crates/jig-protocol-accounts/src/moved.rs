//! A moved record's file (protocol/llm.md, 5.6): one account, opaque and versioned.
use crate::v1;
use alloc::boxed::Box;
use jig_domain_accounts::{DISPLAY, SUBJECT};
use skein_codec::Reason;
use skein_lib::{Reader, Secret, Wall, Writer, bytes};
use skein_oauth::{DecodeError, Limits, SavedToken};

/// The application's name, in bytes (nonempty).
pub const APPLICATION: u32 = 64;
/// The issuer URL, in bytes (nonempty).
pub const ISSUER: u32 = 2_048;
/// The refreshing client id, in bytes (nonempty).
pub const CLIENT_ID: u32 = 255;

/// A moved record's parts; holds a secret, so no Debug.
#[expect(missing_debug_implementations, reason = "carries a refresh token")]
pub struct Moved {
    pub application: Box<[u8]>,
    pub issuer: Box<[u8]>,
    pub client_id: Box<[u8]>,
    pub subject: Box<[u8]>,
    pub display: Box<[u8]>,
    pub record: SavedToken,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unread {
    Version,
    Malformed,
    TooLarge,
    Trailing,
    Record,
}

fn wire_limits(limits: &Limits) -> v1::Limits {
    // A runtime allowance above the schema ceiling cannot admit larger records.
    v1::Limits { moved_record: limits.record_bytes.min(v1::CEILINGS.moved_record), ..v1::CEILINGS }
}

/// The largest file under these limits, including its fixed parts.
#[must_use]
pub fn bound(limits: &Limits) -> u64 {
    // The JIGM header and six length prefixes take 30 bytes (four more
    // than the generated form); its record allowance has no schema ceiling.
    let mut size = 30_u64;
    for cap in [APPLICATION, ISSUER, CLIENT_ID, SUBJECT, DISPLAY, limits.record_bytes] {
        size = size.checked_add(u64::from(cap)).expect("six u32 bounds fit u64");
    }
    size
}

/// Encodes bounded parts and a record skein accepts, allocating the final size.
pub fn encode(moved: &Moved, limits: &Limits) -> Result<Secret, Unread> {
    let wire_limits = wire_limits(limits);
    // Refuse before copying the caller's parts into the generated value.
    for (field, cap) in [
        (moved.application.as_ref(), wire_limits.moved_application),
        (moved.issuer.as_ref(), wire_limits.moved_issuer),
        (moved.client_id.as_ref(), wire_limits.moved_client_id),
        (moved.subject.as_ref(), wire_limits.moved_subject),
        (moved.display.as_ref(), wire_limits.moved_display),
    ] {
        if field.len() > usize::try_from(cap).expect("schema ceiling fits usize") {
            return Err(Unread::TooLarge);
        }
    }
    required(&[&moved.application, &moved.issuer, &moved.client_id, &moved.subject])?;
    let record = Secret::new(skein_oauth::encode_record(&moved.record, limits).or(Err(Unread::Record))?);
    let parts = v1::MovedParts {
        application: bytes::copy_of(&moved.application),
        issuer: bytes::copy_of(&moved.issuer),
        client_id: bytes::copy_of(&moved.client_id),
        subject: bytes::copy_of(&moved.subject),
        display: bytes::copy_of(&moved.display),
        record,
    };
    let value = match v1::Moved::new(&wire_limits, parts) {
        Ok(value) => value,
        Err(problem) => return Err(unread(problem)),
    };
    let mut out = Writer::new(usize::try_from(value.measure()).or(Err(Unread::TooLarge))?);
    value.encode(&mut out).expect("file measured");
    Ok(Secret::new(out.finish()))
}

/// Decodes a whole file, checking its size before reading or allocating parts.
pub fn decode(input: &[u8], limits: &Limits) -> Result<Moved, Unread> {
    if u64::try_from(input.len()).or(Err(Unread::TooLarge))? > bound(limits) {
        return Err(Unread::TooLarge);
    }
    if input.starts_with(b"JIGM") {
        return decode_old(input, limits);
    }
    let value = match v1::Moved::decode(&wire_limits(limits), &mut Reader::new(input)) {
        Ok(value) => value,
        Err(problem) => return Err(unread(problem)),
    };
    required(&[value.application(), value.issuer(), value.client_id(), value.subject(), value.record().expose()])?;
    let record = skein_oauth::decode_record(value.record().expose(), limits).or(Err(Unread::Record))?;
    let parts = value.into_parts();
    Ok(Moved {
        application: parts.application,
        issuer: parts.issuer,
        client_id: parts.client_id,
        subject: parts.subject,
        display: parts.display,
        record,
    })
}

/// Temporary JIGM reader (decision 1345): until the held logins are
/// re-exported, before the release. Its nested SKOT reader goes with it.
/// Encoding always uses the generated form.
fn decode_old(input: &[u8], limits: &Limits) -> Result<Moved, Unread> {
    let mut version = Reader::new(input.get(4..6).ok_or(Unread::Malformed)?);
    if version.u16().ok_or(Unread::Malformed)? != 1 {
        return Err(Unread::Version);
    }
    let mut rest = input.get(6..).ok_or(Unread::Malformed)?;
    let mut parts = [b"".as_slice(); 6];
    for (part, (cap, empty)) in parts.iter_mut().zip([
        (APPLICATION, false),
        (ISSUER, false),
        (CLIENT_ID, false),
        (SUBJECT, false),
        (DISPLAY, true),
        (limits.record_bytes, false),
    ]) {
        // Reader's cursor is limited to u32::MAX, while the whole file can
        // exceed that with record_bytes at its max. Read only the prefix.
        let mut reader = Reader::new(rest.get(..4).ok_or(Unread::Malformed)?);
        let len = reader.u32().ok_or(Unread::Malformed)?;
        if len > cap {
            return Err(Unread::TooLarge);
        }
        if len == 0 && !empty {
            return Err(Unread::Malformed);
        }
        (*part, rest) = rest
            .get(4..)
            .ok_or(Unread::Malformed)?
            .split_at_checked(usize::try_from(len).or(Err(Unread::TooLarge))?)
            .ok_or(Unread::Malformed)?;
    }
    if !rest.is_empty() {
        return Err(Unread::Trailing);
    }
    let [application, issuer, client_id, subject, display, record] = parts;
    let record = if record.starts_with(b"SKOT") {
        decode_skot(record, limits)
    } else {
        skein_oauth::decode_record(record, limits)
    }
    .or(Err(Unread::Record))?;
    Ok(Moved {
        application: bytes::copy_of(application),
        issuer: bytes::copy_of(issuer),
        client_id: bytes::copy_of(client_id),
        subject: bytes::copy_of(subject),
        display: bytes::copy_of(display),
        record,
    })
}

/// Temporary nested reader for JIGM (decision 1345), matching skein cad332f2.
/// Remove with `decode_old` after held logins are re-exported, before release.
fn decode_skot(input: &[u8], limits: &Limits) -> Result<SavedToken, DecodeError> {
    if input.len() > usize::try_from(limits.record_bytes).expect("u32 fits usize") {
        return Err(DecodeError::TooLarge);
    }
    let mut reader = Reader::new(input);
    if reader.bytes(4).ok_or(DecodeError::Malformed)? != b"SKOT" {
        return Err(DecodeError::Malformed);
    }
    let version = reader.u16().ok_or(DecodeError::Malformed)?;
    if version != 2 && version != 3 {
        return Err(DecodeError::Version);
    }
    let key = reader.u32().ok_or(DecodeError::Malformed)?;
    let generation = reader.u64().ok_or(DecodeError::Malformed)?;
    let expires_at = Wall::from_nanos(reader.u64().ok_or(DecodeError::Malformed)?);
    let access_token = read_field(&mut reader, limits.token_bytes)?;
    let refresh = read_field(&mut reader, limits.token_bytes)?;
    let refresh_token = if refresh.is_empty() { None } else { Some(refresh) };
    let id = read_field(&mut reader, limits.token_bytes)?;
    let metadata = if id.is_empty() { None } else { Some(id) };
    let (issued_client_id, granted_scopes, subject, id_token, earliest_refresh_at) = if version == 2 {
        (None, None, None, None, None)
    } else {
        let issued = read_optional(&mut reader, limits.issued_client_bytes)?;
        let granted = read_optional(&mut reader, limits.granted_bytes)?;
        let subject = read_optional(&mut reader, limits.subject_bytes)?;
        let id = read_optional(&mut reader, limits.id_token_bytes)?;
        let earliest = if presence(&mut reader)? {
            Some(Wall::from_nanos(reader.u64().ok_or(DecodeError::Malformed)?))
        } else {
            None
        };
        (issued, granted, subject, id, earliest)
    };
    if !reader.is_empty() {
        return Err(DecodeError::Malformed);
    }
    let record = SavedToken {
        key,
        generation,
        access_token,
        refresh_token,
        metadata,
        expires_at,
        issued_client_id,
        granted_scopes,
        subject,
        id_token,
        earliest_refresh_at,
    };
    // Current validation also enforces schema ceilings and the encoded size.
    let encoded = Secret::new(skein_oauth::encode_record(&record, limits)?);
    skein_oauth::decode_record(encoded.expose(), limits)
}
fn read_field(reader: &mut Reader<'_>, cap: u32) -> Result<Box<[u8]>, DecodeError> {
    let len = reader.u32().ok_or(DecodeError::Malformed)?;
    if len > cap {
        return Err(DecodeError::TooLarge);
    }
    let value = reader.bytes(len).ok_or(DecodeError::Malformed)?;
    Ok(bytes::copy_of(value))
}

fn presence(reader: &mut Reader<'_>) -> Result<bool, DecodeError> {
    match reader.bytes(1).ok_or(DecodeError::Malformed)? {
        [0] => Ok(false),
        [1] => Ok(true),
        _ => Err(DecodeError::Malformed),
    }
}
fn read_optional(reader: &mut Reader<'_>, cap: u32) -> Result<Option<Box<[u8]>>, DecodeError> {
    if presence(reader)? { Ok(Some(read_field(reader, cap)?)) } else { Ok(None) }
}

fn required(fields: &[&[u8]]) -> Result<(), Unread> {
    for field in fields {
        if field.is_empty() {
            return Err(Unread::Malformed);
        }
    }
    Ok(())
}

fn unread(problem: v1::Problem) -> Unread {
    match problem.reason {
        Reason::Version => Unread::Version,
        Reason::Bound => Unread::TooLarge,
        Reason::Trailing => Unread::Trailing,
        Reason::Short | Reason::Tag | Reason::Bool | Reason::Utf8 | Reason::Malformed => Unread::Malformed,
    }
}
