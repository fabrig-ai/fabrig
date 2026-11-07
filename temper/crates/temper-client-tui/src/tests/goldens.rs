//! The TUI's screens and paints (temper/docs/design/client/README.md, 6.14 and 6.15;
//! skein/docs/design/ui-screen.md, sections 4 and 5).

use super::fixtures as f;
use crate::{Refusal, Scheme, Told, Tui, theme};
use skein_ui_screen::{Focus, Frame, Size, render};

// The two review sizes of the TUI's first screen.
const FULL: Size = Size { columns: 80, rows: 24 };
const SMALL: Size = Size { columns: 40, rows: 12 };

#[test]
fn the_tuis_screens_are_their_goldens() {
    let (first, _) = f::live(Some(f::me(&[b"Hello", b"Demo"])));
    let mut connected = Tui::new(f::ADDRESS);
    connected.step(Told::Link { link: jig_client::Link::Live });

    let signed_out = f::signed_out();
    let mut signing_in = Tui::new(f::ADDRESS);
    signing_in.step(Told::Typed { value: Box::from(*b"http://127.0.0.1:9000/ji") });
    signing_in.step(Told::Refused { refusal: Refusal::OtherAddress { address: Box::from(*b"127.0.0.1:9000") } });

    let rows = [
        (
            "first-80x24-dark",
            &first,
            &connected,
            Scheme::Dark,
            FULL,
            include_bytes!("golden/first-80x24-dark.txt").as_slice(),
        ),
        (
            "first-80x24-light",
            &first,
            &connected,
            Scheme::Light,
            FULL,
            include_bytes!("golden/first-80x24-light.txt").as_slice(),
        ),
        (
            "first-40x12-dark",
            &first,
            &connected,
            Scheme::Dark,
            SMALL,
            include_bytes!("golden/first-40x12-dark.txt").as_slice(),
        ),
        (
            "first-40x12-light",
            &first,
            &connected,
            Scheme::Light,
            SMALL,
            include_bytes!("golden/first-40x12-light.txt").as_slice(),
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
    let mut mismatches = Vec::new();
    for (name, root, tui, scheme, size, expected) in rows {
        let mut view = f::view();
        f::build(&mut view, tui, root);
        let mut focus = Focus::default();
        let mut frame = Frame::new(size);
        render(view.tree(), &theme(scheme), size, &mut focus, &mut frame);
        let printed = frame.print();
        if printed.as_ref() != expected {
            mismatches.push(format!(
                "{name}\nexpected:\n{}\nrendered:\n{}",
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&printed),
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
