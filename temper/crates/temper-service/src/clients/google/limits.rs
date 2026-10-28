//! Per-client and transport bounds (protocol/sign-in.md, 6).
use crate::limits::SignIn;
use skein_lib::Duration;
pub(super) fn client(limits: SignIn) -> skein_oauth::ClientLimits {
    skein_oauth::ClientLimits {
        document: skein_oauth::Limits {
            document_bytes: limits.body_bytes,
            string_bytes: 16_384,
            token_bytes: 8192,
            client_bytes: limits.secret_bytes.max(256),
            issued_client_bytes: 256,
            granted_bytes: 256,
            subject_bytes: 255,
            id_token_bytes: 16_384,
            detail_bytes: 256,
            record_bytes: 65_536,
            depth: 16,
            tokens: 2048,
        },
        uri_bytes: limits.url_bytes,
        scope_bytes: 64,
        state_bytes: 64,
        code_bytes: 4096,
        url_bytes: limits.url_bytes,
        request_bytes: 16_384,
        parameter_bytes: 256,
        resource_bytes: 256,
        nonce_bytes: 64,
        metadata_bytes: limits.body_bytes,
        key_set_bytes: limits.body_bytes,
        keys: 8,
        modulus_bits: 8192,
        skew: Duration::ZERO,
        sign_in_time: limits.deadline,
        request_time: limits.deadline,
        backoff_base: Duration::from_secs(1),
        backoff_ceiling: Duration::from_secs(1),
        max_attempts: 1,
    }
}
pub(super) fn fetch(limits: SignIn) -> super::super::fetch::Limits {
    super::super::fetch::Limits {
        body: limits.body_bytes,
        http: skein_http::client::Limits { request: 8192, head: 8192, headers: 32, read: 16_384, send: 16_384 },
        tls: skein_tls::client::Limits { read: 16_384, send: 16_384, records: 131_072 },
    }
}
