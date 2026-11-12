use skein_world::domain::heap::Counting;

#[global_allocator]
static HEAP: Counting = Counting;

mod confirmations;
mod host;
mod memory;
mod placeholder;
mod switcher;

#[test]
fn no_source_of_the_bundle_or_the_servers_pages_names_the_old_mark() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root).expect("bundle source directory") {
        let path = entry.expect("bundle source entry").path();
        if path.is_file() {
            files.push(path);
        }
    }
    let mut directories = vec![
        root.join("files"),
        root.join("src"),
        root.join("../temper-client-web-page/src"),
        root.join("../temper-playground-page/playground"),
        root.join("../temper-playground-page/src"),
    ];
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory).expect("bundle source directory") {
            let path = entry.expect("bundle source entry").path();
            if path.is_dir() {
                if path != root.join("files/fonts") {
                    directories.push(path);
                }
            } else {
                files.push(path);
            }
        }
    }
    for page in ["src/clients/google/pages.rs", "src/setup/routes.rs", "src/clients/no-bundle.html"] {
        files.push(root.join("../temper-service").join(page));
    }
    files.sort();
    for path in files {
        let source = std::fs::read_to_string(&path).expect("bundle or server page source is text");
        assert!(!source.to_ascii_lowercase().contains("#b8664e"), "old mark in {}", path.display());
    }
}

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

#[test]
fn phone_home_size_is_muted_without_chip_ground_or_files() {
    let css = include_str!("../../temper.css");
    let phone = css
        .split("@media (max-width: 760px) {")
        .skip(1)
        .map(|block| block.split("\n}").next().unwrap())
        .find(|block| block.contains("#app > .temper-home .temper-size-files"))
        .expect("Home's phone size rule");
    let (_, rule) = phone.split_once("#app > .temper-home .temper-chips .temper-chip {").expect("phone chip");
    let (rule, _) = rule.split_once('}').unwrap();
    for declaration in ["padding: 0;", "border: 0;", "background: none;", "font-size: 13px;", "color: var(--gf-muted);"]
    {
        assert!(rule.contains(declaration), "phone size: {declaration}: {rule}");
    }
    assert!(phone.contains("#app > .temper-home .temper-size-files { display: none; }"));
    assert!(!phone.contains(".temper-needs"), "Needs you keeps its chip and files");
    let (_, chip) = css.split_once(".temper-needs .temper-chip {").unwrap();
    let (chip, _) = chip.split_once('}').unwrap();
    assert!(chip.contains("border: 1px solid var(--gf-line);") && chip.contains("background: var(--gf-surface);"));
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
    let (_, outside) = css.split_once("/* ==== ui: outside ==== */").expect("outside block");
    let (_, rule) = outside.split_once(".temper-outside .temper-approve-code {").expect("approval code rule");
    let (rule, _) = rule.split_once('}').expect("code declarations");
    assert!(rule.contains("font: 500 30px/1.25 var(--gf-mono);"), "large mono code: {rule}");
    assert!(rule.contains("letter-spacing: .14em;"), "spaced code: {rule}");
    assert!(rule.contains("color: var(--gf-ink);"), "code ink: {rule}");
}
