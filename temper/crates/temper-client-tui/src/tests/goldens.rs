//! The TUI's screens and paints (temper/docs/design/client/tui/pages/README.md and temper/docs/design/client/tui/theme.md;
//! skein/docs/design/ui/screen.md, sections 4 and 5).
//!
//! Frames pin the view trees and theme contracts; `SKEIN_CODEGEN_WRITE=1` rewrites them.

use super::fixtures as f;
use crate::{Refusal, Scheme, Told, Tui, theme};
use skein_lib::{List, Wall};
use skein_ui_screen::{Focus, Frame, Size, render};

// The two review sizes of the TUI's first screen.
const FULL: Size = Size { columns: 80, rows: 24 };

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden test embeds its fixed inputs and formats readable mismatches")]
fn the_tuis_screens_are_their_goldens() {
    let signed_out = f::signed_out();
    let mut signing_in = Tui::new(f::ADDRESS);
    signing_in.step(Told::Typed { value: Box::from(*b"http://127.0.0.1:9000/ji") });
    signing_in.step(Told::Refused { refusal: Refusal::OtherAddress { address: Box::from(*b"127.0.0.1:9000") } });

    let approval = super::approval::asked(Some(temper_client_domain::asked::State::Asked {
        code: super::approval::code(),
        expires: Wall::from_nanos(600_000_000_000),
    }));
    let mut opened = Tui::new(f::ADDRESS);
    opened.step(Told::Browse);
    opened.step(Told::Opened { opened: true });
    let mut no_browser = Tui::new(f::ADDRESS);
    no_browser.step(Told::Browse);
    no_browser.step(Told::Opened { opened: false });

    let rows = [
        (
            "approval-80x24-dark",
            &approval,
            &opened,
            Scheme::Dark,
            FULL,
            include_bytes!("golden/approval-80x24-dark.txt").as_slice(),
        ),
        (
            "approval-80x24-light",
            &approval,
            &opened,
            Scheme::Light,
            FULL,
            include_bytes!("golden/approval-80x24-light.txt").as_slice(),
        ),
        (
            "approval-no-browser-80x24-dark",
            &approval,
            &no_browser,
            Scheme::Dark,
            FULL,
            include_bytes!("golden/approval-no-browser-80x24-dark.txt").as_slice(),
        ),
        (
            "sign-in-80x24-dark",
            &signed_out,
            &signing_in,
            Scheme::Dark,
            FULL,
            include_bytes!("golden/sign-in-80x24-dark.txt").as_slice(),
        ),
    ];
    let mut views = List::with_capacity(8);
    for (_, root, tui, _, _, _) in &rows {
        let mut view = f::view();
        f::build(&mut view, tui, root);
        views.push(view).unwrap();
    }
    let mut checks = List::with_capacity(8);
    for ((name, _, _, scheme, size, want), view) in rows.iter().zip(views.iter()) {
        checks
            .push(Golden {
                name,
                tree: view.tree(),
                scheme: *scheme,
                background: Some(theme::palette(*scheme)[0].1),
                size: *size,
                focus: Focus::default(),
                want,
            })
            .unwrap();
    }
    check(checks.as_slice());
}

#[derive(Debug)]
pub(crate) struct Golden<'a> {
    pub name: &'a str,
    pub tree: &'a skein_ui::Tree,
    pub scheme: Scheme,
    pub size: Size,
    pub background: Option<skein_ui_screen::Rgb>,
    pub focus: Focus,
    pub want: &'a [u8],
}

#[expect(clippy::disallowed_macros, reason = "golden diagnostics format readable mismatches")]
#[expect(clippy::disallowed_types, reason = "test diagnostics collect every mismatching frame as text")]
#[expect(clippy::disallowed_methods, reason = "test diagnostics join the mismatching frames into one failure")]
pub(crate) fn check(rows: &[Golden<'_>]) {
    let mut mismatches = Vec::new();
    for Golden { name, tree, scheme, background, size, focus, want } in rows {
        let mut focus = *focus;
        let mut frame = Frame::new(*size);
        render(tree, &theme(*scheme, *background), *size, &mut focus, &mut frame);
        let printed = frame.print();
        if temper_test_env::rewrite() {
            let path = skein_scratch::paths::tree()
                .join("crates/temper-client-tui/src/tests/golden")
                .join(format!("{name}.txt"));
            std::fs::write(path, &printed).expect("write the TUI golden");
            continue;
        }
        if printed.as_ref() != *want {
            mismatches.push(format!(
                "{name}\nexpected:\n{}\nrendered:\n{}",
                String::from_utf8_lossy(want),
                String::from_utf8_lossy(&printed),
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
