use super::{BOTTOM, LIMITS, SIDE, TOP, binding, button, finish, leaf};
use skein_ui::{Builder, Role, State, Tree};

/// The 60-by-16 golden leaves twelve body rows: both the log and draft exceed them.
pub fn side(shown: bool) -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    b.open(Role::Banner);
    b.region(TOP);
    leaf(&mut b, Role::Heading { level: 1 }, b"Session");
    leaf(&mut b, Role::Paragraph, b"Chat and draft");
    b.close();
    b.open(Role::Log);
    b.end_request(1);
    for text in [
        b"Entry 01: a conversation line wraps beside the draft.",
        b"Entry 02: a conversation line wraps beside the draft.",
        b"Entry 03: a conversation line wraps beside the draft.",
        b"Entry 04: a conversation line wraps beside the draft.",
        b"Entry 05: a conversation line wraps beside the draft.",
        b"Entry 06: a conversation line wraps beside the draft.",
        b"Entry 07: a conversation line wraps beside the draft.",
        b"Entry 08: a conversation line wraps beside the draft.",
        b"Entry 09: a conversation line wraps beside the draft.",
        b"Entry 10: a conversation line wraps beside the draft.",
        b"Entry 11: a conversation line wraps beside the draft.",
        b"Entry 12: a conversation line wraps beside the draft.",
        b"Entry 13: a conversation line wraps beside the draft.",
        b"Entry 14: a conversation line wraps beside the draft.",
        b"Entry 15: a conversation line wraps beside the draft.",
        b"Entry 16: a conversation line wraps beside the draft.",
    ] {
        leaf(&mut b, Role::Paragraph, text);
    }
    b.close();
    if shown {
        b.open(Role::Complementary);
        b.region(SIDE);
        b.open(Role::TextField);
        b.name(b"Draft");
        b.value(b"notes", 0);
        b.bind(binding());
        b.close();
        leaf(&mut b, Role::Heading { level: 1 }, b"away-mode.md");
        leaf(&mut b, Role::Paragraph, b"A person's needs wait while they are away.");
        // Neither this newer request nor its continuing log may affect the body.
        b.open(Role::Log);
        b.end_request(99);
        b.state(State::Continues);
        for text in [
            b"Draft row 01",
            b"Draft row 02",
            b"Draft row 03",
            b"Draft row 04",
            b"Draft row 05",
            b"Draft row 06",
            b"Draft row 07",
            b"Draft row 08",
            b"Draft row 09",
            b"Draft row 10",
            b"Draft row 11",
            b"Draft row 12",
            b"Draft row 13",
            b"Draft row 14",
            b"Draft row 15",
            b"Draft row 16",
        ] {
            leaf(&mut b, Role::Paragraph, text);
        }
        b.close();
        button(&mut b, b"Later");
        b.close();
    }
    b.open(Role::ContentInfo);
    b.region(BOTTOM);
    leaf(&mut b, Role::Status, b"Ready");
    b.close();
    b.close();
    finish(b)
}

/// Short body and side content expose rounding, unused rows and preorder selection.
pub fn side_regions(regions: &[skein_ui::Region]) -> Tree {
    let mut b = Builder::new(&LIMITS);
    b.open(Role::Page);
    leaf(&mut b, Role::Paragraph, b"Body");
    for (region, text) in regions.iter().zip([b"First".as_slice(), b"Second"]) {
        b.open(Role::Complementary);
        b.region(*region);
        leaf(&mut b, Role::Paragraph, text);
        b.close();
    }
    b.close();
    finish(b)
}
