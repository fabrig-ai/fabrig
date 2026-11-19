//! The TUI's checked bounds (temper/docs/design/client/tui/README.md, 4).
use skein_lib::Duration;
use skein_ui_screen::Size;
use temper_client_shell_terminal::limits as native;

pub const MOST: Size = Size { rows: 200, columns: 500 };
pub const PASTE: u32 = 512;
pub const BACKGROUND: Duration = Duration::from_millis(200);
pub const INTERVAL: Duration = Duration::from_millis(20);

#[derive(Clone, Debug)]
pub struct Limits {
    pub native: native::Limits,
    pub view: skein_ui::Limits,
    pub most: Size,
    pub paste: u32,
    pub interval: Duration,
}

#[must_use]
pub fn tui() -> Limits {
    let native = native::native();
    let tree = native::view(&native).expect("native view capacities are checked").0.tree;
    stock(&native, &tree)
}

fn stock(native: &native::Limits, tree: &skein_ui::Limits) -> Limits {
    let view = temper_client_tui::limits(&native.client, tree, PASTE).expect("TUI view capacities are checked");
    Limits { native: *native, view, most: MOST, paste: PASTE, interval: INTERVAL }
}

/// Stock bounds and words pass the same checks as configured bounds, without rebuilding the words.
pub(crate) fn checked_tui() -> (Limits, temper_client_tui::words::Words) {
    let native = native::native();
    let (view, words) = native::checked(&native).expect("native capacities are checked");
    let limits = stock(&native, &view.tree);
    let words = checked_words(&limits, &limits.view, words).expect("TUI capacities are checked");
    (limits, words)
}

#[must_use]
pub fn valid(limits: &Limits) -> bool {
    checked(limits).is_some()
}
/// Validate once and keep the words for the host's view.
pub(crate) fn checked(limits: &Limits) -> Option<temper_client_tui::words::Words> {
    let (view, words) = native::checked(&limits.native)?;
    let required = temper_client_tui::limits(&limits.native.client, &view.tree, limits.paste)?;
    checked_words(limits, &required, words)
}

fn checked_words(
    limits: &Limits,
    required: &skein_ui::Limits,
    words: temper_client_view::words::Words,
) -> Option<temper_client_tui::words::Words> {
    let usable = skein_ui::valid(&limits.view)
        && limits.view.nodes >= required.nodes
        && limits.view.depth >= required.depth
        && limits.view.text_per_node >= required.text_per_node
        && limits.view.text_bytes >= required.text_bytes
        && limits.view.styles_per_node >= required.styles_per_node
        && limits.view.patches >= required.patches
        && limits.view.bindings >= required.bindings
        && limits.most.rows > 0
        && limits.most.columns > 0
        && limits.paste > 0
        && limits.interval > Duration::ZERO
        && skein_ui_screen_terminal::Terminal::operations(&terminal(
            limits,
            skein_ui_screen_terminal::Depth::True,
            true,
        ))
        .is_some();
    if !usable {
        return None;
    }
    temper_client_tui::words::Words::english(words, &limits.native.client).ok()
}

#[must_use]
pub fn terminal(
    limits: &Limits,
    depth: skein_ui_screen_terminal::Depth,
    _scheme: bool,
) -> skein_ui_screen_terminal::Settings {
    skein_ui_screen_terminal::Settings {
        interval: limits.interval,
        depth,
        most: limits.most,
        paste: limits.paste,
        background: Some(BACKGROUND),
    }
}
