//! The TUI's checked bounds (temper/docs/design/client/README.md, 6.16).
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
    let view = temper_client_tui::limits(&native.client, PASTE).expect("TUI view capacities are checked");
    Limits { native, view, most: MOST, paste: PASTE, interval: INTERVAL }
}

#[must_use]
pub fn valid(limits: &Limits) -> bool {
    let Some(required) = temper_client_tui::limits(&limits.native.client, limits.paste) else { return false };
    let Some((_, words)) = native::view(&limits.native) else { return false };
    native::valid(&limits.native)
        && temper_client_tui::words::Words::english(words, &limits.native.client).is_ok()
        && skein_ui::valid(&limits.view)
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
        .is_some()
}

#[must_use]
pub fn terminal(
    limits: &Limits,
    depth: skein_ui_screen_terminal::Depth,
    scheme: bool,
) -> skein_ui_screen_terminal::Settings {
    skein_ui_screen_terminal::Settings {
        interval: limits.interval,
        depth,
        most: limits.most,
        paste: limits.paste,
        background: if scheme { None } else { Some(BACKGROUND) },
    }
}
