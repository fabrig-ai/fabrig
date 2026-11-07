//! The allocator answers null past what a layout can describe, and counts nothing.

#[repr(align(4096))]
struct Page {
    _bytes: [u8; 4096],
}

#[test]
fn a_block_past_what_a_layout_can_describe_is_refused_and_not_counted() {
    let most = usize::try_from(isize::MAX).expect("isize::MAX fits");
    for size in [most, most - 7, most - 8, most - 15] {
        let meter = skein_heap::Meter::new();
        let mut bytes: Vec<u8> = Vec::new();
        assert!(bytes.try_reserve_exact(size).is_err(), "{size} bytes");
        assert_eq!(meter.held(), 0, "{size} bytes");
    }
    for pages in [most / size_of::<Page>(), most / size_of::<Page>() - 1] {
        let meter = skein_heap::Meter::new();
        let mut bytes: Vec<Page> = Vec::new();
        assert!(bytes.try_reserve_exact(pages).is_err(), "{pages} pages");
        assert_eq!(meter.held(), 0, "{pages} pages");
    }
}
