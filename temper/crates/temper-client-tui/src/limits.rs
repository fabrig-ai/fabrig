//! The two screens' bound (temper/docs/design/client/README.md, 6.14).
use crate::words::{ALL, Word};
use temper_client_domain::Limits as Domain;

#[must_use]
pub fn limits(domain: &Domain, field: u32) -> Option<skein_ui::Limits> {
    // SignInFull is the longest borrowed line: its literals, ten count digits
    // and two thirteen-byte moments fit in 256 bytes. The other borrowed
    // phrases are shorter. Own phrases derive their capacities from the table.
    let mut line = 256_u32;
    for word in ALL {
        line = line.max(word.capacity(domain)?);
    }
    let opened = Word::ApprovalTab
        .capacity(domain)?
        .checked_add(Word::ApprovalCode.capacity(domain)?)?
        .checked_add(Word::ApprovalNoTab.capacity(domain)?)?;
    let no_browser = Word::ApprovalOpen
        .capacity(domain)?
        .checked_add(95)? // The scheme, authority and approval path with its shown code.
        .checked_add(Word::ApprovalCheck.capacity(domain)?)?
        .checked_add(Word::ApprovalTunnel.capacity(domain)?)?;
    let approval = opened.max(no_browser).checked_add(Word::ApprovalExpires.capacity(domain)?)?.max(line);
    let field = field.checked_add(Word::Link.capacity(domain)?)?.checked_add(Word::LinkShape.capacity(domain)?)?;
    let signing = Word::Title
        .capacity(domain)?
        .checked_add(11)? // SignedOutMain, with room for the terminal's period.
        .checked_add(Word::SignIn.capacity(domain)?)?
        .checked_add(approval)?
        .checked_add(Word::Paste.capacity(domain)?)?
        .checked_add(field)?
        .checked_add(line)?
        .checked_add(Word::QuitKeys.capacity(domain)?)?
        .checked_add(Word::State.capacity(domain)?)?;
    let connected = Word::Connected.capacity(domain)?.max(Word::Connecting.capacity(domain)?).max(256);
    let projects = Word::Projects.capacity(domain)?.max(256); // NoProjectYet fits the borrowed-line bound.
    let first = Word::Title
        .capacity(domain)?
        .checked_add(connected)?
        .checked_add(projects)?
        .checked_add(Word::NotKept.capacity(domain)?)?
        .checked_add(Word::Quit.capacity(domain)?)?
        .checked_add(Word::State.capacity(domain)?)?;
    let limits = skein_ui::Limits {
        // Signing in: Page, Main, ten pairs of blocks and text leaves,
        // the field, ContentInfo and its paragraph and status text pairs.
        nodes: 28,
        depth: 4,
        text_per_node: field.max(line).max(connected).max(projects),
        text_bytes: signing.max(first),
        styles_per_node: 1,
        patches: 28_u32.checked_mul(4)?,
        bindings: 1,
        ..skein_ui::Limits::EMPTY
    };
    if skein_ui::valid(&limits) { Some(limits) } else { None }
}
