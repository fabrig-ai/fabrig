//! The pages and forge share jig's receiving and request bounds.
use crate::forge;
use temper_client_protocol_wire::v1 as w;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub forge: forge::Limits,
    pub client: jig_client::Limits,
    /// The uses one account row names before it says how many more. jig's
    /// `AccountCounts` bounds what the client receives; what a row draws is temper's.
    pub row_uses: u32,
    /// A conversation's window narrower than `document_bytes` (client/pages/chat/conversation.md, 2).
    /// `None` everywhere but the browser tier's bundle.
    pub window_bytes: Option<u64>,
}
impl Limits {
    /// Parts in one text, including its unsplit remainder (client/pages/chat/conversation.md, 3).
    #[must_use]
    pub const fn view_parts() -> u32 {
        16
    }
    /// Unique views retained by a chat (client/pages/chat/conversation.md, 3).
    #[must_use]
    pub const fn chat_views() -> u32 {
        32
    }
}
/// Refuses unusable client limits, unrepresentable page counts and forge wire bounds.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    if !jig_client::valid(&limits.client) || limits.forge.field_bytes < 7 {
        return false;
    }
    if let Some(window) = limits.window_bytes
        && (window < u64::from(limits.client.domain.long_text) || window > limits.client.domain.document_bytes)
    {
        return false;
    }
    let Some(wire) = forge::wire_limits(&limits.forge) else { return false };
    let Some(lines) = limits.client.domain.rows.checked_mul(4) else { return false };
    let Ok(document) = u32::try_from(limits.client.domain.document_bytes) else { return false };
    lines.checked_add(1).is_some()
        && limits.client.domain.rows.checked_mul(3).is_some()
        && document.checked_add(1).is_some()
        && w::Answer::decoded_bound(&wire).is_some()
        && w::ChangePanel::decoded_bound(&wire).is_some()
        && w::Page::decoded_bound(&wire).is_some()
        && w::Forges::decoded_bound(&wire).is_some()
}
