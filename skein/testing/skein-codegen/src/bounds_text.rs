//! The bounds file for a compiled codec (skein/docs/design/data/codec/golden.md, 1).

use core::fmt::Write;
use skein_codec::bounds::{self, Bounds, Declaration};

/// One line per declaration in table order, then the family's maxima.
/// Each line gives encoded and decoded bytes at the ceilings and at clamp 3.
/// Panics if the generated shapes or limits cannot be bounded.
#[must_use]
#[expect(clippy::panic, reason = "a bounds file must name a declaration whose generated shape cannot be bounded")]
pub fn bounds_text(declarations: &[Declaration], ceilings: &[u32]) -> String {
    let small: Box<[u32]> = ceilings.iter().map(|&ceiling| ceiling.min(3)).collect();
    let mut text = String::new();
    for declaration in declarations {
        let name = declaration.name;
        let bound = |limits| {
            bounds::declaration(declarations, limits, ceilings, name)
                .unwrap_or_else(|| panic!("cannot bound declaration {name}"))
        };
        line(&mut text, name, bound(ceilings), bound(&small));
    }
    let family = |limits| bounds::family(declarations, limits, ceilings).expect("cannot bound family");
    line(&mut text, "family", family(ceilings), family(&small));
    text
}

fn line(text: &mut String, name: &str, ceilings: Bounds, small: Bounds) {
    writeln!(
        text,
        "{name}: ceilings encoded={} decoded={}; at_most(3) encoded={} decoded={}",
        ceilings.encoded, ceilings.decoded, small.encoded, small.decoded
    )
    .expect("String accepts formatting");
}

#[cfg(test)]
mod tests {
    use super::{Declaration, bounds_text};
    use skein_codec::bounds::Node;

    #[test]
    fn bounds_text_lists_shapes_then_family_at_both_limits() {
        const SHAPES: &[Declaration] = &[
            Declaration { name: "Item", nodes: &[Node::Bytes { limit: 0 }] },
            Declaration {
                name: "Root",
                nodes: &[Node::Named("Item"), Node::List { limit: size_of::<u32>(), item: 0, size: 16 }],
            },
            Declaration { name: "Wide", nodes: &[Node::Scalar(64)] },
        ];
        assert_eq!(
            bounds_text(SHAPES, &[8, 2]),
            "Item: ceilings encoded=12 decoded=8; at_most(3) encoded=7 decoded=3\n\
             Root: ceilings encoded=28 decoded=48; at_most(3) encoded=18 decoded=38\n\
             Wide: ceilings encoded=64 decoded=0; at_most(3) encoded=64 decoded=0\n\
             family: ceilings encoded=64 decoded=48; at_most(3) encoded=64 decoded=38\n"
        );
    }
}
