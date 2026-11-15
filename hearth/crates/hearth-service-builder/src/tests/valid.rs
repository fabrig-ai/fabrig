use crate::limits;

#[test]
fn valid_accepts_shipped_limits_and_refuses_unrepresentable_edges() {
    let mut tree_depth = limits::SHIPPED;
    tree_depth.protocol.tree.depth = u32::MAX;
    let mut printer_chunk = limits::SHIPPED;
    printer_chunk.protocol.printer.chunk = 0;
    let mut render_made = limits::SHIPPED;
    render_made.domain.render.made = u64::MAX;
    for (name, limits, expected) in [
        ("shipped", limits::SHIPPED, true),
        ("tree depth", tree_depth, false),
        ("printer chunk", printer_chunk, false),
        ("render made", render_made, false),
    ] {
        assert_eq!(crate::valid(&limits), expected, "{name}");
    }
}
