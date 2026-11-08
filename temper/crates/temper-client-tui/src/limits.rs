//! The screens' and chat page's bound (temper/docs/design/client/README.md, 6.14).
use crate::words::Word;
use temper_client_domain::Limits as Domain;

#[must_use]
pub fn limits(domain: &Domain, view: &skein_ui::Limits, field: u32) -> Option<skein_ui::Limits> {
    // SignInFull is the longest borrowed line: its literals, ten count digits
    // and two thirteen-byte moments fit in 256 bytes. The other borrowed
    // phrases are shorter. Own phrases derive their capacities from the table.
    let mut line = 256_u32;
    let mut own = 0_u32;
    for word in crate::words::ALL {
        let capacity = word.capacity(domain)?;
        line = line.max(capacity);
        own = own.checked_add(capacity)?;
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
        .checked_add(Word::State.capacity(domain)?)?;
    // The TUI owns its form and command menu even with no supplied page bound.
    // The browser page bound covers model, effort, chat and question lists.
    let foot_nodes = 64_u32;
    let foot_text = field.checked_add(line.checked_mul(16)?)?;
    let screens = skein_ui::Limits {
        // Signing in: Page, Main, ten pairs of blocks and text leaves,
        // the field, ContentInfo and its paragraph and status text pairs.
        nodes: foot_nodes,
        depth: 7,
        text_per_node: field.max(line).max(connected).max(projects),
        text_bytes: signing.max(first.checked_add(own)?).max(foot_text),
        styles_per_node: 1,
        patches: foot_nodes.checked_mul(4)?,
        bindings: 14,
        ..skein_ui::Limits::EMPTY
    };
    let limits = skein_ui::Limits {
        nodes: screens.nodes.max(view.nodes),
        depth: screens.depth.max(view.depth),
        text_per_node: screens.text_per_node.max(view.text_per_node),
        text_bytes: screens.text_bytes.max(view.text_bytes),
        drawing_bytes: screens.drawing_bytes.max(view.drawing_bytes),
        styles_per_node: screens.styles_per_node.max(view.styles_per_node),
        patches: screens.patches.max(view.patches),
        bindings: screens.bindings.max(view.bindings),
        slots: screens.slots.max(view.slots),
        markdown_depth: screens.markdown_depth.max(view.markdown_depth),
        scene_marks: screens.scene_marks.max(view.scene_marks),
        scene_bytes: screens.scene_bytes.max(view.scene_bytes),
    };
    if skein_ui::valid(&limits) { Some(limits) } else { None }
}
