use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod confirmations;
mod host;
mod memory;
mod placeholder;
mod switcher;

#[test]
fn stylesheet_assets_follow_the_same_origin_policy() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for mut css in [include_str!("../../temper.css"), include_str!("../../task-pages.css")] {
        while let Some((_, reference)) = css.split_once("url(") {
            let (url, rest) = if let Some(quoted) = reference.strip_prefix('"') {
                quoted.split_once("\")").expect("quoted asset URL ends")
            } else if let Some(quoted) = reference.strip_prefix('\'') {
                quoted.split_once("')").expect("quoted asset URL ends")
            } else {
                reference.split_once(')').expect("asset URL ends")
            };
            css = rest;
            assert!(url.starts_with("files/") && !url.contains(".."), "asset must be served from self: {url}");
            assert!(root.join(url).is_file(), "stylesheet asset exists: {url}");
        }
    }
}

#[test]
fn draft_section_headings_keep_the_document_line_height() {
    let css = include_str!("../../temper.css");
    let (_, rule) = css.split_once("#app > .temper-session .temper-doc-inner h3 {").expect("draft section heading");
    let (rule, _) = rule.split_once('}').expect("heading declarations");
    assert!(rule.contains("font: 600 17px/1.7 var(--gf-sans);"), "heading rhythm: {rule}");
}

#[test]
fn the_boards_landings_link_reads_as_a_link() {
    let css = include_str!("../../temper.css");
    let (_, rule) = css.split_once("#app > .temper-public-page h2 {").expect("public section heading");
    let (rule, _) = rule.split_once('}').expect("heading declarations");
    assert!(rule.contains("color: var(--gf-ink);"), "heading ink: {rule}");
    assert!(!rule.contains("uppercase"), "heading words: {rule}");
    let (_, rule) = css.split_once("#app > .temper-public-page h2 > a {").expect("public section link");
    let (rule, _) = rule.split_once('}').expect("link declarations");
    assert!(rule.contains("text-decoration: underline;"), "link underline: {rule}");
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

#[test]
fn settings_controls_keep_the_connect_flows_rules() {
    let css = include_str!("../../temper.css");
    for (selector, declarations) in [
        (
            "#app#app .temper-settings select {",
            &[
                "appearance: none;",
                "padding-right: 36px;",
                "background-image: url(files/chevron.svg);",
                "background-repeat: no-repeat;",
                "background-position: right 14px center;",
                "background-size: 12px;",
            ][..],
        ),
        (".temper-settings [role=radiogroup]:has(> input):not(.temper-connect-sources) {", &["display: grid;"][..]),
        (".temper-settings textarea.temper-connect-addresses {", &["min-height: 107px;"][..]),
        (".temper-connect-list > li.temper-connect-none {", &["padding: 14px 0;"][..]),
        (
            ".temper-connect-sheet p.temper-connect-title:has(+ .temper-settings-text) {",
            &["margin: 0 0 4px;", "font: 600 14px/1.55 var(--gf-sans);"][..],
        ),
        (".temper-connect-row > div > span:not(:has(> strong)) {", &["font-size: 13.5px;"][..]),
    ] {
        let (_, rule) = css.split_once(selector).unwrap_or_else(|| panic!("missing selector: {selector}"));
        let (rule, _) = rule.split_once('}').expect("control declarations");
        for declaration in declarations {
            assert!(rule.contains(declaration), "{selector}: {declaration}: {rule}");
        }
    }
    assert_eq!(css.matches("#app#app").count(), 1, "only the select rule needs two ids");
    assert!(!css.contains(".temper-connect-row span:not("), "row descriptions exclude action labels");
}

#[test]
fn the_approval_code_reads_at_a_glance() {
    let css = include_str!("../../temper.css");
    let (before, approval) = css.split_once("/* ==== tui: approval ==== */").expect("approval block");
    assert!(
        before.trim_end().ends_with(".temper-sign-in > .temper-card { padding: 23.33px 21px 21px; } }"),
        "approval follows the sign-in rules"
    );
    let (_, rule) = approval.split_once(".temper-approve-code {").expect("approval code rule");
    let (rule, _) = rule.split_once('}').expect("code declarations");
    assert!(rule.contains("font: 28px var(--temper-mono);"), "large mono code: {rule}");
    assert!(rule.contains("letter-spacing: 0.12em;"), "spaced code: {rule}");
    assert!(rule.contains("color: var(--temper-ink);"), "code ink: {rule}");
}
