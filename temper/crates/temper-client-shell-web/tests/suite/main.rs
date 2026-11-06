use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod host;
mod memory;

#[test]
fn draft_section_headings_keep_the_document_line_height() {
    let css = include_str!("../../temper.css");
    let (_, rule) = css.split_once("#app > .temper-session .temper-doc-inner h3 {").expect("draft section heading");
    let (rule, _) = rule.split_once('}').expect("heading declarations");
    assert!(rule.contains("font: 600 17px/1.7 var(--gf-sans);"), "heading rhythm: {rule}");
}
