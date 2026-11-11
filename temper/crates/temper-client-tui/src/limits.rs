//! The screens' and chat page's bound (temper/docs/design/client/tui/pages.md and temper/docs/design/client/tui/box.md).
use crate::words::Word;
use temper_client_domain::Limits as Domain;

pub const BOX_LOG: u32 = 50;
pub const LOG_LINE: u32 = 512;
pub const STEP: u32 = 64;
pub const WALK: u32 = 64;
pub const NOTICE: skein_lib::Duration = skein_lib::Duration::from_secs(6);

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
    let forge = temper_client_domain::forge::wire_limits(&domain.forge)?;
    let brief = forge.status_brief_sentence.checked_add(1)?.checked_add(forge.status_brief_lede)?;
    let need = domain
        .client
        .domain
        .text
        .max(domain.forge.field_bytes)
        .checked_add(domain.client.domain.long_text.max(domain.forge.field_bytes))?
        .checked_add(256)?;
    line = line.max(brief).max(need);
    let launchpad = Word::Title
        .capacity(domain)?
        .checked_add(Word::Separator.capacity(domain)?)?
        .checked_add(domain.client.domain.text)?
        .checked_add(64)?
        .checked_add(brief)?
        .checked_add(Word::Byline.capacity(domain)?)?
        .checked_add(Word::NotKept.capacity(domain)?)?
        .checked_add(Word::LeftOff.capacity(domain)?)?
        .checked_add(domain.client.domain.text)?
        .checked_add(256)?
        .checked_add(Word::NeedsYou.capacity(domain)?)?
        .checked_add(need.checked_mul(5)?)?;
    // The TUI owns its form and command menu even with no supplied page bound.
    // The browser page bound covers model, effort, chat and question lists.
    let foot_nodes = 64_u32;
    // Up to eight help notes of eighteen lines, and the TUI's unknown-command note.
    let note_lines = 8_u32.checked_mul(18)?.checked_add(1)?;
    let nodes = foot_nodes.checked_add(64)?.checked_add(note_lines.checked_mul(2)?)?;
    let most = u32::try_from(crate::BoxLine::MOST).ok()?;
    // /log and the stopping note can each draw the retained log.
    let front_text = BOX_LOG.checked_mul(2)?.checked_mul(LOG_LINE)?.checked_add(most)?.checked_add(4096)?;
    let front_nodes = 192_u32.checked_add(BOX_LOG.checked_mul(2)?)?;
    let foot_text = field.checked_add(line.checked_mul(16)?)?;
    let screens = skein_ui::Limits {
        // Signing in: Page, Main, ten pairs of blocks and text leaves,
        // the field, ContentInfo and its paragraph and status text pairs.
        nodes,
        depth: 9,
        text_per_node: field.max(line).max(most),
        text_bytes: signing
            .max(launchpad.checked_add(own)?.checked_add(foot_text)?.checked_add(line.checked_mul(note_lines)?)?),
        styles_per_node: 2,
        patches: nodes.checked_mul(4)?,
        bindings: 14,
        ..skein_ui::Limits::EMPTY
    };
    let limits = skein_ui::Limits {
        nodes: screens.nodes.max(view.nodes).checked_add(front_nodes)?,
        depth: screens.depth.max(view.depth),
        text_per_node: screens.text_per_node.max(view.text_per_node),
        text_bytes: screens.text_bytes.max(view.text_bytes).checked_add(front_text)?,
        drawing_bytes: screens.drawing_bytes.max(view.drawing_bytes),
        styles_per_node: screens.styles_per_node.max(view.styles_per_node),
        patches: screens.patches.max(view.patches).checked_add(front_nodes.checked_mul(4)?)?,
        bindings: screens.bindings.max(view.bindings).checked_add(1)?,
        slots: screens.slots.max(view.slots),
        markdown_depth: screens.markdown_depth.max(view.markdown_depth),
        scene_marks: screens.scene_marks.max(view.scene_marks),
        scene_bytes: screens.scene_bytes.max(view.scene_bytes),
    };
    if skein_ui::valid(&limits) { Some(limits) } else { None }
}
