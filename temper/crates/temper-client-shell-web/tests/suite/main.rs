use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod host;
mod memory;
mod switcher;

#[test]
fn draft_section_headings_keep_the_document_line_height() {
    let css = include_str!("../../temper.css");
    let (_, rule) = css.split_once("#app > .temper-session .temper-doc-inner h3 {").expect("draft section heading");
    let (rule, _) = rule.split_once('}').expect("heading declarations");
    assert!(rule.contains("font: 600 17px/1.7 var(--gf-sans);"), "heading rhythm: {rule}");
}

#[test]
fn phone_session_hides_the_keyboard_hint() {
    let css = include_str!("../../temper.css");
    assert!(
        css.split("@media (max-width: 760px) {").skip(1).any(|block| {
            let phone = block.split("\n}").next().expect("phone rules");
            phone.contains(".temper-session-hint) { display: none; }")
        }),
        "phone sessions must hide the keyboard hint",
    );
}

#[test]
fn programmatic_page_targets_do_not_draw_control_focus_rings() {
    let css = include_str!("../../temper.css");
    let (_, rule) =
        css.split_once("#app :is(main, h1)[tabindex=\"-1\"]:focus {").expect("page focus targets have their own rule");
    let (rule, _) = rule.split_once('}').expect("focus declarations");
    assert!(rule.contains("outline: none;"), "page focus target: {rule}");
}
