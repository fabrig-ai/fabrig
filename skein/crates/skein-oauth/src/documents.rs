use crate::{DecodeError, Json, Limits, common, json};
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::{Secret, Wall, bytes, usize_of};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RefreshRequest {
    pub client_id: Box<[u8]>,
    pub refresh_token: Secret,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TokenResponse {
    pub access_token: Secret,
    pub refresh_token: Option<Secret>,
    pub expires_in: u64,
    pub scope: Option<Box<[u8]>>,
    pub id_token: Option<Secret>,
    pub earliest_refresh_at: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct OAuthError {
    pub code: Box<[u8]>,
    pub detail: Box<[u8]>,
}

pub fn encode_request(request: &RefreshRequest, limits: &Limits) -> Result<Box<[u8]>, DecodeError> {
    validate_request(request, limits)?;
    let mut measure = Encoder::measure(&limits.writer_limits());
    write_request(&mut measure, request);
    let len = common::measured(measure)?;
    let mut out = Encoder::write(len, &limits.writer_limits());
    write_request(&mut out, request);
    Ok(out.finish())
}
fn write_request(out: &mut Encoder, request: &RefreshRequest) {
    out.object_start();
    out.key(b"grant_type");
    out.string(b"refresh_token");
    out.key(b"client_id");
    out.string(&request.client_id);
    out.key(b"refresh_token");
    out.string(request.refresh_token.expose());
    out.object_end();
}
fn validate_request(request: &RefreshRequest, limits: &Limits) -> Result<(), DecodeError> {
    common::bounded(&request.client_id, limits.client_bytes)?;
    common::bounded(request.refresh_token.expose(), limits.token_bytes)?;
    common::text(request.refresh_token.expose())
}
pub fn decode_request(value: &Json, limits: &Limits) -> Result<RefreshRequest, DecodeError> {
    value.admit(limits)?;
    let tokens = value.as_tokens();
    if json::text_ref(json::value_at(tokens, json::required(tokens, b"grant_type")?)?)? != b"refresh_token" {
        return Err(DecodeError::Malformed);
    }
    let client = json::text_ref(json::value_at(tokens, json::required(tokens, b"client_id")?)?)?;
    let refresh = json::text_ref(json::value_at(tokens, json::required(tokens, b"refresh_token")?)?)?;
    common::bounded(client, limits.client_bytes)?;
    common::bounded(refresh, limits.token_bytes)?;
    Ok(RefreshRequest { client_id: bytes::copy_of(client), refresh_token: Secret::new(bytes::copy_of(refresh)) })
}
pub fn encode_response(response: &TokenResponse, limits: &Limits) -> Result<Box<[u8]>, DecodeError> {
    validate_response(response, limits)?;
    let mut measure = Encoder::measure(&limits.writer_limits());
    write_response(&mut measure, response);
    let len = common::measured(measure)?;
    let mut out = Encoder::write(len, &limits.writer_limits());
    write_response(&mut out, response);
    Ok(out.finish())
}
fn write_response(out: &mut Encoder, response: &TokenResponse) {
    out.object_start();
    out.key(b"access_token");
    out.string(response.access_token.expose());
    out.key(b"token_type");
    out.string(b"Bearer");
    if let Some(refresh) = &response.refresh_token {
        out.key(b"refresh_token");
        out.string(refresh.expose());
    }
    out.key(b"expires_in");
    out.unsigned(response.expires_in);
    for (name, value) in [
        (b"scope".as_slice(), response.scope.as_deref()),
        (b"id_token".as_slice(), crate::record::secret_ref(response.id_token.as_ref())),
    ] {
        if let Some(value) = value {
            out.key(name);
            out.string(value);
        }
    }
    if let Some(wall) = response.earliest_refresh_at {
        out.key(b"earliest_refresh_at");
        out.unsigned(wall.as_nanos() / 1_000_000_000);
    }
    out.object_end();
}
pub(crate) fn validate_response(response: &TokenResponse, limits: &Limits) -> Result<(), DecodeError> {
    common::bearer(response.access_token.expose(), limits)?;
    if let Some(refresh) = &response.refresh_token {
        common::bounded(refresh.expose(), limits.token_bytes)?;
        common::text(refresh.expose())?;
    }
    for (value, cap) in [
        (response.scope.as_deref(), limits.granted_bytes),
        (crate::record::secret_ref(response.id_token.as_ref()), limits.id_token_bytes),
    ] {
        if let Some(value) = value {
            common::optional_text(value, cap)?;
        }
    }
    if response.expires_in == 0 {
        return Err(DecodeError::Malformed);
    }
    Ok(())
}
#[expect(clippy::manual_map, reason = "programming-model.md, 10.3 forbids closures in step code")]
pub fn decode_response(value: &Json, limits: &Limits) -> Result<TokenResponse, DecodeError> {
    value.admit(limits)?;
    let tokens = value.as_tokens();
    if let Some(token_type) = json::optional_at(tokens, json::field(tokens, b"token_type")?)?
        && !json::text_ref(token_type)?.eq_ignore_ascii_case(b"Bearer")
    {
        return Err(DecodeError::WrongType);
    }
    let access = json::text_ref(json::value_at(tokens, json::required(tokens, b"access_token")?)?)?;
    common::bearer(access, limits)?;
    let refresh_token = match json::optional_at(tokens, json::field(tokens, b"refresh_token")?)? {
        Some(value) => {
            let refresh = json::text_ref(value)?;
            common::bounded(refresh, limits.token_bytes)?;
            Some(Secret::new(bytes::copy_of(refresh)))
        }
        None => None,
    };
    let response = TokenResponse {
        access_token: Secret::new(bytes::copy_of(access)),
        refresh_token,
        expires_in: json::unsigned(json::value_at(tokens, json::required(tokens, b"expires_in")?)?)?,
        scope: optional_text(tokens, b"scope", limits.granted_bytes)?,
        id_token: match optional_text(tokens, b"id_token", limits.id_token_bytes)? {
            Some(value) => Some(Secret::new(value)),
            None => None,
        },
        earliest_refresh_at: earliest(tokens)?,
    };
    validate_response(&response, limits)?;
    Ok(response)
}
pub fn encode_error(error: &OAuthError, limits: &Limits) -> Result<Box<[u8]>, DecodeError> {
    common::bounded(&error.code, limits.detail_bytes)?;
    if error.detail.len() > usize_of(limits.detail_bytes) {
        return Err(DecodeError::TooLarge);
    }
    let mut measure = Encoder::measure(&limits.writer_limits());
    write_error(&mut measure, error);
    let len = common::measured(measure)?;
    let mut out = Encoder::write(len, &limits.writer_limits());
    write_error(&mut out, error);
    Ok(out.finish())
}
fn write_error(out: &mut Encoder, error: &OAuthError) {
    out.object_start();
    out.key(b"error");
    out.string(&error.code);
    out.key(b"error_description");
    out.string(&error.detail);
    out.object_end();
}
pub fn decode_error(value: &Json, limits: &Limits) -> Result<OAuthError, DecodeError> {
    value.admit(limits)?;
    let tokens = value.as_tokens();
    let code = json::text_ref(json::value_at(tokens, json::required(tokens, b"error")?)?)?;
    common::bounded(code, limits.detail_bytes)?;
    let detail = match json::optional_at(tokens, json::field(tokens, b"error_description")?)? {
        Some(value) => common::clipped(json::text_ref(value)?, limits.detail_bytes),
        None => bytes::copy_of(b""),
    };
    Ok(OAuthError { code: bytes::copy_of(code), detail })
}

fn optional_text(tokens: &[skein_json::Token], name: &[u8], cap: u32) -> Result<Option<Box<[u8]>>, DecodeError> {
    match json::optional_at(tokens, json::field(tokens, name)?)? {
        Some(value) => {
            let value = json::text_ref(value)?;
            common::optional_text(value, cap)?;
            Ok(Some(bytes::copy_of(value)))
        }
        None => Ok(None),
    }
}
fn earliest(tokens: &[skein_json::Token]) -> Result<Option<Wall>, DecodeError> {
    let Some(value) = json::optional_at(tokens, json::field(tokens, b"earliest_refresh_at")?)? else {
        return Ok(None);
    };
    let Ok(seconds) = json::unsigned(value) else {
        return Ok(None);
    };
    match seconds.checked_mul(1_000_000_000) {
        Some(nanos) => Ok(Some(Wall::from_nanos(nanos))),
        None => Ok(None),
    }
}
