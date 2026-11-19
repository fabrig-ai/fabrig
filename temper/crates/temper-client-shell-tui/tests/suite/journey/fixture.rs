//! Journey fixtures for this binary (temper/docs/design/client/tui/journeys.md, 7).
//! Library tests and the suite keep separate copies of their own types (tools/crates/design.md, 3.2).
use skein_lib::{Time, Wall};
use skein_ui_screen::Size;
use temper_client_shell_tui::journey::{Facts, Line};
use temper_client_tui::Scheme;

#[must_use]
pub(super) fn facts() -> Facts {
    Facts {
        set_up: false,
        box_log: Box::default(),
        seed: 7,
        now: Time::ZERO,
        wall: Wall::EPOCH,
        size: Size { rows: 12, columns: 48 },
        scheme: Some(Scheme::Dark),
        background: None,
        plain: false,
        version: Box::default(),
        boxed: false,
        address: temper_client_tui::Origin::Http("127.0.0.1:8367".parse().expect("fixture socket")),
        front: false,
        local: false,
        note: None,
        kept: Vec::new(),
        credential: None,
    }
}

pub(super) const SETUPS: [(bool, &[u8]); 5] = [
    (false, b""),
    (true, b""),
    (false, b"/data/box/logs/box.log"),
    (true, b"/data/box/logs/box.log"),
    (true, b"/data/\"\xff/box/logs/box.log"),
];

#[must_use]
pub(super) fn setup(set_up: bool, log: &[u8]) -> Facts {
    let mut given = facts();
    given.front = true;
    given.boxed = true;
    given.set_up = set_up;
    given.box_log = log.into();
    given
}

#[must_use]
pub(super) fn written(records: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for record in records {
        record.write(&mut bytes);
    }
    bytes.into()
}
