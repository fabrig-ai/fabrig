//! One reload for a refused page (temper/docs/design/protocol/wires.md, 8).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    None,
    Reload,
    Clear,
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn page(host: &crate::host::Host) -> bool {
    // Outside the small store's `temper.` prefix: this is the page's marker, not a client record.
    const KEY: &str = "temper-out-of-date-reload";
    let Some(window) = web_sys::window() else { return false };
    let Ok(Some(storage)) = window.session_storage() else { return false };
    let marked = storage.get_item(KEY).ok().map(|value| value.is_some());
    match host.reload(marked) {
        Action::None => false,
        Action::Clear => {
            let _removed = storage.remove_item(KEY);
            false
        }
        Action::Reload => {
            // Never reload without a marker that survives it, including when storage is full.
            storage.set_item(KEY, "1").is_ok() && window.location().reload().is_ok()
        }
    }
}
