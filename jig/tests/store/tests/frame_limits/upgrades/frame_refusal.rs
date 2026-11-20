use jig_protocol_store::sizes;
use jig_store_world::root;
use jig_store_world::root::check_commits as checked;

#[test]
fn a_page_below_the_store_rows_fails_the_size_check() {
    let mut limits = root::limits();
    let bound = checked(&limits).expect("fixture limits pass the shell's check");
    let floor = bound.key + bound.value;
    eprintln!("store row floor: {floor}, key {}, value {}", bound.key, bound.value);
    limits.store.page.bytes = floor - 1;
    let result = checked(&limits);
    assert!(
        matches!(result, Err(sizes::SizeRefusal::Page { needs, has }) if needs == floor && has == floor - 1),
        "expected row floor {floor} refusal: {result:?}"
    );
    limits.store.page.bytes = floor;
    checked(&limits).expect("page exactly at row floor passes");
}
