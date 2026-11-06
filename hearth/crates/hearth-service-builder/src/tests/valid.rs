use crate::{Configuration, Roots, Service, domain::vocabulary::Call, limits, protocol::Form};
use alloc::boxed::Box;
use skein_io::kernel::Fd;
use skein_lib::bytes;

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

#[test]
#[should_panic(expected = "representable limits")]
fn startup_refuses_unrepresentable_limits() {
    let mut limits = limits::SHIPPED;
    limits.domain.render.made = u64::MAX;
    let _service = Service::new(
        &Configuration { proxy: None, store: bytes::copy_of(b"/store"), limits, uid: super::fixtures::UID },
        Roots { store: Fd::new(3), cache: Fd::new(4), resolutions: Fd::new(5), staging: Fd::new(6) },
        Call::Render {
            resolve: false,
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD"),
            targets: Box::new([]),
        },
        Form::Text,
        super::fixtures::SERVICE_SEED,
    );
}

#[test]
#[should_panic(expected = "a watermark that admits work")]
fn startup_refuses_a_watermark_of_nothing() {
    let mut limits = limits::SHIPPED;
    limits.watermark = 0;
    let _service = Service::new(
        &Configuration { proxy: None, store: bytes::copy_of(b"/store"), limits, uid: super::fixtures::UID },
        Roots { store: Fd::new(3), cache: Fd::new(4), resolutions: Fd::new(5), staging: Fd::new(6) },
        Call::Render {
            resolve: false,
            repository: bytes::copy_of(b"/repo"),
            revision: bytes::copy_of(b"HEAD"),
            targets: Box::new([]),
        },
        Form::Text,
        super::fixtures::SERVICE_SEED,
    );
}
