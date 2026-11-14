//! Connection and exchange admission (protocol/client/README.md, section 10 and protocol/client/openid.md).
use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub public: PublicLimits,
    pub streams: u32,
    pub watches_per_stream: u32,
    pub queued_bytes: u32,
    pub identity_bytes: u32,
    pub families: u32,
    pub providers: u32,
    pub exchanges: u32,
    pub mints: u32,
    /// Maximum volatile authorization codes.
    pub provider_codes: u32,
    /// Maximum volatile provider access tokens.
    pub provider_tokens: u32,
    pub exchanges_per_client: u32,
    /// A message's bytes, owned by the component (protocol/client/wire.md, 2.2, and protocol/client/README.md, 10).
    pub message_bytes: u32,
    pub wire: jig_client_protocol_wire::v22::Limits,
    pub link_span: Duration,
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    if limits.wire.signed_in_token_some < 40
        || limits.wire.made_service_credential < 40
        || limits.wire.link_sign_in_secret < 32
        || limits.wire.approval_asked_secret < 32
        || limits.wire.approval_sign_in_secret < 32
        || limits.identity_bytes < 8
        || limits.streams == 0
        || limits.watches_per_stream == 0
        || limits.queued_bytes < 128
        || limits.provider_codes == 0
        || limits.provider_tokens == 0
        || limits.mints == 0
        || limits.message_bytes == 0
        || limits.exchanges == 0
        || limits.exchanges_per_client == 0
        || limits.link_span == Duration::ZERO
    {
        return false;
    }
    jig_client_protocol_wire::v22::check_limits(&limits.wire).is_ok()
        && jig_client_protocol_wire::v22::KeyedAnswer::encoded_bound(&limits.wire).is_some()
        && jig_client_protocol_wire::v22::ReadAnswer::encoded_bound(&limits.wire).is_some()
        && jig_client_protocol_wire::v22::AuthorizeAnswer::encoded_bound(&limits.wire).is_some()
        && limits.exchanges.checked_add(limits.exchanges).is_some()
        && provider_bounds(&provider_limits(limits)).is_some()
}

// The issuer has no validity API at this pin. Its document encoder uses u32
// lengths, and form parsing derives one slot beyond the request's byte cap.
fn provider_bounds(limits: &skein_oauth_issuer::Limits) -> Option<u32> {
    let claims = u64::from(limits.claim_bytes)
        .checked_mul(4)?
        .checked_add(u64::from(limits.uri_bytes))?
        .checked_add(u64::from(limits.client_bytes))?
        .checked_add(u64::from(limits.state_bytes))?
        .checked_mul(6)?
        .checked_add(1024)?;
    let jwt = claims.checked_add(2)?.checked_div(3)?.checked_mul(4)?.checked_add(1024)?;
    let answer = jwt.checked_add(u64::from(limits.token_bytes).checked_mul(6)?)?.checked_add(128)?;
    u32::try_from(answer).ok()?;
    u32::try_from(u64::from(limits.uri_bytes).checked_mul(30)?.checked_add(2048)?).ok()?;
    limits.request_bytes.checked_add(1)
}

/// Issuer bounds shared with configuration's startup helper (protocol/client/openid.md).
#[must_use]
pub fn provider_limits(limits: &Limits) -> skein_oauth_issuer::Limits {
    skein_oauth_issuer::Limits {
        clients: 256,
        uri_bytes: 2048,
        client_bytes: 255,
        request_bytes: limits.message_bytes.min(limits.wire.authorize_query),
        state_bytes: 1024,
        token_bytes: 43,
        claim_bytes: limits.identity_bytes.saturating_add(6),
    }
}

/// Public reading's admission (jig/docs/design/domain/people/public-reading.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PublicLimits {
    pub reads: u32,
    pub rate: u32,
    pub rate_burst: u32,
    pub bytes: u64,
    pub bytes_burst: u64,
    pub answer_bytes: u32,
    pub max_age: u32,
}
impl PublicLimits {
    pub const OFF: Self =
        Self { reads: 0, rate: 0, rate_burst: 0, bytes: 0, bytes_burst: 0, answer_bytes: 0, max_age: 0 };

    #[must_use]
    pub fn on(&self) -> bool {
        self.rate > 0
    }
}

/// Whole first deliveries for every watch beside the in-flight hello, including stream-event tags
/// (jig/docs/design/protocol/client/README.md, 10).
#[must_use]
pub fn stream_queue_bytes(watches: u32, wire: &jig_client_protocol_wire::v22::Limits) -> Option<u32> {
    let delivery = jig_client_protocol_wire::v22::Delivery::encoded_bound(wire)?.checked_add(1)?;
    let hello = jig_client_protocol_wire::v22::Hello::encoded_bound(wire)?.checked_add(1)?;
    u32::try_from(delivery.checked_mul(u64::from(watches))?.checked_add(hello)?).ok()
}
