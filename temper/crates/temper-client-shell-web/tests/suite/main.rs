use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod confirmations;
mod host;
mod memory;
mod placeholder;
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

mod welcome_copy;

#[test]
fn phone_latest_takes_its_own_row_below_each_log() {
    let css = include_str!("../../temper.css");
    let (_, phone) = css.split_once("/* ==== ui: phone ==== */").expect("phone area");
    let (_, phone) = phone.split_once("@media (max-width: 760px) {").expect("phone width");
    let (phone, _) = phone.split_once("\n}").expect("phone block");
    for selector in
        ["#app > .temper-session .temper-thread > .temper-latest", ".temper-agent-page .temper-thread > .temper-latest"]
    {
        let (_, rule) = phone.split_once(selector).expect("phone Latest selector");
        let (rule, _) = rule.split_once('}').expect("Latest declarations");
        for declaration in
            ["position: static;", "transform: none;", "margin: 0 auto 12px;", "align-self: center;", "flex: none;"]
        {
            assert!(rule.contains(declaration), "{selector}: {declaration}: {rule}");
        }
        assert!(!rule.contains("margin: -") && !rule.contains("calc("), "{selector}: {rule}");
    }
    for selector in ["#app > .temper-session .temper-thread:has(> [role=log]) {", ".temper-agent-page .temper-thread {"]
    {
        let (_, rule) = css.split_once(selector).expect("thread selector");
        let (rule, _) = rule.split_once('}').expect("thread declarations");
        assert!(rule.contains("display: flex;") && rule.contains("flex-direction: column;"), "{selector}: {rule}");
    }
}
